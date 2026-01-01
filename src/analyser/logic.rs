use anyhow::Result;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// DATA STRUCTURES

#[derive(Clone, Deserialize, Serialize)]
pub struct ColumnSummary {
    pub name: String,
    pub kind: ColumnKind,
    pub count: usize,
    pub nulls: usize,
    pub has_special: bool,
    pub stats: ColumnStats,
    pub interpretation: String,
    pub business_summary: String,
    pub samples: Vec<String>,
}

impl ColumnSummary {
    pub fn generate_interpretation(&self) -> String {
        let mut signals = Vec::new();

        // 1. Missing Data Signal
        let null_pct = if self.count > 0 {
            (self.nulls as f64 / self.count as f64) * 100.0
        } else {
            0.0
        };

        if self.nulls == 0 {
            signals.push("Complete data set with no missing values.");
        } else if null_pct > 15.0 {
            signals.push("Significant missing data; results may be biased.");
        } else if null_pct > 5.0 {
            signals.push("Material amount of missing data.");
        }

        // 2. Type-Specific Analytical Signals
        match &self.stats {
            ColumnStats::Numeric(s) => {
                if let (Some(mean), Some(median), Some(min), Some(max), Some(q1), Some(q3)) =
                    (s.mean, s.median, s.min, s.max, s.q1, s.q3)
                {
                    let range = max - min;
                    let iqr = q3 - q1;

                    // Skewness / Distribution Shape
                    if let Some(skew) = s.skew {
                        if skew.abs() < 0.1 {
                            signals.push("Symmetric distribution.");
                        } else if skew > 0.0 {
                            signals.push("Right-skewed; average is influenced by high outliers.");
                        } else {
                            signals.push("Left-skewed; average is influenced by low values.");
                        }
                    } else if range > 0.0 {
                        // Fallback if skew is not available
                        let diff_ratio = (mean - median).abs() / iqr.max(s.std_dev.unwrap_or(0.0) * 0.1).max(1e-9);
                        if diff_ratio < 0.1 {
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
                        if gap_ratio > 0.1 {
                            signals.push("Outliers likely influencing the average.");
                        }
                    }

                    // Standard Deviation Reliability
                    if let Some(std_dev) = s.std_dev {
                        if std_dev > 0.0 {
                            let nonparametric_skew = (mean - median).abs() / std_dev;
                            if nonparametric_skew > 0.3 {
                                signals.push("Standard deviation may be less reliable as an indicator of 'typical' spread because the mean is significantly offset by skew or outliers.");
                            }
                        }
                    }

                    // Variability
                    if range > 0.0 {
                        if iqr / range < 0.1 {
                            signals.push("Values are tightly clustered around the center.");
                        } else if iqr / range > 0.6 {
                            signals.push("High variability across the range.");
                        }

                        // Range-based signals (Skinny bars & Gaps)
                        if range > 0.0 {
                            let mut has_gaps = false;
                            if s.histogram.len() > 1 {
                                for i in 0..s.histogram.len() - 1 {
                                    if s.histogram[i + 1].0 - s.histogram[i].0 > s.bin_width * 1.5 {
                                        has_gaps = true;
                                        break;
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
                                if range > 3.0 * zoom_range && zoom_range > 0.0 {
                                    signals.push("Histogram bars appear skinny because extreme outliers are stretching the scale.");
                                }
                            }
                        }

                        // Height-based signals (Invisible bars)
                        if !s.histogram.is_empty() {
                            let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(0) as f64;
                            let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                            
                            let has_tiny_bars = s.histogram.iter().any(|&(_, c)| c > 0 && (c as f64) < max_count * 0.005);
                            if has_tiny_bars {
                                signals.push("Some bars are so short compared to the tallest one that they may be invisible.");
                            }
                            
                            if total_count > 0 && max_count / total_count as f64 > 0.9 {
                                signals.push("A single bin dominates the distribution, making other values difficult to see.");
                            }
                        }
                    }

                    // Gaussian vs Histogram height
                    if let (Some(sigma), _) = (s.std_dev, s.mean) {
                        if sigma > 0.0 && !s.histogram.is_empty() {
                            let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                            let gauss_peak = (total_count as f64 * s.bin_width)
                                / (sigma * (2.0 * std::f64::consts::PI).sqrt());
                            let max_bar = s.histogram.iter().map(|h| h.1).max().unwrap_or(0) as f64;

                            if max_bar > gauss_peak * 1.5 {
                                signals.push("The orange distribution curve is low because the data is more concentrated than a 'normal' distribution.");
                            }
                        }
                    }

                }
            }
            ColumnStats::Categorical(freq) => {
                signals.push("Categorical field.");
                if freq.len() == 2 {
                    signals.push("Binary field; suggests a toggle or yes/no choice.");
                }
                if freq.len() > 1 {
                    if let (Some(max_v), Some(min_v)) = (freq.values().max(), freq.values().min()) {
                        if (*max_v as f64 / *min_v as f64) > 5.0 {
                            signals.push("Value distribution is heavily uneven.");
                        }
                    }
                }
            }
            ColumnStats::Text(s) => {
                if s.distinct == self.count && self.nulls == 0 {
                    signals.push("Likely a unique identifier or sequential ID.");
                } else {
                    signals.push("Standard text field.");
                }
            }
            ColumnStats::Temporal(_) => {
                signals.push("Time-based data sequence.");
            }
            ColumnStats::Boolean(s) => {
                signals.push("Binary field; suggests a toggle or yes/no choice.");
                if s.true_count == 0 || s.false_count == 0 {
                    signals.push("Field is constant.");
                }
            }
        }

        // 3. Quality Signals
        if self.has_special {
            signals.push("Contains unusual or hidden characters.");
        }

        if signals.is_empty() {
            "No significant patterns detected.".to_string()
        } else {
            signals.join(" ")
        }
    }

    pub fn generate_business_summary(&self) -> String {
        let mut insights = Vec::new();

        // 1. Quality & Completeness
        let null_pct = if self.count > 0 {
            (self.nulls as f64 / self.count as f64) * 100.0
        } else {
            0.0
        };

        if self.nulls == 0 {
            insights.push("This data is 100% complete and reliable.");
        } else if null_pct > 15.0 {
            insights.push("Caution: A significant amount of information is missing here, which may lead to incomplete insights.");
        } else if null_pct > 5.0 {
            insights.push("Most of the data is present, but some records are missing specific details.");
        }

        // 2. Business Meaning
        match &self.stats {
            ColumnStats::Numeric(s) => {
                if let (Some(mean), Some(median), Some(std_dev)) = (s.mean, s.median, s.std_dev) {
                    if let Some(skew) = s.skew {
                        if skew.abs() < 0.1 {
                            insights.push("The values are balanced and follow a predictable pattern.");
                        } else if skew > 0.0 {
                            insights.push("While most values are lower, a few high-value exceptions are pulling the average up.");
                        } else {
                            insights.push("Most values are on the higher side, but some unusually low entries are pulling the average down.");
                        }
                    }

                    if std_dev > 0.0 {
                        let nonparametric_skew = (mean - median).abs() / std_dev;
                        if nonparametric_skew > 0.3 {
                            insights.push("The 'average' is being heavily distorted by extreme outliers and might not represent a 'typical' case.");
                        }
                    }
                }
            }
            ColumnStats::Categorical(freq) => {
                if freq.len() == 2 {
                    insights.push("This captures a simple choice or binary state (like Yes/No).");
                } else if freq.len() > 1 {
                    if let (Some(max_v), Some(min_v)) = (freq.values().max(), freq.values().min()) {
                        if (*max_v as f64 / *min_v as f64) > 5.0 {
                            insights.push("The distribution is uneven, with some categories appearing much more frequently than others.");
                        } else {
                            insights.push("The data is relatively well-distributed across different categories.");
                        }
                    }
                }
            }
            ColumnStats::Text(s) => {
                if s.distinct == self.count && self.nulls == 0 {
                    insights.push("This appears to be a unique tracking number or identifier for each record.");
                } else {
                    insights.push("This is a standard text field containing descriptive information.");
                }
            }
            ColumnStats::Temporal(_) => {
                insights.push("This column tracks when events occurred, allowing for time-based trend analysis.");
            }
            ColumnStats::Boolean(_) => {
                insights.push("This represents a simple toggle or true/false status.");
            }
        }

        if insights.is_empty() {
            "Standard data column with no unusual patterns identified.".to_string()
        } else {
            insights.join(" ")
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ColumnStats {
    Numeric(NumericStats),
    Text(TextStats),
    Categorical(HashMap<String, usize>),
    Temporal(TemporalStats),
    Boolean(BooleanStats),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BooleanStats {
    pub true_count: usize,
    pub false_count: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TemporalStats {
    pub min: Option<String>,
    pub max: Option<String>,
    pub p05: Option<f64>,
    pub p95: Option<f64>,
    pub bin_width: f64,
    pub histogram: Vec<(f64, usize)>, // timestamp (ms) and count
}

#[derive(Clone, Deserialize, Serialize)]
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
    pub bin_width: f64,
    pub histogram: Vec<(f64, usize)>, // bin center and count
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TextStats {
    pub distinct: usize,
    pub top_value: Option<(String, usize)>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
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
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FileHealth {
    pub score: f32,
    pub risks: Vec<String>,
}

pub fn calculate_file_health(summaries: &[ColumnSummary]) -> FileHealth {
    let mut risks = Vec::new();
    let mut score: f64 = 100.0;

    for col in summaries {
        let null_pct = if col.count > 0 {
            (col.nulls as f64 / col.count as f64) * 100.0
        } else {
            0.0
        };

        if null_pct > 15.0 {
            risks.push(format!(
                "⚠️ Column '{}' has significant missing data ({:.1}%).",
                col.name, null_pct
            ));
            score -= 10.0;
        } else if null_pct > 5.0 {
            score -= 5.0;
        }

        if col.has_special {
            risks.push(format!(
                "🔍 Hidden/special characters detected in '{}'.",
                col.name
            ));
            score -= 5.0;
        }

        match &col.stats {
            ColumnStats::Numeric(s) => {
                if let (Some(mean), Some(median), Some(min), Some(max)) =
                    (s.mean, s.median, s.min, s.max)
                {
                    let range = max - min;
                    if range > 0.0 {
                        let diff_ratio = (mean - median).abs() / range;
                        if diff_ratio > 0.1 {
                            risks.push(format!(
                                "📈 Column '{}' is heavily skewed; averages may be misleading.",
                                col.name
                            ));
                            score -= 5.0;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    FileHealth { 
        score: score.max(0.0) as f32,
        risks 
    }
}

pub type AnalysisReceiver = crossbeam_channel::Receiver<
    Result<(
        String,
        u64,
        Vec<ColumnSummary>,
        FileHealth,
        std::time::Duration,
        DataFrame,
    )>,
>;

// HELPER FUNCTIONS

pub fn load_df(path: &std::path::Path, progress: Arc<AtomicU64>) -> Result<DataFrame> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut df = match ext.as_str() {
        "json" => {
            let file = std::fs::File::open(path)?;
            JsonReader::new(file).finish()?
        }
        "jsonl" | "ndjson" => JsonLineReader::from_path(path)?.finish()?,
        "parquet" => {
            let file = std::fs::File::open(path)?;
            ParquetReader::new(file).finish()?
        }
        _ => LazyCsvReader::new(path.to_str().expect("Invalid path"))
            .with_try_parse_dates(true)
            .finish()?
            .collect()?,
    };

    if ext == "json" || ext == "jsonl" || ext == "ndjson" {
        df = try_parse_temporal_columns(df)?;
    }

    // Update progress to 100% since we loaded the whole thing
    progress.store(std::fs::metadata(path)?.len(), Ordering::SeqCst);

    Ok(df)
}

pub fn try_parse_temporal_columns(df: DataFrame) -> Result<DataFrame> {
    let mut columns = df.get_columns().to_vec();
    let mut changed = false;

    for i in 0..columns.len() {
        let col = &columns[i];
        if col.dtype().is_string() {
            let s = col.as_materialized_series();

            // Try Datetime (Microseconds is a good default for Polars)
            if let Ok(dt) = s.cast(&DataType::Datetime(TimeUnit::Microseconds, None)) {
                // If the number of nulls didn't increase, it's a perfect match
                if dt.null_count() == s.null_count() && s.len() > 0 {
                    columns[i] = Column::from(dt);
                    changed = true;
                    continue;
                }
            }

            // Try Date
            if let Ok(d) = s.cast(&DataType::Date) {
                if d.null_count() == s.null_count() && s.len() > 0 {
                    columns[i] = Column::from(d);
                    changed = true;
                }
            }
        }
    }

    if changed {
        DataFrame::new(columns).map_err(anyhow::Error::from)
    } else {
        Ok(df)
    }
}

pub fn analyse_df(df: DataFrame, trim_pct: f64) -> Result<Vec<ColumnSummary>> {
    let row_count = df.height();
    let mut summaries = Vec::new();

    for col in df.get_columns() {
        let name = col.name().to_string();
        let nulls = col.null_count();
        let count = row_count;

        let samples = {
            let series = col.as_materialized_series();
            let head = series.drop_nulls().head(Some(10));
            match head.cast(&DataType::String) {
                Ok(s_ca) => s_ca
                    .str()
                    .map(|ca| {
                        ca.into_iter()
                            .flatten()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                Err(_) => head.iter().map(|v| v.to_string()).collect(),
            }
        };

        let dtype = col.dtype();
        let mut has_special = false;

        let (kind, stats) = if dtype.is_bool() {
            let series = col.as_materialized_series();
            let ca = series.bool()?;
            let true_count = ca.sum().unwrap_or(0) as usize;
            let false_count = ca.len() - ca.null_count() - true_count;
            (
                ColumnKind::Boolean,
                ColumnStats::Boolean(BooleanStats {
                    true_count,
                    false_count,
                }),
            )
        } else if dtype.is_numeric() {
            let series = col.as_materialized_series();

            // Cast to f64 for common stats if it's numeric
            let f64_series = series.cast(&DataType::Float64)?;
            let ca = f64_series.f64()?;

            let min = ca.min();
            let max = ca.max();

            // Heuristic for "Effective Boolean" (Binary 0/1)
            let distinct_count = series.n_unique()?;
            let has_nulls = series.null_count() > 0;
            let non_null_distinct = if has_nulls {
                distinct_count.saturating_sub(1)
            } else {
                distinct_count
            };

            let is_effective_bool = if non_null_distinct <= 2 {
                if let (Some(min_v), Some(max_v)) = (min, max) {
                    (min_v == 0.0 && max_v == 1.0)
                        || (min_v == 0.0 && max_v == 0.0)
                        || (min_v == 1.0 && max_v == 1.0)
                } else {
                    false
                }
            } else {
                false
            };

            if is_effective_bool {
                let true_count = ca.into_iter().flatten().filter(|&v| v == 1.0).count();
                let false_count = ca.len() - ca.null_count() - true_count;
                (
                    ColumnKind::Boolean,
                    ColumnStats::Boolean(BooleanStats {
                        true_count,
                        false_count,
                    }),
                )
            } else {
                let mean = ca.mean();
                let std_dev = ca.std(1);

                let q1 = ca.quantile(0.25, QuantileMethod::Linear)?;
                let median = ca.median();
                let q3 = ca.quantile(0.75, QuantileMethod::Linear)?;
                let p05 = ca.quantile(0.05, QuantileMethod::Linear)?;
                let p95 = ca.quantile(0.95, QuantileMethod::Linear)?;

                let skew = if let (Some(m), Some(md), Some(q1v), Some(q3v)) = (mean, median, q1, q3) {
                    let iqr = q3v - q1v;
                    if iqr > 0.0 {
                        Some((m - md) / iqr)
                    } else if let Some(s) = std_dev {
                        if s > 0.0 {
                            Some((m - md) / s)
                        } else {
                            Some(0.0)
                        }
                    } else {
                        Some(0.0)
                    }
                } else {
                    None
                };

                // Trimmed Mean calculation
                let trimmed_mean = if trim_pct > 0.0 {
                    let sorted = ca.drop_nulls().sort(false);
                    let n = sorted.len();
                    let trim_count = (n as f64 * trim_pct).floor() as usize;
                    if n > 2 * trim_count && trim_count > 0 {
                        sorted.slice(trim_count as i64, n - 2 * trim_count).mean()
                    } else {
                        mean
                    }
                } else {
                    mean
                };

                // Compute Histogram using Freedman-Diaconis rule for adaptive binning
                let mut histogram = Vec::new();
                let mut final_bin_width = 1.0;
                if let (Some(min_v), Some(max_v), Some(q1_v), Some(q3_v)) = (min, max, q1, q3) {
                    let n = ca.len() - ca.null_count();
                    if n > 0 {
                        let iqr = q3_v - q1_v;
                        let mut bin_width = if iqr > 0.0 {
                            2.0 * iqr / (n as f64).powf(1.0 / 3.0)
                        } else {
                            (max_v - min_v).max(1.0) / 20.0
                        };

                        if bin_width <= 0.0 || bin_width.is_nan() {
                            bin_width = (max_v - min_v).max(1.0) / 20.0;
                        }
                        if bin_width <= 0.0 { bin_width = 1.0; }

                        if min_v < max_v {
                            let mut bin_counts: HashMap<i64, usize> = HashMap::new();
                            for val in ca.into_iter().flatten() {
                                let b = ((val - min_v) / bin_width).floor() as i64;
                                *bin_counts.entry(b).or_insert(0) += 1;
                            }

                            // If too many non-empty bins, merge them to keep UI responsive
                            while bin_counts.len() > 1000 {
                                bin_width *= 2.0;
                                let mut new_counts = HashMap::new();
                                for (b, count) in bin_counts {
                                    let new_b = if b >= 0 { b / 2 } else { (b - 1) / 2 };
                                    *new_counts.entry(new_b).or_insert(0) += count;
                                }
                                bin_counts = new_counts;
                            }

                            final_bin_width = bin_width;
                            for (b, count) in bin_counts {
                                let center = min_v + (b as f64 + 0.5) * bin_width;
                                histogram.push((center, count));
                            }
                            histogram.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                        } else {
                            // All values are the same
                            final_bin_width = 1.0 / 20.0;
                            for i in 0..20 {
                                let center = min_v - 0.5 + (i as f64 + 0.5) * final_bin_width;
                                let count = if i == 10 { n } else { 0 };
                                histogram.push((center, count));
                            }
                        }
                    }
                }

                (
                    ColumnKind::Numeric,
                    ColumnStats::Numeric(NumericStats {
                        min,
                        p05,
                        q1,
                        median,
                        mean,
                        trimmed_mean,
                        q3,
                        p95,
                        max,
                        std_dev,
                        skew,
                        bin_width: final_bin_width,
                        histogram,
                    }),
                )
            }
        } else if dtype.is_string() {
            let series = col.as_materialized_series();
            let ca = series.str()?;
            let distinct = ca.n_unique()?;

            has_special = ca.into_iter().any(|opt_s| {
                opt_s.map_or(false, |s| {
                    s.chars().any(|c| {
                        // Special if:
                        // 1. Not alphanumeric, not whitespace, and not common punctuation
                        // 2. OR it's a control character that isn't a standard newline or tab (like carriage return \r)
                        (!c.is_alphanumeric()
                            && !c.is_whitespace()
                            && !".,-_/:()!?;'\"".contains(c))
                            || (c.is_control() && c != '\n' && c != '\t')
                    })
                })
            });

            // Heuristic for Categorical vs Text
            if distinct <= 20 && (distinct as f64 / row_count as f64) < 0.5 {
                let mut freq = HashMap::new();
                let value_counts = series.value_counts(true, false, "counts".into(), false)?;
                let values = value_counts.column(&name)?.as_materialized_series();
                let counts = value_counts.column("counts")?.as_materialized_series();

                let val_ca = values.str()?;
                let count_ca = counts.u32()?;

                for i in 0..val_ca.len() {
                    if let (Some(v), Some(c)) = (val_ca.get(i), count_ca.get(i)) {
                        freq.insert(v.to_owned(), c as usize);
                    }
                }

                (ColumnKind::Categorical, ColumnStats::Categorical(freq))
            } else {
                let top_value = if distinct > 0 {
                    let value_counts = series.value_counts(true, false, "counts".into(), false)?;
                    let values = value_counts.column(&name)?.as_materialized_series();
                    let counts = value_counts.column("counts")?.as_materialized_series();

                    let v = values.str()?.get(0).map(|s| s.to_owned());
                    let c = counts.u32()?.get(0).map(|c| c as usize);

                    if let (Some(v_str), Some(c_val)) = (v, c) {
                        Some((v_str, c_val))
                    } else {
                        None
                    }
                } else {
                    None
                };

                (
                    ColumnKind::Text,
                    ColumnStats::Text(TextStats {
                        distinct,
                        top_value,
                    }),
                )
            }
        } else if dtype.is_temporal() {
            let series = col.as_materialized_series();
            let sorted = series.sort(SortOptions::default())?;
            let min_str = if sorted.len() > 0 {
                Some(sorted.get(0)?.to_string())
            } else {
                None
            };
            let max_str = if sorted.len() > 0 {
                Some(sorted.get(sorted.len() - 1)?.to_string())
            } else {
                None
            };

            // Compute Histogram for Temporal
            let mut histogram = Vec::new();
            let ts_series = series.cast(&DataType::Int64)?;
            let ts_ca = ts_series.i64()?;
            let min_ts = ts_ca.min();
            let max_ts = ts_ca.max();

            let mut final_bin_width = 1.0;
            let mut p05 = None;
            let mut p95 = None;
            if let (Some(min_v), Some(max_v)) = (min_ts, max_ts) {
                let n = ts_ca.len() - ts_ca.null_count();
                if n > 0 {
                    let q1 = ts_ca.quantile(0.25, QuantileMethod::Linear)?;
                    let q3 = ts_ca.quantile(0.75, QuantileMethod::Linear)?;
                    p05 = ts_ca.quantile(0.05, QuantileMethod::Linear)?;
                    p95 = ts_ca.quantile(0.95, QuantileMethod::Linear)?;

                    let mut bin_width = if let (Some(q1_v), Some(q3_v)) = (q1, q3) {
                        let iqr = q3_v - q1_v;
                        if iqr > 0.0 {
                            2.0 * iqr / (n as f64).powf(1.0 / 3.0)
                        } else {
                            (max_v - min_v).max(1) as f64 / 20.0
                        }
                    } else {
                        (max_v - min_v).max(1) as f64 / 20.0
                    };

                    if bin_width <= 0.0 || bin_width.is_nan() {
                        bin_width = (max_v - min_v).max(1) as f64 / 20.0;
                    }
                    if bin_width <= 0.0 { bin_width = 1.0; }

                    if min_v < max_v {
                        let mut bin_counts: HashMap<i64, usize> = HashMap::new();
                        for val in ts_ca.into_iter().flatten() {
                            let b = ((val - min_v) as f64 / bin_width).floor() as i64;
                            *bin_counts.entry(b).or_insert(0) += 1;
                        }

                        while bin_counts.len() > 1000 {
                            bin_width *= 2.0;
                            let mut new_counts = HashMap::new();
                            for (b, count) in bin_counts {
                                let new_b = if b >= 0 { b / 2 } else { (b - 1) / 2 };
                                *new_counts.entry(new_b).or_insert(0) += count;
                            }
                            bin_counts = new_counts;
                        }

                        final_bin_width = bin_width;
                        for (b, count) in bin_counts {
                            let center = min_v as f64 + (b as f64 + 0.5) * bin_width;
                            histogram.push((center, count));
                        }
                        histogram.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    } else {
                        final_bin_width = 1.0;
                        histogram.push((min_v as f64, n));
                    }
                }
            }

            (
                ColumnKind::Temporal,
                ColumnStats::Temporal(TemporalStats {
                    min: min_str,
                    max: max_str,
                    p05,
                    p95,
                    bin_width: final_bin_width,
                    histogram,
                }),
            )
        } else {
            // Default fallback for other types (including Nested like List/Struct)
            let series = col.as_materialized_series();
            let kind = if matches!(dtype, DataType::List(_) | DataType::Struct(_)) {
                ColumnKind::Nested
            } else {
                ColumnKind::Text
            };

            let distinct = series.n_unique()?;
            let top_value = if distinct > 0 {
                let value_counts = series.value_counts(true, false, "counts".into(), false)?;
                let values = value_counts.column(&name)?.as_materialized_series();
                let counts = value_counts.column("counts")?.as_materialized_series();

                // Try to get a string representation for the top value
                let v = values
                    .cast(&DataType::String)
                    .ok()
                    .and_then(|s| s.str().ok().and_then(|ca| ca.get(0).map(|s| s.to_owned())));
                let c = counts
                    .u32()
                    .ok()
                    .and_then(|ca| ca.get(0).map(|c| c as usize));

                if let (Some(v_str), Some(c_val)) = (v, c) {
                    Some((v_str, c_val))
                } else {
                    None
                }
            } else {
                None
            };

            (
                kind,
                ColumnStats::Text(TextStats {
                    distinct,
                    top_value,
                }),
            )
        };

        let mut summary = ColumnSummary {
            name,
            kind,
            count,
            nulls,
            has_special,
            stats,
            interpretation: String::new(),
            business_summary: String::new(),
            samples,
        };
        summary.interpretation = summary.generate_interpretation();
        summary.business_summary = summary.generate_business_summary();
        summaries.push(summary);
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carriage_return_detection() -> Result<()> {
        let s = Series::new("col".into(), vec!["line1\r\nline2"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        assert!(summaries[0].has_special, "Should detect \r as special");
        Ok(())
    }

    #[test]
    fn test_normal_whitespace_not_special() -> Result<()> {
        let s = Series::new("col".into(), vec!["line1\nline2\twith spaces"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        assert!(
            !summaries[0].has_special,
            "Standard whitespace (\\n, \\t, space) should NOT be special"
        );
        Ok(())
    }

    #[test]
    fn test_histogram_calculation() -> Result<()> {
        let s = Series::new("col".into(), vec![1.0, 1.0, 2.0, 3.0, 10.0]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;

        if let ColumnStats::Numeric(stats) = &summaries[0].stats {
            assert!(!stats.histogram.is_empty(), "Histogram should not be empty");
            // Check that the sum of counts matches the number of non-null elements
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
        let summaries = analyse_df(df, 0.0)?;

        if let ColumnStats::Numeric(stats) = &summaries[0].stats {
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
        let summaries = analyse_df(df, 0.0)?;
        assert!(
            summaries[0].interpretation.contains("unique identifier"),
            "Should detect unique identifier"
        );

        let s2 = Series::new("age".into(), vec![Some(25.0), Some(30.0), None]);
        let df2 = DataFrame::new(vec![Column::from(s2)])?;
        let summaries2 = analyse_df(df2, 0.0)?;
        assert!(
            summaries2[0].interpretation.contains("missing data"),
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
        let summaries = analyse_df(df, 0.0)?;

        assert_eq!(summaries[0].kind.as_str(), "Boolean");
        if let ColumnStats::Boolean(stats) = &summaries[0].stats {
            assert_eq!(stats.true_count, 2);
            assert_eq!(stats.false_count, 1);
        } else {
            panic!("Expected BooleanStats");
        }
        assert!(
            summaries[0].interpretation.contains("Binary field"),
            "Should detect binary signal"
        );
        Ok(())
    }

    #[test]
    fn test_effective_boolean_detection() -> Result<()> {
        let s = Series::new("is_active".into(), vec![Some(1), Some(0), None, Some(1)]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;

        // This is what we WANT it to be
        assert_eq!(summaries[0].kind.as_str(), "Boolean");
        if let ColumnStats::Boolean(stats) = &summaries[0].stats {
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
        let summaries = analyse_df(df, 0.0)?;

        if let ColumnStats::Numeric(stats) = &summaries[0].stats {
            // Skewness should be positive
            assert!(stats.skew.unwrap() > 0.1, "Should detect right skew");
            assert!(summaries[0].interpretation.contains("Right-skewed"), "Should report right skew");

            // Histogram should have multiple bins for the central mass
            // Outlier is at 1000, max is 1000, min is 1.0.
            // IQR is roughly 3.0 - 1.5 = 1.5. n=11.
            // bin_width = 2 * 1.5 / 11^(1/3) approx 3 / 2.2 = 1.36.
            // Previous logic would have bin_width = (1000 - 1) / 500 = 2.0.
            // All central values (1 to 4) would fall into the first 2 bins.
            // With my fix, bin_width should be based on IQR.
            assert!(stats.bin_width < 2.0, "Bin width should be small based on IQR, not large based on range");
            assert!(stats.histogram.len() > 2, "Should have more than 2 bins");
        } else {
            panic!("Expected NumericStats");
        }
        Ok(())
    }

    #[test]
    fn test_trimmed_mean() -> Result<()> {
        // Values: 0, 10, 20, 30, 100
        // Mean = 160 / 5 = 32
        // Trim 20% from each end (1 element each) -> 10, 20, 30
        // Trimmed Mean = 60 / 3 = 20
        let s = Series::new("col".into(), vec![0.0, 10.0, 20.0, 30.0, 100.0]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.2)?;

        if let ColumnStats::Numeric(stats) = &summaries[0].stats {
            assert_eq!(stats.mean, Some(32.0));
            assert_eq!(stats.trimmed_mean, Some(20.0));
        } else {
            panic!("Expected NumericStats");
        }
        Ok(())
    }

    #[test]
    fn test_temporal_histogram() -> Result<()> {
        // Create some timestamps
        let base = 1_700_000_000_000_i64; // Approx 2023
        let vals = vec![
            Some(base), 
            Some(base + 1000), 
            Some(base + 2000), 
            Some(base + 3000), 
            Some(base + 1_000_000) // Outlier
        ];
        let s = Series::new("ts".into(), vals).cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?;
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;

        if let ColumnStats::Temporal(stats) = &summaries[0].stats {
            assert!(!stats.histogram.is_empty());
            // IQR should be roughly (base+3000) - (base+1000) = 2000
            // bin_width should be based on that, not on 1,000,000/20
            assert!(stats.bin_width < 10000.0, "Bin width should be small based on IQR");
            assert!(stats.histogram.len() > 2, "Should have multiple bins for central mass");
        } else {
            panic!("Expected TemporalStats");
        }
        Ok(())
    }

    #[test]
    fn test_interpretation_histogram_signals() -> Result<()> {
        // Case 1: Extreme outliers (skinny bars and gaps)
        let mut vals = (0..100).map(|i| i as f64 * 0.1).collect::<Vec<_>>(); // 0.0 to 9.9
        vals.push(10000.0); // Extreme outlier
        let s = Series::new("outliers".into(), vals);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        let interp = &summaries[0].interpretation;
        
        assert!(interp.contains("Histogram bars appear very skinny"), "Should report skinny bars. Interp: {}", interp);
        assert!(interp.contains("Gaps between bars"), "Should report gaps. Interp: {}", interp);

        // Case 2: Highly concentrated data (low Gaussian curve)
        // A lot of 5.0, few others
        let mut vals = vec![5.0; 100];
        vals.push(1.0);
        vals.push(9.0);
        let s = Series::new("concentrated".into(), vals);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        let interp = &summaries[0].interpretation;
        
        assert!(interp.contains("orange distribution curve is low"), "Should report low curve. Interp: {}", interp);
        
        Ok(())
    }

    #[test]
    fn test_user_reported_uniform_sequence_not_skinny() -> Result<()> {
        // Uniform sequence
        // n^(1/3) for 2,000,000 is ~126, which is > 100
        let vals: Vec<f64> = (1..=2_000_000).map(|i| i as f64).collect(); 
        let s = Series::new("order_id".into(), vals);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        let interp = &summaries[0].interpretation;

        println!("Interp: {}", interp);

        // It should NOT contain "very skinny"
        assert!(!interp.contains("Histogram bars appear very skinny"), "Should not report skinny bars for uniform sequence. Interp: {}", interp);
        
        Ok(())
    }

    #[test]
    fn test_user_reported_delivery_minutes_zoom() -> Result<()> {
        // Range 20 to 530. Mean 45.9. Median 35.0. IQR 20.
        // n=100. 95 elements from 20 to 100. 5 elements from 100 to 530.
        let mut vals = Vec::new();
        for i in 0..95 {
            vals.push(20.0 + i as f64 * 0.84); // 20 to ~99.8
        }
        for i in 0..5 {
            vals.push(100.0 + i as f64 * 107.5); // 100 to 530
        }
        
        let s = Series::new("delivery_minutes".into(), vals);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        let interp = &summaries[0].interpretation;
        
        println!("Interp: {}", interp);
        
        // With 3x threshold, it SHOULD detect skinny bars because of outliers
        assert!(interp.contains("stretching the scale"), "Should detect skinny bars with 3x threshold. Interp: {}", interp);
        
        Ok(())
    }

    #[test]
    fn test_std_dev_reliability_signal() -> Result<()> {
        // Highly skewed data: many 1s, one 1000
        let mut vals = vec![1.0; 100];
        vals.push(1000.0);
        
        let s = Series::new("skewed".into(), vals);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        let interp = &summaries[0].interpretation;
        
        println!("Interp: {}", interp);
        
        // Nonparametric skew = (mean - median) / std_dev
        // Mean approx 10.9
        // Median = 1.0
        // Std Dev approx 99
        // (10.9 - 1) / 99 approx 0.1 ... wait, maybe not high enough for 0.3 threshold.
        
        // Let's make it more extreme
        // 10 values: [1, 1, 1, 1, 1, 1, 1, 1, 1, 100]
        // Mean = 109 / 10 = 10.9
        // Median = 1
        // Std Dev = sqrt( sum(x-mean)^2 / 9 )
        // (1-10.9)^2 * 9 + (100-10.9)^2 = (-9.9)^2 * 9 + (89.1)^2 = 98.01 * 9 + 7938.81 = 882.09 + 7938.81 = 8820.9
        // Std Dev = sqrt(8820.9 / 9) = sqrt(980.1) = 31.3
        // (10.9 - 1) / 31.3 = 9.9 / 31.3 approx 0.316.
        // This should trigger it!
        
        let mut vals2 = vec![1.0; 9];
        vals2.push(100.0);
        let s2 = Series::new("skewed2".into(), vals2);
        let df2 = DataFrame::new(vec![Column::from(s2)])?;
        let summaries2 = analyse_df(df2, 0.0)?;
        let interp2 = &summaries2[0].interpretation;
        
        println!("Interp2: {}", interp2);
        assert!(interp2.contains("Standard deviation may be less reliable"), "Should detect unreliable std dev. Interp: {}", interp2);
        
        Ok(())
    }

    #[test]
    fn test_business_summary_generation() -> Result<()> {
        // Case 1: Perfect Unique ID
        let s = Series::new("id".into(), vec!["1", "2", "3"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df, 0.0)?;
        assert!(
            summaries[0].business_summary.contains("unique tracking number"),
            "Should identify unique ID in business terms. Got: {}", summaries[0].business_summary
        );

        // Case 2: Skewed Numeric Data
        let mut vals = vec![1.0; 10];
        vals.push(1000.0); // Extreme Outlier
        let s2 = Series::new("sales".into(), vals);
        let df2 = DataFrame::new(vec![Column::from(s2)])?;
        let summaries2 = analyse_df(df2, 0.0)?;
        let biz = &summaries2[0].business_summary;
        assert!(biz.contains("high-value exceptions"), "Should mention high-value exceptions. Got: {}", biz);
        assert!(biz.contains("heavily distorted"), "Should mention distorted average. Got: {}", biz);

        // Case 3: Missing Data
        let s3 = Series::new("notes".into(), vec![Some("A"), None, None]);
        let df3 = DataFrame::new(vec![Column::from(s3)])?;
        let summaries3 = analyse_df(df3, 0.0)?;
        assert!(
            summaries3[0].business_summary.contains("significant amount of information is missing"),
            "Should warn about missing data. Got: {}", summaries3[0].business_summary
        );

        Ok(())
    }
}
