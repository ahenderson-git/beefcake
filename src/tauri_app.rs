#![allow(
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::print_stderr,
    clippy::exit,
    clippy::collapsible_if
)]
use beefcake::ai::client::AIAssistant;
use beefcake::analyser::logic::flows::analyze_file_flow;
use beefcake::analyser::logic::{AnalysisResponse, ColumnCleanConfig};
use beefcake::utils::AIConfig;
use beefcake::utils::{AppConfig, DbSettings, load_app_config, push_audit_log, save_app_config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr as _;
use tauri::Manager as _;

use crate::export;
use crate::python_runner;

// ============================================================================
// Constants
// ============================================================================

/// Stack size for worker threads (50MB) - used for memory-intensive operations
const WORKER_THREAD_STACK_SIZE: usize = 50 * 1024 * 1024;

/// File size threshold (50MB) for warning about memory-intensive operations
const LARGE_FILE_WARNING_THRESHOLD: u64 = 50 * 1024 * 1024;

fn ensure_security_acknowledged() -> Result<(), String> {
    let config = load_app_config();
    if config.settings.security_warning_acknowledged {
        Ok(())
    } else {
        Err("Security warning not acknowledged. Please confirm before running scripts.".to_owned())
    }
}

async fn run_on_worker_thread<F, Fut, R>(name: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
    R: Send + 'static,
{
    let thread_name = name.to_owned();
    let spawn_name = thread_name.clone();
    let panic_name = thread_name.clone();

    tauri::async_runtime::spawn_blocking(move || {
        std::thread::Builder::new()
            .name(thread_name)
            .stack_size(WORKER_THREAD_STACK_SIZE)
            .spawn(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    tauri::async_runtime::block_on(f())
                }))
                .unwrap_or_else(|_| Err(format!("{panic_name} panicked")))
            })
            .map_err(|e| format!("Failed to spawn {spawn_name}: {e}"))?
            .join()
            .map_err(|e| format!("Thread join error: {e:?}"))?
    })
    .await
    .map_err(|e| format!("Worker task failed: {e}"))?
}

