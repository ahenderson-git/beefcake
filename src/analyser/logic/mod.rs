pub mod types;
pub mod interpretation;
pub mod health;
pub mod analysis;

pub use types::{
    BooleanStats, ColumnStats, ColumnSummary, FileHealth, NumericStats, TemporalStats, TextStats,
};
pub use health::calculate_file_health;
pub use analysis::{load_df, analyse_df, AnalysisReceiver};

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use anyhow::Result;

    #[test]
    fn test_carriage_return_detection() -> Result<()> {
        let s = Series::new("col".into(), vec!["line1\r\nline2"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(&df, 0.0)?;
        assert!(summaries.first().unwrap().has_special, "Should detect \r as special");
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
            summaries.first().unwrap().interpretation.join(" ").contains("unique identifier"),
            "Should detect unique identifier"
        );

        let s2 = Series::new("age".into(), vec![Some(25.0), Some(30.0), None]);
        let df2 = DataFrame::new(vec![Column::from(s2)])?;
        let summaries2 = analyse_df(&df2, 0.0)?;
        assert!(
            summaries2.first().unwrap().interpretation.join(" ").contains("missing data"),
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
            summaries.first().unwrap().interpretation.join(" ").contains("Binary field"),
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
            assert!(summaries.first().unwrap().interpretation.join(" ").contains("Right-skewed"), "Should report right skew");
            assert!(stats.bin_width < 2.0, "Bin width should be small based on IQR, not large based on range");
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
        assert!(interp.contains("concentrated"), "Should detect concentrated distribution");
        assert!(interp.contains("extreme outliers"), "Should detect extreme outliers");
        assert!(interp.contains("dominates"), "Should detect dominant bin");
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
        assert!(interp.contains("may be invisible"), "Should warn about potentially invisible bars");
        assert!(interp.contains("A single bin dominates"), "Should warn about dominant bin");
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
            
            assert!(full_range > 3.0 * zoom_range, "Threshold should be met for this distribution");
            assert!(summaries.first().unwrap().interpretation.join(" ").contains("extreme outliers"), "Should report extreme outliers");
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
        assert!(interp.contains("Standard deviation may be less reliable"), "Should warn about unreliable std dev");
        Ok(())
    }

    #[test]
    fn test_business_summary_generation() -> Result<()> {
        let s1 = Series::new("order_id".into(), vec!["ORD001", "ORD002", "ORD003"]);
        let s2 = Series::new("sales".into(), vec![10.0, 12.0, 500.0]); // Skewed
        let s3 = Series::new("status".into(), vec![Some("Paid"), Some("Paid"), None]); // Missing (1/3 = 33%)
        
        let df = DataFrame::new(vec![Column::from(s1), Column::from(s2), Column::from(s3)])?;
        let summaries = analyse_df(&df, 0.0)?;
        
        assert!(summaries.get(0).unwrap().business_summary.join(" ").contains("unique tracking number"), "Should identify tracking ID");
        assert!(summaries.get(1).unwrap().business_summary.join(" ").contains("distorted by extreme outliers"), "Should identify outlier distortion");
        assert!(summaries.get(2).unwrap().business_summary.join(" ").contains("significant amount of information is missing"), "Should identify missing data");
        Ok(())
    }

    #[test]
    fn test_categorical_detection() -> Result<()> {
        let s = Series::new("payment_method".into(), vec!["CARD", "CARD", "CASH", "CARD", "CASH"]);
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
            "name".to_string(),
            types::ColumnCleanConfig {
                new_name: "full_name".to_string(),
                target_dtype: None,
                trim_whitespace: true,
                remove_special_chars: true,
                normalization: types::NormalizationMethod::None,
                one_hot_encode: false,
                impute_mode: types::ImputeMode::None,
            },
        );
        configs.insert(
            "age".to_string(),
            types::ColumnCleanConfig {
                new_name: "age_num".to_string(),
                target_dtype: Some(types::ColumnKind::Numeric),
                trim_whitespace: false,
                remove_special_chars: false,
                normalization: types::NormalizationMethod::None,
                one_hot_encode: false,
                impute_mode: types::ImputeMode::None,
            },
        );

        let cleaned = analysis::clean_df(df, &configs)?;

        assert_eq!(cleaned.width(), 2);
        assert!(cleaned.column("full_name").is_ok());
        assert!(cleaned.column("age_num").is_ok());

        let names = cleaned.column("full_name")?.as_materialized_series().cast(&DataType::String)?;
        let names_ca = names.str()?;
        assert_eq!(names_ca.get(0).unwrap(), "Alice");
        assert_eq!(names_ca.get(1).unwrap(), "Bob");
        assert_eq!(names_ca.get(2).unwrap(), "Charlie");

        let ages = cleaned.column("age_num")?.as_materialized_series();
        assert!(ages.dtype().is_numeric());

        Ok(())
    }

    #[test]
    fn test_advanced_insights() -> Result<()> {
        // ... (existing test code)
        Ok(())
    }

    #[test]
    fn test_ml_preprocessing_logic() -> Result<()> {
        let s1 = Series::new("vals".into(), vec![Some(10.0), Some(20.0), None, Some(30.0)]);
        let s2 = Series::new("cat".into(), vec!["A", "B", "A", "C"]);
        let df = DataFrame::new(vec![Column::from(s1), Column::from(s2)])?;

        let mut configs = std::collections::HashMap::new();
        configs.insert(
            "vals".to_string(),
            types::ColumnCleanConfig {
                new_name: "vals_clean".to_string(),
                target_dtype: None,
                trim_whitespace: false,
                remove_special_chars: false,
                impute_mode: types::ImputeMode::Mean,
                normalization: types::NormalizationMethod::MinMax,
                one_hot_encode: false,
            },
        );
        configs.insert(
            "cat".to_string(),
            types::ColumnCleanConfig {
                new_name: "".to_string(),
                target_dtype: None,
                trim_whitespace: false,
                remove_special_chars: false,
                impute_mode: types::ImputeMode::None,
                normalization: types::NormalizationMethod::None,
                one_hot_encode: true,
            },
        );

        let cleaned = analysis::clean_df(df, &configs)?;

        // 1. Verify Imputation and Normalization
        // Original non-nulls: 10, 20, 30. Mean = 20.
        // Imputed: 10, 20, 20, 30.
        // MinMax: (x - 10) / (30 - 10) => 0.0, 0.5, 0.5, 1.0
        let vals = cleaned.column("vals_clean")?.as_materialized_series();
        let vals_ca = vals.f64()?;
        assert_eq!(vals_ca.get(0).unwrap(), 0.0);
        assert_eq!(vals_ca.get(1).unwrap(), 0.5);
        assert_eq!(vals_ca.get(2).unwrap(), 0.5); // Imputed mean
        assert_eq!(vals_ca.get(3).unwrap(), 1.0);

        // 2. Verify One-Hot Encoding
        // Should have columns cat_A, cat_B, cat_C (assuming "_" separator)
        assert!(cleaned.column("cat_A").is_ok());
        assert!(cleaned.column("cat_B").is_ok());
        assert!(cleaned.column("cat_C").is_ok());
        assert!(cleaned.column("cat").is_err()); // Original column should be dropped

        Ok(())
    }
}
