use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Line, Plot, PlotBounds};
use polars::prelude::DataFrame;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::logic::{AnalysisReceiver, ColumnStats, ColumnSummary, FileHealth};
use crate::utils::fmt_opt;

#[derive(Default, Deserialize, Serialize)]
pub struct App {
    pub file_path: Option<String>,
    pub file_size: u64,
    pub status: String,
    pub summary: Vec<ColumnSummary>,
    pub health: Option<FileHealth>,
    pub trim_pct: f64,
    pub show_full_range: bool,
    #[serde(skip)]
    pub is_loading: bool,
    #[serde(skip)]
    pub progress_counter: Arc<AtomicU64>,
    #[serde(skip)]
    pub receiver: Option<AnalysisReceiver>,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>, // To track active time
    pub last_duration: Option<std::time::Duration>, // To show final time
    #[serde(skip)]
    pub df: Option<DataFrame>,
    #[serde(skip)]
    pub expanded_rows: std::collections::HashSet<String>,
    pub pg_url: String,
    pub pg_schema: String,
    pub pg_table: String,
    #[serde(skip)]
    pub is_pushing: bool,
    #[serde(skip)]
    pub push_receiver: Option<crossbeam_channel::Receiver<anyhow::Result<()>>>,
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
                    Ok((path, size, summary, health, duration, df)) => {
                        self.file_path = Some(path);
                        self.file_size = size;
                        self.summary = summary;
                        self.health = Some(health);
                        self.last_duration = Some(duration);
                        self.df = Some(df);
                        self.status = format!(
                            "Loaded {} in {:.2}s",
                            self.file_path.as_deref().unwrap_or("file"),
                            duration.as_secs_f32()
                        );
                    }
                    Err(e) => {
                        self.status = format!("Error: {e}");
                    }
                }
            }
        }

        if let Some(rx) = &self.push_receiver {
            if let Ok(result) = rx.try_recv() {
                self.is_pushing = false;
                self.push_receiver = None;
                match result {
                    Ok(_) => {
                        self.status = "Successfully pushed to PostgreSQL".to_string();
                    }
                    Err(e) => {
                        self.status = format!("PostgreSQL Push Error: {e}");
                    }
                }
            }
        }

        egui::TopBottomPanel::top("analyser_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();
                ui.heading("File Analyser");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open File").clicked() && !self.is_loading {
                    self.start_analysis(ctx.clone());
                }

                ui.separator();
                ui.label("Trim %:").on_hover_text("Percentage to trim from EACH end of the data for the Trimmed Mean calculation.");
                let slider = ui.add(
                    egui::Slider::new(&mut self.trim_pct, 0.0..=0.2)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                );

                if slider.changed() && !self.is_loading {
                    if self.df.is_some() {
                        self.trigger_reanalysis(ctx.clone());
                    }
                }

                ui.separator();
                ui.checkbox(&mut self.show_full_range, "Full Histogram Range")
                    .on_hover_text("If unchecked, the histogram zooms into the 5th-95th percentile to avoid being 'crushed' by extreme outliers.");

                ui.separator();
                if ui.button("Expand All").clicked() {
                    for col in &self.summary {
                        self.expanded_rows.insert(col.name.clone());
                    }
                }
                if ui.button("Collapse All").clicked() {
                    self.expanded_rows.clear();
                }

                if self.is_loading {
                    ui.separator();
                    ui.spinner();
                    let bytes = self.progress_counter.load(Ordering::Relaxed);
                    if bytes > 0 {
                        ui.label(format!("Loading... ({:.1} MB)", bytes as f64 / 1_000_000.0));
                    } else {
                        ui.label("Processing...");
                    }

                    if let Some(start) = self.start_time {
                        ui.label(format!("Time: {:.1}s", start.elapsed().as_secs_f32()));
                    }
                } else if let Some(duration) = self.last_duration {
                    ui.separator();
                    ui.label(format!("Last analysis took: {:.2}s", duration.as_secs_f32()));
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("PostgreSQL URL:");
                ui.add(egui::TextEdit::singleline(&mut self.pg_url).hint_text("postgres://user:pass@host/db").desired_width(250.0));
                
                ui.separator();
                ui.label("Schema:");
                ui.add(egui::TextEdit::singleline(&mut self.pg_schema).hint_text("public").desired_width(80.0));

                ui.separator();
                ui.label("Table:");
                ui.add(egui::TextEdit::singleline(&mut self.pg_table).hint_text("data_123").desired_width(120.0));

                ui.separator();
                if self.is_pushing {
                    ui.spinner();
                    ui.label("Pushing to DB...");
                } else if ui.button("🚀 Push to DB").on_hover_text("Initialize schema and push analysis results + data to PostgreSQL").clicked() {
                    if let (Some(df), Some(health), Some(path)) = (self.df.as_ref(), self.health.as_ref(), self.file_path.as_ref()) {
                        self.start_push_to_db(ctx.clone(), path.clone(), self.file_size, health.clone(), self.summary.clone(), df.clone());
                    }
                }
            });

            ui.label(&self.status);

            if let Some(health) = &self.health {
                if !health.risks.is_empty() {
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgba_premultiplied(255, 200, 0, 10))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.strong("Data Health Summary");
                                for risk in &health.risks {
                                    ui.label(risk);
                                }
                            });
                        });
                }
            }

            if !self.summary.is_empty() {
                ui.separator();

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(30.0))  // Expand
                        .column(Column::auto().at_least(100.0)) // Name
                        .column(Column::auto().at_least(80.0))  // Type
                        .column(Column::auto().at_least(80.0))  // Nulls
                        .column(Column::auto().at_least(80.0))  // Special
                        .column(Column::auto().at_least(200.0)) // Stats
                        .column(Column::initial(150.0).at_least(150.0)) // Summary (Technical)
                        .column(Column::initial(150.0).at_least(150.0)) // Stakeholder Insight
                        .column(Column::initial(100.0).at_least(100.0)) // Histogram
                        .column(Column::remainder()) // Spacer
                        .min_scrolled_height(0.0);

                    table
                        .header(25.0, |mut header| {
                            header.col(|_| {}); // Expand
                            header.col(|ui| {
                                ui.strong("Column");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("Nulls");
                            });
                            header.col(|ui| {
                                ui.strong("Special");
                            });
                            header.col(|ui| {
                                ui.strong("Stats / Info");
                            });
                            header.col(|ui| {
                                ui.strong("Summary").on_hover_text("Technical analysis of data patterns.");
                            });
                            header.col(|ui| {
                                ui.strong("Stakeholder Insight").on_hover_text("Plain-English interpretation for business stakeholders.");
                            });
                            header.col(|ui| {
                                ui.strong("Histogram");
                            });
                            header.col(|_| {}); // Spacer
                        })
                        .body(|mut body| {
                            for col in &self.summary {
                                let is_expanded = self.expanded_rows.contains(&col.name);
                                let row_height = if is_expanded {
                                    250.0f32
                                } else {
                                    match &col.stats {
                                        ColumnStats::Numeric(_) => 110.0f32,
                                        ColumnStats::Text(_) => 45.0f32,
                                        ColumnStats::Categorical(_) => 100.0f32,
                                        ColumnStats::Temporal(_) => 100.0f32,
                                        ColumnStats::Boolean(_) => 45.0f32,
                                    }
                                }
                                .max(35.0);

                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        let icon = if is_expanded { "⏷" } else { "⏵" };
                                        if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                            if is_expanded {
                                                self.expanded_rows.remove(&col.name);
                                            } else {
                                                self.expanded_rows.insert(col.name.clone());
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.label(&col.name);
                                    });
                                    row.col(|ui| {
                                        ui.label(col.kind.as_str());
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{} ({:.1}%)",
                                            col.nulls,
                                            (col.nulls as f64 / col.count as f64) * 100.0
                                        ));
                                    });
                                    row.col(|ui| {
                                        if col.has_special {
                                            ui.colored_label(egui::Color32::RED, "⚠ Yes");
                                        } else {
                                            ui.label("No");
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                        match &col.stats {
                                            ColumnStats::Numeric(s) => {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label("Range:").on_hover_text("The full spread of values from minimum to maximum.");
                                                        ui.label(format!("{} to {}", fmt_opt(s.min), fmt_opt(s.max)));
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Mean:").on_hover_text("The mathematical average. If significantly different from the median, the data is skewed.");
                                                        ui.label(fmt_opt(s.mean));
                                                        ui.label("|");
                                                        ui.label("Median:").on_hover_text("The middle value. 50% of the data is above this, and 50% is below.");
                                                        ui.label(fmt_opt(s.median));

                                                        if let (Some(mean), Some(median)) = (s.mean, s.median) {
                                                            if median.abs() > 1e-9 && (mean - median).abs() / median.abs() > 0.1 {
                                                                ui.label("⚠").on_hover_text("Gap > 10%: Outliers likely influencing the average");
                                                            }
                                                        }
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("Trimmed Mean ({:.0}%):", self.trim_pct * 100.0))
                                                            .on_hover_text("The mean after removing the top and bottom X% of values. Helps reduce the influence of outliers.");
                                                        ui.label(fmt_opt(s.trimmed_mean));
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Std Dev:").on_hover_text("Typical deviation from the average. High values mean the data is spread out.");
                                                        ui.label(fmt_opt(s.std_dev));

                                                        if let (Some(mean), Some(median), Some(std_dev)) = (s.mean, s.median, s.std_dev) {
                                                            if std_dev > 0.0 {
                                                                let nonparametric_skew = (mean - median).abs() / std_dev;
                                                                if nonparametric_skew > 0.3 {
                                                                    ui.label("⚠").on_hover_text("Standard deviation may be less reliable because the mean is heavily influenced by outliers or skew.");
                                                                }
                                                            }
                                                        }

                                                        ui.label("|");
                                                        ui.label("IQR:").on_hover_text("Interquartile Range: The range of the middle 50% of your data.");
                                                        ui.label(format!("{}-{}", fmt_opt(s.q1), fmt_opt(s.q3)));
                                                    });
                                                });
                                            }
                                            ColumnStats::Text(s) => {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label("Distinct:").on_hover_text("The number of unique values in this column.");
                                                        ui.label(s.distinct.to_string());
                                                    });
                                                    if let Some((val, count)) = &s.top_value {
                                                        ui.label(format!("Top: \"{}\" ({})", val, count));
                                                    }
                                                });
                                            }
                                            ColumnStats::Categorical(freq) => {
                                                let total = col.count as f64;
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label("Categories:").on_hover_text("The number of unique groups detected.");
                                                        ui.label(freq.len().to_string());
                                                    });
                                                    let mut sorted: Vec<_> = freq.iter().collect();
                                                    sorted.sort_by(|a, b| b.1.cmp(a.1));
                                                    let limit = if is_expanded { 10 } else { 3 };
                                                    for (val, count) in sorted.iter().take(limit) {
                                                        let pct = if total > 0.0 {
                                                            (**count as f64 / total) * 100.0
                                                        } else {
                                                            0.0
                                                        };
                                                        ui.label(format!(
                                                            "- {}: {} ({:.1}%)",
                                                            val, count, pct
                                                        ));
                                                    }
                                                    if sorted.len() > limit {
                                                        ui.label("...");
                                                    }
                                                });
                                            }
                                            ColumnStats::Temporal(s) => {
                                                ui.vertical(|ui| {
                                                    ui.label(format!(
                                                        "Min: {}",
                                                        s.min.as_deref().unwrap_or("—")
                                                    ));
                                                    ui.label(format!(
                                                        "Max: {}",
                                                        s.max.as_deref().unwrap_or("—")
                                                    ));
                                                });
                                            }
                                            ColumnStats::Boolean(s) => {
                                                let total = col.count as f64;
                                                ui.vertical(|ui| {
                                                    let true_pct = if total > 0.0 {
                                                        (s.true_count as f64 / total) * 100.0
                                                    } else {
                                                        0.0
                                                    };
                                                    let false_pct = if total > 0.0 {
                                                        (s.false_count as f64 / total) * 100.0
                                                    } else {
                                                        0.0
                                                    };
                                                    ui.label(format!("True: {} ({:.1}%)", s.true_count, true_pct));
                                                    ui.label(format!("False: {} ({:.1}%)", s.false_count, false_pct));
                                                });
                                            }
                                        }

                                        if is_expanded && !col.samples.is_empty() {
                                            ui.add_space(8.0);
                                            ui.separator();
                                            ui.strong("Sample Values:");
                                            ui.label(
                                                egui::RichText::new(col.samples.join(", "))
                                                    .italics()
                                                    .size(11.0),
                                            );
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                        ui.label(&col.interpretation);
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                        ui.label(&col.business_summary);
                                    });
                                    row.col(|ui| {
                                        render_distribution(
                                            ui,
                                            &col.name,
                                            &col.stats,
                                            self.show_full_range,
                                            is_expanded,
                                        );
                                    });
                                    row.col(|_| {}); // Spacer
                                });
                            }
                        });
                });
            }
        });

        go_back
    }

    pub fn start_analysis(&mut self, ctx: egui::Context) {
        let path = FileDialog::new()
            .add_filter("Data Files", &["csv", "json", "jsonl", "ndjson", "parquet"])
            .pick_file();

        let Some(path) = path else { return };

        self.is_loading = true;
        self.status = format!("Loading: {}", path.display());
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());
        self.expanded_rows.clear();

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();
        let trim_pct = self.trim_pct;

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let df = super::logic::load_df(&path, progress)?;
                let file_size = std::fs::metadata(&path)?.len();
                let summary = super::logic::analyse_df(df.clone(), trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send analysis result");
            }
            ctx.request_repaint();
        });
    }

    pub fn trigger_reanalysis(&mut self, ctx: egui::Context) {
        let Some(df) = self.df.clone() else { return };
        let Some(path_str) = self.file_path.clone() else { return };
        let file_size = self.file_size;
        let trim_pct = self.trim_pct;

        self.is_loading = true;
        self.status = format!("Re-analysing with {:.0}% trim...", trim_pct * 100.0);
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let summary = super::logic::analyse_df(df.clone(), trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send re-analysis result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_push_to_db(
        &mut self,
        ctx: egui::Context,
        file_path: String,
        file_size: u64,
        health: FileHealth,
        summary: Vec<ColumnSummary>,
        df: DataFrame,
    ) {
        let pg_url = self.pg_url.clone();
        let pg_schema = self.pg_schema.clone();
        let pg_table = self.pg_table.clone();
        if pg_url.is_empty() {
            self.status = "Error: PostgreSQL URL is required".to_string();
            return;
        }

        self.is_pushing = true;
        self.status = "Connecting to PostgreSQL...".to_string();

        let (tx, rx) = crossbeam_channel::unbounded();
        self.push_receiver = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = super::db::DbClient::connect(&pg_url).await?;
                client.init_schema().await?;
                
                let schema_opt = if pg_schema.is_empty() { None } else { Some(pg_schema.as_str()) };
                let table_opt = if pg_table.is_empty() { None } else { Some(pg_table.as_str()) };

                client.push_analysis(&file_path, file_size, &health, &summary, &df, schema_opt, table_opt).await?;
                Ok(())
            });

            if tx.send(result).is_err() {
                log::error!("Failed to send push result");
            }
            ctx.request_repaint();
        });
    }
}

