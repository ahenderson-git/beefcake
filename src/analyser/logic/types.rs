use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CorrelationMatrix {
    pub columns: Vec<String>,
    pub data: Vec<Vec<f64>>,
}

#[derive(Serialize)]
pub struct AnalysisResponse {
    pub file_name: String,
    pub path: String,
    pub file_size: u64,
    pub row_count: usize,
    pub total_row_count: usize,
    pub column_count: usize,
    pub summary: Vec<ColumnSummary>,
    pub health: FileHealth,
    #[serde(with = "duration_serde", rename = "analysis_duration")]
    pub duration: std::time::Duration,
    #[serde(skip)]
    pub df: DataFrame,
    pub correlation_matrix: Option<CorrelationMatrix>,
}

mod duration_serde {
    use serde::{Serializer, ser::SerializeStruct as _};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Duration", 2)?;
        state.serialize_field("secs", &duration.as_secs())?;
        state.serialize_field("nanos", &duration.subsec_nanos())?;
        state.end()
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ColumnSummary {
    pub name: String,
    pub standardised_name: String,
    pub kind: ColumnKind,
    pub count: usize,
    pub nulls: usize,
    pub has_special: bool,
    pub stats: ColumnStats,
    pub interpretation: Vec<String>,
    pub business_summary: Vec<String>,
    pub ml_advice: Vec<String>,
    pub samples: Vec<String>,
}

impl ColumnSummary {
    pub fn null_pct(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.nulls as f64 / self.count as f64) * 100.0
        }
    }

    pub fn uniqueness_ratio(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.stats.n_distinct() as f64 / self.count as f64
        }
    }

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
                            s.parse::<f64>().is_ok()
                                || s.contains('-')
                                || s.contains('/')
                                || s.contains(':')
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

    pub fn apply_advice_to_config(&self, config: &mut ColumnCleanConfig) {
        // Automatically enable basic text cleaning for string-like columns
        if self.kind == ColumnKind::Text || self.kind == ColumnKind::Categorical {
            config.trim_whitespace = true;
            config.standardise_nulls = true;
        }

        // Automatically enable special character removal if they were detected during analysis
        if self.has_special {
            config.remove_special_chars = true;
            // If we have special characters, we might also have non-ascii junk
            config.remove_non_ascii = true;
        }

        for advice in &self.ml_advice {
            if advice.contains("Outlier Clipping") {
                config.clip_outliers = true;
            }
            if advice.contains("Normalization") {
                config.normalisation = NormalisationMethod::ZScore;
            }
            if advice.contains("Mean or Median Imputation") {
                config.impute_mode = ImputeMode::Mean;
            }
            if advice.contains("Recommend One-Hot encoding") {
                config.one_hot_encode = true;
            }
        }
    }
}

