//! Core data structures for data dictionary snapshots.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete data dictionary snapshot for a dataset export.
///
/// Combines technical metadata (immutable) with business metadata (user-editable).
/// Each snapshot is immutable once created; edits create new snapshot versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDictionary {
    /// Unique identifier for this snapshot
    pub snapshot_id: Uuid,

    /// Human-readable dataset name
    pub dataset_name: String,

    /// Timestamp when this snapshot was created
    pub export_timestamp: DateTime<Utc>,

    /// Dataset-level metadata (technical + business)
    pub dataset_metadata: DatasetMetadata,

    /// Per-column metadata array
    pub columns: Vec<ColumnMetadata>,

    /// Link to previous snapshot (for versioning)
    pub previous_snapshot_id: Option<Uuid>,
}

/// Dataset-level metadata container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// Automatically-captured technical metadata
    pub technical: TechnicalMetadata,

    /// User-editable business metadata
    pub business: DatasetBusinessMetadata,
}

/// Technical metadata automatically captured at export time (immutable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalMetadata {
    /// Input file paths with their content hashes
    pub input_sources: Vec<InputSource>,

    /// Pipeline ID that produced this dataset
    pub pipeline_id: Option<Uuid>,

    /// Full pipeline JSON specification
    pub pipeline_json: Option<String>,

    /// Hash of input dataset (before transformations)
    pub input_dataset_hash: Option<String>,

    /// Hash of output dataset (after transformations)
    pub output_dataset_hash: String,

    /// Total number of rows in output dataset
    pub row_count: usize,

    /// Total number of columns in output dataset
    pub column_count: usize,

    /// Export file format (csv, parquet, json, etc.)
    pub export_format: String,

    /// Data quality summary metrics
    pub quality_summary: QualitySummary,
}

/// Input source file with hash for lineage tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSource {
    pub path: String,
    pub hash: Option<String>,
}

/// Data quality summary metrics for the dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    /// Average null percentage across all columns
    pub avg_null_percentage: f64,

    /// Number of completely empty columns (all nulls)
    pub empty_column_count: usize,

    /// Number of constant columns (single distinct value)
    pub constant_column_count: usize,

    /// Estimated duplicate row count (if calculated)
    pub duplicate_row_count: Option<usize>,

    /// Overall data quality score (0-100)
    pub overall_score: f64,
}

/// User-editable business metadata at dataset level.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetBusinessMetadata {
    /// High-level description of dataset purpose and contents
    pub description: Option<String>,

    /// Intended use cases for this dataset
    pub intended_use: Option<String>,

    /// Data owner or steward (person/team responsible)
    pub owner_or_steward: Option<String>,

    /// Expected refresh cadence (e.g., "Daily", "Weekly", "On-demand")
    pub refresh_expectation: Option<String>,

    /// Sensitivity classification (e.g., "Public", "Internal", "Confidential")
    pub sensitivity_classification: Option<String>,

    /// Known limitations, caveats, or warnings about the data
    pub known_limitations: Option<String>,

    /// Additional free-form tags
    pub tags: Vec<String>,
}

/// Per-column metadata combining technical profiling and business context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    /// Stable UUID for this column (derived from name hash for cross-version tracking)
    pub column_id: Uuid,

    /// Current column name in the output dataset
    pub current_name: String,

    /// Original column name before any renames (if applicable)
    pub original_name: Option<String>,

    /// Technical metadata (immutable, auto-captured)
    pub technical: ColumnTechnicalMetadata,

    /// Business metadata (user-editable)
    pub business: ColumnBusinessMetadata,
}

/// Technical metadata for a single column (immutable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnTechnicalMetadata {
    /// Data type of the column (e.g., "Int64", "Utf8", "Float64")
    pub data_type: String,

    /// Whether the column allows null values
    pub nullable: bool,

    /// Percentage of null values (0.0 - 100.0)
    pub null_percentage: f64,

    /// Number of distinct values (or cardinality estimate)
    pub distinct_count: usize,

    /// Minimum value (for numeric/date columns)
    pub min_value: Option<String>,

    /// Maximum value (for numeric/date columns)
    pub max_value: Option<String>,

    /// Sample values (up to 5 examples)
    pub sample_values: Vec<String>,

    /// Warnings detected during analysis
    pub warnings: Vec<String>,

    /// Statistical summary (optional, JSON-encoded `ColumnStats`)
    pub stats_json: Option<String>,
}

