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
                    if range > 0.0 {
                        let diff_ratio = (mean - median).abs() / range;
                        if diff_ratio < 0.02 {
                            signals.push("Symmetric distribution.");
                        } else if mean > median {
                            signals.push("Right-skewed; average is influenced by high outliers.");
                        } else {
                            signals.push("Left-skewed; average is influenced by low values.");
                        }
                    }

                    // Variability
                    if range > 0.0 {
                        if iqr / range < 0.1 {
                            signals.push("Values are tightly clustered around the center.");
                        } else if iqr / range > 0.6 {
                            signals.push("High variability across the range.");
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
                    if let (Some(max_v), Some(min_v)) = (
                        freq.values().max(),
                        freq.values().min(),
                    ) {
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
    pub histogram: Vec<(i64, usize)>, // timestamp (ms) and count
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NumericStats {
    pub min: Option<f64>,
    pub q1: Option<f64>,
    pub median: Option<f64>,
    pub mean: Option<f64>,
    pub q3: Option<f64>,
    pub max: Option<f64>,
    pub std_dev: Option<f64>,
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
    pub risks: Vec<String>,
}

pub fn calculate_file_health(summaries: &[ColumnSummary]) -> FileHealth {
    let mut risks = Vec::new();
    for col in summaries {
        let null_pct = if col.count > 0 {
            (col.nulls as f64 / col.count as f64) * 100.0
        } else {
            0.0
        };

        if null_pct > 15.0 {
            risks.push(format!("⚠️ Column '{}' has significant missing data ({:.1}%).", col.name, null_pct));
        }
        if col.has_special {
            risks.push(format!("🔍 Hidden/special characters detected in '{}'.", col.name));
        }
        
        match &col.stats {
            ColumnStats::Numeric(s) => {
                if let (Some(mean), Some(median), Some(min), Some(max)) = (s.mean, s.median, s.min, s.max) {
                    let range = max - min;
                    if range > 0.0 {
                        let diff_ratio = (mean - median).abs() / range;
                        if diff_ratio > 0.1 {
                             risks.push(format!("📈 Column '{}' is heavily skewed; averages may be misleading.", col.name));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    FileHealth { risks }
}

pub type AnalysisReceiver =
    crossbeam_channel::Receiver<Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration)>>;

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

pub fn analyse_df(df: DataFrame) -> Result<Vec<ColumnSummary>> {
    let row_count = df.height();
    let mut summaries = Vec::new();

    for col in df.get_columns() {
        let name = col.name().to_string();
        let nulls = col.null_count();
        let count = row_count;

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

                // Compute Histogram
                let mut histogram = Vec::new();
                if let (Some(min_v), Some(max_v)) = (min, max) {
                    let bins = 20;
                    if min_v < max_v {
                        let bin_width = (max_v - min_v) / bins as f64;
                        let mut counts = vec![0; bins];
                        for val in ca.into_iter().flatten() {
                            let mut b = ((val - min_v) / bin_width).floor() as usize;
                            if b >= bins {
                                b = bins - 1;
                            }
                            counts[b] += 1;
                        }
                        for (i, count) in counts.into_iter().enumerate() {
                            let center = min_v + (i as f64 + 0.5) * bin_width;
                            histogram.push((center, count));
                        }
                    } else {
                        // All values are the same, put them in the middle
                        for i in 0..bins {
                            let count = if i == bins / 2 { ca.len() } else { 0 };
                            histogram.push((min_v, count));
                        }
                    }
                }

                (
                    ColumnKind::Numeric,
                    ColumnStats::Numeric(NumericStats {
                        min,
                        q1,
                        median,
                        mean,
                        q3,
                        max,
                        std_dev,
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
                        (!c.is_alphanumeric() && !c.is_whitespace() && !".,-_/:()!?;'\"".contains(c))
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

            if let (Some(min_v), Some(max_v)) = (min_ts, max_ts) {
                let bins = 20;
                if min_v < max_v {
                    let bin_width = (max_v - min_v) as f64 / bins as f64;
                    let mut counts = vec![0; bins];
                    for val in ts_ca.into_iter().flatten() {
                        let mut b = ((val - min_v) as f64 / bin_width).floor() as usize;
                        if b >= bins {
                            b = bins - 1;
                        }
                        counts[b] += 1;
                    }
                    for (i, count) in counts.into_iter().enumerate() {
                        let center = min_v + ((i as f64 + 0.5) * bin_width) as i64;
                        histogram.push((center, count));
                    }
                } else {
                    // All values are the same
                    for i in 0..bins {
                        let count = if i == bins / 2 { ts_ca.len() } else { 0 };
                        histogram.push((min_v, count));
                    }
                }
            }

            (
                ColumnKind::Temporal,
                ColumnStats::Temporal(TemporalStats {
                    min: min_str,
                    max: max_str,
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
                let v = values.cast(&DataType::String).ok().and_then(|s| {
                    s.str()
                        .ok()
                        .and_then(|ca| ca.get(0).map(|s| s.to_owned()))
                });
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
        };
        summary.interpretation = summary.generate_interpretation();
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
        let summaries = analyse_df(df)?;
        assert!(summaries[0].has_special, "Should detect \r as special");
        Ok(())
    }

    #[test]
    fn test_normal_whitespace_not_special() -> Result<()> {
        let s = Series::new("col".into(), vec!["line1\nline2\twith spaces"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df)?;
        assert!(!summaries[0].has_special, "Standard whitespace (\\n, \\t, space) should NOT be special");
        Ok(())
    }

    #[test]
    fn test_histogram_calculation() -> Result<()> {
        let s = Series::new("col".into(), vec![1.0, 1.0, 2.0, 3.0, 10.0]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df)?;
        
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
        let summaries = analyse_df(df)?;
        
        if let ColumnStats::Numeric(stats) = &summaries[0].stats {
            assert_eq!(stats.histogram.len(), 20, "Should have 20 bins even for single value");
            let total_count: usize = stats.histogram.iter().map(|h| h.1).sum();
            assert_eq!(total_count, 3);
            assert_eq!(stats.histogram[10].1, 3, "Middle bin should have all counts");
        } else {
            panic!("Expected NumericStats");
        }
        Ok(())
    }

    #[test]
    fn test_interpretation_generation() -> Result<()> {
        let s = Series::new("id".into(), vec!["1", "2", "3"]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df)?;
        assert!(summaries[0].interpretation.contains("unique identifier"), "Should detect unique identifier");

        let s2 = Series::new("age".into(), vec![Some(25.0), Some(30.0), None]);
        let df2 = DataFrame::new(vec![Column::from(s2)])?;
        let summaries2 = analyse_df(df2)?;
        assert!(summaries2[0].interpretation.contains("missing data"), "Should detect nulls");
        
        Ok(())
    }

    #[test]
    fn test_boolean_detection() -> Result<()> {
        let s = Series::new("bool_col".into(), vec![Some(true), Some(false), None, Some(true)]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df)?;
        
        assert_eq!(summaries[0].kind.as_str(), "Boolean");
        if let ColumnStats::Boolean(stats) = &summaries[0].stats {
            assert_eq!(stats.true_count, 2);
            assert_eq!(stats.false_count, 1);
        } else {
            panic!("Expected BooleanStats");
        }
        assert!(summaries[0].interpretation.contains("Binary field"), "Should detect binary signal");
        Ok(())
    }

    #[test]
    fn test_effective_boolean_detection() -> Result<()> {
        let s = Series::new("is_active".into(), vec![Some(1), Some(0), None, Some(1)]);
        let df = DataFrame::new(vec![Column::from(s)])?;
        let summaries = analyse_df(df)?;
        
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
}
