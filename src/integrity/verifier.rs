//! Integrity receipt verification logic.
//!
//! This module provides functions to verify that a file hasn't been tampered
//! with by comparing its current hash against the hash stored in a receipt.

use crate::error::{BeefcakeError, Result, ResultExt as _};
use crate::integrity::hasher::compute_file_hash;
use crate::integrity::receipt::IntegrityReceipt;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Result of an integrity verification check.
///
/// This structure provides detailed diagnostics for both passing and failing
/// verifications, suitable for logging, auditing, and user display.
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    /// Whether verification passed (true) or failed (false)
    pub passed: bool,

    /// Human-readable message describing the result
    pub message: String,

    /// Path to the file that was verified
    pub file_path: String,

    /// Expected hash from receipt
    pub expected_hash: String,

    /// Actual hash computed from file (if available)
    pub actual_hash: Option<String>,

    /// Receipt metadata for context
    pub receipt: IntegrityReceipt,
}

impl VerificationResult {
    /// Create a passing verification result.
    fn pass(file_path: String, hash: String, receipt: IntegrityReceipt) -> Self {
        Self {
            passed: true,
            message: "File integrity verified successfully".to_owned(),
            file_path,
            expected_hash: hash.clone(),
            actual_hash: Some(hash),
            receipt,
        }
    }

    /// Create a failing verification result.
    fn fail(
        file_path: String,
        expected: String,
        actual: Option<String>,
        reason: String,
        receipt: IntegrityReceipt,
    ) -> Self {
        Self {
            passed: false,
            message: reason,
            file_path,
            expected_hash: expected,
            actual_hash: actual,
            receipt,
        }
    }

    /// Format verification result for CLI display.
    ///
    /// Produces user-friendly output suitable for terminal display with
    /// color-coded pass/fail indicators.
    pub fn format_cli(&self) -> String {
        if self.passed {
            format!(
                "✓ PASS: File integrity verified\n  \
                File: {}\n  \
                Hash: {} ({})\n  \
                Rows: {}, Columns: {}\n  \
                Created: {}",
                self.file_path,
                &self.expected_hash[..16],
                self.receipt.integrity.hash_algorithm,
                self.receipt.export.row_count,
                self.receipt.export.column_count,
                self.receipt.created_utc.format("%Y-%m-%d %H:%M:%S UTC")
            )
        } else {
            let mut output = format!(
                "✗ FAIL: {}\n  \
                File: {}\n  \
                Expected: {}\n  ",
                self.message, self.file_path, self.expected_hash
            );

            if let Some(actual) = &self.actual_hash {
                output.push_str(&format!("Actual:   {actual}\n  "));
            }

            output.push_str("File may have been modified or corrupted");
            output
        }
    }
}

