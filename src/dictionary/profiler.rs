//! Technical metadata profiler for data dictionary snapshots.
//!
//! Extracts and analyzes dataset characteristics to populate immutable technical metadata.

use super::metadata::{
    ColumnBusinessMetadata, ColumnMetadata, ColumnTechnicalMetadata, DataDictionary,
    DatasetBusinessMetadata, DatasetMetadata, InputSource, QualitySummary, TechnicalMetadata,
    column_name_to_uuid,
};
use crate::analyser::logic::{AnalysisResponse, ColumnSummary, analyse_df_lazy};
use anyhow::{Context as _, Result};
use chrono::Utc;
use polars::prelude::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Create a complete data dictionary snapshot from a dataset.
///
/// # Arguments
///
/// * `dataset_name` - Human-readable name for the dataset
/// * `df` - The output `DataFrame` after transformations
/// * `input_path` - Path to original input file
/// * `output_path` - Path where output will be written
/// * `pipeline_json` - Optional JSON of the pipeline that produced this dataset
/// * `previous_snapshot_id` - Optional link to previous snapshot for versioning
///
/// # Returns
///
/// A complete `DataDictionary` with technical metadata populated and empty business metadata.
pub fn create_snapshot(
    dataset_name: &str,
    df: &DataFrame,
    input_path: PathBuf,
    output_path: PathBuf,
    pipeline_json: Option<String>,
    previous_snapshot_id: Option<Uuid>,
) -> Result<DataDictionary> {
    let snapshot_id = Uuid::new_v4();
    let export_timestamp = Utc::now();

    // Calculate output hash
    let output_dataset_hash = calculate_dataframe_hash(df);

    // Determine export format from output path
    let export_format = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    // Calculate input hash if file exists
    let input_hash = calculate_file_hash(&input_path).ok();
    let input_sources = vec![InputSource {
        path: input_path.to_string_lossy().to_string(),
        hash: input_hash.clone(),
    }];

    // Calculate input dataset hash (use file hash as proxy)
    let input_dataset_hash = input_hash;

    // Profile the DataFrame to get column statistics
    let analysis = analyse_dataframe_for_dictionary(df)?;

    // Build column metadata from analysis
    let columns = build_column_metadata(&analysis.summary)?;

    // Calculate quality summary
    let quality_summary = calculate_quality_summary(&analysis, &columns);

    // Build technical metadata
    let technical = TechnicalMetadata {
        input_sources,
        pipeline_id: None, // Will be set by caller if available
        pipeline_json,
        input_dataset_hash,
        output_dataset_hash,
        row_count: df.height(),
        column_count: df.width(),
        export_format,
        quality_summary,
    };

    // Create dataset metadata with empty business metadata
    let dataset_metadata = DatasetMetadata {
        technical,
        business: DatasetBusinessMetadata::default(),
    };

    Ok(DataDictionary {
        snapshot_id,
        dataset_name: dataset_name.to_owned(),
        export_timestamp,
        dataset_metadata,
        columns,
        previous_snapshot_id,
    })
}

/// Profile a `DataFrame` for dictionary creation (lightweight analysis).
fn analyse_dataframe_for_dictionary(df: &DataFrame) -> Result<AnalysisResponse> {
    // Convert to LazyFrame for efficient analysis
    let lf = df.clone().lazy();

    // Use existing analysis logic but with minimal sampling
    // Note: trim_pct of 0.0 means no trimming for the dictionary snapshot
    let summary = analyse_df_lazy(lf, 0.0).context("Failed to analyze DataFrame for dictionary")?;

    Ok(AnalysisResponse {
        file_name: "dictionary_snapshot".to_owned(),
        path: String::new(),
        file_size: 0,
        row_count: df.height(),
        total_row_count: df.height(),
        column_count: df.width(),
        summary,
        health: crate::analyser::logic::FileHealth {
            score: 100.0,
            risks: vec![],
        },
        duration: std::time::Duration::from_secs(0),
        df: df.clone(),
        correlation_matrix: None,
    })
}

/// Build column metadata from analysis summary.
fn build_column_metadata(summary: &[ColumnSummary]) -> Result<Vec<ColumnMetadata>> {
    summary
        .iter()
        .map(|col| {
            let column_id = column_name_to_uuid(&col.name);

            // Serialize stats to JSON for storage
            let stats_json = serde_json::to_string(&col.stats).ok();

            // Extract min/max values based on column kind
            let (min_value, max_value) = extract_min_max_from_stats(col);

            // Detect warnings based on analysis
            let warnings = detect_column_warnings(col);

            let technical = ColumnTechnicalMetadata {
                data_type: col.kind.to_string(),
                nullable: col.nulls > 0,
                null_percentage: col.null_pct(),
                distinct_count: col.stats.n_distinct(),
                min_value,
                max_value,
                sample_values: col.samples.clone(),
                warnings,
                stats_json,
            };

            Ok(ColumnMetadata {
                column_id,
                current_name: col.name.clone(),
                original_name: if col.name == col.standardised_name {
                    None
                } else {
                    Some(col.standardised_name.clone())
                },
                technical,
                business: ColumnBusinessMetadata::default(),
            })
        })
        .collect()
}

