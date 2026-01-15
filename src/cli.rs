use anyhow::{Context as _, Result};
use beefcake::analyser::logic::types::ColumnCleanConfig;
use beefcake::analyser::logic::{
    clean_df_lazy, flows, get_parquet_write_options, load_df_lazy, save_df,
};
use clap::{Parser, Subcommand};
use polars::prelude::*;
use sqlx::postgres::PgConnectOptions;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

#[derive(Parser)]
#[command(name = "beefcake", about = "Data analysis and migration tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Context object for CLI operations, holding shared application state.
struct CliContext {
    app_config: beefcake::utils::AppConfig,
}

impl CliContext {
    fn new() -> Self {
        Self {
            app_config: beefcake::utils::load_app_config(),
        }
    }

    /// Resolve database URL from explicit parameter or config.
    fn resolve_db_url(
        &self,
        explicit_url: Option<String>,
        config_id: Option<String>,
    ) -> Result<String> {
        if let Some(url) = explicit_url {
            return Ok(url);
        }

        if let Some(id) = config_id {
            return self
                .app_config
                .settings
                .connections
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.settings.connection_string(&c.id))
                .ok_or_else(|| anyhow::anyhow!("Connection with ID '{id}' not found in config"));
        }

        anyhow::bail!("No database URL provided and no active connection set")
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Import a file into the database
    Import {
        /// Path to the file to import (CSV, Parquet, JSON). Defaults to first file in the input directory.
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Target table name. Defaults to filename stem.
        #[arg(short, long)]
        table: Option<String>,

        /// Target schema name
        #[arg(long, default_value = "public")]
        schema: String,

        /// Database connection URL (e.g. <postgres://user:pass@localhost/db>)
        #[arg(long, env = "DATABASE_URL")]
        db_url: Option<String>,

        /// Automatically clean the data using heuristics before importing
        #[arg(long)]
        clean: bool,

        /// Path to a JSON cleaning configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Export database table or file to a different format
    Export {
        /// Input file path. Defaults to first file in the input directory.
        #[arg(short, long)]
        input: Option<String>,

        /// Output file path.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Database connection URL (required if input is a table)
        #[arg(long, env = "DATABASE_URL")]
        db_url: Option<String>,

        /// Source schema (if input is a table)
        #[arg(long, default_value = "public")]
        schema: String,

        /// Automatically clean the data using heuristics before exporting
        #[arg(long)]
        clean: bool,

        /// Path to a JSON cleaning configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Clean a file and save the result
    Clean {
        /// Input file path. Defaults to first file in the input directory.
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Output file path. Defaults to a cleaned file in the processed directory.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to a JSON cleaning configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Run a pipeline specification
    Run {
        /// Path to the pipeline spec JSON file
        #[arg(long, required = true)]
        spec: PathBuf,

        /// Path to the input data file
        #[arg(long, required = true)]
        input: PathBuf,

        /// Path for the output file (overrides spec `output.path_template`)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Date string for path template substitution (format: YYYY-MM-DD, default: today)
        #[arg(long)]
        date: Option<String>,

        /// Path to write execution log
        #[arg(long)]
        log: Option<PathBuf>,

        /// Fail with error if warnings are generated
        #[arg(long)]
        fail_on_warnings: bool,
    },
}

pub async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Import {
            file,
            table,
            schema,
            db_url,
            clean,
            config,
        } => handle_import(file, table, schema, db_url, clean, config).await,
        Commands::Export {
            input,
            output,
            db_url,
            schema,
            clean,
            config,
        } => handle_export(input, output, db_url, schema, clean, config).await,
        Commands::Clean {
            file,
            output,
            config,
        } => handle_clean(file, output, config).await,
        Commands::Run {
            spec,
            input,
            output,
            date: _,
            log,
            fail_on_warnings,
        } => handle_run(spec, input, output, log, fail_on_warnings).await,
    }
}

async fn handle_import(
    file: Option<PathBuf>,
    table: Option<String>,
    schema: String,
    db_url: Option<String>,
    clean: bool,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let ctx = CliContext::new();
    let file = file.unwrap_or(get_default_input_file()?);
    let table = table.unwrap_or_else(|| {
        file.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    println!(
        "Importing {0} into table {schema}.{table} (streaming)...",
        file.display()
    );

    let lf = load_df_lazy(&file).context("Failed to load dataframe lazily")?;
    let configs = resolve_cleaning_config(config_path, clean, lf.clone())?;

    let effective_url =
        ctx.resolve_db_url(db_url, ctx.app_config.settings.active_import_id.clone())?;
    let opts =
        PgConnectOptions::from_str(&effective_url).context("Failed to parse database URL")?;

    flows::push_to_db_flow(file.clone(), opts, schema, table, configs).await?;

    println!("Successfully imported.");
    archive_and_log(&file, "File archived to")?;
    Ok(())
}

async fn handle_export(
    input: Option<String>,
    output: Option<PathBuf>,
    _db_url: Option<String>,
    _schema: String,
    clean: bool,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let input_path = input
        .map(PathBuf::from)
        .unwrap_or(get_default_input_file()?);

    if !input_path.exists() {
        anyhow::bail!("Input file not found. Table export from DB is not yet implemented.");
    }

    let output_path =
        output.unwrap_or_else(|| get_default_output_path(&input_path, "exported", "parquet"));

    println!(
        "Converting {0} to {1} (lazily)...",
        input_path.display(),
        output_path.display()
    );

    let lf = load_df_lazy(&input_path).context("Failed to load input file lazily")?;
    let configs = resolve_cleaning_config(config_path, clean, lf.clone())?;

    println!("Applying transformations...");
    let cleaned_lf = clean_df_lazy(lf, &configs, true)?;

    sink_to_file(cleaned_lf, &output_path)?;

    println!("Successfully exported.");
    archive_and_log(&input_path, "Input file archived to")?;
    Ok(())
}

async fn handle_clean(
    file: Option<PathBuf>,
    output: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let input_file = file.unwrap_or(get_default_input_file()?);
    let output_file =
        output.unwrap_or_else(|| get_default_output_path(&input_file, "cleaned", "parquet"));

    println!(
        "Cleaning {0} and saving to {1} (lazily)...",
        input_file.display(),
        output_file.display()
    );

    let lf = load_df_lazy(&input_file).context("Failed to load input file lazily")?;

    // For clean command, always auto-clean if no config provided
    let configs = resolve_cleaning_config(config_path, true, lf.clone())?;
    let cleaned_lf = clean_df_lazy(lf, &configs, true)?;

    sink_to_file(cleaned_lf, &output_file)?;

    println!("Successfully cleaned.");
    archive_and_log(&input_file, "Original file archived to")?;
    Ok(())
}

/// Load cleaning configuration from a JSON file.
fn load_config(path: &PathBuf) -> Result<HashMap<String, ColumnCleanConfig>> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;
    serde_json::from_str(&content).context("Failed to parse JSON config")
}

/// Get the first file from the default input directory.
fn get_default_input_file() -> Result<PathBuf> {
    let mut entries = std::fs::read_dir(beefcake::utils::DATA_INPUT_DIR)
        .context(format!(
            "Failed to read {} directory. Please ensure it exists.",
            beefcake::utils::DATA_INPUT_DIR
        ))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());
    entries.first().map(|e| e.path()).ok_or_else(|| {
        anyhow::anyhow!(
            "No files found in {} and no input file provided.",
            beefcake::utils::DATA_INPUT_DIR
        )
    })
}

/// Resolve cleaning configuration from config path or auto-cleaning flag.
fn resolve_cleaning_config(
    config_path: Option<PathBuf>,
    auto_clean: bool,
    lf: LazyFrame,
) -> Result<HashMap<String, ColumnCleanConfig>> {
    if let Some(path) = config_path {
        println!("Loading config from {}...", path.display());
        load_config(&path)
    } else if auto_clean {
        println!("Auto-cleaning (sampling 500k rows for analysis)...");
        flows::generate_auto_clean_configs(lf)
    } else {
        Ok(HashMap::new())
    }
}

/// Generate default output path in the processed directory.
fn get_default_output_path(input_path: &Path, prefix: &str, extension: &str) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    PathBuf::from(format!(
        "{}/{prefix}_{stem}.{extension}",
        beefcake::utils::DATA_PROCESSED_DIR
    ))
}

