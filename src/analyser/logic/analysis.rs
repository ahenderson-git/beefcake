use anyhow::{Context as _, Result};
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::types::{
    BooleanStats, ColumnCleanConfig, ColumnKind, ColumnStats, ColumnSummary, ImputeMode,
    NormalizationMethod, NumericStats, TemporalStats, TextStats,
};

pub type AnalysisReceiver = crossbeam_channel::Receiver<
    Result<(
        String,
        u64,
        Vec<ColumnSummary>,
        crate::analyser::logic::FileHealth,
        std::time::Duration,
        DataFrame,
    )>,
>;

pub fn load_df(path: &std::path::Path, progress: &Arc<AtomicU64>) -> Result<DataFrame> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut df = match ext.as_str() {
        "json" => {
            let file = std::fs::File::open(path).context("Failed to open JSON file")?;
            JsonReader::new(file)
                .finish()
                .context("Failed to parse JSON")?
        }
        "jsonl" | "ndjson" => JsonLineReader::from_path(path)
            .context("Failed to open JSONL file")?
            .finish()
            .context("Failed to parse JSONL")?,
        "parquet" => {
            let file = std::fs::File::open(path).context("Failed to open Parquet file")?;
            ParquetReader::new(file)
                .finish()
                .context("Failed to parse Parquet")?
        }
        _ => LazyCsvReader::new(path)
            .with_try_parse_dates(true)
            .finish()
            .context("Failed to initialize CSV reader")?
            .collect()
            .context("Failed to parse CSV data")?,
    };

    if ext == "json" || ext == "jsonl" || ext == "ndjson" {
        df = try_parse_temporal_columns(df).context("Failed to auto-parse temporal columns")?;
    }

    // Update progress to 100% since we loaded the whole thing
    let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
    progress.store(metadata.len(), Ordering::SeqCst);

    Ok(df)
}

pub fn try_parse_temporal_columns(df: DataFrame) -> Result<DataFrame> {
    let mut columns = df.get_columns().to_vec();
    let mut changed = false;

    for col in &mut columns {
        if col.dtype().is_string() {
            let s = col.as_materialized_series();

            // Try Datetime (Microseconds is a good default for Polars)
            if let Ok(dt) = s.cast(&DataType::Datetime(TimeUnit::Microseconds, None)) {
                // If the number of nulls didn't increase, it's a perfect match
                if dt.null_count() == s.null_count() && !s.is_empty() {
                    *col = Column::from(dt);
                    changed = true;
                    continue;
                }
            }

            // Try Date
            if let Ok(d) = s.cast(&DataType::Date) {
                if d.null_count() == s.null_count() && !s.is_empty() {
                    *col = Column::from(d);
                    changed = true;
                }
            }
        }
    }

    if changed {
        DataFrame::new(columns).context("Failed to reconstruct DataFrame after temporal parsing")
    } else {
        Ok(df)
    }
}

pub fn save_df(df: &mut DataFrame, path: &std::path::Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file = std::fs::File::create(path).context("Failed to create export file")?;

    match ext.as_str() {
        "parquet" => {
            ParquetWriter::new(file)
                .finish(df)
                .context("Failed to write Parquet file")?;
        }
        _ => {
            CsvWriter::new(file)
                .include_header(true)
                .finish(df)
                .context("Failed to write CSV file")?;
        }
    }

    Ok(())
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
            let head = series.drop_nulls().head(Some(10));
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
            let (k, s) = analyse_boolean(col).context(format!("Analysis failed for boolean column '{name}'"))?;
            (k, s, false)
        } else if dtype.is_numeric() {
            let (k, s) = analyse_numeric(col, trim_pct)
                .context(format!("Analysis failed for numeric column '{name}'"))?;
            (k, s, false)
        } else if dtype.is_temporal() {
            let (k, s) = analyse_temporal(col)
                .context(format!("Analysis failed for temporal column '{name}'"))?;
            (k, s, false)
        } else {
            analyse_text_or_fallback(&name, col)
                .context(format!("Analysis failed for text column '{name}'"))?
        };

        let mut summary = ColumnSummary {
            name,
            kind,
            count,
            nulls,
            has_special,
            stats,
            interpretation: Vec::new(),
            business_summary: Vec::new(),
            samples,
        };
        summary.interpretation = summary.generate_interpretation();
        summary.business_summary = summary.generate_business_summary();
        summaries.push(summary);
    }

    Ok(summaries)
}

