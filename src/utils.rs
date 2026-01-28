use chrono::Local;
use keyring::Entry;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub const DATA_INPUT_DIR: &str = "data/input";
pub const DATA_PROCESSED_DIR: &str = "data/processed";
pub const KEYRING_SERVICE: &str = "au.com.ahenderson.beefcake";

/// Maximum number of audit log entries to keep (prevents config file bloat)
pub const MAX_AUDIT_LOG_ENTRIES: usize = 100;

pub static ABORT_SIGNAL: AtomicBool = AtomicBool::new(false);

/// Standard app directories under the app data directory.
#[derive(Debug, Clone)]
pub struct StandardPaths {
    pub base_dir: PathBuf,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub templates_dir: PathBuf,
}

pub fn app_data_dir() -> PathBuf {
    if let Some(dir) = dirs::data_local_dir() {
        dir.join("beefcake")
    } else {
        PathBuf::from("data")
    }
}

pub fn standard_paths() -> StandardPaths {
    let base_dir = app_data_dir();
    StandardPaths {
        input_dir: base_dir.join("input"),
        output_dir: base_dir.join("output"),
        scripts_dir: base_dir.join("scripts"),
        logs_dir: base_dir.join("logs"),
        templates_dir: base_dir.join("templates"),
        base_dir,
    }
}

pub fn ensure_standard_dirs() -> anyhow::Result<StandardPaths> {
    let paths = standard_paths();
    fs::create_dir_all(&paths.input_dir)?;
    fs::create_dir_all(&paths.output_dir)?;
    fs::create_dir_all(&paths.scripts_dir)?;
    fs::create_dir_all(&paths.logs_dir)?;
    fs::create_dir_all(&paths.templates_dir)?;
    Ok(paths)
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

/// Get the AI API key from the system keyring
pub fn get_ai_api_key() -> Option<String> {
    let entry = Entry::new(KEYRING_SERVICE, "ai_api_key").ok()?;
    entry.get_password().ok()
}

/// Check if an AI API key is configured in the system keyring
pub fn has_ai_api_key() -> bool {
    get_ai_api_key().is_some()
}

/// Set the AI API key in the system keyring
pub fn set_ai_api_key(api_key: &str) -> anyhow::Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, "ai_api_key")?;
    entry.set_password(api_key)?;
    Ok(())
}

/// Delete the AI API key from the system keyring
pub fn delete_ai_api_key() -> anyhow::Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, "ai_api_key")?;
    entry.delete_credential().map_err(|e| anyhow::anyhow!(e))
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
        return "0 B".to_owned();
    }
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let i = std::cmp::min(i, units.len() - 1);
    let value = bytes as f64 / 1024.0f64.powi(i as i32);
    let unit = units.get(i).unwrap_or(&"EB");
    format!("{value:.2} {unit}")
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
        if let Some(path) = &self.path
            && path.exists()
        {
            let _ = fs::remove_file(path);
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
        std::fs::write(&test_file, "test content").expect("Write failed");
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
        std::fs::write(&test_file, "test content").expect("Write failed");
        assert!(test_file.exists());

        // Create guard, keep it, then drop
        {
            let guard = TempFileGuard::new(test_file.clone());
            let _ = guard.keep();
        }

        // File should still exist because we called keep()
        assert!(test_file.exists());

        // Clean up
        std::fs::remove_file(&test_file).expect("Remove failed");
    }

    #[test]
    fn test_temp_file_collection() {
        let temp_dir = std::env::temp_dir();
        let test_file1 = temp_dir.join("beefcake_test_collection_1.txt");
        let test_file2 = temp_dir.join("beefcake_test_collection_2.txt");

        // Create test files
        std::fs::write(&test_file1, "test1").expect("Write failed");
        std::fs::write(&test_file2, "test2").expect("Write failed");
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
