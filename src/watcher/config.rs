//! Watcher configuration management
//!
//! Handles persistent configuration for the folder watcher service.

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatcherConfig {
    /// Whether the watcher is enabled
    pub enabled: bool,
    /// Folder to watch
    pub folder: PathBuf,
    /// Whether to automatically ingest new files
    pub auto_ingest: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            folder: PathBuf::new(),
            auto_ingest: true,
        }
    }
}

impl WatcherConfig {
    /// Get the config file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        Ok(config_dir.join("beefcake").join("watcher.json"))
    }

    /// Load configuration from disk
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read watcher config from {}", path.display()))?;

        let config: Self =
            serde_json::from_str(&contents).context("Failed to parse watcher config JSON")?;

        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize watcher config")?;

        std::fs::write(&path, json)
            .with_context(|| format!("Failed to write watcher config to {}", path.display()))?;

        Ok(())
    }
}
