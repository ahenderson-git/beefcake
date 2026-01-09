#![allow(clippy::let_underscore_must_use, clippy::let_underscore_untyped, clippy::print_stderr, clippy::exit, clippy::collapsible_if)]
use beefcake::analyser::logic::{AnalysisResponse, ColumnCleanConfig};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[tauri::command]
async fn analyze_file(path: String, trim_pct: Option<f64>) -> Result<AnalysisResponse, String> {
    if path.is_empty() {
        return Err("File path is empty".to_owned());
    }

    let mut path_buf = PathBuf::from(&path);
    if path_buf.is_relative() {
        if let Ok(abs_path) = std::env::current_dir() {
            path_buf = abs_path.join(path_buf);
        }
    }
    
    let path_str = path_buf.to_string_lossy().to_string();
    beefcake::utils::log_event("Analyser", &format!("Started analysis of {path_str}"));

    let progress = Arc::new(AtomicU64::new(0));
    let start = std::time::Instant::now();

    let file_size = std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0);

    // For very large files, we use lazy loading and sampling for the SUMMARY analysis to avoid OOM.
    // However, the actual processing (export/cleaning) will always use the full LazyFrame.
    let (df, is_sampled, total_rows) = if file_size > 100 * 1024 * 1024 { // > 100MB
        beefcake::utils::log_event("Analyser", "Large file detected, using sampling for summary analysis...");
        let lf = beefcake::analyser::logic::load_df_lazy(&path_buf)
            .map_err(|e| format!("Failed to load file lazily: {e}"))?;
        
        let total = lf.clone().select([polars::prelude::len()]).collect()
            .map_err(|e| format!("Failed to count rows: {e}"))?
            .column("len").map_err(|e| e.to_string())?
            .u32().map_err(|e| e.to_string())?
            .get(0).unwrap_or(0) as usize;

        // We use a larger sample (500k) for better statistical accuracy on 10M+ rows
        (lf.limit(500_000)
            .collect()
            .map_err(|e| format!("Failed to sample data for analysis: {e}"))?,
        true,
        total)
    } else {
        let df = beefcake::analyser::logic::load_df(&path_buf, &progress).map_err(|e| e.to_string())?;
        let total = df.height();
        (df, false, total)
    };

    let mut response = beefcake::analyser::logic::analysis::run_full_analysis(
        df,
        path_str,
        file_size,
        total_rows,
        trim_pct.unwrap_or(0.05),
        start,
    )
    .map_err(|e| e.to_string())?;

    // If sampled, adjust the reported row count to reflect the full file (estimate if necessary, or just label as sampled)
    if is_sampled {
        // Simple heuristic: if it's a CSV, we can estimate rows based on size vs sample size
        // but for now, let's just mark it clearly in the response.
        // We add a special entry to business_summary for the first column
        if let Some(first_col) = response.summary.get_mut(0) {
            first_col.business_summary.insert(0, format!("NOTE: This analysis is based on a sample of 500,000 rows due to large file size ({})", beefcake::utils::fmt_bytes(file_size)));
        }
    }
    
    Ok(response)
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    beefcake::utils::log_event("File", &format!("Read file: {path}"));
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    beefcake::utils::log_event("File", &format!("Saved file: {path}"));
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn get_config() -> Result<beefcake::utils::AppConfig, String> {
    Ok(beefcake::utils::load_app_config())
}

#[tauri::command]
async fn save_config(mut config: beefcake::utils::AppConfig) -> Result<(), String> {
    use beefcake::utils::{set_db_password, KEYRING_PLACEHOLDER};
    use secrecy::ExposeSecret as _;

    for conn in &mut config.connections {
        let pwd = conn.settings.password.expose_secret();
        if pwd != KEYRING_PLACEHOLDER && !pwd.is_empty() {
            set_db_password(&conn.id, pwd).map_err(|e| e.to_string())?;
        }
    }

    // If the audit log was just cleared (length is 0), don't immediately push a "Config updated" log
    // because that would make the log never look empty.
    if !config.audit_log.is_empty() {
        beefcake::utils::push_audit_log(&mut config, "Config", "Updated application settings");
    }
    beefcake::utils::save_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_powershell(script: String) -> Result<String, String> {
    use std::process::Command;

    beefcake::utils::log_event("PowerShell", "Executed script");

    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&script)
            .output()
    } else {
        Command::new("pwsh")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&script)
            .output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(format!("Error: {stdout}\n{stderr}"))
            }
        }
        Err(e) => Err(format!("Failed to execute powershell: {e}")),
    }
}

