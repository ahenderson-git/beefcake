use anyhow::{Context as _, Result};
use polars::prelude::NonExistent as NonExistentStrategy;
use polars::prelude::{
    DataType as PolarsDataType, QuantileMethod as PolarsQuantileMethod, TimeUnit as PolarsTimeUnit,
    *,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::types::{
    AnalysisResponse, BooleanStats, ColumnCleanConfig, ColumnKind, ColumnStats, ColumnSummary,
    CorrelationMatrix, ImputeMode, NormalizationMethod, NumericStats, TemporalStats, TextCase,
    TextStats,
};

pub fn run_full_analysis(
    df: DataFrame,
    path: String,
    file_size: u64,
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
        .unwrap_or("Unknown").to_owned();

    Ok(AnalysisResponse {
        file_name,
        path,
        file_size,
        row_count,
        column_count,
        summary,
        health,
        duration: start_time.elapsed(),
        df,
        correlation_matrix,
    })
}

pub fn sanitize_column_name(name: &str) -> String {
    let mut clean = name.trim().to_lowercase();

    // Replace non-alphanumeric with underscore
    clean = clean
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    // Collapse multiple underscores
    let mut result = String::new();
    let mut last_was_underscore = false;
    for c in clean.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
                last_was_underscore = true;
            }
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    // Trim underscores from ends
    let mut result = result.trim_matches('_').to_string();

    // Ensure it doesn't start with a number
    if !result.is_empty() && result.chars().next().unwrap().is_ascii_digit() {
        result = format!("col_{}", result);
    }

    if result.is_empty() {
        "col".to_string()
    } else {
        result
    }
}

