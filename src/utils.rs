use chrono::Local;
use keyring::Entry;
use secrecy::SecretString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const DATA_INPUT_DIR: &str = "data/input";
pub const DATA_PROCESSED_DIR: &str = "data/processed";
pub const KEYRING_SERVICE: &str = "au.com.ahenderson.beefcake";
pub const KEYRING_PLACEHOLDER: &str = "__KEYRING__";

pub static ABORT_SIGNAL: AtomicBool = AtomicBool::new(false);

// Pending audit log entries that will be flushed periodically
lazy_static::lazy_static! {
    static ref PENDING_AUDIT_ENTRIES: Arc<Mutex<Vec<AuditEntry>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn is_aborted() -> bool {
    ABORT_SIGNAL.load(Ordering::SeqCst)
}

pub fn reset_abort_signal() {
    ABORT_SIGNAL.store(false, Ordering::SeqCst);
}

pub fn trigger_abort() {
    ABORT_SIGNAL.store(true, Ordering::SeqCst);
}

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

/// Audit log for tracking application events.
/// Separated from settings to allow independent management.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add an entry to the audit log.
    /// Automatically limits to the last 100 entries to prevent unbounded growth.
    pub fn push(&mut self, action: impl Into<String>, details: impl Into<String>) {
        self.entries.push(AuditEntry {
            timestamp: Local::now(),
            action: action.into(),
            details: details.into(),
        });
        // Keep only last 100 entries to prevent config file bloat
        if self.entries.len() > 100 {
            self.entries.remove(0);
        }
    }

    /// Get all entries in the log.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries in the log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Application settings (connections, fonts, preferences).
/// Separated from audit log for cleaner organization.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)]
pub struct AppSettings {
    pub connections: Vec<DbConnection>,
    pub active_import_id: Option<String>,
    pub active_export_id: Option<String>,
    pub powershell_font_size: u32,
    pub python_font_size: u32,
    pub sql_font_size: u32,
    /// Maximum number of rows to display in SQL/Python previews (default: 100)
    pub preview_row_limit: u32,
    /// Whether to show security warning on first Python/PowerShell execution
    pub security_warning_acknowledged: bool,
    /// Whether to skip full row counting for large CSV files (improves load times)
    pub skip_full_row_count: bool,
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
            preview_row_limit: 100,
            security_warning_acknowledged: false,
            skip_full_row_count: false,
        }
    }
}

/// Top-level configuration structure that combines settings and audit log.
/// Maintains backward compatibility with existing JSON config files.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    #[serde(flatten)]
    pub settings: AppSettings,
    #[serde(rename = "audit_log")]
    audit_log_entries: Vec<AuditEntry>,
}

impl AppConfig {
    /// Get a reference to the settings.
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Get a mutable reference to the settings.
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Get the audit log.
    pub fn audit_log(&self) -> AuditLog {
        AuditLog {
            entries: self.audit_log_entries.clone(),
        }
    }

    /// Set the audit log.
    pub fn set_audit_log(&mut self, log: AuditLog) {
        self.audit_log_entries = log.entries;
    }

    /// Add an entry to the audit log (convenience method).
    pub fn log_event(&mut self, action: impl Into<String>, details: impl Into<String>) {
        let mut log = self.audit_log();
        log.push(action, details);
        self.set_audit_log(log);
    }

    // Maintain backward compatibility with direct field access
    #[allow(dead_code)]
    pub fn connections(&self) -> &[DbConnection] {
        &self.settings.connections
    }

    #[allow(dead_code)]
    pub fn connections_mut(&mut self) -> &mut Vec<DbConnection> {
        &mut self.settings.connections
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            audit_log_entries: Vec::new(),
        }
    }
}

/// Helper to push a new entry to an audit log.
/// This is a convenience function that uses the new AppConfig::log_event method.
pub fn push_audit_log(config: &mut AppConfig, action: &str, details: &str) {
    config.log_event(action, details);
}

/// Loads config, adds an audit entry, and saves it.
/// Uses debouncing to reduce I/O overhead - entries are queued and flushed periodically.
pub fn log_event(action: &str, details: &str) {
    // Add to pending queue instead of immediate write
    if let Ok(mut pending) = PENDING_AUDIT_ENTRIES.lock() {
        pending.push(AuditEntry {
            timestamp: Local::now(),
            action: action.to_owned(),
            details: details.to_owned(),
        });

        // If we have accumulated 10+ entries, flush them
        if pending.len() >= 10 {
            flush_pending_audit_entries_internal(&mut pending);
        }
    }
}