#[expect(clippy::too_many_lines)]
pub fn clean_df(df: DataFrame, configs: &HashMap<String, ColumnCleanConfig>) -> Result<DataFrame> {
    let mut lf = df.lazy();
    let mut rename_map = Vec::new();
    let mut one_hot_cols = Vec::new();

    // Sort keys for deterministic processing order
    let mut sorted_keys: Vec<_> = configs.keys().collect();
    sorted_keys.sort();

    for old_name in sorted_keys {
        let config = configs
            .get(old_name)
            .context("Missing configuration for column")?;

        if !config.active {
            lf = lf.select([col("*").exclude([old_name.as_str()])]);
            continue;
        }

        let mut expr = col(old_name);

        // 1. Imputation (Must happen early)
        if config.ml_preprocessing {
            match config.impute_mode {
                ImputeMode::None => {}
                ImputeMode::Zero => {
                    expr = expr.fill_null(lit(0));
                }
                ImputeMode::Mean => {
                    expr = expr.clone().fill_null(expr.clone().mean());
                }
                ImputeMode::Median => {
                    expr = expr.clone().fill_null(expr.clone().median());
                }
                ImputeMode::Mode => {
                    expr = expr.clone().fill_null(expr.clone().mode().first());
                }
            }
        }

        // 2. Trim whitespace and tabs
        if config.advanced_cleaning && config.trim_whitespace {
            expr = expr.str().strip_chars(lit(Null {}));
        }

        // 3. Remove special characters (CR, Null, control chars)
        if config.advanced_cleaning && config.remove_special_chars {
            // Regex for common non-printable/control characters
            expr = expr.str().replace_all(lit(r"[\r\x00-\x1F]"), lit(""), true);
        }

        // 4. Change Datatypes
        if let Some(kind) = config.target_dtype {
            let dtype = match kind {
                ColumnKind::Numeric => DataType::Float64,
                ColumnKind::Boolean => DataType::Boolean,
                ColumnKind::Temporal => DataType::Datetime(TimeUnit::Microseconds, None),
                _ => DataType::String,
            };
            expr = expr.cast(dtype);
        }

        // 5. Normalization
        if config.ml_preprocessing {
            match config.normalization {
                NormalizationMethod::None => {}
                NormalizationMethod::ZScore => {
                    let mean = expr.clone().mean();
                    let std = expr.clone().std(1);
                    expr = (expr - mean) / std;
                }
                NormalizationMethod::MinMax => {
                    let min = expr.clone().min();
                    let max = expr.clone().max();
                    expr = (expr.clone() - min.clone()) / (max - min);
                }
            }
        }

        let needs_update = (config.advanced_cleaning && (config.trim_whitespace || config.remove_special_chars))
            || config.target_dtype.is_some()
            || (config.ml_preprocessing && (config.impute_mode != ImputeMode::None || config.normalization != NormalizationMethod::None));

        if needs_update {
            lf = lf.with_column(expr.alias(old_name));
        }

        // Queue renaming for the final step
        if !config.new_name.is_empty() && config.new_name != *old_name {
            rename_map.push((old_name.clone(), config.new_name.clone()));
        }

        // Queue One-Hot encoding (will use the name AFTER rename if any)
        if config.ml_preprocessing && config.one_hot_encode {
            let final_name = if !config.new_name.is_empty() {
                config.new_name.clone()
            } else {
                old_name.clone()
            };
            one_hot_cols.push(final_name);
        }
    }

    // 4. Change Column Names
    if !rename_map.is_empty() {
        let (old_names, new_names): (Vec<String>, Vec<String>) = rename_map.into_iter().unzip();
        lf = lf.rename(old_names, new_names, false);
    }

    let mut result_df = lf
        .collect()
        .context("Failed to apply cleaning steps to DataFrame")?;

    // 6. One-Hot Encoding (Manual Implementation)
    if !one_hot_cols.is_empty() {
        for col_name in one_hot_cols {
            let column = result_df
                .column(&col_name)
                .context("Failed to access column for One-Hot encoding")?
                .as_materialized_series();
            let s_str = column
                .cast(&DataType::String)
                .context("Failed to cast column to string for One-Hot encoding")?;
            let s_str = s_str
                .str()
                .context("Failed to access string data for One-Hot encoding")?;
            let unique_values = s_str
                .unique()
                .context("Failed to get unique values for One-Hot encoding")?;

            for val in unique_values.into_iter().flatten() {
                let dummy_col_name = format!("{col_name}_{val}");
                let dummy_series = s_str
                    .equal(val)
                    .into_series()
                    .cast(&DataType::Int32)
                    .context("Failed to create binary series for One-Hot encoding")?;
                result_df
                    .with_column(Column::from(dummy_series).with_name(dummy_col_name.into()))
                    .context("Failed to add One-Hot column to DataFrame")?;
            }
            result_df
                .drop_in_place(&col_name)
                .context("Failed to drop original column after One-Hot encoding")?;
        }
    }

    Ok(result_df)
}

