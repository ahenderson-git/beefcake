//! Data Dictionary module for capturing dataset metadata and business context.
//!
//! This module provides functionality to create immutable snapshots of dataset metadata,
//! combining automatically-captured technical statistics with user-editable business
//! semantics. Snapshots are versioned and linked to pipeline executions.
//!
//! ## Core Concepts
//!
//! - **Snapshot**: Immutable point-in-time record of dataset and column metadata
//! - **Technical Metadata**: Auto-captured statistics (read-only)
//! - **Business Metadata**: User-editable semantic layer (descriptions, ownership, etc.)
//! - **Versioning**: Snapshots link to previous versions via `previous_snapshot_id`
//!
//! ## Usage
//!
//! ```no_run
//! use beefcake::dictionary::{create_snapshot, storage::save_snapshot};
//! use polars::prelude::*;
//! use std::path::Path;
//!
//! # fn example(df: &DataFrame) -> anyhow::Result<()> {
//! let snapshot = create_snapshot(
//!     "my_dataset",
//!     df,
//!     Path::new("input.csv").to_path_buf(),
//!     Path::new("output.parquet").to_path_buf(),
//!     Some("pipeline-json-here".to_string()),
//!     None, // No previous snapshot
//! )?;
//!
//! // Save to disk
//! save_snapshot(&snapshot, Path::new("data/dictionaries"))?;
//!
//! // Export as Markdown
//! let markdown = beefcake::dictionary::render_markdown(&snapshot)?;
//! std::fs::write("dictionary.md", markdown)?;
//! # Ok(())
//! # }
//! ```

pub mod metadata;
pub mod profiler;
pub mod renderer;
pub mod storage;

pub use metadata::{
    ColumnBusinessMetadata, ColumnMetadata, DataDictionary, DatasetBusinessMetadata,
    DatasetMetadata, QualitySummary, TechnicalMetadata,
};
pub use profiler::create_snapshot;
pub use renderer::render_markdown;
pub use storage::{list_snapshots, load_snapshot, save_snapshot};
