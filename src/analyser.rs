use anyhow::{Context, Result, anyhow};
use csv::StringRecord;
use eframe::egui;
use std::{collections::HashMap, path::Path};

pub fn run_analyser(file_path: Option<String>) -> App {
    App::new(file_path)
}

#[derive(Clone)]
struct ColumnSummary {
    name: String,
    kind: ColumnKind,
    count: usize,
    nulls: usize,
    stats: ColumnStats,
}

#[derive(Clone)]
enum ColumnStats {
    Numeric(NumericStats),
    Text(TextStats),
    Categorical(HashMap<String, usize>),
}

#[derive(Clone)]
struct NumericStats {
    min: Option<f64>,
    q1: Option<f64>,
    median: Option<f64>,
    mean: Option<f64>,
    q3: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone)]
struct TextStats {
    distinct: usize,
    top_value: Option<(String, usize)>,
}

#[derive(Clone, Copy)]
enum ColumnKind {
    Numeric,
    Text,
    Categorical,
}

impl ColumnKind {
    fn as_str(&self) -> &'static str {
        match self {
            ColumnKind::Numeric => "Numeric",
            ColumnKind::Text => "Text",
            ColumnKind::Categorical => "Categorical",
        }
    }
}
#[derive(Default)]
pub struct App {
    file_path: Option<String>,
    status: String,
    summary: Vec<ColumnSummary>,
}

impl App {
    pub fn new(file_path: Option<String>) -> Self {
        let mut app = Self::default();

        if let Some(path) = file_path {
            app.load_csv_from_path(path);
        } else {
            app.status = "Select a CSV to see its summary.".to_string();
        }

        app
    }

    fn load_csv_from_path(&mut self, path: String) {
        match summarise_csv(Path::new(&path)) {
            Ok(summary) => self.set_loaded_csv(path, summary),
            Err(e) => self.status = format!("Error: {e:#}"),
        }
    }

    fn set_loaded_csv(&mut self, path: String, summary: Vec<ColumnSummary>) {
        self.file_path = Some(path);
        self.summary = summary;
        self.status = "Loaded and summarised.".to_string();
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;

        egui::TopBottomPanel::top("top_analyser").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();

                if ui.button("Open CSV...").clicked() {
                    match pick_and_summarise_csv() {
                        Ok((path, summary)) => {
                            self.set_loaded_csv(path, summary);
                        }
                        Err(e) => {
                            self.status = format!("Error: {e:#}");
                        }
                    }
                }

                if let Some(p) = &self.file_path {
                    ui.label(format!("File: {p}"));
                }
            });

            if !self.status.is_empty() {
                ui.separator();
                ui.label(&self.status);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Summary");

            if self.summary.is_empty() {
                ui.label("Open a CSV to see per-column statistics (like R's summary()).");
                return;
            }

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(egui_extras::Column::auto(), 10)
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Column");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("Count");
                            });
                            header.col(|ui| {
                                ui.strong("Nulls");
                            });
                            header.col(|ui| {
                                ui.strong("Min");
                            });
                            header.col(|ui| {
                                ui.strong("Q1");
                            });
                            header.col(|ui| {
                                ui.strong("Median");
                            });
                            header.col(|ui| {
                                ui.strong("Mean");
                            });
                            header.col(|ui| {
                                ui.strong("Q3");
                            });
                            header.col(|ui| {
                                ui.strong("Max / Top");
                            });
                        })
                        .body(|mut body| {
                            for col in &self.summary {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&col.name);
                                    });
                                    row.col(|ui| {
                                        if let ColumnStats::Text(s) = &col.stats {
                                            ui.label(format!("Text ({})", s.distinct));
                                        } else {
                                            ui.label(col.kind.as_str());
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.label(col.count.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(col.nulls.to_string());
                                    });

                                    match &col.stats {
                                        ColumnStats::Numeric(s) => {
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.min));
                                            });
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.q1));
                                            });
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.median));
                                            });
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.mean));
                                            });
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.q3));
                                            });
                                            row.col(|ui| {
                                                ui.label(fmt_opt(s.max));
                                            });
                                        }
                                        ColumnStats::Categorical(freq) => {
                                            for _ in 0..5 {
                                                row.col(|ui| {
                                                    ui.label("—");
                                                });
                                            }
                                            row.col(|ui| {
                                                let top = freq
                                                    .iter()
                                                    .max_by_key(|(_k, v)| *v)
                                                    .map(|(k, v)| format!("{k} ({v})"))
                                                    .unwrap_or_else(|| "—".to_string());
                                                ui.label(top);
                                            });
                                        }
                                        ColumnStats::Text(s) => {
                                            for _ in 0..5 {
                                                row.col(|ui| {
                                                    ui.label("—");
                                                });
                                            }
                                            row.col(|ui| {
                                                let top = s
                                                    .top_value
                                                    .as_ref()
                                                    .map(|(v, n)| format!("{v} ({n})"))
                                                    .unwrap_or_else(|| "—".to_string());
                                                ui.label(top);
                                            });
                                        }
                                    }
                                })
                            }
                        });
                });
        });

        go_back
    }
}

fn pick_and_summarise_csv() -> Result<(String, Vec<ColumnSummary>)> {
    let file = rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .pick_file()
        .ok_or_else(|| anyhow!("No file selected"))?;

    let path = file.display().to_string();
    let summary = summarise_csv(&file)?;
    Ok((path, summary))
}

