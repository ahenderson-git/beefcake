//! Filesystem watcher module
//!
//! Monitors a folder for new CSV/JSON/Parquet files and automatically ingests them
//! into the dataset lifecycle system.
//!
//! ## Architecture
//!
//! ```text
//! WatcherService (background thread)
//!   │
//!   ├─> notify::Watcher (filesystem events)
//!   ├─> StabilityChecker (ensures file is fully written)
//!   └─> Event Emission (to frontend via Tauri)
//! ```
//!
//! ## Features
//!
//! - Non-recursive folder watching (single directory only)
//! - File stability detection (prevents reading incomplete files)
//! - Supported formats: CSV, JSON, Parquet
//! - Real-time event emission to frontend via Tauri
//! - Persistent configuration with auto-start
//! - Activity feed with retry functionality
//!
//! ## Example Usage
//!
//! ```no_run
//! use beefcake::watcher::{init, start, stop, get_state};
//! use std::path::PathBuf;
//! use tauri::AppHandle;
//!
//! // Initialize watcher on app startup
//! # fn example(app: AppHandle) -> anyhow::Result<()> {
//! init(app)?;
//!
//! // Start watching a folder
//! start(PathBuf::from("/path/to/watch"))?;
//!
//! // Check watcher state
//! let state = get_state()?;
//! println!("Watcher enabled: {}", state.enabled);
//!
//! // Stop watching
//! stop()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Event Flow
//!
//! 1. File created/modified in watched folder
//! 2. `notify` emits filesystem event
//! 3. Watcher filters for supported extensions (csv/json/parquet)
//! 4. File added to stability checker
//! 5. Stability checker waits for writes to complete (default 2s)
//! 6. Once stable, ingestion begins
//! 7. Dataset created in Raw lifecycle stage
//! 8. Success/failure event emitted to UI
//!
//! ## Configuration
//!
//! Configuration is persisted in `config/watcher.json`:
//! ```json
//! {
//!   "enabled": true,
//!   "folder": "/path/to/watch",
//!   "stability_window_secs": 2
//! }
//! ```

pub mod config;
pub mod events;
pub mod service;

pub use config::WatcherConfig;
pub use events::*;
pub use service::{WatcherMessage, WatcherService};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::AppHandle;

/// Global watcher service instance
static WATCHER_SERVICE: LazyLock<Arc<Mutex<Option<WatcherService>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

/// Initialize the watcher service
pub fn init(app: AppHandle) -> Result<()> {
    let config = WatcherConfig::load()?;

    let service = WatcherService::new(app, config.clone())?;

    *WATCHER_SERVICE.lock().unwrap() = Some(service);

    // Auto-start if configured
    if config.enabled && !config.folder.as_os_str().is_empty() {
        start(config.folder)?;
    }

    Ok(())
}

/// Start watching a folder
pub fn start(folder: PathBuf) -> Result<()> {
    let service = WATCHER_SERVICE.lock().unwrap();
    if let Some(svc) = service.as_ref() {
        svc.send_command(WatcherMessage::Start(folder.clone()))?;

        // Update and save config
        let mut config = WatcherConfig::load()?;
        config.enabled = true;
        config.folder = folder;
        config.save()?;
    }
    Ok(())
}

/// Stop watching
pub fn stop() -> Result<()> {
    let service = WATCHER_SERVICE.lock().unwrap();
    if let Some(svc) = service.as_ref() {
        svc.send_command(WatcherMessage::Stop)?;

        // Update and save config
        let mut config = WatcherConfig::load()?;
        config.enabled = false;
        config.save()?;
    }
    Ok(())
}

/// Set the watched folder
pub fn set_folder(folder: PathBuf) -> Result<()> {
    // Update config
    let mut config = WatcherConfig::load()?;
    config.folder = folder.clone();
    config.save()?;

    // Restart watcher if currently enabled
    if config.enabled {
        let service = WATCHER_SERVICE.lock().unwrap();
        if let Some(svc) = service.as_ref() {
            svc.send_command(WatcherMessage::Stop)?;
            svc.send_command(WatcherMessage::Start(folder))?;
        }
    }

    Ok(())
}

/// Manually trigger ingestion of a specific file
pub fn ingest_now(path: PathBuf) -> Result<()> {
    let service = WATCHER_SERVICE.lock().unwrap();
    if let Some(svc) = service.as_ref() {
        svc.send_command(WatcherMessage::IngestNow(path))?;
    }
    Ok(())
}

/// Get current watcher state
pub fn get_state() -> Result<WatcherStatusPayload> {
    let config = WatcherConfig::load()?;
    let state = {
        let service = WATCHER_SERVICE.lock().unwrap();
        service
            .as_ref()
            .map(|s| s.get_state())
            .unwrap_or(WatcherServiceState::Idle)
    };

    Ok(WatcherStatusPayload {
        enabled: config.enabled,
        folder: config.folder.display().to_string(),
        state,
        message: None,
    })
}
