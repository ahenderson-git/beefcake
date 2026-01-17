//! Profile stage - detect issues and generate recommendations

use super::{LifecycleStage, StageExecutor};
use crate::analyser::lifecycle::transforms::TransformPipeline;
use anyhow::Result;
use polars::prelude::*;

/// Profile stage executor
/// This stage runs statistical analysis and generates recommendations
/// but does not modify the data - it only produces metadata
pub struct ProfileStageExecutor {
    pub trim_pct: f64,
}

impl Default for ProfileStageExecutor {
    fn default() -> Self {
        Self { trim_pct: 0.0 }
    }
}

impl StageExecutor for ProfileStageExecutor {
    fn execute(&self, _lf: LazyFrame) -> Result<TransformPipeline> {
        // Profile stage doesn't transform data, just analyzes it
        // The actual profiling is done via analyse_df_lazy which returns ColumnSummary
        // This stage just returns an empty pipeline since no transforms are applied
        Ok(TransformPipeline::empty())
    }

    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Profiled
    }

    fn description(&self) -> String {
        "Analyze data quality, detect issues, and generate cleaning recommendations".to_owned()
    }
}

/// Profile a `LazyFrame` and return analysis results
pub fn profile_data(
    lf: LazyFrame,
    trim_pct: f64,
) -> Result<Vec<crate::analyser::logic::ColumnSummary>> {
    crate::analyser::logic::analyse_df_lazy(lf, trim_pct, 10_000)
}

/// Generate auto-clean configurations from profile results
pub fn generate_clean_configs_from_profile(
    summaries: &[crate::analyser::logic::ColumnSummary],
) -> std::collections::HashMap<String, crate::analyser::logic::ColumnCleanConfig> {
    use crate::analyser::logic::ColumnCleanConfig;
    use std::collections::HashMap;

    let mut configs = HashMap::new();
    for summary in summaries {
        let mut config = ColumnCleanConfig {
            new_name: summary.standardised_name.clone(),
            active: true,
            ..Default::default()
        };
        summary.apply_advice_to_config(&mut config);
        configs.insert(summary.name.clone(), config);
    }
    configs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_executor() {
        let executor = ProfileStageExecutor::default();
        assert_eq!(executor.stage(), LifecycleStage::Profiled);
        assert!(!executor.description().is_empty());
    }
}
