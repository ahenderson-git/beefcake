use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ColumnSummary {
    pub name: String,
    pub kind: ColumnKind,
    pub count: usize,
    pub nulls: usize,
    pub has_special: bool,
    pub stats: ColumnStats,
    pub interpretation: Vec<String>,
    pub business_summary: Vec<String>,
    pub samples: Vec<String>,
}

impl ColumnSummary {
    pub fn is_compatible_with(&self, target: ColumnKind) -> bool {
        if self.kind == target {
            return true;
        }

        // Basic check based on kinds
        if !self.kind.is_compatible_with(target) {
            return false;
        }

        // Smarter check for Categorical based on content
        if self.kind == ColumnKind::Categorical {
            if let ColumnStats::Categorical(freq) = &self.stats {
                match target {
                    ColumnKind::Numeric => {
                        // Are all categories potentially numeric?
                        freq.keys().all(|s| s.parse::<f64>().is_ok())
                    }
                    ColumnKind::Boolean => {
                        // Are all categories potentially boolean?
                        freq.keys().all(|s| {
                            let s = s.to_lowercase();
                            matches!(s.as_str(), "true" | "false" | "1" | "0" | "yes" | "no")
                        })
                    }
                    ColumnKind::Temporal => {
                        // If it's not numeric and has no separators, it's likely not a date
                        freq.keys().all(|s| {
                            s.parse::<f64>().is_ok() || s.contains('-') || s.contains('/') || s.contains(':')
                        })
                    }
                    _ => true,
                }
            } else {
                true
            }
        } else {
            true
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum NormalizationMethod {
    #[default]
    None,
    ZScore,
    MinMax,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum ImputeMode {
    #[default]
    None,
    Mean,
    Median,
    Zero,
    Mode,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum MlModelKind {
    #[default]
    LinearRegression,
    DecisionTree,
    LogisticRegression,
}

impl MlModelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinearRegression => "Linear Regression",
            Self::DecisionTree => "Decision Tree",
            Self::LogisticRegression => "Logistic Regression",
        }
    }

    pub fn is_suitable_target(&self, kind: ColumnKind) -> bool {
        match self {
            Self::LinearRegression => kind == ColumnKind::Numeric,
            Self::DecisionTree | Self::LogisticRegression => {
                matches!(kind, ColumnKind::Boolean | ColumnKind::Categorical)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MlResults {
    pub model_kind: MlModelKind,
    pub target_column: String,
    pub feature_columns: Vec<String>,
    pub r2_score: Option<f64>,
    pub accuracy: Option<f64>,
    pub mse: Option<f64>,
    pub duration: std::time::Duration,
    pub coefficients: Option<HashMap<String, f64>>,
    pub intercept: Option<f64>,
    pub interpretation: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColumnCleanConfig {
    pub new_name: String,
    pub target_dtype: Option<ColumnKind>,
    pub active: bool,
    pub trim_whitespace: bool,
    pub remove_special_chars: bool,
    pub normalization: NormalizationMethod,
    pub one_hot_encode: bool,
    pub impute_mode: ImputeMode,
}

impl Default for ColumnCleanConfig {
    fn default() -> Self {
        Self {
            new_name: String::new(),
            target_dtype: None,
            active: true,
            trim_whitespace: false,
            remove_special_chars: false,
            normalization: NormalizationMethod::None,
            one_hot_encode: false,
            impute_mode: ImputeMode::None,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ColumnStats {
    Numeric(NumericStats),
    Text(TextStats),
    Categorical(HashMap<String, usize>),
    Temporal(TemporalStats),
    Boolean(BooleanStats),
}

impl ColumnStats {
    pub fn n_distinct(&self) -> usize {
        match self {
            Self::Numeric(_) => 0, // Not tracked for numeric yet
            Self::Text(s) => s.distinct,
            Self::Categorical(freq) => freq.len(),
            Self::Temporal(_) => 0, // Not tracked
            Self::Boolean(s) => {
                let mut count = 0;
                if s.true_count > 0 { count += 1; }
                if s.false_count > 0 { count += 1; }
                count
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BooleanStats {
    pub true_count: usize,
    pub false_count: usize,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TemporalStats {
    pub min: Option<String>,
    pub max: Option<String>,
    pub p05: Option<f64>,
    pub p95: Option<f64>,
    pub is_sorted: bool,
    pub is_sorted_rev: bool,
    pub bin_width: f64,
    pub histogram: Vec<(f64, usize)>, // timestamp (ms) and count
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NumericStats {
    pub min: Option<f64>,
    pub p05: Option<f64>,
    pub q1: Option<f64>,
    pub median: Option<f64>,
    pub mean: Option<f64>,
    pub trimmed_mean: Option<f64>,
    pub q3: Option<f64>,
    pub p95: Option<f64>,
    pub max: Option<f64>,
    pub std_dev: Option<f64>,
    pub skew: Option<f64>,
    pub zero_count: usize,
    pub negative_count: usize,
    pub is_integer: bool,
    pub is_sorted: bool,
    pub is_sorted_rev: bool,
    pub bin_width: f64,
    pub histogram: Vec<(f64, usize)>, // bin center and count
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TextStats {
    pub distinct: usize,
    pub top_value: Option<(String, usize)>,
    pub min_length: usize,
    pub max_length: usize,
    pub avg_length: f64,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum ColumnKind {
    Numeric,
    Text,
    Categorical,
    Temporal,
    Boolean,
    Nested,
}

impl ColumnKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Numeric => "Numeric",
            Self::Text => "Text",
            Self::Categorical => "Categorical",
            Self::Temporal => "Temporal",
            Self::Boolean => "Boolean",
            Self::Nested => "Nested",
        }
    }

    pub fn is_compatible_with(&self, target: ColumnKind) -> bool {
        if *self == target {
            return true;
        }

        match (self, target) {
            // Everything can be converted to Text or Categorical
            (_, ColumnKind::Text) | (_, ColumnKind::Categorical) => true,

            // Text and Categorical can potentially be anything (parsing)
            (ColumnKind::Text, _) => target != ColumnKind::Nested,
            (ColumnKind::Categorical, _) => target != ColumnKind::Nested,

            // Numeric can be Boolean (0/1) or Temporal (timestamp)
            (ColumnKind::Numeric, ColumnKind::Boolean) => true,
            (ColumnKind::Numeric, ColumnKind::Temporal) => true,

            // Boolean can be Numeric (0/1)
            (ColumnKind::Boolean, ColumnKind::Numeric) => true,

            // Temporal can be Numeric (timestamp)
            (ColumnKind::Temporal, ColumnKind::Numeric) => true,

            _ => false,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct FileHealth {
    pub score: f32,
    pub risks: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_kind_compatibility() {
        // Numeric compatibility
        assert!(ColumnKind::Numeric.is_compatible_with(ColumnKind::Text));
        assert!(ColumnKind::Numeric.is_compatible_with(ColumnKind::Boolean));
        assert!(ColumnKind::Numeric.is_compatible_with(ColumnKind::Temporal));
        assert!(!ColumnKind::Numeric.is_compatible_with(ColumnKind::Nested));

        // Text compatibility
        assert!(ColumnKind::Text.is_compatible_with(ColumnKind::Numeric));
        assert!(ColumnKind::Text.is_compatible_with(ColumnKind::Boolean));
        assert!(ColumnKind::Text.is_compatible_with(ColumnKind::Temporal));
        assert!(!ColumnKind::Text.is_compatible_with(ColumnKind::Nested));

        // Boolean compatibility
        assert!(ColumnKind::Boolean.is_compatible_with(ColumnKind::Text));
        assert!(ColumnKind::Boolean.is_compatible_with(ColumnKind::Numeric));
        assert!(!ColumnKind::Boolean.is_compatible_with(ColumnKind::Temporal));
        assert!(!ColumnKind::Boolean.is_compatible_with(ColumnKind::Nested));

        // Temporal compatibility
        assert!(ColumnKind::Temporal.is_compatible_with(ColumnKind::Text));
        assert!(ColumnKind::Temporal.is_compatible_with(ColumnKind::Numeric));
        assert!(!ColumnKind::Temporal.is_compatible_with(ColumnKind::Boolean));
        assert!(!ColumnKind::Temporal.is_compatible_with(ColumnKind::Nested));

        // Nested compatibility
        assert!(ColumnKind::Nested.is_compatible_with(ColumnKind::Text));
        assert!(!ColumnKind::Nested.is_compatible_with(ColumnKind::Numeric));
        assert!(!ColumnKind::Nested.is_compatible_with(ColumnKind::Boolean));
        assert!(!ColumnKind::Nested.is_compatible_with(ColumnKind::Temporal));
    }

    #[test]
    fn test_column_summary_compatibility() {
        let mut freq = HashMap::new();
        freq.insert("SYD".to_string(), 10);
        freq.insert("MEL".to_string(), 5);

        let summary = ColumnSummary {
            name: "city".to_string(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq),
            interpretation: vec![],
            business_summary: vec![],
            samples: vec![],
        };

        assert!(summary.is_compatible_with(ColumnKind::Text));
        assert!(!summary.is_compatible_with(ColumnKind::Numeric));
        assert!(!summary.is_compatible_with(ColumnKind::Boolean));
        assert!(!summary.is_compatible_with(ColumnKind::Temporal));

        // Now with numeric categories
        let mut freq_num = HashMap::new();
        freq_num.insert("1".to_string(), 10);
        freq_num.insert("2".to_string(), 5);

        let summary_num = ColumnSummary {
            name: "id".to_string(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_num),
            interpretation: vec![],
            business_summary: vec![],
            samples: vec![],
        };

        assert!(summary_num.is_compatible_with(ColumnKind::Numeric));
        assert!(!summary_num.is_compatible_with(ColumnKind::Boolean));
        assert!(summary_num.is_compatible_with(ColumnKind::Temporal)); // Numeric is allowed for Temporal

        // Now with date-like categories
        let mut freq_date = HashMap::new();
        freq_date.insert("2023-01-01".to_string(), 10);
        
        let summary_date = ColumnSummary {
            name: "date".to_string(),
            kind: ColumnKind::Categorical,
            count: 10,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_date),
            interpretation: vec![],
            business_summary: vec![],
            samples: vec![],
        };
        assert!(summary_date.is_compatible_with(ColumnKind::Temporal));

        // Now with boolean categories
        let mut freq_bool = HashMap::new();
        freq_bool.insert("1".to_string(), 10);
        freq_bool.insert("0".to_string(), 5);

        let summary_bool = ColumnSummary {
            name: "active".to_string(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_bool),
            interpretation: vec![],
            business_summary: vec![],
            samples: vec![],
        };

        assert!(summary_bool.is_compatible_with(ColumnKind::Numeric));
        assert!(summary_bool.is_compatible_with(ColumnKind::Boolean));
    }

    #[test]
    fn test_ml_target_suitability() {
        assert!(MlModelKind::LinearRegression.is_suitable_target(ColumnKind::Numeric));
        assert!(!MlModelKind::LinearRegression.is_suitable_target(ColumnKind::Categorical));
        assert!(!MlModelKind::LinearRegression.is_suitable_target(ColumnKind::Boolean));

        assert!(!MlModelKind::LogisticRegression.is_suitable_target(ColumnKind::Numeric));
        assert!(MlModelKind::LogisticRegression.is_suitable_target(ColumnKind::Categorical));
        assert!(MlModelKind::LogisticRegression.is_suitable_target(ColumnKind::Boolean));

        assert!(!MlModelKind::DecisionTree.is_suitable_target(ColumnKind::Numeric));
        assert!(MlModelKind::DecisionTree.is_suitable_target(ColumnKind::Categorical));
        assert!(MlModelKind::DecisionTree.is_suitable_target(ColumnKind::Boolean));
    }
}