/// Verify the integrity of a file using its receipt.
///
/// This function:
/// 1. Loads the integrity receipt from disk
/// 2. Locates the associated data file
/// 3. Recomputes the file's cryptographic hash
/// 4. Compares expected vs actual hash
/// 5. Returns detailed pass/fail result
///
/// # Arguments
///
/// * `receipt_path` - Path to the `.receipt.json` file
///
/// # Returns
///
/// A `VerificationResult` containing:
/// - Pass/fail status
/// - Expected and actual hashes
/// - Diagnostic messages
/// - Receipt metadata
///
/// # Errors
///
/// Returns error if:
/// - Receipt file doesn't exist or can't be read
/// - Receipt JSON is malformed
/// - Data file referenced by receipt doesn't exist
/// - Hash computation fails
///
/// # Example
///
/// ```no_run
/// use beefcake::integrity::verify_receipt;
/// use std::path::Path;
///
/// # fn example() -> anyhow::Result<()> {
/// let result = verify_receipt(Path::new("output/data.csv.receipt.json"))?;
///
/// if result.passed {
///     println!("{}", result.format_cli());
/// } else {
///     eprintln!("{}", result.format_cli());
///     std::process::exit(1);
/// }
/// # Ok(())
/// # }
/// ```
pub fn verify_receipt(receipt_path: &Path) -> Result<VerificationResult> {
    // Load receipt from disk
    let receipt_json = fs::read_to_string(receipt_path)
        .with_context(|| format!("Failed to read receipt file: {}", receipt_path.display()))?;

    let receipt: IntegrityReceipt = serde_json::from_str(&receipt_json)
        .context("Failed to parse receipt JSON (file may be corrupted)")?;

    // Determine data file path
    // Receipt stores filename; we look for it in the same directory as the receipt
    let receipt_dir = receipt_path
        .parent()
        .ok_or_else(|| BeefcakeError::InvalidPath("Receipt has no parent directory".to_owned()))?;

    let data_file_path = receipt_dir.join(&receipt.export.filename);

    // Check if data file exists
    if !data_file_path.exists() {
        return Ok(VerificationResult::fail(
            data_file_path.display().to_string(),
            receipt.integrity.hash.clone(),
            None,
            format!(
                "Data file not found: {}. File may have been moved or deleted.",
                receipt.export.filename
            ),
            receipt,
        ));
    }

    // Recompute hash
    let actual_hash = match compute_file_hash(&data_file_path) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(VerificationResult::fail(
                data_file_path.display().to_string(),
                receipt.integrity.hash.clone(),
                None,
                format!("Failed to compute hash: {e}"),
                receipt,
            ));
        }
    };

    // Compare hashes
    if actual_hash == receipt.integrity.hash {
        Ok(VerificationResult::pass(
            data_file_path.display().to_string(),
            actual_hash,
            receipt,
        ))
    } else {
        Ok(VerificationResult::fail(
            data_file_path.display().to_string(),
            receipt.integrity.hash.clone(),
            Some(actual_hash),
            "Hash mismatch detected".to_owned(),
            receipt,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::receipt::{create_receipt, save_receipt};
    use tempfile::TempDir;

    #[test]
    fn test_verify_receipt_pass() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test.csv");
        fs::write(&data_file, b"col1,col2\n1,2\n3,4").unwrap();

        // Create receipt
        let receipt = create_receipt(&data_file, None).unwrap();
        let receipt_path = save_receipt(&receipt, &data_file).unwrap();

        // Verify
        let result = verify_receipt(&receipt_path).unwrap();

        assert!(result.passed);
        assert_eq!(result.expected_hash, result.actual_hash.unwrap());
        assert!(result.message.contains("verified successfully"));
    }

    #[test]
    fn test_verify_receipt_fail_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test.csv");
        fs::write(&data_file, b"original content").unwrap();

        // Create receipt
        let receipt = create_receipt(&data_file, None).unwrap();
        let receipt_path = save_receipt(&receipt, &data_file).unwrap();

        // Modify file
        fs::write(&data_file, b"MODIFIED CONTENT").unwrap();

        // Verify
        let result = verify_receipt(&receipt_path).unwrap();

        assert!(!result.passed);
        assert!(result.message.contains("Hash mismatch"));
        assert_ne!(result.expected_hash, result.actual_hash.unwrap());
    }

    #[test]
    fn test_verify_receipt_fail_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test.csv");
        fs::write(&data_file, b"content").unwrap();

        // Create receipt
        let receipt = create_receipt(&data_file, None).unwrap();
        let receipt_path = save_receipt(&receipt, &data_file).unwrap();

        // Delete data file
        fs::remove_file(&data_file).unwrap();

        // Verify
        let result = verify_receipt(&receipt_path).unwrap();

        assert!(!result.passed);
        assert!(result.message.contains("not found"));
        assert!(result.actual_hash.is_none());
    }

    #[test]
    fn test_verify_receipt_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let receipt_path = temp_dir.path().join("bad.receipt.json");
        fs::write(&receipt_path, b"{ invalid json }").unwrap();

        let result = verify_receipt(&receipt_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_verification_result_format_cli_pass() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test.csv");
        fs::write(&data_file, b"data").unwrap();

        let receipt = create_receipt(&data_file, None).unwrap();
        let receipt_path = save_receipt(&receipt, &data_file).unwrap();
        let result = verify_receipt(&receipt_path).unwrap();

        let output = result.format_cli();
        assert!(output.contains("✓ PASS"));
        assert!(output.contains("File integrity verified"));
        assert!(output.contains("test.csv"));
    }

    #[test]
    fn test_verification_result_format_cli_fail() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test.csv");
        fs::write(&data_file, b"original").unwrap();

        let receipt = create_receipt(&data_file, None).unwrap();
        let receipt_path = save_receipt(&receipt, &data_file).unwrap();

        // Modify file
        fs::write(&data_file, b"modified").unwrap();

        let result = verify_receipt(&receipt_path).unwrap();
        let output = result.format_cli();

        assert!(output.contains("✗ FAIL"));
        assert!(output.contains("Hash mismatch"));
        assert!(output.contains("Expected:"));
        assert!(output.contains("Actual:"));
    }
}
