//! Transform trait and pipeline for serializable data transformations

use anyhow::{Context as _, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::analyser::logic::{ColumnCleanConfig, clean_df_lazy};

/// Trait for all data transformations
/// Each transform must be:
/// - Serializable (to/from JSON)
/// - Parameterized (all params in `HashMap`)
/// - Able to generate diff summary
/// - Deterministic (same input + params = same output)
pub trait Transform: Send + Sync {
    /// Apply this transform to a `LazyFrame`
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame>;

    /// Get the name of this transform
    fn name(&self) -> &str;

    /// Serialize parameters to JSON-compatible map
    fn parameters(&self) -> HashMap<String, serde_json::Value>;

    /// Create a summary of what this transform does
    fn description(&self) -> String;

    /// Serialize the entire transform to JSON
    fn to_json(&self) -> Result<String>;
}

/// A pipeline of transforms applied sequentially
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPipeline {
    transforms: Vec<TransformSpec>,
}

/// Serializable specification of a transform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformSpec {
    pub transform_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl TransformPipeline {
    pub fn empty() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    pub fn new(transforms: Vec<TransformSpec>) -> Self {
        Self { transforms }
    }

    pub fn add(&mut self, spec: TransformSpec) {
        self.transforms.push(spec);
    }

    /// Apply all transforms in sequence
    pub fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        let mut result = lf;

        for (idx, spec) in self.transforms.iter().enumerate() {
            let transform = instantiate_transform(spec).with_context(|| {
                format!(
                    "Failed to instantiate transform {}: {}",
                    idx, spec.transform_type
                )
            })?;

            result = transform.apply(result).with_context(|| {
                format!("Failed to apply transform {}: {}", idx, spec.transform_type)
            })?;
        }

        Ok(result)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize pipeline")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialize pipeline")
    }

    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TransformSpec> {
        self.transforms.iter()
    }
}

/// Instantiate a concrete transform from a spec
fn instantiate_transform(spec: &TransformSpec) -> Result<Box<dyn Transform>> {
    match spec.transform_type.as_str() {
        "clean" => Ok(Box::new(CleanTransform::from_parameters(&spec.parameters)?)),
        "select_columns" => Ok(Box::new(SelectColumnsTransform::from_parameters(
            &spec.parameters,
        )?)),
        "filter_rows" => Ok(Box::new(FilterRowsTransform::from_parameters(
            &spec.parameters,
        )?)),
        "rename_columns" => Ok(Box::new(RenameColumnsTransform::from_parameters(
            &spec.parameters,
        )?)),
        "drop_nulls" => Ok(Box::new(DropNullsTransform::from_parameters(
            &spec.parameters,
        )?)),
        "sort" => Ok(Box::new(SortTransform::from_parameters(&spec.parameters)?)),
        _ => Err(anyhow::anyhow!(
            "Unknown transform type: {}",
            spec.transform_type
        )),
    }
}

// ============================================================================
// Concrete Transform Implementations
// ============================================================================

/// Transform that wraps the existing `clean_df_lazy` logic
#[derive(Debug, Clone)]
pub struct CleanTransform {
    configs: HashMap<String, ColumnCleanConfig>,
    restricted: bool,
}

impl CleanTransform {
    pub fn new(configs: HashMap<String, ColumnCleanConfig>, restricted: bool) -> Self {
        Self {
            configs,
            restricted,
        }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let configs_json = params
            .get("configs")
            .ok_or_else(|| anyhow::anyhow!("Missing 'configs' parameter"))?;

        let configs: HashMap<String, ColumnCleanConfig> =
            serde_json::from_value(configs_json.clone())
                .context("Failed to deserialize configs")?;

        let restricted = params
            .get("restricted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Self {
            configs,
            restricted,
        })
    }
}

