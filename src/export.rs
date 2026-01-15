use crate::python_runner::{
    execute_python, python_adaptive_sink_snippet, python_load_snippet, python_preamble,
};
use beefcake::analyser::logic::ColumnCleanConfig;
use beefcake::error::{BeefcakeError, Result, ResultExt as _};
use polars::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr as _;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum ExportSourceType {
    Analyser,
    Python,
    Sql,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExportSource {
    #[serde(rename = "type")]
    pub source_type: ExportSourceType,
    pub content: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum ExportDestinationType {
    File,
    Database,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExportDestination {
    #[serde(rename = "type")]
    pub dest_type: ExportDestinationType,
    pub target: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExportOptions {
    pub source: ExportSource,
    pub configs: HashMap<String, ColumnCleanConfig>,
    pub destination: ExportDestination,
    #[serde(default = "default_create_dictionary")]
    pub create_dictionary: bool,
}

fn default_create_dictionary() -> bool {
    true // Default ON
}

pub async fn prepare_export_source(
    source: &ExportSource,
    temp_files: &mut beefcake::utils::TempFileCollection,
) -> Result<LazyFrame> {
    match source.source_type {
        ExportSourceType::Analyser => {
            let path = source.path.as_ref().ok_or_else(|| {
                BeefcakeError::InvalidPath("No path provided for Analyser source".to_owned())
            })?;
            beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
                .context("Failed to load data")
        }
        ExportSourceType::Sql => {
            let query = source.content.as_ref().ok_or_else(|| {
                BeefcakeError::Config("No query provided for Sql source".to_owned())
            })?;
            let path = source.path.as_ref().ok_or_else(|| {
                BeefcakeError::InvalidPath("No data path provided for Sql source".to_owned())
            })?;

            let temp_dir = std::env::temp_dir();
            let temp_output =
                temp_dir.join(format!("beefcake_export_sql_{}.parquet", Uuid::new_v4()));
            temp_files.add(temp_output.clone());

            let python_script = format!(
                r#"{}
data_path = r"{}"
try:
{}
    
    ctx = pl.SQLContext()
    ctx.register("data", lf)
    
    query = """{}"""
    result = ctx.execute(query)
    
{}
except Exception as e:
    print(f"Sql Export Error: {{e}}")
    sys.exit(1)
"#,
                python_preamble(),
                path,
                python_load_snippet("data_path", "lf"),
                query.replace('"', r#"\"""#),
                python_adaptive_sink_snippet("result", &temp_output)
            );

            execute_python(&python_script, None, "Export (Sql)").await?;
            LazyFrame::scan_parquet(temp_output, Default::default())
                .context("Failed to scan Sql result")
        }
        ExportSourceType::Python => {
            let script = source.content.as_ref().ok_or_else(|| {
                BeefcakeError::Config("No script provided for Python source".to_owned())
            })?;
            let path = source.path.as_ref();

            let temp_dir = std::env::temp_dir();
            let temp_output =
                temp_dir.join(format!("beefcake_export_python_{}.parquet", Uuid::new_v4()));
            temp_files.add(temp_output.clone());

            let load_snippet = if let Some(p) = path {
                format!(
                    "data_path = r\"{}\"\n{}",
                    p,
                    python_load_snippet("data_path", "df")
                )
            } else {
                String::new()
            };

            let wrapped_script = format!(
                r#"{}
{}
# User script start
{}
# User script end

try:
    target_lf = None
    if 'df' in locals():
        val = locals()['df']
        if isinstance(val, pl.LazyFrame): target_lf = val
        elif isinstance(val, pl.DataFrame): target_lf = val.lazy()
    
    if target_lf is None:
        # Try to find any LazyFrame or DataFrame in locals
        for name, val in locals().items():
            if name == 'pl': continue
            if isinstance(val, pl.LazyFrame):
                target_lf = val
                break
            elif isinstance(val, pl.DataFrame):
                target_lf = val.lazy()
                break
    
    if target_lf is None:
        print("Error: No Polars DataFrame or LazyFrame found in script (use variable 'df')")
        sys.exit(1)

{}
except Exception as e:
    print(f"Python Export Error: {{e}}")
    sys.exit(1)
"#,
                python_preamble(),
                load_snippet,
                script,
                python_adaptive_sink_snippet("target_lf", &temp_output)
            );

            execute_python(&wrapped_script, None, "Export (Python)").await?;
            LazyFrame::scan_parquet(temp_output, Default::default())
                .context("Failed to scan Python result")
        }
    }
}

pub async fn execute_export_destination(
    options: &ExportOptions,
    mut lf: LazyFrame,
    temp_files: &mut beefcake::utils::TempFileCollection,
) -> Result<()> {
    lf = lf.with_streaming(true);

    match options.destination.dest_type {
        ExportDestinationType::File => {
            let final_path = PathBuf::from(&options.destination.target);
            let ext = final_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_export_{}.{}", Uuid::new_v4(), ext));
            temp_files.add(temp_path.clone());

            beefcake::utils::log_event(
                "Export",
                &format!("Sinking data to temporary file: {}", temp_path.display()),
            );

            match ext.as_str() {
                "parquet" => {
                    let options = beefcake::analyser::logic::get_parquet_write_options(&lf)
                        .context("Failed to determine Parquet options")?;
                    lf.sink_parquet(&temp_path, options, None)
                        .context("Parquet export failed")?;
                }
                "csv" => {
                    lf.sink_csv(&temp_path, Default::default(), None)
                        .context("CSV export failed")?;
                }
                _ => {
                    let mut df = lf.collect().context("Export failed (collect)")?;
                    beefcake::analyser::logic::save_df(&mut df, &temp_path)
                        .context("Failed to save file")?;
                }
            }

            // Move temp file to final destination
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if let Err(e) = std::fs::rename(&temp_path, &final_path) {
                std::fs::copy(&temp_path, &final_path)
                    .with_context(|| format!("Failed to move file (Rename error: {e})"))?;
                let _ = std::fs::remove_file(&temp_path);
            }

            beefcake::utils::log_event(
                "Export",
                &format!("Successfully exported to {}", final_path.display()),
            );
            Ok(())
        }
        ExportDestinationType::Database => {
            let connection_id = options.destination.target.clone();

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_db_push_{}.csv", Uuid::new_v4()));
            temp_files.add(temp_path.clone());

            beefcake::utils::log_event("Export", "Sinking to temp CSV for database push...");

            lf.sink_csv(&temp_path, Default::default(), None)
                .context("Failed to prepare database push")?;

            // Resolve connection and call flow
            let config = beefcake::utils::load_app_config();
            let conn = config
                .settings
                .connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| BeefcakeError::Database("Connection not found".to_owned()))?;

            let url = conn.settings.connection_string(&connection_id);
            let opts = sqlx::postgres::PgConnectOptions::from_str(&url)
                .context("Invalid connection URL")?;

            beefcake::analyser::logic::flows::push_to_db_flow(
                temp_path,
                opts,
                conn.settings.schema.clone(),
                conn.settings.table.clone(),
                options.configs.clone(),
            )
            .await
            .map_err(BeefcakeError::from)
        }
    }
}

pub async fn export_data_execution(
    options: ExportOptions,
    temp_files: &mut beefcake::utils::TempFileCollection,
) -> Result<()> {
    if beefcake::utils::is_aborted() {
        return Err(BeefcakeError::Aborted);
    }

    beefcake::utils::log_event(
        "Export",
        &format!(
            "Starting export: Source={:?}, Dest={:?}",
            options.source.source_type, options.destination.dest_type
        ),
    );

    // 1. Get the LazyFrame based on source
    beefcake::utils::log_event("Export", "Step 1/3: Preparing data source (streaming)...");
    let mut lf = prepare_export_source(&options.source, temp_files).await?;

    // 2. Apply cleaning/transformation logic
    if !options.configs.is_empty() {
        beefcake::utils::log_event(
            "Export",
            "Step 2/3: Applying optimized streaming cleaning pipeline...",
        );
        lf = beefcake::analyser::logic::clean_df_lazy(lf, &options.configs, false)
            .context("Failed to apply cleaning")?;

        if beefcake::utils::is_aborted() {
            return Err(BeefcakeError::Aborted);
        }
    }

    // 3. Write to destination
    execute_export_destination(&options, lf, temp_files).await?;

    if beefcake::utils::is_aborted() {
        return Err(BeefcakeError::Aborted);
    }

    // 4. Create data dictionary snapshot if requested and destination is a file
    if options.create_dictionary
        && matches!(options.destination.dest_type, ExportDestinationType::File)
        && let Err(e) = create_dictionary_snapshot(&options).await
    {
        beefcake::utils::log_event(
            "Export",
            &format!("Warning: Failed to create data dictionary: {e}"),
        );
        // Don't fail the export if dictionary creation fails
    }

    Ok(())
}

/// Create a data dictionary snapshot for the exported dataset.
async fn create_dictionary_snapshot(options: &ExportOptions) -> Result<()> {
    beefcake::utils::log_event("Export", "Creating data dictionary snapshot...");

    // Get input path
    let input_path = match &options.source.path {
        Some(p) => PathBuf::from(p),
        None => return Ok(()), // Skip if no input path available
    };

    // Get output path
    let output_path = PathBuf::from(&options.destination.target);

    // Load the exported file to analyze it
    let dummy_progress = Arc::new(AtomicU64::new(0));
    let df = match beefcake::analyser::logic::load_df(&output_path, &dummy_progress) {
        Ok(df) => df,
        Err(_) => {
            beefcake::utils::log_event(
                "Export",
                "Could not load exported file for dictionary analysis",
            );
            return Ok(());
        }
    };

    // Determine dataset name from output filename
    let dataset_name = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("exported_dataset")
        .to_owned();

    // Create snapshot
    let snapshot = beefcake::dictionary::create_snapshot(
        &dataset_name,
        &df,
        input_path,
        output_path.clone(),
        None, // TODO: Could pass pipeline JSON if available
        None, // No previous snapshot for now
    )?;

    // Save snapshot to dictionaries folder (in data/ directory or alongside export)
    let dict_base_path = if let Some(parent) = output_path.parent() {
        parent.join("data")
    } else {
        PathBuf::from("data")
    };

    let snapshot_path = beefcake::dictionary::save_snapshot(&snapshot, &dict_base_path)?;

    beefcake::utils::log_event(
        "Export",
        &format!("Data dictionary saved: {}", snapshot_path.display()),
    );

    // Also export as Markdown
    let markdown = beefcake::dictionary::render_markdown(&snapshot)?;
    let md_path = output_path.with_extension("md");
    std::fs::write(&md_path, markdown)
        .with_context(|| format!("Failed to write markdown dictionary: {}", md_path.display()))?;

    beefcake::utils::log_event(
        "Export",
        &format!("Data dictionary markdown: {}", md_path.display()),
    );

    Ok(())
}