async fn prepare_data(
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
    log_tag: &str,
) -> Result<Option<String>, String> {
    if let (Some(path), Some(cfgs)) = (&data_path, &configs) {
        if !cfgs.is_empty() && !path.is_empty() {
            beefcake::utils::log_event(log_tag, "Applying cleaning configurations before execution (streaming)");
            
            let lf = beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
                .map_err(|e| format!("Failed to load data for cleaning: {e}"))?;

            let cleaned_lf = beefcake::analyser::logic::clean_df_lazy(lf, cfgs, false)
                .map_err(|e| format!("Failed to apply cleaning: {e}"))?;

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_cleaned_data_{}_{}.parquet", log_tag.to_lowercase(), Uuid::new_v4()));

            // Use adaptive sink_parquet for memory efficiency
            let options = beefcake::analyser::logic::get_parquet_write_options(&cleaned_lf)
                .map_err(|e| format!("Failed to determine Parquet options: {e}"))?;

            if let Some(rgs) = options.row_group_size {
                beefcake::utils::log_event(
                    log_tag,
                    &format!("Streaming to Parquet (adaptive). Row group size: {}", rgs),
                );
            }

            cleaned_lf.with_streaming(true)
                .sink_parquet(&temp_path, options, None)
                .map_err(|e| format!("Failed to save cleaned data to temp file: {e}"))?;

            return Ok(Some(temp_path.to_string_lossy().to_string()));
        }
    }
    Ok(data_path)
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
enum ExportSourceType {
    Analyser,
    Python,
    SQL,
}

#[derive(Debug, Deserialize, Clone)]
struct ExportSource {
    #[serde(rename = "type")]
    source_type: ExportSourceType,
    content: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
enum ExportDestinationType {
    File,
    Database,
}

#[derive(Debug, Deserialize, Clone)]
struct ExportDestination {
    #[serde(rename = "type")]
    dest_type: ExportDestinationType,
    target: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ExportOptions {
    source: ExportSource,
    configs: HashMap<String, ColumnCleanConfig>,
    destination: ExportDestination,
}

async fn export_data_internal(options: ExportOptions) -> Result<(), String> {
    beefcake::utils::log_event(
        "Export",
        &format!(
            "Starting export: Source={:?}, Dest={:?}",
            options.source.source_type, options.destination.dest_type
        ),
    );

    // 1. Get the LazyFrame based on source
    beefcake::utils::log_event("Export", "Step 1/3: Preparing data source (streaming)...");
    let mut lf = match options.source.source_type {
        ExportSourceType::Analyser => {
            let path = options
                .source
                .path
                .ok_or_else(|| "No path provided for Analyser source".to_string())?;
            beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
                .map_err(|e| format!("Failed to load data: {e}"))?
        }
        ExportSourceType::SQL => {
            let query = options
                .source
                .content
                .ok_or_else(|| "No query provided for SQL source".to_string())?;
            let path = options
                .source
                .path
                .ok_or_else(|| "No data path provided for SQL source".to_string())?;

            // Generate Python script to run SQL and sink result to temp Parquet
            let temp_dir = std::env::temp_dir();
            let temp_output = temp_dir.join(format!(
                "beefcake_export_sql_{}.parquet",
                Uuid::new_v4()
            ));

            let python_script = format!(
                r#"import os
import polars as pl
import sys

data_path = r"{}"
try:
    if data_path.endswith(".parquet"):
        lf = pl.scan_parquet(data_path)
    elif data_path.endswith(".json"):
        # standard JSON doesn't support lazy scanning, but we can read it
        lf = pl.read_json(data_path).lazy()
    else:
        lf = pl.scan_csv(data_path, try_parse_dates=True)
    
    ctx = pl.SQLContext()
    ctx.register("data", lf)
    
    query = """{}"""
    result = ctx.execute(query)
    
    # Adaptive row group sizing
    col_count = len(result.schema)
    rgs = 65536
    if col_count >= 200: rgs = 16384
    elif col_count >= 100: rgs = 32768
    
    env_rgs = os.environ.get('BEEFCAKE_PARQUET_ROW_GROUP_SIZE')
    if env_rgs: 
        try: rgs = int(env_rgs)
        except: pass

    # Use sink_parquet for memory efficiency
    result.sink_parquet(r"{}", row_group_size=rgs)
except Exception as e:
    print(f"SQL Export Error: {{e}}")
    sys.exit(1)
"#,
                path,
                query.replace(r#"""#, r#"\"""#),
                temp_output.to_string_lossy()
            );

            execute_python(&python_script, None, "SQL_Export").await?;

            let lf = beefcake::analyser::logic::load_df_lazy(&temp_output)
                .map_err(|e| format!("Failed to load SQL result: {e}"))?;

            // Note: We can't remove the file yet because lf might still need to read it when sinking.
            // We should ideally clean it up after the whole process.
            lf
        }
        ExportSourceType::Python => {
            let script = options
                .source
                .content
                .ok_or_else(|| "No script provided for Python source".to_string())?;
            let path = options.source.path;

            // Generate Python script to run user script and sink result to temp Parquet
            let temp_dir = std::env::temp_dir();
            let temp_output = temp_dir.join(format!(
                "beefcake_export_python_{}.parquet",
                Uuid::new_v4()
            ));

            let wrapped_script = format!(
                r#"import os
import polars as pl
import sys

# User script start
{}
# User script end

try:
    target_lf = None
    if 'df' in locals():
        val = locals()['df']
        if isinstance(val, pl.DataFrame):
            target_lf = val.lazy()
        elif isinstance(val, pl.LazyFrame):
            target_lf = val
            
    if target_lf is None:
        # Fallback: look for any DataFrame or LazyFrame in locals
        for val in reversed(list(locals().values())):
            if isinstance(val, pl.DataFrame):
                target_lf = val.lazy()
                break
            elif isinstance(val, pl.LazyFrame):
                target_lf = val
                break
                
    if target_lf is not None:
        # Adaptive row group sizing
        col_count = len(target_lf.schema)
        rgs = 65536
        if col_count >= 200: rgs = 16384
        elif col_count >= 100: rgs = 32768
        
        env_rgs = os.environ.get('BEEFCAKE_PARQUET_ROW_GROUP_SIZE')
        if env_rgs: 
            try: rgs = int(env_rgs)
            except: pass
            
        target_lf.sink_parquet(r"{}", row_group_size=rgs)
    else:
        print("Error: No Polars DataFrame or LazyFrame found to export. Please ensure your script creates a variable named 'df'.")
        sys.exit(1)
except Exception as e:
    print(f"Python Export Error: {{e}}")
    sys.exit(1)
"#,
                script,
                temp_output.to_string_lossy()
            );

            execute_python(&wrapped_script, path, "Python_Export").await?;

            let lf = beefcake::analyser::logic::load_df_lazy(&temp_output)
                .map_err(|e| format!("Failed to load Python result: {e}"))?;

            lf
        }
    };

    // 2. Apply cleaning configs (Lazy)
    if !options.configs.is_empty() {
        beefcake::utils::log_event("Export", "Step 2/3: Applying optimized streaming cleaning pipeline...");
        lf = beefcake::analyser::logic::clean_df_lazy(lf, &options.configs, false)
            .map_err(|e| format!("Failed to apply cleaning: {e}"))?;
        beefcake::utils::log_event("Export", "Cleaning pipeline prepared.");
    }

    // 3. Export to destination
    beefcake::utils::log_event("Export", "Step 3/3: Writing to destination (streaming)...");
    match options.destination.dest_type {
        ExportDestinationType::File => {
            let final_path = PathBuf::from(options.destination.target);
            let ext = final_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Write to a temporary file in the system temp directory first.
            // This is a critical fix for cloud-sync folders like OneDrive, which may attempt to
            // sync or lock the file while Polars is still streaming data to it, causing crashes.
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_export_{}.{}", Uuid::new_v4(), ext));

            beefcake::utils::log_event("Export", &format!("Sinking data to temporary file: {:?}", temp_path));
            
            match ext.as_str() {
                "parquet" => {
                    let options = beefcake::analyser::logic::get_parquet_write_options(&lf)
                        .map_err(|e| e.to_string())?;

                    if let Some(rgs) = options.row_group_size {
                        beefcake::utils::log_event(
                            "Export",
                            &format!("Starting Parquet sink (adaptive). Row group size: {}", rgs),
                        );
                    }

                    lf.with_streaming(true)
                        .sink_parquet(&temp_path, options, None)
                        .map_err(|e| format!("Failed to sink Parquet: {e}"))?;
                }
                "csv" => {
                    beefcake::utils::log_event("Export", "Starting CSV sink (optimized)...");
                    lf.with_streaming(true)
                        .sink_csv(&temp_path, Default::default(), None)
                        .map_err(|e| format!("Failed to sink CSV: {e}"))?;
                }
                _ => {
                    // Fallback to collect for formats that don't support sink (like JSON)
                    beefcake::utils::log_event("Export", &format!("Collecting data for {} export...", ext));
                    let mut df = lf.with_streaming(true)
                        .collect()
                        .map_err(|e| format!("Failed to collect for export: {e}"))?;
                    beefcake::analyser::logic::save_df(&mut df, &temp_path)
                        .map_err(|e| format!("Failed to save file: {e}"))?;
                }
            }

            // Move temp file to final destination
            beefcake::utils::log_event("Export", &format!("Finalizing: Moving file to {:?}", final_path));
            if let Err(e) = std::fs::rename(&temp_path, &final_path) {
                // Fallback: try copy + remove if rename fails (e.g. cross-device move)
                std::fs::copy(&temp_path, &final_path)
                    .map_err(|err| format!("Failed to move file to destination: {err} (Rename also failed: {e})"))?;
                let _ = std::fs::remove_file(&temp_path);
            }

            beefcake::utils::log_event("Export", &format!("Successfully exported to {:?}", final_path));
        }
        ExportDestinationType::Database => {
            use beefcake::analyser::db::DbClient;
            use beefcake::utils::{load_app_config, push_audit_log, save_app_config};
            use sqlx::postgres::PgConnectOptions;
            use std::str::FromStr as _;

            let connection_id = options.destination.target;
            let mut config = load_app_config();

            let conn = config
                .connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| "Connection not found".to_string())?;

            let url = conn.settings.connection_string(&connection_id);
            let opts = PgConnectOptions::from_str(&url)
                .map_err(|e| format!("Invalid connection URL: {e}"))?;

            let client = DbClient::connect(opts)
                .await
                .map_err(|e| format!("Database connection failed: {e}"))?;

            let conn_name = conn.name.clone();
            let table_name = conn.settings.table.clone();
            let schema_name = conn.settings.schema.clone();

            push_audit_log(
                &mut config,
                "Export",
                &format!("Exporting data to database {conn_name}.{table_name}"),
            );
            let _ = save_app_config(&config).ok();

            // Use streaming sink for database push to avoid OOM
            let schema = lf.collect_schema().map_err(|e| e.to_string())?;
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("beefcake_db_push_{}.csv", Uuid::new_v4()));

            beefcake::utils::log_event("Export", "Sinking to temp CSV for database push (streaming)...");
            lf.with_streaming(true)
                .sink_csv(&temp_path, Default::default(), None)
                .map_err(|e| format!("Failed to sink to CSV for DB push: {e}"))?;

            client
                .push_from_csv_file(&temp_path, &schema, Some(schema_name.as_str()), Some(table_name.as_str()))
                .await
                .map_err(|e| format!("Database push failed: {e}"))?;

            let _ = std::fs::remove_file(&temp_path);
        }
    }

    Ok(())
}

#[tauri::command]
async fn export_data(options: ExportOptions) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let builder = std::thread::Builder::new()
            .name("export-worker".to_string())
            .stack_size(50 * 1024 * 1024); // 50MB stack to be super safe

        builder
            .spawn(move || {
                let res = std::panic::catch_unwind(move || {
                    beefcake::utils::TOKIO_RUNTIME.block_on(export_data_internal(options))
                });

                let final_res = match res {
                    Ok(r) => r,
                    Err(_) => Err("Export thread panicked. This can happen with extremely complex data plans or low memory. Try disabling some cleaning options.".to_string()),
                };
                let _ = tx.send(final_res);
            })
            .map_err(|e| format!("Failed to spawn export thread: {e}"))?;

        rx.recv()
            .map_err(|e| format!("Export thread communication failed: {e}"))?
    })
    .await
    .map_err(|e| format!("Export task execution failed: {e}"))?
}