impl Transform for CleanTransform {
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        clean_df_lazy(lf, &self.configs, self.restricted)
    }

    fn name(&self) -> &'static str {
        "clean"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "configs".to_owned(),
            serde_json::to_value(&self.configs).unwrap_or(serde_json::Value::Null),
        );
        params.insert(
            "restricted".to_owned(),
            serde_json::Value::Bool(self.restricted),
        );
        params
    }

    fn description(&self) -> String {
        format!(
            "Clean {} columns (restricted: {})",
            self.configs.len(),
            self.restricted
        )
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

/// Select specific columns
#[derive(Debug, Clone)]
pub struct SelectColumnsTransform {
    columns: Vec<String>,
}

impl SelectColumnsTransform {
    pub fn new(columns: Vec<String>) -> Self {
        Self { columns }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let columns_json = params
            .get("columns")
            .ok_or_else(|| anyhow::anyhow!("Missing 'columns' parameter"))?;

        let columns: Vec<String> = serde_json::from_value(columns_json.clone())
            .context("Failed to deserialize columns")?;

        Ok(Self { columns })
    }
}

impl Transform for SelectColumnsTransform {
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        let exprs: Vec<Expr> = self.columns.iter().map(col).collect();
        Ok(lf.select(exprs))
    }

    fn name(&self) -> &'static str {
        "select_columns"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "columns".to_owned(),
            serde_json::to_value(&self.columns).unwrap_or(serde_json::Value::Null),
        );
        params
    }

    fn description(&self) -> String {
        format!("Select {} columns", self.columns.len())
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

/// Filter rows based on a condition expression
#[derive(Debug, Clone)]
pub struct FilterRowsTransform {
    condition: String,
}

impl FilterRowsTransform {
    pub fn new(condition: String) -> Self {
        Self { condition }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let condition = params
            .get("condition")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'condition' parameter"))?
            .to_owned();

        Ok(Self { condition })
    }
}

impl Transform for FilterRowsTransform {
    fn apply(&self, _lf: LazyFrame) -> Result<LazyFrame> {
        // For now, this is a placeholder - we'd need to parse Sql-like conditions
        // or use a custom expression language
        Err(anyhow::anyhow!(
            "FilterRowsTransform not yet implemented. Condition: {}",
            self.condition
        ))
    }

    fn name(&self) -> &'static str {
        "filter_rows"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "condition".to_owned(),
            serde_json::Value::String(self.condition.clone()),
        );
        params
    }

    fn description(&self) -> String {
        format!("Filter rows where: {}", self.condition)
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

/// Rename columns
#[derive(Debug, Clone)]
pub struct RenameColumnsTransform {
    mapping: HashMap<String, String>,
}

impl RenameColumnsTransform {
    pub fn new(mapping: HashMap<String, String>) -> Self {
        Self { mapping }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let mapping_json = params
            .get("mapping")
            .ok_or_else(|| anyhow::anyhow!("Missing 'mapping' parameter"))?;

        let mapping: HashMap<String, String> = serde_json::from_value(mapping_json.clone())
            .context("Failed to deserialize mapping")?;

        Ok(Self { mapping })
    }
}

impl Transform for RenameColumnsTransform {
    fn apply(&self, mut lf: LazyFrame) -> Result<LazyFrame> {
        let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
        let mut exprs = Vec::new();

        for (name, _) in schema.iter() {
            let name_str = name.as_str();
            if let Some(new_name) = self.mapping.get(name_str) {
                exprs.push(col(name_str).alias(new_name));
            } else {
                exprs.push(col(name_str));
            }
        }

        Ok(lf.select(exprs))
    }

    fn name(&self) -> &'static str {
        "rename_columns"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "mapping".to_owned(),
            serde_json::to_value(&self.mapping).unwrap_or(serde_json::Value::Null),
        );
        params
    }

    fn description(&self) -> String {
        format!("Rename {} columns", self.mapping.len())
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

/// Drop rows with null values in specified columns
#[derive(Debug, Clone)]
pub struct DropNullsTransform {
    columns: Option<Vec<String>>,
}

impl DropNullsTransform {
    pub fn new(columns: Option<Vec<String>>) -> Self {
        Self { columns }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let columns = params
            .get("columns")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        Ok(Self { columns })
    }
}

impl Transform for DropNullsTransform {
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        match &self.columns {
            Some(cols) => {
                let mut result = lf;
                for col_name in cols {
                    result = result.filter(col(col_name).is_not_null());
                }
                Ok(result)
            }
            None => {
                // Drop rows with any null
                Ok(lf.drop_nulls(None))
            }
        }
    }

    fn name(&self) -> &'static str {
        "drop_nulls"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(cols) = &self.columns {
            params.insert(
                "columns".to_owned(),
                serde_json::to_value(cols).unwrap_or(serde_json::Value::Null),
            );
        }
        params
    }

    fn description(&self) -> String {
        match &self.columns {
            Some(cols) => format!("Drop nulls in {} columns", cols.len()),
            None => "Drop rows with any null".to_owned(),
        }
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

/// Sort by columns
#[derive(Debug, Clone)]
pub struct SortTransform {
    by_columns: Vec<String>,
    descending: Vec<bool>,
}

impl SortTransform {
    pub fn new(by_columns: Vec<String>, descending: Vec<bool>) -> Self {
        Self {
            by_columns,
            descending,
        }
    }

    pub fn from_parameters(params: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let by_columns: Vec<String> = params
            .get("by_columns")
            .ok_or_else(|| anyhow::anyhow!("Missing 'by_columns' parameter"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'by_columns' must be an array"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let descending: Vec<bool> = params
            .get("descending")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_bool()).collect())
            .unwrap_or_else(|| vec![false; by_columns.len()]);

        Ok(Self {
            by_columns,
            descending,
        })
    }
}

impl Transform for SortTransform {
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame> {
        Ok(lf.sort_by_exprs(
            self.by_columns.iter().map(col).collect::<Vec<_>>(),
            SortMultipleOptions::default().with_order_descending_multi(self.descending.clone()),
        ))
    }

    fn name(&self) -> &'static str {
        "sort"
    }

    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "by_columns".to_owned(),
            serde_json::to_value(&self.by_columns).unwrap_or(serde_json::Value::Null),
        );
        params.insert(
            "descending".to_owned(),
            serde_json::to_value(&self.descending).unwrap_or(serde_json::Value::Null),
        );
        params
    }

    fn description(&self) -> String {
        format!("Sort by {} columns", self.by_columns.len())
    }

    fn to_json(&self) -> Result<String> {
        let spec = TransformSpec {
            transform_type: self.name().to_owned(),
            parameters: self.parameters(),
        };
        serde_json::to_string_pretty(&spec).context("Failed to serialize transform")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pipeline() {
        let pipeline = TransformPipeline::empty();
        assert_eq!(pipeline.len(), 0);
        assert!(pipeline.is_empty());
    }

    #[test]
    fn test_pipeline_serialization() -> Result<()> {
        let mut pipeline = TransformPipeline::empty();

        let spec = TransformSpec {
            transform_type: "select_columns".to_owned(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("columns".to_owned(), serde_json::json!(["col1", "col2"]));
                params
            },
        };

        pipeline.add(spec);

        let json = pipeline.to_json()?;
        let deserialized = TransformPipeline::from_json(&json)?;

        assert_eq!(pipeline.len(), deserialized.len());
        Ok(())
    }
}
