//! Pipeline specification validation.
//!
//! Validates pipeline specs against input data schemas before execution,
//! catching errors early with actionable error messages.

use super::spec::{PipelineSpec, SchemaMatchMode, Step};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashSet;

/// Validation error with helpful context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub step_index: Option<usize>,
    pub message: String,
}

impl ValidationError {
    fn new(step_index: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            step_index,
            message: message.into(),
        }
    }

    fn step(step_index: usize, message: impl Into<String>) -> Self {
        Self::new(Some(step_index), message)
    }

    fn schema(message: impl Into<String>) -> Self {
        Self::new(None, message)
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(idx) = self.step_index {
            write!(f, "Step {}: {}", idx + 1, self.message)
        } else {
            write!(f, "Schema: {}", self.message)
        }
    }
}

/// Validate a pipeline spec against an input schema
pub fn validate_pipeline(
    spec: &PipelineSpec,
    input_schema: &Schema,
) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate spec version
    if spec.version != super::spec::SPEC_VERSION {
        errors.push(ValidationError::schema(format!(
            "Unsupported spec version '{}', expected '{}'",
            spec.version,
            super::spec::SPEC_VERSION
        )));
    }

    // Validate schema requirements
    validate_schema_requirements(spec, input_schema, &mut errors);

    // Simulate step-by-step execution to track schema changes
    let mut current_columns: HashSet<String> = input_schema
        .iter_names()
        .map(|s| s.as_str().to_owned())
        .collect();

    for (idx, step) in spec.steps.iter().enumerate() {
        validate_step(step, idx, &mut current_columns, &mut errors);
    }

    Ok(errors)
}

/// Validate schema matching requirements
fn validate_schema_requirements(
    spec: &PipelineSpec,
    input_schema: &Schema,
    errors: &mut Vec<ValidationError>,
) {
    let input_cols: HashSet<String> = input_schema
        .iter_names()
        .map(|s| s.as_str().to_owned())
        .collect();

    // Check required columns exist
    for required in &spec.schema.required_columns {
        if !input_cols.contains(required) {
            errors.push(ValidationError::schema(format!(
                "Required column '{required}' not found in input"
            )));
        }
    }

    // Strict mode: no extra columns allowed
    if matches!(spec.schema.match_mode, SchemaMatchMode::Strict) {
        let required_set: HashSet<_> = spec.schema.required_columns.iter().cloned().collect();
        let extra_cols: Vec<_> = input_cols.difference(&required_set).collect();

        if !extra_cols.is_empty() {
            errors.push(ValidationError::schema(format!(
                "Strict mode: unexpected columns found: {extra_cols:?}"
            )));
        }
    }
}

/// Validate a single step and update column tracking
fn validate_step(
    step: &Step,
    idx: usize,
    columns: &mut HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    match step {
        Step::DropColumns { columns: drop_cols } => {
            for col in drop_cols {
                if !columns.contains(col) {
                    errors.push(ValidationError::step(
                        idx,
                        format!("Cannot drop non-existent column '{col}'"),
                    ));
                } else {
                    columns.remove(col);
                }
            }
        }

        Step::RenameColumns { mapping } =>
        {
            #[expect(clippy::iter_over_hash_type)]
            for (from, to) in mapping {
                if !columns.contains(from) {
                    errors.push(ValidationError::step(
                        idx,
                        format!("Cannot rename non-existent column '{from}'"),
                    ));
                } else if columns.contains(to) && from != to {
                    errors.push(ValidationError::step(
                        idx,
                        format!("Cannot rename '{from}' to '{to}': target already exists"),
                    ));
                } else {
                    columns.remove(from);
                    columns.insert(to.clone());
                }
            }
        }

        Step::TrimWhitespace { columns: trim_cols } => {
            validate_columns_exist(trim_cols, columns, idx, "trim whitespace", errors);
        }

        Step::CastTypes { columns: cast_cols } => {
            validate_columns_exist(
                &cast_cols.keys().cloned().collect::<Vec<_>>(),
                columns,
                idx,
                "cast type",
                errors,
            );

            // Validate type strings
            #[expect(clippy::iter_over_hash_type)]
            for (col, type_str) in cast_cols {
                if !is_valid_type_string(type_str) {
                    errors.push(ValidationError::step(
                        idx,
                        format!("Invalid type string '{type_str}' for column '{col}'"),
                    ));
                }
            }
        }

        Step::ParseDates { columns: date_cols } => {
            validate_columns_exist(
                &date_cols.keys().cloned().collect::<Vec<_>>(),
                columns,
                idx,
                "parse dates",
                errors,
            );
        }

        Step::Impute {
            strategy: _,
            columns: impute_cols,
        } => {
            validate_columns_exist(impute_cols, columns, idx, "impute", errors);
        }

        Step::OneHotEncode {
            columns: encode_cols,
            drop_original,
        } => {
            validate_columns_exist(encode_cols, columns, idx, "one-hot encode", errors);

            // After one-hot encoding, original columns are replaced with encoded versions
            if *drop_original {
                for col in encode_cols {
                    columns.remove(col);
                    // We don't know the exact encoded column names without data,
                    // so we just note that new columns will be created
                }
            }
        }

        Step::NormaliseColumns {
            method: _,
            columns: norm_cols,
        } => {
            validate_columns_exist(norm_cols, columns, idx, "normalize", errors);
        }

        Step::ClipOutliers {
            columns: clip_cols,
            lower_quantile,
            upper_quantile,
        } => {
            validate_columns_exist(clip_cols, columns, idx, "clip outliers", errors);

            if *lower_quantile < 0.0 || *lower_quantile > 1.0 {
                errors.push(ValidationError::step(
                    idx,
                    format!("Invalid lower_quantile: {lower_quantile} (must be 0-1)"),
                ));
            }

            if *upper_quantile < 0.0 || *upper_quantile > 1.0 {
                errors.push(ValidationError::step(
                    idx,
                    format!("Invalid upper_quantile: {upper_quantile} (must be 0-1)"),
                ));
            }

            if lower_quantile >= upper_quantile {
                errors.push(ValidationError::step(
                    idx,
                    "lower_quantile must be less than upper_quantile".to_owned(),
                ));
            }
        }

        Step::ExtractNumbers {
            columns: extract_cols,
        } => {
            validate_columns_exist(extract_cols, columns, idx, "extract numbers", errors);
        }

        Step::RegexReplace {
            columns: regex_cols,
            pattern,
            replacement: _,
        } => {
            validate_columns_exist(regex_cols, columns, idx, "regex replace", errors);

            // Validate regex pattern
            if let Err(e) = regex::Regex::new(pattern) {
                errors.push(ValidationError::step(
                    idx,
                    format!("Invalid regex pattern: {e}"),
                ));
            }
        }
    }
}

