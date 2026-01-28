use super::analysis::analyse_df_lazy;
use super::cleaning::clean_df_lazy;
use super::io::load_df_lazy;
use super::types::{AnalysisResponse, ColumnCleanConfig};
use crate::analyser::db::DbClient;
use anyhow::{Context as _, Result};
use polars::prelude::*;
use sqlx::postgres::PgConnectOptions;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Stratified sampling: Sample evenly across the entire file
/// For 1B rows, 500K sample: Takes rows at regular intervals across entire dataset
fn stratified_sample(lf: LazyFrame, total_rows: usize, sample_size: u32) -> Result<DataFrame> {
    // Calculate stride (how many rows to skip between samples)
    let stride = (total_rows / sample_size as usize).max(1);

    crate::config::log_event(
        "Analyser",
        &format!(
            "Using stratified sampling: every {}th row from {} total rows",
            stride,
            crate::utils::fmt_count(total_rows)
        ),
    );

    // Add row index and filter every Nth row
    let sampled = lf
        .with_row_index("__sample_idx__", None)
        .filter((col("__sample_idx__").cast(DataType::Int64) % lit(stride as i64)).eq(lit(0)))
        .limit(sample_size)
        .select([col("*").exclude(["__sample_idx__"])])
        .collect()?;

    Ok(sampled)
}

use crate::config::DbSettings;
use std::str::FromStr as _;

pub async fn test_connection_flow(settings: DbSettings, password: String) -> Result<String> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        settings.user, password, settings.host, settings.port, settings.database
    );
    let opts = PgConnectOptions::from_str(&url).context("Invalid connection URL")?;
    let _client = DbClient::connect(opts).await?;
    Ok("Connection successful".to_owned())
}

pub async fn push_to_db_flow(
    path: PathBuf,
    opts: PgConnectOptions,
    schema_name: String,
    table_name: String,
    configs: HashMap<String, ColumnCleanConfig>,
) -> Result<()> {
    let lf = load_df_lazy(&path).context("Failed to load data")?;

    let mut cleaned_lf = clean_df_lazy(lf, &configs, false).context("Cleaning failed")?;

    let schema = cleaned_lf
        .collect_schema()
        .map_err(|e| anyhow::anyhow!(e))?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("beefcake_db_push_{}.csv", Uuid::new_v4()));

    // Use RAII guard for automatic cleanup
    let _temp_guard = crate::utils::TempFileGuard::new(temp_path.clone());

    crate::config::log_event(
        "Database",
        "Sinking to temp CSV for database push (streaming)...",
    );

    cleaned_lf
        .with_streaming(true)
        .sink_csv(&temp_path, Default::default(), None)
        .context("Failed to sink to CSV for DB push")?;

    let client = DbClient::connect(opts).await?;
    client
        .push_from_csv_file(&temp_path, &schema, Some(&schema_name), Some(&table_name))
        .await?;

    // _temp_guard will automatically clean up the temp file when dropped
    Ok(())
}

