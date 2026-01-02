use super::types::{ColumnSummary, ColumnStats};
use std::f64::consts::PI;

pub const MISSING_DATA_HIGH: f64 = 15.0;
pub const MISSING_DATA_MEDIUM: f64 = 5.0;
pub const SKEW_THRESHOLD: f64 = 0.1;
pub const GAP_RATIO_THRESHOLD: f64 = 0.1;
pub const NONPARAMETRIC_SKEW_THRESHOLD: f64 = 0.3;
pub const VARIABILITY_LOW: f64 = 0.1;
pub const VARIABILITY_HIGH: f64 = 0.6;
pub const OUTLIER_ZOOM_THRESHOLD: f64 = 3.0;
pub const TINY_BAR_THRESHOLD: f64 = 0.005;
pub const DOMINANT_BIN_THRESHOLD: f64 = 0.9;
pub const GAUSS_PEAK_CONCENTRATION: f64 = 1.5;
pub const UNEVEN_DISTRIBUTION_THRESHOLD: f64 = 5.0;

impl ColumnSummary {
    pub fn generate_interpretation(&self) -> Vec<String> {
        let mut signals = Vec::new();

        // 1. Missing Data Signal
        let null_pct = if self.count > 0 {
            (self.nulls as f64 / self.count as f64) * 100.0
        } else {
            0.0
        };

        if self.nulls == 0 {
            signals.push("Complete data set with no missing values.");
        } else if null_pct > MISSING_DATA_HIGH {
            signals.push("Significant missing data; results may be biased.");
        } else if null_pct > MISSING_DATA_MEDIUM {
            signals.push("Material amount of missing data.");
        }

        // 2. Type-Specific Analytical Signals
        match &self.stats {
            ColumnStats::Numeric(s) => Self::collect_numeric_signals(s, &mut signals),
            ColumnStats::Categorical(freq) => Self::collect_categorical_signals(freq, &mut signals),
            ColumnStats::Text(s) => self.collect_text_signals(s, &mut signals),
            ColumnStats::Temporal(s) => Self::collect_temporal_signals(s, &mut signals),
            ColumnStats::Boolean(s) => Self::collect_boolean_signals(s, &mut signals),
        }

        // 3. Quality Signals
        if self.has_special {
            signals.push("Contains unusual or hidden characters.");
        }

        if signals.is_empty() {
            vec!["No significant patterns detected.".to_owned()]
        } else {
            signals.into_iter().map(|s| s.to_owned()).collect()
        }
    }

    fn collect_numeric_signals(s: &super::types::NumericStats, signals: &mut Vec<&'static str>) {
        if let (Some(mean), Some(median), Some(min), Some(max), Some(q1), Some(q3)) =
            (s.mean, s.median, s.min, s.max, s.q1, s.q3)
        {
            let range = max - min;
            let iqr = q3 - q1;

            if range == 0.0 {
                signals.push("Constant value across all records.");
            }

            if s.is_sorted {
                signals.push("Values are strictly increasing.");
            } else if s.is_sorted_rev {
                signals.push("Values are strictly decreasing.");
            }

            if s.is_integer {
                signals.push("Values are discrete whole numbers.");
            }

            if s.zero_count > 0 {
                signals.push("Contains zero values.");
            }
            if s.negative_count > 0 {
                signals.push("Contains negative values.");
            }

            Self::collect_numeric_distribution_signals(s, mean, median, iqr, signals);
            Self::collect_numeric_histogram_signals(s, range, iqr, signals);
        }
    }

    fn collect_numeric_distribution_signals(
        s: &super::types::NumericStats,
        mean: f64,
        median: f64,
        iqr: f64,
        signals: &mut Vec<&'static str>,
    ) {
        // Skewness / Distribution Shape
        if let Some(skew) = s.skew {
            if skew.abs() < SKEW_THRESHOLD {
                signals.push("Symmetric distribution.");
            } else if skew > 0.0 {
                signals.push("Right-skewed; average is influenced by high outliers.");
            } else {
                signals.push("Left-skewed; average is influenced by low values.");
            }
        } else {
            // Fallback if skew is not available
            let diff_ratio = (mean - median).abs() / iqr.max(s.std_dev.unwrap_or(0.0) * 0.1).max(1e-9);
            if diff_ratio < SKEW_THRESHOLD {
                signals.push("Symmetric distribution.");
            } else if mean > median {
                signals.push("Right-skewed; average is influenced by high outliers.");
            } else {
                signals.push("Left-skewed; average is influenced by low values.");
            }
        }

        // Mean vs Median Gap Indicator
        if median.abs() > 1e-9 {
            let gap_ratio = (mean - median).abs() / median.abs();
            if gap_ratio > GAP_RATIO_THRESHOLD {
                signals.push("Outliers likely influencing the average.");
            }
        }

        // Standard Deviation Reliability
        if let Some(std_dev) = s.std_dev {
            if std_dev > 0.0 {
                let nonparametric_skew = (mean - median).abs() / std_dev;
                if nonparametric_skew > NONPARAMETRIC_SKEW_THRESHOLD {
                    signals.push("Standard deviation may be less reliable as an indicator of 'typical' spread because the mean is significantly offset by skew or outliers.");
                }
            }
        }
    }

