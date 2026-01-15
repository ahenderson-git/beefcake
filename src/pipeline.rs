//! Pipeline specification and execution system for automated data processing workflows.
//!
//! This module enables capturing GUI data operations as versioned JSON "pipeline specs"
//! that can be executed headlessly via CLI or exported as `PowerShell` automation scripts.
//!
//! # Overview
//!
//! The pipeline system provides 11 transformation steps organized into categories:
//! - **Column Management**: `drop_columns`, `rename_columns`
//! - **Text Processing**: `trim_whitespace`, `regex_replace`
//! - **Type Conversion**: `cast_types`, `parse_dates`
//! - **Missing Values**: impute (mean/median/mode/zero)
//! - **ML Preprocessing**: `normalize_columns`, `one_hot_encode`, `clip_outliers`, `extract_numbers`
//!
//! # Example: Programmatic Pipeline Creation
//!
//! ```no_run
//! use beefcake::pipeline::{PipelineSpec, InputConfig, OutputConfig, run_pipeline};
//! use std::path::PathBuf;
//!
//! let spec = PipelineSpec {
//!     version: "0.1".to_string(),
//!     name: "Data Cleaning".to_string(),
//!     description: Some("Clean customer data".to_string()),
//!     steps: vec![],  // Add your steps here
//!     input: Some(InputConfig { format: Some("csv".to_string()), path: None }),
//!     output: Some(OutputConfig {
//!         format: Some("parquet".to_string()),
//!         path: Some("output.parquet".to_string())
//!     }),
//! };
//!
//! let report = run_pipeline(&spec, PathBuf::from("data.csv"), Some(PathBuf::from("output.parquet")))?;
//! println!("Processed {} rows", report.rows_after);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Pipeline Templates
//!
//! The system includes 8 built-in templates available in the GUI:
//! 1. **Data Cleaning**: Trim whitespace, drop unwanted columns, impute missing values
//! 2. **ML Preprocessing**: Cast types, impute, normalize, one-hot encode categoricals
//! 3. **Date Normalization**: Parse dates with common formats
//! 4. **Text Processing**: Trim, rename to lowercase, ensure type consistency
//! 5. **Outlier Handling**: Clip outliers (1st-99th percentile), normalize with z-score
//! 6. **Column Selection**: Drop unwanted columns, rename for clarity
//! 7. **Missing Data Handling**: Drop high-missingness columns, impute remaining
//! 8. **Type Conversion**: Cast types, parse dates with custom formats

pub mod executor;
pub mod powershell;
pub mod spec;
pub mod validation;

pub use executor::{RunReport, run_pipeline};
pub use powershell::generate_powershell_script;
pub use spec::{
    ImputeStrategy, InputConfig, OutputConfig, PipelineSpec, SPEC_VERSION, SchemaMatchMode, Step,
};
pub use validation::{ValidationError, validate_pipeline};