async fn execute_python(
    script: &str,
    data_path: Option<String>,
    log_tag: &str,
) -> Result<String, String> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("POLARS_FMT_MAX_COLS", "-1");
    cmd.env("POLARS_FMT_MAX_ROWS", "100");
    cmd.env("POLARS_FMT_STR_LEN", "1000");

    if let Some(path) = &data_path {
        if !path.is_empty() {
            beefcake::utils::log_event(log_tag, &format!("Setting BEEFCAKE_DATA_PATH to: {}", path));
            cmd.env("BEEFCAKE_DATA_PATH", path);
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python for {log_tag}: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    stdin
        .write_all(script.as_bytes())
        .map_err(|e| format!("Failed to write to stdin: {e}"))?;
    drop(stdin);

    let out = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for python: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Error: {stdout}\n{stderr}"))
    }
}

#[tauri::command]
async fn run_python(
    script: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    beefcake::utils::log_event(
        "Python",
        &format!(
            "Executing Python script. Argument data_path: {:?}, configs: {}",
            data_path,
            configs.as_ref().map(|c| c.len()).unwrap_or(0)
        ),
    );

    let actual_data_path = prepare_data(data_path, configs, "Python").await?;
    execute_python(&script, actual_data_path, "Python").await
}

#[tauri::command]
async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    beefcake::utils::log_event(
        "SQL",
        &format!(
            "Executing SQL query. Argument data_path: {:?}, configs: {}",
            data_path,
            configs.as_ref().map(|c| c.len()).unwrap_or(0)
        ),
    );

    let actual_data_path = prepare_data(data_path, configs, "SQL").await?;

    let python_script = format!(
        r#"import os
import polars as pl
import sys

# Disable column truncation
pl.Config.set_tbl_cols(-1)
pl.Config.set_tbl_rows(100)

data_path = os.environ.get("BEEFCAKE_DATA_PATH")
if not data_path:
    print("Error: No data path provided in environment variable BEEFCAKE_DATA_PATH")
    sys.exit(1)

try:
    if data_path.endswith(".parquet"):
        df = pl.scan_parquet(data_path)
    elif data_path.endswith(".json"):
        df = pl.read_json(data_path).lazy()
    else:
        df = pl.scan_csv(data_path, try_parse_dates=True)
    
    ctx = pl.SQLContext()
    ctx.register("data", df)
    
    query = """{}"""
    result = ctx.execute(query)
    # Limit for preview to avoid OOM on large datasets
    print(result.limit(100).collect())
except Exception as e:
    print(f"SQL Error: {{e}}")
    sys.exit(1)
"#,
        query.replace(r#"""#, r#"\"""#)
    );

    execute_python(&python_script, actual_data_path, "SQL").await
}

