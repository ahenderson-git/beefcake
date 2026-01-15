//! Pipeline specification data structures.
//!
//! Defines the JSON schema for pipeline specs, including input/output configuration,
//! transformation steps, and schema matching rules.

use crate::analyser::logic::types::ColumnCleanConfig;
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Current pipeline spec version
pub const SPEC_VERSION: &str = "0.1";

/// Root pipeline specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    /// Specification version for future migrations
    pub version: String,

    /// Human-readable pipeline name
    pub name: String,

    /// Input file configuration
    pub input: InputConfig,

    /// Schema validation rules
    pub schema: SchemaConfig,

    /// Ordered sequence of transformation steps
    pub steps: Vec<Step>,

    /// Output file configuration
    pub output: OutputConfig,
}

impl PipelineSpec {
    /// Create a new pipeline spec with default settings
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: SPEC_VERSION.to_owned(),
            name: name.into(),
            input: InputConfig::default(),
            schema: SchemaConfig::default(),
            steps: Vec::new(),
            output: OutputConfig::default(),
        }
    }

    /// Load a pipeline spec from a JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).context("Failed to read pipeline spec file")?;
        Self::from_json(&content)
    }

    /// Parse a pipeline spec from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse pipeline spec JSON")
    }

    /// Save pipeline spec to a JSON file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path.as_ref(), json).context("Failed to write pipeline spec file")
    }

    /// Serialize pipeline spec to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize pipeline spec")
    }

    /// Convert from existing cleaning configurations
    pub fn from_clean_configs(
        name: impl Into<String>,
        configs: &HashMap<String, ColumnCleanConfig>,
        input_format: &str,
        output_path: &str,
    ) -> Self {
        let mut spec = Self::new(name);

        // Configure input based on format
        spec.input.format = input_format.to_owned();

        // Convert configs to steps
        #[expect(clippy::iter_over_hash_type)]
        for (col_name, config) in configs {
            if !config.active {
                continue;
            }

            // Drop columns (mark inactive columns)
            // Note: We handle this by only processing active configs

            // Rename columns
            if !config.new_name.is_empty() && config.new_name != *col_name {
                let mut mapping = HashMap::new();
                mapping.insert(col_name.clone(), config.new_name.clone());
                spec.steps.push(Step::RenameColumns { mapping });
            }

            // Trim whitespace
            if config.trim_whitespace && config.advanced_cleaning {
                spec.steps.push(Step::TrimWhitespace {
                    columns: vec![col_name.clone()],
                });
            }

            // Cast types
            if let Some(target_dtype) = config.target_dtype {
                let mut columns = HashMap::new();
                columns.insert(col_name.clone(), target_dtype.as_str().to_owned());
                spec.steps.push(Step::CastTypes { columns });
            }

            // Parse dates (if temporal format specified)
            if !config.temporal_format.is_empty() {
                let mut columns = HashMap::new();
                columns.insert(col_name.clone(), config.temporal_format.clone());
                spec.steps.push(Step::ParseDates { columns });
            }

            // Imputation
            if config.ml_preprocessing
                && config.impute_mode != crate::analyser::logic::types::ImputeMode::None
            {
                let strategy = match config.impute_mode {
                    crate::analyser::logic::types::ImputeMode::Mean => ImputeStrategy::Mean,
                    crate::analyser::logic::types::ImputeMode::Median => ImputeStrategy::Median,
                    crate::analyser::logic::types::ImputeMode::Mode => ImputeStrategy::Mode,
                    crate::analyser::logic::types::ImputeMode::Zero => ImputeStrategy::Zero,
                    crate::analyser::logic::types::ImputeMode::None => continue,
                };
                spec.steps.push(Step::Impute {
                    strategy,
                    columns: vec![col_name.clone()],
                });
            }

            // One-hot encoding
            if config.ml_preprocessing && config.one_hot_encode {
                spec.steps.push(Step::OneHotEncode {
                    columns: vec![col_name.clone()],
                    drop_original: true,
                });
            }
        }

        // Configure output
        spec.output.path_template = output_path.to_owned();

        // Deduplicate steps by merging similar operations
        spec.optimize_steps();

        spec
    }

    /// Optimize steps by merging similar operations
    fn optimize_steps(&mut self) {
        // Merge all TrimWhitespace steps
        let mut trim_cols = Vec::new();
        let mut other_steps = Vec::new();

        for step in self.steps.drain(..) {
            match step {
                Step::TrimWhitespace { columns } => trim_cols.extend(columns),
                other => other_steps.push(other),
            }
        }

        if !trim_cols.is_empty() {
            self.steps.push(Step::TrimWhitespace { columns: trim_cols });
        }
        self.steps.extend(other_steps);
    }
}

