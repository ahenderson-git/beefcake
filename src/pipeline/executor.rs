//! Pipeline execution engine.
//!
//! Executes pipeline specs against input data, applying transformations sequentially
//! and generating detailed run reports.

use super::spec::{ImputeStrategy, NormalisationMethod, OutputConfig, PipelineSpec, Step};
use super::validation::validate_pipeline;
use crate::analyser::logic::{get_parquet_write_options, load_df_lazy};
use anyhow::{Context as _, Result};
use chrono::Local;
use polars::prelude::*;
use std::path::{Path, PathBuf};

/// Report generated after pipeline execution
#[derive(Debug, Clone)]
pub struct RunReport {
    /// Number of rows before processing
    pub rows_before: usize,

    /// Number of columns before processing
    pub columns_before: usize,

    /// Number of rows after processing
    pub rows_after: usize,

    /// Number of columns after processing
    pub columns_after: usize,

    /// Number of steps successfully applied
    pub steps_applied: usize,

    /// Warnings generated during execution
    pub warnings: Vec<String>,

    /// Time taken for execution
    pub duration: std::time::Duration,
}

impl RunReport {
    /// Create a summary message
    pub fn summary(&self) -> String {
        format!(
            "Pipeline completed: {} rows ({} → {}), {} columns ({} → {}), {} steps, {:.2}s",
            if self.rows_after > self.rows_before {
                "added"
            } else if self.rows_after < self.rows_before {
                "removed"
            } else {
                "unchanged"
            },
            self.rows_before,
            self.rows_after,
            if self.columns_after > self.columns_before {
                "added"
            } else if self.columns_after < self.columns_before {
                "removed"
            } else {
                "unchanged"
            },
            self.columns_before,
            self.columns_after,
            self.steps_applied,
            self.duration.as_secs_f64()
        )
    }
}

/// Execute a pipeline spec on input data
pub fn run_pipeline(
    spec: &PipelineSpec,
    input_path: impl AsRef<Path>,
    output_path_override: Option<impl AsRef<Path>>,
) -> Result<RunReport> {
    let start = std::time::Instant::now();
    let mut warnings = Vec::new();

    // Load input data
    let mut input_lf = load_df_lazy(input_path.as_ref()).context("Failed to load input file")?;

    let input_schema = input_lf
        .collect_schema()
        .map_err(|e| anyhow::anyhow!("Failed to collect input schema: {e}"))?;

    let columns_before = input_schema.len();

    // Validate pipeline
    let validation_errors = validate_pipeline(spec, &input_schema)?;
    if !validation_errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Pipeline validation failed:\n{}",
            validation_errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    // Count input rows (streaming)
    let rows_before = count_rows(&input_lf)?;

    // Apply transformations
    let mut lf = input_lf;
    let mut steps_applied = 0;

    for (idx, step) in spec.steps.iter().enumerate() {
        match apply_step(step, lf.clone()) {
            Ok(new_lf) => {
                lf = new_lf;
                steps_applied += 1;
            }
            Err(e) => {
                warnings.push(format!("Step {}: {} (skipped)", idx + 1, e));
            }
        }
    }

    // Count output rows
    let rows_after = count_rows(&lf)?;
    let output_schema = lf
        .collect_schema()
        .map_err(|e| anyhow::anyhow!("Failed to collect output schema: {e}"))?;
    let columns_after = output_schema.len();

    // Determine output path
    let output_path = if let Some(override_path) = output_path_override {
        override_path.as_ref().to_path_buf()
    } else if !spec.output.path_template.is_empty() {
        expand_path_template(&spec.output.path_template)
    } else {
        return Err(anyhow::anyhow!(
            "No output path specified (provide --output or set output.path_template in spec)"
        ));
    };

    // Write output
    write_output(lf, &output_path, &spec.output)?;

    let duration = start.elapsed();

    Ok(RunReport {
        rows_before,
        columns_before,
        rows_after,
        columns_after,
        steps_applied,
        warnings,
        duration,
    })
}

