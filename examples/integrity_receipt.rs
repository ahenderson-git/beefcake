//! Example: Export Integrity Receipts
//!
//! This example demonstrates how to:
//! 1. Create an integrity receipt for an exported file
//! 2. Verify file integrity using the receipt
//! 3. Handle verification failures (tampered files)
//!
//! Run with: cargo run --example integrity_receipt

use beefcake::integrity::{create_receipt, save_receipt, verify_receipt};
use polars::prelude::*;
use std::fs;
use tempfile::TempDir;

fn main() -> anyhow::Result<()> {
    println!("=== Beefcake Integrity Receipt Example ===\n");

    // Setup: Create a temporary directory for our example
    let temp_dir = TempDir::new()?;
    let export_path = temp_dir.path().join("sales_data.csv");

    // 1. Create sample dataset
    println!("1. Creating sample dataset...");
    let df = create_sample_dataframe()?;
    println!(
        "   Created DataFrame with {} rows, {} columns",
        df.height(),
        df.width()
    );

    // 2. Export dataset to CSV
    println!("\n2. Exporting dataset to CSV...");
    let mut df_clone = df.clone();
    let mut file = fs::File::create(&export_path)?;
    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut df_clone)?;
    println!("   Exported to: {}", export_path.display());

    // 3. Create integrity receipt
    println!("\n3. Creating integrity receipt...");
    let receipt = create_receipt(&export_path, Some(&df))?;
    println!("   Receipt version: {}", receipt.receipt_version);
    println!("   Hash algorithm: {}", receipt.integrity.hash_algorithm);
    println!("   File hash: {}...", &receipt.integrity.hash[..16]);
    println!("   File size: {} bytes", receipt.export.file_size_bytes);
    println!("   Schema columns: {}", receipt.export.schema.len());

    // 4. Save receipt to disk
    println!("\n4. Saving receipt...");
    let receipt_path = save_receipt(&receipt, &export_path)?;
    println!("   Receipt saved to: {}", receipt_path.display());

    // 5. Verify integrity (should PASS)
    println!("\n5. Verifying file integrity...");
    let result = verify_receipt(&receipt_path)?;
    println!("{}", result.format_cli());

    // 6. Demonstrate tamper detection
    println!("\n6. Simulating file tampering...");
    fs::write(&export_path, b"TAMPERED DATA")?;
    println!("   File has been modified");

    println!("\n7. Verifying tampered file...");
    let tampered_result = verify_receipt(&receipt_path)?;
    println!("{}", tampered_result.format_cli());

    // 8. Show receipt JSON structure
    println!("\n8. Receipt JSON structure:");
    let receipt_json = fs::read_to_string(&receipt_path)?;
    println!("{receipt_json}");

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- Receipts capture complete export metadata + cryptographic hash");
    println!("- Verification is deterministic: same file = same hash");
    println!("- Any modification (even 1 byte) causes verification to fail");
    println!("- Receipts are human-readable JSON for transparency");

    Ok(())
}

/// Create a sample DataFrame for demonstration.
fn create_sample_dataframe() -> PolarsResult<DataFrame> {
    df! {
        "product_id" => [101, 102, 103, 104, 105],
        "product_name" => ["Widget A", "Widget B", "Gadget X", "Gadget Y", "Tool Z"],
        "quantity" => [50, 75, 120, 30, 95],
        "price" => [19.99, 29.99, 45.50, 12.99, 89.99],
        "region" => ["North", "South", "East", "West", "North"],
    }
}
