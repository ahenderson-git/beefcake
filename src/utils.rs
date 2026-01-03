use eframe::egui;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub action: String,
    pub details: String,
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

    let color = if status.contains("failed") || status.contains("Error") || status.contains("❌") {
        egui::Color32::RED
    } else if status.contains("successful")
        || status.contains("Successfully")
        || status.contains("✅")
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
}