    fn collect_numeric_histogram_signals(
        s: &super::types::NumericStats,
        range: f64,
        iqr: f64,
        signals: &mut Vec<&'static str>,
    ) {
        // Variability
        if range > 0.0 {
            if iqr / range < VARIABILITY_LOW {
                signals.push("Values are tightly clustered around the center.");
            } else if iqr / range > VARIABILITY_HIGH {
                signals.push("High variability across the range.");
            }

            // Range-based signals (Skinny bars & Gaps)
            let mut has_gaps = false;
            if s.histogram.len() > 1 {
                for window in s.histogram.windows(2) {
                    if let [w0, w1] = window {
                        if w1.0 - w0.0 > s.bin_width * 1.5 {
                            has_gaps = true;
                            break;
                        }
                    }
                }
            }

            if has_gaps {
                signals.push("Gaps between bars indicate sparse data or isolated clusters.");
                let num_bins = range / s.bin_width;
                if num_bins > 100.0 {
                    signals.push("Histogram bars appear very skinny because the data is spread across many small bins.");
                }
            }

            if let (Some(p05), Some(p95)) = (s.p05, s.p95) {
                let zoom_range = p95 - p05;
                if range > OUTLIER_ZOOM_THRESHOLD * zoom_range && zoom_range > 0.0 {
                    signals.push("Histogram bars appear skinny because extreme outliers are stretching the scale.");
                }
            }

            // Height-based signals (Invisible bars)
            if !s.histogram.is_empty() {
                let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(0) as f64;
                let total_count: usize = s.histogram.iter().map(|h| h.1).sum();

                let has_tiny_bars = s.histogram.iter().any(|&(_, c)| {
                    c > 0 && (c as f64) < max_count * TINY_BAR_THRESHOLD
                });
                if has_tiny_bars {
                    signals.push("Some bars are so short compared to the tallest one that they may be invisible.");
                }

                if total_count > 0 && max_count / total_count as f64 > DOMINANT_BIN_THRESHOLD {
                    signals.push("A single bin dominates the distribution, making other values difficult to see.");
                }
            }
        }

        // Gaussian vs Histogram height
        if let Some(sigma) = s.std_dev {
            if sigma > 0.0 && !s.histogram.is_empty() {
                let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                let gauss_peak =
                    (total_count as f64 * s.bin_width) / (sigma * (2.0 * PI).sqrt());
                let max_bar = s.histogram.iter().map(|h| h.1).max().unwrap_or(0) as f64;

                if max_bar > gauss_peak * GAUSS_PEAK_CONCENTRATION {
                    signals.push("The orange distribution curve is low because the data is more concentrated than a 'normal' distribution.");
                }
            }
        }
    }

    fn collect_categorical_signals(freq: &std::collections::HashMap<String, usize>, signals: &mut Vec<&'static str>) {
        if freq.len() == 1 {
            signals.push("Constant value across all records.");
        }
        signals.push("Categorical field.");
        if freq.len() == 2 {
            signals.push("Binary field; suggests a toggle or yes/no choice.");
        }
        if freq.len() > 1 {
            if let (Some(max_v), Some(min_v)) = (freq.values().max(), freq.values().min()) {
                if (*max_v as f64 / *min_v as f64) > UNEVEN_DISTRIBUTION_THRESHOLD {
                    signals.push("Value distribution is heavily uneven.");
                }
            }
        }
    }

    fn collect_temporal_signals(s: &super::types::TemporalStats, signals: &mut Vec<&'static str>) {
        signals.push("Time-based data sequence.");
        if s.is_sorted {
            signals.push("Strictly chronological order.");
        } else if s.is_sorted_rev {
            signals.push("Reverse chronological order.");
        }
    }

    fn collect_text_signals(&self, s: &super::types::TextStats, signals: &mut Vec<&'static str>) {
        if s.distinct == 1 {
            signals.push("Constant value across all records.");
        }

        if s.distinct == self.count && self.nulls == 0 {
            signals.push("Likely a unique identifier or sequential ID.");
        } else {
            signals.push("Standard text field.");
        }

        if s.min_length == s.max_length && s.min_length > 0 {
            signals.push("Fixed-length text entries.");
        }
    }

