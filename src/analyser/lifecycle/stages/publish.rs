//! Publish stage - finalize dataset as view or snapshot

use super::{LifecycleStage, PublishMode, StageExecutor};
use crate::analyser::lifecycle::transforms::TransformPipeline;
use anyhow::Result;
use polars::prelude::*;

/// Publish stage executor
/// Finalizes a dataset for consumption:
/// - View mode: keeps data as `LazyFrame` pipeline (computed on access)
/// - Snapshot mode: materializes data to physical storage
pub struct PublishStageExecutor {
    pub mode: PublishMode,
}

impl PublishStageExecutor {
    pub fn new(mode: PublishMode) -> Self {
        Self { mode }
    }

    pub fn as_view() -> Self {
        Self {
            mode: PublishMode::View,
        }
    }

    pub fn as_snapshot() -> Self {
        Self {
            mode: PublishMode::Snapshot,
        }
    }
}

impl StageExecutor for PublishStageExecutor {
    fn execute(&self, _lf: LazyFrame) -> Result<TransformPipeline> {
        // Publish stage doesn't add transforms
        // The mode (view vs snapshot) is handled by the storage layer
        Ok(TransformPipeline::empty())
    }

    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Published
    }

    fn description(&self) -> String {
        format!("Publish dataset as {}", self.mode.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_executor_view() {
        let executor = PublishStageExecutor::as_view();
        assert_eq!(executor.mode, PublishMode::View);
        assert_eq!(executor.stage(), LifecycleStage::Published);
    }

    #[test]
    fn test_publish_executor_snapshot() {
        let executor = PublishStageExecutor::as_snapshot();
        assert_eq!(executor.mode, PublishMode::Snapshot);
        assert_eq!(executor.stage(), LifecycleStage::Published);
    }
}
