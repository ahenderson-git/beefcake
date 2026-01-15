//! Centralized error handling for the beefcake application.
//!
//! This module demonstrates several Rust error handling patterns:
//!
//! ## Custom Error Types with `enum`
//!
//! We use an `enum` to represent different error categories. This is more
//! type-safe than using strings and allows pattern matching:
//!
//! ```
//! use beefcake::error::BeefcakeError;
//!
//! fn handle_error(err: BeefcakeError) {
//!     match err {
//!         BeefcakeError::Io(e) => eprintln!("I/O error: {}", e),
//!         BeefcakeError::DataProcessing(msg) => eprintln!("Data error: {}", msg),
//!         BeefcakeError::Aborted => println!("User cancelled"),
//!         _ => eprintln!("Other error: {}", err),
//!     }
//! }
//! ```
//!
//! ## The `From` Trait for Error Conversion
//!
//! We implement `From<E>` for automatic error type conversion. This allows
//! the `?` operator to work seamlessly:
//!
//! ```no_run
//! use beefcake::error::{BeefcakeError, Result};
//! use std::fs;
//!
//! fn read_config(path: &str) -> Result<String> {
//!     // std::io::Error automatically converts to BeefcakeError via From trait
//!     let content = fs::read_to_string(path)?;
//!     Ok(content)
//! }
//! ```
//!
//! ## Context Extension Trait
//!
//! The `ResultExt` trait adds `.context()` method to any `Result` for
//! adding contextual information to errors:
//!
//! ```no_run
//! use beefcake::error::ResultExt;
//! use std::fs;
//!
//! fn load_data() -> beefcake::error::Result<String> {
//!     fs::read_to_string("data.csv")
//!         .context("Failed to load dataset")?;
//!     Ok("".to_string())
//! }
//! // Error message will include both context and original error
//! ```
//!
//! ## Tauri Integration
//!
//! Tauri commands need `String` errors for JSON serialization. We implement
//! `From<BeefcakeError> for String` to enable this:
//!
//! ```no_run
//! # use beefcake::error::{BeefcakeError, Result};
//! // Tauri command signature
//! fn analyze_file(path: String) -> Result<String, String> {
//!     // BeefcakeError auto-converts to String
//!     let result = process_file(&path)?;
//!     Ok(result)
//! }
//!
//! fn process_file(path: &str) -> Result<String> {
//!     // ... implementation
//!     Ok("done".to_string())
//! }
//! ```
//!
//! ## Learning Resources
//!
//! For more on Rust error handling patterns, see `docs/RUST_CONCEPTS.md`.

use std::fmt;

/// Main error type for beefcake operations.
#[derive(Debug)]
pub enum BeefcakeError {
    /// I/O errors (file operations, network, etc.)
    Io(std::io::Error),

    /// Data processing errors (Polars, parsing, etc.)
    DataProcessing(String),

    /// Database operation errors
    Database(String),

    /// Python execution errors
    Python(String),

    /// Configuration errors
    Config(String),

    /// File not found or invalid path
    InvalidPath(String),

    /// Operation aborted by user
    Aborted,

    /// Generic error with context
    Other(String),
}

impl fmt::Display for BeefcakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::DataProcessing(msg) => write!(f, "Data processing error: {msg}"),
            Self::Database(msg) => write!(f, "Database error: {msg}"),
            Self::Python(msg) => write!(f, "Python execution error: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::InvalidPath(msg) => write!(f, "Invalid path: {msg}"),
            Self::Aborted => write!(f, "Operation aborted by user"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for BeefcakeError {}

impl From<std::io::Error> for BeefcakeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<anyhow::Error> for BeefcakeError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<serde_json::Error> for BeefcakeError {
    fn from(err: serde_json::Error) -> Self {
        Self::Config(format!("JSON error: {err}"))
    }
}

impl From<polars::error::PolarsError> for BeefcakeError {
    fn from(err: polars::error::PolarsError) -> Self {
        Self::DataProcessing(err.to_string())
    }
}

impl From<sqlx::Error> for BeefcakeError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

// For Tauri commands, we need to convert to String
impl From<BeefcakeError> for String {
    fn from(err: BeefcakeError) -> Self {
        err.to_string()
    }
}

/// Result type alias for beefcake operations.
pub type Result<T> = std::result::Result<T, BeefcakeError>;

/// Extension trait to add context to results.
pub trait ResultExt<T> {
    /// Add context to an error.
    fn context(self, msg: impl Into<String>) -> Result<T>;

    /// Add context using a closure (lazy evaluation).
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<BeefcakeError>,
{
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let err: BeefcakeError = e.into();
            BeefcakeError::Other(format!("{}: {}", msg.into(), err))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let err: BeefcakeError = e.into();
            BeefcakeError::Other(format!("{}: {}", f(), err))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BeefcakeError::DataProcessing("column not found".to_owned());
        assert_eq!(err.to_string(), "Data processing error: column not found");
    }

    #[test]
    fn test_error_conversion_to_string() {
        let err = BeefcakeError::Aborted;
        let s: String = err.into();
        assert_eq!(s, "Operation aborted by user");
    }

    #[test]
    fn test_result_context() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file.txt",
        ));

        let result: Result<()> = result.context("Failed to read file");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read file")
        );
    }
}
