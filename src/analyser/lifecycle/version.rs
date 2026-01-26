//! Version management for dataset lifecycle

use super::stages::{LifecycleStage, PublishMode};
use super::storage::{DataLocation, VersionStore};
use super::transforms::TransformPipeline;
use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Metadata associated with a dataset version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub row_count: Option<usize>,
    pub column_count: Option<usize>,
    pub file_size_bytes: Option<u64>,
    pub created_by: String,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for VersionMetadata {
    fn default() -> Self {
        Self {
            description: String::new(),
            tags: Vec::new(),
            row_count: None,
            column_count: None,
            file_size_bytes: None,
            created_by: "system".to_owned(),
            custom_fields: HashMap::new(),
        }
    }
}

/// A specific version of a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetVersion {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub stage: LifecycleStage,
    pub pipeline: TransformPipeline,
    pub data_location: DataLocation,
    pub metadata: VersionMetadata,
    pub created_at: DateTime<Utc>,
}

impl DatasetVersion {
    pub fn new_raw(dataset_id: Uuid, data_location: DataLocation) -> Self {
        Self {
            id: Uuid::new_v4(),
            dataset_id,
            parent_id: None,
            stage: LifecycleStage::Raw,
            pipeline: TransformPipeline::empty(),
            data_location,
            metadata: VersionMetadata {
                description: "Raw ingestion".to_owned(),
                ..Default::default()
            },
            created_at: Utc::now(),
        }
    }

    pub fn new_derived(
        id: Uuid,
        dataset_id: Uuid,
        parent_id: Uuid,
        stage: LifecycleStage,
        pipeline: TransformPipeline,
        data_location: DataLocation,
    ) -> Self {
        Self {
            id,
            dataset_id,
            parent_id: Some(parent_id),
            stage,
            pipeline,
            data_location,
            metadata: VersionMetadata {
                description: format!("Stage: {}", stage.as_str()),
                ..Default::default()
            },
            created_at: Utc::now(),
        }
    }

    /// Load the data for this version
    /// Applies any lazy transforms stored in the pipeline (e.g., column selection)
    pub fn load_data(&self, store: &VersionStore) -> Result<LazyFrame> {
        let lf = store.load_version_data(&self.data_location)?;

        // Apply pipeline transforms that were stored as metadata-only
        // This ensures operations like column selection are applied during data loading
        if !self.pipeline.is_empty() {
            self.pipeline.apply(lf)
        } else {
            Ok(lf)
        }
    }

    /// Get a serialized representation of this version
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize version")
    }

    /// Load a version from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialize version")
    }
}

/// Tree structure tracking version lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTree {
    versions: HashMap<Uuid, DatasetVersion>,
    root_id: Uuid,
}

impl VersionTree {
    pub fn new(root_version: DatasetVersion) -> Self {
        let root_id = root_version.id;
        let mut versions = HashMap::new();
        versions.insert(root_id, root_version);

        Self { versions, root_id }
    }

    pub fn add_version(&mut self, version: DatasetVersion) -> Result<()> {
        if let Some(parent_id) = version.parent_id
            && !self.versions.contains_key(&parent_id)
        {
            return Err(anyhow::anyhow!(
                "Parent version {parent_id} not found in tree"
            ));
        }

        self.versions.insert(version.id, version);
        Ok(())
    }

    pub fn get_version(&self, id: &Uuid) -> Option<&DatasetVersion> {
        self.versions.get(id)
    }

    pub fn root(&self) -> &DatasetVersion {
        self.versions
            .get(&self.root_id)
            .expect("Root version must exist")
    }

    pub fn list_all(&self) -> Vec<&DatasetVersion> {
        self.versions.values().collect()
    }

    pub fn get_lineage(&self, version_id: &Uuid) -> Vec<&DatasetVersion> {
        let mut lineage = Vec::new();
        let mut current_id = *version_id;

        while let Some(version) = self.versions.get(&current_id) {
            lineage.push(version);
            if let Some(parent_id) = version.parent_id {
                current_id = parent_id;
            } else {
                break;
            }
        }

        lineage.reverse();
        lineage
    }

    pub fn get_children(&self, version_id: &Uuid) -> Vec<&DatasetVersion> {
        self.versions
            .values()
            .filter(|v| v.parent_id == Some(*version_id))
            .collect()
    }
}

/// Top-level dataset containing all versions
#[derive(Debug, Clone, Serialize)]
pub struct Dataset {
    pub id: Uuid,
    pub name: String,
    pub raw_version_id: Uuid,
    pub active_version_id: Uuid,
    pub versions: VersionTree,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub store: Arc<VersionStore>,
}

impl Dataset {
    pub fn new(name: String, raw_data_path: PathBuf, store: Arc<VersionStore>) -> Result<Self> {
        let id = Uuid::new_v4();

        // Copy raw data to version storage
        let data_location = store.store_raw_data(&id, &raw_data_path)?;

        // Create raw version
        let raw_version = DatasetVersion::new_raw(id, data_location);
        let raw_version_id = raw_version.id;

        // Save version metadata
        store.save_version_metadata(&raw_version)?;

        Ok(Self {
            id,
            name,
            raw_version_id,
            active_version_id: raw_version_id,
            versions: VersionTree::new(raw_version),
            created_at: Utc::now(),
            store,
        })
    }

