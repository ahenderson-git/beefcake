use super::types::{ColumnCleanConfig, ColumnKind, ImputeMode, NormalizationMethod, TextCase};
use anyhow::{Context as _, Result};
use polars::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StatsValues {
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub mode: Option<f64>,
    pub std: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

pub fn clean_df(
    df: DataFrame,
    configs: &HashMap<String, ColumnCleanConfig>,
    restricted: bool,
) -> Result<DataFrame> {
    let lf = df.lazy();
    let cleaned_lf = clean_df_lazy(lf, configs, restricted)?;
    cleaned_lf
        .collect()
        .context("Failed to collect cleaned dataframe")
}

pub fn clean_df_lazy(
    lf: LazyFrame,
    configs: &HashMap<String, ColumnCleanConfig>,
    restricted: bool,
) -> Result<LazyFrame> {
    let mut lf = lf;
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let mut expressions = Vec::new();
    let mut one_hot_cols = Vec::new();

    for (name, _) in schema.iter() {
        if let Some(config) = configs.get(name.as_str()) {
            if !config.active {
                continue;
            }

            let mut expr = col(name.as_str());

            // 1. Text cleaning & Regex
            expr = apply_text_cleaning(expr, config, restricted);

            // 2. Extract numbers if requested (produces Float64)
            if config.extract_numbers {
                expr = expr
                    .str()
                    .extract(lit(r"(\d+\.?\d*)"), 1)
                    .cast(DataType::Float64);
            }

            // 3. Casting to target type
            expr = apply_dtype_casting(expr, config);

            // 4. Imputation
            expr = apply_imputation_with_stats(expr, config, None);

            // 5. Numeric Refinement (Clips, Rounding - NO extract_numbers here anymore)
            if !restricted {
                expr = apply_numeric_refinement(expr, config);
            }

            // 6. Normalization
            if !restricted {
                expr = apply_normalization_with_stats(expr, config, None);
            }

            // 6. Rename if needed
            if !config.new_name.is_empty() && config.new_name != *name {
                if !restricted {
                    expr = expr.alias(&config.new_name);
                } else {
                    expr = expr.alias(name.as_str());
                }
            } else {
                expr = expr.alias(name.as_str());
            }

            // 7. Categorical Refinement (One-hot encoding is handled separately)
            if config.ml_preprocessing && config.one_hot_encode {
                one_hot_cols.push(if config.new_name.is_empty() {
                    name.to_string()
                } else {
                    config.new_name.clone()
                });
            }

            expressions.push(expr);
        } else {
            expressions.push(col(name.as_str()));
        }
    }

    lf = lf.select(expressions);

    if !one_hot_cols.is_empty() {
        lf = apply_one_hot_encoding_lazy(lf, one_hot_cols)?;
    }

    Ok(lf)
}

pub fn auto_clean_df(df: DataFrame, restricted: bool) -> Result<DataFrame> {
    let mut configs = HashMap::new();
    for col_name in df.get_column_names() {
        let mut config = ColumnCleanConfig::default();
        config.active = true;
        config.trim_whitespace = true;
        configs.insert(col_name.to_string(), config);
    }
    clean_df(df, &configs, restricted)
}

pub fn apply_text_cleaning(expr: Expr, config: &ColumnCleanConfig, _restricted: bool) -> Expr {
    let mut expr = expr;

    if config.advanced_cleaning {
        if config.trim_whitespace {
            expr = expr.str().strip_chars(lit(NULL));
        }

        match config.text_case {
            TextCase::Lowercase => expr = expr.str().to_lowercase(),
            TextCase::Uppercase => expr = expr.str().to_uppercase(),
            TextCase::TitleCase => {}
            TextCase::None => {}
        }

        if config.remove_special_chars {
            expr = expr
                .str()
                .replace_all(lit(r"[^a-zA-Z0-9\s]"), lit(""), true);
        }

        if config.remove_non_ascii {
            expr = expr.str().replace_all(lit(r"[^\x00-\x7F]"), lit(""), true);
        }

        if !config.regex_find.is_empty() {
            expr = expr.str().replace_all(
                lit(config.regex_find.as_str()),
                lit(config.regex_replace.as_str()),
                true,
            );
        }

        if config.standardize_nulls {
            let null_values =
                Series::new("nulls".into(), &["null", "NULL", "", "N/A", "nan", "NaN"]);
            expr = when(expr.clone().is_in(lit(null_values)))
                .then(lit(NULL))
                .otherwise(expr);
        }
    }

    expr
}

pub fn apply_dtype_casting(expr: Expr, config: &ColumnCleanConfig) -> Expr {
    if let Some(kind) = config.target_dtype {
        match kind {
            ColumnKind::Numeric => expr.cast(DataType::Float64),
            ColumnKind::Text => expr.cast(DataType::String),
            ColumnKind::Boolean => {
                let lower = expr.cast(DataType::String).str().to_lowercase();
                when(
                    lower
                        .clone()
                        .eq(lit("true"))
                        .or(lower.clone().eq(lit("1")))
                        .or(lower.clone().eq(lit("yes"))),
                )
                .then(lit(true))
                .when(
                    lower
                        .clone()
                        .eq(lit("false"))
                        .or(lower.clone().eq(lit("0")))
                        .or(lower.eq(lit("no"))),
                )
                .then(lit(false))
                .otherwise(lit(NULL))
                .cast(DataType::Boolean)
            }
            ColumnKind::Temporal => expr.cast(DataType::Datetime(TimeUnit::Milliseconds, None)),
            ColumnKind::Categorical => expr.cast(DataType::Categorical(None, Default::default())),
            ColumnKind::Nested => expr,
        }
    } else {
        expr
    }
}

pub fn apply_imputation_with_stats(
    expr: Expr,
    config: &ColumnCleanConfig,
    _stats: Option<&StatsValues>,
) -> Expr {
    if !config.ml_preprocessing {
        return expr;
    }
    match config.impute_mode {
        ImputeMode::None => expr,
        ImputeMode::Zero => expr.fill_null(lit(0)),
        ImputeMode::Mean => expr.clone().fill_null(expr.mean()),
        ImputeMode::Median => expr.clone().fill_null(expr.median()),
        ImputeMode::Mode => expr.clone().fill_null(expr.mode().first()),
    }
}

pub fn apply_numeric_refinement(expr: Expr, config: &ColumnCleanConfig) -> Expr {
    let mut expr = expr;

    if config.ml_preprocessing && config.clip_outliers {
        let lower = expr.clone().quantile(lit(0.05), QuantileMethod::Linear);
        let upper = expr.clone().quantile(lit(0.95), QuantileMethod::Linear);
        expr = expr.clip(lower, upper);
    }

    if let Some(decimals) = config.rounding {
        expr = expr.round(decimals);
    }

    expr
}

pub fn apply_normalization_with_stats(
    expr: Expr,
    config: &ColumnCleanConfig,
    _stats: Option<&StatsValues>,
) -> Expr {
    if !config.ml_preprocessing {
        return expr;
    }
    match config.normalization {
        NormalizationMethod::None => expr,
        NormalizationMethod::MinMax => {
            let min = expr.clone().min();
            let max = expr.clone().max();
            (expr - min.clone()) / (max - min)
        }
        NormalizationMethod::ZScore => {
            let mean = expr.clone().mean();
            let std = expr.clone().std(1);
            (expr - mean) / std
        }
    }
}

pub fn apply_one_hot_encoding_lazy(lf: LazyFrame, _one_hot_cols: Vec<String>) -> Result<LazyFrame> {
    Ok(lf)
}
