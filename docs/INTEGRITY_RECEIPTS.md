# Export Integrity Receipts

## Overview

The Integrity Receipt system provides cryptographic verification for exported datasets. When Beefcake exports data, it can optionally generate a **receipt**—a JSON file containing metadata and a SHA-256 hash of the exported file. This enables tamper detection and audit trails for regulated data workflows.

## Why Integrity Receipts?

### Use Cases

1. **Regulatory Compliance**: Prove datasets haven't been modified since export (HIPAA, SOX, GDPR)
2. **Data Lineage**: Track when, how, and by whom data was exported
3. **Quality Assurance**: Verify data integrity before downstream processing
4. **Incident Response**: Detect unauthorized modifications to exported files

### What This Is NOT

- ❌ **Not blockchain**: No distributed ledger, just local cryptographic hashing
- ❌ **Not digital signatures**: No public/private key cryptography (yet)
- ❌ **Not access control**: Receipts verify *integrity*, not *authenticity*

## Receipt Format

Receipts are stored as `.receipt.json` files alongside exports:

```
data/
  sales_2026.csv
  sales_2026.csv.receipt.json  ← Receipt file
```

### JSON Structure

```json
{
  "receipt_version": 1,
  "created_utc": "2026-01-24T12:34:56.789Z",

  "producer": {
    "app_name": "beefcake",
    "app_version": "0.2.3",
    "platform": "windows"
  },

  "export": {
    "filename": "sales_2026.csv",
    "format": "csv",
    "file_size_bytes": 1048576,
    "row_count": 10000,
    "column_count": 15,
    "schema": [
      { "name": "customer_id", "dtype": "Int64" },
      { "name": "amount", "dtype": "Float64" },
      { "name": "timestamp", "dtype": "Datetime(Milliseconds, None)" }
    ]
  },

  "integrity": {
    "hash_algorithm": "SHA-256",
    "hash": "a3b2c1d4e5f6789..."
  }
}
```

### Field Descriptions

| Field | Description |
|-------|-------------|
| `receipt_version` | Schema version (currently `1`). Future versions can add fields while maintaining backward compatibility |
| `created_utc` | ISO-8601 timestamp when receipt was created |
| `producer.app_name` | Always `"beefcake"` |
| `producer.app_version` | Beefcake version that created the export |
| `producer.platform` | OS: `"windows"`, `"linux"`, or `"macos"` |
| `export.filename` | Exported file name (relative path) |
| `export.format` | File extension (`csv`, `parquet`, `json`) |
| `export.file_size_bytes` | File size for quick validation |
| `export.row_count` | Number of rows (0 if schema unavailable) |
| `export.column_count` | Number of columns |
| `export.schema` | Per-column name and Polars data type |
| `integrity.hash_algorithm` | Always `"SHA-256"` in v1 |
| `integrity.hash` | Lowercase hex SHA-256 hash (64 characters) |

## Usage

### Creating Receipts (Automatic via Export)

Receipts are automatically created during export when `create_receipt` is enabled:

```rust
use beefcake::export::{ExportOptions, ExportDestination};

let options = ExportOptions {
    source: /* ... */,
    destination: ExportDestination {
        dest_type: ExportDestinationType::File,
        target: "output/data.csv".to_string(),
    },
    configs: HashMap::new(),
    create_dictionary: true,
    create_receipt: true,  // ← Enable receipt generation
};

export_data_execution(options, &mut temp_files).await?;
```

After export, you'll have:
- `output/data.csv` - Your exported data
- `output/data.csv.receipt.json` - Integrity receipt

### Creating Receipts (Manual)

You can also create receipts manually:

```rust
use beefcake::integrity::{create_receipt, save_receipt};
use polars::prelude::*;
use std::path::Path;

// Load your DataFrame
let df = LazyCsvReader::new("data.csv").finish()?.collect()?;

// Create receipt
let receipt = create_receipt(Path::new("data.csv"), Some(&df))?;

// Save alongside file
let receipt_path = save_receipt(&receipt, Path::new("data.csv"))?;
println!("Receipt saved to: {}", receipt_path.display());
```

### Verifying Integrity

Verification recomputes the hash and compares it to the receipt:

```rust
use beefcake::integrity::verify_receipt;
use std::path::Path;

let result = verify_receipt(Path::new("data.csv.receipt.json"))?;

if result.passed {
    println!("✓ PASS: File integrity verified");
} else {
    eprintln!("✗ FAIL: {}", result.message);
    eprintln!("Expected: {}", result.expected_hash);
    eprintln!("Actual:   {}", result.actual_hash.unwrap_or_default());
    std::process::exit(1);
}
```

#### CLI-Friendly Output

```rust
println!("{}", result.format_cli());
```

Output for **passing** verification:
```
✓ PASS: File integrity verified
  File: data/sales.csv
  Hash: a3b2c1d4e5f67890 (SHA-256)
  Rows: 10000, Columns: 15
  Created: 2026-01-24 12:34:56 UTC
```

Output for **failing** verification:
```
✗ FAIL: Hash mismatch detected
  File: data/sales.csv
  Expected: a3b2c1d4e5f67890abcd...
  Actual:   ff00112233445566aabb...
  File may have been modified or corrupted
```

## Edge Cases & Limitations

### 1. CSV Line Ending Determinism

**Problem**: Converting between CRLF (Windows) and LF (Unix) changes file bytes.

```bash
# Create receipt on Windows (CRLF line endings)
beefcake export data.csv

# Transfer to Linux (Git auto-converts to LF)
git clone repo
cd repo

# Verification FAILS because line endings changed
beefcake verify data.csv.receipt.json
```

