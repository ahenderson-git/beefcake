use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Line, Plot};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::utils::fmt_opt;
use super::logic::{AnalysisReceiver, ColumnStats, ColumnSummary, FileHealth};

#[derive(Default, Deserialize, Serialize)]
pub struct App {
    pub file_path: Option<String>,
    pub file_size: u64,
    pub status: String,
    pub summary: Vec<ColumnSummary>,
    pub health: Option<FileHealth>,
    #[serde(skip)]
    pub is_loading: bool,
    #[serde(skip)]
    pub progress_counter: Arc<AtomicU64>,
    #[serde(skip)]
    pub receiver: Option<AnalysisReceiver>,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>, // To track active time
    pub last_duration: Option<std::time::Duration>, // To show final time
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
                    Ok((path, size, summary, health, duration)) => {
                        self.file_path = Some(path);
                        self.file_size = size;
                        self.summary = summary;
                        self.health = Some(health);
                        self.last_duration = Some(duration);
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

                if self.is_loading {
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
                    ui.label(format!("Last analysis took: {:.2}s", duration.as_secs_f32()));
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
                        .column(Column::auto().at_least(100.0)) // Name
                        .column(Column::auto().at_least(80.0))  // Type
                        .column(Column::auto().at_least(80.0))  // Nulls
                        .column(Column::auto().at_least(80.0))  // Special
                        .column(Column::auto().at_least(200.0)) // Stats
                        .column(Column::remainder().at_least(250.0)) // Summary
                        .column(Column::initial(150.0).at_least(150.0)) // Distribution
                        .min_scrolled_height(0.0);

                    table
                        .header(25.0, |mut header| {
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
                                ui.strong("Summary").on_hover_text("Plain-English interpretation of the data patterns.");
                            });
                            header.col(|ui| {
                                ui.strong("Distribution");
                            });
                        })
                        .body(|mut body| {
                            for col in &self.summary {
                                let row_height = match &col.stats {
                                    ColumnStats::Numeric(_) => 100.0f32,
                                    ColumnStats::Text(_) => 45.0f32,
                                    ColumnStats::Categorical(_) => 100.0f32,
                                    ColumnStats::Temporal(_) => 100.0f32,
                                    ColumnStats::Boolean(_) => 45.0f32,
                                }
                                .max(35.0);

                                body.row(row_height, |mut row| {
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
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Std Dev:").on_hover_text("Typical deviation from the average. High values mean the data is spread out.");
                                                        ui.label(fmt_opt(s.std_dev));
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
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label("Categories:").on_hover_text("The number of unique groups detected.");
                                                        ui.label(freq.len().to_string());
                                                    });
                                                    let mut sorted: Vec<_> = freq.iter().collect();
                                                    sorted.sort_by(|a, b| b.1.cmp(a.1));
                                                    for (val, count) in sorted.iter().take(3) {
                                                        ui.label(format!("- {}: {}", val, count));
                                                    }
                                                    if sorted.len() > 3 {
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
                                                ui.vertical(|ui| {
                                                    ui.label(format!("True: {}", s.true_count));
                                                    ui.label(format!("False: {}", s.false_count));
                                                });
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                        ui.label(&col.interpretation);
                                    });
                                    row.col(|ui| {
                                        render_distribution(ui, &col.name, &col.stats);
                                    });
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
            .add_filter("Data Files", &["csv", "json", "jsonl", "ndjson"])
            .pick_file();

        let Some(path) = path else { return };

        self.is_loading = true;
        self.status = format!("Loading: {}", path.display());
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration)> {
                let df = super::logic::load_df(&path, progress)?;
                let file_size = std::fs::metadata(&path)?.len();
                let summary = super::logic::analyse_df(df)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed()))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send analysis result");
            }
            ctx.request_repaint();
        });
    }
}

