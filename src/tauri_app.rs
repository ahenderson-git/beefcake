use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use beefcake::analyser::logic::AnalysisResponse;

#[tauri::command]
async fn analyze_file(path: String, trim_pct: Option<f64>) -> Result<AnalysisResponse, String> {
    let path = PathBuf::from(path);
    let progress = Arc::new(AtomicU64::new(0));
    let start = std::time::Instant::now();
    
    let df = beefcake::analyser::logic::load_df(&path, &progress)
        .map_err(|e| e.to_string())?;
    
    let file_size = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    beefcake::analyser::logic::analysis::run_full_analysis(
        df, 
        path.to_string_lossy().to_string(), 
        file_size, 
        trim_pct.unwrap_or(0.05),
        start
    ).map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config() -> Result<beefcake::utils::AppConfig, String> {
    Ok(beefcake::utils::load_app_config())
}

#[tauri::command]
async fn save_config(config: beefcake::utils::AppConfig) -> Result<(), String> {
    beefcake::utils::save_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_powershell(script: String) -> Result<String, String> {
    use std::process::Command;

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
                Err(format!("Error: {}\n{}", stdout, stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute powershell: {}", e)),
    }
}

#[tauri::command]
async fn push_to_db(path: String, connection_id: String) -> Result<(), String> {
    use beefcake::utils::load_app_config;
    use beefcake::analyser::db::DbClient;
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr;
    use std::sync::atomic::AtomicU64;

    let config = load_app_config();
    let conn = config.connections.iter().find(|c| c.id == connection_id)
        .ok_or_else(|| "Connection not found".to_string())?;
    
    let url = conn.settings.connection_string();
    let opts = PgConnectOptions::from_str(&url)
        .map_err(|e| format!("Invalid connection URL: {}", e))?;

    let client = DbClient::connect(opts).await
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let progress = Arc::new(AtomicU64::new(0));
    let df = beefcake::analyser::logic::load_df(&PathBuf::from(&path), &progress)
        .map_err(|e| format!("Failed to load data: {}", e))?;

    client.init_schema().await.map_err(|e| format!("Schema initialization failed: {}", e))?;
    
    client.push_dataframe(0, &df, Some(&conn.settings.schema), Some(&conn.settings.table))
        .await.map_err(|e| format!("Data push failed: {}", e))?;

    Ok(())
}

pub fn run() {
    #[cfg(debug_assertions)]
    {
        use std::net::TcpStream;
        use std::time::Duration;
        let addr = "127.0.0.1:14206";
        if TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(500)).is_err() {
            eprintln!("\n\x1b[1;33m[WARNING] Dev server not detected at {}.\x1b[0m", addr);
            eprintln!("\x1b[1;33m[WARNING] Did you forget to run 'npm run tauri dev' or 'cargo tauri dev'?\x1b[0m");
            eprintln!("\x1b[1;33m[WARNING] Running via 'cargo run' will result in a 'refused to connect' error in the window.\n\x1b[0m");
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
            push_to_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
