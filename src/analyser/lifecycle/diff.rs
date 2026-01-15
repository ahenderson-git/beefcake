//! Version diff computation

use super::storage::VersionStore;
use super::version::DatasetVersion;
use anyhow::{Context as _, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Summary of differences between two dataset versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub version1_id: String,
    pub version2_id: String,
    pub schema_changes: SchemaChanges,
    pub row_changes: RowChanges,
    pub statistical_changes: Vec<StatisticalChange>,
    pub sample_changes: Vec<SampleChange>,
}

/// Changes to schema between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChanges {
    pub columns_added: Vec<String>,
    pub columns_removed: Vec<String>,
    pub columns_renamed: Vec<(String, String)>,
    pub type_changes: Vec<TypeChange>,
}

/// Row-level changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowChanges {
    pub rows_v1: usize,
    pub rows_v2: usize,
    pub rows_added: Option<usize>,
    pub rows_removed: Option<usize>,
    pub rows_modified: Option<usize>,
}

/// Statistical change in a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalChange {
    pub column: String,
    pub metric: String,
    pub value_v1: Option<f64>,
    pub value_v2: Option<f64>,
    pub change_percent: Option<f64>,
}

/// Sample of changed values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleChange {
    pub row_index: usize,
    pub column: String,
    pub old_value: String,
    pub new_value: String,
}

/// Type change in a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeChange {
    pub column: String,
    pub old_type: String,
    pub new_type: String,
}

/// Compute diff between two dataset versions
pub fn compute_version_diff(
    v1: &DatasetVersion,
    v2: &DatasetVersion,
    store: &VersionStore,
) -> Result<DiffSummary> {
    let mut lf1 = v1.load_data(store)?;
    let mut lf2 = v2.load_data(store)?;

    let schema1 = lf1.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
    let schema2 = lf2.collect_schema().map_err(|e| anyhow::anyhow!(e))?;

    // Compute schema changes
    let schema_changes = compute_schema_changes(&schema1, &schema2);

    // Compute row changes
    let row_changes = compute_row_changes(&lf1, &lf2)?;

    // Compute statistical changes for common numeric columns
    let statistical_changes = compute_statistical_changes(&lf1, &lf2, &schema1, &schema2);

    // Get sample changes (limited to 100 samples for performance)
    let sample_changes = compute_sample_changes(&lf1, &lf2, &schema1, &schema2);

    Ok(DiffSummary {
        version1_id: v1.id.to_string(),
        version2_id: v2.id.to_string(),
        schema_changes,
        row_changes,
        statistical_changes,
        sample_changes,
    })
}

fn compute_schema_changes(schema1: &Schema, schema2: &Schema) -> SchemaChanges {
    let cols1: HashSet<String> = schema1.iter_names().map(|s| s.to_string()).collect();
    let cols2: HashSet<String> = schema2.iter_names().map(|s| s.to_string()).collect();

    let columns_added: Vec<String> = cols2.difference(&cols1).cloned().collect();
    let columns_removed: Vec<String> = cols1.difference(&cols2).cloned().collect();

    let mut type_changes = Vec::new();
    for name in cols1.intersection(&cols2) {
        let type1 = schema1.get(name).map(|dt| format!("{dt:?}"));
        let type2 = schema2.get(name).map(|dt| format!("{dt:?}"));

        if type1 != type2
            && let (Some(t1), Some(t2)) = (type1, type2)
        {
            type_changes.push(TypeChange {
                column: name.clone(),
                old_type: t1,
                new_type: t2,
            });
        }
    }

    SchemaChanges {
        columns_added,
        columns_removed,
        columns_renamed: Vec::new(), // Rename detection would require heuristics
        type_changes,
    }
}

fn compute_row_changes(lf1: &LazyFrame, lf2: &LazyFrame) -> Result<RowChanges> {
    let count1_df = lf1
        .clone()
        .select([len()])
        .collect()
        .context("Failed to count v1 rows")?;
    let count2_df = lf2
        .clone()
        .select([len()])
        .collect()
        .context("Failed to count v2 rows")?;

    let rows_v1 = count1_df
        .column("len")
        .context("Failed to get v1 row count")?
        .as_materialized_series()
        .u32()
        .context("Row count not u32")?
        .get(0)
        .unwrap_or(0) as usize;

    let rows_v2 = count2_df
        .column("len")
        .context("Failed to get v2 row count")?
        .as_materialized_series()
        .u32()
        .context("Row count not u32")?
        .get(0)
        .unwrap_or(0) as usize;

    let (rows_added, rows_removed) = if rows_v2 > rows_v1 {
        (Some(rows_v2 - rows_v1), None)
    } else if rows_v1 > rows_v2 {
        (None, Some(rows_v1 - rows_v2))
    } else {
        (None, None)
    };

    Ok(RowChanges {
        rows_v1,
        rows_v2,
        rows_added,
        rows_removed,
        rows_modified: None, // Would require row-by-row comparison
    })
}

