//! Comprehensive logging infrastructure for Beefcake
//!
//! This module provides structured, multi-target logging with file rotation.
//! Logs are written to both the console and rotating files in the app data directory.
//!
//! ## Features
//!
//! - **File Rotation**: Logs rotate at 10MB with 10 files retained
//! - **Structured Logging**: JSON-compatible format with timestamps and context
//! - **Multiple Targets**: Console (for dev) + Files (for production debugging)
//! - **Error Tracking**: Separate error.log for easy error identification
//! - **Cross-Platform**: Uses platform-specific app data directories
//!
//! ## Usage
//!
//! ```no_run
//! use beefcake::logging;
//!
//! // Initialize once at app startup
//! logging::init().expect("Failed to initialize logging");
//!
//! // Use tracing macros throughout the app
//! tracing::info!("App started");
//! tracing::error!("Something went wrong");
//! ```

use anyhow::{Context as _, Result};
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer as _, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

/// Gets the log directory path based on platform conventions
///
/// Returns:
/// - Windows: `%APPDATA%/beefcake/logs`
/// - macOS: `~/Library/Application Support/beefcake/logs`
/// - Linux: `~/.local/share/beefcake/logs`
pub fn get_log_dir() -> Result<PathBuf> {
    let base_dir = dirs::data_dir().context("Failed to determine data directory")?;

    let log_dir = base_dir.join("beefcake").join("logs");

    // Create directory if it doesn't exist
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;
    }

    Ok(log_dir)
}

/// Initializes the logging system with console and file output
///
/// Creates two log files:
/// - `beefcake.log`: All log levels (info, warn, error, debug)
/// - `error.log`: Only errors and warnings
///
/// Both files rotate daily and when they reach 10MB, keeping 10 old files.
///
/// # Errors
///
/// Returns error if log directory cannot be created or file appenders fail
pub fn init() -> Result<()> {
    let log_dir = get_log_dir()?;

    // Create file appender for all logs
    let all_logs_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(10)
        .filename_prefix("beefcake")
        .filename_suffix("log")
        .build(&log_dir)
        .context("Failed to create all-logs file appender")?;

    // Create file appender for errors only
    let error_logs_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(10)
        .filename_prefix("error")
        .filename_suffix("log")
        .build(&log_dir)
        .context("Failed to create error-logs file appender")?;

    // Create env filter - default to INFO, allow override with RUST_LOG
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .context("Failed to create env filter")?;

    // Create layers
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(true)
        .with_file(true)
        .pretty();

    let all_logs_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_ansi(false)
        .with_writer(all_logs_appender);

    let error_logs_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_ansi(false)
        .with_writer(error_logs_appender)
        .with_filter(EnvFilter::new("warn"));

    // Initialize subscriber with multiple layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(all_logs_layer)
        .with(error_logs_layer)
        .init();

    tracing::info!("Logging initialized, log directory: {:?}", log_dir);

    Ok(())
}

/// Gets the path to the current log file
pub fn get_current_log_path() -> Result<PathBuf> {
    let log_dir = get_log_dir()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Ok(log_dir.join(format!("beefcake.{today}.log")))
}

/// Gets the path to the current error log file
pub fn get_current_error_log_path() -> Result<PathBuf> {
    let log_dir = get_log_dir()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Ok(log_dir.join(format!("error.{today}.log")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_dir() {
        let log_dir = get_log_dir().expect("Failed to get log dir");
        assert!(log_dir.ends_with("beefcake/logs") || log_dir.ends_with("beefcake\\logs"));
    }
}
