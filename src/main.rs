//! # Beefcake Application Entry Point
//!
//! This is the main entry point for the Beefcake desktop application.
//! It handles both CLI mode and GUI mode based on command-line arguments.
//!
//! ## Application Flow
//!
//! ```text
//! main()
//!   │
//!   ├─> Parse CLI arguments (clap)
//!   │
//!   ├─> If command provided:
//!   │   ├─> Create Tokio runtime (async)
//!   │   └─> Execute CLI command
//!   │
//!   └─> Otherwise:
//!       └─> Launch Tauri GUI application
//! ```
//!
//! ## CLI Mode
//!
//! When run with arguments, operates as a command-line tool:
//! ```bash
//! beefcake analyze data.csv
//! beefcake pipeline execute spec.json
//! ```
//!
//! ## GUI Mode
//!
//! When run without arguments, launches the desktop application:
//! ```bash
//! beefcake
//! ```
//!
//! ## Rust Concepts Demonstrated
//!
//! ### Module System
//! ```rust
//! mod cli;           // Imports src/cli.rs
//! mod tauri_app;     // Imports src/tauri_app.rs
//! ```
//! These are private modules. For public modules, use `pub mod`.
//!
//! ### Conditional Compilation
//! ```rust
//! #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! ```
//! This attribute tells Windows to run without console window in release builds.
//! - `debug_assertions`: true in debug mode, false in release
//! - `windows_subsystem = "windows"`: no console window
//!
//! ### Tokio Runtime
//! ```rust
//! tokio::runtime::Runtime::new()?.block_on(async_function())?;
//! ```
//! Creates async runtime and runs async code synchronously (blocks until complete).
//!
//! ## Error Handling
//!
//! Returns `Result<(), Box<dyn std::error::Error>>` which means:
//! - `()` on success (no meaningful return value)
//! - `Box<dyn std::error::Error>` on failure (any error type)
//!
//! The `?` operator propagates errors to the caller (Rust runtime).

#![warn(clippy::all, rust_2018_idioms)]
#![expect(clippy::print_stdout)]  // Allow println! in main binary
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Private modules - only accessible within this binary
mod cli;
mod error;
mod export;
mod python_runner;
mod system;
mod tauri_app;

use anyhow::Result;
use clap::Parser as _;

/// Main entry point for the Beefcake application.
///
/// Determines whether to run in CLI mode or GUI mode based on
/// command-line arguments.
///
/// # Examples
///
/// CLI mode (with arguments):
/// ```bash
/// beefcake analyze data.csv
/// ```
///
/// GUI mode (no arguments):
/// ```bash
/// beefcake
/// ```
///
/// # Errors
///
/// Returns error if:
/// - CLI command fails
/// - Tokio runtime initialization fails
/// - Tauri application fails to start
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment logger for debugging
    // Set RUST_LOG=debug to see detailed logs
    env_logger::init();

    // Parse command-line arguments using clap
    // This reads std::env::args() and validates against Cli struct
    let cli = cli::Cli::parse();

    // Check if a subcommand was provided (e.g., "analyze", "pipeline")
    if let Some(command) = cli.command {
        // CLI mode: Run command and exit
        //
        // We need a Tokio runtime because some operations are async
        // (database queries, HTTP requests, etc.)
        tokio::runtime::Runtime::new()?
            .block_on(cli::run_command(command))?;
        return Ok(());
    }

    // GUI mode: Launch Tauri desktop application
    // This call blocks until the application window is closed
    tauri_app::run();
    Ok(())
}
