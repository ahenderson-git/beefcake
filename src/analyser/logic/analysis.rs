use super::naming;
use super::profiling;
use super::types::{
    AnalysisResponse, BooleanStats, ColumnKind, ColumnStats, ColumnSummary, CorrelationMatrix,
    NumericStats,
};
use anyhow::{Context as _, Result};
use polars::prelude::*;
use std::collections::HashMap;

pub fn run_full_analysis_streaming(
    lf: LazyFrame,
    path: String,
    file_size: u64,
    total_row_count: usize,
    sampled_row_count: usize,
    trim_pct: f64,
    start_time: std::time::Instant,
) -> Result<AnalysisResponse> {
    let summary = analyse_df_lazy(lf.clone(), trim_pct)?;
    let health = super::health::calculate_file_health(&summary);
    let correlation_matrix = calculate_correlation_matrix_lazy(lf.clone())?;

    // Collect a small sample for the response (e.g. 100 rows)
    let df = lf.limit(100).collect()?;

    let row_count = sampled_row_count;
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

pub fn run_full_analysis(
    lf: LazyFrame,
    path: String,
    file_size: u64,
    total_row_count: usize,
    sampled_row_count: usize,
    trim_pct: f64,
    start_time: std::time::Instant,
) -> Result<AnalysisResponse> {
    run_full_analysis_streaming(lf, path, file_size, total_row_count, sampled_row_count, trim_pct, start_time)
}

pub fn analyse_df(df: &DataFrame, trim_pct: f64) -> Result<Vec<ColumnSummary>> {
    analyse_df_lazy(df.clone().lazy(), trim_pct)
}

pub fn analyse_df_lazy(mut lf: LazyFrame, trim_pct: f64) -> Result<Vec<ColumnSummary>> {
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let mut summaries = Vec::new();

    // Get a small sample for samples and for stats that are hard to do streaming
    let sample_df = lf.clone().limit(100).collect()?;

    let total_rows = lf
        .clone()
        .select([len()])
        .with_streaming(true)
        .collect()?
        .column("len")?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let adaptive_sample_size = profiling::get_adaptive_sample_size(total_rows);

    for (name, dtype) in schema.iter() {
        let name_str = name.as_str();
        let col_lf = lf.clone().select([col(name_str)]);

        let summary = match dtype {
            DataType::Int64
            | DataType::Int32
            | DataType::Float64
            | DataType::Float32
            | DataType::UInt64
            | DataType::UInt32 => compute_numeric_stats_streaming(
                col_lf,
                name_str,
                trim_pct,
                total_rows,
                &sample_df,
                adaptive_sample_size,
            )?,
            DataType::String => compute_categorical_stats_bounded(
                col_lf,
                name_str,
                total_rows,
                &sample_df,
                adaptive_sample_size,
            )?,
            DataType::Boolean => {
                compute_boolean_stats_streaming(col_lf, name_str, total_rows, &sample_df)?
            }
            _ => {
                if dtype.is_temporal() {
                    compute_temporal_stats_streaming(col_lf, name_str, total_rows, &sample_df)?
                } else {
                    // Fallback for other types
                    compute_text_stats_streaming(col_lf, name_str, total_rows, &sample_df)?
                }
            }
        };

        summaries.push(summary);
    }

    let names: Vec<String> = summaries.iter().map(|s| s.name.clone()).collect();
    let sanitized_names = naming::sanitize_column_names(&names);
    for (i, summary) in summaries.iter_mut().enumerate() {
        if let Some(sanitized) = sanitized_names.get(i) {
            summary.standardised_name = sanitized.clone();
        }
    }

    Ok(summaries)
}

fn extract_samples(sample_df: &DataFrame, name: &str) -> Result<Vec<String>> {
    let series = sample_df.column(name)?.as_materialized_series();
    let mut head = series.drop_nulls().head(Some(10));
    if head.is_empty() && !series.is_empty() {
        head = series.head(Some(10));
    }
    match head.cast(&DataType::String) {
        Ok(s_ca) => Ok(s_ca
            .str()
            .map(|ca| {
                ca.into_iter()
                    .flatten()
                    .map(|s| s.to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()),
        Err(_) => Ok(head.iter().map(|v| v.to_string()).collect()),
    }
}

fn check_special_characters_streaming(col_lf: LazyFrame, name: &str) -> bool {
    if let Ok(df) = col_lf
        .select([col(name).str().contains(lit(r"\r"), false).any(false)])
        .collect()
        && let Ok(col) = df.column(name) {
            return col
                .as_materialized_series()
                .bool()
                .ok()
                .and_then(|ca| ca.get(0))
                .unwrap_or(false);
        }
    false
}

fn compute_numeric_stats_streaming(
    lf: LazyFrame,
    name: &str,
    trim_pct: f64,
    total_rows: usize,
    sample_df: &DataFrame,
    adaptive_sample_size: usize,
) -> Result<ColumnSummary> {
    let (kind, stats) = compute_numeric_stats(lf.clone(), name, trim_pct, adaptive_sample_size)?;
    let samples = extract_samples(sample_df, name)?;

    let null_count = lf
        .select([col(name).null_count()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let mut summary = ColumnSummary {
        name: name.to_owned(),
        standardised_name: String::new(),
        kind,
        count: total_rows,
        nulls: null_count,
        has_special: false,
        stats,
        interpretation: Vec::new(),
        business_summary: Vec::new(),
        ml_advice: Vec::new(),
        samples,
    };
    summary.interpretation = summary.generate_interpretation();
    summary.business_summary = summary.generate_business_summary();
    summary.ml_advice = summary.generate_ml_advice();
    Ok(summary)
}

fn compute_categorical_stats_bounded(
    lf: LazyFrame,
    name: &str,
    total_rows: usize,
    sample_df: &DataFrame,
    adaptive_sample_size: usize,
) -> Result<ColumnSummary> {
    let has_special = check_special_characters_streaming(lf.clone(), name);
    let samples = extract_samples(sample_df, name)?;

    let null_count = lf
        .clone()
        .select([col(name).null_count()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let unique_count = lf
        .clone()
        .select([col(name).n_unique()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let is_categorical = unique_count < 100 || (unique_count as f64 / total_rows as f64) < 0.05;

    let (kind, stats) = if is_categorical {
        compute_categorical_stats(lf, name, adaptive_sample_size)?
    } else {
        let (k, s, _) = profiling::analyse_text_or_fallback(name, sample_df.column(name)?)?;
        (k, s)
    };

    let mut summary = ColumnSummary {
        name: name.to_owned(),
        standardised_name: String::new(),
        kind,
        count: total_rows,
        nulls: null_count,
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
    Ok(summary)
}

fn compute_boolean_stats_streaming(
    lf: LazyFrame,
    name: &str,
    total_rows: usize,
    sample_df: &DataFrame,
) -> Result<ColumnSummary> {
    let (kind, stats) = compute_boolean_stats(lf.clone(), name)?;
    let samples = extract_samples(sample_df, name)?;

    let null_count = lf
        .select([col(name).null_count()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let mut summary = ColumnSummary {
        name: name.to_owned(),
        standardised_name: String::new(),
        kind,
        count: total_rows,
        nulls: null_count,
        has_special: false,
        stats,
        interpretation: Vec::new(),
        business_summary: Vec::new(),
        ml_advice: Vec::new(),
        samples,
    };
    summary.interpretation = summary.generate_interpretation();
    summary.business_summary = summary.generate_business_summary();
    summary.ml_advice = summary.generate_ml_advice();
    Ok(summary)
}

fn compute_temporal_stats_streaming(
    lf: LazyFrame,
    name: &str,
    total_rows: usize,
    sample_df: &DataFrame,
) -> Result<ColumnSummary> {
    let (kind, stats) = profiling::analyse_temporal(sample_df.column(name)?)?;
    let samples = extract_samples(sample_df, name)?;

    let null_count = lf
        .select([col(name).null_count()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let mut summary = ColumnSummary {
        name: name.to_owned(),
        standardised_name: String::new(),
        kind,
        count: total_rows,
        nulls: null_count,
        has_special: false,
        stats,
        interpretation: Vec::new(),
        business_summary: Vec::new(),
        ml_advice: Vec::new(),
        samples,
    };
    summary.interpretation = summary.generate_interpretation();
    summary.business_summary = summary.generate_business_summary();
    summary.ml_advice = summary.generate_ml_advice();
    Ok(summary)
}

fn compute_text_stats_streaming(
    lf: LazyFrame,
    name: &str,
    total_rows: usize,
    sample_df: &DataFrame,
) -> Result<ColumnSummary> {
    let (kind, stats, _) = profiling::analyse_text_or_fallback(name, sample_df.column(name)?)?;
    let samples = extract_samples(sample_df, name)?;

    let null_count = lf
        .select([col(name).null_count()])
        .collect()?
        .column(name)?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let mut summary = ColumnSummary {
        name: name.to_owned(),
        standardised_name: String::new(),
        kind,
        count: total_rows,
        nulls: null_count,
        has_special: false,
        stats,
        interpretation: Vec::new(),
        business_summary: Vec::new(),
        ml_advice: Vec::new(),
        samples,
    };
    summary.interpretation = summary.generate_interpretation();
    summary.business_summary = summary.generate_business_summary();
    summary.ml_advice = summary.generate_ml_advice();
    Ok(summary)
}

pub fn compute_numeric_stats(
    lf: LazyFrame,
    name: &str,
    trim_pct: f64,
    adaptive_sample_size: usize,
) -> Result<(ColumnKind, ColumnStats)> {
    let stats_df = lf
        .clone()
        .select([
            col(name).min().alias("min"),
            col(name).max().alias("max"),
            col(name).mean().alias("mean"),
            col(name).median().alias("median"),
            col(name).std(1).alias("std"),
            col(name)
                .quantile(lit(0.25), QuantileMethod::Linear)
                .alias("q1"),
            col(name)
                .quantile(lit(0.75), QuantileMethod::Linear)
                .alias("q3"),
            col(name)
                .quantile(lit(0.05), QuantileMethod::Linear)
                .alias("p05"),
            col(name)
                .quantile(lit(0.95), QuantileMethod::Linear)
                .alias("p95"),
            col(name).n_unique().alias("distinct_count"),
            col(name).null_count().alias("null_count"),
            col(name).sum().alias("sum"),
            col(name).eq(lit(0)).sum().alias("zero_count"),
            col(name).lt(lit(0)).sum().alias("negative_count"),
            col(name).floor().eq(col(name)).all(false).alias("is_integer"),
            len().alias("count"),
        ])
        .with_streaming(true)
        .collect()
        .context("Failed to compute numeric stats via LazyFrame")?;

    let get_f64 = |c: &str| -> Option<f64> {
        let col = stats_df.column(c).ok()?;
        let s = col.as_materialized_series();
        if let Ok(ca) = s.f64() {
            ca.get(0)
        } else if let Ok(casted) = s.cast(&DataType::Float64) {
            let ca = casted.f64().ok()?;
            ca.get(0)
        } else {
            None
        }
    };

    let get_usize = |c: &str| -> usize {
        stats_df
            .column(c)
            .ok()
            .and_then(|col| {
                let s = col.as_materialized_series();
                let sc = s.cast(&DataType::UInt64)
                    .ok()?;
                let ca = sc.u64().ok()?;
                ca.get(0)
            })
            .unwrap_or(0) as usize
    };

    let get_bool = |c: &str| -> bool {
        stats_df
            .column(c)
            .ok()
            .and_then(|col| {
                let s = col.as_materialized_series();
                let ca = s.bool().ok()?;
                ca.get(0)
            })
            .unwrap_or(false)
    };

    let min = get_f64("min");
    let max = get_f64("max");
    let mean = get_f64("mean");
    let median = get_f64("median");
    let std_dev = get_f64("std");
    let q1 = get_f64("q1");
    let q3 = get_f64("q3");
    let p05 = get_f64("p05");
    let p95 = get_f64("p95");
    let distinct_count = get_usize("distinct_count");
    let zero_count = get_usize("zero_count");
    let negative_count = get_usize("negative_count");
    let is_integer = get_bool("is_integer");
    let null_count = get_usize("null_count");
    let count = get_usize("count");
    let sum = get_f64("sum").unwrap_or(0.0);

    if distinct_count > 0
        && distinct_count <= 3
        && min.unwrap_or(0.0) >= 0.0
        && max.unwrap_or(0.0) <= 1.0
    {
        let true_count = sum as usize;
        let false_count = count.saturating_sub(null_count).saturating_sub(true_count);
        return Ok((
            ColumnKind::Boolean,
            ColumnStats::Boolean(BooleanStats {
                true_count,
                false_count,
            }),
        ));
    }

    let skew = profiling::calculate_skew(mean, median, q1, q3, std_dev);

    // Use a larger sample for histogram and sorted checks (but only this column to save memory)
    let sample_column = lf
        .clone()
        .limit(adaptive_sample_size as u32)
        .select([col(name)])
        .collect()?;
    let sample_series = sample_column.column(name)?.as_materialized_series();

    let sample_ca = sample_series.cast(&DataType::Float64)?;
    let sample_ca = sample_ca.f64()?;

    let is_sorted = sample_series.is_sorted(Default::default())?;
    let is_sorted_rev = sample_series.is_sorted(SortOptions {
        descending: true,
        ..Default::default()
    })?;

    let trimmed_mean = profiling::calculate_trimmed_mean(sample_ca, mean, trim_pct);
    let (bin_width, histogram) =
        profiling::build_histogram_streaming(lf, name, min, max, q1, q3, count, null_count)?;

    Ok((
        ColumnKind::Numeric,
        ColumnStats::Numeric(NumericStats {
            min,
            distinct_count,
            p05,
            q1,
            median,
            mean,
            trimmed_mean,
            q3,
            p95,
            max,
            std_dev,
            skew,
            zero_count,
            negative_count,
            is_integer,
            is_sorted,
            is_sorted_rev,
            bin_width,
            histogram,
        }),
    ))
}

const MAX_UNIQUE_TRACKED: usize = 1000;
const CARDINALITY_SAMPLE_SIZE: usize = 10_000;

pub fn compute_categorical_stats(
    lf: LazyFrame,
    name: &str,
    adaptive_sample_size: usize,
) -> Result<(ColumnKind, ColumnStats)> {
    // Step 1: Quick cardinality estimate on sample
    let sample_uniques = lf
        .clone()
        .select([col(name)])
        .limit(CARDINALITY_SAMPLE_SIZE as u32)
        .collect()?
        .column(name)?
        .n_unique()?;

    if sample_uniques > MAX_UNIQUE_TRACKED {
        // High-cardinality: don't build full frequency map
        return Ok((
            ColumnKind::Categorical,
            ColumnStats::Categorical(HashMap::new()),
        ));
    }

    // Step 2: Low-cardinality: safe to build full HashMap
    let counts_df = lf
        .select([col(name)])
        .limit(adaptive_sample_size as u32)
        .group_by([col(name)])
        .agg([len().alias("count")])
        .sort(
            ["count"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(MAX_UNIQUE_TRACKED as u32)
        .collect()
        .context("Failed to compute categorical stats via LazyFrame")?;

    let mut freq: HashMap<String, usize> = HashMap::new();
    let s_val = counts_df.column(name)?;
    let s_count = counts_df.column("count")?;

    let s_val_str = s_val.cast(&DataType::String)?;
    let ca_val = s_val_str.as_materialized_series();
    let ca_val = ca_val.str()?;

    let ca_count = s_count.cast(&DataType::UInt64)?;
    let ca_count = ca_count.as_materialized_series();
    let ca_count = ca_count.u64()?;

    for (val, count) in ca_val.into_iter().zip(ca_count.into_iter()) {
        if let (Some(v), Some(c)) = (val, count) {
            freq.insert(v.to_owned(), c as usize);
        }
    }

    if sample_uniques > MAX_UNIQUE_TRACKED {
        freq.insert("__TRUNCATED__".to_owned(), 1);
    }

    Ok((ColumnKind::Categorical, ColumnStats::Categorical(freq)))
}

pub fn compute_boolean_stats(lf: LazyFrame, name: &str) -> Result<(ColumnKind, ColumnStats)> {
    let stats_df = lf
        .select([
            col(name).sum().alias("true_count"),
            col(name).null_count().alias("null_count"),
            len().alias("total_count"),
        ])
        .with_streaming(true)
        .collect()?;

    let get_usize = |c: &str| -> usize {
        stats_df
            .column(c)
            .ok()
            .and_then(|col| {
                let s = col.as_materialized_series();
                let sc = s.cast(&DataType::UInt64)
                    .ok()?;
                let ca = sc.u64().ok()?;
                ca.get(0)
            })
            .unwrap_or(0) as usize
    };

    let true_count = get_usize("true_count");
    let null_count = get_usize("null_count");
    let total_count = get_usize("total_count");
    let false_count = total_count.saturating_sub(null_count).saturating_sub(true_count);

    Ok((
        ColumnKind::Boolean,
        ColumnStats::Boolean(BooleanStats {
            true_count,
            false_count,
        }),
    ))
}

pub fn calculate_correlation_matrix(df: &DataFrame) -> Result<Option<CorrelationMatrix>> {
    calculate_correlation_matrix_lazy(df.clone().lazy())
}

#[expect(clippy::needless_range_loop, clippy::indexing_slicing)]
pub fn calculate_correlation_matrix_lazy(mut lf: LazyFrame) -> Result<Option<CorrelationMatrix>> {
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let mut numeric_cols: Vec<String> = schema
        .iter()
        .filter(|(_, dtype)| dtype.is_numeric())
        .map(|(name, _)| name.to_string())
        .collect();

    if numeric_cols.len() < 2 {
        return Ok(None);
    }

    // Limit columns for correlation to avoid OOM and massive matrices
    // Use adaptive limits: fewer columns for wide datasets to control memory
    let max_cols = if numeric_cols.len() > 100 {
        15  // Very wide: limit to 15 cols
    } else if numeric_cols.len() > 50 {
        20  // Wide: limit to 20 cols
    } else {
        30  // Normal: limit to 30 cols
    };

    if numeric_cols.len() > max_cols {
        numeric_cols.truncate(max_cols);
    }

    // Use a much smaller sample for correlation to save memory
    // Correlation calculations are memory-intensive (N^2 pairwise operations)
    // Statistical accuracy: 10k samples is sufficient for stable correlation estimates
    let total_rows = lf
        .clone()
        .select([len()])
        .collect()?
        .column("len")?
        .as_materialized_series()
        .cast(&DataType::UInt64)?
        .u64()?
        .get(0)
        .unwrap_or(0) as usize;

    let sample_size = if total_rows > 10_000 {
        10_000  // Reduced from 100k to 10k (10x memory reduction)
    } else {
        total_rows
    };

    let lf_sample = lf.limit(sample_size as u32);

    let mut exprs = Vec::new();
    for i in 0..numeric_cols.len() {
        for j in i + 1..numeric_cols.len() {
            let name_i = &numeric_cols[i];
            let name_j = &numeric_cols[j];
            exprs.push(
                polars::prelude::pearson_corr(col(name_i), col(name_j))
                    .alias(format!("{i}_{j}")),
            );
        }
    }

    let results = lf_sample.select(exprs).with_streaming(true).collect()?;

    let mut matrix = vec![vec![0.0; numeric_cols.len()]; numeric_cols.len()];
    for i in 0..numeric_cols.len() {
        matrix[i][i] = 1.0;
    }

    for i in 0..numeric_cols.len() {
        for j in i + 1..numeric_cols.len() {
            let col_name = format!("{i}_{j}");
            let val = results
                .column(&col_name)?
                .as_materialized_series()
                .f64()?
                .get(0)
                .unwrap_or(0.0);
            matrix[i][j] = val;
            matrix[j][i] = val;
        }
    }

    Ok(Some(CorrelationMatrix {
        columns: numeric_cols,
        data: matrix,
    }))
}
