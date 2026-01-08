#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    clippy::indexing_slicing
)]
use super::*;
use anyhow::Result;
use polars::prelude::*;

#[test]
fn test_carriage_return_detection() -> Result<()> {
    let s = Series::new("col".into(), vec!["line1\r\nline2"]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;
    assert!(
        summaries.first().expect("Summary exists").has_special,
        "Should detect \r as special"
    );
    Ok(())
}

#[test]
fn test_normal_whitespace_not_special() -> Result<()> {
    let s = Series::new("col".into(), vec!["line1\nline2\twith spaces"]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;
    assert!(
        !summaries.first().unwrap().has_special,
        "Standard whitespace (\\n, \\t, space) should NOT be special"
    );
    Ok(())
}

#[test]
fn test_histogram_calculation() -> Result<()> {
    let s = Series::new("col".into(), vec![1.0, 1.0, 2.0, 3.0, 10.0]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    if let types::ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
        assert!(!stats.histogram.is_empty(), "Histogram should not be empty");
        let total_count: usize = stats.histogram.iter().map(|h| h.1).sum();
        assert_eq!(total_count, 5);
        assert!(stats.std_dev.is_some(), "Should compute std_dev");
    } else {
        panic!("Expected NumericStats");
    }
    Ok(())
}

#[test]
fn test_histogram_single_value() -> Result<()> {
    let s = Series::new("col".into(), vec![2.0, 2.0, 2.0]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    if let types::ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
        assert_eq!(
            stats.histogram.len(),
            20,
            "Should have 20 bins even for single value"
        );
        let total_count: usize = stats.histogram.iter().map(|h| h.1).sum();
        assert_eq!(total_count, 3);
        assert_eq!(
            stats.histogram[10].1, 3,
            "Middle bin should have all counts"
        );
    } else {
        panic!("Expected NumericStats");
    }
    Ok(())
}

#[test]
fn test_interpretation_generation() -> Result<()> {
    let s = Series::new("id".into(), vec!["1", "2", "3"]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;
    assert!(
        summaries
            .first()
            .unwrap()
            .interpretation
            .join(" ")
            .contains("unique identifier"),
        "Should detect unique identifier"
    );

    let s2 = Series::new("age".into(), vec![Some(25.0), Some(30.0), None]);
    let df2 = DataFrame::new(vec![Column::from(s2)])?;
    let summaries2 = analyse_df(&df2, 0.0)?;
    assert!(
        summaries2
            .first()
            .unwrap()
            .interpretation
            .join(" ")
            .contains("missing data"),
        "Should detect nulls"
    );

    Ok(())
}

#[test]
fn test_boolean_detection() -> Result<()> {
    let s = Series::new(
        "bool_col".into(),
        vec![Some(true), Some(false), None, Some(true)],
    );
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    assert_eq!(summaries.first().unwrap().kind.as_str(), "Boolean");
    if let types::ColumnStats::Boolean(stats) = &summaries.first().unwrap().stats {
        assert_eq!(stats.true_count, 2);
        assert_eq!(stats.false_count, 1);
    } else {
        panic!("Expected BooleanStats");
    }
    assert!(
        summaries
            .first()
            .unwrap()
            .interpretation
            .join(" ")
            .contains("Binary field"),
        "Should detect binary signal"
    );
    Ok(())
}

#[test]
fn test_effective_boolean_detection() -> Result<()> {
    let s = Series::new("is_active".into(), vec![Some(1), Some(0), None, Some(1)]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    assert_eq!(summaries.first().unwrap().kind.as_str(), "Boolean");
    if let types::ColumnStats::Boolean(stats) = &summaries.first().unwrap().stats {
        assert_eq!(stats.true_count, 2);
        assert_eq!(stats.false_count, 1);
    } else {
        panic!("Expected BooleanStats for 0/1 numeric column");
    }
    Ok(())
}

#[test]
fn test_skewed_data_histogram() -> Result<()> {
    let mut vals = vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.2, 2.5, 3.0, 3.5, 4.0]; // Central mass
    vals.push(1000.0); // Extreme outlier
    let s = Series::new("col".into(), vals);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    if let types::ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
        assert!(stats.skew.unwrap() > 0.1, "Should detect right skew");
        assert!(
            summaries
                .first()
                .unwrap()
                .interpretation
                .join(" ")
                .contains("Right-skewed"),
            "Should report right skew"
        );
        assert!(
            stats.bin_width < 2.0,
            "Bin width should be small based on IQR, not large based on range"
        );
        assert!(stats.histogram.len() > 2, "Should have more than 2 bins");
    } else {
        panic!("Expected NumericStats");
    }
    Ok(())
}

#[test]
fn test_trimmed_mean() -> Result<()> {
    let s = Series::new("col".into(), vec![0.0, 10.0, 20.0, 30.0, 100.0]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.2)?;

    if let types::ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
        assert_eq!(stats.mean, Some(32.0));
        assert_eq!(stats.trimmed_mean, Some(20.0));
    } else {
        panic!("Expected NumericStats");
    }
    Ok(())
}

#[test]
fn test_interpretation_histogram_signals() -> Result<()> {
    let mut vals = vec![10.0; 95];
    vals.extend(vec![1000.0, 2000.0, 3000.0, 4000.0, 5000.0]);
    let s = Series::new("col".into(), vals);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    let interp = summaries.first().unwrap().interpretation.join(" ");
    assert!(
        interp.contains("concentrated"),
        "Should detect concentrated distribution"
    );
    assert!(
        interp.contains("Extreme outliers"),
        "Should detect extreme outliers"
    );
    assert!(interp.contains("vast majority"), "Should detect dominant bin");
    Ok(())
}

#[test]
fn test_user_reported_invisible_bars_scenario() -> Result<()> {
    let mut vals = vec![0.0; 1000];
    vals.push(100.0);
    vals.push(200.0);
    let s = Series::new("col".into(), vals);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    let interp = summaries.first().unwrap().interpretation.join(" ");
    assert!(
        interp.contains("Significant scale differences"),
        "Should warn about potentially invisible bars"
    );
    assert!(
        interp.contains("single value range contains the vast majority"),
        "Should warn about dominant bin"
    );
    Ok(())
}

#[test]
fn test_user_reported_delivery_minutes_zoom() -> Result<()> {
    let mut vals: Vec<f64> = (0..100).map(|i| i as f64).collect();
    vals.push(5000.0); // Outlier
    let s = Series::new("delivery_minutes".into(), vals);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    if let types::ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
        let min = stats.min.unwrap();
        let max = stats.max.unwrap();
        let p05 = stats.p05.unwrap();
        let p95 = stats.p95.unwrap();

        let full_range = max - min;
        let zoom_range = p95 - p05;

        assert!(
            full_range > 3.0 * zoom_range,
            "Threshold should be met for this distribution"
        );
        assert!(
            summaries
                .first()
                .unwrap()
                .interpretation
                .join(" ")
                .contains("Extreme outliers"),
            "Should report extreme outliers"
        );
    }
    Ok(())
}