/// Apply a single transformation step
fn apply_step(step: &Step, mut lf: LazyFrame) -> Result<LazyFrame> {
    match step {
        Step::DropColumns { columns } => {
            let cols_to_keep: Vec<_> = lf
                .collect_schema()
                .map_err(|e| anyhow::anyhow!(e))?
                .iter_names()
                .filter(|name| !columns.contains(&name.to_string()))
                .map(|name| col(name.as_str()))
                .collect();

            Ok(lf.select(cols_to_keep))
        }

        Step::RenameColumns { mapping } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if let Some(new_name) = mapping.get(name.as_str()) {
                        col(name.as_str()).alias(new_name)
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::TrimWhitespace { columns } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        col(name.as_str())
                            .str()
                            .strip_chars(lit(NULL))
                            .alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::CastTypes { columns: cast_map } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if let Some(type_str) = cast_map.get(name.as_str()) {
                        let target_type = parse_type_string(type_str)?;
                        Ok(col(name.as_str()).cast(target_type))
                    } else {
                        Ok(col(name.as_str()))
                    }
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(lf.select(exprs))
        }

        Step::ParseDates { columns: date_map } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if date_map.contains_key(name.as_str()) {
                        // Cast to datetime (format parsing would require strptime which needs format info)
                        col(name.as_str())
                            .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
                            .alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::Impute { strategy, columns } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        let expr = col(name.as_str());
                        let filled = match strategy {
                            ImputeStrategy::Zero => expr.fill_null(lit(0)),
                            ImputeStrategy::Mean => {
                                let mean_val = expr.clone().mean();
                                expr.fill_null(mean_val)
                            }
                            ImputeStrategy::Median => {
                                let median_val = expr.clone().median();
                                expr.fill_null(median_val)
                            }
                            ImputeStrategy::Mode => {
                                let mode_val = expr.clone().mode().first();
                                expr.fill_null(mode_val)
                            }
                        };
                        filled.alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::OneHotEncode {
            columns,
            drop_original,
        } => {
            // One-hot encoding requires collecting to get unique values
            // Apply for each column sequentially
            let mut result_lf = lf;
            for col_name in columns {
                result_lf = apply_one_hot_encoding(result_lf, col_name, *drop_original)?;
            }
            Ok(result_lf)
        }

        Step::NormaliseColumns { method, columns } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        let expr = col(name.as_str());
                        let normalized = match method {
                            NormalisationMethod::MinMax => {
                                let min_val = expr.clone().min();
                                let max_val = expr.clone().max();
                                (expr.clone() - min_val.clone()) / (max_val - min_val)
                            }
                            NormalisationMethod::ZScore => {
                                let mean_val = expr.clone().mean();
                                let std_val = expr.clone().std(1);
                                (expr.clone() - mean_val) / std_val
                            }
                        };
                        normalized.alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::ClipOutliers {
            columns,
            lower_quantile,
            upper_quantile,
        } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        let expr = col(name.as_str());
                        let lower = expr
                            .clone()
                            .quantile(lit(*lower_quantile), QuantileMethod::Linear);
                        let upper = expr
                            .clone()
                            .quantile(lit(*upper_quantile), QuantileMethod::Linear);
                        expr.clip(lower, upper).alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::ExtractNumbers { columns } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        col(name.as_str())
                            .str()
                            .extract(lit(r"(\d+\.?\d*)"), 1)
                            .cast(DataType::Float64)
                            .alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }

        Step::RegexReplace {
            columns,
            pattern,
            replacement,
        } => {
            let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
            let exprs: Vec<_> = schema
                .iter_names()
                .map(|name| {
                    if columns.contains(&name.to_string()) {
                        col(name.as_str())
                            .str()
                            .replace_all(lit(pattern.as_str()), lit(replacement.as_str()), true)
                            .alias(name.as_str())
                    } else {
                        col(name.as_str())
                    }
                })
                .collect();

            Ok(lf.select(exprs))
        }
    }
}

