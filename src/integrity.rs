//! Export Integrity Receipt System
//!
//! This module provides cryptographic integrity verification for exported datasets.
//! When Beefcake exports a dataset, it can optionally generate an integrity receipt—a
//! JSON file containing metadata and a cryptographic hash of the exported file.
//!
//! ## Purpose
//!
//! - **Tamper Detection**: Verify that exported files haven't been modified
//! - **Audit Trail**: Track when, how, and by whom data was exported
//! - **Forward Compatibility**: Receipt schema supports future enhancements
//! - **Enterprise Use**: Designed for regulated data workflows requiring data integrity
//!
//! ## Key Concepts
//!
//! - **Receipt**: Immutable JSON record of export metadata + cryptographic hash
//! - **Streaming Hash**: Computed without loading entire file into memory
//! - **Verification**: Deterministic pass/fail check with detailed diagnostics
//!
//! ## Usage
//!
//! ### Generate Receipt During Export
//!
//! ```no_run
//! use beefcake::integrity;
//! use polars::prelude::*;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let export_path = Path::new("output/dataset.csv");
//! let df = DataFrame::default(); // Your data
//!
//! // Create receipt after export
//! let receipt = integrity::create_receipt(export_path, Some(&df))?;
//! integrity::save_receipt(&receipt, export_path)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Verify Integrity Later
//!
//! ```no_run
//! use beefcake::integrity;
//! use std::path::Path;
//!
//! # fn example() -> anyhow::Result<()> {
//! let receipt_path = Path::new("output/dataset.csv.receipt.json");
//! let result = integrity::verify_receipt(receipt_path)?;
//!
//! if result.passed {
//!     println!("✓ PASS: File integrity verified");
//! } else {
//!     println!("✗ FAIL: {}", result.message);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Receipt Format
//!
//! Receipts are stored as human-readable JSON files:
//!
//! ```json
//! {
//!   "receipt_version": 1,
//!   "created_utc": "2026-01-24T12:34:56.789Z",
//!   "producer": {
//!     "app_name": "beefcake",
//!     "app_version": "0.2.3",
//!     "platform": "windows"
//!   },
//!   "export": {
//!     "filename": "dataset.csv",
//!     "format": "csv",
//!     "file_size_bytes": 1048576,
//!     "row_count": 10000,
//!     "column_count": 15,
//!     "schema": [...]
//!   },
//!   "integrity": {
//!     "hash_algorithm": "SHA-256",
//!     "hash": "a3b2c1d4..."
//!   }
//! }
//! ```
//!
//! ## Edge Cases & Limitations
//!
//! - **CSV Line Endings**: Converting CRLF ↔ LF will fail verification
//! - **Moved Files**: Verification uses relative path from receipt; moved files require manual path adjustment
//! - **Determinism**: Hash is sensitive to byte-level changes (metadata, compression settings, etc.)
//!
//! ## Architecture
//!
//! This module is organized into focused submodules:
//!
//! - [`receipt`]: Data structures for integrity receipts
//! - [`hasher`]: Streaming hash computation
//! - [`verifier`]: Verification logic and diagnostics

pub mod hasher;
pub mod receipt;
pub mod verifier;

pub use hasher::compute_file_hash;
pub use receipt::{
    ExportInfo, IntegrityInfo, IntegrityReceipt, ProducerInfo, SchemaColumn, create_receipt,
    save_receipt,
};
pub use verifier::{VerificationResult, verify_receipt};