/// Helper to validate that all specified columns exist
fn validate_columns_exist(
    target_cols: &[String],
    available_cols: &HashSet<String>,
    step_idx: usize,
    operation: &str,
    errors: &mut Vec<ValidationError>,
) {
    for col in target_cols {
        if !available_cols.contains(col) {
            errors.push(ValidationError::step(
                step_idx,
                format!("Cannot {operation} non-existent column '{col}'"),
            ));
        }
    }
}

/// Check if a type string is valid
fn is_valid_type_string(type_str: &str) -> bool {
    matches!(
        type_str,
        "i64" | "f64" | "String" | "Boolean" | "Numeric" | "Text" | "Categorical" | "Temporal"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::spec::ImputeStrategy;
    use std::collections::HashMap;

    fn create_test_schema() -> Schema {
        Schema::from_iter(vec![
            Field::new("id".into(), DataType::Int64),
            Field::new("name".into(), DataType::String),
            Field::new("age".into(), DataType::Int64),
        ])
    }

    #[test]
    fn test_validate_drop_columns() {
        let spec = PipelineSpec {
            version: super::super::spec::SPEC_VERSION.to_owned(),
            name: "test".to_owned(),
            input: Default::default(),
            schema: Default::default(),
            steps: vec![Step::DropColumns {
                columns: vec!["id".to_owned(), "nonexistent".to_owned()],
            }],
            output: Default::default(),
        };

        let schema = create_test_schema();
        let errors = validate_pipeline(&spec, &schema).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_validate_rename_conflict() {
        let mut mapping = HashMap::new();
        mapping.insert("id".to_owned(), "name".to_owned());

        let spec = PipelineSpec {
            version: super::super::spec::SPEC_VERSION.to_owned(),
            name: "test".to_owned(),
            input: Default::default(),
            schema: Default::default(),
            steps: vec![Step::RenameColumns { mapping }],
            output: Default::default(),
        };

        let schema = create_test_schema();
        let errors = validate_pipeline(&spec, &schema).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("target already exists"));
    }

    #[test]
    fn test_validate_schema_requirements() {
        let spec = PipelineSpec {
            version: super::super::spec::SPEC_VERSION.to_owned(),
            name: "test".to_owned(),
            input: Default::default(),
            schema: super::super::spec::SchemaConfig {
                match_mode: SchemaMatchMode::Strict,
                required_columns: vec!["id".to_owned(), "missing".to_owned()],
            },
            steps: vec![],
            output: Default::default(),
        };

        let schema = create_test_schema();
        let errors = validate_pipeline(&spec, &schema).unwrap();

        // Should have error for missing required column + strict mode extras
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("missing")));
    }

    #[test]
    fn test_validate_valid_pipeline() {
        let spec = PipelineSpec {
            version: super::super::spec::SPEC_VERSION.to_owned(),
            name: "test".to_owned(),
            input: Default::default(),
            schema: Default::default(),
            steps: vec![
                Step::TrimWhitespace {
                    columns: vec!["name".to_owned()],
                },
                Step::Impute {
                    strategy: ImputeStrategy::Mean,
                    columns: vec!["age".to_owned()],
                },
            ],
            output: Default::default(),
        };

        let schema = create_test_schema();
        let errors = validate_pipeline(&spec, &schema).unwrap();

        assert_eq!(errors.len(), 0);
    }
}
