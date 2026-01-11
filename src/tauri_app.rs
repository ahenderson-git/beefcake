#![allow(
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::print_stderr,
    clippy::exit,
    clippy::collapsible_if
)]
use beefcake::analyser::logic::flows::analyze_file_flow;
use beefcake::analyser::logic::{AnalysisResponse, ColumnCleanConfig};
use beefcake::utils::{AppConfig, DbSettings, load_app_config, push_audit_log, save_app_config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::export;
use crate::python_runner;

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

        handle
            .join()
            .map_err(|_| format!("{name_for_join} thread joined with error (panic)"))?
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
    use beefcake::utils::{KEYRING_PLACEHOLDER, set_db_password};
    use secrecy::ExposeSecret as _;

    for conn in &mut config.settings.connections {
        let pwd = conn.settings.password.expose_secret();
        if pwd != KEYRING_PLACEHOLDER && !pwd.is_empty() {
            set_db_password(&conn.id, pwd).map_err(|e| e.to_string())?;
        }
    }

    if !config.audit_log().is_empty() {
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
        let mut temp_files = beefcake::utils::TempFileCollection::new();
        let res = export::export_data_execution(options, &mut temp_files).await;

        if let Err(e) = &res {
            beefcake::utils::log_event("Export", &format!("Export failed: {e}"));
        }

        // temp_files will be automatically cleaned up when it goes out of scope
        res.map_err(String::from)
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

    let (actual_data_path, _temp_guard) =
        python_runner::prepare_data(data_path, configs, "Python").await.map_err(String::from)?;
    let res = python_runner::execute_python(&script, actual_data_path, "Python").await.map_err(String::from);

    // _temp_guard will automatically clean up the temp file when dropped
    res
}

#[tauri::command]
pub async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    beefcake::utils::log_event("SQL", "Executing SQL query.");

    let (actual_data_path, _temp_guard) = python_runner::prepare_data(data_path, configs, "SQL").await.map_err(String::from)?;

    // Generate the load snippet and indent it properly for the try block
    let load_snippet = python_runner::python_load_snippet("data_path", "df");
    let indented_load = load_snippet
        .lines()
        .map(|line| if line.is_empty() { line.to_string() } else { format!("    {}", line) })
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
    print("Error: No SQL query provided in environment variable BEEFCAKE_SQL_QUERY")
    sys.exit(1)

try:
{}
    # Execute SQL query
    ctx = pl.SQLContext()
    ctx.register("data", df)

    result = ctx.execute(query_str)
    # Limit for preview to avoid OOM on large datasets
    result_df = result.limit(100).collect()

    # Print the result - Polars will format it nicely
    print(result_df)
except Exception as e:
    print(f"SQL Error: {{e}}")
    sys.exit(1)
"#,
        python_runner::python_preamble(),
        indented_load
    );

    let res = python_runner::execute_python_with_env(&python_script, actual_data_path, Some(query), "SQL").await.map_err(String::from);

    // _temp_guard will automatically clean up the temp file when dropped
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
            .settings.connections
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
        .settings.connections
        .iter()
        .find(|c| c.id == connection_id)
        .ok_or_else(|| "Connection not found".to_owned())?;

    let url = conn.settings.connection_string(&connection_id);
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

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
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

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

#[tauri::command]
pub async fn check_python_environment() -> Result<String, String> {
    beefcake::utils::log_event("System", "Checking Python environment");
    crate::system::check_python_environment().map_err(|e| e.to_string())
}

// ============================================================================
// Dataset Lifecycle Commands
// ============================================================================

