#![warn(clippy::all, rust_2018_idioms)]
#![expect(clippy::print_stdout)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod export;
mod python_runner;
mod system;
mod tauri_app;

use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let cli = cli::Cli::parse();

    if let Some(command) = cli.command {
        tokio::runtime::Runtime::new()?.block_on(cli::run_command(command))?;
        return Ok(());
    }

    tauri_app::run();
    Ok(())
}
