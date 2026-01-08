#![allow(clippy::let_underscore_must_use, clippy::let_underscore_untyped, clippy::print_stderr, clippy::exit, clippy::collapsible_if)]
use beefcake::analyser::logic::AnalysisResponse;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[tauri::command]
async fn analyze_file(path: String, trim_pct: Option<f64>) -> Result<AnalysisResponse, String> {
    let mut config = beefcake::utils::load_app_config();
    beefcake::utils::push_audit_log(
        &mut config,
        "Analyser",
        &format!("Started analysis of {path}"),
    );
    let _ = beefcake::utils::save_app_config(&config).ok();

    let path_buf = PathBuf::from(&path);
    let progress = Arc::new(AtomicU64::new(0));
    let start = std::time::Instant::now();

    let df = beefcake::analyser::logic::load_df(&path_buf, &progress).map_err(|e| e.to_string())?;

    let file_size = std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0);

    beefcake::analyser::logic::analysis::run_full_analysis(
        df,
        path,
        file_size,
        trim_pct.unwrap_or(0.05),
        start,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    let mut config = beefcake::utils::load_app_config();
    beefcake::utils::push_audit_log(&mut config, "File", &format!("Read file: {path}"));
    let _ = beefcake::utils::save_app_config(&config).ok();
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    let mut config = beefcake::utils::load_app_config();
    beefcake::utils::push_audit_log(&mut config, "File", &format!("Saved file: {path}"));
    let _ = beefcake::utils::save_app_config(&config).ok();
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config() -> Result<beefcake::utils::AppConfig, String> {
    Ok(beefcake::utils::load_app_config())
}

#[tauri::command]
async fn save_config(mut config: beefcake::utils::AppConfig) -> Result<(), String> {
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

    let mut config = beefcake::utils::load_app_config();
    beefcake::utils::push_audit_log(&mut config, "PowerShell", "Executed script");
    let _ = beefcake::utils::save_app_config(&config).ok();

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

#[tauri::command]
async fn push_to_db(
    path: String,
    connection_id: String,
    configs: std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>,
) -> Result<(), String> {
    use beefcake::analyser::db::DbClient;
    use beefcake::analyser::logic::clean_df;
    use beefcake::utils::load_app_config;
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr as _;
    use std::sync::atomic::AtomicU64;

    let mut config = load_app_config();
    let (conn_name, table_name, connection_id) = {
        let conn = config
            .connections
            .iter()
            .find(|c| c.id == connection_id)
            .ok_or_else(|| "Connection not found".to_owned())?;
        (
            conn.name.clone(),
            conn.settings.table.clone(),
            conn.id.clone(),
        )
    };

    beefcake::utils::push_audit_log(
        &mut config,
        "Database",
        &format!("Pushing data to {conn_name} ({table_name})"),
    );
    let _ = beefcake::utils::save_app_config(&config).ok();

    let conn = config
        .connections
        .iter()
        .find(|c| c.id == connection_id)
        .ok_or_else(|| "Connection not found".to_owned())?;

    let url = conn.settings.connection_string();
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    let client = DbClient::connect(opts)
        .await
        .map_err(|e| format!("Database connection failed: {e}"))?;

    let progress = Arc::new(AtomicU64::new(0));
    let df = beefcake::analyser::logic::load_df(&PathBuf::from(&path), &progress)
        .map_err(|e| format!("Failed to load data: {e}"))?;

    // Apply cleaning configurations from Analyser
    let cleaned_df =
        clean_df(df, &configs, false).map_err(|e| format!("Cleaning failed: {e}"))?;

    client
        .init_schema()
        .await
        .map_err(|e| format!("Schema initialization failed: {e}"))?;

    client
        .push_dataframe(
            0,
            &cleaned_df,
            Some(&conn.settings.schema),
            Some(&conn.settings.table),
        )
        .await
        .map_err(|e| format!("Data push failed: {e}"))?;

    Ok(())
}

#[tauri::command]
async fn test_connection(settings: beefcake::utils::DbSettings) -> Result<String, String> {
    use beefcake::analyser::db::DbClient;
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr as _;

    let url = settings.connection_string();
    let opts =
        PgConnectOptions::from_str(&url).map_err(|e| format!("Invalid connection URL: {e}"))?;

    match DbClient::connect(opts).await {
        Ok(_) => Ok("Connection successful!".to_owned()),
        Err(e) => Err(format!("Connection failed: {e}")),
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
            analyze_file,
            run_powershell,
            read_text_file,
            write_text_file,
            get_config,
            save_config,
            push_to_db,
            test_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
