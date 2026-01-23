//! Watcher service implementation
//!
//! Monitors a folder for new CSV/JSON files and handles ingestion.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::print_stderr
)]

use anyhow::{Context as _, Result};
use chrono::Local;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter as _};

use super::config::WatcherConfig;
use super::events::{
    FileDetectedPayload, FileReadyPayload, IngestFailedPayload, IngestStartedPayload,
    IngestSucceededPayload, WatcherServiceState, WatcherStatusPayload,
};
use crate::utils;

/// Maximum time to wait for file stability (30 seconds)
const STABILITY_TIMEOUT: Duration = Duration::from_secs(30);

/// Time between stability checks (500ms)
const STABILITY_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// Number of consecutive unchanged checks required for stability
const STABILITY_REQUIRED_CHECKS: u32 = 3;

/// Message types for the watcher service
#[derive(Debug)]
pub enum WatcherMessage {
    /// Start watching a folder
    Start(PathBuf),
    /// Stop watching
    Stop,
    /// Manually trigger ingestion of a specific file
    IngestNow(PathBuf),
}

/// File stability checker
struct StabilityChecker {
    path: PathBuf,
    last_size: u64,
    unchanged_count: u32,
    start_time: Instant,
}

impl StabilityChecker {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            last_size: 0,
            unchanged_count: 0,
            start_time: Instant::now(),
        }
    }

    /// Check if file is stable (size hasn't changed for N checks)
    fn check(&mut self) -> Result<bool> {
        // Check timeout
        if self.start_time.elapsed() > STABILITY_TIMEOUT {
            anyhow::bail!("File stability timeout exceeded");
        }

        // Get current file size
        let metadata = std::fs::metadata(&self.path)
            .with_context(|| format!("Failed to read file metadata: {}", self.path.display()))?;
        let current_size = metadata.len();

        // Check if size changed
        if current_size == self.last_size {
            self.unchanged_count += 1;
        } else {
            self.unchanged_count = 0;
            self.last_size = current_size;
        }

        // File is stable if unchanged for required number of checks
        Ok(self.unchanged_count >= STABILITY_REQUIRED_CHECKS)
    }
}

/// Watcher service
pub struct WatcherService {
    _app: AppHandle,
    config: Arc<Mutex<WatcherConfig>>,
    state: Arc<Mutex<WatcherServiceState>>,
    command_tx: Sender<WatcherMessage>,
}

impl WatcherService {
    /// Create a new watcher service
    pub fn new(app: AppHandle, config: WatcherConfig) -> Result<Self> {
        let (command_tx, command_rx) = channel();

        let service = Self {
            _app: app.clone(),
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(WatcherServiceState::Idle)),
            command_tx,
        };

        // Start the worker thread
        let app_clone = app.clone();
        let config_clone = Arc::clone(&service.config);
        let state_clone = Arc::clone(&service.state);

        std::thread::Builder::new()
            .name("watcher-service".to_owned())
            .spawn(move || {
                Self::worker_thread(app_clone, config_clone, state_clone, command_rx);
            })
            .context("Failed to spawn watcher service thread")?;