/// Sink a `LazyFrame` to a file with appropriate format handling.
fn sink_to_file(lf: LazyFrame, output_path: &Path) -> Result<()> {
    let ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "parquet" => {
            let options = get_parquet_write_options(&lf)?;
            if let Some(rgs) = options.row_group_size {
                println!(
                    "Streaming to parquet: {} (adaptive row group size: {})...",
                    output_path.display(),
                    rgs
                );
            } else {
                println!("Streaming to parquet: {}...", output_path.display());
            }

            lf.with_streaming(true)
                .sink_parquet(&output_path, options, None)
                .context("Failed to sink to parquet")?;
        }
        "csv" => {
            println!("Streaming to csv: {}...", output_path.display());
            lf.with_streaming(true)
                .sink_csv(output_path, Default::default(), None)
                .context("Failed to sink to csv")?;
        }
        _ => {
            println!("Collecting and saving to {}...", output_path.display());
            let mut df = lf.collect().context("Failed to collect data for saving")?;
            save_df(&mut df, output_path).context("Failed to save file")?;
        }
    }

    Ok(())
}

/// Archive the input file and print the result.
fn archive_and_log(input_path: &Path, message: &str) -> Result<()> {
    let archived = beefcake::utils::archive_processed_file(input_path)?;
    println!("{message}: {}", archived.display());
    Ok(())
}