#[test]
fn test_std_dev_reliability_signal() -> Result<()> {
    let mut vals = vec![10.0, 11.0, 12.0, 13.0, 14.0];
    vals.push(1000.0); // One massive outlier
    let s = Series::new("col".into(), vals);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    let interp = summaries.first().unwrap().interpretation.join(" ");
    assert!(
        interp.contains("Standard deviation may be less reliable"),
        "Should warn about unreliable std dev"
    );
    Ok(())
}

#[test]
fn test_business_summary_generation() -> Result<()> {
    let s1 = Series::new("order_id".into(), vec!["ORD001", "ORD002", "ORD003"]);
    let s2 = Series::new("sales".into(), vec![10.0, 12.0, 500.0]); // Skewed
    let s3 = Series::new("status".into(), vec![Some("Paid"), Some("Paid"), None]); // Missing (1/3 = 33%)

    let df = DataFrame::new(vec![Column::from(s1), Column::from(s2), Column::from(s3)])?;
    let summaries = analyse_df(&df, 0.0)?;

    assert!(
        summaries
            .first()
            .unwrap()
            .business_summary
            .join(" ")
            .contains("unique tracking number"),
        "Should identify tracking ID"
    );
    assert!(
        summaries[1]
            .business_summary
            .join(" ")
            .contains("distorted by extreme outliers"),
        "Should identify outlier distortion"
    );
    assert!(
        summaries[2]
            .business_summary
            .join(" ")
            .contains("significant amount of information is missing"),
        "Should identify missing data"
    );
    Ok(())
}

#[test]
fn test_categorical_detection() -> Result<()> {
    let s = Series::new(
        "payment_method".into(),
        vec!["CARD", "CARD", "CASH", "CARD", "CASH"],
    );
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;

    let summary = summaries.first().unwrap();
    assert_eq!(summary.kind.as_str(), "Categorical");
    if let types::ColumnStats::Categorical(freq) = &summary.stats {
        assert_eq!(freq.get("CARD"), Some(&3));
        assert_eq!(freq.get("CASH"), Some(&2));
    } else {
        panic!("Expected CategoricalStats");
    }
    Ok(())
}

