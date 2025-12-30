/// Formats an optional f64 to 4 decimal places, or returns "—" if None or non-finite.
pub fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.4}"),
        _ => "—".to_owned(),
    }
}
