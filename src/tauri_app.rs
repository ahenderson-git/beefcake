#![allow(clippy::let_underscore_must_use, clippy::let_underscore_untyped, clippy::print_stderr, clippy::exit, clippy::collapsible_if)]
use beefcake::analyser::logic::flows::analyze_file_flow;
use beefcake::analyser::logic::{AnalysisResponse, ColumnCleanConfig};
use beefcake::utils::{load_app_config, push_audit_log, save_app_config, AppConfig, DbSettings};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::python_runner;
use crate::export;

async fn run_on_worker_thread<F, Fut, R>(name_str: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<R, String>> + Send + 'static,
    R: Send + 'static,
{
    let name = name_str.to_string();
    let name_outer = name.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let name_for_thread = name.clone();
        let name_for_err = name.clone();
        let name_for_join = name.clone();

        let builder = std::thread::Builder::new()
            .name(name)
            .stack_size(50 * 1024 * 1024);

        let handle = builder
            .spawn(move || {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    tauri::async_runtime::block_on(f())
                }));

                match res {
                    Ok(r) => r,
                    Err(_) => Err(format!("{name_for_thread} thread panicked.")),
                }
            })
            .map_err(|e| format!("Failed to spawn {name_for_err} thread: {e}"))?;

        handle.join().map_err(|_| format!("{name_for_join} thread joined with error (panic)"))?
    })
    .await
    .map_err(|e| format!("{name_outer} task execution failed: {e}"))?
}

#[tauri::command]
pub async fn analyze_file(path: String, trim_pct: Option<f64>) -> Result<AnalysisResponse, String> {
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

    beefcake::utils::reset_abort_signal();

    analyze_file_flow(path_buf, trim_pct)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String, String> {
    beefcake::utils::log_event("File", &format!("Read file: {path}"));
    crate::system::read_text_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    beefcake::utils::log_event("File", &format!("Saved file: {path}"));
    crate::system::write_text_file(&path, &contents).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    Ok(beefcake::utils::load_app_config())
}

#[tauri::command]
pub async fn abort_processing() -> Result<(), String> {
    beefcake::utils::log_event("App", "User triggered abort signal");
    beefcake::utils::trigger_abort();
    Ok(())
}

#[tauri::command]
pub async fn reset_abort_signal() -> Result<(), String> {
    beefcake::utils::reset_abort_signal();
    Ok(())
}

#[tauri::command]
pub async fn save_config(mut config: AppConfig) -> Result<(), String> {
    use beefcake::utils::{set_db_password, KEYRING_PLACEHOLDER};
    use secrecy::ExposeSecret as _;

    for conn in &mut config.connections {
        let pwd = conn.settings.password.expose_secret();
        if pwd != KEYRING_PLACEHOLDER && !pwd.is_empty() {
            set_db_password(&conn.id, pwd).map_err(|e| e.to_string())?;
        }
    }

    if !config.audit_log.is_empty() {
        beefcake::utils::push_audit_log(&mut config, "Config", "Updated application settings");
    }
    beefcake::utils::save_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_powershell(script: String) -> Result<String, String> {
    beefcake::utils::log_event("PowerShell", "Executed script");
    crate::system::run_powershell(&script).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_data(options: export::ExportOptions) -> Result<(), String> {
    use beefcake::analyser::logic::types::ImputeMode;

    beefcake::utils::reset_abort_signal();

    // Memory safeguard logic
    let mut high_mem_ops = 0;
    for config in options.configs.values() {
        if config.active && config.ml_preprocessing {
            if config.impute_mode == ImputeMode::Median || config.impute_mode == ImputeMode::Mode {
                high_mem_ops += 1;
            }
            if config.clip_outliers {
                high_mem_ops += 1;
            }
        }
    }

    if high_mem_ops > 0 {
        if let export::ExportSourceType::Analyser = options.source.source_type {
            if let Some(path) = &options.source.path {
                if let Ok(meta) = std::fs::metadata(path) {
                    if meta.len() > 50 * 1024 * 1024 {
                        beefcake::utils::log_event(
                            "Export",
                            &format!(
                                "Warning: {} memory-intensive operations selected for a large file ({}). This may cause OOM.",
                                high_mem_ops,
                                beefcake::utils::fmt_bytes(meta.len())
                            ),
                        );
                    }
                }
            }
        }
    }

    run_on_worker_thread("export-worker", move || async move {
        let mut temp_files = Vec::new();
        let res = export::export_data_execution(options, &mut temp_files).await;

        if let Err(e) = &res {
            beefcake::utils::log_event("Export", &format!("Export failed: {e}"));
        }

        for path in temp_files {
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
        res
    })
    .await
}

#[tauri::command]
pub async fn run_python(
    script: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    beefcake::utils::log_event("Python", "Executing Python script.");

    let actual_data_path = python_runner::prepare_data(data_path.clone(), configs, "Python").await?;
    let res = python_runner::execute_python(&script, actual_data_path.clone(), "Python").await;

    if let (Some(actual), Some(original)) = (&actual_data_path, &data_path) {
        if actual != original {
            let _ = std::fs::remove_file(actual);
        }
    }
    res
}

#[tauri::command]
pub async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    beefcake::utils::log_event("SQL", "Executing SQL query.");

    let actual_data_path = python_runner::prepare_data(data_path.clone(), configs, "SQL").await?;

    let python_script = format!(
        r#"{}
data_path = os.environ.get("BEEFCAKE_DATA_PATH")
if not data_path:
    print("Error: No data path provided in environment variable BEEFCAKE_DATA_PATH")
    sys.exit(1)

try:
{}
    
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
        python_runner::python_preamble(),
        python_runner::python_load_snippet("data_path", "df"),
        query.replace(r#"""#, r#"\"""#)
    );

    let res = python_runner::execute_python(&python_script, actual_data_path.clone(), "SQL").await;

    if let (Some(actual), Some(original)) = (&actual_data_path, &data_path) {
        if actual != original {
            let _ = std::fs::remove_file(actual);
        }
    }
    res
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
            .connections
            .iter()
            .find(|c| c.id == connection_id)
            .ok_or_else(|| "Connection not found".to_owned())?;
        (conn.name.clone(), conn.settings.table.clone(), conn.settings.schema.clone())
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
    let opts = PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    beefcake::analyser::logic::flows::push_to_db_flow(
        PathBuf::from(path),
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
    settings: DbSettings,
    connection_id: Option<String>,
) -> Result<String, String> {
    use beefcake::analyser::db::DbClient;
    use sqlx::postgres::PgConnectOptions;

    let url = settings.connection_string(&connection_id.unwrap_or_default());
    let opts = PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    match DbClient::connect(opts).await {
        Ok(_) => Ok("Connection successful!".to_owned()),
        Err(e) => Err(format!("Connection failed: {e}")),
    }
}

#[tauri::command]
pub async fn delete_connection(id: String) -> Result<(), String> {
    let _ = beefcake::utils::delete_db_password(&id);
    Ok(())
}

#[tauri::command]
pub async fn install_python_package(package: String) -> Result<String, String> {
    beefcake::utils::log_event("Python", &format!("Installing package: {package}"));
    crate::system::install_python_package(&package).map_err(|e| e.to_string())
}

pub fn run() {
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
            abort_processing,
            reset_abort_signal,
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