fn summarise_csv(path: &std::path::Path) -> Result<Vec<ColumnSummary>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("Failed to open CSV: {}", path.display()))?;

    let headers = rdr.headers().context("Failed to read headers")?.clone();

    let col_count = headers.len();
    if col_count == 0 {
        return Err(anyhow!("CSV has no columns"));
    }

    // Collect raw strings per column (Option<String> where empty -> None)
    let mut cols: Vec<Vec<Option<String>>> = vec![Vec::new(); col_count];

    for record in rdr.records() {
        let rec = record.context("Failed to read a CSV record")?;
        push_record(&mut cols, &rec, col_count)?;
    }

    // Build summaries
    let mut out = Vec::with_capacity(col_count);
    for (i, name) in headers.iter().enumerate() {
        let values = &cols[i];
        let nulls = values.iter().filter(|v| v.is_none()).count();
        let count = values.len();

        let kind = infer_kind(values);

        let summary = match kind {
            ColumnKind::Numeric => {
                let nums: Vec<f64> = values
                    .iter()
                    .filter_map(|v| v.as_ref())
                    .filter_map(|s| parse_number(s))
                    .collect();

                let stats = numeric_stats(&nums);

                ColumnSummary {
                    name: name.to_string(),
                    kind,
                    count,
                    nulls,
                    stats: ColumnStats::Numeric(stats),
                }
            }
            ColumnKind::Text => {
                let texts: Vec<&str> = values
                    .iter()
                    .filter_map(|v| v.as_deref())
                    .filter(|s| !s.trim().is_empty())
                    .collect();

                let stats = text_stats(&texts);

                ColumnSummary {
                    name: name.to_string(),
                    kind,
                    count,
                    nulls,
                    stats: ColumnStats::Text(stats),
                }
            }
            ColumnKind::Categorical => {
                let mut freq: HashMap<String, usize> = HashMap::new();
                for v in values.iter().filter_map(|v| v.as_deref()) {
                    let s = v.trim();
                    if !s.is_empty() {
                        *freq.entry(s.to_string()).or_insert(0) += 1;
                    }
                }

                ColumnSummary {
                    name: name.to_string(),
                    kind,
                    count,
                    nulls,
                    stats: ColumnStats::Categorical(freq),
                }
            }
        };

        out.push(summary);
    }

    Ok(out)
}

fn push_record(
    cols: &mut [Vec<Option<String>>],
    rec: &StringRecord,
    col_count: usize,
) -> Result<()> {
    // Handle ragged rows by treating missing fields as null.
    for i in 0..col_count {
        let v = rec.get(i).unwrap_or("");
        let v = v.trim();
        if v.is_empty() {
            cols[i].push(None);
        } else {
            cols[i].push(Some(v.to_string()));
        }
    }
    Ok(())
}

fn infer_kind(values: &[Option<String>]) -> ColumnKind {
    // Simple heuristic:
    // - consider non-null values
    // - if >= 90% parse as numbers -> Numeric
    // - else if distinct values <= 20 and < 50% of total -> Categorical
    // - else Text
    let mut non_null = 0usize;
    let mut numeric = 0usize;
    let mut distinct_values: std::collections::HashSet<String> = std::collections::HashSet::new();

    for v in values.iter().filter_map(|v| v.as_deref()) {
        let s = v.trim();
        if s.is_empty() {
            continue;
        }
        non_null += 1;
        distinct_values.insert(s.to_string());
        if parse_number(s).is_some() {
            numeric += 1;
        }
    }

    if non_null == 0 {
        return ColumnKind::Text; // empty column, treat as text
    }

    let ratio = numeric as f64 / non_null as f64;
    if ratio >= 0.9 {
        return ColumnKind::Numeric;
    }

    let distinct_count = distinct_values.len();
    let distinct_ratio = distinct_count as f64 / non_null as f64;

    if distinct_count <= 20 && distinct_ratio < 0.5 {
        ColumnKind::Categorical
    } else {
        ColumnKind::Text
    }
}

fn parse_number(s: &str) -> Option<f64> {
    // Basic numeric parsing: allow commas in thousands (e.g. "1,234.56")
    let cleaned = s.replace(',', "");
    cleaned.parse::<f64>().ok()
}

fn numeric_stats(nums: &[f64]) -> NumericStats {
    if nums.is_empty() {
        return NumericStats {
            min: None,
            q1: None,
            median: None,
            mean: None,
            q3: None,
            max: None,
        };
    }

    let mut sorted = nums.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = sorted.first().copied();
    let max = sorted.last().copied();
    let mean = Some(sorted.iter().sum::<f64>() / sorted.len() as f64);

    let q1 = Some(quantile(&sorted, 0.25));
    let median = Some(quantile(&sorted, 0.50));
    let q3 = Some(quantile(&sorted, 0.75));

    NumericStats {
        min,
        q1,
        median,
        mean,
        q3,
        max,
    }
}

// Linear interpolation quantile (good enough for v1)
fn quantile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let pos = (n - 1) as f64 * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let w = pos - lo as f64;
        sorted[lo] * (1.0 - w) + sorted[hi] * w
    }
}

fn text_stats(values: &[&str]) -> TextStats {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for &v in values {
        *freq.entry(v.to_string()).or_insert(0) += 1;
    }

    let distinct = freq.len();
    let top_value = freq.into_iter().max_by_key(|(_k, v)| *v);

    TextStats {
        distinct,
        top_value,
    }
}
fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{:.4}", x),
        _ => "—".to_string(),
    }
}
