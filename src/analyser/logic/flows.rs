use super::analysis::analyse_df;
use super::cleaning::clean_df_lazy;
use super::io::{load_df, load_df_lazy};
use super::types::{AnalysisResponse, ColumnCleanConfig};
use crate::analyser::db::DbClient;
use anyhow::{Context as _, Result};
use polars::prelude::*;
use sqlx::postgres::PgConnectOptions;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
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

    let _ = std::fs::remove_file(&temp_path);
    Ok(())
}

pub fn generate_auto_clean_configs(lf: LazyFrame) -> Result<HashMap<String, ColumnCleanConfig>> {
    let sample_df = lf
        .limit(500_000)
        .collect()
        .context("Failed to sample data for auto-cleaning")?;
    let summaries =
        analyse_df(&sample_df, 0.0).context("Failed to analyse sample for auto-cleaning")?;

    let mut configs = HashMap::new();
    for summary in summaries {
        let mut config = ColumnCleanConfig::default();
        config.new_name = summary.standardized_name.clone();
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    Ok(configs)
}

pub async fn analyze_file_flow(path: PathBuf, trim_pct: Option<f64>) -> Result<AnalysisResponse> {
    let start = std::time::Instant::now();
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let path_str = path.to_string_lossy().to_string();

    let mut lf = load_df_lazy(&path).context("Failed to probe file")?;
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let col_count = schema.len();

    let total_rows = match lf.clone().select([polars::prelude::len()]).collect() {
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

    let cell_count = total_rows * col_count;
    let (df, is_sampled) = if cell_count > 5_000_000 || file_size > 20 * 1024 * 1024 {
        crate::utils::log_event(
            "Analyser",
            &format!("Large dataset detected, using sampling for summary analysis..."),
        );
        (
            lf.limit(500_000)
                .collect()
                .context("Failed to sample data")?,
            true,
        )
    } else {
        (load_df(&path, &Arc::new(AtomicU64::new(0)))?, false)
    };

    let mut response = crate::analyser::logic::analysis::run_full_analysis(
        df,
        path_str,
        file_size,
        total_rows,
        trim_pct.unwrap_or(0.05),
        start,
    )?;

    if is_sampled {
        if let Some(first_col) = response.summary.get_mut(0) {
            first_col.business_summary.insert(0, format!("NOTE: This analysis is based on a sample of 500,000 rows due to large file size ({})", crate::utils::fmt_bytes(file_size)));
        }
    }

    Ok(response)
}
