use crate::analyser::logic::ml;
use crate::analyser::logic::*;
use anyhow::Result;
use polars::prelude::*;

#[test]
fn test_ml_training_linear_regression() -> Result<()> {
    // Create simple linear data: y = 2x + 1
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = Series::new("y".into(), vec![3.0, 5.0, 7.0, 9.0, 11.0]);
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let results = ml::train_model(&df, "y", MlModelKind::LinearRegression, &progress)?;

    assert!(results.r2_score.unwrap() > 0.99);
    let coeffs = results.coefficients.unwrap();
    assert!((&coeffs["x"] - 2.0).abs() < 1e-6);
    assert!((results.intercept.unwrap() - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_ml_training_logistic_regression() -> Result<()> {
    // Create simple classification data
    let x = Series::new("x".into(), vec![1.0, 2.0, 10.0, 11.0]);
    let y = Series::new("y".into(), vec![0, 0, 1, 1]); // Binary target
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let results = ml::train_model(&df, "y", MlModelKind::LogisticRegression, &progress)?;

    assert!(results.accuracy.unwrap() > 0.9);

    Ok(())
}

#[test]
fn test_ml_training_single_class_error() -> Result<()> {
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0]);
    let y = Series::new("y".into(), vec![1, 1, 1]); // Only one class
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let result = ml::train_model(&df, "y", MlModelKind::LogisticRegression, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("must have at least two distinct classes"));

    let result_tree = ml::train_model(&df, "y", MlModelKind::DecisionTree, &progress);
    assert!(result_tree.is_err());
    assert!(
        result_tree
            .unwrap_err()
            .to_string()
            .contains("at least two distinct classes")
    );

    Ok(())
}

#[test]
fn test_ml_training_null_target_class_error() -> Result<()> {
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0]);
    let y = Series::new("y".into(), vec![Some(1), None, Some(1)]); // 2 unique values (1, null) but only 1 class
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let result = ml::train_model(&df, "y", MlModelKind::LogisticRegression, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    // It should be our custom error message, not linfa's
    assert!(
        err.contains("must have at least two distinct classes"),
        "Error was: {err}"
    );

    Ok(())
}

#[test]
fn test_ml_interpretation() -> Result<()> {
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = Series::new("y".into(), vec![3.0, 5.0, 7.0, 9.0, 11.0]);
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let results = ml::train_model(&df, "y", MlModelKind::LinearRegression, &progress)?;

    assert!(!results.interpretation.is_empty());
    let joined = results.interpretation.join(" ");
    assert!(joined.contains("Strong predictive model"));
    assert!(joined.contains("Primary Driver"));

    Ok(())
}

#[test]
fn test_ml_advice_generation() -> Result<()> {
    let s1 = Series::new("numeric".into(), vec![1.0, 2.0, 1.0]); // Not unique
    let s2 = Series::new("categorical".into(), vec!["A", "B", "A"]);
    let s3 = Series::new("id".into(), vec!["ID1", "ID2", "ID3"]);
    let s4 = Series::new("order_id".into(), vec![101, 102, 103]); // Numeric ID
    let df = DataFrame::new(vec![
        Column::from(s1),
        Column::from(s2),
        Column::from(s3),
        Column::from(s4),
    ])?;

    let summaries = analyse_df(&df, 0.0)?;

    // 1. Numeric advice (non-ID)
    let num_summary = summaries.iter().find(|c| c.name == "numeric").unwrap();
    assert!(
        num_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("Linear Regression"))
    );

    // 2. Categorical advice
    let cat_summary = summaries.iter().find(|c| c.name == "categorical").unwrap();
    assert!(
        cat_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("One-Hot encoding"))
    );

    // 3. Text ID advice
    let id_summary = summaries.iter().find(|c| c.name == "id").unwrap();
    assert!(
        id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("excluding this identifier"))
    );

    // 4. Numeric ID advice
    let order_id_summary = summaries.iter().find(|c| c.name == "order_id").unwrap();
    assert!(
        order_id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("excluding this identifier"))
    );
    assert!(
        order_id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("Likely an identifier column"))
    );

    // 5. Numeric ID with repetitions (Regression reproduction)
    let s5 = Series::new("customer_id".into(), vec![1, 2, 1, 2, 3]); // 60% uniqueness
    let df5 = DataFrame::new(vec![Column::from(s5)])?;
    let summaries5 = analyse_df(&df5, 0.0)?;
    let customer_id_summary = summaries5.iter().find(|c| c.name == "customer_id").unwrap();
    assert!(
        customer_id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("excluding this identifier")),
        "customer_id with repetitions should be flagged as ID"
    );
    assert!(
        customer_id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("Warning: Numeric IDs should NOT be used")),
        "should warn about numeric IDs in linear models"
    );
    assert!(
        !customer_id_summary
            .ml_advice
            .iter()
            .any(|a| a.contains("Suitable for Linear Regression")),
        "should NOT suggest Linear Regression for numeric IDs"
    );

    Ok(())
}

