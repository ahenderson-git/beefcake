use anyhow::{Context as _, Result};
use beefcake::analyser::db::DbClient;
use beefcake::analyser::logic::types::ColumnCleanConfig;
use beefcake::analyser::logic::*;
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
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
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
}

#[expect(clippy::too_many_lines)]
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
    let file = match file {
        Some(f) => f,
        None => get_default_input_file()?,
    };

    let table = match table {
        Some(t) => t,
        None => file
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string(),
    };

    println!(
        "Importing {0} into table {schema}.{table}...",
        file.display()
    );
    let progress = Arc::new(AtomicU64::new(0));
    let df = load_df(&file, &progress).context("Failed to load dataframe")?;

    let df = apply_cleaning(df, config_path, clean)?;

    let config = beefcake::utils::load_app_config();
    let effective_url = if let Some(url) = db_url {
        url
    } else if let Some(id) = config.active_import_id {
        config
            .connections
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.settings.connection_string(&c.id))
            .ok_or_else(|| anyhow::anyhow!("Active import connection not found in config"))?
    } else {
        anyhow::bail!("No database URL provided and no active import connection set.");
    };

    let opts =
        PgConnectOptions::from_str(&effective_url).context("Failed to parse database URL")?;
    let client = DbClient::connect(opts).await?;

    client.init_schema().await?;
    client
        .push_dataframe(0, &df, Some(&schema), Some(&table))
        .await?;
    println!("Successfully imported {} rows.", df.height());

    // Archive
    let archived = beefcake::utils::archive_processed_file(&file)?;
    println!("File archived to: {}", archived.display());
    Ok(())
}

async fn handle_export(
    input: Option<String>,
    output: Option<PathBuf>,
    db_url: Option<String>,
    _schema: String,
    clean: bool,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let config_data = beefcake::utils::load_app_config();
    let _effective_url = if let Some(url) = db_url {
        Some(url)
    } else if let Some(id) = config_data.active_export_id {
        config_data
            .connections
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.settings.connection_string(&c.id))
    } else {
        None
    };

    let input_path = match input {
        Some(i) => PathBuf::from(i),
        None => get_default_input_file()?,
    };

    let output_path = if let Some(o) = output {
        o
    } else {
        let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!(
            "{}/exported_{stem}.parquet",
            beefcake::utils::DATA_PROCESSED_DIR
        ))
    };

    if input_path.exists() {
        println!(
            "Converting {0} to {1} (lazily)...",
            input_path.display(),
            output_path.display()
        );
        
        let lf = load_df_lazy(&input_path).context("Failed to load input file lazily")?;

        let configs = if let Some(config_path) = config_path {
            println!("Loading config from {}...", config_path.display());
            load_config(&config_path)?
        } else if clean {
            println!("Auto-cleaning (sampling 500k rows for analysis)...");
            // Important: Use limit only for the analysis/config generation
            let sample_df = lf.clone().limit(500_000).collect().context("Failed to sample data for auto-cleaning")?;
            let summaries = beefcake::analyser::logic::analyse_df(&sample_df, 0.0).context("Failed to analyse sample for auto-cleaning")?;
            let mut configs = HashMap::new();
            for summary in summaries {
                let mut config = ColumnCleanConfig::default();
                config.new_name = summary.standardized_name.clone();
                summary.apply_advice_to_config(&mut config);
                configs.insert(summary.name.clone(), config);
            }
            configs
        } else {
            HashMap::new()
        };

        println!("Applying transformations...");
        let cleaned_lf = clean_df_lazy(lf, &configs, true)?;

        if output_path.extension().and_then(|s| s.to_str()) == Some("parquet") {
            println!("Streaming to parquet: {}...", output_path.display());
            cleaned_lf
                .with_streaming(true)
                .sink_parquet(&output_path, Default::default(), None)
                .context("Failed to sink to parquet")?;
        } else {
            println!("Collecting and saving to {}...", output_path.display());
            let mut df = cleaned_lf.collect().context("Failed to collect data for export")?;
            save_df(&mut df, &output_path).context("Failed to save output file")?;
        }

        println!("Successfully exported.");

        // Archive
        let archived = beefcake::utils::archive_processed_file(&input_path)?;
        println!("Input file archived to: {}", archived.display());
    } else {
        anyhow::bail!("Input file not found. Table export from DB is not yet implemented.");
    }
    Ok(())
}

async fn handle_clean(
    file: Option<PathBuf>,
    output: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let input_file = match file {
        Some(f) => f,
        None => get_default_input_file()?,
    };

    let output_file = if let Some(o) = output {
        o
    } else {
        let stem = input_file.file_stem().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!(
            "{}/cleaned_{stem}.parquet",
            beefcake::utils::DATA_PROCESSED_DIR
        ))
    };

    println!(
        "Cleaning {0} and saving to {1} (lazily)...",
        input_file.display(),
        output_file.display()
    );

    let lf = load_df_lazy(&input_file).context("Failed to load input file lazily")?;

    let configs = if let Some(cp) = config_path {
        println!("Loading config from {}...", cp.display());
        load_config(&cp)?
    } else {
        println!("Auto-cleaning (sampling 500k rows for analysis)...");
        // Important: Use limit only for analysis, but clean the full LazyFrame later
        let sample_df = lf.clone().limit(500_000).collect().context("Failed to sample data for auto-cleaning")?;
        let summaries = beefcake::analyser::logic::analyse_df(&sample_df, 0.0).context("Failed to analyse sample for auto-cleaning")?;
        let mut configs = HashMap::new();
        for summary in summaries {
            let mut config = ColumnCleanConfig::default();
            config.new_name = summary.standardized_name.clone();
            summary.apply_advice_to_config(&mut config);
            configs.insert(summary.name.clone(), config);
        }
        configs
    };

    let cleaned_lf = clean_df_lazy(lf, &configs, true)?;

    if output_file.extension().and_then(|s| s.to_str()) == Some("parquet") {
        println!("Streaming to parquet: {}...", output_file.display());
        cleaned_lf
            .with_streaming(true)
            .sink_parquet(&output_file, Default::default(), None)
            .context("Failed to sink to parquet")?;
    } else {
        println!("Collecting and saving to {}...", output_file.display());
        let mut df = cleaned_lf.collect().context("Failed to collect data for saving")?;
        save_df(&mut df, &output_file).context("Failed to save cleaned file")?;
    }

    println!("Successfully cleaned.");

    // Archive
    let archived = beefcake::utils::archive_processed_file(&input_file)?;
    println!("Original file archived to: {}", archived.display());
    Ok(())
}

fn apply_cleaning(
    mut df: DataFrame,
    config: Option<PathBuf>,
    clean: bool,
) -> Result<DataFrame> {
    if let Some(config_path) = config {
        let configs = load_config(&config_path)?;
        println!("Applying configuration from {}...", config_path.display());
        df = clean_df(df, &configs, true)?;
    } else if clean {
        println!("Applying heuristic cleaning...");
        df = auto_clean_df(df, true)?;
    }
    Ok(df)
}

fn load_config(path: &PathBuf) -> Result<HashMap<String, ColumnCleanConfig>> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;
    serde_json::from_str(&content).context("Failed to parse JSON config")
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