impl ColumnCleanConfig {
    /// Returns true if the advanced cleaning master switch is on AND at least one
    /// specific advanced cleaning step is enabled.
    pub fn has_any_advanced_cleaning(&self) -> bool {
        self.advanced_cleaning
            && (self.trim_whitespace
                || self.remove_special_chars
                || self.remove_non_ascii
                || self.standardise_nulls
                || self.text_case != TextCase::None
                || !self.regex_find.is_empty())
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum NormalisationMethod {
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

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TextCase {
    #[default]
    None,
    Lowercase,
    Uppercase,
    TitleCase,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColumnCleanConfig {
    pub new_name: String,
    pub target_dtype: Option<ColumnKind>,
    pub active: bool,
    pub advanced_cleaning: bool,
    pub ml_preprocessing: bool,
    pub trim_whitespace: bool,
    pub remove_special_chars: bool,
    pub text_case: TextCase,
    pub standardise_nulls: bool,
    pub remove_non_ascii: bool,
    pub regex_find: String,
    pub regex_replace: String,
    pub rounding: Option<u32>,
    pub extract_numbers: bool,
    pub clip_outliers: bool,
    pub temporal_format: String,
    pub timezone_utc: bool,
    pub freq_threshold: Option<usize>,
    pub normalisation: NormalisationMethod,
    pub one_hot_encode: bool,
    pub impute_mode: ImputeMode,
}

impl Default for ColumnCleanConfig {
    fn default() -> Self {
        Self {
            new_name: String::new(),
            target_dtype: None,
            active: true,
            advanced_cleaning: true,
            ml_preprocessing: false,
            trim_whitespace: false,
            remove_special_chars: false,
            text_case: TextCase::None,
            standardise_nulls: false,
            remove_non_ascii: false,
            regex_find: String::new(),
            regex_replace: String::new(),
            rounding: None,
            extract_numbers: false,
            clip_outliers: false,
            temporal_format: String::new(),
            timezone_utc: false,
            freq_threshold: None,
            normalisation: NormalisationMethod::None,
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
            Self::Numeric(s) => s.distinct_count,
            Self::Temporal(s) => s.distinct_count,
            Self::Text(s) => s.distinct,
            Self::Categorical(freq) => freq.len(),
            Self::Boolean(s) => {
                let mut count = 0;
                if s.true_count > 0 {
                    count += 1;
                }
                if s.false_count > 0 {
                    count += 1;
                }
                count
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct BooleanStats {
    pub true_count: usize,
    pub false_count: usize,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct TemporalStats {
    pub min: Option<String>,
    pub max: Option<String>,
    pub distinct_count: usize,
    pub p05: Option<f64>,
    pub p95: Option<f64>,
    pub is_sorted: bool,
    pub is_sorted_rev: bool,
    pub bin_width: f64,
    pub histogram: Vec<(f64, usize)>, // timestamp (ms) and count
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct NumericStats {
    pub min: Option<f64>,
    pub distinct_count: usize,
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
    pub histogram: Vec<(f64, usize)>, // bin centre and count
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
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

impl std::fmt::Display for ColumnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
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

    pub fn is_compatible_with(&self, target: Self) -> bool {
        if *self == target {
            return true;
        }

        match (self, target) {
            // Everything can be converted to Text or Categorical
            (_, Self::Text | Self::Categorical)
            | (Self::Numeric, Self::Boolean | Self::Temporal)
            | (Self::Boolean | Self::Temporal, Self::Numeric) => true,

            // Text and Categorical can potentially be anything (parsing)
            (Self::Text | Self::Categorical, _) => target != Self::Nested,

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
        freq.insert("SYD".to_owned(), 10);
        freq.insert("MEL".to_owned(), 5);

        let summary = ColumnSummary {
            name: "city".to_owned(),
            standardised_name: "city".to_owned(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert!(summary.is_compatible_with(ColumnKind::Text));
        assert!(!summary.is_compatible_with(ColumnKind::Numeric));
        assert!(!summary.is_compatible_with(ColumnKind::Boolean));
        assert!(!summary.is_compatible_with(ColumnKind::Temporal));

        // Now with numeric categories
        let mut freq_num = HashMap::new();
        freq_num.insert("1".to_owned(), 10);
        freq_num.insert("2".to_owned(), 5);

        let summary_num = ColumnSummary {
            name: "id".to_owned(),
            standardised_name: "id".to_owned(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_num),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };

        assert!(summary_num.is_compatible_with(ColumnKind::Numeric));
        assert!(!summary_num.is_compatible_with(ColumnKind::Boolean));
        assert!(summary_num.is_compatible_with(ColumnKind::Temporal)); // Numeric is allowed for Temporal

        // Now with date-like categories
        let mut freq_date = HashMap::new();
        freq_date.insert("2023-01-01".to_owned(), 10);

        let summary_date = ColumnSummary {
            name: "date".to_owned(),
            standardised_name: "date".to_owned(),
            kind: ColumnKind::Categorical,
            count: 10,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_date),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
            samples: vec![],
        };
        assert!(summary_date.is_compatible_with(ColumnKind::Temporal));

        // Now with boolean categories
        let mut freq_bool = HashMap::new();
        freq_bool.insert("1".to_owned(), 10);
        freq_bool.insert("0".to_owned(), 5);

        let summary_bool = ColumnSummary {
            name: "active".to_owned(),
            standardised_name: "active".to_owned(),
            kind: ColumnKind::Categorical,
            count: 15,
            nulls: 0,
            has_special: false,
            stats: ColumnStats::Categorical(freq_bool),
            interpretation: vec![],
            business_summary: vec![],
            ml_advice: vec![],
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
