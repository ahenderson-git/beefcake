//! Query interface for version selection

use super::version::{Dataset, DatasetVersion};
use anyhow::Result;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Query builder for selecting dataset versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionQuery {
    /// Select by specific version ID
    pub version_id: Option<Uuid>,
    /// Select by stage
    pub stage: Option<super::stages::LifecycleStage>,
    /// Select latest version
    pub latest: bool,
    /// Select raw version
    pub raw: bool,
    /// Select active version
    pub active: bool,
}

impl Default for VersionQuery {
    fn default() -> Self {
        Self {
            version_id: None,
            stage: None,
            latest: false,
            raw: false,
            active: true, // Default to active version
        }
    }
}

impl VersionQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version_id(mut self, id: Uuid) -> Self {
        self.version_id = Some(id);
        self
    }

    pub fn stage(mut self, stage: super::stages::LifecycleStage) -> Self {
        self.stage = Some(stage);
        self
    }

    pub fn latest(mut self) -> Self {
        self.latest = true;
        self
    }

    pub fn raw(mut self) -> Self {
        self.raw = true;
        self
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    /// Execute the query against a dataset
    pub fn execute(&self, dataset: &Dataset) -> Result<DatasetVersion> {
        // Priority: version_id > raw > active > stage > latest

        if let Some(id) = self.version_id {
            return dataset.get_version(&id);
        }

        if self.raw {
            return dataset.get_version(&dataset.raw_version_id);
        }

        if self.active {
            return dataset.get_version(&dataset.active_version_id);
        }

        if let Some(stage) = self.stage {
            // Find most recent version in this stage
            let versions = dataset.list_versions();
            let matching: Vec<_> = versions.into_iter().filter(|v| v.stage == stage).collect();

            if matching.is_empty() {
                return Err(anyhow::anyhow!("No versions found in stage {stage:?}"));
            }

            // Return most recent
            let mut sorted = matching;
            sorted.sort_by_key(|v| v.created_at);
            return Ok(sorted.last().expect("matching is not empty").clone());
        }

        if self.latest {
            let versions = dataset.list_versions();
            if versions.is_empty() {
                return Err(anyhow::anyhow!("No versions found"));
            }

            let mut sorted = versions;
            sorted.sort_by_key(|v| v.created_at);
            return Ok(sorted.last().expect("versions is not empty").clone());
        }

        // Default to active
        dataset.get_version(&dataset.active_version_id)
    }

    /// Execute and load data
    pub fn execute_and_load(&self, dataset: &Dataset) -> Result<LazyFrame> {
        let version = self.execute(dataset)?;
        version.load_data(&dataset.store)
    }
}

/// Builder for Sql-like queries on datasets
pub struct DatasetQueryBuilder {
    dataset: Dataset,
    version_query: VersionQuery,
    filters: Vec<String>,
    select_columns: Option<Vec<String>>,
    limit: Option<u32>,
}

impl DatasetQueryBuilder {
    pub fn new(dataset: Dataset) -> Self {
        Self {
            dataset,
            version_query: VersionQuery::default(),
            filters: Vec::new(),
            select_columns: None,
            limit: None,
        }
    }

    pub fn version(mut self, query: VersionQuery) -> Self {
        self.version_query = query;
        self
    }

    pub fn filter(mut self, condition: String) -> Self {
        self.filters.push(condition);
        self
    }

    pub fn select(mut self, columns: Vec<String>) -> Self {
        self.select_columns = Some(columns);
        self
    }

    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn build(self) -> Result<LazyFrame> {
        let mut lf = self.version_query.execute_and_load(&self.dataset)?;

        // Apply column selection
        if let Some(cols) = self.select_columns {
            let exprs: Vec<Expr> = cols.iter().map(col).collect();
            lf = lf.select(exprs);
        }

        // Apply filters (placeholder - would need expression parsing)
        for _filter in self.filters {
            // Would need to parse filter string into Polars expression
        }

        // Apply limit
        if let Some(n) = self.limit {
            lf = lf.limit(n);
        }

        Ok(lf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_query_builder() {
        let query = VersionQuery::new()
            .stage(super::super::stages::LifecycleStage::Cleaned)
            .latest();

        assert!(query.stage.is_some());
        assert!(query.latest);
    }

    #[test]
    fn test_query_serialization() -> Result<()> {
        let query = VersionQuery::new().raw();
        let json = serde_json::to_string(&query)?;
        let deserialized: VersionQuery = serde_json::from_str(&json)?;
        assert_eq!(query.raw, deserialized.raw);
        Ok(())
    }
}
