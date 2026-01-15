//! Persistence layer for data dictionary snapshots.
//!
//! Handles saving/loading snapshots as JSON files with organized directory structure.

use super::metadata::DataDictionary;
use anyhow::{Context as _, Result};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Default directory name for storing dictionary snapshots.
pub const DICTIONARIES_DIR: &str = "dictionaries";

/// Save a data dictionary snapshot to disk as JSON.
///
/// # Arguments
///
/// * `snapshot` - The dictionary to save
/// * `base_path` - Base directory (will create `dictionaries/` subdirectory)
///
/// # Returns
///
/// Path to the saved JSON file.
pub fn save_snapshot(snapshot: &DataDictionary, base_path: &Path) -> Result<PathBuf> {
    let dict_dir = base_path.join(DICTIONARIES_DIR);

    // Create dictionaries directory if it doesn't exist
    fs::create_dir_all(&dict_dir).context("Failed to create dictionaries directory")?;

    // Save to {snapshot_id}.json
    let file_path = dict_dir.join(format!("{}.json", snapshot.snapshot_id));

    let json =
        serde_json::to_string_pretty(snapshot).context("Failed to serialize data dictionary")?;

    fs::write(&file_path, json).context("Failed to write dictionary snapshot")?;

    Ok(file_path)
}

/// Load a data dictionary snapshot from disk.
///
/// # Arguments
///
/// * `snapshot_id` - UUID of the snapshot to load
/// * `base_path` - Base directory containing `dictionaries/` subdirectory
pub fn load_snapshot(snapshot_id: &Uuid, base_path: &Path) -> Result<DataDictionary> {
    let file_path = base_path
        .join(DICTIONARIES_DIR)
        .join(format!("{snapshot_id}.json"));

    let json = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read snapshot file: {}", file_path.display()))?;

    serde_json::from_str(&json).context("Failed to deserialize data dictionary")
}

/// List all dictionary snapshots in a directory.
///
/// # Arguments
///
/// * `base_path` - Base directory containing `dictionaries/` subdirectory
/// * `dataset_hash_filter` - Optional: only return snapshots for a specific dataset hash
///
/// # Returns
///
/// Vector of `(snapshot_id, dataset_name, timestamp)` tuples, sorted by timestamp desc.
pub fn list_snapshots(
    base_path: &Path,
    dataset_hash_filter: Option<&str>,
) -> Result<Vec<SnapshotMetadata>> {
    let dict_dir = base_path.join(DICTIONARIES_DIR);

    if !dict_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();

    for entry in fs::read_dir(&dict_dir)
        .context("Failed to read dictionaries directory")?
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Try to load snapshot metadata (quick parse for listing)
        if let Ok(snapshot) = load_snapshot_from_path(&path) {
            // Filter by dataset hash if provided
            if let Some(filter_hash) = dataset_hash_filter
                && snapshot.dataset_metadata.technical.output_dataset_hash != filter_hash {
                    continue;
                }

            snapshots.push(SnapshotMetadata {
                snapshot_id: snapshot.snapshot_id,
                dataset_name: snapshot.dataset_name.clone(),
                timestamp: snapshot.export_timestamp,
                output_hash: snapshot
                    .dataset_metadata
                    .technical
                    .output_dataset_hash
                    .clone(),
                row_count: snapshot.dataset_metadata.technical.row_count,
                column_count: snapshot.dataset_metadata.technical.column_count,
                completeness_pct: snapshot.documentation_completeness(),
            });
        }
    }

    // Sort by timestamp descending (newest first)
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(snapshots)
}

/// Metadata summary for listing snapshots.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMetadata {
    pub snapshot_id: Uuid,
    pub dataset_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub output_hash: String,
    pub row_count: usize,
    pub column_count: usize,
    pub completeness_pct: f64,
}