fn compute_statistical_changes(
    lf1: &LazyFrame,
    lf2: &LazyFrame,
    schema1: &Schema,
    schema2: &Schema,
) -> Vec<StatisticalChange> {
    let mut changes = Vec::new();

    // Find common numeric columns
    let cols1: HashSet<String> = schema1.iter_names().map(|s| s.to_string()).collect();
    let cols2: HashSet<String> = schema2.iter_names().map(|s| s.to_string()).collect();
    let common_cols: Vec<String> = cols1.intersection(&cols2).cloned().collect();

    for col_name in common_cols.iter().take(20) {
        // Limit to 20 columns for performance
        let dtype1 = schema1.get(col_name);
        let dtype2 = schema2.get(col_name);

        if let (Some(dt1), Some(dt2)) = (dtype1, dtype2)
            && dt1.is_numeric()
            && dt2.is_numeric()
        {
            // Compute mean, median, etc.
            if let Ok(stats_changes) = compute_column_stats_changes(lf1, lf2, col_name) {
                changes.extend(stats_changes);
            }
        }
    }

    changes
}

fn compute_column_stats_changes(
    lf1: &LazyFrame,
    lf2: &LazyFrame,
    col_name: &str,
) -> Result<Vec<StatisticalChange>> {
    let mut changes = Vec::new();

    // Compute mean
    let mean1_df = lf1
        .clone()
        .select([col(col_name).mean().alias("mean")])
        .collect()?;
    let mean2_df = lf2
        .clone()
        .select([col(col_name).mean().alias("mean")])
        .collect()?;

    let mean1 = mean1_df
        .column("mean")?
        .as_materialized_series()
        .f64()?
        .get(0);
    let mean2 = mean2_df
        .column("mean")?
        .as_materialized_series()
        .f64()?
        .get(0);

    if let (Some(m1), Some(m2)) = (mean1, mean2) {
        let change_pct = if m1 != 0.0 {
            Some(((m2 - m1) / m1) * 100.0)
        } else {
            None
        };

        changes.push(StatisticalChange {
            column: col_name.to_owned(),
            metric: "mean".to_owned(),
            value_v1: Some(m1),
            value_v2: Some(m2),
            change_percent: change_pct,
        });
    }

    // Could add median, std, etc. here

    Ok(changes)
}

fn compute_sample_changes(
    _lf1: &LazyFrame,
    _lf2: &LazyFrame,
    _schema1: &Schema,
    _schema2: &Schema,
) -> Vec<SampleChange> {
    // This is a simplified version - a real implementation would need
    // to match rows by key or index and compare values
    // For now, we just return an empty vec as a placeholder
    Vec::new()
}

impl DiffSummary {
    /// Check if there are any significant changes
    pub fn has_changes(&self) -> bool {
        !self.schema_changes.columns_added.is_empty()
            || !self.schema_changes.columns_removed.is_empty()
            || !self.schema_changes.type_changes.is_empty()
            || self.row_changes.rows_added.is_some()
            || self.row_changes.rows_removed.is_some()
            || !self.statistical_changes.is_empty()
    }

    /// Get a human-readable summary
    pub fn summary_text(&self) -> String {
        let mut parts = Vec::new();

        if !self.schema_changes.columns_added.is_empty() {
            parts.push(format!(
                "Added {} columns",
                self.schema_changes.columns_added.len()
            ));
        }

        if !self.schema_changes.columns_removed.is_empty() {
            parts.push(format!(
                "Removed {} columns",
                self.schema_changes.columns_removed.len()
            ));
        }

        if let Some(added) = self.row_changes.rows_added {
            parts.push(format!("Added {added} rows"));
        }

        if let Some(removed) = self.row_changes.rows_removed {
            parts.push(format!("Removed {removed} rows"));
        }

        if !self.statistical_changes.is_empty() {
            parts.push(format!(
                "{} statistical changes",
                self.statistical_changes.len()
            ));
        }

        if parts.is_empty() {
            "No significant changes".to_owned()
        } else {
            parts.join(", ")
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize diff")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_changes() {
        let mut schema1 = Schema::default();
        schema1.insert("col1".into(), DataType::Int64);
        schema1.insert("col2".into(), DataType::String);

        let mut schema2 = Schema::default();
        schema2.insert("col1".into(), DataType::Float64);
        schema2.insert("col3".into(), DataType::String);

        let changes = compute_schema_changes(&schema1, &schema2);

        assert_eq!(changes.columns_added.len(), 1);
        assert!(changes.columns_added.contains(&"col3".to_owned()));

        assert_eq!(changes.columns_removed.len(), 1);
        assert!(changes.columns_removed.contains(&"col2".to_owned()));

        assert_eq!(changes.type_changes.len(), 1);
        assert_eq!(changes.type_changes[0].column, "col1");
    }
}