#[test]
fn test_clean_df_logic() -> Result<()> {
    let s1 = Series::new("name".into(), vec!["  Alice  ", "Bob\r", "Charlie\t"]);
    let s2 = Series::new("age".into(), vec!["25", "30", "35"]);
    let df = DataFrame::new(vec![Column::from(s1), Column::from(s2)])?;

    let mut configs = std::collections::HashMap::new();
    configs.insert(
        "name".to_owned(),
        types::ColumnCleanConfig {
            new_name: "full_name".to_owned(),
            target_dtype: None,
            active: true,
            advanced_cleaning: true,
            ml_preprocessing: false,
            trim_whitespace: true,
            remove_special_chars: true,
            normalization: types::NormalizationMethod::None,
            one_hot_encode: false,
            impute_mode: types::ImputeMode::None,
            ..Default::default()
        },
    );
    configs.insert(
        "age".to_owned(),
        types::ColumnCleanConfig {
            new_name: "age_num".to_owned(),
            target_dtype: Some(types::ColumnKind::Numeric),
            active: true,
            advanced_cleaning: false,
            ml_preprocessing: false,
            trim_whitespace: false,
            remove_special_chars: false,
            normalization: types::NormalizationMethod::None,
            one_hot_encode: false,
            impute_mode: types::ImputeMode::None,
            ..Default::default()
        },
    );

    let cleaned = analysis::clean_df(df, &configs, false)?;

    assert_eq!(cleaned.width(), 2);
    assert!(cleaned.column("full_name").is_ok());
    assert!(cleaned.column("age_num").is_ok());

    let names = cleaned
        .column("full_name")?
        .as_materialized_series()
        .cast(&DataType::String)?;
    let names_ca = names.str()?;
    assert_eq!(names_ca.get(0).unwrap(), "Alice");
    assert_eq!(names_ca.get(1).unwrap(), "Bob");
    assert_eq!(names_ca.get(2).unwrap(), "Charlie");

    let ages = cleaned.column("age_num")?.as_materialized_series();
    assert!(ages.dtype().is_numeric());

    Ok(())
}

#[test]
fn test_ml_training_linear_regression() -> Result<()> {
    // Create simple linear data: y = 2x + 1
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = Series::new("y".into(), vec![3.0, 5.0, 7.0, 9.0, 11.0]);
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let results = ml::train_model(&df, "y", types::MlModelKind::LinearRegression, &progress)?;

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

    let results = ml::train_model(&df, "y", types::MlModelKind::LogisticRegression, &progress)?;

    assert!(results.accuracy.unwrap() > 0.9);

    Ok(())
}