pub fn generate_auto_clean_configs(lf: LazyFrame) -> Result<HashMap<String, ColumnCleanConfig>> {
    let summaries =
        analyse_df_lazy(lf, 0.0, 10_000).context("Failed to analyse for auto-cleaning")?;

    let mut configs = HashMap::new();
    for summary in summaries {
        let mut config = ColumnCleanConfig {
            new_name: summary.standardised_name.clone(),
            ..Default::default()
        };
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    Ok(configs)
}

pub async fn analyze_file_flow(path: PathBuf) -> Result<AnalysisResponse> {
    let start = std::time::Instant::now();
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let path_str = path.to_string_lossy().to_string();

    // Load config to get custom sample size
    let config = crate::config::load_app_config();
    let custom_sample_size = config.settings().analysis_sample_size as usize;

    let lf = load_df_lazy(&path).context("Failed to probe file")?;
    let mut lf_for_schema = lf.clone();
    let schema = lf_for_schema
        .collect_schema()
        .map_err(|e| anyhow::anyhow!(e))?;
    let col_count = schema.len();

    // Use file size to determine sampling (avoid expensive row counting that materializes data)
    // Rule of thumb: files > 20MB or wide schemas (>50 cols) should be sampled
    let should_sample = file_size > 20 * 1024 * 1024 || col_count > 50;

    // Count true total rows from original LazyFrame (streaming, doesn't materialize)
    let true_total_rows = match lf
        .clone()
        .select([polars::prelude::len()])
        .with_streaming(true)
        .collect()
    {
        Ok(df) => {
            let col = df.column("len")?.as_materialized_series();
            if let Ok(ca) = col.u32() {
                ca.get(0).unwrap_or(0) as usize
            } else if let Ok(ca) = col.u64() {
                ca.get(0).unwrap_or(0) as usize
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    // Use custom sample size as the target, but scale for very wide datasets to prevent OOM
    let target_sample_rows = custom_sample_size;
    let sampling_strategy = config.settings().sampling_strategy.as_str();

    let (lf_for_analysis, is_sampled, sampled_rows_count, sampling_method) = if should_sample {
        let sample_rows = if col_count > 100 {
            // For extremely wide datasets (>100 cols), use memory-safe formula
            // But ensure we meet at least 50% of user's target
            let memory_safe_rows = std::cmp::min(500_000, (5_000_000 / col_count) as u32);
            std::cmp::max(memory_safe_rows, (target_sample_rows / 2) as u32)
        } else if col_count > 50 {
            // For wide datasets (50-100 cols), use 75% of target
            std::cmp::min(500_000, ((target_sample_rows * 3) / 4) as u32)
        } else {
            // For normal datasets, use full target
            std::cmp::min(500_000, target_sample_rows as u32)
        };

        // Select sampling method based on strategy and file size
        let (sampled_df, method_used) = match (sampling_strategy, true_total_rows) {
            // Small files: Use fast method regardless of strategy
            (_, n) if n < 10_000_000 => {
                crate::config::log_event(
                    "Analyser",
                    &format!(
                        "Small dataset ({}), using fast sequential sampling",
                        crate::utils::fmt_count(n)
                    ),
                );
                let df = lf.clone().limit(sample_rows * 2).collect()?;
                let n_series = Series::new("n".into(), &[sample_rows as i64]);
                let sampled = df.sample_n(&n_series, false, false, Some(42))?;
                (sampled, "fast")
            }

            // Fast strategy: Always use current method
            ("fast", _) => {
                crate::config::log_event(
                    "Analyser",
                    &format!(
                        "Using fast sequential sampling: {sample_rows} from first {} rows",
                        sample_rows * 2
                    ),
                );
                let df = lf.clone().limit(sample_rows * 2).collect()?;
                let n_series = Series::new("n".into(), &[sample_rows as i64]);
                let sampled = df.sample_n(&n_series, false, false, Some(42))?;
                (sampled, "fast (sequential)")
            }

            // Balanced strategy (default): Use stratified for medium/large files
            ("balanced" | _, _) => {
                crate::config::log_event(
                    "Analyser",
                    &format!(
                        "Using stratified sampling: {} rows from {} total",
                        sample_rows,
                        crate::utils::fmt_count(true_total_rows)
                    ),
                );
                let sampled = stratified_sample(lf.clone(), true_total_rows, sample_rows)?;
                (sampled, "stratified")
            } // Note: "accurate" (reservoir) sampling will be implemented in Phase 2
        };

        (sampled_df.lazy(), true, sample_rows, method_used)
    } else {
        (lf.clone(), false, 0, "none")
    };

    // Count sampled rows (what we're actually analyzing)
    let sampled_rows = if is_sampled {
        sampled_rows_count as usize
    } else {
        true_total_rows
    };

    // Use fixed 5% trim for trimmed_mean calculation
    let mut response = crate::analyser::logic::analysis::run_full_analysis_streaming(
        lf_for_analysis,
        path_str,
        file_size,
        true_total_rows,
        sampled_rows,
        0.05,
        custom_sample_size,
        start,
    )?;

    if is_sampled && let Some(first_col) = response.summary.get_mut(0) {
        let sampling_description = match sampling_method {
            "fast" | "fast (sequential)" => "sequential sample from start of file",
            "stratified" => "stratified sample across entire file",
            "reservoir" => "random reservoir sample across entire file",
            _ => "sample",
        };

        let stats_note = if sampled_rows_count as usize != custom_sample_size {
            format!(
                " Statistics calculated from up to {} rows.",
                crate::utils::fmt_count(custom_sample_size)
            )
        } else {
            String::new()
        };

        first_col.business_summary.insert(0, format!(
            "NOTE: This analysis is based on a {} of {} rows ({} columns) from {} total rows ({}). Sampling method: {}.{}",
            sampling_description,
            crate::utils::fmt_count(sampled_rows_count as usize),
            col_count,
            crate::utils::fmt_count(true_total_rows),
            crate::utils::fmt_bytes(file_size),
            sampling_method,
            stats_note
        ));
    }

    Ok(response)
}
