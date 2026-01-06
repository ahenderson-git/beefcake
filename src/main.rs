#![warn(clippy::all, rust_2018_idioms)]
#![expect(clippy::print_stdout)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context as _, Result};
use beefcake::analyser::db::DbClient;
use beefcake::analyser::logic::{ColumnCleanConfig, analyse_df, clean_df, load_df, save_df};
use clap::{Parser, Subcommand};
use polars::prelude::DataFrame;
use sqlx::postgres::PgConnectOptions;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr as _;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Parser)]
#[command(name = "beefcake", about = "Data analysis and migration tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Import a file into the database
    Import {
        /// Path to the file to import (CSV, Parquet, JSON)
        #[arg(short, long)]
        file: PathBuf,

        /// Target table name
        #[arg(short, long)]
        table: String,

        /// Target schema name
        #[arg(long, default_value = "public")]
        schema: String,

        /// Database connection URL (e.g. <postgres://user:pass@localhost/db>)
        #[arg(long, env = "DATABASE_URL")]
        db_url: String,

        /// Automatically clean the data using heuristics before importing
        #[arg(long)]
        clean: bool,
    },
    /// Export database table or file to a different format
    Export {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Database connection URL (required if input is a table)
        #[arg(long, env = "DATABASE_URL")]
        db_url: Option<String>,

        /// Source schema (if input is a table)
        #[arg(long, default_value = "public")]
        schema: String,

        /// Automatically clean the data using heuristics before exporting
        #[arg(long)]
        clean: bool,
    },
    /// Clean a file and save the result
    Clean {
        /// Input file path
        #[arg(short, long)]
        file: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn auto_clean_df(df: DataFrame) -> Result<DataFrame> {
    let summaries = analyse_df(&df, 0.0).context("Failed to analyse dataframe for cleaning")?;
    let mut configs = HashMap::new();
    for summary in summaries {
        let mut config = ColumnCleanConfig {
            new_name: summary.name.clone(),
            ..Default::default()
        };
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    clean_df(df, &configs).context("Failed to clean dataframe")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            match command {
                Commands::Import {
                    file,
                    table,
                    schema,
                    db_url,
                    clean,
                } => {
                    println!(
                        "Importing {} into table {schema}.{table}...",
                        file.display()
                    );
                    let progress = Arc::new(AtomicU64::new(0));
                    let mut df = load_df(&file, &progress).context("Failed to load dataframe")?;

                    if clean {
                        println!("Applying heuristic cleaning...");
                        df = auto_clean_df(df)?;
                    }

                    let opts = PgConnectOptions::from_str(&db_url)
                        .context("Failed to parse database URL")?;
                    let client = DbClient::connect(opts).await?;

                    client.init_schema().await?;
                    client
                        .push_dataframe(0, &df, Some(&schema), Some(&table))
                        .await?;
                    println!("Successfully imported {} rows.", df.height());
                }
                Commands::Export {
                    input,
                    output,
                    db_url: _,
                    schema: _,
                    clean,
                } => {
                    let input_path = PathBuf::from(&input);
                    if input_path.exists() {
                        println!(
                            "Converting {} to {}...",
                            input_path.display(),
                            output.display()
                        );
                        let progress = Arc::new(AtomicU64::new(0));
                        let mut df =
                            load_df(&input_path, &progress).context("Failed to load input file")?;

                        if clean {
                            println!("Applying heuristic cleaning...");
                            df = auto_clean_df(df)?;
                        }

                        save_df(&mut df, &output).context("Failed to save output file")?;
                        println!("Successfully exported {} rows.", df.height());
                    } else {
                        anyhow::bail!(
                            "Input file not found. Table export from DB is not yet implemented."
                        );
                    }
                }
                Commands::Clean { file, output } => {
                    println!(
                        "Cleaning {} and saving to {}...",
                        file.display(),
                        output.display()
                    );
                    let progress = Arc::new(AtomicU64::new(0));
                    let df = load_df(&file, &progress).context("Failed to load input file")?;
                    let mut cleaned_df = auto_clean_df(df)?;
                    save_df(&mut cleaned_df, &output).context("Failed to save cleaned file")?;
                    println!("Successfully cleaned {} rows.", cleaned_df.height());
                }
            }
            Ok::<(), anyhow::Error>(())
        })?;
        return Ok(());
    }

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                #[expect(clippy::large_include_file)]
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "beefcake",
        native_options,
        Box::new(|cc| Ok(Box::new(beefcake::BeefcakeApp::new(cc)))),
    )
    .map_err(|e| e.into())
}