/// Apply one-hot encoding to a single column
fn apply_one_hot_encoding(
    mut lf: LazyFrame,
    col_name: &str,
    drop_original: bool,
) -> Result<LazyFrame> {
    // Collect to get unique values
    let df_temp = lf
        .clone()
        .select([col(col_name)])
        .collect()
        .context(format!(
            "Failed to collect column {col_name} for one-hot encoding"
        ))?;

    let series = df_temp.column(col_name)?;
    let unique_vals = series.unique()?.drop_nulls();

    let unique_strings: Vec<String> = unique_vals
        .str()
        .context("One-hot encoding requires string column")?
        .into_iter()
        .flatten()
        .map(std::borrow::ToOwned::to_owned)
        .collect();

    // Build expressions
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let mut expressions = Vec::new();

    // Add all existing columns (except original if dropping)
    for (name, _) in schema.iter() {
        if name.as_str() != col_name || !drop_original {
            expressions.push(col(name.as_str()));
        }
    }

    // Add one-hot encoded columns
    for val in unique_strings {
        let new_col_name = format!("{col_name}_{val}");
        expressions.push(
            when(col(col_name).eq(lit(val.as_str())))
                .then(lit(1i32))
                .otherwise(lit(0i32))
                .alias(&new_col_name),
        );
    }

    Ok(lf.select(expressions))
}

/// Count rows in a `LazyFrame` (streaming)
fn count_rows(lf: &LazyFrame) -> Result<usize> {
    let count_df = lf
        .clone()
        .select([len()])
        .with_streaming(true)
        .collect()
        .context("Failed to count rows")?;

    let col = count_df.column("len")?.as_materialized_series();

    if let Ok(ca) = col.u32() {
        Ok(ca.get(0).unwrap_or(0) as usize)
    } else if let Ok(ca) = col.u64() {
        Ok(ca.get(0).unwrap_or(0) as usize)
    } else {
        Ok(0)
    }
}

/// Expand path template with variables (e.g., {date})
fn expand_path_template(template: &str) -> PathBuf {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let expanded = template.replace("{date}", &today);
    PathBuf::from(expanded)
}

/// Write output to file based on configuration
fn write_output(lf: LazyFrame, path: &Path, config: &OutputConfig) -> Result<()> {
    // Check if file exists and overwrite setting
    if path.exists() && !config.overwrite {
        return Err(anyhow::anyhow!(
            "Output file already exists and overwrite is false: {}",
            path.display()
        ));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(format!(
            "Failed to create output directory: {}",
            parent.display()
        ))?;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or(&config.format)
        .to_lowercase();

    match ext.as_str() {
        "parquet" => {
            let options = get_parquet_write_options(&lf)?;
            lf.with_streaming(true)
                .sink_parquet(&path, options, None)
                .context("Failed to sink to parquet")?;
        }
        "csv" => {
            lf.with_streaming(true)
                .sink_csv(path, Default::default(), None)
                .context("Failed to sink to CSV")?;
        }
        "json" => {
            // JSON requires collecting (no streaming sink)
            let mut df = lf.collect().context("Failed to collect for JSON output")?;
            let file = std::fs::File::create(path).context("Failed to create JSON output file")?;
            JsonWriter::new(file)
                .with_json_format(JsonFormat::Json)
                .finish(&mut df)
                .context("Failed to write JSON")?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported output format: {ext}"));
        }
    }

    Ok(())
}

/// Parse type string to Polars `DataType`
fn parse_type_string(type_str: &str) -> Result<DataType> {
    match type_str {
        "i64" | "Numeric" => Ok(DataType::Int64),
        "f64" => Ok(DataType::Float64),
        "String" | "Text" => Ok(DataType::String),
        "Boolean" => Ok(DataType::Boolean),
        "Categorical" => Ok(DataType::Categorical(None, Default::default())),
        "Temporal" => Ok(DataType::Datetime(TimeUnit::Milliseconds, None)),
        _ => Err(anyhow::anyhow!("Unknown type string: {type_str}")),
    }
}
