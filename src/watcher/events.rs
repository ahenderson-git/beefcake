//! Event types for watcher system
//!
//! Defines all event payloads emitted to the frontend via Tauri events.

use serde::Serialize;

/// Current state of the watcher service
#[derive(Debug, Clone, Serialize)]
pub struct WatcherStatusPayload {
    pub enabled: bool,
    pub folder: String,
    pub state: WatcherServiceState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Watcher service state
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatcherServiceState {
    Idle,
    Watching,
    Ingesting,
    Error,
}

impl std::fmt::Display for WatcherServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Watching => write!(f, "watching"),
            Self::Ingesting => write!(f, "ingesting"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// File detected event payload
#[derive(Debug, Clone, Serialize)]
pub struct FileDetectedPayload {
    pub path: String,
    pub file_type: String,   // "csv" | "json"
    pub detected_at: String, // ISO datetime
}

/// File ready (stable) event payload
#[derive(Debug, Clone, Serialize)]
pub struct FileReadyPayload {
    pub path: String,
    pub stable_at: String, // ISO datetime
}

/// Ingestion started event payload
#[derive(Debug, Clone, Serialize)]
pub struct IngestStartedPayload {
    pub path: String,
}

/// Ingestion succeeded event payload
#[derive(Debug, Clone, Serialize)]
pub struct IngestSucceededPayload {
    pub path: String,
    pub dataset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<usize>,
}

/// Ingestion failed event payload
#[derive(Debug, Clone, Serialize)]
pub struct IngestFailedPayload {
    pub path: String,
    pub error: String,
}