/// Input file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// File format (csv, json, parquet)
    #[serde(default = "default_format")]
    pub format: String,

    /// Whether the file has a header row
    #[serde(default = "default_true")]
    pub has_header: bool,

    /// CSV delimiter character
    #[serde(default = "default_delimiter")]
    pub delimiter: String,

    /// File encoding
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            has_header: default_true(),
            delimiter: default_delimiter(),
            encoding: default_encoding(),
        }
    }
}

/// Schema validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// Schema matching mode
    #[serde(default)]
    pub match_mode: SchemaMatchMode,

    /// Required column names
    #[serde(default)]
    pub required_columns: Vec<String>,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            match_mode: SchemaMatchMode::Tolerant,
            required_columns: Vec::new(),
        }
    }
}

/// Schema matching mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SchemaMatchMode {
    /// Required columns must exist, allow extra columns
    #[default]
    Tolerant,

    /// Exact match: required columns only, no extras
    Strict,
}

/// Output file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format (csv, json, parquet)
    #[serde(default = "default_parquet_format")]
    pub format: String,

    /// Output path template (supports {date} substitution)
    #[serde(default)]
    pub path_template: String,

    /// Whether to overwrite existing files
    #[serde(default = "default_true")]
    pub overwrite: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_parquet_format(),
            path_template: String::new(),
            overwrite: default_true(),
        }
    }
}

/// Transformation step (tagged enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Step {
    /// Drop specified columns
    DropColumns { columns: Vec<String> },

    /// Rename columns according to mapping
    RenameColumns { mapping: HashMap<String, String> },

    /// Trim leading/trailing whitespace
    TrimWhitespace { columns: Vec<String> },

    /// Cast columns to target data types
    CastTypes {
        /// Map of column name to type string (e.g., "i64", "f64", "String")
        columns: HashMap<String, String>,
    },

    /// Parse date/time columns with specified format
    ParseDates {
        /// Map of column name to date format string
        columns: HashMap<String, String>,
    },

    /// Impute missing values
    Impute {
        strategy: ImputeStrategy,
        columns: Vec<String>,
    },

    /// One-hot encode categorical columns
    OneHotEncode {
        columns: Vec<String>,
        drop_original: bool,
    },

    /// Normalize numeric columns
    NormaliseColumns {
        method: NormalisationMethod,
        columns: Vec<String>,
    },

    /// Clip outliers using quantiles
    ClipOutliers {
        columns: Vec<String>,
        lower_quantile: f64,
        upper_quantile: f64,
    },

    /// Extract numbers from text using regex
    ExtractNumbers { columns: Vec<String> },

    /// Apply regex replacement
    RegexReplace {
        columns: Vec<String>,
        pattern: String,
        replacement: String,
    },
}

/// Imputation strategy for missing values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImputeStrategy {
    Mean,
    Median,
    Mode,
    Zero,
}

/// Normalization method for numeric columns
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalisationMethod {
    ZScore,
    MinMax,
}

// Default value functions
fn default_format() -> String {
    "csv".to_owned()
}

fn default_parquet_format() -> String {
    "parquet".to_owned()
}

fn default_true() -> bool {
    true
}

fn default_delimiter() -> String {
    ",".to_owned()
}

fn default_encoding() -> String {
    "utf-8".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_serialization() {
        let mut spec = PipelineSpec::new("test_pipeline");
        spec.steps.push(Step::DropColumns {
            columns: vec!["col1".to_owned(), "col2".to_owned()],
        });
        spec.steps.push(Step::TrimWhitespace {
            columns: vec!["name".to_owned()],
        });

        // Serialize to JSON
        let json = spec.to_json().expect("Failed to serialize");
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"op\": \"drop_columns\""));

        // Deserialize back
        let parsed = PipelineSpec::from_json(&json).expect("Failed to parse");
        assert_eq!(parsed.version, "0.1");
        assert_eq!(parsed.name, "test_pipeline");
        assert_eq!(parsed.steps.len(), 2);
    }

    #[test]
    fn test_from_clean_configs() {
        let mut configs = HashMap::new();

        let config = ColumnCleanConfig {
            active: true,
            new_name: "customer_id".to_owned(),
            trim_whitespace: true,
            advanced_cleaning: true,
            ..Default::default()
        };

        configs.insert("cust_id".to_owned(), config);

        let spec =
            PipelineSpec::from_clean_configs("test", &configs, "csv", "output/cleaned.parquet");

        assert_eq!(spec.name, "test");
        assert!(!spec.steps.is_empty());
    }
}