/// User-editable business metadata for a single column.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnBusinessMetadata {
    /// Plain-English definition of what this column means
    pub business_definition: Option<String>,

    /// Business rules or constraints (e.g., "Must be positive", "ISO country code")
    pub business_rules: Option<String>,

    /// Sensitivity tag (e.g., "PII", "Financial", "Public")
    pub sensitivity_tag: Option<String>,

    /// Examples of approved/expected values
    pub approved_examples: Vec<String>,

    /// Free-form notes for this column
    pub notes: Option<String>,
}

impl DataDictionary {
    /// Calculate documentation completeness percentage (0-100).
    ///
    /// Based on how many business metadata fields are populated across
    /// dataset-level and column-level metadata.
    pub fn documentation_completeness(&self) -> f64 {
        let mut filled = 0;
        let mut total = 0;

        // Dataset-level business metadata (6 optional fields)
        let ds_meta = &self.dataset_metadata.business;
        total += 6;
        if ds_meta.description.is_some() {
            filled += 1;
        }
        if ds_meta.intended_use.is_some() {
            filled += 1;
        }
        if ds_meta.owner_or_steward.is_some() {
            filled += 1;
        }
        if ds_meta.refresh_expectation.is_some() {
            filled += 1;
        }
        if ds_meta.sensitivity_classification.is_some() {
            filled += 1;
        }
        if ds_meta.known_limitations.is_some() {
            filled += 1;
        }

        // Column-level business metadata (3 key fields per column)
        for col in &self.columns {
            total += 3;
            if col.business.business_definition.is_some() {
                filled += 1;
            }
            if col.business.business_rules.is_some() {
                filled += 1;
            }
            if col.business.sensitivity_tag.is_some() {
                filled += 1;
            }
        }

        if total == 0 {
            0.0
        } else {
            (filled as f64 / total as f64) * 100.0
        }
    }

    /// Get columns that have no business documentation.
    pub fn undocumented_columns(&self) -> Vec<&ColumnMetadata> {
        self.columns
            .iter()
            .filter(|col| {
                col.business.business_definition.is_none()
                    && col.business.business_rules.is_none()
                    && col.business.sensitivity_tag.is_none()
            })
            .collect()
    }

    /// Get columns flagged with warnings.
    pub fn columns_with_warnings(&self) -> Vec<&ColumnMetadata> {
        self.columns
            .iter()
            .filter(|col| !col.technical.warnings.is_empty())
            .collect()
    }
}

/// Generate stable UUID from column name for cross-version tracking.
pub fn column_name_to_uuid(name: &str) -> Uuid {
    use sha2::{Digest as _, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();

    // Use first 16 bytes of hash as UUID
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);

    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_uuid_stability() {
        let uuid1 = column_name_to_uuid("customer_id");
        let uuid2 = column_name_to_uuid("customer_id");
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_documentation_completeness_empty() {
        let dict = DataDictionary {
            snapshot_id: Uuid::new_v4(),
            dataset_name: "test".to_owned(),
            export_timestamp: Utc::now(),
            dataset_metadata: DatasetMetadata {
                technical: TechnicalMetadata {
                    input_sources: vec![],
                    pipeline_id: None,
                    pipeline_json: None,
                    input_dataset_hash: None,
                    output_dataset_hash: "abc123".to_owned(),
                    row_count: 100,
                    column_count: 2,
                    export_format: "csv".to_owned(),
                    quality_summary: QualitySummary {
                        avg_null_percentage: 0.0,
                        empty_column_count: 0,
                        constant_column_count: 0,
                        duplicate_row_count: None,
                        overall_score: 100.0,
                    },
                },
                business: DatasetBusinessMetadata::default(),
            },
            columns: vec![],
            previous_snapshot_id: None,
        };

        assert_eq!(dict.documentation_completeness(), 0.0);
    }
}
