//! Clean stage - deterministic text and type transformations

use super::{LifecycleStage, StageExecutor};
use crate::analyser::lifecycle::transforms::{TransformPipeline, TransformSpec};
use crate::analyser::logic::ColumnCleanConfig;
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

/// Clean stage executor
/// Applies deterministic transformations:
/// - Text cleaning (trim, case, special chars)
/// - Type casting
/// - Column renaming
/// - Null standardization
pub struct CleanStageExecutor {
    pub configs: HashMap<String, ColumnCleanConfig>,
    pub restricted: bool,
}

impl CleanStageExecutor {
    pub fn new(configs: HashMap<String, ColumnCleanConfig>) -> Self {
        Self {
            configs,
            restricted: false, // Allow column renaming and basic cleaning operations
        }
    }

    pub fn with_restricted(mut self, restricted: bool) -> Self {
        self.restricted = restricted;
        self
    }
}

impl StageExecutor for CleanStageExecutor {
    fn execute(&self, _lf: LazyFrame) -> Result<TransformPipeline> {
        let spec = TransformSpec {
            transform_type: "clean".to_owned(),
            parameters: {
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
            },
        };

        Ok(TransformPipeline::new(vec![spec]))
    }

    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Cleaned
    }

    fn description(&self) -> String {
        format!(
            "Apply deterministic cleaning to {} columns",
            self.configs.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_executor() {
        let configs = HashMap::new();
        let executor = CleanStageExecutor::new(configs);
        assert_eq!(executor.stage(), LifecycleStage::Cleaned);
        assert!(executor.restricted);
    }
}
