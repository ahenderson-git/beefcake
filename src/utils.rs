use regex::Regex;
use secrecy::SecretString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::runtime::Runtime;
use chrono::Local;

pub const DATA_INPUT_DIR: &str = "data/input";
pub const DATA_PROCESSED_DIR: &str = "data/processed";

pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\x1b\x9b]\[[()#;?]*[0-9.;?]*[a-zA-Z]").expect("ANSI stripping regex is valid")
});

/// Strips ANSI escape codes from a string.
pub fn strip_ansi(input: &str) -> String {
    ANSI_RE.replace_all(input, "").into_owned()
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DbConnection {
    pub id: String,
    pub name: String,
    pub settings: DbSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub connections: Vec<DbConnection>,
    pub active_import_id: Option<String>,
    pub active_export_id: Option<String>,
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
    serializer.serialize_str(password.expose_secret())
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
            password: SecretString::default(),
            database: String::new(),
            schema: "public".to_owned(),
            table: String::new(),
        }
    }
}

impl DbSettings {
    pub fn connection_string(&self) -> String {
        use secrecy::ExposeSecret as _;
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        )
    }
}

/// Formats an optional f64 to 4 decimal places, or returns "—" if None or non-finite.
pub fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.4}"),
        _ => "—".to_owned(),
    }
}

/// Formats bytes into a human-readable string (KB, MB, GB).
pub fn fmt_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    let mb = kb / 1024.0;
    let gb = mb / 1024.0;

    if gb >= 1.0 {
        format!("{gb:.2} GB")
    } else if mb >= 1.0 {
        format!("{mb:.2} MB")
    } else if kb >= 1.0 {
        format!("{kb:.2} KB")
    } else {
        format!("{bytes} B")
    }
}

/// Formats a number into a human-readable string (K, M, B).
pub fn fmt_num_human(num: usize) -> String {
    let n = num as f64;
    if n >= 1_000_000_000.0 {
        format!("{:.1}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        num.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_num_human() {
        assert_eq!(fmt_num_human(0), "0");
        assert_eq!(fmt_num_human(999), "999");
        assert_eq!(fmt_num_human(1000), "1.0K");
        assert_eq!(fmt_num_human(1500), "1.5K");
        assert_eq!(fmt_num_human(999_900), "999.9K");
        assert_eq!(fmt_num_human(1_000_000), "1.0M");
        assert_eq!(fmt_num_human(1_500_000), "1.5M");
        assert_eq!(fmt_num_human(1_000_000_000), "1.0B");
        assert_eq!(fmt_num_human(2_100_000_000), "2.1B");
    }

    #[test]
    fn test_strip_ansi() {
        let input = "\x1b[1;38;5;9merror[internal]\x1b[0m";
        assert_eq!(strip_ansi(input), "error[internal]");

        let input2 = "Normal text \x1b[32mGreen\x1b[0m and \x1b[1mBold\x1b[0m";
        assert_eq!(strip_ansi(input2), "Normal text Green and Bold");
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

    let destination = processed_dir.join(format!("{}_{}", timestamp, filename));

    // Move the file
    fs::rename(original, &destination)?;
    Ok(destination)
}