/// Load a snapshot from a specific file path.
fn load_snapshot_from_path(path: &Path) -> Result<DataDictionary> {
    let json = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snapshot file: {}", path.display()))?;

    serde_json::from_str(&json).context("Failed to deserialize data dictionary")
}

/// Update business metadata for an existing snapshot (creates new version).
///
/// This loads the existing snapshot, updates the business metadata fields,
/// creates a new snapshot ID, links to the previous snapshot, and saves as new version.
pub fn update_business_metadata(
    snapshot_id: &Uuid,
    base_path: &Path,
    dataset_business: Option<super::metadata::DatasetBusinessMetadata>,
    column_business_updates: Option<
        std::collections::HashMap<String, super::metadata::ColumnBusinessMetadata>,
    >,
) -> Result<DataDictionary> {
    // Load existing snapshot
    let mut snapshot = load_snapshot(snapshot_id, base_path)?;

    // Update dataset business metadata if provided
    if let Some(new_business) = dataset_business {
        snapshot.dataset_metadata.business = new_business;
    }

    // Update column business metadata if provided
    if let Some(updates) = column_business_updates {
        for col in &mut snapshot.columns {
            if let Some(new_business) = updates.get(&col.current_name) {
                col.business = new_business.clone();
            }
        }
    }

    // Create new snapshot version
    let old_snapshot_id = snapshot.snapshot_id;
    snapshot.snapshot_id = Uuid::new_v4();
    snapshot.previous_snapshot_id = Some(old_snapshot_id);
    snapshot.export_timestamp = chrono::Utc::now();

    // Save new version
    save_snapshot(&snapshot, base_path)?;

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::metadata::*;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_snapshot() -> Result<()> {
        let temp_dir = tempdir()?;
        let snapshot_id = Uuid::new_v4();

        let snapshot = DataDictionary {
            snapshot_id,
            dataset_name: "test_data".to_owned(),
            export_timestamp: Utc::now(),
            dataset_metadata: DatasetMetadata {
                technical: TechnicalMetadata {
                    input_sources: vec![],
                    pipeline_id: None,
                    pipeline_json: None,
                    input_dataset_hash: None,
                    output_dataset_hash: "abc123".to_owned(),
                    row_count: 100,
                    column_count: 5,
                    export_format: "csv".to_owned(),
                    quality_summary: QualitySummary {
                        avg_null_percentage: 5.0,
                        empty_column_count: 0,
                        constant_column_count: 0,
                        duplicate_row_count: None,
                        overall_score: 95.0,
                    },
                },
                business: DatasetBusinessMetadata::default(),
            },
            columns: vec![],
            previous_snapshot_id: None,
        };

        // Save
        let saved_path = save_snapshot(&snapshot, temp_dir.path())?;
        assert!(saved_path.exists());

        // Load
        let loaded = load_snapshot(&snapshot_id, temp_dir.path())?;
        assert_eq!(loaded.snapshot_id, snapshot_id);
        assert_eq!(loaded.dataset_name, "test_data");

        Ok(())
    }

    #[test]
    fn test_list_snapshots() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create multiple snapshots
        for i in 0..3 {
            let snapshot = DataDictionary {
                snapshot_id: Uuid::new_v4(),
                dataset_name: format!("dataset_{i}"),
                export_timestamp: Utc::now(),
                dataset_metadata: DatasetMetadata {
                    technical: TechnicalMetadata {
                        input_sources: vec![],
                        pipeline_id: None,
                        pipeline_json: None,
                        input_dataset_hash: None,
                        output_dataset_hash: format!("hash_{i}"),
                        row_count: 100,
                        column_count: 5,
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
            save_snapshot(&snapshot, temp_dir.path())?;
        }

        // List all
        let snapshots = list_snapshots(temp_dir.path(), None)?;
        assert_eq!(snapshots.len(), 3);

        // Filter by hash
        let filtered = list_snapshots(temp_dir.path(), Some("hash_0"))?;
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].dataset_name, "dataset_0");

        Ok(())
    }
}
