//! Advanced stage - ML preprocessing (imputation, outliers, features)

use super::{LifecycleStage, StageExecutor};
use crate::analyser::lifecycle::transforms::{TransformPipeline, TransformSpec};
use crate::analyser::logic::ColumnCleanConfig;
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

/// Advanced stage executor
/// Applies ML preprocessing transformations:
/// - Imputation (mean, median, mode)
/// - Outlier detection and clipping
/// - Normalization (z-score, min-max)
/// - Feature engineering
/// - One-hot encoding
pub struct AdvancedStageExecutor {
    pub configs: HashMap<String, ColumnCleanConfig>,
}

impl AdvancedStageExecutor {
    pub fn new(configs: HashMap<String, ColumnCleanConfig>) -> Self {
        Self { configs }
    }
}

impl StageExecutor for AdvancedStageExecutor {
    fn execute(&self, _lf: LazyFrame) -> Result<TransformPipeline> {
        // Advanced stage uses clean_df_lazy with restricted=false to enable
        // ML preprocessing operations
        let spec = TransformSpec {
            transform_type: "clean".to_owned(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "configs".to_owned(),
                    serde_json::to_value(&self.configs).unwrap_or(serde_json::Value::Null),
                );
                params.insert("restricted".to_owned(), serde_json::Value::Bool(false));
                params
            },
        };

        Ok(TransformPipeline::new(vec![spec]))
    }

    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Advanced
    }

    fn description(&self) -> String {
        format!(
            "Apply ML preprocessing to {} columns (imputation, normalisation, outliers)",
            self.configs.len()
        )
    }
}

/// Helper to enable ML preprocessing on configs
pub fn enable_ml_preprocessing(
    configs: &mut HashMap<String, ColumnCleanConfig>,
    enable_imputation: bool,
    enable_normalisation: bool,
    enable_outlier_clipping: bool,
) {
    #[expect(clippy::iter_over_hash_type)]
    for config in configs.values_mut() {
        config.ml_preprocessing = true;

        if enable_imputation && config.impute_mode == crate::analyser::logic::ImputeMode::None {
            config.impute_mode = crate::analyser::logic::ImputeMode::Mean;
        }

        if enable_normalisation
            && config.normalisation == crate::analyser::logic::NormalisationMethod::None
        {
            config.normalisation = crate::analyser::logic::NormalisationMethod::ZScore;
        }

        if enable_outlier_clipping {
            config.clip_outliers = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_executor() {
        let configs = HashMap::new();
        let executor = AdvancedStageExecutor::new(configs);
        assert_eq!(executor.stage(), LifecycleStage::Advanced);
    }

    #[test]
    fn test_enable_ml_preprocessing() {
        let mut configs = HashMap::new();
        let mut config = ColumnCleanConfig::default();
        config.active = true;
        configs.insert("col1".to_owned(), config);

        enable_ml_preprocessing(&mut configs, true, true, true);

        let config = &configs["col1"];
        assert!(config.ml_preprocessing);
        assert!(config.clip_outliers);
    }
}
