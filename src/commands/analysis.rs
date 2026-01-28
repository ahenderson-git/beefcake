use beefcake::analyser::logic::flows::analyze_file_flow;
use beefcake::analyser::logic::{AnalysisResponse, ColumnCleanConfig};
use beefcake::config::{load_app_config, push_audit_log, save_app_config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr as _;

use super::system::{ensure_security_acknowledged, run_on_worker_thread};
use crate::python_runner;

#[tauri::command]
pub async fn analyze_file(path: String) -> Result<AnalysisResponse, String> {
    tracing::info!("analyze_file command called with path: {}", path);

    if path.is_empty() {
        tracing::error!("analyze_file failed: path is empty");
        return Err("File path is empty".to_owned());
    }

    let mut path_buf = PathBuf::from(&path);
    if path_buf.is_relative()
        && let Ok(abs_path) = std::env::current_dir()
    {
        path_buf = abs_path.join(path_buf);
    }

    let path_str = path_buf.to_string_lossy().to_string();
    tracing::info!("Analyzing file: {}", path_str);
    beefcake::config::log_event("Analyser", &format!("Started analysis of {path_str}"));

    beefcake::utils::reset_abort_signal();

    match analyze_file_flow(path_buf).await {
        Ok(response) => {
            tracing::info!(
                "File analysis completed successfully: {} rows, {} columns",
                response.row_count,
                response.column_count
            );
            Ok(response)
        }
        Err(e) => {
            tracing::error!("File analysis failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn abort_processing() -> Result<(), String> {
    beefcake::config::log_event("App", "User triggered abort signal");
    beefcake::utils::trigger_abort();
    Ok(())
}

#[tauri::command]
pub async fn reset_abort_signal() -> Result<(), String> {
    beefcake::utils::reset_abort_signal();
    Ok(())
}

#[tauri::command]
pub async fn run_powershell(script: String) -> Result<String, String> {
    tracing::info!(
        "run_powershell command called, script length: {} chars",
        script.len()
    );
    ensure_security_acknowledged()?;
    beefcake::config::log_event("PowerShell", "Executed script");

    match crate::system::run_powershell(&script) {
        Ok(output) => {
            tracing::info!(
                "PowerShell execution completed successfully. Output length: {} chars",
                output.len()
            );
            Ok(output)
        }
        Err(e) => {
            tracing::error!(
                "PowerShell execution failed after {} chars script: {}",
                script.len(),
                e
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn run_python(
    script: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    tracing::info!(
        "run_python command called, script length: {} chars, data_path: {:?}",
        script.len(),
        data_path
    );
    ensure_security_acknowledged()?;
    beefcake::config::log_event("Python", "Executed script");

    let (actual_data_path, _temp_guard) = python_runner::prepare_data(data_path, configs, "Python")
        .await
        .map_err(|e| {
            tracing::error!("Failed to prepare Python data: {}", e);
            String::from(e)
        })?;

    tracing::info!("Python data preparation complete. Executing script...");

    // _temp_guard will automatically clean up the temp file when dropped
    match python_runner::execute_python(&script, actual_data_path, "Python").await {
        Ok(output) => {
            tracing::info!(
                "Python execution completed successfully. Output length: {} chars",
                output.len()
            );
            Ok(output)
        }
        Err(e) => {
            tracing::error!("Python execution failed: {}", e);
            Err(String::from(e))
        }
    }
}

#[tauri::command]
pub async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    tracing::info!(
        "run_sql command called, query length: {} chars, data_path: {:?}",
        query.len(),
        data_path
    );
    ensure_security_acknowledged()?;
    beefcake::config::log_event("Sql", "Executing Sql query.");

    let (actual_data_path, _temp_guard) = python_runner::prepare_data(data_path, configs, "Sql")
        .await
        .map_err(String::from)?;

    tracing::info!("Sql data preparation complete. Generating Python bridge script...");

    // Generate the load snippet and indent it properly for the try block
    let load_snippet = python_runner::python_load_snippet("data_path", "df");
    let indented_load = load_snippet
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_owned()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let python_script = format!(
        r#"{}
data_path = os.environ.get("BEEFCAKE_DATA_PATH")
if not data_path:
    print("Error: No data path provided in environment variable BEEFCAKE_DATA_PATH")
    sys.exit(1)

query_str = os.environ.get("BEEFCAKE_SQL_QUERY")
if not query_str:
    print("Error: No Sql query provided in environment variable BEEFCAKE_SQL_QUERY")
    sys.exit(1)

try:
{}
    # Execute Sql query
    ctx = pl.SQLContext()
    ctx.register("data", df)

    result = ctx.execute(query_str)
    # Limit for preview to avoid OOM on large datasets
    result_df = result.limit(100).collect()

    # Print the result - Polars will format it nicely
    print(result_df)
except Exception as e:
    print(f"Sql Error: {{e}}")
    sys.exit(1)
"#,
        python_runner::python_preamble(),
        indented_load
    );

    // _temp_guard will automatically clean up the temp file when dropped
    match python_runner::execute_python_with_env(
        &python_script,
        actual_data_path,
        Some(query),
        "Sql",
    )
    .await
    {
        Ok(output) => {
            tracing::info!(
                "Sql execution completed successfully. Output length: {} chars",
                output.len()
            );
            Ok(output)
        }
        Err(e) => {
            tracing::error!("Sql execution failed: {}", e);
            Err(String::from(e))
        }
    }
}

#[tauri::command]
pub async fn sanitize_headers(names: Vec<String>) -> Result<Vec<String>, String> {
    Ok(beefcake::analyser::logic::sanitize_column_names(&names))
}

pub async fn push_to_db_internal(
    path: String,
    connection_id: String,
    configs: HashMap<String, ColumnCleanConfig>,
) -> Result<(), String> {
    use sqlx::postgres::PgConnectOptions;

    let mut config = load_app_config();
    let (conn_name, table_name, schema_name) = {
        let conn = config
            .settings
            .connections
            .iter()
            .find(|c| c.id == connection_id)
            .ok_or_else(|| "Connection not found".to_owned())?;
        (
            conn.name.clone(),
            conn.settings.table.clone(),
            conn.settings.schema.clone(),
        )
    };

    push_audit_log(
        &mut config,
        "Database",
        &format!("Pushing data to {conn_name} ({table_name})"),
    );
    let _ = save_app_config(&config).ok();

    let conn = config
        .settings
        .connections
        .iter()
        .find(|c| c.id == connection_id)
        .ok_or_else(|| "Connection not found".to_owned())?;

    let url = conn.settings.connection_string(&connection_id);
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    beefcake::analyser::logic::flows::push_to_db_flow(
        path.into(),
        opts,
        schema_name,
        table_name,
        configs,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_to_db(
    path: String,
    connection_id: String,
    configs: HashMap<String, ColumnCleanConfig>,
) -> Result<(), String> {
    run_on_worker_thread("db-push-worker", move || async move {
        push_to_db_internal(path, connection_id, configs).await
    })
    .await
}

#[tauri::command]
pub async fn test_connection(
    settings: beefcake::config::DbSettings,
    connection_id: Option<String>,
) -> Result<String, String> {
    use secrecy::ExposeSecret as _;
    let pwd = settings.password.expose_secret();

    let actual_pwd = if pwd == beefcake::config::KEYRING_PLACEHOLDER
        && let Some(id) = connection_id
        && let Some(saved_pwd) = beefcake::utils::get_db_password(&id)
    {
        saved_pwd
    } else {
        pwd.to_owned()
    };

    beefcake::analyser::logic::flows::test_connection_flow(settings, actual_pwd)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_connection(id: String) -> Result<(), String> {
    let mut config = load_app_config();
    config.settings.connections.retain(|c| c.id != id);
    let _ = beefcake::utils::delete_db_password(&id);
    save_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_python_package(package: String) -> Result<String, String> {
    crate::system::install_python_package(&package).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_python_environment() -> Result<String, String> {
    crate::system::check_python_environment().map_err(|e| e.to_string())
}
