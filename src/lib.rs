//! # Beefcake - High-Performance Data Analysis Library
//!
//! Beefcake is a Rust library for analyzing, cleaning, and transforming tabular data.
//! It provides a powerful toolkit for data profiling, quality assessment, and automated
//! transformation pipelines.
//!
//! ## Quick Start
//!
//! ```no_run
//! use beefcake::analyser::logic;
//!
//! // Analyze a CSV file
//! # async fn example() -> anyhow::Result<()> {
//! let response = logic::analyze_file_flow("data.csv".into()).await?;
//! println!("Found {} columns", response.column_count);
//!
//! // Access column statistics
//! for col in response.summary {
//!     println!("{}: {} (type: {:?})", col.name, col.count, col.kind);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Core Modules
//!
//! - [`ai`]: AI assistant integration for in-app support and guidance
//! - [`analyser`]: Data analysis, profiling, and quality assessment
//!   - [`analyser::logic`]: Core analysis algorithms
//!   - [`analyser::lifecycle`]: Dataset version management
//! - [`dictionary`]: Data dictionary snapshots and metadata management
//! - [`pipeline`]: Automation and transformation pipeline system
//! - [`error`]: Error types and handling utilities
//! - [`utils`]: Common utility functions
//! - [`watcher`]: File system watcher service
//!
//! ## Key Concepts
//!
//! ### Lazy Evaluation
//!
//! Beefcake uses Polars' `LazyFrame` for efficient data processing. Operations
//! build a query plan that's optimized and executed only when needed:
//!
//! ```no_run
//! use polars::prelude::*;
//!
//! let lf = LazyFrame::scan_parquet("data.parquet", Default::default())?
//!     .select([col("age"), col("name")])
//!     .filter(col("age").gt(18));
//!
//! // Nothing executed yet - just a query plan
//! let df = lf.collect()?;  // Now data is processed
//! # Ok::<(), PolarsError>(())
//! ```
//!
//! ### Immutable Versioning
//!
//! The lifecycle system never modifies original data. Instead, it creates
//! versioned snapshots with transformation pipelines:
//!
//! ```no_run
//! use beefcake::analyser::lifecycle::{DatasetRegistry, LifecycleStage};
//! use std::path::PathBuf;
//!
//! let registry = DatasetRegistry::new(PathBuf::from("./data"))?;
//!
//! // Create dataset with Raw version
//! let dataset_id = registry.create_dataset(
//!     "my-data".to_string(),
//!     PathBuf::from("data.csv")
//! )?;
//!
//! // Apply transformations to create new version
//! // Original data remains unchanged
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ### Type-Safe Error Handling
//!
//! All fallible operations return `Result<T, E>`. Use the `?` operator to
//! propagate errors:
//!
//! ```no_run
//! use anyhow::Result;
//!
//! fn process_data(path: &str) -> Result<String> {
//!     let data = std::fs::read_to_string(path)?;  // Auto-converts errors
//!     let processed = data.trim().to_uppercase();
//!     Ok(processed)
//! }
//! ```
//!
//! ## Learning Rust
//!
//! If you're new to Rust, see `docs/RUST_CONCEPTS.md` for explanations of
//! patterns used in this codebase.
//!
//! ## Architecture
//!
//! For system architecture and design patterns, see `docs/ARCHITECTURE.md`.

#![warn(clippy::all, rust_2018_idioms)]
// Uncomment to see which items need documentation:
// #![warn(missing_docs)]

pub mod ai;
pub mod analyser;
pub mod dictionary;
pub mod error;
pub mod pipeline;
pub mod utils;
pub mod watcher;