/// Flush any pending audit entries to disk.
/// This should be called periodically or before app shutdown.
pub fn flush_pending_audit_entries() {
    if let Ok(mut pending) = PENDING_AUDIT_ENTRIES.lock()
        && !pending.is_empty() {
        flush_pending_audit_entries_internal(&mut pending);
    }
}

fn flush_pending_audit_entries_internal(pending: &mut Vec<AuditEntry>) {
    if pending.is_empty() {
        return;
    }

    let mut config = load_app_config();
    for entry in pending.drain(..) {
        let mut log = config.audit_log();
        log.entries.push(entry);
        config.set_audit_log(log);
    }
    // Intentionally ignore errors during shutdown - logging is best-effort
    drop(save_app_config(&config));
}

pub fn get_config_path() -> PathBuf {
    if let Some(mut path) = dirs::config_dir() {
        path.push("beefcake");
        path.push("config.json");
        path
    } else {
        dirs::home_dir()
            .map(|p| p.join(".beefcake_config.json"))
            .unwrap_or_else(|| PathBuf::from("beefcake_config.json"))
    }
}

pub fn load_app_config() -> AppConfig {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        // Try fallback to old path if it exists
        if let Some(old_path) = dirs::home_dir().map(|p| p.join(".beefcake_config.json")) {
            if old_path.exists() && old_path != path {
                if let Ok(content) = fs::read_to_string(old_path) {
                    return serde_json::from_str(&content).unwrap_or_default();
                }
            }
        }
        AppConfig::default()
    }
}

pub fn save_app_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
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

pub fn fmt_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let i = std::cmp::min(i, units.len() - 1);
    let value = bytes as f64 / 1024.0f64.powi(i as i32);
    format!("{:.2} {}", value, units[i])
}

pub fn fmt_count(count: usize) -> String {
    if count >= 1_000_000_000 {
        format!("{:.2}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// RAII guard that automatically deletes a temporary file when dropped.
/// This ensures cleanup happens even if an error occurs.
pub struct TempFileGuard {
    path: Option<PathBuf>,
}

impl TempFileGuard {
    /// Create a new guard for the given path.
    pub fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    /// Get the path to the temporary file.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Consume the guard and return the path without deleting the file.
    /// Use this if you want to keep the temporary file.
    pub fn keep(mut self) -> Option<PathBuf> {
        self.path.take()
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

/// Manages a collection of temporary files that will be cleaned up when dropped.
pub struct TempFileCollection {
    files: Vec<TempFileGuard>,
}

impl TempFileCollection {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add a temporary file to the collection.
    pub fn add(&mut self, path: PathBuf) {
        self.files.push(TempFileGuard::new(path));
    }

    /// Get all paths in the collection.
    pub fn paths(&self) -> Vec<&Path> {
        self.files.iter().filter_map(|g| g.path()).collect()
    }
}

impl Default for TempFileCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_file_guard_cleanup() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("beefcake_test_temp_file.txt");

        // Create a test file
        std::fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());

        // Create guard and drop it
        {
            let _guard = TempFileGuard::new(test_file.clone());
            assert!(test_file.exists());
        }

        // File should be deleted after guard is dropped
        assert!(!test_file.exists());
    }

    #[test]
    fn test_temp_file_guard_keep() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("beefcake_test_keep_file.txt");

        // Create a test file
        std::fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());

        // Create guard, keep it, then drop
        {
            let guard = TempFileGuard::new(test_file.clone());
            let _ = guard.keep();
        }

        // File should still exist because we called keep()
        assert!(test_file.exists());

        // Clean up
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_temp_file_collection() {
        let temp_dir = std::env::temp_dir();
        let test_file1 = temp_dir.join("beefcake_test_collection_1.txt");
        let test_file2 = temp_dir.join("beefcake_test_collection_2.txt");

        // Create test files
        std::fs::write(&test_file1, "test1").unwrap();
        std::fs::write(&test_file2, "test2").unwrap();
        assert!(test_file1.exists());
        assert!(test_file2.exists());

        // Create collection and drop it
        {
            let mut collection = TempFileCollection::new();
            collection.add(test_file1.clone());
            collection.add(test_file2.clone());
            assert_eq!(collection.paths().len(), 2);
        }

        // Both files should be deleted
        assert!(!test_file1.exists());
        assert!(!test_file2.exists());
    }
}