pub fn sanitize_column_names(names: &[String]) -> Vec<String> {
    let mut cleaned_names = Vec::new();
    let mut seen = std::collections::HashMap::new();

    for name in names {
        let clean_base = sanitize_column_name(name);
        let mut clean = clean_base.clone();
        let mut count = 0;

        while seen.contains_key(&clean) {
            count += 1;
            clean = format!("{}_{}", clean_base, count);
        }

        seen.insert(clean.clone(), true);
        cleaned_names.push(clean);
    }
    cleaned_names
}

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
            if let Ok(dt) = s.cast(&PolarsDataType::Datetime(
                PolarsTimeUnit::Microseconds,
                None,
            )) {
                // If the number of nulls didn't increase, it's a perfect match
                if dt.null_count() == s.null_count() && !s.is_empty() {
                    *col = Column::from(dt);
                    changed = true;
                    continue;
                }
            }

            // Try Date
            if let Ok(d) = s.cast(&PolarsDataType::Date)
                && d.null_count() == s.null_count()
                && !s.is_empty()
            {
                *col = Column::from(d);
                changed = true;
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
        "json" => {
            JsonWriter::new(file)
                .finish(df)
                .context("Failed to write JSON file")?;
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
            match head.cast(&PolarsDataType::String) {
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
            let (k, s) = analyse_boolean(col)
                .context(format!("Analysis failed for boolean column '{name}'"))?;
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
    let sanitized_names = sanitize_column_names(&names);
    for (i, summary) in summaries.iter_mut().enumerate() {
        summary.standardized_name = sanitized_names[i].clone();
    }

    Ok(summaries)
}

#[expect(clippy::indexing_slicing)]
#[expect(clippy::needless_range_loop)]
pub fn calculate_correlation_matrix(df: &DataFrame) -> Result<Option<CorrelationMatrix>> {
    let numeric_cols: Vec<String> = df
        .get_column_names()
        .iter()
        .filter(|&name| {
            df.column(name)
                .map(|c| c.dtype().is_numeric())
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .collect();

    if numeric_cols.len() < 2 {
        return Ok(None);
    }

    let n = numeric_cols.len();
    let mut exprs = Vec::with_capacity(n * (n - 1) / 2);

    for i in 0..n {
        for j in (i + 1)..n {
            let col_a = &numeric_cols[i];
            let col_b = &numeric_cols[j];
            exprs.push(pearson_corr(col(col_a), col(col_b)).alias(format!("{i}_{j}")));
        }
    }

    let corr_df = df.clone().lazy().select(exprs).collect()?;

    let mut data = vec![vec![0.0; n]; n];
    for i in 0..n {
        data[i][i] = 1.0;
        for j in (i + 1)..n {
            let val = corr_df
                .column(&format!("{i}_{j}"))?
                .f64()?
                .get(0)
                .unwrap_or(0.0);
            data[i][j] = val;
            data[j][i] = val;
        }
    }

    Ok(Some(CorrelationMatrix {
        columns: numeric_cols,
        data,
    }))
}

pub fn clean_df(
    df: DataFrame,
    configs: &HashMap<String, ColumnCleanConfig>,
    restricted: bool,
) -> Result<DataFrame> {
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

        // 1. Basic Cleaning: Text operations (Must happen before casting)
        expr = apply_text_cleaning(expr, config, restricted);

        // 2. Change Datatypes
        expr = apply_dtype_casting(expr, config);

        if !restricted {
            // 3. Imputation (After casting so that Numeric/Boolean specific logic works)
            expr = apply_imputation(expr, config);

            // 4. Numeric Refinement (Rounding/Clipping)
            expr = apply_numeric_refinement(expr, config);

            // 5. Normalization
            expr = apply_normalization(expr, config);

            // 6. Categorical Refinement
            expr = apply_categorical_refinement(expr, config, old_name);
        }

        let needs_update = if restricted {
            config.advanced_cleaning || config.target_dtype.is_some()
        } else {
            config.advanced_cleaning || config.target_dtype.is_some() || config.ml_preprocessing
        };

        if needs_update {
            lf = lf.with_column(expr.alias(old_name));
        }

        if !restricted {
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
    }

    // 4. Change Column Names
    if !rename_map.is_empty() {
        let (old_names, new_names): (Vec<String>, Vec<String>) = rename_map.into_iter().unzip();
        lf = lf.rename(old_names, new_names, false);
    }

    let result_df = lf
        .collect()
        .context("Failed to apply cleaning steps to DataFrame")?;

    // 6. One-Hot Encoding (Manual Implementation)
    if !restricted {
        apply_one_hot_encoding(result_df, one_hot_cols)
    } else {
        Ok(result_df)
    }
}

pub fn auto_clean_df(df: DataFrame, restricted: bool) -> Result<DataFrame> {
    let summaries = analyse_df(&df, 0.0).context("Failed to analyse dataframe for cleaning")?;

    let original_names: Vec<String> = summaries.iter().map(|s| s.name.clone()).collect();
    let sanitized_names = sanitize_column_names(&original_names);

    let mut configs = HashMap::new();
    for (i, summary) in summaries.into_iter().enumerate() {
        let mut config = ColumnCleanConfig {
            new_name: sanitized_names[i].clone(),
            ..Default::default()
        };
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    clean_df(df, &configs, restricted).context("Failed to clean dataframe")
}

fn apply_text_cleaning(mut expr: Expr, config: &ColumnCleanConfig, restricted: bool) -> Expr {
    if !config.advanced_cleaning {
        return expr;
    }

    if config.trim_whitespace {
        expr = expr.str().strip_chars(lit(Null {}));
    }
    if config.remove_special_chars {
        expr = expr
            .str()
            .replace_all(lit(r"[\r\x00-\x1F]"), lit(""), false);
    }
    if config.remove_non_ascii {
        expr = expr.str().replace_all(lit(r"[^\x00-\x7F]"), lit(""), false);
    }
    if config.standardize_nulls {
        let null_patterns = ["n/a", "null", "none", "-", "nan"];
        let patterns_s = Series::new("p".into(), &null_patterns);
        expr = when(expr.clone().str().to_lowercase().is_in(lit(patterns_s)))
            .then(lit(Null {}))
            .otherwise(expr);
    }
    if config.extract_numbers {
        expr = expr
            .str()
            .replace_all(lit(r"[$,£€]"), lit(""), false)
            .str()
            .replace_all(lit(","), lit(""), false);
    }

    if !restricted {
        if !config.regex_find.is_empty() {
            expr = expr.str().replace_all(
                lit(config.regex_find.as_str()),
                lit(config.regex_replace.as_str()),
                false,
            );
        }
        match config.text_case {
            TextCase::Lowercase => expr = expr.str().to_lowercase(),
            TextCase::Uppercase => expr = expr.str().to_uppercase(),
            TextCase::TitleCase | TextCase::None => {}
        }
    }
    expr
}

fn apply_dtype_casting(mut expr: Expr, config: &ColumnCleanConfig) -> Expr {
    let Some(kind) = config.target_dtype else {
        return expr;
    };

    if kind == ColumnKind::Temporal && !config.temporal_format.is_empty() {
        expr = expr.str().to_datetime(
            Some(PolarsTimeUnit::Microseconds),
            None,
            StrptimeOptions {
                format: Some(config.temporal_format.clone().into()),
                strict: false,
                cache: true,
                exact: true,
            },
            lit(Null {}),
        );
    } else {
        let dtype = match kind {
            ColumnKind::Numeric => PolarsDataType::Float64,
            ColumnKind::Boolean => PolarsDataType::Boolean,
            ColumnKind::Temporal => PolarsDataType::Datetime(PolarsTimeUnit::Microseconds, None),
            _ => PolarsDataType::String,
        };
        if kind == ColumnKind::Boolean {
            // Handle common string representations of booleans
            let s_expr = expr.cast(PolarsDataType::String).str().to_lowercase();
            expr = when(s_expr.clone().is_in(lit(Series::new(
                "b".into(),
                &["true", "yes", "1", "y", "t"],
            ))))
            .then(lit(true))
            .when(s_expr.is_in(lit(Series::new(
                "b".into(),
                &["false", "no", "0", "n", "f"],
            ))))
            .then(lit(false))
            .otherwise(lit(Null {}));
        } else {
            expr = expr.cast(dtype);
        }
    }

    if kind == ColumnKind::Temporal && config.timezone_utc {
        expr = expr.dt().replace_time_zone(
            Some("UTC".into()),
            lit(Null {}),
            NonExistentStrategy::Null,
        );
    }
    expr
}

fn apply_imputation(mut expr: Expr, config: &ColumnCleanConfig) -> Expr {
    if !config.ml_preprocessing {
        return expr;
    }

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
    expr
}

fn apply_numeric_refinement(mut expr: Expr, config: &ColumnCleanConfig) -> Expr {
    if !config.ml_preprocessing {
        return expr;
    }

    if let Some(decimals) = config.rounding {
        expr = expr.round(decimals);
    }
    if config.clip_outliers {
        let lower = expr
            .clone()
            .quantile(lit(0.05), PolarsQuantileMethod::Linear);
        let upper = expr
            .clone()
            .quantile(lit(0.95), PolarsQuantileMethod::Linear);
        expr = expr.clip(lower, upper);
    }
    expr
}

fn apply_normalization(mut expr: Expr, config: &ColumnCleanConfig) -> Expr {
    if !config.ml_preprocessing {
        return expr;
    }

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
    expr
}

fn apply_categorical_refinement(
    mut expr: Expr,
    config: &ColumnCleanConfig,
    old_name: &str,
) -> Expr {
    if config.ml_preprocessing
        && let Some(threshold) = config.freq_threshold
    {
        expr = when(
            expr.clone()
                .count()
                .over([col(old_name)])
                .lt(lit(threshold as u32)),
        )
        .then(lit("Other"))
        .otherwise(expr);
    }
    expr
}

fn apply_one_hot_encoding(mut df: DataFrame, one_hot_cols: Vec<String>) -> Result<DataFrame> {
    let mut new_columns = Vec::new();
    let mut cols_to_drop = Vec::new();

    for col_name in one_hot_cols {
        let column = df
            .column(&col_name)
            .context("Failed to access column for One-Hot encoding")?
            .as_materialized_series();
        let s_str = column
            .cast(&PolarsDataType::String)
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
                .cast(&PolarsDataType::Int32)
                .context("Failed to create binary series for One-Hot encoding")?;
            new_columns.push(Column::from(dummy_series).with_name(dummy_col_name.into()));
        }
        cols_to_drop.push(col_name);
    }

    if !new_columns.is_empty() {
        df.hstack_mut(&new_columns)?;
    }

    for col_name in cols_to_drop {
        df.drop_in_place(&col_name)?;
    }

    Ok(df)
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
    let distinct_count = series.n_unique()?;
    let f64_series = series.cast(&PolarsDataType::Float64)?;
    let ca = f64_series.f64()?;

    let min = ca.min();
    let max = ca.max();

    if let Some(res) = check_effective_boolean(series, ca, min, max)? {
        return Ok(res);
    }

    let mean = ca.mean();
    let std_dev = ca.std(1);

    let q1 = ca.quantile(0.25, PolarsQuantileMethod::Linear)?;
    let median = ca.median();
    let q3 = ca.quantile(0.75, PolarsQuantileMethod::Linear)?;
    let p05 = ca.quantile(0.05, PolarsQuantileMethod::Linear)?;
    let p95 = ca.quantile(0.95, PolarsQuantileMethod::Linear)?;

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
    let distinct_count = series.n_unique()?;
    let min_str = series
        .cast(&PolarsDataType::String)?
        .min_reduce()?
        .as_any_value()
        .get_str()
        .map(|s| s.to_owned());
    let max_str = series
        .cast(&PolarsDataType::String)?
        .max_reduce()?
        .as_any_value()
        .get_str()
        .map(|s| s.to_owned());

    let ts_ca = series.cast(&PolarsDataType::Int64)?;
    let ts_ca = ts_ca.i64()?;
    let min_ts = ts_ca.min();
    let max_ts = ts_ca.max();

    let p05 = ts_ca.quantile(0.05, PolarsQuantileMethod::Linear)?;
    let q1 = ts_ca.quantile(0.25, PolarsQuantileMethod::Linear)?;
    let q3 = ts_ca.quantile(0.75, PolarsQuantileMethod::Linear)?;
    let p95 = ts_ca.quantile(0.95, PolarsQuantileMethod::Linear)?;

    let is_sorted = series.is_sorted(SortOptions {
        descending: false,
        ..Default::default()
    })?;
    let is_sorted_rev = series.is_sorted(SortOptions {
        descending: true,
        ..Default::default()
    })?;

    let f64_ts = ts_ca.cast(&PolarsDataType::Float64)?;
    let f64_ts_ca = f64_ts.f64()?;
    let (final_bin_width, histogram) = calculate_histogram(
        f64_ts_ca,
        min_ts.map(|v| v as f64),
        max_ts.map(|v| v as f64),
        q1,
        q3,
    );

    Ok((
        ColumnKind::Temporal,
        ColumnStats::Temporal(TemporalStats {
            min: min_str,
            max: max_str,
            distinct_count,
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

    let kind = if matches!(dtype, PolarsDataType::List(_) | PolarsDataType::Struct(_)) {
        ColumnKind::Nested
    } else {
        ColumnKind::Text
    };

    let distinct = series.n_unique()?;
    let row_count = series.len();

    let (min_length, max_length, avg_length) = get_text_lengths(series, dtype)?;

    // Use sorted=true to get descending counts for top_value detection
    let value_counts_df = if distinct > 0 {
        Some(series.value_counts(true, true, "counts".into(), false)?)
    } else {
        None
    };

    let has_special = check_special_characters(name, dtype, &value_counts_df)?;

    // Categorical detection
    if kind == ColumnKind::Text
        && distinct > 0
        && distinct <= 25
        && (distinct < row_count || row_count == 1)
        && let Some(vc) = &value_counts_df
    {
        let values = vc.column(name)?.as_materialized_series();
        let counts = vc.column("counts")?.as_materialized_series();

        let mut freq = HashMap::new();
        let v_ca = values.cast(&PolarsDataType::String)?;
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

    let top_value = if let Some(vc) = &value_counts_df {
        let values = vc.column(name)?.as_materialized_series();
        let counts = vc.column("counts")?.as_materialized_series();

        let v = values.cast(&PolarsDataType::String).ok().and_then(|s| {
            let ca = s.str().ok()?;
            ca.get(0).map(|s| s.to_owned())
        });
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

fn get_text_lengths(series: &Series, dtype: &PolarsDataType) -> Result<(usize, usize, f64)> {
    if dtype.is_string() {
        let ca = series.str()?;
        let lengths = ca.str_len_chars();
        Ok((
            lengths.min().unwrap_or(0) as usize,
            lengths.max().unwrap_or(0) as usize,
            lengths.mean().unwrap_or(0.0),
        ))
    } else if let PolarsDataType::List(_) = dtype {
        let lengths = series.list()?.lst_lengths();
        Ok((
            lengths.min().unwrap_or(0) as usize,
            lengths.max().unwrap_or(0) as usize,
            lengths.mean().unwrap_or(0.0),
        ))
    } else {
        Ok((0, 0, 0.0))
    }
}

fn check_special_characters(
    name: &str,
    dtype: &PolarsDataType,
    value_counts_df: &Option<DataFrame>,
) -> Result<bool> {
    if dtype.is_string()
        && let Some(vc) = value_counts_df
    {
        let values = vc.column(name)?.as_materialized_series();
        let v_ca = values.cast(&PolarsDataType::String)?;
        let v_ca = v_ca.str()?;
        for v in v_ca.into_iter().flatten() {
            if v.chars()
                .any(|c| c == '\r' || (c.is_control() && c != '\n' && c != '\t'))
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