fn analyse_boolean(col: &Column) -> Result<(ColumnKind, ColumnStats)> {
    let series = col.as_materialized_series();
    let ca = series.bool()?;
    let true_count = ca.sum().unwrap_or(0) as usize;
    let false_count = ca.len() - ca.null_count() - true_count;
    Ok((
        ColumnKind::Boolean,
        ColumnStats::Boolean(BooleanStats {
            true_count,
            false_count,
        }),
    ))
}

fn analyse_numeric(col: &Column, trim_pct: f64) -> Result<(ColumnKind, ColumnStats)> {
    let series = col.as_materialized_series();
    let f64_series = series.cast(&DataType::Float64)?;
    let ca = f64_series.f64()?;

    let min = ca.min();
    let max = ca.max();

    if let Some(res) = check_effective_boolean(series, ca, min, max)? {
        return Ok(res);
    }

    let mean = ca.mean();
    let std_dev = ca.std(1);

    let q1 = ca.quantile(0.25, QuantileMethod::Linear)?;
    let median = ca.median();
    let q3 = ca.quantile(0.75, QuantileMethod::Linear)?;
    let p05 = ca.quantile(0.05, QuantileMethod::Linear)?;
    let p95 = ca.quantile(0.95, QuantileMethod::Linear)?;

    let skew = calculate_skew(mean, median, q1, q3, std_dev);
    let trimmed_mean = calculate_trimmed_mean(ca, mean, trim_pct);
    let (bin_width, histogram) = calculate_histogram(ca, min, max, q1, q3);

    let mut zero_count = 0;
    let mut negative_count = 0;
    let mut is_integer = true;
    for v in ca.into_iter().flatten() {
        if v == 0.0 {
            zero_count += 1;
        }
        if v < 0.0 {
            negative_count += 1;
        }
        if is_integer && v.fract() != 0.0 {
            is_integer = false;
        }
    }

    // Sorting flags
    let is_sorted = series.is_sorted(SortOptions {
        descending: false,
        ..Default::default()
    })?;
    let is_sorted_rev = series.is_sorted(SortOptions {
        descending: true,
        ..Default::default()
    })?;

    Ok((
        ColumnKind::Numeric,
        ColumnStats::Numeric(NumericStats {
            min,
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

fn check_effective_boolean(
    series: &Series,
    ca: &Float64Chunked,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<Option<(ColumnKind, ColumnStats)>> {
    let distinct_count = series.n_unique()?;
    let has_nulls = series.null_count() > 0;
    let non_null_distinct = if has_nulls {
        distinct_count.saturating_sub(1)
    } else {
        distinct_count
    };

    let is_effective_bool = if non_null_distinct <= 2 {
        if let (Some(min_v), Some(max_v)) = (min, max) {
            (max_v == 0.0 || (max_v - 1.0).abs() < 1e-9) && min_v == 0.0
        } else {
            false
        }
    } else {
        false
    };

    if is_effective_bool {
        let true_count = ca
            .into_iter()
            .flatten()
            .filter(|&v| (v - 1.0).abs() < 1e-9)
            .count();
        let false_count = ca.len() - ca.null_count() - true_count;
        return Ok(Some((
            ColumnKind::Boolean,
            ColumnStats::Boolean(BooleanStats {
                true_count,
                false_count,
            }),
        )));
    }
    Ok(None)
}

fn calculate_skew(
    mean: Option<f64>,
    median: Option<f64>,
    q1: Option<f64>,
    q3: Option<f64>,
    std_dev: Option<f64>,
) -> Option<f64> {
    if let (Some(m), Some(md), Some(q1v), Some(q3v)) = (mean, median, q1, q3) {
        let iqr = q3v - q1v;
        if iqr > 0.0 {
            Some((m - md) / iqr)
        } else if let Some(s) = std_dev {
            if s > 0.0 {
                Some((m - md) / s)
            } else {
                Some(0.0)
            }
        } else {
            Some(0.0)
        }
    } else {
        None
    }
}

fn calculate_trimmed_mean(ca: &Float64Chunked, mean: Option<f64>, trim_pct: f64) -> Option<f64> {
    if trim_pct > 0.0 {
        let sorted = ca.sort(false);
        let n = sorted.len();
        let k = (n as f64 * trim_pct).floor() as usize;
        if k * 2 < n {
            let sliced = sorted.slice(k as i64, n - 2 * k);
            sliced.mean()
        } else {
            mean
        }
    } else {
        mean
    }
}

fn calculate_histogram(
    ca: &Float64Chunked,
    min: Option<f64>,
    max: Option<f64>,
    q1: Option<f64>,
    q3: Option<f64>,
) -> (f64, Vec<(f64, usize)>) {
    let mut histogram = Vec::new();
    let mut final_bin_width = 1.0;
    if let (Some(min_v), Some(max_v)) = (min, max) {
        let n = ca.len() - ca.null_count();
        if n > 0 {
            let iqr = q3.unwrap_or(0.0) - q1.unwrap_or(0.0);
            let mut bin_width = if iqr > 0.0 {
                2.0 * iqr / (n as f64).cbrt()
            } else {
                (max_v - min_v) / 20.0
            };
            if bin_width <= 0.0 {
                bin_width = 1.0;
            }

            if min_v < max_v {
                let mut bin_counts: HashMap<i64, usize> = HashMap::new();
                for val in ca.into_iter().flatten() {
                    let b = ((val - min_v) / bin_width).floor() as i64;
                    *bin_counts.entry(b).or_insert(0) += 1;
                }

                while bin_counts.len() > 1000 {
                    bin_width *= 2.0;
                    let mut new_counts = HashMap::new();
                    #[expect(clippy::iter_over_hash_type)]
                    for (b, count) in bin_counts {
                        let new_b = if b >= 0 { b / 2 } else { (b - 1) / 2 };
                        *new_counts.entry(new_b).or_insert(0) += count;
                    }
                    bin_counts = new_counts;
                }

                final_bin_width = bin_width;
                #[expect(clippy::iter_over_hash_type)]
                for (b, count) in bin_counts {
                    let center = min_v + (b as f64 + 0.5) * bin_width;
                    histogram.push((center, count));
                }
                histogram
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                final_bin_width = 1.0;
                let center = min_v;
                let step = 1.0;
                for i in -10..10 {
                    let c = center + i as f64 * step;
                    let count = if i == 0 { n } else { 0 };
                    histogram.push((c, count));
                }
            }
        }
    }
    (final_bin_width, histogram)
}

fn analyse_temporal(col: &Column) -> Result<(ColumnKind, ColumnStats)> {
    let series = col.as_materialized_series();
    let min_str = series
        .cast(&DataType::String)?
        .min_reduce()?
        .as_any_value()
        .get_str()
        .map(|s| s.to_owned());
    let max_str = series
        .cast(&DataType::String)?
        .max_reduce()?
        .as_any_value()
        .get_str()
        .map(|s| s.to_owned());

    let ts_ca = series.cast(&DataType::Int64)?;
    let ts_ca = ts_ca.i64()?;
    let min_ts = ts_ca.min();
    let max_ts = ts_ca.max();

    let p05 = ts_ca.quantile(0.05, QuantileMethod::Linear)?;
    let p95 = ts_ca.quantile(0.95, QuantileMethod::Linear)?;

    let is_sorted = series.is_sorted(SortOptions {
        descending: false,
        ..Default::default()
    })?;
    let is_sorted_rev = series.is_sorted(SortOptions {
        descending: true,
        ..Default::default()
    })?;

    let mut histogram = Vec::new();
    let mut final_bin_width = 1.0;

    if let (Some(min_v), Some(max_v)) = (min_ts, max_ts) {
        let n = ts_ca.len() - ts_ca.null_count();
        if n > 0 {
            let mut bin_width = (max_v - min_v) as f64 / 50.0;
            if bin_width <= 0.0 {
                bin_width = 1.0;
            }

            if min_v < max_v {
                let mut bin_counts: HashMap<i64, usize> = HashMap::new();
                for val in ts_ca.into_iter().flatten() {
                    let b = ((val - min_v) as f64 / bin_width).floor() as i64;
                    *bin_counts.entry(b).or_insert(0) += 1;
                }

                while bin_counts.len() > 1000 {
                    bin_width *= 2.0;
                    let mut new_counts = HashMap::new();
                    #[expect(clippy::iter_over_hash_type)]
                    for (b, count) in bin_counts {
                        let new_b = if b >= 0 { b / 2 } else { (b - 1) / 2 };
                        *new_counts.entry(new_b).or_insert(0) += count;
                    }
                    bin_counts = new_counts;
                }

                final_bin_width = bin_width;
                #[expect(clippy::iter_over_hash_type)]
                for (b, count) in bin_counts {
                    let center = min_v as f64 + (b as f64 + 0.5) * bin_width;
                    histogram.push((center, count));
                }
                histogram
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                final_bin_width = 1.0;
                histogram.push((min_v as f64, n));
            }
        }
    }

    Ok((
        ColumnKind::Temporal,
        ColumnStats::Temporal(TemporalStats {
            min: min_str,
            max: max_str,
            p05,
            p95,
            is_sorted,
            is_sorted_rev,
            bin_width: final_bin_width,
            histogram,
        }),
    ))
}

fn analyse_text_or_fallback(name: &str, col: &Column) -> Result<(ColumnKind, ColumnStats, bool)> {
    let series = col.as_materialized_series();
    let dtype = col.dtype();

    let kind = if matches!(dtype, DataType::List(_) | DataType::Struct(_)) {
        ColumnKind::Nested
    } else {
        ColumnKind::Text
    };

    let distinct = series.n_unique()?;
    let row_count = series.len();

    let (min_length, max_length, avg_length) = if dtype.is_string() {
        let ca = series.str()?;
        let lengths = ca.str_len_chars();
        (
            lengths.min().unwrap_or(0) as usize,
            lengths.max().unwrap_or(0) as usize,
            lengths.mean().unwrap_or(0.0),
        )
    } else {
        (0, 0, 0.0)
    };

    // Use sorted=true to get descending counts for top_value detection
    let value_counts_df = if distinct > 0 {
        Some(series.value_counts(true, true, "counts".into(), false)?)
    } else {
        None
    };

    let mut has_special = false;
    if dtype.is_string() {
        if let Some(vc) = &value_counts_df {
            let values = vc.column(name)?.as_materialized_series();
            let v_ca = values.cast(&DataType::String)?;
            let v_ca = v_ca.str()?;
            for v in v_ca.into_iter().flatten() {
                if v.chars()
                    .any(|c| c == '\r' || (c.is_control() && c != '\n' && c != '\t'))
                {
                    has_special = true;
                    break;
                }
            }
        }
    }

    // Categorical detection
    if kind == ColumnKind::Text
        && distinct > 0
        && distinct <= 25
        && (distinct < row_count || row_count == 1)
    {
        if let Some(vc) = &value_counts_df {
            let values = vc.column(name)?.as_materialized_series();
            let counts = vc.column("counts")?.as_materialized_series();

            let mut freq = HashMap::new();
            let v_ca = values.cast(&DataType::String)?;
            let v_ca = v_ca.str()?;
            let c_ca = counts.u32()?;

            for (v, c) in v_ca.into_iter().zip(c_ca.into_iter()) {
                if let (Some(v_str), Some(c_val)) = (v, c) {
                    freq.insert(v_str.to_owned(), c_val as usize);
                }
            }

            return Ok((
                ColumnKind::Categorical,
                ColumnStats::Categorical(freq),
                has_special,
            ));
        }
    }

    let top_value = if let Some(vc) = &value_counts_df {
        let values = vc.column(name)?.as_materialized_series();
        let counts = vc.column("counts")?.as_materialized_series();

        let v = values
            .cast(&DataType::String)
            .ok()
            .and_then(|s| s.str().ok().and_then(|ca| ca.get(0).map(|s| s.to_owned())));
        let c = counts
            .u32()
            .ok()
            .and_then(|ca| ca.get(0).map(|c| c as usize));

        if let (Some(v_str), Some(c_val)) = (v, c) {
            Some((v_str, c_val))
        } else {
            None
        }
    } else {
        None
    };

    Ok((
        kind,
        ColumnStats::Text(TextStats {
            distinct,
            top_value,
            min_length,
            max_length,
            avg_length,
        }),
        has_special,
    ))
}
