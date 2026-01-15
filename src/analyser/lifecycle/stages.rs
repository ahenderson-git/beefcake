//! Stage-specific logic for dataset lifecycle

pub mod advanced;
pub mod clean;
pub mod profile;
pub mod publish;
pub mod validate;

use anyhow::Result;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

use super::transforms::TransformPipeline;

/// Lifecycle stages for a dataset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Raw ingestion - immutable original data
    Raw,
    /// Profiled - analysis complete with issues and recommendations
    Profiled,
    /// Cleaned - deterministic text/type transformations applied
    Cleaned,
    /// Advanced - ML preprocessing (imputation, outliers, features)
    Advanced,
    /// Validated - QA gates passed
    Validated,
    /// Published - finalized as view or snapshot
    Published,
}

impl LifecycleStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "Raw",
            Self::Profiled => "Profiled",
            Self::Cleaned => "Cleaned",
            Self::Advanced => "Advanced",
            Self::Validated => "Validated",
            Self::Published => "Published",
        }
    }

    pub fn parse_stage(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "raw" => Some(Self::Raw),
            "profiled" => Some(Self::Profiled),
            "cleaned" => Some(Self::Cleaned),
            "advanced" => Some(Self::Advanced),
            "validated" => Some(Self::Validated),
            "published" => Some(Self::Published),
            _ => None,
        }
    }

    /// Get the next logical stage in the pipeline
    pub fn next_stage(&self) -> Option<Self> {
        match self {
            Self::Raw => Some(Self::Profiled),
            Self::Profiled => Some(Self::Cleaned),
            Self::Cleaned => Some(Self::Advanced),
            Self::Advanced => Some(Self::Validated),
            Self::Validated => Some(Self::Published),
            Self::Published => None,
        }
    }

    /// Check if this stage can transition to another stage
    pub fn can_transition_to(&self, target: Self) -> bool {
        match (self, target) {
            // Can always go forward or skip stages going forward
            (
                Self::Raw,
                Self::Profiled | Self::Cleaned | Self::Advanced | Self::Validated | Self::Published,
            )
            | (
                Self::Profiled,
                Self::Cleaned | Self::Advanced | Self::Validated | Self::Published,
            )
            | (Self::Cleaned, Self::Advanced | Self::Validated | Self::Published)
            | (Self::Advanced, Self::Validated | Self::Published)
            | (Self::Validated, Self::Published) => true,
            // Can't go backwards
            _ => false,
        }
    }
}

/// Publish mode - view vs snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PublishMode {
    /// View - logical view (pipeline reference only, computed on access)
    View,
    /// Snapshot - physical copy (materialized data)
    Snapshot,
}

impl PublishMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::View => "View",
            Self::Snapshot => "Snapshot",
        }
    }
}

/// Trait for executing stage-specific operations
pub trait StageExecutor {
    /// Execute this stage and return a transform pipeline
    fn execute(&self, lf: LazyFrame) -> Result<TransformPipeline>;

    /// Get the stage this executor handles
    fn stage(&self) -> LifecycleStage;

    /// Get a description of what this stage does
    fn description(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_transitions() {
        assert!(LifecycleStage::Raw.can_transition_to(LifecycleStage::Profiled));
        assert!(LifecycleStage::Raw.can_transition_to(LifecycleStage::Cleaned));
        assert!(LifecycleStage::Raw.can_transition_to(LifecycleStage::Published));

        assert!(!LifecycleStage::Cleaned.can_transition_to(LifecycleStage::Raw));
        assert!(!LifecycleStage::Published.can_transition_to(LifecycleStage::Raw));
    }

    #[test]
    fn test_next_stage() {
        assert_eq!(
            LifecycleStage::Raw.next_stage(),
            Some(LifecycleStage::Profiled)
        );
        assert_eq!(LifecycleStage::Published.next_stage(), None);
    }

    #[test]
    fn test_stage_string_conversion() {
        assert_eq!(LifecycleStage::Raw.as_str(), "Raw");
        assert_eq!(
            LifecycleStage::parse_stage("cleaned"),
            Some(LifecycleStage::Cleaned)
        );
        assert_eq!(LifecycleStage::parse_stage("invalid"), None);
    }
}