use beefcake::analyser::lifecycle::{
    DatasetRegistry, DiffSummary, LifecycleStage, PublishMode, TransformPipeline,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

// Global registry instance
lazy_static::lazy_static! {
    static ref LIFECYCLE_REGISTRY: Arc<RwLock<Option<DatasetRegistry>>> = Arc::new(RwLock::new(None));
}

fn get_or_create_registry() -> Result<DatasetRegistry, String> {
    let mut reg_guard = LIFECYCLE_REGISTRY.write()
        .map_err(|e| format!("Lock poisoned: {e}"))?;

    if reg_guard.is_none() {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| "Could not find data directory".to_string())?
            .join("beefcake")
            .join("datasets");

        let registry = DatasetRegistry::new(data_dir)
            .map_err(|e| format!("Failed to create registry: {e}"))?;

        *reg_guard = Some(registry);
    }

    Ok(reg_guard.clone().unwrap())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub async fn lifecycle_create_dataset(request: CreateDatasetRequest) -> Result<String, String> {
    beefcake::utils::log_event("Lifecycle", &format!("Creating dataset: {}", request.name));

    let registry = get_or_create_registry()?;
    let path_buf = PathBuf::from(&request.path);

    let dataset_id = registry.create_dataset(request.name, path_buf)
        .map_err(|e| format!("Failed to create dataset: {e}"))?;

    Ok(dataset_id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyTransformsRequest {
    pub dataset_id: String,
    pub pipeline_json: String,
    pub stage: String,
}

#[tauri::command]
pub async fn lifecycle_apply_transforms(request: ApplyTransformsRequest) -> Result<String, String> {
    beefcake::utils::log_event("Lifecycle", "Applying transforms");

    let registry = get_or_create_registry()?;
    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| format!("Invalid dataset ID: {e}"))?;

    let pipeline = TransformPipeline::from_json(&request.pipeline_json)
        .map_err(|e| format!("Failed to parse pipeline: {e}"))?;

    let stage = LifecycleStage::from_str(&request.stage)
        .ok_or_else(|| format!("Invalid stage: {}", request.stage))?;

    let version_id = registry.apply_transforms(&dataset_id, pipeline, stage)
        .map_err(|e| format!("Failed to apply transforms: {e}"))?;

    Ok(version_id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetActiveVersionRequest {
    pub dataset_id: String,
    pub version_id: String,
}

#[tauri::command]
pub async fn lifecycle_set_active_version(request: SetActiveVersionRequest) -> Result<(), String> {
    beefcake::utils::log_event("Lifecycle", "Setting active version");

    let registry = get_or_create_registry()?;
    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version_id = Uuid::parse_str(&request.version_id)
        .map_err(|e| format!("Invalid version ID: {e}"))?;

    registry.set_active_version(&dataset_id, &version_id)
        .map_err(|e| format!("Failed to set active version: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishVersionRequest {
    pub dataset_id: String,
    pub version_id: String,
    pub mode: String, // "view" or "snapshot"
}

#[tauri::command]
pub async fn lifecycle_publish_version(request: PublishVersionRequest) -> Result<String, String> {
    beefcake::utils::log_event("Lifecycle", &format!("Publishing version as {}", request.mode));

    let registry = get_or_create_registry()?;
    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version_id = Uuid::parse_str(&request.version_id)
        .map_err(|e| format!("Invalid version ID: {e}"))?;

    let mode = match request.mode.to_lowercase().as_str() {
        "view" => PublishMode::View,
        "snapshot" => PublishMode::Snapshot,
        _ => return Err(format!("Invalid publish mode: {}", request.mode)),
    };

    let published_id = registry.publish_version(&dataset_id, &version_id, mode)
        .map_err(|e| format!("Failed to publish version: {e}"))?;

    Ok(published_id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetVersionDiffRequest {
    pub dataset_id: String,
    pub version1_id: String,
    pub version2_id: String,
}

#[tauri::command]
pub async fn lifecycle_get_version_diff(request: GetVersionDiffRequest) -> Result<DiffSummary, String> {
    beefcake::utils::log_event("Lifecycle", "Computing version diff");

    let registry = get_or_create_registry()?;
    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version1_id = Uuid::parse_str(&request.version1_id)
        .map_err(|e| format!("Invalid version1 ID: {e}"))?;
    let version2_id = Uuid::parse_str(&request.version2_id)
        .map_err(|e| format!("Invalid version2 ID: {e}"))?;

    registry.compute_diff(&dataset_id, &version1_id, &version2_id)
        .map_err(|e| format!("Failed to compute diff: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListVersionsRequest {
    pub dataset_id: String,
}

#[tauri::command]
pub async fn lifecycle_list_versions(request: ListVersionsRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| format!("Invalid dataset ID: {e}"))?;

    let versions = registry.list_versions(&dataset_id)
        .map_err(|e| format!("Failed to list versions: {e}"))?;

    serde_json::to_string_pretty(&versions)
        .map_err(|e| format!("Failed to serialize versions: {e}"))
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
            check_python_environment,
            run_sql,
            sanitize_headers,
            export_data,
            lifecycle_create_dataset,
            lifecycle_apply_transforms,
            lifecycle_set_active_version,
            lifecycle_publish_version,
            lifecycle_get_version_diff,
            lifecycle_list_versions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
