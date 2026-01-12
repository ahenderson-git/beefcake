use anyhow::{Context as _, Result};
use polars::prelude::*;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub fn load_df(path: &std::path::Path, _progress: &Arc<AtomicU64>) -> Result<DataFrame> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let df = match ext.as_str() {
        "csv" => LazyCsvReader::new(path)
            .with_infer_schema_length(Some(10000))
            .with_has_header(true)
            .finish()?
            .collect()
            .context("Failed to read CSV")?,
        "parquet" => ParquetReader::new(std::fs::File::open(path)?)
            .finish()
            .context("Failed to read Parquet")?,
        "json" => JsonReader::new(std::fs::File::open(path)?)
            .finish()
            .context("Failed to read JSON")?,
        _ => return Err(anyhow::anyhow!("Unsupported file extension: {ext}")),
    };

    try_parse_temporal_columns(df)
}

pub fn try_parse_temporal_columns(df: DataFrame) -> Result<DataFrame> {
    let mut df = df;
    let schema = df.schema();

    for (name, dtype) in schema.iter() {
        if dtype.is_numeric() || dtype.is_temporal() || dtype.is_bool() {
            continue;
        }

        // Try parsing string columns as datetime
        if let Ok(s) = df.column(name) {
            let s = s.as_materialized_series();
            if let Ok(casted) = s.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
                && casted.null_count() < s.len() / 2 {
                    let _ = df.replace(name, casted);
                }
        }
    }
    Ok(df)
}

pub fn save_df(df: &mut DataFrame, path: &std::path::Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext.as_str() == "parquet" {
        let file = std::fs::File::create(path).context("Failed to create Parquet file")?;
        ParquetWriter::new(file)
            .finish(df)
            .context("Failed to write Parquet file")?;
    } else {
        let file = std::fs::File::create(path).context("Failed to create CSV file")?;
        CsvWriter::new(file)
            .include_header(true)
            .finish(df)
            .context("Failed to write CSV file")?;
    }

    Ok(())
}

pub fn get_parquet_write_options(lf: &LazyFrame) -> Result<ParquetWriteOptions> {
    // Adaptive row group sizing based on column count to prevent OOM on large/wide datasets
    let schema = lf
        .clone()
        .collect_schema()
        .map_err(|e| anyhow::anyhow!("Failed to collect schema: {e}"))?;
    let col_count = schema.len();

    // Conservative row group sizing to prevent OOM on large datasets
    let mut row_group_size = if col_count >= 100 { 16_384 } else { 32_768 };

    // Allow environment variable override for emergency debugging
    if let Ok(env_val) = std::env::var("BEEFCAKE_PARQUET_ROW_GROUP_SIZE")
        && let Ok(parsed) = env_val.parse::<usize>() {
            row_group_size = parsed;
        }

    Ok(ParquetWriteOptions {
        maintain_order: false,
        row_group_size: Some(row_group_size),
        ..Default::default()
    })
}

pub fn load_df_lazy(path: &std::path::Path) -> Result<LazyFrame> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "csv" => LazyCsvReader::new(path)
            .with_infer_schema_length(Some(10000))
            .with_has_header(true)
            .with_try_parse_dates(true)
            .finish()
            .context("Failed to scan CSV"),
        "parquet" => {
            LazyFrame::scan_parquet(path, Default::default()).context("Failed to scan Parquet")
        }
        "json" => {
            // Polars doesn't have a truly lazy JSON reader in the same way as CSV/Parquet (it usually reads it all)
            // but we can read it and then convert to lazy.
            let df = JsonReader::new(std::fs::File::open(path)?)
                .finish()
                .context("Failed to read JSON")?;
            Ok(df.lazy())
        }
        _ => Err(anyhow::anyhow!("Unsupported file extension: {ext}")),
    }
}
