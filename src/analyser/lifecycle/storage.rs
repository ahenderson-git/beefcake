//! Storage backend for dataset versions

use anyhow::{Context as _, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::version::DatasetVersion;

/// Location of version data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataLocation {
    /// Stored as parquet file
    ParquetFile(PathBuf),
    /// Reference to original file (for raw versions)
    OriginalFile(PathBuf),
}

impl DataLocation {
    pub fn path(&self) -> &std::path::Path {
        match self {
            Self::ParquetFile(p) | Self::OriginalFile(p) => p,
        }
    }
}

/// Storage backend for dataset versions
#[derive(Debug)]
pub struct VersionStore {
    base_path: PathBuf,
}

impl VersionStore {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base_path).context("Failed to create version store directory")?;
        Ok(Self { base_path })
    }

    /// Get the directory path for a dataset
    fn dataset_dir(&self, dataset_id: &Uuid) -> PathBuf {
        self.base_path.join(dataset_id.to_string())
    }

    /// Get the path for a version's data file
    fn version_data_path(&self, dataset_id: &Uuid, version_id: &Uuid) -> PathBuf {
        self.dataset_dir(dataset_id)
            .join(format!("{version_id}.parquet"))
    }

    /// Get the path for a version's metadata file
    fn version_metadata_path(&self, dataset_id: &Uuid, version_id: &Uuid) -> PathBuf {
        self.dataset_dir(dataset_id)
            .join(format!("{version_id}.meta.json"))
    }

    /// Store raw data from an external file
    pub fn store_raw_data(&self, dataset_id: &Uuid, source_path: &Path) -> Result<DataLocation> {
        let dataset_dir = self.dataset_dir(dataset_id);
        fs::create_dir_all(&dataset_dir).context("Failed to create dataset directory")?;

        // For raw data, reference the original file instead of copying
        // This avoids expensive re-loading and Parquet conversion during analysis
        // The file will be converted to Parquet only when transforms are applied
        crate::config::log_event(
            "Lifecycle",
            &format!(
                "Created raw version reference to: {}",
                source_path.display()
            ),
        );

        Ok(DataLocation::OriginalFile(source_path.to_path_buf()))
    }

    /// Store transformed version data
    pub fn store_version_data(
        &self,
        dataset_id: &Uuid,
        version_id: &Uuid,
        lf: &LazyFrame,
    ) -> Result<DataLocation> {
        let dataset_dir = self.dataset_dir(dataset_id);
        fs::create_dir_all(&dataset_dir).context("Failed to create dataset directory")?;

        let dest_path = self.version_data_path(dataset_id, version_id);

        // Write with streaming and compression
        let write_opts = crate::analyser::logic::get_parquet_write_options(lf)?;
        lf.clone()
            .with_streaming(true)
            .sink_parquet(&dest_path, write_opts, None)
            .context("Failed to sink version data to parquet")?;

        Ok(DataLocation::ParquetFile(dest_path))
    }

    /// Load data for a version
    pub fn load_version_data(&self, location: &DataLocation) -> Result<LazyFrame> {
        match location {
            DataLocation::ParquetFile(path) => LazyFrame::scan_parquet(path, Default::default())
                .context("Failed to scan parquet file"),
            DataLocation::OriginalFile(path) => {
                crate::analyser::logic::load_df_lazy(path).context("Failed to load original file")
            }
        }
    }

    /// Save version metadata
    pub fn save_version_metadata(&self, version: &DatasetVersion) -> Result<()> {
        let dataset_dir = self.dataset_dir(&version.dataset_id);
        fs::create_dir_all(&dataset_dir).context("Failed to create dataset directory")?;

        let meta_path = self.version_metadata_path(&version.dataset_id, &version.id);
        let json = version.to_json()?;

        fs::write(&meta_path, json).context("Failed to write version metadata")?;

        Ok(())
    }

    /// Load version metadata
    pub fn load_version_metadata(
        &self,
        dataset_id: &Uuid,
        version_id: &Uuid,
    ) -> Result<DatasetVersion> {
        let meta_path = self.version_metadata_path(dataset_id, version_id);
        let json = fs::read_to_string(&meta_path).context("Failed to read version metadata")?;

        DatasetVersion::from_json(&json)
    }

    /// Delete a version (both data and metadata)
    pub fn delete_version(&self, dataset_id: &Uuid, version_id: &Uuid) -> Result<()> {
        let data_path = self.version_data_path(dataset_id, version_id);
        let meta_path = self.version_metadata_path(dataset_id, version_id);

        if data_path.exists() {
            fs::remove_file(&data_path).context("Failed to delete version data")?;
        }

        if meta_path.exists() {
            fs::remove_file(&meta_path).context("Failed to delete version metadata")?;
        }

        Ok(())
    }

    /// Get storage statistics for a dataset
    pub fn get_dataset_stats(&self, dataset_id: &Uuid) -> Result<DatasetStorageStats> {
        let dataset_dir = self.dataset_dir(dataset_id);

        if !dataset_dir.exists() {
            return Ok(DatasetStorageStats::default());
        }

        let mut total_bytes = 0u64;
        let mut version_count = 0usize;

        for entry in fs::read_dir(&dataset_dir).context("Failed to read dataset directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                version_count += 1;
                if let Ok(metadata) = fs::metadata(&path) {
                    total_bytes += metadata.len();
                }
            }
        }

        Ok(DatasetStorageStats {
            total_bytes,
            version_count,
        })
    }

    /// Clean up unused versions (except specified `keep_versions`)
    pub fn cleanup_versions(&self, dataset_id: &Uuid, keep_versions: &[Uuid]) -> Result<usize> {
        let dataset_dir = self.dataset_dir(dataset_id);

        if !dataset_dir.exists() {
            return Ok(0);
        }

        let mut deleted_count = 0usize;

        for entry in fs::read_dir(&dataset_dir).context("Failed to read dataset directory")? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Extract version ID from filename (format: {uuid}.parquet or {uuid}.meta.json)
            if let Some(uuid_str) = file_name_str
                .strip_suffix(".parquet")
                .or_else(|| file_name_str.strip_suffix(".meta.json"))
                && let Ok(version_id) = Uuid::parse_str(uuid_str)
                && !keep_versions.contains(&version_id)
            {
                fs::remove_file(&path)?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count / 2) // Each version has 2 files (data + metadata)
    }
}

/// Storage statistics for a dataset
#[derive(Debug, Clone, Default)]
pub struct DatasetStorageStats {
    pub total_bytes: u64,
    pub version_count: usize,
}

impl DatasetStorageStats {
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn avg_version_bytes(&self) -> u64 {
        if self.version_count == 0 {
            0
        } else {
            self.total_bytes / self.version_count as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_creation() -> Result<()> {
        let temp = TempDir::new()?;
        let _store = VersionStore::new(temp.path().to_path_buf())?;
        assert!(temp.path().exists());
        Ok(())
    }

    #[test]
    fn test_dataset_paths() -> Result<()> {
        let temp = TempDir::new()?;
        let store = VersionStore::new(temp.path().to_path_buf())?;

        let dataset_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();

        let dataset_dir = store.dataset_dir(&dataset_id);
        assert_eq!(dataset_dir, temp.path().join(dataset_id.to_string()));

        let data_path = store.version_data_path(&dataset_id, &version_id);
        assert_eq!(data_path, dataset_dir.join(format!("{version_id}.parquet")));

        Ok(())
    }
}
