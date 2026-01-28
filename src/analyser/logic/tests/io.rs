use crate::analyser::logic::*;
use crate::analyser::logic::{clean_df, save_df};
use anyhow::Result;
use polars::prelude::*;

#[test]
fn test_save_df_formats() -> Result<()> {
    let mut df = df!(
        "a" => &[1, 2, 3],
        "b" => &["x", "y", "z"]
    )?;

    let temp_dir = std::env::temp_dir();

    // Test CSV
    let csv_path = temp_dir.join("test_export.csv");
    save_df(&mut df, &csv_path)?;
    assert!(csv_path.exists());
    let _ = std::fs::remove_file(csv_path);

    // Test Parquet
    let parquet_path = temp_dir.join("test_export.parquet");
    save_df(&mut df, &parquet_path)?;
    assert!(parquet_path.exists());
    let _ = std::fs::remove_file(parquet_path);

    // Test JSON
    let json_path = temp_dir.join("test_export.json");
    save_df(&mut df, &json_path)?;
    assert!(json_path.exists());
    let _ = std::fs::remove_file(json_path);

    Ok(())
}

#[test]
fn test_export_with_excluded_columns() -> Result<()> {
    let df = df!(
        "keep" => &[1, 2, 3],
        "drop" => &[4, 5, 6]
    )?;

    let mut configs = std::collections::HashMap::new();
    configs.insert(
        "keep".to_owned(),
        ColumnCleanConfig {
            active: true,
            ..Default::default()
        },
    );
    configs.insert(
        "drop".to_owned(),
        ColumnCleanConfig {
            active: false,
            ..Default::default()
        },
    );

    let mut cleaned = clean_df(df, &configs, false)?;
    assert_eq!(cleaned.width(), 1);

    let temp_dir = std::env::temp_dir();
    let parquet_path = temp_dir.join("test_exclude_export.parquet");
    save_df(&mut cleaned, &parquet_path)?;
    assert!(parquet_path.exists());
    let _ = std::fs::remove_file(parquet_path);

    Ok(())
}

#[test]
fn test_export_massive_columns() -> Result<()> {
    let num_cols = 200; // Large number of columns
    let num_rows = 100;

    let mut columns = Vec::new();
    for i in 0..num_cols {
        let s = Series::new(
            format!("col_{i}").into(),
            vec![Some(format!(" {i} ")); num_rows],
        );
        columns.push(Column::from(s));
    }
    let df = DataFrame::new(columns)?;

    let mut configs = std::collections::HashMap::new();
    for i in 0..num_cols {
        configs.insert(
            format!("col_{i}"),
            ColumnCleanConfig {
                active: true,
                target_dtype: Some(ColumnKind::Numeric),
                normalisation: NormalisationMethod::ZScore,
                impute_mode: ImputeMode::Mean,
                advanced_cleaning: true,
                trim_whitespace: true,
                ..Default::default()
            },
        );
    }

    // This should not overflow the stack because of our batching optimisation
    let mut cleaned = clean_df(df, &configs, false)?;
    assert_eq!(cleaned.width(), num_cols);

    let temp_dir = std::env::temp_dir();
    let parquet_path = temp_dir.join("test_massive_export.parquet");
    save_df(&mut cleaned, &parquet_path)?;
    assert!(parquet_path.exists());
    let _ = std::fs::remove_file(parquet_path);

    Ok(())
}

#[test]
fn test_export_super_massive_columns() -> Result<()> {
    let num_cols = 1000; // Even larger
    let num_rows = 10; // Keep rows small to focus on plan complexity

    let mut columns = Vec::new();
    for i in 0..num_cols {
        let s = Series::new(
            format!("col_{i}").into(),
            vec![Some(format!(" {i} ")); num_rows],
        );
        columns.push(Column::from(s));
    }
    let df = DataFrame::new(columns)?;

    let mut configs = std::collections::HashMap::new();
    for i in 0..num_cols {
        configs.insert(
            format!("col_{i}"),
            ColumnCleanConfig {
                active: true,
                target_dtype: Some(ColumnKind::Numeric),
                normalisation: NormalisationMethod::ZScore,
                impute_mode: ImputeMode::Mean,
                advanced_cleaning: true,
                trim_whitespace: true,
                remove_special_chars: true,
                standardise_nulls: true,
                ml_preprocessing: true,
                ..Default::default()
            },
        );
    }

    // This might crash the test runner if the stack is small
    let cleaned = clean_df(df, &configs, false)?;
    assert_eq!(cleaned.width(), num_cols);

    Ok(())
}
