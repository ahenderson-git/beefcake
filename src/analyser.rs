use anyhow::Result;
use eframe::egui;
use polars::prelude::*;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn run_analyser() -> App {
    App::default()
}

// DATA STRUCTURES

#[derive(Clone, Deserialize, Serialize)]
struct ColumnSummary {
    name: String,
    kind: ColumnKind,
    count: usize,
    nulls: usize,
    stats: ColumnStats,
}

#[derive(Clone, Deserialize, Serialize)]
enum ColumnStats {
    Numeric(NumericStats),
    Text(TextStats),
    Categorical(HashMap<String, usize>),
}

#[derive(Clone, Deserialize, Serialize)]
struct NumericStats {
    min: Option<f64>,
    q1: Option<f64>,
    median: Option<f64>,
    mean: Option<f64>,
    q3: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone, Deserialize, Serialize)]
struct TextStats {
    distinct: usize,
    top_value: Option<(String, usize)>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
enum ColumnKind {
    Numeric,
    Text,
    Categorical,
}

impl ColumnKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Numeric => "Numeric",
            Self::Text => "Text",
            Self::Categorical => "Categorical",
        }
    }
}

// MAIN APP STATE
type AnalysisReceiver = crossbeam_channel::Receiver<Result<(String, u64, Vec<ColumnSummary>, std::time::Duration)>>;

#[derive(Default, Deserialize, Serialize)]
pub struct App {
    file_path: Option<String>,
    file_size: u64,
    status: String,
    summary: Vec<ColumnSummary>,
    #[serde(skip)]
    is_loading: bool,
    #[serde(skip)]
    progress_counter: Arc<AtomicU64>,
    #[serde(skip)]
    receiver: Option<AnalysisReceiver>, 
    #[serde(skip)]
    start_time: Option<std::time::Instant>, // To track active time
    last_duration: Option<std::time::Duration>, // To show final time
}

impl App {
    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;

        if let Some(rx) = &self.receiver {
            if let Ok(result) = rx.try_recv() {
                self.is_loading = false;
                self.receiver = None;
                self.start_time = None; // Reset timer
                match result {
                    Ok((path, size, summary, duration)) => {
                        self.file_path = Some(path);
                        self.file_size = size;
                        self.summary = summary;
                        self.last_duration = Some(duration); // Store final time
                        self.status = "Loaded successfully".into();
                    }
                    Err(e) => self.status = format!("Error: {e:#}"),
                }
            }
        }

        egui::TopBottomPanel::top("top_analyser").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();

                ui.add_enabled_ui(!self.is_loading, |ui| {
                    if ui.button("Open CSV...").clicked() {
                        self.start_analysis(ctx.clone());
                    }
                });

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
            if self.is_loading {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    let bytes_read = self.progress_counter.load(Ordering::Relaxed);
                    let progress = if self.file_size > 0 {
                        bytes_read as f32 / self.file_size as f32
                    } else { 0.0 };

                    ui.heading("Analysing records...");
                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                    
                    // Show live timer
                    if let Some(start) = self.start_time {
                        ui.label(format!("Elapsed: {:.1}s", start.elapsed().as_secs_f32()));
                    }

                    ui.label(format!("{:.2} MB / {:.2} MB", bytes_read as f64 / 1e6, self.file_size as f64 / 1e6));
                    ctx.request_repaint();
                });
                return;
            }

            ui.horizontal(|ui| {
                ui.heading("Summary");
                if let Some(duration) = self.last_duration {
                    ui.label(format!("(Processed in {:.2}s)", duration.as_secs_f32()));
                }
            });

            if self.summary.is_empty() {
                ui.label("Open a CSV to see per-column statistics.");
                return;
            }

            ui.collapsing("📄 File Metadata", |ui| {
                egui::Grid::new("file_info_grid").num_columns(2).spacing([40.0, 4.0]).show(ui, |ui| {
                    ui.label("Path:");
                    ui.label(self.file_path.as_deref().unwrap_or("Unknown"));
                    ui.end_row();

                    ui.label("File Size:");
                    ui.label(format!("{:.2} MB", self.file_size as f64 / 1_048_576.0));
                    ui.end_row();

                    ui.label("Records:");
                    ui.label(self.summary.first().map(|s| s.count.to_string()).unwrap_or_else(|| "0".to_owned()));
                    ui.end_row();

                    ui.label("Columns:");
                    ui.label(self.summary.len().to_string());
                    ui.end_row();
                });
            });
            ui.separator();

            egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(egui_extras::Column::auto(), 10)
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.strong("Column"); });
                        header.col(|ui| { ui.strong("Type"); });
                        header.col(|ui| { ui.strong("Count"); });
                        header.col(|ui| { ui.strong("Nulls"); });
                        header.col(|ui| { ui.strong("Min"); });
                        header.col(|ui| { ui.strong("Q1"); });
                        header.col(|ui| { ui.strong("Median"); });
                        header.col(|ui| { ui.strong("Mean"); });
                        header.col(|ui| { ui.strong("Q3"); });
                        header.col(|ui| { ui.strong("Max / Top"); });
                    })
                    .body(|mut body| {
                        for col in &self.summary {
                            body.row(18.0, |mut row| {
                                row.col(|ui| { ui.label(&col.name); });
                                row.col(|ui| {
                                    if let ColumnStats::Text(s) = &col.stats {
                                        ui.label(format!("Text ({})", s.distinct));
                                    } else {
                                        ui.label(col.kind.as_str());
                                    }
                                });
                                row.col(|ui| { ui.label(col.count.to_string()); });
                                row.col(|ui| { ui.label(col.nulls.to_string()); });

                                match &col.stats {
                                    ColumnStats::Numeric(s) => {
                                        row.col(|ui| { ui.label(fmt_opt(s.min)); });
                                        row.col(|ui| { ui.label(fmt_opt(s.q1)); });
                                        row.col(|ui| { ui.label(fmt_opt(s.median)); });
                                        row.col(|ui| { ui.label(fmt_opt(s.mean)); });
                                        row.col(|ui| { ui.label(fmt_opt(s.q3)); });
                                        row.col(|ui| { ui.label(fmt_opt(s.max)); });
                                    }
                                    ColumnStats::Categorical(freq) => {
                                        for _ in 0..5 { row.col(|ui| { ui.label("—"); }); }
                                        row.col(|ui| {
                                            let top = freq.iter()
                                                .max_by_key(|(_k, v)| *v)
                                                .map(|(k, v)| format!("{k} ({v})"))
                                                .unwrap_or_else(|| "—".to_owned());
                                            ui.label(top);
                                        });
                                    }
                                    ColumnStats::Text(s) => {
                                        for _ in 0..5 { row.col(|ui| { ui.label("—"); }); }
                                        row.col(|ui| {
                                            let top = s.top_value.as_ref()
                                                .map(|(v, n)| format!("{v} ({n})"))
                                                .unwrap_or_else(|| "—".to_owned());
                                            ui.label(top);
                                        });
                                    }
                                }
                            });
                        }
                    });
            });
        });

        go_back
    }

    fn start_analysis(&mut self, ctx: egui::Context) {
        let Some(file) = FileDialog::new().add_filter("CSV", &["csv"]).pick_file() else { return };

        let (tx, rx) = crossbeam_channel::unbounded();
        let metadata = std::fs::metadata(&file).expect("Failed to read file info");
        
        self.file_size = metadata.len();
        self.progress_counter.store(0, Ordering::SeqCst);
        self.receiver = Some(rx);
        self.is_loading = true;
        self.start_time = Some(std::time::Instant::now()); // Start the clock

        let progress_clone = self.progress_counter.clone();
        
        std::thread::spawn(move || {
            let timer = std::time::Instant::now(); // Internal thread timer
            let path_str = file.display().to_string();
            let result = summarise_csv(&file, progress_clone)
                .map(|s| (path_str, metadata.len(), s, timer.elapsed())); // Send duration back
            if tx.send(result).is_err() {
                log::error!("Failed to send analysis result");
            }
            ctx.request_repaint();
        });
    }
}