#[test]
fn test_ml_advice_auto_config() {
    // Case 1: Skewed data -> Clip Outliers
    let mut summary = ColumnSummary {
        name: "skewed".to_owned(),
        standardised_name: "skewed".to_owned(),
        kind: ColumnKind::Numeric,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: ColumnStats::Numeric(NumericStats {
            skew: Some(2.5), // High skew
            ..Default::default()
        }),
        interpretation: vec![],
        business_summary: vec![],
        ml_advice: vec![],
        samples: vec![],
    };
    summary.ml_advice = summary.generate_ml_advice();
    assert!(
        summary
            .ml_advice
            .iter()
            .any(|a| a.contains("Outlier Clipping"))
    );

    let mut config = ColumnCleanConfig::default();
    summary.apply_advice_to_config(&mut config);
    assert!(
        config.clip_outliers,
        "Clip outliers should be auto-enabled for skewed data"
    );

    // Case 2: Numeric features -> Z-Score Normalization
    let mut summary2 = ColumnSummary {
        name: "feature".to_owned(),
        standardised_name: "feature".to_owned(),
        kind: ColumnKind::Numeric,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: ColumnStats::Numeric(NumericStats {
            skew: Some(0.0),
            ..Default::default()
        }),
        interpretation: vec![],
        business_summary: vec![],
        ml_advice: vec![],
        samples: vec![],
    };
    summary2.ml_advice = summary2.generate_ml_advice();
    assert!(
        summary2
            .ml_advice
            .iter()
            .any(|a| a.contains("Normalization"))
    );

    let mut config2 = ColumnCleanConfig::default();
    summary2.apply_advice_to_config(&mut config2);
    assert_eq!(
        config2.normalisation,
        NormalisationMethod::ZScore,
        "Z-Score should be auto-enabled for numeric features"
    );

    // Case 3: Missing data -> Mean Imputation
    let mut summary3 = ColumnSummary {
        name: "missing".to_owned(),
        standardised_name: "missing".to_owned(),
        kind: ColumnKind::Numeric,
        count: 100,
        nulls: 10,
        has_special: false,
        stats: ColumnStats::Numeric(Default::default()),
        interpretation: vec![],
        business_summary: vec![],
        ml_advice: vec![],
        samples: vec![],
    };
    summary3.ml_advice = summary3.generate_ml_advice();
    assert!(
        summary3
            .ml_advice
            .iter()
            .any(|a| a.contains("Mean or Median Imputation"))
    );

    let mut config3 = ColumnCleanConfig::default();
    summary3.apply_advice_to_config(&mut config3);
    assert_eq!(
        config3.impute_mode,
        ImputeMode::Mean,
        "Mean imputation should be auto-enabled for numeric missing data"
    );

    // Case 4: Categorical data -> One-Hot Encoding
    let mut summary4 = ColumnSummary {
        name: "category".to_owned(),
        standardised_name: "category".to_owned(),
        kind: ColumnKind::Categorical,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: ColumnStats::Categorical(std::collections::HashMap::new()),
        interpretation: vec![],
        business_summary: vec![],
        ml_advice: vec![],
        samples: vec![],
    };
    summary4.ml_advice = summary4.generate_ml_advice();
    assert!(
        summary4
            .ml_advice
            .iter()
            .any(|a| a.contains("One-Hot encoding"))
    );

    let mut config4 = ColumnCleanConfig::default();
    summary4.apply_advice_to_config(&mut config4);
    assert!(
        config4.one_hot_encode,
        "One-Hot encoding should be auto-enabled for categorical data"
    );

    // Case 5: Special characters detected -> Remove Special Chars
    let summary5 = ColumnSummary {
        name: "special".to_owned(),
        standardised_name: "special".to_owned(),
        kind: ColumnKind::Text,
        count: 100,
        nulls: 0,
        has_special: true,
        stats: ColumnStats::Text(Default::default()),
        interpretation: vec![],
        business_summary: vec![],
        ml_advice: vec![],
        samples: vec![],
    };
    let mut config5 = ColumnCleanConfig::default();
    summary5.apply_advice_to_config(&mut config5);
    assert!(
        config5.remove_special_chars,
        "Remove special chars should be auto-enabled when special chars are detected"
    );
}
