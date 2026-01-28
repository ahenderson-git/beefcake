//! Integrity receipt data structures and creation logic.

use crate::error::{Result, ResultExt as _};
use chrono::{DateTime, Utc};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Current receipt schema version.
///
/// Increment this when making breaking changes to the receipt format.
/// Verifiers can use this to handle multiple receipt versions gracefully.
pub const RECEIPT_VERSION: u32 = 1;

/// Complete integrity receipt for an exported dataset.
///
/// This structure captures everything needed to verify file integrity and
/// understand export provenance. All fields are immutable once created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReceipt {
    /// Schema version for forward compatibility
    pub receipt_version: u32,

    /// UTC timestamp when receipt was created
    pub created_utc: DateTime<Utc>,

    /// Information about the application that produced this export
    pub producer: ProducerInfo,

    /// Metadata about the exported file
    pub export: ExportInfo,

    /// Cryptographic integrity data
    pub integrity: IntegrityInfo,
}

/// Information about the application that produced the export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerInfo {
    /// Application name (e.g., "beefcake")
    pub app_name: String,

    /// Application version (e.g., "0.2.3")
    pub app_version: String,

    /// Platform identifier (e.g., "windows", "linux", "macos")
    pub platform: String,
}

/// Metadata about the exported file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    /// Filename (relative path preferred for portability)
    pub filename: String,

    /// File format (e.g., "csv", "parquet", "json")
    pub format: String,

    /// File size in bytes
    pub file_size_bytes: u64,

    /// Number of rows in the dataset
    pub row_count: usize,

    /// Number of columns in the dataset
    pub column_count: usize,

    /// Schema: column names and data types
    pub schema: Vec<SchemaColumn>,
}

/// Column schema information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaColumn {
    /// Column name
    pub name: String,

    /// Polars data type as string (e.g., "Int64", "Utf8", "Float64")
    pub dtype: String,
}

/// Cryptographic integrity information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityInfo {
    /// Hash algorithm used (currently only "SHA-256")
    pub hash_algorithm: String,

    /// Cryptographic hash as lowercase hexadecimal string
    pub hash: String,
}

/// Create an integrity receipt for an exported file.
///
/// # Arguments
///
/// * `file_path` - Path to the exported file
/// * `df` - Optional DataFrame to extract schema information
///
/// # Returns
///
/// An `IntegrityReceipt` containing all metadata and cryptographic hash.
///
/// # Errors
///
/// Returns error if:
/// - File doesn't exist or can't be read
/// - Hash computation fails
/// - File metadata is inaccessible
pub fn create_receipt(file_path: &Path, df: Option<&DataFrame>) -> Result<IntegrityReceipt> {
    // Compute cryptographic hash
    let hash = crate::integrity::hasher::compute_file_hash(file_path)
        .with_context(|| format!("Failed to compute hash for {}", file_path.display()))?;

    // Get file metadata
    let metadata = fs::metadata(file_path)
        .with_context(|| format!("Failed to read file metadata: {}", file_path.display()))?;

    let file_size_bytes = metadata.len();

    // Extract filename (prefer relative path for portability)
    let filename = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_owned();

    // Determine format from extension
    let format = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    // Extract schema from DataFrame if provided
    let (row_count, column_count, schema) = if let Some(dataframe) = df {
        let row_count = dataframe.height();
        let column_count = dataframe.width();
        let schema: Vec<SchemaColumn> = dataframe
            .schema()
            .iter()
            .map(|(name, dtype)| SchemaColumn {
                name: name.to_string(),
                dtype: format!("{dtype:?}"),
            })
            .collect();

        (row_count, column_count, schema)
    } else {
        // If no DataFrame provided, we can't determine schema
        (0, 0, Vec::new())
    };

    // Build producer info
    let producer = ProducerInfo {
        app_name: env!("CARGO_PKG_NAME").to_owned(),
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        platform: std::env::consts::OS.to_owned(),
    };

    // Build export info
    let export = ExportInfo {
        filename,
        format,
        file_size_bytes,
        row_count,
        column_count,
        schema,
    };

    // Build integrity info
    let integrity = IntegrityInfo {
        hash_algorithm: "SHA-256".to_owned(),
        hash,
    };

    Ok(IntegrityReceipt {
        receipt_version: RECEIPT_VERSION,
        created_utc: Utc::now(),
        producer,
        export,
        integrity,
    })
}

/// Save an integrity receipt to disk.
///
/// The receipt is written as a `.receipt.json` file alongside the exported file.
///
/// # Arguments
///
/// * `receipt` - The receipt to save
/// * `export_path` - Path to the exported file (receipt will be saved alongside)
///
/// # Returns
///
/// Path to the saved receipt file.
///
/// # Errors
///
/// Returns error if file cannot be written or JSON serialization fails.
pub fn save_receipt(receipt: &IntegrityReceipt, export_path: &Path) -> Result<std::path::PathBuf> {
    let receipt_path = export_path.with_extension(format!(
        "{}.receipt.json",
        export_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    ));

    let json = serde_json::to_string_pretty(receipt).context("Failed to serialize receipt")?;

    fs::write(&receipt_path, json)
        .with_context(|| format!("Failed to write receipt to {}", receipt_path.display()))?;

    Ok(receipt_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_receipt_basic() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test data").unwrap();

        let receipt = create_receipt(temp_file.path(), None).unwrap();

        assert_eq!(receipt.receipt_version, RECEIPT_VERSION);
        assert_eq!(receipt.producer.app_name, "beefcake");
        assert_eq!(receipt.integrity.hash_algorithm, "SHA-256");
        assert!(receipt.integrity.hash.len() == 64); // SHA-256 hex length
        assert!(receipt.export.file_size_bytes > 0);
    }

    #[test]
    fn test_save_receipt() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test data").unwrap();

        let receipt = create_receipt(temp_file.path(), None).unwrap();
        let receipt_path = save_receipt(&receipt, temp_file.path()).unwrap();

        assert!(receipt_path.exists());
        assert!(receipt_path.to_string_lossy().contains(".receipt.json"));

        // Verify we can read it back
        let content = fs::read_to_string(&receipt_path).unwrap();
        let loaded: IntegrityReceipt = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.integrity.hash, receipt.integrity.hash);

        // Clean up
        let _ = fs::remove_file(receipt_path);
    }

    #[test]
    fn test_receipt_with_dataframe() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test,data\n1,2\n3,4").unwrap();

        let df = df! {
            "col1" => [1i64, 2, 3],
            "col2" => ["a", "b", "c"],
        }
        .unwrap();

        let receipt = create_receipt(temp_file.path(), Some(&df)).unwrap();

        assert_eq!(receipt.export.row_count, 3);
        assert_eq!(receipt.export.column_count, 2);
        assert_eq!(receipt.export.schema.len(), 2);
        assert_eq!(receipt.export.schema[0].name, "col1");
    }
}
