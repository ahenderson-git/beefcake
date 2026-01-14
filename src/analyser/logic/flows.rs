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

    crate::utils::log_event(
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
        analyse_df_lazy(lf, 0.0).context("Failed to analyse for auto-cleaning")?;

    let mut configs = HashMap::new();
    for summary in summaries {
        let mut config = ColumnCleanConfig::default();
        config.new_name = summary.standardised_name.clone();
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    Ok(configs)
}

pub async fn analyze_file_flow(path: PathBuf) -> Result<AnalysisResponse> {
    let start = std::time::Instant::now();
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let path_str = path.to_string_lossy().to_string();

    let lf = load_df_lazy(&path).context("Failed to probe file")?;
    let mut lf_for_schema = lf.clone();
    let schema = lf_for_schema.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let col_count = schema.len();

    // Use file size to determine sampling (avoid expensive row counting that materializes data)
    // Rule of thumb: files > 20MB or wide schemas (>50 cols) should be sampled
    let should_sample = file_size > 20 * 1024 * 1024 || col_count > 50;

    // Count true total rows from original LazyFrame (streaming, doesn't materialize)
    let true_total_rows = match lf.clone().select([polars::prelude::len()]).with_streaming(true).collect() {
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

    let max_cells = 5_000_000;
    let (lf_for_analysis, is_sampled, sampled_rows_count) = if should_sample {
        let sample_rows = if col_count > 0 {
            std::cmp::min(500_000, (max_cells / col_count) as u32)
        } else {
            500_000
        };
        crate::utils::log_event(
            "Analyser",
            &format!("Large dataset detected, using random sampling ({sample_rows} rows) for representative analysis..."),
        );
        // Use random sampling for better representativeness
        // Note: We need to collect, sample, and convert back to lazy for random sampling
        // For very large datasets, this may use more memory than .limit(), but provides
        // statistically better results. Consider using .limit() for datasets > 10M rows.
        let df = lf.clone().limit(sample_rows * 2).collect()?; // Collect 2x for sampling pool
        let n_series = Series::new("n".into(), &[sample_rows as i64]);
        // Use a fixed seed (42) to ensure deterministic sampling for consistent health scores
        let sampled_df = df.sample_n(&n_series, false, false, Some(42))?;
        (sampled_df.lazy(), true, sample_rows)
    } else {
        (lf.clone(), false, 0)
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
        start,
    )?;

    if is_sampled
        && let Some(first_col) = response.summary.get_mut(0) {
            first_col.business_summary.insert(0, format!("NOTE: This analysis is based on a sample of {} rows due to large dataset size ({})",
                crate::utils::fmt_count(sampled_rows_count as usize),
                crate::utils::fmt_bytes(file_size)));
        }

    Ok(response)
}
