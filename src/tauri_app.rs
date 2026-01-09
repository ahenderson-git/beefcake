#![allow(clippy::let_underscore_must_use, clippy::let_underscore_untyped, clippy::print_stderr, clippy::exit, clippy::collapsible_if)]
use beefcake::analyser::logic::AnalysisResponse;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

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

    let df = beefcake::analyser::logic::load_df(&path_buf, &progress).map_err(|e| e.to_string())?;

    let file_size = std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0);

    beefcake::analyser::logic::analysis::run_full_analysis(
        df,
        path_str,
        file_size,
        trim_pct.unwrap_or(0.05),
        start,
    )
    .map_err(|e| e.to_string())
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

#[tauri::command]
async fn run_python(
    script: String,
    data_path: Option<String>,
    configs: Option<std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>>,
) -> Result<String, String> {
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    beefcake::utils::log_event(
        "Python",
        &format!(
            "Executing Python script. Argument data_path: {:?}, configs: {}",
            data_path,
            configs.as_ref().map(|c| c.len()).unwrap_or(0)
        ),
    );

    let mut actual_data_path = data_path.clone();

    // If cleaning configs are provided and we have a data path, apply them
    if let (Some(path), Some(cfgs)) = (&data_path, &configs) {
        if !cfgs.is_empty() && !path.is_empty() {
            beefcake::utils::log_event("Python", "Applying cleaning configurations before execution");
            let progress = Arc::new(AtomicU64::new(0));
            let df = beefcake::analyser::logic::load_df(&PathBuf::from(path), &progress)
                .map_err(|e| format!("Failed to load data for cleaning: {e}"))?;

            let mut cleaned_df = beefcake::analyser::logic::clean_df(df, cfgs, false)
                .map_err(|e| format!("Failed to apply cleaning: {e}"))?;

            // Save to a temporary file. Parquet is fast and preserves types.
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join("beefcake_cleaned_data.parquet");

            beefcake::analyser::logic::save_df(&mut cleaned_df, &temp_path)
                .map_err(|e| format!("Failed to save cleaned data to temp file: {e}"))?;

            actual_data_path = Some(temp_path.to_string_lossy().to_string());
        }
    }

    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    // Force UTF-8 encoding for Python IO to avoid UnicodeEncodeError on Windows
    cmd.env("PYTHONIOENCODING", "utf-8");
    
    // Configure Polars to avoid truncation in output
    cmd.env("POLARS_FMT_MAX_COLS", "-1");
    cmd.env("POLARS_FMT_MAX_ROWS", "100");
    cmd.env("POLARS_FMT_STR_LEN", "1000");

    if let Some(path) = &actual_data_path {
        if !path.is_empty() {
            beefcake::utils::log_event("Python", &format!("Setting BEEFCAKE_DATA_PATH to: {}", path));
            cmd.env("BEEFCAKE_DATA_PATH", path);
        } else {
            beefcake::utils::log_event("Python", "actual_data_path is empty string, NOT setting BEEFCAKE_DATA_PATH");
        }
    } else {
        beefcake::utils::log_event("Python", "actual_data_path is None, NOT setting BEEFCAKE_DATA_PATH");
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python: {e}"))?;

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
async fn run_sql(
    query: String,
    data_path: Option<String>,
    configs: Option<std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>>,
) -> Result<String, String> {
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    beefcake::utils::log_event(
        "SQL",
        &format!(
            "Executing SQL query. Argument data_path: {:?}, configs: {}",
            data_path,
            configs.as_ref().map(|c| c.len()).unwrap_or(0)
        ),
    );

    let mut actual_data_path = data_path.clone();

    // Prepare data path if configs are provided (same as run_python)
    if let (Some(path), Some(cfgs)) = (&data_path, &configs) {
        if !cfgs.is_empty() && !path.is_empty() {
            beefcake::utils::log_event("SQL", "Applying cleaning configurations before execution");
            let progress = Arc::new(AtomicU64::new(0));
            let df = beefcake::analyser::logic::load_df(&PathBuf::from(path), &progress)
                .map_err(|e| format!("Failed to load data for cleaning: {e}"))?;

            let mut cleaned_df = beefcake::analyser::logic::clean_df(df, cfgs, false)
                .map_err(|e| format!("Failed to apply cleaning: {e}"))?;

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join("beefcake_cleaned_data_sql.parquet");

            beefcake::analyser::logic::save_df(&mut cleaned_df, &temp_path)
                .map_err(|e| format!("Failed to save cleaned data to temp file: {e}"))?;

            actual_data_path = Some(temp_path.to_string_lossy().to_string());
        }
    }

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
        df = pl.read_parquet(data_path)
    elif data_path.endswith(".json"):
        df = pl.read_json(data_path)
    else:
        df = pl.read_csv(data_path)
    
    ctx = pl.SQLContext()
    ctx.register("data", df)
    
    query = """{}"""
    result = ctx.execute(query).collect()
    print(result._repr_html_())
except Exception as e:
    print(f"SQL Error: {{e}}")
    sys.exit(1)
"#,
        query.replace(r#"""#, r#"\"""#)
    );

    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    cmd.env("PYTHONIOENCODING", "utf-8");

    if let Some(path) = &actual_data_path {
        if !path.is_empty() {
            cmd.env("BEEFCAKE_DATA_PATH", path);
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python for SQL: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    stdin
        .write_all(python_script.as_bytes())
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
async fn sanitize_headers(names: Vec<String>) -> Result<Vec<String>, String> {
    Ok(beefcake::analyser::logic::sanitize_column_names(&names))
}

#[tauri::command]
async fn push_to_db(
    path: String,
    connection_id: String,
    configs: std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig>,
) -> Result<(), String> {
    use beefcake::analyser::db::DbClient;
    use beefcake::analyser::logic::clean_df;
    use beefcake::utils::{load_app_config, push_audit_log, save_app_config};
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr as _;
    use std::sync::atomic::AtomicU64;

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
            sanitize_headers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