#[tauri::command]
pub async fn analyze_file(path: String) -> Result<AnalysisResponse, String> {
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

    analyze_file_flow(path_buf).await.map_err(|e| e.to_string())
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
    Ok(env!("CARGO_PKG_VERSION").to_owned())
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    Ok(load_app_config())
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
        push_audit_log(&mut config, "Config", "Updated application settings");
    }
    save_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_powershell(script: String) -> Result<String, String> {
    ensure_security_acknowledged()?;
    beefcake::utils::log_event("PowerShell", "Executed script");
    crate::system::run_powershell(&script).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_data(options: export::ExportOptions) -> Result<(), String> {
    use beefcake::analyser::logic::types::ImputeMode;

    beefcake::utils::reset_abort_signal();

    // Memory safeguard logic
    let mut high_mem_ops = 0;
    #[expect(clippy::iter_over_hash_type)]
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
        if matches!(
            options.source.source_type,
            export::ExportSourceType::Analyser
        ) {
            if let Some(path) = &options.source.path {
                if let Ok(meta) = std::fs::metadata(path) {
                    if meta.len() > LARGE_FILE_WARNING_THRESHOLD {
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
    ensure_security_acknowledged()?;
    beefcake::utils::log_event("Python", "Executing Python script.");

    let (actual_data_path, _temp_guard) = python_runner::prepare_data(data_path, configs, "Python")
        .await
        .map_err(String::from)?;

    // _temp_guard will automatically clean up the temp file when dropped
    python_runner::execute_python(&script, actual_data_path, "Python")
        .await
        .map_err(String::from)
}

#[tauri::command]
pub async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
) -> Result<String, String> {
    ensure_security_acknowledged()?;
    beefcake::utils::log_event("Sql", "Executing Sql query.");

    let (actual_data_path, _temp_guard) = python_runner::prepare_data(data_path, configs, "Sql")
        .await
        .map_err(String::from)?;

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
    python_runner::execute_python_with_env(&python_script, actual_data_path, Some(query), "Sql")
        .await
        .map_err(String::from)
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
    ensure_security_acknowledged()?;
    beefcake::utils::log_event("Python", &format!("Installing package: {package}"));
    crate::system::install_python_package(&package).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_python_environment() -> Result<String, String> {
    beefcake::utils::log_event("System", "Checking Python environment");
    crate::system::check_python_environment().map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct StandardPathsPayload {
    pub base_dir: String,
    pub input_dir: String,
    pub output_dir: String,
    pub scripts_dir: String,
    pub logs_dir: String,
    pub templates_dir: String,
}

#[tauri::command]
pub async fn get_standard_paths() -> Result<StandardPathsPayload, String> {
    let paths = beefcake::utils::standard_paths();
    Ok(StandardPathsPayload {
        base_dir: paths.base_dir.to_string_lossy().to_string(),
        input_dir: paths.input_dir.to_string_lossy().to_string(),
        output_dir: paths.output_dir.to_string_lossy().to_string(),
        scripts_dir: paths.scripts_dir.to_string_lossy().to_string(),
        logs_dir: paths.logs_dir.to_string_lossy().to_string(),
        templates_dir: paths.templates_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    crate::system::open_path(&path).map_err(|e| e.to_string())
}

// ============================================================================
// Dataset Lifecycle Commands
// ============================================================================

use beefcake::analyser::lifecycle::{
    DatasetRegistry, DiffSummary, LifecycleStage, PublishMode, TransformPipeline,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock, RwLock};
use uuid::Uuid;

// Global registry instance
static LIFECYCLE_REGISTRY: LazyLock<Arc<RwLock<Option<Arc<DatasetRegistry>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

fn get_or_create_registry() -> Result<Arc<DatasetRegistry>, String> {
    // First, try to get the existing registry with a read lock (non-blocking for concurrent access)
    {
        let reg_guard = LIFECYCLE_REGISTRY
            .read()
            .map_err(|e| format!("Lock poisoned: {e}"))?;

        if let Some(registry) = reg_guard.as_ref() {
            return Ok(Arc::clone(registry));
        }
    }

    // If not initialised, acquire a write lock to initialise
    let mut reg_guard = LIFECYCLE_REGISTRY
        .write()
        .map_err(|e| format!("Lock poisoned: {e}"))?;

    // Double-check in case another thread initialised while we waited
    if let Some(registry) = reg_guard.as_ref() {
        return Ok(Arc::clone(registry));
    }

    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not find data directory".to_owned())?
        .join("beefcake")
        .join("datasets");

    let registry = Arc::new(
        DatasetRegistry::new(data_dir).map_err(|e| format!("Failed to create registry: {e}"))?,
    );

    *reg_guard = Some(Arc::clone(&registry));

    Ok(registry)
}

fn normalize_trusted_root(path: &str) -> Result<String, String> {
    let raw = PathBuf::from(path);
    let absolute = if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to resolve current directory: {e}"))?
            .join(raw)
    };

    let candidate = if absolute.is_dir() {
        absolute
    } else {
        absolute
            .parent()
            .ok_or_else(|| "Failed to resolve parent directory".to_owned())?
            .to_path_buf()
    };

    let normalized = candidate.canonicalize().unwrap_or(candidate);
    Ok(normalized.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn list_trusted_paths() -> Result<Vec<String>, String> {
    Ok(load_app_config().settings.trusted_paths)
}

#[tauri::command]
pub async fn add_trusted_path(path: String) -> Result<Vec<String>, String> {
    let mut config = load_app_config();
    let normalized = normalize_trusted_root(&path)?;
    if !config.settings.trusted_paths.contains(&normalized) {
        config.settings.trusted_paths.push(normalized);
        save_app_config(&config).map_err(|e| e.to_string())?;
    }
    Ok(config.settings.trusted_paths)
}

#[tauri::command]
pub async fn remove_trusted_path(path: String) -> Result<Vec<String>, String> {
    let mut config = load_app_config();
    let normalized = normalize_trusted_root(&path)?;
    config.settings.trusted_paths.retain(|p| p != &normalized);
    save_app_config(&config).map_err(|e| e.to_string())?;
    Ok(config.settings.trusted_paths)
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

    let dataset_id = registry
        .create_dataset(request.name, path_buf)
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
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;

    let pipeline = TransformPipeline::from_json(&request.pipeline_json)
        .map_err(|e| format!("Failed to parse pipeline: {e}"))?;

    let stage = LifecycleStage::parse_stage(&request.stage)
        .ok_or_else(|| format!("Invalid stage: {}", request.stage))?;

    let version_id = registry
        .apply_transforms(&dataset_id, pipeline, stage)
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
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version_id =
        Uuid::parse_str(&request.version_id).map_err(|e| format!("Invalid version ID: {e}"))?;

    registry
        .set_active_version(&dataset_id, &version_id)
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
    beefcake::utils::log_event(
        "Lifecycle",
        &format!("Publishing version as {}", request.mode),
    );

    let registry = get_or_create_registry()?;
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version_id =
        Uuid::parse_str(&request.version_id).map_err(|e| format!("Invalid version ID: {e}"))?;

    let mode = match request.mode.to_lowercase().as_str() {
        "view" => PublishMode::View,
        "snapshot" => PublishMode::Snapshot,
        _ => return Err(format!("Invalid publish mode: {}", request.mode)),
    };

    let published_id = registry
        .publish_version(&dataset_id, &version_id, mode)
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
pub async fn lifecycle_get_version_diff(
    request: GetVersionDiffRequest,
) -> Result<DiffSummary, String> {
    beefcake::utils::log_event("Lifecycle", "Computing version diff");

    let registry = get_or_create_registry()?;
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version1_id =
        Uuid::parse_str(&request.version1_id).map_err(|e| format!("Invalid version1 ID: {e}"))?;
    let version2_id =
        Uuid::parse_str(&request.version2_id).map_err(|e| format!("Invalid version2 ID: {e}"))?;

    registry
        .compute_diff(&dataset_id, &version1_id, &version2_id)
        .map_err(|e| format!("Failed to compute diff: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListVersionsRequest {
    pub dataset_id: String,
}

#[tauri::command]
pub async fn lifecycle_list_versions(request: ListVersionsRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;

    let versions = registry
        .list_versions(&dataset_id)
        .map_err(|e| format!("Failed to list versions: {e}"))?;

    serde_json::to_string_pretty(&versions)
        .map_err(|e| format!("Failed to serialize versions: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetVersionSchemaRequest {
    pub dataset_id: String,
    pub version_id: String,
}

#[tauri::command]
pub async fn lifecycle_get_version_schema(
    request: GetVersionSchemaRequest,
) -> Result<Vec<ColumnInfo>, String> {
    beefcake::utils::log_event("Lifecycle", "Getting version schema");

    let registry = get_or_create_registry()?;
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("Invalid dataset ID: {e}"))?;
    let version_id =
        Uuid::parse_str(&request.version_id).map_err(|e| format!("Invalid version ID: {e}"))?;

    // Get the dataset and version
    let dataset = registry
        .get_dataset(&dataset_id)
        .map_err(|e| format!("Failed to get dataset: {e}"))?;
    let version = dataset
        .get_version(&version_id)
        .map_err(|e| format!("Failed to get version: {e}"))?;

    // Load the version data to get schema
    let mut lf = version
        .load_data(&dataset.store)
        .map_err(|e| format!("Failed to load version data: {e}"))?;

    // Collect schema information
    let schema = lf
        .collect_schema()
        .map_err(|e| format!("Failed to get schema: {e}"))?;

    let columns: Vec<ColumnInfo> = schema
        .iter()
        .map(|(name, dtype)| ColumnInfo {
            name: name.as_str().to_owned(),
            dtype: dtype.to_string(),
        })
        .collect();

    Ok(columns)
}

// ============================================================================
// Pipeline Automation Commands
// ============================================================================

#[tauri::command]
pub async fn save_pipeline_spec(spec_json: String, path: String) -> Result<(), String> {
    use beefcake::pipeline::PipelineSpec;

    beefcake::utils::log_event("Pipeline", &format!("Saving spec to: {path}"));

    // Parse spec to validate
    let spec =
        PipelineSpec::from_json(&spec_json).map_err(|e| format!("Invalid pipeline spec: {e}"))?;

    // Ensure directory exists
    let path_buf = PathBuf::from(&path);
    if let Some(parent) = path_buf.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Write to a file
    spec.to_file(&path_buf)
        .map_err(|e| format!("Failed to save pipeline spec: {e}"))
}

#[tauri::command]
pub async fn load_pipeline_spec(path: String) -> Result<String, String> {
    use beefcake::pipeline::PipelineSpec;

    beefcake::utils::log_event("Pipeline", &format!("Loading spec from: {path}"));

    let spec =
        PipelineSpec::from_file(&path).map_err(|e| format!("Failed to load pipeline spec: {e}"))?;

    spec.to_json()
        .map_err(|e| format!("Failed to serialize pipeline spec: {e}"))
}

#[tauri::command]
pub async fn validate_pipeline_spec(
    spec_json: String,
    input_path: String,
) -> Result<Vec<String>, String> {
    use beefcake::analyser::logic::load_df_lazy;
    use beefcake::pipeline::{PipelineSpec, validate_pipeline};

    beefcake::utils::log_event("Pipeline", "Validating pipeline spec");

    // Parse spec
    let spec = PipelineSpec::from_json(&spec_json)
        .map_err(|e| format!("Invalid pipeline spec JSON: {e}"))?;

    // Load the input file to get schema
    let mut lf = load_df_lazy(std::path::Path::new(&input_path))
        .map_err(|e| format!("Failed to load input file: {e}"))?;

    let schema = lf
        .collect_schema()
        .map_err(|e| format!("Failed to collect schema: {e}"))?;

    // Validate
    let errors = validate_pipeline(&spec, &schema).map_err(|e| format!("Validation error: {e}"))?;

    Ok(errors.iter().map(|e| e.to_string()).collect())
}

#[tauri::command]
pub async fn generate_powershell(spec_json: String, output_path: String) -> Result<String, String> {
    use beefcake::pipeline::{PipelineSpec, generate_powershell_script};

    beefcake::utils::log_event(
        "Pipeline",
        &format!("Generating PowerShell to: {output_path}"),
    );

    // Parse spec
    let spec =
        PipelineSpec::from_json(&spec_json).map_err(|e| format!("Invalid pipeline spec: {e}"))?;

    // Determine a spec path (adjacent to ps1 file)
    let ps1_path = PathBuf::from(&output_path);
    let spec_path = ps1_path.with_extension("json");

    // Generate PowerShell script
    let script = generate_powershell_script(&spec, &spec_path);

    // Ensure directory exists
    if let Some(parent) = ps1_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Write a script file
    std::fs::write(&ps1_path, &script)
        .map_err(|e| format!("Failed to write PowerShell script: {e}"))?;

    // Also write a spec file alongside
    spec.to_file(&spec_path)
        .map_err(|e| format!("Failed to write spec file: {e}"))?;

    Ok(format!(
        "Generated:\n  - {}\n  - {}",
        ps1_path.display(),
        spec_path.display()
    ))
}

#[tauri::command]
pub async fn pipeline_from_configs(
    name: String,
    configs_json: String,
    input_format: String,
    output_path: String,
) -> Result<String, String> {
    use beefcake::analyser::logic::types::ColumnCleanConfig;
    use beefcake::pipeline::PipelineSpec;
    use std::collections::HashMap;

    beefcake::utils::log_event(
        "Pipeline",
        &format!("Creating pipeline from configs: {name}"),
    );

    // Parse configs
    let configs: HashMap<String, ColumnCleanConfig> =
        serde_json::from_str(&configs_json).map_err(|e| format!("Failed to parse configs: {e}"))?;

    // Generate pipeline spec
    let spec = PipelineSpec::from_clean_configs(name, &configs, &input_format, &output_path);

    // Serialize to JSON
    spec.to_json()
        .map_err(|e| format!("Failed to serialize pipeline: {e}"))
}

#[tauri::command]
pub async fn execute_pipeline_spec(
    spec_json: String,
    input_path: String,
    output_path: Option<String>,
) -> Result<String, String> {
    use beefcake::pipeline::{PipelineSpec, run_pipeline};

    beefcake::utils::log_event("Pipeline", &format!("Executing pipeline on: {input_path}"));

    // Parse spec
    let spec =
        PipelineSpec::from_json(&spec_json).map_err(|e| format!("Invalid pipeline spec: {e}"))?;

    // Execute pipeline
    let report = run_pipeline(&spec, &input_path, output_path.as_deref())
        .map_err(|e| format!("Pipeline execution failed: {e}"))?;

    // Return JSON report
    let result = serde_json::json!({
        "success": true,
        "rows_before": report.rows_before,
        "rows_after": report.rows_after,
        "columns_before": report.columns_before,
        "columns_after": report.columns_after,
        "steps_applied": report.steps_applied,
        "warnings": report.warnings,
        "duration_secs": report.duration.as_secs_f64(),
        "summary": report.summary()
    });

    serde_json::to_string(&result).map_err(|e| format!("Failed to serialize result: {e}"))
}

#[tauri::command]
pub async fn list_pipeline_specs() -> Result<String, String> {
    use std::fs;

    beefcake::utils::log_event("Pipeline", "Listing pipeline specs");

    // Get pipelines directory - use data/pipelines in current directory
    let pipelines_dir = PathBuf::from("data").join("pipelines");

    // Create directory if it doesn't exist
    if !pipelines_dir.exists() {
        fs::create_dir_all(&pipelines_dir)
            .map_err(|e| format!("Failed to create pipelines directory: {e}"))?;
    }

    // Scan for .json files
    let mut pipelines = Vec::new();

    if let Ok(entries) = fs::read_dir(&pipelines_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to load pipeline to get metadata
                if let Ok(spec) = beefcake::pipeline::PipelineSpec::from_file(&path) {
                    let metadata = entry.metadata().ok();
                    let created = metadata
                        .as_ref()
                        .and_then(|m| m.created().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default();
                    let modified = metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default();

                    pipelines.push(serde_json::json!({
                        "name": spec.name,
                        "path": path.to_string_lossy(),
                        "created": created,
                        "modified": modified,
                        "step_count": spec.steps.len()
                    }));
                }
            }
        }
    }

    serde_json::to_string(&pipelines).map_err(|e| format!("Failed to serialize pipeline list: {e}"))
}

#[tauri::command]
pub async fn delete_pipeline_spec(path: String) -> Result<(), String> {
    use std::fs;

    beefcake::utils::log_event("Pipeline", &format!("Deleting spec at: {path}"));

    let path_buf = PathBuf::from(&path);

    // Security check: ensure the path is within the pipelines directory
    let pipelines_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?
        .join("data")
        .join("pipelines");

    let absolute_path = if path_buf.is_absolute() {
        path_buf.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {e}"))?
            .join(path_buf)
    };

    if !absolute_path.starts_with(&pipelines_dir) {
        return Err("Access denied: Path is outside the pipelines directory".to_owned());
    }

    if !absolute_path.exists() {
        return Err("File not found".to_owned());
    }

    fs::remove_file(absolute_path).map_err(|e| format!("Failed to delete pipeline spec: {e}"))
}

#[tauri::command]
pub async fn list_pipeline_templates() -> Result<String, String> {
    use std::fs;

    beefcake::utils::log_event("Pipeline", "Listing pipeline templates");

    // Get templates directory
    let templates_dir = PathBuf::from("data").join("pipelines").join("templates");

    // Create directory if it doesn't exist
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)
            .map_err(|e| format!("Failed to create templates directory: {e}"))?;
    }

    // Scan for .json files
    let mut templates = Vec::new();

    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to load template to get metadata
                if let Ok(spec) = beefcake::pipeline::PipelineSpec::from_file(&path) {
                    let metadata = entry.metadata().ok();
                    let created = metadata
                        .as_ref()
                        .and_then(|m| m.created().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default();
                    let modified = metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default();

                    let info = serde_json::json!({
                        "name": spec.name,
                        "path": path.to_string_lossy(),
                        "created": created,
                        "modified": modified,
                        "step_count": spec.steps.len(),
                    });
                    templates.push(info);
                }
            }
        }
    }

    serde_json::to_string(&templates).map_err(|e| format!("Failed to serialize templates: {e}"))
}

#[tauri::command]
pub async fn load_pipeline_template(template_name: String) -> Result<String, String> {
    beefcake::utils::log_event("Pipeline", &format!("Loading template: {template_name}"));

    // Construct path to template
    let template_path = PathBuf::from("data")
        .join("pipelines")
        .join("templates")
        .join(format!(
            "{}.json",
            template_name.to_lowercase().replace(' ', "-")
        ));

    // Load template
    let spec = beefcake::pipeline::PipelineSpec::from_file(&template_path)
        .map_err(|e| format!("Failed to load template: {e}"))?;

    // Return as JSON
    spec.to_json()
        .map_err(|e| format!("Failed to serialize template: {e}"))
}

// ============================================================================
// Data Dictionary Commands
// ============================================================================

use beefcake::dictionary::{
    ColumnBusinessMetadata, DataDictionary, DatasetBusinessMetadata, storage::SnapshotMetadata,
};

#[tauri::command]
pub async fn dictionary_load_snapshot(snapshot_id: String) -> Result<DataDictionary, String> {
    beefcake::utils::log_event("Dictionary", &format!("Loading snapshot: {snapshot_id}"));

    let snapshot_uuid =
        Uuid::parse_str(&snapshot_id).map_err(|e| format!("Invalid snapshot ID: {e}"))?;

    let base_path = PathBuf::from("data");

    beefcake::dictionary::load_snapshot(&snapshot_uuid, &base_path)
        .map_err(|e| format!("Failed to load snapshot: {e}"))
}

#[tauri::command]
pub async fn dictionary_list_snapshots(
    dataset_hash: Option<String>,
) -> Result<Vec<SnapshotMetadata>, String> {
    beefcake::utils::log_event("Dictionary", "Listing snapshots");

    let base_path = PathBuf::from("data");

    beefcake::dictionary::list_snapshots(&base_path, dataset_hash.as_deref())
        .map_err(|e| format!("Failed to list snapshots: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBusinessMetadataRequest {
    pub snapshot_id: String,
    pub dataset_business: Option<DatasetBusinessMetadata>,
    pub column_business_updates: Option<HashMap<String, ColumnBusinessMetadata>>,
}

#[tauri::command]
pub async fn dictionary_update_business_metadata(
    request: UpdateBusinessMetadataRequest,
) -> Result<String, String> {
    beefcake::utils::log_event("Dictionary", "Updating business metadata");

    let snapshot_uuid =
        Uuid::parse_str(&request.snapshot_id).map_err(|e| format!("Invalid snapshot ID: {e}"))?;

    let base_path = PathBuf::from("data");

    let updated = beefcake::dictionary::storage::update_business_metadata(
        &snapshot_uuid,
        &base_path,
        request.dataset_business,
        request.column_business_updates,
    )
    .map_err(|e| format!("Failed to update business metadata: {e}"))?;

    Ok(updated.snapshot_id.to_string())
}

#[tauri::command]
pub async fn dictionary_export_markdown(
    snapshot_id: String,
    output_path: String,
) -> Result<(), String> {
    beefcake::utils::log_event("Dictionary", &format!("Exporting markdown: {snapshot_id}"));

    let snapshot_uuid =
        Uuid::parse_str(&snapshot_id).map_err(|e| format!("Invalid snapshot ID: {e}"))?;

    let base_path = PathBuf::from("data");

    // Load snapshot
    let snapshot = beefcake::dictionary::load_snapshot(&snapshot_uuid, &base_path)
        .map_err(|e| format!("Failed to load snapshot: {e}"))?;

    // Render markdown
    let markdown = beefcake::dictionary::render_markdown(&snapshot)
        .map_err(|e| format!("Failed to render markdown: {e}"))?;

    // Write to file
    std::fs::write(&output_path, markdown)
        .map_err(|e| format!("Failed to write markdown file: {e}"))?;

    beefcake::utils::log_event(
        "Dictionary",
        &format!("Markdown exported to: {output_path}"),
    );

    Ok(())
}

#[tauri::command]
pub async fn watcher_get_state() -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_start(
    folder: String,
) -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    let path = PathBuf::from(folder);
    beefcake::watcher::start(path).map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_stop() -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::watcher::stop().map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_set_folder(
    folder: String,
) -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    let path = PathBuf::from(folder);
    beefcake::watcher::set_folder(path).map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_ingest_now(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    beefcake::watcher::ingest_now(path_buf).map_err(|e| e.to_string())
}

/// Documentation file metadata for the frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct DocFileMetadata {
    /// Relative path from docs/ directory (e.g., "README.md", "typescript/PATTERNS.md")
    pub path: String,
    /// Display name for the UI
    pub title: String,
    /// Category for grouping (e.g., "Getting Started", "Reference", "Architecture")
    pub category: String,
}

#[tauri::command]
pub async fn list_documentation_files(
    app: tauri::AppHandle,
) -> Result<Vec<DocFileMetadata>, String> {
    let docs_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {e}"))?
        .join("docs");

    if !docs_dir.exists() {
        return Err(format!(
            "Documentation directory not found: {}",
            docs_dir.display()
        ));
    }

    let mut docs = Vec::new();

    // Manually list important documentation files with metadata
    let doc_files = vec![
        ("README.md", "Documentation Index", "Getting Started"),
        ("LEARNING_GUIDE.md", "Learning Guide", "Getting Started"),
        ("FEATURES.md", "Features Overview", "Reference"),
        ("HELPFUL_LINKS.md", "Helpful Links", "Reference"),
        ("LIMITATIONS.md", "Known Limitations", "Reference"),
        ("ARCHITECTURE.md", "System Architecture", "Architecture"),
        ("MODULES.md", "Module Reference", "Architecture"),
        ("RUST_CONCEPTS.md", "Rust Concepts", "Learning"),
        ("TYPESCRIPT_PATTERNS.md", "TypeScript Patterns", "Learning"),
        ("AUTOMATION.md", "Pipeline Automation", "Guide"),
        ("ROADMAP.md", "Development Roadmap", "Planning"),
        (
            "PIPELINE_BUILDER_SPEC.md",
            "Pipeline Builder Spec",
            "Reference",
        ),
        (
            "PIPELINE_IMPLEMENTATION_GUIDE.md",
            "Pipeline Implementation",
            "Guide",
        ),
        ("CODE_QUALITY.md", "Code Quality Standards", "Development"),
        ("testing.md", "Testing Guide", "Development"),
        ("test-matrix.md", "Test Matrix", "Development"),
    ];

    for (filename, title, category) in doc_files {
        let file_path = docs_dir.join(filename);
        if file_path.exists() {
            docs.push(DocFileMetadata {
                path: filename.to_owned(),
                title: title.to_owned(),
                category: category.to_owned(),
            });
        }
    }

    Ok(docs)
}

#[tauri::command]
pub async fn read_documentation_file(
    doc_path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::fs;

    // Security: only allow reading from docs/ directory
    if doc_path.contains("..")
        || doc_path.contains("\\..")
        || doc_path.starts_with('/')
        || doc_path.starts_with('\\')
    {
        return Err("Invalid documentation path: path traversal not allowed".to_owned());
    }

    let docs_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {e}"))?
        .join("docs");

    let file_path = docs_dir.join(&doc_path);

    // Verify the resolved path is still within docs directory (canonical path check)
    let docs_dir_canonical = docs_dir
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize docs dir: {e}"))?;

    let file_path_canonical = file_path
        .canonicalize()
        .map_err(|e| format!("Documentation file not found: {e}"))?;

    if !file_path_canonical.starts_with(&docs_dir_canonical) {
        return Err("Invalid documentation path: outside docs directory".to_owned());
    }

    // Read and return the file content
    fs::read_to_string(&file_path_canonical)
        .map_err(|e| format!("Failed to read documentation file: {e}"))
}

// ============================================================================
// AI Assistant Commands
// ============================================================================

#[tauri::command]
pub async fn ai_send_query(query: String, context: Option<String>) -> Result<String, String> {
    // Get API key from keyring
    let api_key = beefcake::utils::get_ai_api_key()
        .ok_or("AI API key not configured. Please set your OpenAI API key in settings.")?;

    // Get AI config from app settings
    let config = load_app_config();

    let ai_config = config.settings().ai_config.clone();

    if !ai_config.enabled {
        return Err("AI assistant is disabled. Please enable it in settings.".to_owned());
    }

    // Create AI assistant
    let assistant = AIAssistant::new(api_key, ai_config)
        .map_err(|e| format!("Failed to initialize AI assistant: {e}"))?;

    // Send query
    assistant
        .send_query(&query, context.as_deref())
        .await
        .map_err(|e| format!("AI query failed: {e}"))
}

#[tauri::command]
pub async fn ai_set_api_key(api_key: String) -> Result<(), String> {
    beefcake::utils::set_ai_api_key(&api_key).map_err(|e| format!("Failed to save API key: {e}"))
}

#[tauri::command]
pub async fn ai_delete_api_key() -> Result<(), String> {
    beefcake::utils::delete_ai_api_key().map_err(|e| format!("Failed to delete API key: {e}"))
}

#[tauri::command]
pub fn ai_has_api_key() -> bool {
    beefcake::utils::has_ai_api_key()
}

#[tauri::command]
pub async fn ai_test_connection() -> Result<(), String> {
    // Get API key from keyring
    let api_key = beefcake::utils::get_ai_api_key().ok_or("AI API key not configured")?;

    // Get AI config from app settings
    let config = load_app_config();

    let ai_config = config.settings().ai_config.clone();

    // Create AI assistant and test
    let assistant = AIAssistant::new(api_key, ai_config)
        .map_err(|e| format!("Failed to initialize AI assistant: {e}"))?;

    assistant
        .test_connection()
        .await
        .map_err(|e| format!("Connection test failed: {e}"))
}

#[tauri::command]
pub fn ai_get_config() -> AIConfig {
    let config = load_app_config();

    config.settings().ai_config.clone()
}

#[tauri::command]
pub async fn ai_update_config(ai_config: AIConfig) -> Result<(), String> {
    let mut config = load_app_config();

    config.settings_mut().ai_config = ai_config;

    save_app_config(&config).map_err(|e| format!("Failed to save config: {e}"))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
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
            get_standard_paths,
            open_path,
            run_sql,
            sanitize_headers,
            export_data,
            lifecycle_create_dataset,
            lifecycle_apply_transforms,
            lifecycle_set_active_version,
            lifecycle_publish_version,
            lifecycle_get_version_diff,
            lifecycle_list_versions,
            lifecycle_get_version_schema,
            save_pipeline_spec,
            load_pipeline_spec,
            validate_pipeline_spec,
            generate_powershell,
            pipeline_from_configs,
            execute_pipeline_spec,
            delete_pipeline_spec,
            list_pipeline_specs,
            list_pipeline_templates,
            load_pipeline_template,
            dictionary_load_snapshot,
            dictionary_list_snapshots,
            dictionary_update_business_metadata,
            dictionary_export_markdown,
            watcher_get_state,
            watcher_start,
            watcher_stop,
            watcher_set_folder,
            watcher_ingest_now,
            list_documentation_files,
            read_documentation_file,
            ai_send_query,
            ai_set_api_key,
            ai_delete_api_key,
            ai_has_api_key,
            ai_test_connection,
            ai_get_config,
            ai_update_config,
            list_trusted_paths,
            add_trusted_path,
            remove_trusted_path
        ])
        .setup(|app| {
            // Initialize watcher service
            if let Err(e) = beefcake::watcher::init(app.handle().clone()) {
                eprintln!("Failed to initialize watcher service: {e}");
            }
            if let Err(e) = beefcake::utils::ensure_standard_dirs() {
                eprintln!("Failed to initialize standard directories: {e}");
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                // Flush any pending audit log entries before exit
                beefcake::utils::flush_pending_audit_entries();
            }
        });
}