    fn collect_boolean_signals(s: &super::types::BooleanStats, signals: &mut Vec<&'static str>) {
        signals.push("Binary field; suggests a toggle or yes/no choice.");
        if s.true_count == 0 || s.false_count == 0 {
            signals.push("Field is constant.");
        }
    }

    pub fn generate_business_summary(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // 1. Quality & Completeness
        let null_pct = if self.count > 0 {
            (self.nulls as f64 / self.count as f64) * 100.0
        } else {
            0.0
        };

        if self.nulls == 0 {
            insights.push("This data is 100% complete and reliable.");
        } else if null_pct > MISSING_DATA_HIGH {
            insights.push("Caution: A significant amount of information is missing here, which may lead to incomplete insights.");
        } else if null_pct > MISSING_DATA_MEDIUM {
            insights.push("Most of the data is present, but some records are missing specific details.");
        }

        // 2. Business Meaning
        match &self.stats {
            ColumnStats::Numeric(s) => Self::collect_numeric_insights(s, &mut insights),
            ColumnStats::Categorical(freq) => Self::collect_categorical_insights(freq, &mut insights),
            ColumnStats::Text(s) => self.collect_text_insights(s, &mut insights),
            ColumnStats::Temporal(s) => Self::collect_temporal_insights(s, &mut insights),
            ColumnStats::Boolean(_) => insights.push("This represents a simple toggle or true/false status."),
        }

        if insights.is_empty() {
            vec!["Standard data column with no unusual patterns identified.".to_owned()]
        } else {
            insights.into_iter().map(|s| s.to_owned()).collect()
        }
    }

    fn collect_numeric_insights(s: &super::types::NumericStats, insights: &mut Vec<&'static str>) {
        if let (Some(mean), Some(median), Some(min), Some(max)) = (s.mean, s.median, s.min, s.max) {
            if min == max {
                insights.push("Every record has the same value, providing no variety for analysis.");
            }

            if s.is_sorted {
                insights.push("The data follows a perfect sequential order.");
            }

            if s.is_integer {
                insights.push("This contains only whole numbers, typical for counts or discrete quantities.");
            }

            if s.negative_count > 0 {
                insights.push("Includes negative values, which may indicate refunds, adjustments, or errors.");
            }

            if let Some(skew) = s.skew {
                if skew.abs() < SKEW_THRESHOLD {
                    insights.push("The values are balanced and follow a predictable pattern.");
                } else if skew > 0.0 {
                    insights.push("While most values are lower, a few high-value exceptions are pulling the average up.");
                } else {
                    insights.push("Most values are on the higher side, but some unusually low entries are pulling the average down.");
                }
            }

            if let Some(std_dev) = s.std_dev {
                if std_dev > 0.0 {
                    let nonparametric_skew = (mean - median).abs() / std_dev;
                    if nonparametric_skew > NONPARAMETRIC_SKEW_THRESHOLD {
                        insights.push("The 'average' is being heavily distorted by extreme outliers and might not represent a 'typical' case.");
                    }
                }
            }
        }
    }

    fn collect_categorical_insights(freq: &std::collections::HashMap<String, usize>, insights: &mut Vec<&'static str>) {
        if freq.len() == 1 {
            insights.push("This column is constant; every record has the same category.");
        } else if freq.len() == 2 {
            insights.push("This captures a simple choice or binary state (like Yes/No).");
        } else if freq.len() > 1 {
            if let (Some(max_v), Some(min_v)) = (freq.values().max(), freq.values().min()) {
                if (*max_v as f64 / *min_v as f64) > UNEVEN_DISTRIBUTION_THRESHOLD {
                    insights.push("The distribution is uneven, with some categories appearing much more frequently than others.");
                } else {
                    insights.push("The data is relatively well-distributed across different categories.");
                }
            }
        }
    }

    fn collect_temporal_insights(s: &super::types::TemporalStats, insights: &mut Vec<&'static str>) {
        insights.push("This tracks when events occurred, allowing for time-based trend analysis.");
        if s.is_sorted {
            insights.push("Events are recorded in a perfect chronological sequence.");
        }
    }

    fn collect_text_insights(&self, s: &super::types::TextStats, insights: &mut Vec<&'static str>) {
        if s.distinct == 1 {
            insights.push("Every record in this column is identical.");
        }

        if s.distinct == self.count && self.nulls == 0 {
            insights.push("This appears to be a unique tracking number or identifier for each record.");
        } else {
            insights.push("This is a standard text field containing descriptive information.");
        }

        if s.min_length == s.max_length && s.min_length > 0 {
            insights.push("Every entry has the same length, which is characteristic of codes or formatted IDs.");
        }
    }
}