fn render_distribution(
    ui: &mut egui::Ui,
    name: &str,
    stats: &ColumnStats,
    show_full_range: bool,
    is_expanded: bool,
) {
    ui.scope(|ui| {
        // Force light mode visuals for the distribution plots so they look the same in both themes
        ui.style_mut().visuals = egui::Visuals::light();
        // Set the plot background to match our light theme background
        ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_rgb(220, 220, 215);

        let plot_height = if is_expanded { 220.0 } else { 80.0 };

        match stats {
            ColumnStats::Numeric(s) => {
                if s.histogram.is_empty() {
                    ui.label("—");
                    return;
                }

                let (view_min, view_max) = {
                    let min = s.min.unwrap_or(0.0);
                    let max = s.max.unwrap_or(1.0);
                    let p05 = s.p05.unwrap_or(min);
                    let p95 = s.p95.unwrap_or(max);

                    if show_full_range {
                        (min, max)
                    } else {
                        let full_range = max - min;
                        let zoom_range = p95 - p05;

                        // Only auto-zoom if the full range is significantly (3x) larger than the central 90%.
                        // This prevents "zooming in too far" on distributions that are just normally skewed.
                        if full_range > 3.0 * zoom_range && zoom_range > 0.0 {
                            (p05, p95)
                        } else {
                            (min, max)
                        }
                    }
                };

                let range = view_max - view_min;
                let margin = if range > 0.0 { range * 0.05 } else { 1.0 };

                let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

                let bars: Vec<egui_plot::Bar> = s
                    .histogram
                    .iter()
                    .filter(|&&(val, _)| {
                        show_full_range || (val >= view_min - margin && val <= view_max + margin)
                    })
                    .map(|&(val, count)| {
                        egui_plot::Bar::new(val, count as f64)
                            .width(s.bin_width)
                            .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(100, 140, 240)))
                    })
                    .collect();

                let chart = BarChart::new("Histogram", bars)
                    .color(egui::Color32::from_rgb(140, 180, 240))
                    .element_formatter(Box::new(|bar, _| {
                        format!("Value: {:.4}\nCount: {}", bar.argument, bar.value)
                    }));

                // Gaussian Curve Overlay
                let mut curve_points = Vec::new();
                if let (Some(mu), Some(sigma)) = (s.mean, s.std_dev) {
                    if sigma > 0.0 {
                        let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                        let scale = total_count as f64 * s.bin_width;

                        let plot_min = view_min - margin;
                        let plot_max = view_max + margin;
                        if plot_max > plot_min {
                            let step = (plot_max - plot_min) / 100.0;
                            for i in 0..=100 {
                                let x = plot_min + i as f64 * step;
                                let z = (x - mu) / sigma;
                                let y = scale
                                    * (1.0 / (sigma * (2.0 * std::f64::consts::PI).sqrt()))
                                    * (-0.5 * z * z).exp();
                                curve_points.push([x, y]);
                            }
                        }
                    }
                }
                let has_curve = !curve_points.is_empty();
                let curve = Line::new("", curve_points)
                    .color(egui::Color32::from_rgb(250, 150, 100))
                    .width(1.5);

                // Box Plot Overlay
                let mut box_lines = Vec::new();
                if let (Some(q1), Some(median), Some(q3), Some(min), Some(max)) =
                    (s.q1, s.median, s.q3, s.min, s.max)
                {
                    if max > min {
                        let y_pos = -max_count * 0.15;
                        let y_h = max_count * 0.08;

                        // Box
                        box_lines.push(Line::new(
                            "Box",
                            vec![
                                [q1, y_pos - y_h],
                                [q3, y_pos - y_h],
                                [q3, y_pos + y_h],
                                [q1, y_pos + y_h],
                                [q1, y_pos - y_h],
                            ],
                        ));
                        // Median
                        box_lines.push(Line::new(
                            "Median",
                            vec![[median, y_pos - y_h], [median, y_pos + y_h]],
                        ));
                        // Whiskers - Only draw if they are somewhat within view, or clip them
                        let w_min = if show_full_range { min } else { min.max(view_min - margin) };
                        let w_max = if show_full_range { max } else { max.min(view_max + margin) };
                        
                        if w_min < q1 {
                            box_lines.push(Line::new("Whisker1", vec![[w_min, y_pos], [q1, y_pos]]));
                        }
                        if w_max > q3 {
                            box_lines.push(Line::new("Whisker2", vec![[q3, y_pos], [w_max, y_pos]]));
                        }
                    }
                }

                ui.vertical(|ui| {
                    Plot::new(format!("plot_num_{name}"))
                        .show_axes([false, false])
                        .show_x(false)
                        .show_y(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .include_y(max_count)
                        .include_y(-max_count * 0.3)
                        .include_x(view_min - margin)
                        .include_x(view_max + margin)
                        .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                        .height(plot_height)
                        .show(ui, |plot_ui| {
                            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                [view_min - margin, -max_count * 0.3],
                                [view_max + margin, max_count * 1.1],
                            ));
                            plot_ui.bar_chart(chart);
                            if has_curve {
                                plot_ui.line(curve);
                            }
                            for line in box_lines {
                                plot_ui.line(line.color(egui::Color32::from_gray(120)).width(1.0));
                            }
                        });

                    if !show_full_range {
                        let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                        let in_view_count: usize = s.histogram.iter()
                                .filter(|&&(v, _)| v >= view_min - s.bin_width/2.0 && v <= view_max + s.bin_width/2.0)
                                .map(|&(_, c)| c)
                                .sum();
                        let out_pct = (1.0 - (in_view_count as f64 / total_count as f64)) * 100.0;
                        if out_pct > 0.5 {
                            ui.label(egui::RichText::new(format!("Zoomed to 5th-95th ({:.1}% hidden)", out_pct)).size(9.0).color(egui::Color32::GRAY));
                        }
                    }
                });
            }
            ColumnStats::Categorical(freq) => {
                if freq.is_empty() {
                    ui.label("—");
                    return;
                }
                let mut sorted: Vec<_> = freq.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));

                let chart = BarChart::new(
                    "Categories",
                    sorted
                        .iter()
                        .take(10)
                        .enumerate()
                        .map(|(i, &(_val, &count))| Bar::new(i as f64, count as f64))
                        .collect(),
                )
                .width(0.75)
                .color(egui::Color32::from_rgb(100, 200, 100));

                Plot::new(format!("plot_cat_{name}"))
                    .show_axes([false, false])
                    .show_x(false)
                    .show_y(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .include_y(0.0)
                    .include_x(-1.0)
                    .include_x(10.0)
                    .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                    .height(plot_height)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
            }
            ColumnStats::Temporal(s) => {
                if s.histogram.is_empty() {
                    ui.label("—");
                    return;
                }

                let (view_min, view_max) = {
                    let min = s.histogram.first().map(|h| h.0).unwrap_or(0.0);
                    let max = s.histogram.last().map(|h| h.0).unwrap_or(1.0);
                    let p05 = s.p05.unwrap_or(min);
                    let p95 = s.p95.unwrap_or(max);

                    if show_full_range {
                        (min, max)
                    } else {
                        let full_range = max - min;
                        let zoom_range = p95 - p05;

                        if full_range > 3.0 * zoom_range && zoom_range > 0.0 {
                            (p05, p95)
                        } else {
                            (min, max)
                        }
                    }
                };

                let range = view_max - view_min;
                let margin = if range > 0.0 { range * 0.05 } else { 1.0 };

                let bars: Vec<egui_plot::Bar> = s
                    .histogram
                    .iter()
                    .filter(|&&(ts, _)| {
                        show_full_range || (ts >= view_min - margin && ts <= view_max + margin)
                    })
                    .map(|&(ts, count)| {
                        Bar::new(ts, count as f64)
                            .width(s.bin_width)
                            .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(160, 110, 60)))
                    })
                    .collect();

                let chart = BarChart::new("Temporal", bars).color(egui::Color32::from_rgb(200, 150, 100));

                let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

                ui.vertical(|ui| {
                    Plot::new(format!("plot_temp_{name}"))
                        .show_axes([false, false])
                        .show_x(false)
                        .show_y(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .include_y(max_count)
                        .include_y(0.0)
                        .include_x(view_min - margin)
                        .include_x(view_max + margin)
                        .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                        .height(plot_height)
                        .show(ui, |plot_ui| {
                            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                [view_min - margin, -max_count * 0.1],
                                [view_max + margin, max_count * 1.1],
                            ));
                            plot_ui.bar_chart(chart);
                        });

                    if !show_full_range {
                        let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
                        let in_view_count: usize = s.histogram
                            .iter()
                            .filter(|&&(v, _)| {
                                v >= view_min - s.bin_width / 2.0 && v <= view_max + s.bin_width / 2.0
                            })
                            .map(|&(_, c)| c)
                            .sum();
                        let out_pct = (1.0 - (in_view_count as f64 / total_count as f64)) * 100.0;
                        if out_pct > 0.5 {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Zoomed to 5th-95th ({:.1}% hidden)",
                                    out_pct
                                ))
                                .size(9.0)
                                .color(egui::Color32::GRAY),
                            );
                        }
                    }
                });
            }
            ColumnStats::Text(_) => {
                ui.label("—");
            }
            ColumnStats::Boolean(s) => {
                let chart = BarChart::new(
                    "Boolean",
                    vec![
                        Bar::new(0.0, s.true_count as f64)
                            .fill(egui::Color32::from_rgb(100, 200, 100)),
                        Bar::new(1.0, s.false_count as f64)
                            .fill(egui::Color32::from_rgb(200, 100, 100)),
                    ],
                )
                .width(0.75);

                Plot::new(format!("plot_bool_{name}"))
                    .show_axes([false, false])
                    .show_x(false)
                    .show_y(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .include_y(0.0)
                    .include_x(-1.0)
                    .include_x(2.0)
                    .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                    .height(plot_height)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
            }
        }
    });
}
