use eframe::egui;
use egui_phosphor::regular as icons;
use regex::Regex;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\x1b\x9b]\[[()#;?]*[0-9.;?]*[a-zA-Z]").expect("ANSI stripping regex is valid")
});

/// Strips ANSI escape codes from a string.
pub fn strip_ansi(input: &str) -> String {
    ANSI_RE.replace_all(input, "").into_owned()
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub action: String,
    pub details: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DetailedError {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub task: String,
    pub message: String,
    pub chain: Vec<String>,
    pub suggestions: Vec<String>,
}

pub fn get_error_diagnostics(err: &anyhow::Error, task: &str) -> DetailedError {
    let mut chain = Vec::new();
    for cause in err.chain().skip(1) {
        chain.push(cause.to_string());
    }

    let message = err.to_string();
    let suggestions = generate_suggestions(&message, &chain);

    DetailedError {
        timestamp: chrono::Local::now(),
        task: task.to_owned(),
        message,
        chain,
        suggestions,
    }
}

fn generate_suggestions(message: &str, chain: &[String]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let full_text = format!("{} {}", message, chain.join(" ")).to_lowercase();

    if full_text.contains("connection refused")
        || full_text.contains("timeout")
        || full_text.contains("no such host")
    {
        suggestions.push("Check if the database server is running and reachable.".to_owned());
        suggestions.push("Verify your host, port, and firewall settings.".to_owned());
    }
    if full_text.contains("authentication failed")
        || full_text.contains("password")
        || full_text.contains("role")
    {
        suggestions.push("Double-check your database username and password.".to_owned());
        suggestions.push(
            "Ensure the user has sufficient permissions for the target database/schema.".to_owned(),
        );
    }
    if full_text.contains("one-hot")
        || full_text.contains("categorical")
        || full_text.contains("float")
    {
        suggestions.push(
            "Try applying One-Hot encoding to your categorical columns before training.".to_owned(),
        );
        suggestions.push(
            "Ensure there are no unexpected non-numeric values in numeric columns.".to_owned(),
        );
    }
    if full_text.contains("csv") && (full_text.contains("parse") || full_text.contains("delimiter"))
    {
        suggestions.push(
            "Check if the CSV file has a valid header and consistent column count.".to_owned(),
        );
        suggestions.push(
            "Verify the delimiter (comma, semicolon, etc.) is correctly detected.".to_owned(),
        );
    }
    if full_text.contains("parquet") {
        suggestions
            .push("The Parquet file might be corrupted or in an unsupported version.".to_owned());
    }
    if full_text.contains("memory") || full_text.contains("allocation") {
        suggestions
            .push("The dataset might be too large for the available system memory.".to_owned());
        suggestions
            .push("Try closing other applications or using a machine with more RAM.".to_owned());
    }

    if suggestions.is_empty() {
        suggestions.push(
            "Consult the application logs or search for the error message online for more details."
                .to_owned(),
        );
    }

    suggestions
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

/// Renders a colored status message based on its content.
pub fn render_status_message(ui: &mut egui::Ui, status: &str) {
    if status.is_empty() {
        return;
    }

    let color = if status.contains("failed")
        || status.contains("Error")
        || status.contains("❌")
        || status.contains(icons::X_CIRCLE)
    {
        egui::Color32::RED
    } else if status.contains("successful")
        || status.contains("Successfully")
        || status.contains("✅")
        || status.contains(icons::CHECK_CIRCLE)
        || status.contains("saved")
        || status.contains("loaded")
        || status.contains("deleted")
    {
        egui::Color32::from_rgb(0, 150, 0)
    } else {
        ui.visuals().text_color()
    };

    ui.label(egui::RichText::new(status).color(color).strong());
}

/// Helper to push a new entry to an audit log.
pub fn push_audit_log(log: &mut Vec<AuditEntry>, action: &str, details: &str) {
    log.push(AuditEntry {
        timestamp: chrono::Local::now(),
        action: action.to_owned(),
        details: details.to_owned(),
    });
}

/// Helper to push a new entry to an error diagnostics log.
pub fn push_error_log(log: &mut Vec<DetailedError>, err: &anyhow::Error, task: &str) {
    log.push(get_error_diagnostics(err, task));
    if log.len() > 50 {
        log.remove(0);
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