#[tauri::command]
async fn sanitize_headers(names: Vec<String>) -> Result<Vec<String>, String> {
    Ok(beefcake::analyser::logic::sanitize_column_names(&names))
}

async fn push_to_db_internal(
    path: String,
    connection_id: String,
    configs: std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>,
) -> Result<(), String> {
    use beefcake::analyser::db::DbClient;
    use beefcake::utils::{load_app_config, push_audit_log, save_app_config};
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr as _;

    let mut config = load_app_config();
    let (conn_name, table_name) = {
        let conn = config
            .connections
            .iter()
            .find(|c| c.id == connection_id)
            .ok_or_else(|| "Connection not found".to_owned())?;
        (conn.name.clone(), conn.settings.table.clone())
    };

    push_audit_log(
        &mut config,
        "Database",
        &format!("Pushing data to {conn_name} ({table_name})"),
    );
    let _ = save_app_config(&config).ok();

    let conn = config
        .connections
        .iter()
        .find(|c| c.id == connection_id)
        .ok_or_else(|| "Connection not found".to_owned())?;

    let url = conn.settings.connection_string(&connection_id);
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    let client = DbClient::connect(opts)
        .await
        .map_err(|e| format!("Database connection failed: {e}"))?;

    let lf = beefcake::analyser::logic::load_df_lazy(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to load data: {e}"))?;

    // Apply cleaning configurations from Analyser
    let mut cleaned_lf =
        beefcake::analyser::logic::clean_df_lazy(lf, &configs, false).map_err(|e| format!("Cleaning failed: {e}"))?;

    let schema = cleaned_lf.collect_schema().map_err(|e| e.to_string())?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("beefcake_db_push_{}.csv", Uuid::new_v4()));

    beefcake::utils::log_event("Database", "Sinking to temp CSV for database push (streaming)...");
    cleaned_lf.with_streaming(true)
        .sink_csv(&temp_path, Default::default(), None)
        .map_err(|e| format!("Failed to sink to CSV for DB push: {e}"))?;

    client
        .push_from_csv_file(
            &temp_path,
            &schema,
            Some(&conn.settings.schema),
            Some(&conn.settings.table),
        )
        .await
        .map_err(|e| format!("Data push failed: {e}"))?;

    let _ = std::fs::remove_file(&temp_path);
    Ok(())
}