    pub fn apply_pipeline(
        &mut self,
        pipeline: TransformPipeline,
        stage: LifecycleStage,
    ) -> Result<Uuid> {
        // Get active version
        let active_version = self
            .versions
            .get_version(&self.active_version_id)
            .ok_or_else(|| anyhow::anyhow!("Active version not found"))?;

        // Optimisation: If pipeline is empty or no-op, reuse parent's data location
        // This is common for:
        // - Metadata-only stages like Profiled (empty pipeline)
        // - Cleaned -> Advanced with restricted:true (no data changes)
        let new_version_id = Uuid::new_v4();
        let should_reuse_data = pipeline.is_empty() || is_pipeline_no_op(&pipeline, active_version);

        let data_location = if should_reuse_data {
            crate::config::log_event(
                "Lifecycle",
                &format!(
                    "Stage '{}' pipeline does not modify data, reusing parent data location",
                    stage.as_str()
                ),
            );
            active_version.data_location.clone()
        } else {
            // Load data and apply transforms
            let lf = active_version.load_data(&self.store)?;
            let transformed_lf = pipeline.apply(lf)?;

            // Store transformed data as new Parquet file
            self.store
                .store_version_data(&self.id, &new_version_id, &transformed_lf)?
        };

        // Create new version
        let new_version = DatasetVersion::new_derived(
            new_version_id,
            self.id,
            self.active_version_id,
            stage,
            pipeline,
            data_location,
        );
        let new_version_id = new_version.id;

        // Save metadata
        self.store.save_version_metadata(&new_version)?;

        // Add to tree
        self.versions.add_version(new_version)?;

        // Update active version
        self.active_version_id = new_version_id;

        Ok(new_version_id)
    }

    pub fn set_active_version(&mut self, version_id: &Uuid) -> Result<()> {
        if self.versions.get_version(version_id).is_none() {
            return Err(anyhow::anyhow!("Version {version_id} not found"));
        }
        self.active_version_id = *version_id;
        Ok(())
    }

    pub fn get_active_data(&self) -> Result<LazyFrame> {
        let active_version = self
            .versions
            .get_version(&self.active_version_id)
            .ok_or_else(|| anyhow::anyhow!("Active version not found"))?;
        active_version.load_data(&self.store)
    }

    pub fn get_version(&self, version_id: &Uuid) -> Result<DatasetVersion> {
        self.versions
            .get_version(version_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Version {version_id} not found"))
    }

    pub fn list_versions(&self) -> Vec<DatasetVersion> {
        self.versions.list_all().into_iter().cloned().collect()
    }

    pub fn publish_version(&mut self, version_id: &Uuid, mode: PublishMode) -> Result<Uuid> {
        let version = self.get_version(version_id)?;
        let published_id = Uuid::new_v4();

        let data_location = match mode {
            PublishMode::View => {
                // View mode: just create a reference, no data copy
                version.data_location.clone()
            }
            PublishMode::Snapshot => {
                // Snapshot mode: materialize and store
                let lf = version.load_data(&self.store)?;
                self.store
                    .store_version_data(&self.id, &published_id, &lf)?
            }
        };

        let published_version = DatasetVersion::new_derived(
            published_id,
            self.id,
            *version_id,
            LifecycleStage::Published,
            version.pipeline.clone(),
            data_location,
        );

        let published_id = published_version.id;
        self.store.save_version_metadata(&published_version)?;
        self.versions.add_version(published_version)?;

        Ok(published_id)
    }
}

/// Check if a pipeline would result in a no-op (no data changes requiring file rewrite)
/// This detects metadata-only operations that don't require materializing and rewriting data
fn is_pipeline_no_op(pipeline: &TransformPipeline, active_version: &DatasetVersion) -> bool {
    // Empty pipeline is always a no-op
    if pipeline.is_empty() {
        return true;
    }

    // Single transform optimizations
    if pipeline.len() == 1 {
        if let Some(transform_spec) = pipeline.iter().next() {
            // Case 1: Column selection is metadata-only for Parquet files
            // Parquet supports native column projection, so we don't need to rewrite the file
            // This optimization provides massive performance gains for large files
            if transform_spec.transform_type == "select_columns" {
                crate::config::log_event(
                    "Lifecycle",
                    "Column selection detected - using metadata-only optimization",
                );
                return true;
            }

            // Case 2: Clean transform with restricted=true (Cleaned -> Advanced transition)
            if transform_spec.transform_type == "clean" {
                let restricted = transform_spec
                    .parameters
                    .get("restricted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // If restricted=true and parent is Cleaned stage, this is a no-op
                // because Cleaned -> Advanced with restricted=true doesn't actually transform data
                return restricted && active_version.stage == LifecycleStage::Cleaned;
            }
        }
    }

    false
}
