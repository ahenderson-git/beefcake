use beefcake::config::{AppConfig, load_app_config, save_app_config};
use std::future::Future;
use tauri::Manager as _;

/// Stack size for worker threads (50MB) - used for memory-intensive operations
pub const WORKER_THREAD_STACK_SIZE: usize = 50 * 1024 * 1024;

/// File size threshold (50MB) for warning about memory-intensive operations
pub const LARGE_FILE_WARNING_THRESHOLD: u64 = 50 * 1024 * 1024;

pub fn ensure_security_acknowledged() -> Result<(), String> {
    let config = load_app_config();
    if config.settings.security_warning_acknowledged {
        Ok(())
    } else {
        Err("Security warning not acknowledged. Please confirm before running scripts.".to_owned())
    }
}

pub async fn run_on_worker_thread<F, Fut, R>(name: &str, f: F) -> Result<R, String>
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
pub async fn read_text_file(path: String) -> Result<String, String> {
    beefcake::config::log_event("File", &format!("Read file: {path}"));
    crate::system::read_text_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    beefcake::config::log_event("File", &format!("Saved file: {path}"));
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
pub async fn save_config(mut config: AppConfig) -> Result<(), String> {
    use beefcake::config::{KEYRING_PLACEHOLDER, push_audit_log};
    use beefcake::utils::set_db_password;
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

#[derive(serde::Serialize)]
pub struct StandardPathsPayload {
    pub base_dir: String,
    pub input_dir: String,
    pub output_dir: String,
    pub scripts_dir: String,
    pub logs_dir: String,
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
    })
}

#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    crate::system::open_path(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_trusted_paths() -> Result<Vec<String>, String> {
    let config = load_app_config();
    Ok(config.settings.trusted_paths.clone())
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
    config.settings.trusted_paths.retain(|p| p != &path);
    save_app_config(&config).map_err(|e| e.to_string())?;
    Ok(config.settings.trusted_paths)
}

fn normalize_trusted_root(path: &str) -> Result<String, String> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Err("Path does not exist".to_owned());
    }
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?.join(p)
    };

    let canonical = abs.canonicalize().map_err(|e| e.to_string())?;
    Ok(canonical.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
pub struct DocFileMetadata {
    pub path: String,
    pub title: String,
    pub category: String,
}

#[tauri::command]
pub async fn list_documentation_files(
    app: tauri::AppHandle,
) -> Result<Vec<DocFileMetadata>, String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?
        .join("docs");

    if !resource_path.exists() {
        return Ok(vec![]);
    }

    let mut docs = vec![];
    let entries = std::fs::read_dir(resource_path).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let filename = path
                .file_name()
                .ok_or_else(|| "Invalid file name".to_owned())?
                .to_string_lossy();
            if filename == "README.md" {
                continue;
            }

            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").to_owned())
                .unwrap_or_else(|| filename.to_string());

            docs.push(DocFileMetadata {
                path: filename.to_string(),
                title,
                category: "General".to_owned(),
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
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?
        .join("docs")
        .join(doc_path);

    if !resource_path.exists() {
        return Err("Documentation file not found".to_owned());
    }

    std::fs::read_to_string(resource_path).map_err(|e| e.to_string())
}

/// Logs a message from the frontend to the backend log files
#[tauri::command]
pub async fn log_frontend_error(
    level: String,
    message: String,
    context: Option<serde_json::Value>,
) -> Result<(), String> {
    let context_str = context
        .map(|c| format!(" | context: {c}"))
        .unwrap_or_default();

    match level.as_str() {
        "error" => tracing::error!("[Frontend] {}{}", message, context_str),
        "warn" => tracing::warn!("[Frontend] {}{}", message, context_str),
        "info" => tracing::info!("[Frontend] {}{}", message, context_str),
        "debug" => tracing::debug!("[Frontend] {}{}", message, context_str),
        _ => tracing::info!("[Frontend] {}{}", message, context_str),
    }

    Ok(())
}

/// Logs a frontend event into the audit log and backend logs
#[tauri::command]
pub async fn log_frontend_event(
    level: String,
    action: String,
    details: String,
    context: Option<serde_json::Value>,
) -> Result<(), String> {
    let context_str = context
        .map(|c| format!(" | context: {c}"))
        .unwrap_or_default();

    match level.as_str() {
        "error" => tracing::error!("[Frontend] {}: {}{}", action, details, context_str),
        "warn" => tracing::warn!("[Frontend] {}: {}{}", action, details, context_str),
        "info" => tracing::info!("[Frontend] {}: {}{}", action, details, context_str),
        "debug" => tracing::debug!("[Frontend] {}: {}{}", action, details, context_str),
        _ => tracing::info!("[Frontend] {}: {}{}", action, details, context_str),
    }

    beefcake::config::log_event(&action, &details);
    Ok(())
}

/// Returns the path to the log directory
#[tauri::command]
pub async fn get_log_directory() -> Result<String, String> {
    beefcake::logging::get_log_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// Returns the path to the current log file
#[tauri::command]
pub async fn get_current_log_file() -> Result<String, String> {
    beefcake::logging::get_current_log_path()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// Returns the path to the current error log file
#[tauri::command]
pub async fn get_current_error_log_file() -> Result<String, String> {
    beefcake::logging::get_current_error_log_path()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}