async fn handle_run(
    spec_path: PathBuf,
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    fail_on_warnings: bool,
) -> Result<()> {
    use beefcake::pipeline::{PipelineSpec, run_pipeline};

    println!("Loading pipeline spec from {}...", spec_path.display());

    // Load pipeline spec
    let spec = PipelineSpec::from_file(&spec_path).context(format!(
        "Failed to load pipeline spec: {}",
        spec_path.display()
    ))?;

    println!("Pipeline: {}", spec.name);
    println!("Version: {}", spec.version);
    println!("Steps: {}", spec.steps.len());
    println!();

    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }

    println!("Input: {}", input_path.display());

    // Execute pipeline
    println!("Running pipeline...");
    let report = run_pipeline(&spec, &input_path, output_path.as_ref())
        .context("Pipeline execution failed")?;

    // Print report
    println!();
    println!("=== Pipeline Execution Report ===");
    println!("{}", report.summary());

    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }

    // Write log if requested
    if let Some(log_path) = log_path {
        let log_content = format!(
            "Pipeline: {}\nSpec Version: {}\nInput: {}\nTimestamp: {}\n\n{}\n\nWarnings:\n{}\n",
            spec.name,
            spec.version,
            input_path.display(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            report.summary(),
            if report.warnings.is_empty() {
                "None".to_owned()
            } else {
                report.warnings.join("\n")
            }
        );

        // Ensure log directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create log directory: {}",
                parent.display()
            ))?;
        }

        std::fs::write(&log_path, log_content)
            .context(format!("Failed to write log file: {}", log_path.display()))?;

        println!("Log written to: {}", log_path.display());
    }

    // Check fail on warnings
    if fail_on_warnings && !report.warnings.is_empty() {
        println!();
        println!("Pipeline completed with warnings and --fail-on-warnings is set.");
        std::process::exit(3);
    }

    println!();
    println!("Pipeline completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory as _;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