        Ok(service)
    }

    /// Worker thread that runs the watcher
    fn worker_thread(
        app: AppHandle,
        config: Arc<Mutex<WatcherConfig>>,
        state: Arc<Mutex<WatcherServiceState>>,
        command_rx: Receiver<WatcherMessage>,
    ) {
        let (file_tx, file_rx) = channel();
        let mut _watcher: Option<Box<dyn Watcher + Send>> = None;

        // Main event loop
        loop {
            // Check for commands
            match command_rx.try_recv() {
                Ok(msg) => match msg {
                    WatcherMessage::Start(folder) => {
                        // Stop existing watcher if any
                        _watcher = None;

                        // Create new watcher
                        match notify::recommended_watcher({
                            let file_tx = file_tx.clone();
                            move |res: Result<Event, notify::Error>| {
                                if let Ok(event) = res {
                                    let _ = file_tx.send(event);
                                }
                            }
                        }) {
                            Ok(mut w) => {
                                if let Err(e) = w.watch(&folder, RecursiveMode::NonRecursive) {
                                    Self::emit_status(
                                        &app,
                                        &state,
                                        Some(format!("Failed to watch folder: {e}")),
                                    );
                                } else {
                                    if let Ok(mut s) = state.lock() {
                                        *s = WatcherServiceState::Watching;
                                    }
                                    _watcher = Some(Box::new(w));
                                    Self::emit_status(&app, &state, None);
                                    utils::log_event(
                                        "Watcher",
                                        &format!("Started watching: {}", folder.display()),
                                    );
                                }
                            }
                            Err(e) => {
                                Self::emit_status(
                                    &app,
                                    &state,
                                    Some(format!("Failed to create watcher: {e}")),
                                );
                            }
                        }
                    }
                    WatcherMessage::Stop => {
                        _watcher = None;
                        if let Ok(mut s) = state.lock() {
                            *s = WatcherServiceState::Idle;
                        }
                        Self::emit_status(&app, &state, None);
                        utils::log_event("Watcher", "Stopped watching");
                    }
                    WatcherMessage::IngestNow(path) => {
                        Self::handle_file_ingestion(&app, &config, &state, path);
                    }
                },
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }

            // Check for file events (non-blocking)
            if let Ok(event) = file_rx.try_recv()
                && let EventKind::Create(_) = event.kind
            {
                for path in event.paths {
                    if Self::is_supported_file(&path) {
                        Self::handle_new_file(&app, &config, &state, path);
                    }
                }
            }

            // Small sleep to prevent busy-waiting
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Check if file extension is supported
    fn is_supported_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "csv" || ext_str == "json"
        } else {
            false
        }
    }

    /// Handle a newly detected file
    fn handle_new_file(
        app: &AppHandle,
        config: &Arc<Mutex<WatcherConfig>>,
        state: &Arc<Mutex<WatcherServiceState>>,
        path: PathBuf,
    ) {
        let file_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Emit file detected event
        let _ = app.emit(
            "watcher:file_detected",
            FileDetectedPayload {
                path: path.display().to_string(),
                file_type: file_type.clone(),
                detected_at: Local::now().to_rfc3339(),
            },
        );

        utils::log_event("Watcher", &format!("Detected file: {}", path.display()));

        // Check if auto-ingest is enabled
        let should_ingest = config.lock().map(|cfg| cfg.auto_ingest).unwrap_or(false);

        if should_ingest {
            Self::handle_file_ingestion(app, config, state, path);
        }
    }

    /// Handle file ingestion with stability check
    fn handle_file_ingestion(
        app: &AppHandle,
        _config: &Arc<Mutex<WatcherConfig>>,
        state: &Arc<Mutex<WatcherServiceState>>,
        path: PathBuf,
    ) {
        let app_clone = app.clone();
        let state_clone = Arc::clone(state);
        let path_clone = path.clone();

        // Spawn ingestion in a separate thread to not block watcher
        std::thread::spawn(move || {
            // Wait for file stability
            let mut checker = StabilityChecker::new(path_clone.clone());
            loop {
                match checker.check() {
                    Ok(true) => {
                        // File is stable
                        let _ = app_clone.emit(
                            "watcher:file_ready",
                            FileReadyPayload {
                                path: path_clone.display().to_string(),
                                stable_at: Local::now().to_rfc3339(),
                            },
                        );
                        break;
                    }
                    Ok(false) => {
                        // Not yet stable, continue checking
                        std::thread::sleep(STABILITY_CHECK_INTERVAL);
                    }
                    Err(e) => {
                        // Stability check failed
                        let _ = app_clone.emit(
                            "watcher:ingest_failed",
                            IngestFailedPayload {
                                path: path_clone.display().to_string(),
                                error: format!("Stability check failed: {e}"),
                            },
                        );
                        return;
                    }
                }
            }

            // File is stable, begin ingestion
            if let Ok(mut s) = state_clone.lock() {
                *s = WatcherServiceState::Ingesting;
            }
            Self::emit_status(&app_clone, &state_clone, None);

            let _ = app_clone.emit(
                "watcher:ingest_started",
                IngestStartedPayload {
                    path: path_clone.display().to_string(),
                },
            );

            utils::log_event(
                "Watcher",
                &format!("Ingesting file: {}", path_clone.display()),
            );

            // Perform actual ingestion
            match Self::ingest_file(&path_clone) {
                Ok((dataset_id, rows, cols)) => {
                    let _ = app_clone.emit(
                        "watcher:ingest_succeeded",
                        IngestSucceededPayload {
                            path: path_clone.display().to_string(),
                            dataset_id: dataset_id.to_string(),
                            rows: Some(rows),
                            cols: Some(cols),
                        },
                    );

                    utils::log_event(
                        "Watcher",
                        &format!(
                            "Successfully ingested {} ({} rows, {} cols) -> dataset {}",
                            path_clone.display(),
                            rows,
                            cols,
                            dataset_id
                        ),
                    );
                }
                Err(e) => {
                    let _ = app_clone.emit(
                        "watcher:ingest_failed",
                        IngestFailedPayload {
                            path: path_clone.display().to_string(),
                            error: format!("Ingestion failed: {e}"),
                        },
                    );

                    utils::log_event("Watcher", &format!("Ingestion failed: {e}"));
                }
            }

            if let Ok(mut s) = state_clone.lock() {
                *s = WatcherServiceState::Watching;
            }
            Self::emit_status(&app_clone, &state_clone, None);
        });
    }

    /// Ingest a file and create a lifecycle dataset
    /// Returns (`dataset_id`, `row_count`, `col_count`)
    fn ingest_file(path: &Path) -> Result<(uuid::Uuid, usize, usize)> {
        use crate::analyser::lifecycle::{
            DatasetRegistry, stages::LifecycleStage, transforms::TransformPipeline,
        };
        use crate::analyser::logic::flows::analyze_file_flow;

        // Run analysis on the file
        let rt = tokio::runtime::Runtime::new()?;
        let analysis_response = rt.block_on(analyze_file_flow(path.to_path_buf()))?;

        let row_count = analysis_response.total_row_count;
        let col_count = analysis_response.column_count;

        // Extract filename for dataset name
        let file_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Unnamed Dataset")
            .to_owned();

        // Get data directory and create registry
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("beefcake")
            .join("datasets");

        let registry = DatasetRegistry::new(data_dir)?;

        // Create lifecycle dataset
        let dataset_id = registry.create_dataset(file_name, path.to_path_buf())?;

        // Create Profiled version with the analysis results
        // Note: The Raw version is created automatically by create_dataset
        // We'll store the analysis as a Profiled version
        let empty_pipeline = TransformPipeline::empty();

        let _profiled_version_id =
            registry.apply_transforms(&dataset_id, empty_pipeline, LifecycleStage::Profiled)?;

        Ok((dataset_id, row_count, col_count))
    }

    /// Emit status event to frontend
    fn emit_status(
        app: &AppHandle,
        state: &Arc<Mutex<WatcherServiceState>>,
        message: Option<String>,
    ) {
        let current_state = state
            .lock()
            .map(|s| *s)
            .unwrap_or(WatcherServiceState::Idle);
        let config = WatcherConfig::load().unwrap_or_default();

        let _ = app.emit(
            "watcher:status",
            WatcherStatusPayload {
                enabled: config.enabled,
                folder: config.folder.display().to_string(),
                state: current_state,
                message,
            },
        );
    }

    /// Send a command to the watcher service
    pub fn send_command(&self, cmd: WatcherMessage) -> Result<()> {
        self.command_tx
            .send(cmd)
            .context("Failed to send command to watcher service")
    }

    /// Get current state
    pub fn get_state(&self) -> WatcherServiceState {
        self.state
            .lock()
            .map(|s| *s)
            .unwrap_or(WatcherServiceState::Idle)
    }
}