// HELPER FUNCTIONS

struct ProgressReader<R> {
    inner: R,
    counter: Arc<AtomicU64>,
}

impl<R: std::io::Read> std::io::Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.counter.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}

impl<R: std::io::Seek> std::io::Seek for ProgressReader<R> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = self.inner.seek(pos)?;
        self.counter.store(new_pos, Ordering::Relaxed);
        Ok(new_pos)
    }
}

// Polars requires this trait for CsvReader
impl<R: std::io::Read + std::io::Seek + Send + Sync> polars::io::mmap::MmapBytesReader for ProgressReader<R> {}

fn summarise_csv(path: &std::path::Path, progress: Arc<AtomicU64>) -> Result<Vec<ColumnSummary>> {
    let file = std::fs::File::open(path)?;
    let reader = ProgressReader {
        inner: file,
        counter: progress,
    };

    let df = CsvReader::new(reader)
        .finish()?;

    let row_count = df.height();
    let mut summaries = Vec::new();

    for col in df.get_columns() {
        let name = col.name().to_string();
        let nulls = col.null_count();
        let count = row_count;

        let dtype = col.dtype();
        let (kind, stats) = if dtype.is_numeric() {
            let series = col.as_materialized_series();
            
            // Cast to f64 for common stats if it's numeric
            let f64_series = series.cast(&DataType::Float64)?;
            let ca = f64_series.f64()?;

            let min = ca.min();
            let max = ca.max();
            let mean = ca.mean();
            
            let q1 = ca.quantile(0.25, QuantileMethod::Linear)?;
            let median = ca.median();
            let q3 = ca.quantile(0.75, QuantileMethod::Linear)?;

            (ColumnKind::Numeric, ColumnStats::Numeric(NumericStats {
                min,
                q1,
                median,
                mean,
                q3,
                max,
            }))
        } else if dtype.is_string() {
            let series = col.as_materialized_series();
            let ca = series.str()?;
            let distinct = ca.n_unique()?;
            
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

                (ColumnKind::Text, ColumnStats::Text(TextStats {
                    distinct,
                    top_value,
                }))
            }
        } else {
            // Default fallback for other types
            (ColumnKind::Text, ColumnStats::Text(TextStats {
                distinct: col.as_materialized_series().n_unique()?,
                top_value: None,
            }))
        };

        summaries.push(ColumnSummary {
            name,
            kind,
            count,
            nulls,
            stats,
        });
    }

    Ok(summaries)
}


fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.4}"),
        _ => "—".to_owned(),
    }
}