#[tauri::command]
async fn push_to_db(
    path: String,
    connection_id: String,
    configs: std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let builder = std::thread::Builder::new()
            .name("db-push-worker".to_string())
            .stack_size(50 * 1024 * 1024); // 50MB stack

        builder
            .spawn(move || {
                let res = std::panic::catch_unwind(move || {
                    beefcake::utils::TOKIO_RUNTIME
                        .block_on(push_to_db_internal(path, connection_id, configs))
                });

                let final_res = match res {
                    Ok(r) => r,
                    Err(_) => Err("Database push thread panicked.".to_string()),
                };
                let _ = tx.send(final_res);
            })
            .map_err(|e| format!("Failed to spawn db-push thread: {e}"))?;

        rx.recv()
            .map_err(|e| format!("Db-push thread communication failed: {e}"))?
    })
    .await
    .map_err(|e| format!("Db-push task execution failed: {e}"))?
}

#[tauri::command]
async fn test_connection(
    settings: beefcake::utils::DbSettings,
    connection_id: Option<String>,
) -> Result<String, String> {
    use beefcake::analyser::db::DbClient;
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr as _;

    let url = settings.connection_string(&connection_id.unwrap_or_default());
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    match DbClient::connect(opts).await {
        Ok(_) => Ok("Connection successful!".to_owned()),
        Err(e) => Err(format!("Connection failed: {e}")),
    }
}

