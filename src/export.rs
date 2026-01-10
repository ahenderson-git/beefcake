use crate::python_runner::{
    execute_python, python_adaptive_sink_snippet, python_load_snippet, python_preamble,
};
use beefcake::analyser::logic::ColumnCleanConfig;
use polars::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr as _;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum ExportSourceType {
    Analyser,
    Python,
    SQL,
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
}

pub async fn prepare_export_source(
    source: &ExportSource,
    temp_files: &mut Vec<PathBuf>,
) -> Result<LazyFrame, String> {
    match source.source_type {
        ExportSourceType::Analyser => {
            let path = source
                .path
                .as_ref()
                .ok_or_else(|| "No path provided for Analyser source".to_string())?;
            beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
                .map_err(|e| format!("Failed to load data: {e}"))
        }
        ExportSourceType::SQL => {
            let query = source
                .content
                .as_ref()
                .ok_or_else(|| "No query provided for SQL source".to_string())?;
            let path = source
                .path
                .as_ref()
                .ok_or_else(|| "No data path provided for SQL source".to_string())?;

            let temp_dir = std::env::temp_dir();
            let temp_output =
                temp_dir.join(format!("beefcake_export_sql_{}.parquet", Uuid::new_v4()));
            temp_files.push(temp_output.clone());

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
    print(f"SQL Export Error: {{e}}")
    sys.exit(1)
"#,
                python_preamble(),
                path,
                python_load_snippet("data_path", "lf"),
                query.replace(r#"""#, r#"\"""#),
                python_adaptive_sink_snippet("result", &temp_output)
            );

            execute_python(&python_script, None, "Export (SQL)").await?;
            LazyFrame::scan_parquet(temp_output, Default::default())
                .map_err(|e| format!("Failed to scan SQL result: {e}"))
        }
        ExportSourceType::Python => {
            let script = source
                .content
                .as_ref()
                .ok_or_else(|| "No script provided for Python source".to_string())?;
            let path = source.path.as_ref();

            let temp_dir = std::env::temp_dir();
            let temp_output =
                temp_dir.join(format!("beefcake_export_python_{}.parquet", Uuid::new_v4()));
            temp_files.push(temp_output.clone());

            let load_snippet = if let Some(p) = path {
                format!(
                    "data_path = r\"{}\"\n{}",
                    p,
                    python_load_snippet("data_path", "df")
                )
            } else {
                "".to_string()
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
                .map_err(|e| format!("Failed to scan Python result: {e}"))
        }
    }
}

pub async fn execute_export_destination(
    options: &ExportOptions,
    mut lf: LazyFrame,
    temp_files: &mut Vec<PathBuf>,
) -> Result<(), String> {
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
            temp_files.push(temp_path.clone());

            beefcake::utils::log_event(
                "Export",
                &format!("Sinking data to temporary file: {:?}", temp_path),
            );

            match ext.as_str() {
                "parquet" => {
                    let options = beefcake::analyser::logic::get_parquet_write_options(&lf)
                        .map_err(|e| format!("Failed to determine Parquet options: {e}"))?;
                    lf.sink_parquet(&temp_path, options, None)
                        .map_err(|e| format!("Parquet export failed: {e}"))?;
                }
                "csv" => {
                    lf.sink_csv(&temp_path, Default::default(), None)
                        .map_err(|e| format!("CSV export failed: {e}"))?;
                }
                _ => {
                    let mut df = lf
                        .collect()
                        .map_err(|e| format!("Export failed (collect): {e}"))?;
                    beefcake::analyser::logic::save_df(&mut df, &temp_path)
                        .map_err(|e| format!("Failed to save file: {e}"))?;
                }
            }

            // Move temp file to final destination
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            if let Err(e) = std::fs::rename(&temp_path, &final_path) {
                std::fs::copy(&temp_path, &final_path)
                    .map_err(|err| format!("Failed to move file: {err} (Rename error: {e})"))?;
                let _ = std::fs::remove_file(&temp_path);
            }

            beefcake::utils::log_event(
                "Export",
                &format!("Successfully exported to {:?}", final_path),
            );
            Ok(())
        }
        ExportDestinationType::Database => {
            let connection_id = options.destination.target.clone();

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_db_push_{}.csv", Uuid::new_v4()));
            temp_files.push(temp_path.clone());

            beefcake::utils::log_event("Export", "Sinking to temp CSV for database push...");

            lf.sink_csv(&temp_path, Default::default(), None)
                .map_err(|e| format!("Failed to prepare database push: {e}"))?;

            // Resolve connection and call flow
            let config = beefcake::utils::load_app_config();
            let conn = config
                .connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| "Connection not found".to_string())?;

            let url = conn.settings.connection_string(&connection_id);
            let opts = sqlx::postgres::PgConnectOptions::from_str(&url)
                .map_err(|e| format!("Invalid connection URL: {e}"))?;

            beefcake::analyser::logic::flows::push_to_db_flow(
                temp_path,
                opts,
                conn.settings.schema.clone(),
                conn.settings.table.clone(),
                options.configs.clone(),
            )
            .await
            .map_err(|e| e.to_string())
        }
    }
}

pub async fn export_data_execution(
    options: ExportOptions,
    temp_files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if beefcake::utils::is_aborted() {
        return Err("Operation aborted by user".to_string());
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
            .map_err(|e| format!("Failed to apply cleaning: {e}"))?;

        if beefcake::utils::is_aborted() {
            return Err("Operation aborted by user".to_string());
        }
    }

    // 3. Write to destination
    execute_export_destination(&options, lf, temp_files).await?;

    if beefcake::utils::is_aborted() {
        return Err("Operation aborted by user".to_string());
    }

    Ok(())
}
