use super::naming;
use super::profiling;
use super::types::{AnalysisResponse, ColumnSummary, CorrelationMatrix};
use anyhow::{Context as _, Result};
use polars::prelude::*;

pub fn run_full_analysis(
    df: DataFrame,
    path: String,
    file_size: u64,
    total_row_count: usize,
    trim_pct: f64,
    start_time: std::time::Instant,
) -> Result<AnalysisResponse> {
    let summary = analyse_df(&df, trim_pct)?;
    let health = super::health::calculate_file_health(&summary);
    let correlation_matrix = calculate_correlation_matrix(&df)?;
    let row_count = df.height();
    let column_count = df.width();
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_owned();

    Ok(AnalysisResponse {
        file_name,
        path,
        file_size,
        row_count,
        total_row_count,
        column_count,
        summary,
        health,
        duration: start_time.elapsed(),
        df,
        correlation_matrix,
    })
}

pub fn analyse_df(df: &DataFrame, trim_pct: f64) -> Result<Vec<ColumnSummary>> {
    let row_count = df.height();
    let mut summaries = Vec::new();

    for col in df.get_columns() {
        let name = col.name().to_string();
        let nulls = col.null_count();
        let count = row_count;

        let samples = {
            let series = col.as_materialized_series();
            let mut head = series.drop_nulls().head(Some(10));
            if head.is_empty() && !series.is_empty() {
                head = series.head(Some(10));
            }
            match head.cast(&DataType::String) {
                Ok(s_ca) => s_ca
                    .str()
                    .map(|ca| {
                        ca.into_iter()
                            .flatten()
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                Err(_) => head.iter().map(|v| v.to_string()).collect(),
            }
        };

        let dtype = col.dtype();

        let (kind, stats, has_special) = if dtype.is_bool() {
            let (k, s) = profiling::analyse_boolean(col)
                .context(format!("Analysis failed for boolean column '{}'", name))?;
            (k, s, false)
        } else if dtype.is_numeric() {
            let (k, s) = profiling::analyse_numeric(col, trim_pct)
                .context(format!("Analysis failed for numeric column '{}'", name))?;
            (k, s, false)
        } else if dtype.is_temporal() {
            let (k, s) = profiling::analyse_temporal(col)
                .context(format!("Analysis failed for temporal column '{}'", name))?;
            (k, s, false)
        } else {
            profiling::analyse_text_or_fallback(&name, col)
                .context(format!("Analysis failed for text column '{}'", name))?
        };

        let mut summary = ColumnSummary {
            name,
            standardized_name: String::new(),
            kind,
            count,
            nulls,
            has_special,
            stats,
            interpretation: Vec::new(),
            business_summary: Vec::new(),
            ml_advice: Vec::new(),
            samples,
        };
        summary.interpretation = summary.generate_interpretation();
        summary.business_summary = summary.generate_business_summary();
        summary.ml_advice = summary.generate_ml_advice();
        summaries.push(summary);
    }

    let names: Vec<String> = summaries.iter().map(|s| s.name.clone()).collect();
    let sanitized_names = naming::sanitize_column_names(&names);
    for (i, summary) in summaries.iter_mut().enumerate() {
        summary.standardized_name = sanitized_names[i].clone();
    }

    Ok(summaries)
}

pub fn calculate_correlation_matrix(df: &DataFrame) -> Result<Option<CorrelationMatrix>> {
    let numeric_cols: Vec<String> = df
        .get_column_names()
        .iter()
        .filter(|&name| {
            df.column(name)
                .map(|c| c.dtype().is_numeric())
                .unwrap_or(false)
        })
        .map(|&s| s.to_string())
        .collect();

    if numeric_cols.len() < 2 {
        return Ok(None);
    }

    let mut matrix = Vec::new();
    for i in 0..numeric_cols.len() {
        let mut row = Vec::new();
        for j in 0..numeric_cols.len() {
            if i == j {
                row.push(1.0);
            } else {
                let s1 = df.column(&numeric_cols[i])?.as_materialized_series();
                let s2 = df.column(&numeric_cols[j])?.as_materialized_series();

                // Pearson correlation
                let corr = if let (Ok(ca1), Ok(ca2)) = (s1.f64(), s2.f64()) {
                    polars::prelude::cov::pearson_corr(ca1, ca2)
                } else {
                    None
                };
                row.push(corr.unwrap_or(0.0));
            }
        }
        matrix.push(row);
    }

    Ok(Some(CorrelationMatrix {
        columns: numeric_cols,
        data: matrix,
    }))
}
