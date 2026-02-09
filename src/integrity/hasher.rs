//! Streaming cryptographic hash computation.
//!
//! This module provides efficient file hashing without loading entire files
//! into memory. Critical for large datasets in enterprise environments.

use crate::error::{Result, ResultExt as _};
use sha2::{Digest as _, Sha256};
use std::fs::File;
use std::io::{BufReader, Read as _};
use std::path::Path;

/// Buffer size for streaming file reads (8 KB).
///
/// This is a conservative choice that balances memory usage with I/O efficiency.
/// Larger buffers (64KB+) may improve performance for very large files on SSDs,
/// but 8KB works well across spinning disks, SSDs, and network filesystems.
const BUFFER_SIZE: usize = 8192;

/// Compute SHA-256 hash of a file using streaming I/O.
///
/// This function reads the file in chunks and updates the hash incrementally,
/// making it memory-efficient even for multi-gigabyte files.
///
/// # Arguments
///
/// * `path` - Path to the file to hash
///
/// # Returns
///
/// SHA-256 hash as a lowercase hexadecimal string (64 characters).
///
/// # Errors
///
/// Returns error if:
/// - File doesn't exist or can't be opened
/// - I/O error occurs during reading
/// - Insufficient permissions
///
/// # Performance
///
/// - **Memory**: O(1) - uses fixed 8KB buffer regardless of file size
/// - **Time**: O(n) - linear in file size
/// - **Throughput**: Typically 200-500 MB/s depending on storage and CPU
///
/// # Example
///
/// ```no_run
/// use beefcake::integrity::hasher::compute_file_hash;
/// use std::path::Path;
///
/// # fn example() -> anyhow::Result<()> {
/// let hash = compute_file_hash(Path::new("large_dataset.csv"))?;
/// println!("SHA-256: {}", hash);
/// # Ok(())
/// # }
/// ```
pub fn compute_file_hash(path: &Path) -> Result<String> {
    // Open file with buffering
    let file = File::open(path)
        .with_context(|| format!("Failed to open file for hashing: {}", path.display()))?;

    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; BUFFER_SIZE];

    // Stream file through hasher
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        if bytes_read == 0 {
            break; // EOF
        }

        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize hash and convert to hex
    let hash = hasher.finalize();
    Ok(format!("{hash:x}"))
}

/// Hash algorithm identifier used in receipts.
pub const HASH_ALGORITHM: &str = "SHA-256";

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_file_hash_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        // Empty file
        let hash = compute_file_hash(temp_file.path()).unwrap();

        // SHA-256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_compute_file_hash_known_value() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();

        // Known SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_file_hash_large_file() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write data larger than buffer size to test streaming
        let data = vec![0u8; BUFFER_SIZE * 3 + 100];
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();

        // Verify hash is valid hex string of correct length
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_file_hash_nonexistent() {
        let result = compute_file_hash(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_hashing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let hash1 = compute_file_hash(temp_file.path()).unwrap();
        let hash2 = compute_file_hash(temp_file.path()).unwrap();

        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_different_content_different_hash() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        temp_file1.write_all(b"content A").unwrap();
        temp_file1.flush().unwrap();

        let mut temp_file2 = NamedTempFile::new().unwrap();
        temp_file2.write_all(b"content B").unwrap();
        temp_file2.flush().unwrap();

        let hash1 = compute_file_hash(temp_file1.path()).unwrap();
        let hash2 = compute_file_hash(temp_file2.path()).unwrap();

        assert_ne!(
            hash1, hash2,
            "Different content should produce different hashes"
        );
    }
}