#[test]
fn test_ml_training_single_class_error() -> Result<()> {
    let x = Series::new("x".into(), vec![1.0, 2.0, 3.0]);
    let y = Series::new("y".into(), vec![1, 1, 1]); // Only one class
    let df = DataFrame::new(vec![Column::from(x), Column::from(y)])?;
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let result = ml::train_model(&df, "y", types::MlModelKind::LogisticRegression, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("must have at least two distinct classes"));

    let result_tree = ml::train_model(&df, "y", types::MlModelKind::DecisionTree, &progress);
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

    let result = ml::train_model(&df, "y", types::MlModelKind::LogisticRegression, &progress);
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

    let results = ml::train_model(&df, "y", types::MlModelKind::LinearRegression, &progress)?;

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
#[expect(clippy::too_many_lines)]
fn test_ml_advice_auto_config() {
    use super::types::{ColumnCleanConfig, NormalizationMethod};

    // Case 1: Skewed data -> Clip Outliers
    let mut summary = ColumnSummary {
        name: "skewed".to_owned(),
        kind: types::ColumnKind::Numeric,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: types::ColumnStats::Numeric(types::NumericStats {
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
        kind: types::ColumnKind::Numeric,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: types::ColumnStats::Numeric(types::NumericStats {
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
        config2.normalization,
        NormalizationMethod::ZScore,
        "Z-Score should be auto-enabled for numeric features"
    );

    // Case 3: Missing data -> Mean Imputation
    let mut summary3 = ColumnSummary {
        name: "missing".to_owned(),
        kind: types::ColumnKind::Numeric,
        count: 100,
        nulls: 10,
        has_special: false,
        stats: types::ColumnStats::Numeric(Default::default()),
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
        types::ImputeMode::Mean,
        "Mean imputation should be auto-enabled for numeric missing data"
    );

    // Case 4: Categorical data -> One-Hot Encoding
    let mut summary4 = ColumnSummary {
        name: "category".to_owned(),
        kind: types::ColumnKind::Categorical,
        count: 100,
        nulls: 0,
        has_special: false,
        stats: types::ColumnStats::Categorical(std::collections::HashMap::new()),
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
        kind: types::ColumnKind::Text,
        count: 100,
        nulls: 0,
        has_special: true,
        stats: types::ColumnStats::Text(Default::default()),
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

#[test]
fn test_ml_preprocessing_logic() -> Result<()> {
    let s1 = Series::new(
        "vals".into(),
        vec![Some(10.0), Some(20.0), None, Some(30.0)],
    );
    let s2 = Series::new("cat".into(), vec!["A", "B", "A", "C"]);
    let df = DataFrame::new(vec![Column::from(s1), Column::from(s2)])?;

    let mut configs = std::collections::HashMap::new();
    configs.insert(
        "vals".to_owned(),
        types::ColumnCleanConfig {
            new_name: "vals_clean".to_owned(),
            target_dtype: None,
            active: true,
            advanced_cleaning: false,
            ml_preprocessing: true,
            trim_whitespace: false,
            remove_special_chars: false,
            impute_mode: types::ImputeMode::Mean,
            normalization: types::NormalizationMethod::MinMax,
            one_hot_encode: false,
            ..Default::default()
        },
    );
    configs.insert(
        "cat".to_owned(),
        types::ColumnCleanConfig {
            new_name: String::new(),
            target_dtype: None,
            active: true,
            advanced_cleaning: false,
            ml_preprocessing: true,
            trim_whitespace: false,
            remove_special_chars: false,
            impute_mode: types::ImputeMode::None,
            normalization: types::NormalizationMethod::None,
            one_hot_encode: true,
            ..Default::default()
        },
    );

    let cleaned = analysis::clean_df(df, &configs, false)?;

    // 1. Verify Imputation and Normalization
    // Original non-nulls: 10, 20, 30. Mean = 20.
    // Imputed: 10, 20, 20, 30.
    // MinMax: (x - 10) / (30 - 10) => 0.0, 0.5, 0.5, 1.0
    let vals = cleaned.column("vals_clean")?.as_materialized_series();
    let vals_ca = vals.f64()?;
    assert_eq!(vals_ca.get(0), Some(0.0));
    assert_eq!(vals_ca.get(1), Some(0.5));
    assert_eq!(vals_ca.get(2), Some(0.5)); // Imputed mean
    assert_eq!(vals_ca.get(3), Some(1.0));

    // 2. Verify One-Hot Encoding
    // Should have columns cat_A, cat_B, cat_C (assuming "_" separator)
    assert!(cleaned.column("cat_A").is_ok());
    assert!(cleaned.column("cat_B").is_ok());
    assert!(cleaned.column("cat_C").is_ok());
    assert!(cleaned.column("cat").is_err()); // Original column should be dropped

    Ok(())
}

#[test]
fn test_column_deactivation() -> Result<()> {
    let s1 = Series::new("keep".into(), vec![1, 2, 3]);
    let s2 = Series::new("drop".into(), vec![4, 5, 6]);
    let df = DataFrame::new(vec![Column::from(s1), Column::from(s2)])?;

    let mut configs = std::collections::HashMap::new();
    configs.insert(
        "keep".to_owned(),
        types::ColumnCleanConfig {
            active: true,
            ..Default::default()
        },
    );
    configs.insert(
        "drop".to_owned(),
        types::ColumnCleanConfig {
            active: false,
            ..Default::default()
        },
    );

    let cleaned = analysis::clean_df(df, &configs, false)?;
    assert_eq!(cleaned.width(), 1);
    assert!(cleaned.column("keep").is_ok());
    assert!(cleaned.column("drop").is_err());

    Ok(())
}

#[test]
fn test_health_score_range() -> Result<()> {
    let s1 = Series::new("col".into(), vec![1.0, 2.0, 3.0]);
    let df = DataFrame::new(vec![Column::from(s1)])?;
    let summaries = analyse_df(&df, 0.0)?;
    let health = calculate_file_health(&summaries);

    // Perfect health should be 1.0 (100%)
    assert!(
        health.score <= 1.0,
        "Score should not exceed 1.0, got {}",
        health.score
    );
    assert_eq!(health.score, 1.0);

    // Now create a summary with issues
    let mut summaries_bad = summaries.clone();
    if let Some(s) = summaries_bad.first_mut() {
        s.nulls = 100; // 100% nulls
        s.count = 100;
    }

    let health_bad = calculate_file_health(&summaries_bad);
    assert!(health_bad.score < 1.0, "Score should be reduced");
    assert!(health_bad.score >= 0.0, "Score should not be negative");

    Ok(())
}

#[test]
fn test_correlation_matrix() -> Result<()> {
    let s1 = Series::new("a".into(), vec![1.0, 2.0, 3.0]);
    let s2 = Series::new("b".into(), vec![2.0, 4.0, 6.0]); // Correlation 1.0
    let s3 = Series::new("c".into(), vec![3.0, 2.0, 1.0]); // Correlation -1.0
    let s4 = Series::new("d".into(), vec!["x", "y", "z"]); // Non-numeric

    let df = DataFrame::new(vec![
        Column::from(s1),
        Column::from(s2),
        Column::from(s3),
        Column::from(s4),
    ])?;

    let matrix_opt = calculate_correlation_matrix(&df)?;
    assert!(matrix_opt.is_some());
    let matrix = matrix_opt.unwrap();

    assert_eq!(matrix.columns, vec!["a", "b", "c"]);
    assert_eq!(matrix.data.len(), 3);

    // a vs b should be ~1.0
    assert!((matrix.data[0][1] - 1.0).abs() < 1e-6);
    // a vs c should be ~-1.0
    assert!((matrix.data[0][2] - (-1.0)).abs() < 1e-6);
    // diagonal should be 1.0
    assert_eq!(matrix.data[0][0], 1.0);
    assert_eq!(matrix.data[1][1], 1.0);
    assert_eq!(matrix.data[2][2], 1.0);

    Ok(())
}

#[test]
fn test_restricted_cleaning() -> Result<()> {
    use crate::analyser::logic::analysis;
    use crate::analyser::logic::types;
    use polars::prelude::*;
    use std::collections::HashMap;

    let df = df!(
        "text" => &["  hello  ", "world\r", "ascii©", "n/a", "$1,000"],
        "to_cast" => &["true", "false", "true", "false", "true"],
        "to_skip" => &["KeepCase", "RegexMe", "ImputeMe", "RoundMe", "NormMe"]
    )?;

    let mut configs = HashMap::new();

    // Config for "text" column - should apply all included functions
    configs.insert(
        "text".to_string(),
        types::ColumnCleanConfig {
            advanced_cleaning: true,
            trim_whitespace: true,
            remove_special_chars: true,
            remove_non_ascii: true,
            standardize_nulls: true,
            extract_numbers: true,
            // These should be skipped in restricted mode
            text_case: types::TextCase::Lowercase,
            regex_find: "hello".to_string(),
            regex_replace: "hi".to_string(),
            ..Default::default()
        },
    );

    // Config for "to_cast" column - should cast to boolean
    configs.insert(
        "to_cast".to_string(),
        types::ColumnCleanConfig {
            target_dtype: Some(types::ColumnKind::Boolean),
            ..Default::default()
        },
    );

    // Config for "to_skip" column - should skip all these
    configs.insert(
        "to_skip".to_string(),
        types::ColumnCleanConfig {
            new_name: "renamed".to_string(),      // Should be skipped
            impute_mode: types::ImputeMode::Zero, // Should be skipped
            rounding: Some(0),                    // Should be skipped
            normalization: types::NormalizationMethod::MinMax, // Should be skipped
            one_hot_encode: true,                 // Should be skipped
            ..Default::default()
        },
    );

    let cleaned = analysis::clean_df(df, &configs, true)?;

    // Verify "text" column
    let text_col = cleaned.column("text")?.as_materialized_series();
    let text_ca = text_col.str()?;
    // 1. "  hello  " -> "hello" (trimmed, regex skipped)
    assert_eq!(text_ca.get(0), Some("hello"));
    // 2. "world\r" -> "world" (special char removed)
    assert_eq!(text_ca.get(1), Some("world"));
    // 3. "ascii©" -> "ascii" (non-ascii removed)
    assert_eq!(text_ca.get(2), Some("ascii"));
    // 4. "n/a" -> null (standardized null)
    assert_eq!(text_ca.get(3), None);
    // 5. "$1,000" -> "1000" (extract numbers)
    assert_eq!(text_ca.get(4), Some("1000"));

    // Verify "to_cast" column
    let cast_col = cleaned.column("to_cast")?;
    assert_eq!(*cast_col.dtype(), DataType::Boolean);

    // Verify "to_skip" column
    assert!(cleaned.column("renamed").is_err()); // Rename skipped
    assert!(cleaned.column("to_skip").is_ok()); // Original name kept

    // Verify one-hot skipped (no new columns added)
    assert_eq!(cleaned.width(), 3);

    Ok(())
}