#[tauri::command]
async fn delete_connection(id: String) -> Result<(), String> {
    let _ = beefcake::utils::delete_db_password(&id);
    Ok(())
}

#[tauri::command]
async fn install_python_package(package: String) -> Result<String, String> {
    use std::process::Command;

    beefcake::utils::log_event("Python", &format!("Installing package: {package}"));

    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    let output = cmd
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg(&package)
        .env("PYTHONIOENCODING", "utf-8")
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(format!("Successfully installed {package}\n{stdout}"))
            } else {
                Err(format!("Failed to install {package}: {stdout}\n{stderr}"))
            }
        }
        Err(e) => Err(format!("Failed to execute pip: {e}")),
    }
}

pub fn run() {
    #[cfg(debug_assertions)]
    {
        use std::net::TcpStream;
        use std::time::Duration;
        let addr = "127.0.0.1:14206";
        if let Ok(socket_addr) = addr.parse() {
            if TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500)).is_err() {
                eprintln!("\n\x1b[1;33m[WARNING] Dev server not detected at {addr}.\x1b[0m");
                eprintln!(
                    "\x1b[1;33m[WARNING] Did you forget to run 'npm run tauri dev' or 'cargo tauri dev'?\x1b[0m"
                );
                eprintln!(
                    "\x1b[1;33m[WARNING] Running via 'cargo run' will result in a 'refused to connect' error in the window.\n\x1b[0m"
                );
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            analyze_file,
            run_powershell,
            run_python,
            read_text_file,
            write_text_file,
            get_config,
            save_config,
            push_to_db,
            test_connection,
            delete_connection,
            install_python_package,
            run_sql,
            sanitize_headers,
            export_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
