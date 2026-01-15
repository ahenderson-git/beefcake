//! Validate stage - QA gates and schema validation

use super::{LifecycleStage, StageExecutor};
use crate::analyser::lifecycle::transforms::TransformPipeline;
use anyhow::{Context as _, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Check null percentage is below threshold
    MaxNullPercent { column: String, max_percent: f64 },
    /// Check value is within range
    ValueRange { column: String, min: f64, max: f64 },
    /// Check column exists
    ColumnExists { column: String },
    /// Check row count is within range
    RowCountRange { min: usize, max: usize },
    /// Check no duplicate values in column
    NoDuplicates { column: String },
    /// Check all values match regex pattern
    MatchesPattern { column: String, pattern: String },
    /// Custom Sql-like condition
    CustomCondition { condition: String },
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule: ValidationRule,
    pub passed: bool,
    pub message: String,
}

/// Validate stage executor
/// Runs QA checks and validation rules
/// This stage doesn't transform data, just validates it
pub struct ValidateStageExecutor {
    pub rules: Vec<ValidationRule>,
}

impl ValidateStageExecutor {
    pub fn new(rules: Vec<ValidationRule>) -> Self {
        Self { rules }
    }

    pub fn default_rules() -> Vec<ValidationRule> {
        vec![
            // Add some sensible default validation rules
        ]
    }

    /// Execute validation rules and return results
    pub fn validate(&self, mut lf: LazyFrame) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for rule in &self.rules {
            let result = Self::validate_rule(rule, &mut lf)?;
            results.push(result);
        }

        Ok(results)
    }

    fn validate_rule(rule: &ValidationRule, lf: &mut LazyFrame) -> Result<ValidationResult> {
        match rule {
            ValidationRule::MaxNullPercent {
                column,
                max_percent,
            } => {
                let df = lf
                    .clone()
                    .select([col(column)])
                    .collect()
                    .context("Failed to collect column")?;
                let series = df
                    .column(column)
                    .context("Column not found")?
                    .as_materialized_series();
                let null_count = series.null_count();
                let total_count = series.len();
                let null_pct = if total_count > 0 {
                    (null_count as f64 / total_count as f64) * 100.0
                } else {
                    0.0
                };

                let passed = null_pct <= *max_percent;
                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed,
                    message: format!(
                        "Column '{column}' has {null_pct:.2}% nulls (max allowed: {max_percent:.2}%)"
                    ),
                })
            }
            ValidationRule::ColumnExists { column } => {
                let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!(e))?;
                let passed = schema.contains(column);
                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed,
                    message: if passed {
                        format!("Column '{column}' exists")
                    } else {
                        format!("Column '{column}' does not exist")
                    },
                })
            }
            ValidationRule::RowCountRange { min, max } => {
                let count_df = lf
                    .clone()
                    .select([len()])
                    .collect()
                    .context("Failed to count rows")?;
                let row_count = count_df
                    .column("len")
                    .context("Failed to get row count")?
                    .as_materialized_series()
                    .u32()
                    .context("Row count not u32")?
                    .get(0)
                    .unwrap_or(0) as usize;

                let passed = row_count >= *min && row_count <= *max;
                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed,
                    message: format!(
                        "Row count {} is {}in range [{}, {}]",
                        row_count,
                        if passed { "" } else { "not " },
                        min,
                        max
                    ),
                })
            }
            ValidationRule::NoDuplicates { column } => {
                let df = lf
                    .clone()
                    .select([col(column)])
                    .collect()
                    .context("Failed to collect column")?;
                let series = df
                    .column(column)
                    .context("Column not found")?
                    .as_materialized_series();
                let total = series.len();
                let unique = series.n_unique().unwrap_or(0);
                let passed = total == unique;

                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed,
                    message: format!(
                        "Column '{}' has {} unique values out of {} (duplicates: {})",
                        column,
                        unique,
                        total,
                        total - unique
                    ),
                })
            }
            ValidationRule::ValueRange { column, min, max } => {
                let df = lf
                    .clone()
                    .select([col(column)])
                    .collect()
                    .context("Failed to collect column")?;
                let series = df
                    .column(column)
                    .context("Column not found")?
                    .as_materialized_series();

                let series_f64 = series
                    .cast(&DataType::Float64)
                    .context("Failed to cast to float")?;
                let ca = series_f64.f64().context("Not a float column")?;

                let min_val = ca.min();
                let max_val = ca.max();

                let passed = match (min_val, max_val) {
                    (Some(min_v), Some(max_v)) => min_v >= *min && max_v <= *max,
                    _ => false,
                };

                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed,
                    message: format!(
                        "Column '{}' range [{:?}, {:?}] {}within expected [{}, {}]",
                        column,
                        min_val,
                        max_val,
                        if passed { "" } else { "not " },
                        min,
                        max
                    ),
                })
            }
            ValidationRule::MatchesPattern { column, pattern: _ } => {
                // Pattern matching validation - placeholder for now
                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed: true,
                    message: format!("Pattern validation for '{column}' not yet implemented"),
                })
            }
            ValidationRule::CustomCondition { condition } => {
                // Custom condition validation - placeholder for now
                Ok(ValidationResult {
                    rule: rule.clone(),
                    passed: true,
                    message: format!("Custom condition '{condition}' not yet implemented"),
                })
            }
        }
    }
}

impl StageExecutor for ValidateStageExecutor {
    fn execute(&self, lf: LazyFrame) -> Result<TransformPipeline> {
        // Run validation
        let results = self.validate(lf.clone())?;

        // Check if all validations passed
        let all_passed = results.iter().all(|r| r.passed);

        if !all_passed {
            let failed_rules: Vec<String> = results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| r.message.clone())
                .collect();

            return Err(anyhow::anyhow!(
                "Validation failed:\n{}",
                failed_rules.join("\n")
            ));
        }

        // Validation stage doesn't transform data
        Ok(TransformPipeline::empty())
    }

    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Validated
    }

    fn description(&self) -> String {
        format!("Run {} QA validation rules", self.rules.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_executor() {
        let rules = vec![ValidationRule::ColumnExists {
            column: "test".to_owned(),
        }];
        let executor = ValidateStageExecutor::new(rules);
        assert_eq!(executor.stage(), LifecycleStage::Validated);
    }
}
