use chrono::Local;
use keyring::Entry;
use secrecy::SecretString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::runtime::Runtime;

pub const DATA_INPUT_DIR: &str = "data/input";
pub const DATA_PROCESSED_DIR: &str = "data/processed";
pub const KEYRING_SERVICE: &str = "com.beefcake.app";
pub const KEYRING_PLACEHOLDER: &str = "__KEYRING__";

pub fn get_db_password(connection_id: &str) -> Option<String> {
    let entry = Entry::new(KEYRING_SERVICE, connection_id).ok()?;
    entry.get_password().ok()
}

pub fn set_db_password(connection_id: &str, password: &str) -> anyhow::Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, connection_id)?;
    entry.set_password(password)?;
    Ok(())
}

pub fn delete_db_password(connection_id: &str) -> anyhow::Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, connection_id)?;
    entry.delete_credential().map_err(|e| anyhow::anyhow!(e))
}

pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DbConnection {
    pub id: String,
    pub name: String,
    pub settings: DbSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<Local>,
    pub action: String,
    pub details: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub connections: Vec<DbConnection>,
    pub active_import_id: Option<String>,
    pub active_export_id: Option<String>,
    pub powershell_font_size: u32,
    pub python_font_size: u32,
    pub sql_font_size: u32,
    pub audit_log: Vec<AuditEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            active_import_id: None,
            active_export_id: None,
            powershell_font_size: 14,
            python_font_size: 14,
            sql_font_size: 14,
            audit_log: Vec::new(),
        }
    }
}

/// Helper to push a new entry to an audit log.
pub fn push_audit_log(config: &mut AppConfig, action: &str, details: &str) {
    config.audit_log.push(AuditEntry {
        timestamp: Local::now(),
        action: action.to_owned(),
        details: details.to_owned(),
    });
    // Keep only last 100 entries to prevent config file bloat
    if config.audit_log.len() > 100 {
        config.audit_log.remove(0);
    }
}

/// Loads config, adds an audit entry, and saves it.
pub fn log_event(action: &str, details: &str) {
    let mut config = load_app_config();
    push_audit_log(&mut config, action, details);
    let _ = save_app_config(&config);
}

pub fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".beefcake_config.json"))
        .unwrap_or_else(|| PathBuf::from("beefcake_config.json"))
}

pub fn load_app_config() -> AppConfig {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

pub fn save_app_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DbSettings {
    pub db_type: String,
    pub host: String,
    pub port: String,
    pub user: String,
    #[serde(
        serialize_with = "serialize_password",
        deserialize_with = "deserialize_password"
    )]
    pub password: SecretString,
    pub database: String,
    pub schema: String,
    pub table: String,
}

fn serialize_password<S>(password: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use secrecy::ExposeSecret as _;
    let pwd = password.expose_secret();
    if pwd.is_empty() {
        serializer.serialize_str("")
    } else {
        serializer.serialize_str(KEYRING_PLACEHOLDER)
    }
}

fn deserialize_password<'de, D>(deserializer: D) -> Result<SecretString, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    let s = String::deserialize(deserializer)?;
    Ok(SecretString::from(s))
}

impl Default for DbSettings {
    fn default() -> Self {
        Self {
            db_type: "postgres".to_owned(),
            host: "localhost".to_owned(),
            port: "5432".to_owned(),
            user: "postgres".to_owned(),
            password: SecretString::from(KEYRING_PLACEHOLDER.to_owned()),
            database: String::new(),
            schema: "public".to_owned(),
            table: String::new(),
        }
    }
}

impl DbSettings {
    pub fn get_real_password(&self, connection_id: &str) -> String {
        use secrecy::ExposeSecret as _;
        let current = self.password.expose_secret();
        if current == KEYRING_PLACEHOLDER {
            get_db_password(connection_id).unwrap_or_default()
        } else {
            current.to_owned()
        }
    }

    pub fn connection_string(&self, connection_id: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.get_real_password(connection_id),
            self.host,
            self.port,
            self.database
        )
    }
}

pub fn archive_processed_file(file_path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let original = file_path.as_ref();
    let processed_dir = Path::new(DATA_PROCESSED_DIR);

    // Ensure directory exists
    fs::create_dir_all(processed_dir)?;

    // Create new filename: YYYYMMDD_HHMMSS_filename.ext
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = original
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
        .to_string_lossy();

    let destination = processed_dir.join(format!("{timestamp}_{filename}"));

    // Move the file
    fs::rename(original, &destination)?;
    Ok(destination)
}
