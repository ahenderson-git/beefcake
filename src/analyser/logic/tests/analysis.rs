use crate::analyser::logic::health::calculate_file_health;
use crate::analyser::logic::profiling;
use crate::analyser::logic::*;
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

    if let ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
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

    if let ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
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
fn test_histogram_streaming_large() -> Result<()> {
    let mut values = Vec::new();
    for i in 0..100_000 {
        values.push(i as f64);
    }
    let s = Series::new("col".into(), values);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let lf = df.lazy();

    // Directly call the streaming histogram function
    let histogram_config = profiling::HistogramConfig {
        min: Some(0.0),
        max: Some(99999.0),
        q1: Some(25000.0),
        q3: Some(75000.0),
        total_count: 100_000,
        null_count: 0,
        custom_sample_size: 10_000,
    };
    let (bin_width, histogram) = profiling::build_histogram_streaming(lf, "col", histogram_config)?;

    assert!(!histogram.is_empty(), "Histogram should not be empty");
    let total_count: usize = histogram.iter().map(|h| h.1).sum();
    // Updated expectation: get_adaptive_sample_size() now caps at 10k for memory efficiency
    assert_eq!(total_count, 10_000);
    assert!(bin_width > 0.0);

    Ok(())
}

#[test]
fn test_interpretation_generation() -> Result<()> {
    let s = Series::new("id".into(), vec!["1", "2", "3"]);
    let df = DataFrame::new(vec![Column::from(s)])?;
    let summaries = analyse_df(&df, 0.0)?;
    let interpretation_text = summaries.first().unwrap().interpretation.join(" ");
    assert!(
        interpretation_text
            .to_lowercase()
            .contains("unique identifier"),
        "Should detect unique identifier. Got: {interpretation_text}"
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
    if let ColumnStats::Boolean(stats) = &summaries.first().unwrap().stats {
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
    if let ColumnStats::Boolean(stats) = &summaries.first().unwrap().stats {
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

    if let ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
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
            stats.bin_width < 25.0,
            "Bin width should be reasonable based on IQR. Got: {}, histogram bins: {}",
            stats.bin_width,
            stats.histogram.len()
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

    if let ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
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
    assert!(
        interp.contains("vast majority"),
        "Should detect dominant bin"
    );
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

    if let ColumnStats::Numeric(stats) = &summaries.first().unwrap().stats {
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
    if let ColumnStats::Categorical(freq) = &summary.stats {
        assert_eq!(freq.get("CARD"), Some(&3));
        assert_eq!(freq.get("CASH"), Some(&2));
    } else {
        panic!("Expected CategoricalStats");
    }
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
fn test_export_tall_dataset() -> Result<()> {
    // Create a dataset with 100,000 rows to verify vectorized stats
    let n = 100_000;
    let s = Series::new("vals".into(), (0..n).map(|i| i as f64).collect::<Vec<_>>());
    let df = DataFrame::new(vec![Column::from(s)])?;

    let summaries = analyse_df(&df, 0.0)?;
    let summary = summaries.first().unwrap();

    if let ColumnStats::Numeric(stats) = &summary.stats {
        assert_eq!(stats.min, Some(0.0));
        assert_eq!(stats.max, Some((n - 1) as f64));
        assert!(stats.mean.unwrap() > 0.0);
        assert_eq!(stats.zero_count, 1);
        assert_eq!(stats.negative_count, 0);
        assert!(stats.is_integer);
    } else {
        panic!("Expected NumericStats");
    }

    Ok(())
}