fn render_distribution(ui: &mut egui::Ui, name: &str, stats: &ColumnStats) {
    ui.scope(|ui| {
        // Force light mode visuals for the distribution plots so they look the same in both themes
        ui.style_mut().visuals = egui::Visuals::light();
        // Set the plot background to match our light theme background
        ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_rgb(220, 220, 215);

        match stats {
            ColumnStats::Numeric(s) => {
                if s.histogram.is_empty() {
                    ui.label("—");
                    return;
                }

                let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

                let bars: Vec<egui_plot::Bar> = s
                    .histogram
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, count))| egui_plot::Bar::new(i as f64, count as f64))
                    .collect();

                let chart = BarChart::new("Histogram", bars)
                    .width(0.75)
                    .color(egui::Color32::from_rgb(140, 180, 240));

                // Gaussian Curve Overlay
                let mut curve_points = Vec::new();
                if let (Some(mu), Some(sigma), Some(min), Some(max)) =
                    (s.mean, s.std_dev, s.min, s.max)
                {
                    if sigma > 0.0 && max > min {
                        let bin_width = (max - min) / 20.0;
                        let mu_bin = (mu - min) / bin_width;
                        let sigma_bin = sigma / bin_width;

                        let peak_val = 1.0 / (sigma_bin * (2.0 * std::f64::consts::PI).sqrt());
                        let scale = max_count / (peak_val * 1.2); // Scale to 120% of max count for visual fit

                        for i in 0..=100 {
                            let x_bin = -1.0 + (22.0 * i as f64 / 100.0);
                            let z = (x_bin - mu_bin) / sigma_bin;
                            let y = scale
                                * (1.0 / (sigma_bin * (2.0 * std::f64::consts::PI).sqrt()))
                                * (-0.5 * z * z).exp();
                            curve_points.push([x_bin, y]);
                        }
                    }
                }
                let has_curve = !curve_points.is_empty();
                let curve = Line::new("", curve_points)
                    .color(egui::Color32::from_rgb(250, 150, 100))
                    .width(1.5);

                // Box Plot Overlay (Manual Drawing)
                let mut box_lines = Vec::new();
                if let (Some(q1), Some(median), Some(q3), Some(min), Some(max)) =
                    (s.q1, s.median, s.q3, s.min, s.max)
                {
                    if max > min {
                        let bin_width = (max - min) / 20.0;
                        let q1_b = (q1 - min) / bin_width;
                        let med_b = (median - min) / bin_width;
                        let q3_b = (q3 - min) / bin_width;

                        let y_pos = -max_count * 0.15;
                        let y_h = max_count * 0.08;

                        // Box
                        box_lines.push(Line::new(
                            "Box",
                            vec![
                                [q1_b, y_pos - y_h],
                                [q3_b, y_pos - y_h],
                                [q3_b, y_pos + y_h],
                                [q1_b, y_pos + y_h],
                                [q1_b, y_pos - y_h],
                            ],
                        ));
                        // Median
                        box_lines.push(Line::new(
                            "Median",
                            vec![[med_b, y_pos - y_h], [med_b, y_pos + y_h]],
                        ));
                        // Whiskers (simplified to min/max range)
                        box_lines.push(Line::new("Whisker1", vec![[0.0, y_pos], [q1_b, y_pos]]));
                        box_lines.push(Line::new("Whisker2", vec![[q3_b, y_pos], [20.0, y_pos]]));
                    }
                }

                Plot::new(format!("plot_num_{name}"))
                    .show_axes([false, false])
                    .show_x(false)
                    .show_y(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .include_y(max_count)
                    .include_y(-max_count * 0.3)
                    .include_x(-1.0)
                    .include_x(21.0)
                    .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                    .height(80.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                        if has_curve {
                            plot_ui.line(curve);
                        }
                        for line in box_lines {
                            plot_ui.line(line.color(egui::Color32::from_gray(120)).width(1.0));
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
                    .height(80.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
            }
            ColumnStats::Temporal(s) => {
                if s.histogram.is_empty() {
                    ui.label("—");
                    return;
                }
                let chart = BarChart::new(
                    "Temporal",
                    s.histogram
                        .iter()
                        .enumerate()
                        .map(|(i, &(_ts, count))| Bar::new(i as f64, count as f64))
                        .collect(),
                )
                .width(0.75)
                .color(egui::Color32::from_rgb(200, 150, 100));

                Plot::new(format!("plot_temp_{name}"))
                    .show_axes([false, false])
                    .show_x(false)
                    .show_y(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .include_y(0.0)
                    .include_x(-1.0)
                    .include_x(20.0)
                    .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
                    .height(80.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
            }
            ColumnStats::Text(_) => {
                ui.label("—");
            }
            ColumnStats::Boolean(s) => {
                let chart = BarChart::new(
                    "Boolean",
                    vec![
                        Bar::new(0.0, s.true_count as f64).fill(egui::Color32::from_rgb(100, 200, 100)),
                        Bar::new(1.0, s.false_count as f64).fill(egui::Color32::from_rgb(200, 100, 100)),
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
                    .height(80.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
            }
        }
    });
}
