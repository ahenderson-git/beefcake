use anyhow::Result;
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub const KEYRING_PLACEHOLDER: &str = "__KEYRING__";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConnection {
    pub id: String,
    pub name: String,
    pub settings: DbSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, action: impl Into<String>, details: impl Into<String>) {
        self.entries.push(AuditEntry {
            timestamp: Utc::now(),
            action: action.into(),
            details: details.into(),
        });

        // Keep only last 1000 entries
        if self.entries.len() > 1000 {
            self.entries.drain(0..self.entries.len() - 1000);
        }
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub enabled: bool,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model: "gpt-4o".to_owned(),
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub connections: Vec<DbConnection>,
    pub active_import_id: Option<String>,
    pub active_export_id: Option<String>,
    pub powershell_font_size: u32,
    pub python_font_size: u32,
    pub sql_font_size: u32,
    /// Whether the first-run setup wizard has been completed
    pub first_run_completed: bool,
    /// User-approved roots for file IO operations
    pub trusted_paths: Vec<String>,
    /// Maximum number of rows to display in Sql/Python previews (default: 100)
    pub preview_row_limit: u32,
    /// Whether to show security warning on first Python/PowerShell execution
    pub security_warning_acknowledged: bool,
    /// Whether to skip full row counting for large CSV files (improves load times)
    pub skip_full_row_count: bool,
    /// Sample size for statistical analysis - rows used for histograms, quantiles, and distribution stats
    /// (default: 10000, min: 1000, max: 500000)
    pub analysis_sample_size: u32,
    /// Sampling strategy for large datasets
    /// "fast" - Samples from first N rows (fastest, may be biased for sorted data)
    /// "balanced" - Stratified sampling across entire file (recommended, good accuracy)
    /// "accurate" - Reservoir sampling from entire file (slowest, perfectly unbiased)
    pub sampling_strategy: String,
    /// AI assistant configuration
    pub ai_config: AIConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            active_import_id: None,
            active_export_id: None,
            powershell_font_size: 14,
            python_font_size: 14,
            sql_font_size: 14,
            first_run_completed: false,
            trusted_paths: Vec::new(),
            preview_row_limit: 100,
            security_warning_acknowledged: false,
            skip_full_row_count: false,
            analysis_sample_size: 10_000,
            sampling_strategy: "balanced".to_owned(),
            ai_config: AIConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub settings: AppSettings,
    pub audit_log: AuditLog,
}

impl AppConfig {
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    pub fn audit_log(&self) -> &AuditLog {
        &self.audit_log
    }

    pub fn set_audit_log(&mut self, log: AuditLog) {
        self.audit_log = log;
    }

    pub fn log_event(&mut self, action: impl Into<String>, details: impl Into<String>) {
        self.audit_log.push(action, details);
    }

    pub fn connections(&self) -> &[DbConnection] {
        &self.settings.connections
    }

    pub fn connections_mut(&mut self) -> &mut Vec<DbConnection> {
        &mut self.settings.connections
    }
}

pub fn push_audit_log(config: &mut AppConfig, action: &str, details: &str) {
    config.log_event(action, details);
}

static PENDING_AUDIT_ENTRIES: Mutex<Vec<AuditEntry>> = Mutex::new(Vec::new());

pub fn log_event(action: &str, details: &str) {
    let entry = AuditEntry {
        timestamp: Utc::now(),
        action: action.to_owned(),
        details: details.to_owned(),
    };

    if let Ok(mut pending) = PENDING_AUDIT_ENTRIES.lock() {
        pending.push(entry);
        if pending.len() >= 10 {
            flush_pending_audit_entries_internal(&mut pending);
        }
    }
}

pub fn flush_pending_audit_entries() {
    if let Ok(mut pending) = PENDING_AUDIT_ENTRIES.lock()
        && !pending.is_empty()
    {
        flush_pending_audit_entries_internal(&mut pending);
    }
}

fn flush_pending_audit_entries_internal(pending: &mut Vec<AuditEntry>) {
    let mut config = load_app_config();
    for entry in pending.drain(..) {
        config.audit_log.entries.push(entry);
    }
    // Keep only last 1000 entries
    if config.audit_log.entries.len() > 1000 {
        let len = config.audit_log.entries.len();
        config.audit_log.entries.drain(0..len - 1000);
    }
    let _ = save_app_config(&config);
}

pub fn get_config_path() -> PathBuf {
    crate::utils::standard_paths().base_dir.join("config.json")
}

pub fn load_app_config() -> AppConfig {
    let path = get_config_path();
    if path.exists()
        && let Ok(content) = std::fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str::<AppConfig>(&content)
    {
        return config;
    }

    AppConfig {
        settings: AppSettings::default(),
        audit_log: AuditLog::new(),
    }
}

pub fn save_app_config(config: &AppConfig) -> Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    let s = String::deserialize(deserializer)?;
    Ok(SecretString::new(s.into()))
}

impl Default for DbSettings {
    fn default() -> Self {
        Self {
            db_type: "Postgres".to_owned(),
            host: "localhost".to_owned(),
            port: "5432".to_owned(),
            user: "postgres".to_owned(),
            password: SecretString::new(String::new().into()),
            database: "postgres".to_owned(),
            schema: "public".to_owned(),
            table: "data".to_owned(),
        }
    }
}

impl DbSettings {
    pub fn get_real_password(&self, connection_id: &str) -> String {
        use secrecy::ExposeSecret as _;
        let pwd = self.password.expose_secret();
        if pwd == KEYRING_PLACEHOLDER {
            crate::utils::get_db_password(connection_id).unwrap_or_default()
        } else {
            pwd.to_owned()
        }
    }

    pub fn connection_string(&self, connection_id: &str) -> String {
        let password = self.get_real_password(connection_id);
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, password, self.host, self.port, self.database
        )
    }
}
