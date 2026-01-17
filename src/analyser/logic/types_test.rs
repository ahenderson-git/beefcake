//! Unit tests for type detection and column summary logic

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_summary_null_pct() {
        let summary = ColumnSummary {
            name: "test".to_string(),
            standardised_name: "test".to_string(),
            kind: ColumnKind::Numeric,
            count: 100,
            nulls: 20,
            has_special: false,
            stats: ColumnStats::Numeric(NumericStats::default()),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert_eq!(summary.null_pct(), 20.0);
    }

    #[test]
    fn test_column_summary_null_pct_zero_count() {
        let summary = ColumnSummary {
            name: "empty".to_string(),
            standardised_name: "empty".to_string(),
            kind: ColumnKind::Text,
            count: 0,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Text(TextStats::default()),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert_eq!(summary.null_pct(), 0.0);
    }

    #[test]
    fn test_column_summary_uniqueness_ratio() {
        let mut numeric_stats = NumericStats::default();
        numeric_stats.distinct_count = 50;

        let summary = ColumnSummary {
            name: "id".to_string(),
            standardised_name: "id".to_string(),
            kind: ColumnKind::Numeric,
            count: 100,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Numeric(numeric_stats),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert_eq!(summary.uniqueness_ratio(), 0.5);
    }

    #[test]
    fn test_is_compatible_with_same_kind() {
        let summary = ColumnSummary {
            name: "age".to_string(),
            standardised_name: "age".to_string(),
            kind: ColumnKind::Numeric,
            count: 100,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Numeric(NumericStats::default()),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert!(summary.is_compatible_with(ColumnKind::Numeric));
    }

    #[test]
    fn test_column_kind_to_string() {
        assert_eq!(ColumnKind::Numeric.to_string(), "Numeric");
        assert_eq!(ColumnKind::Text.to_string(), "Text");
        assert_eq!(ColumnKind::Boolean.to_string(), "Boolean");
        assert_eq!(ColumnKind::Temporal.to_string(), "Temporal");
        assert_eq!(ColumnKind::Categorical.to_string(), "Categorical");
    }

    #[test]
    fn test_file_health_score_ranges() {
        let health = FileHealth {
            score: 95,
            risks: vec![],
        };
        assert!(health.score >= 0 && health.score <= 100);

        let unhealthy = FileHealth {
            score: 30,
            risks: vec!["High null percentage".to_string()],
        };
        assert!(unhealthy.score < 50);
        assert!(!unhealthy.risks.is_empty());
    }
}
