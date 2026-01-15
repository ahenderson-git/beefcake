//! Dataset lifecycle management system
//!
//! This module implements an immutable, versioned dataset lifecycle with six stages:
//! - **Raw**: Immutable original ingestion (never modified)
//! - **Profiled**: Statistical analysis with quality assessment
//! - **Cleaned**: Deterministic text/type transformations (trim, cast, rename)
//! - **Advanced**: ML preprocessing (imputation, normalisation, outliers, features)
//! - **Validated**: Quality gates passed, ready for production
//! - **Published**: Finalized as view (lazy) or snapshot (materialized)
//!
//! ## Key Principles
//!
//! - **Immutability**: Raw data is never modified; all transformations create new versions
//! - **Serialization**: All transforms are JSON-serializable with parameters
//! - **Diff Tracking**: Each version includes diff summary (schema + statistical changes)
//! - **Active Pointer**: Single "active version" for consumption by downstream systems
//! - **Storage Modes**: Views (lazy `LazyFrame`) vs Snapshots (materialized Parquet)
//!
//! ## Example Usage
//!
//! ```no_run
//! use beefcake::analyser::lifecycle::{DatasetRegistry, TransformPipeline, LifecycleStage};
//! use std::path::PathBuf;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Create registry
//! let registry = DatasetRegistry::new(PathBuf::from("data/lifecycle"))?;
//!
//! // Ingest raw data
//! let dataset_id = registry.create_dataset(
//!     "Sales Data".to_string(),
//!     PathBuf::from("sales.csv")
//! )?;
//!
//! // Apply transformations to create new version
//! let pipeline = TransformPipeline { transforms: vec![] };
//! let new_version_id = registry.apply_transforms(&dataset_id, pipeline, LifecycleStage::Cleaned)?;
//!
//! // Set as active version
//! registry.set_active_version(&dataset_id, &new_version_id)?;
//!
//! // Get active data for consumption
//! let data = registry.get_active_data(&dataset_id)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Version Diff Engine
//!
//! Compare any two versions to see:
//! - Schema changes (columns added/removed/renamed)
//! - Row count changes
//! - Statistical changes per column (mean, median, min, max, etc.)
//!
//! ```no_run
//! # use beefcake::analyser::lifecycle::DatasetRegistry;
//! # use std::path::PathBuf;
//! # use uuid::Uuid;
//! # fn example(registry: DatasetRegistry, dataset_id: Uuid, v1: Uuid, v2: Uuid) -> anyhow::Result<()> {
//! let diff = registry.compute_diff(&dataset_id, &v1, &v2)?;
//! println!("Columns added: {:?}", diff.schema_changes.columns_added);
//! println!("Row change: {} → {}", diff.row_changes.rows_v1, diff.row_changes.rows_v2);
//! # Ok(())
//! # }
//! ```

pub mod diff;
pub mod query;
pub mod stages;
pub mod storage;
pub mod transforms;
pub mod version;

pub use diff::{DiffSummary, compute_version_diff};
pub use query::VersionQuery;
pub use stages::{LifecycleStage, PublishMode, StageExecutor};
pub use storage::{DataLocation, VersionStore};
pub use transforms::{Transform, TransformPipeline};
pub use version::{Dataset, DatasetVersion, VersionMetadata, VersionTree};

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Central registry for all datasets and their versions
#[derive(Clone)]
pub struct DatasetRegistry {
    datasets: Arc<RwLock<HashMap<Uuid, Dataset>>>,
    store: Arc<VersionStore>,
}

impl DatasetRegistry {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let store = Arc::new(VersionStore::new(base_path)?);
        Ok(Self {
            datasets: Arc::new(RwLock::new(HashMap::new())),
            store,
        })
    }

    /// Create a new dataset from a raw data file
    pub fn create_dataset(&self, name: String, raw_data_path: PathBuf) -> Result<Uuid> {
        let dataset = Dataset::new(name, raw_data_path, Arc::clone(&self.store))?;
        let id = dataset.id;

        let mut datasets = self
            .datasets
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;
        datasets.insert(id, dataset);

        Ok(id)
    }

    /// Get a dataset by ID
    pub fn get_dataset(&self, id: &Uuid) -> Result<Dataset> {
        let datasets = self
            .datasets
            .read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;
        datasets
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Dataset not found: {id}"))
    }

    /// Apply transforms to create a new version
    pub fn apply_transforms(
        &self,
        dataset_id: &Uuid,
        pipeline: TransformPipeline,
        stage: LifecycleStage,
    ) -> Result<Uuid> {
        let mut datasets = self
            .datasets
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;

        let dataset = datasets
            .get_mut(dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found: {dataset_id}"))?;

        dataset.apply_pipeline(pipeline, stage)
    }

    /// Set the active version for a dataset
    pub fn set_active_version(&self, dataset_id: &Uuid, version_id: &Uuid) -> Result<()> {
        let mut datasets = self
            .datasets
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;

        let dataset = datasets
            .get_mut(dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found: {dataset_id}"))?;

        dataset.set_active_version(version_id)
    }

    /// Get the active version `LazyFrame` for consumption
    pub fn get_active_data(&self, dataset_id: &Uuid) -> Result<polars::prelude::LazyFrame> {
        let dataset = self.get_dataset(dataset_id)?;
        dataset.get_active_data()
    }

    /// Publish a version as a snapshot or view
    pub fn publish_version(
        &self,
        dataset_id: &Uuid,
        version_id: &Uuid,
        mode: stages::PublishMode,
    ) -> Result<Uuid> {
        let mut datasets = self
            .datasets
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;

        let dataset = datasets
            .get_mut(dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found: {dataset_id}"))?;

        dataset.publish_version(version_id, mode)
    }

    /// Compute diff between two versions
    pub fn compute_diff(
        &self,
        dataset_id: &Uuid,
        version1_id: &Uuid,
        version2_id: &Uuid,
    ) -> Result<DiffSummary> {
        let dataset = self.get_dataset(dataset_id)?;
        let v1 = dataset.get_version(version1_id)?;
        let v2 = dataset.get_version(version2_id)?;

        compute_version_diff(&v1, &v2, &self.store)
    }

    /// List all versions for a dataset
    pub fn list_versions(&self, dataset_id: &Uuid) -> Result<Vec<DatasetVersion>> {
        let dataset = self.get_dataset(dataset_id)?;
        Ok(dataset.list_versions())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_creation() -> Result<()> {
        let temp = TempDir::new()?;
        let registry = DatasetRegistry::new(temp.path().to_path_buf())?;
        assert!(
            registry
                .datasets
                .read()
                .expect("Lock should not be poisoned")
                .is_empty()
        );
        Ok(())
    }
}