**Solutions**:
- Configure Git: `git config core.autocrlf false`
- Use Parquet/JSON instead of CSV (binary formats unaffected)
- Document line-ending expectations in your workflow

### 2. Moved or Renamed Files

Receipts store the **filename** but assume it's in the same directory as the receipt.

**Problem**:
```
before/
  data.csv
  data.csv.receipt.json

after/
  archive/data.csv        ← Moved
  data.csv.receipt.json   ← Receipt still references "data.csv"
```

**Solution**: Move both files together or update the receipt manually.

### 3. Compression & Parquet Metadata

Parquet files may include non-deterministic metadata (creation timestamp, writer version).

**Workaround**: Beefcake's Parquet writer uses consistent settings, but third-party tools may produce different byte-level outputs for identical logical data.

### 4. Large Files (Performance)

SHA-256 hashing is CPU-bound. Expected throughput:

| File Size | Hash Time (approx) |
|-----------|-------------------|
| 10 MB | 0.05s |
| 100 MB | 0.5s |
| 1 GB | 5s |
| 10 GB | 50s |

For multi-GB files, hashing adds measurable overhead. Future optimization: optional parallel chunk hashing.

## Security Considerations

### What Receipts Protect Against

✅ **Accidental modification**: Detects unintentional changes (e.g., editor saved wrong file)
✅ **Basic tampering**: Detects malicious modification by unsophisticated actors
✅ **Data corruption**: Detects bit-rot or incomplete file transfers

### What Receipts DON'T Protect Against

❌ **Sophisticated attackers**: Receipts can be regenerated if attacker has write access
❌ **Receipt tampering**: No cryptographic signing (receipts themselves can be modified)
❌ **Replay attacks**: Receipts don't include nonces or timestamps verification

### Future Enhancements

Potential v2 features:

1. **Digital signatures**: Sign receipts with private key (verify with public key)
2. **Merkle trees**: Verify subsets of large files without rehashing entire file
3. **Timestamping**: Integrate with RFC 3161 timestamp authorities
4. **Multi-algorithm hashing**: Support SHA-3, BLAKE3 alongside SHA-256

## Integration Examples

### Automated Pipeline Verification

```rust
use beefcake::integrity::verify_receipt;

fn validate_before_processing(data_path: &Path) -> anyhow::Result<()> {
    let receipt_path = data_path.with_extension("csv.receipt.json");

    if !receipt_path.exists() {
        anyhow::bail!("No integrity receipt found for {}", data_path.display());
    }

    let result = verify_receipt(&receipt_path)?;

    if !result.passed {
        anyhow::bail!(
            "Integrity check failed: {}",
            result.message
        );
    }

    println!("✓ Integrity verified for {}", data_path.display());
    Ok(())
}
```

### Audit Logging

```rust
use beefcake::integrity::verify_receipt;
use chrono::Utc;

fn audit_verification(receipt_path: &Path) -> anyhow::Result<()> {
    let result = verify_receipt(receipt_path)?;

    let log_entry = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "file": result.file_path,
        "verification_status": if result.passed { "PASS" } else { "FAIL" },
        "expected_hash": result.expected_hash,
        "actual_hash": result.actual_hash,
        "receipt_created": result.receipt.created_utc,
        "producer_version": result.receipt.producer.app_version,
    });

    // Write to audit log
    std::fs::write("audit.log", log_entry.to_string())?;

    Ok(())
}
```

## Best Practices

1. **Always generate receipts for production exports**: Set `create_receipt: true` by default
2. **Verify before downstream processing**: Add verification to pipeline entry points
3. **Store receipts with version control**: Commit `.receipt.json` files alongside data
4. **Document line-ending policy**: Specify CRLF vs LF in your data standards
5. **Test verification in CI/CD**: Add automated tests that verify receipt generation and checking

## Comparison to Alternatives

| Approach | Pros | Cons |
|----------|------|------|
| **Beefcake Receipts** | Simple, no external deps, human-readable JSON | No cryptographic signing, basic protection |
| **PGP/GPG Signatures** | Strong cryptographic guarantees | Requires key management, not self-describing |
| **Blockchain** | Immutable, timestamped, distributed | Overkill for local workflows, complex |
| **Database checksums** | Built into DB engines | Requires database, not portable |
| **Git LFS** | Version control integration | Designed for source code, not data pipelines |

## Troubleshooting

### "Hash mismatch detected"

**Causes**:
1. File was modified after export
2. Line endings changed (CSV files)
3. File corrupted during transfer

**Debug steps**:
```bash
# 1. Check file size matches receipt
ls -l data.csv
cat data.csv.receipt.json | jq '.export.file_size_bytes'

# 2. Manually recompute hash
sha256sum data.csv

# 3. Compare to receipt
cat data.csv.receipt.json | jq '.integrity.hash'
```

### "Data file not found"

**Cause**: File moved or deleted after receipt creation.

**Solution**: Receipts assume file is in same directory. Move both files together.

### "Failed to parse receipt JSON"

**Cause**: Receipt file corrupted or manually edited incorrectly.

**Solution**: Regenerate receipt from original export.

## Reference Implementation

See [`examples/integrity_receipt.rs`](../examples/integrity_receipt.rs) for a complete working example.

## API Documentation

Full API docs available at [docs.rs/beefcake](https://docs.rs/beefcake) under the `integrity` module.

---

**Questions or Issues?** File a ticket at [github.com/yourusername/beefcake/issues](https://github.com/yourusername/beefcake/issues)
