use super::types::{ColumnSummary, ColumnStats, FileHealth};

pub fn calculate_file_health(summaries: &[ColumnSummary]) -> FileHealth {
    let mut risks = Vec::new();
    let mut score: f64 = 100.0;

    for col in summaries {
        let null_pct = if col.count > 0 {
            (col.nulls as f64 / col.count as f64) * 100.0
        } else {
            0.0
        };

        if null_pct > 15.0 {
            risks.push(format!(
                "⚠️ Column '{}' has significant missing data ({:.1}%).",
                col.name, null_pct
            ));
            score -= 10.0;
        } else if null_pct > 5.0 {
            score -= 5.0;
        }

        if col.has_special {
            risks.push(format!(
                "🔍 Hidden/special characters detected in '{}'.",
                col.name
            ));
            score -= 5.0;
        }

        if let ColumnStats::Numeric(s) = &col.stats {
            if let (Some(mean), Some(median), Some(min), Some(max)) =
                (s.mean, s.median, s.min, s.max)
            {
                let range = max - min;
                if range > 0.0 {
                    let diff_ratio = (mean - median).abs() / range;
                    if diff_ratio > 0.1 {
                        risks.push(format!(
                            "📈 Column '{}' is heavily skewed; averages may be misleading.",
                            col.name
                        ));
                        score -= 5.0;
                    }
                }
            }
        }
    }
    FileHealth { 
        score: (score.max(0.0) / 100.0) as f32,
        risks 
    }
}