/// Extract min/max values from column stats.
fn extract_min_max_from_stats(col: &ColumnSummary) -> (Option<String>, Option<String>) {
    use crate::analyser::logic::ColumnStats;

    match &col.stats {
        ColumnStats::Numeric(stats) => (
            stats.min.map(|v| v.to_string()),
            stats.max.map(|v| v.to_string()),
        ),
        ColumnStats::Temporal(stats) => (stats.min.clone(), stats.max.clone()),
        _ => (None, None),
    }
}

/// Detect warnings about column quality.
fn detect_column_warnings(col: &ColumnSummary) -> Vec<String> {
    let mut warnings = Vec::new();

    // High missingness
    if col.null_pct() > 50.0 {
        warnings.push(format!(
            "High missingness: {:.1}% null values",
            col.null_pct()
        ));
    }

    // Constant column
    if col.stats.n_distinct() == 1 {
        warnings.push("Constant column: only one distinct value".to_owned());
    }

    // Potential ID column (high uniqueness)
    let uniqueness = col.uniqueness_ratio();
    if uniqueness > 0.95 && col.stats.n_distinct() > 100 {
        warnings.push(format!(
            "ID-like column: {:.1}% unique values",
            uniqueness * 100.0
        ));
    }

    // Copy interpretation warnings
    warnings.extend(
        col.interpretation
            .iter()
            .filter(|s| s.contains("warning") || s.contains("Warning") || s.contains("caution"))
            .cloned(),
    );

    warnings
}

/// Calculate overall quality summary for the dataset.
fn calculate_quality_summary(
    analysis: &AnalysisResponse,
    columns: &[ColumnMetadata],
) -> QualitySummary {
    let total_columns = columns.len() as f64;

    // Calculate average null percentage
    let avg_null_percentage = if total_columns > 0.0 {
        columns
            .iter()
            .map(|c| c.technical.null_percentage)
            .sum::<f64>()
            / total_columns
    } else {
        0.0
    };

    // Count empty columns (100% null)
    let empty_column_count = columns
        .iter()
        .filter(|c| c.technical.null_percentage >= 100.0)
        .count();

    // Count constant columns
    let constant_column_count = columns
        .iter()
        .filter(|c| c.technical.distinct_count <= 1)
        .count();

    // Use file health score if available, otherwise calculate
    let overall_score = analysis.health.score;

    QualitySummary {
        avg_null_percentage,
        empty_column_count,
        constant_column_count,
        duplicate_row_count: None, // TODO: Could calculate if needed
        overall_score: overall_score.into(),
    }
}

/// Calculate hash of a `DataFrame`'s content for versioning.
fn calculate_dataframe_hash(df: &DataFrame) -> String {
    use sha2::{Digest as _, Sha256};

    let mut hasher = Sha256::new();

    // Hash schema
    hasher.update(format!("{:?}", df.schema()).as_bytes());

    // Hash row count
    hasher.update(df.height().to_string().as_bytes());

    // Hash sample of data (first few rows)
    let sample_size = df.height().min(100);
    for i in 0..sample_size {
        if let Some(row) = df.get(i) {
            hasher.update(format!("{row:?}").as_bytes());
        }
    }

    let hash = hasher.finalize();
    format!("{hash:x}")
}

/// Calculate hash of a file's content.
fn calculate_file_hash(path: &Path) -> Result<String> {
    use sha2::{Digest as _, Sha256};
    use std::fs::File;
    use std::io::{BufReader, Read as _};

    let file = File::open(path).context("Failed to open file for hashing")?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer).context("Failed to read file")?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("{hash:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_snapshot_basic() -> Result<()> {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }?;

        let snapshot = create_snapshot(
            "test_dataset",
            &df,
            PathBuf::from("input.csv"),
            PathBuf::from("output.csv"),
            None,
            None,
        )?;

        assert_eq!(snapshot.dataset_name, "test_dataset");
        assert_eq!(snapshot.columns.len(), 2);
        assert_eq!(snapshot.dataset_metadata.technical.row_count, 3);
        assert_eq!(snapshot.dataset_metadata.technical.column_count, 2);

        Ok(())
    }
}
