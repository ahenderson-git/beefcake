//! This module implements the Graphical User Interface for the File Analyser.
//!
//! It uses `egui` for the UI and manages the state of file analysis,
//! data cleaning configurations, and database export workflows.

use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Line, Plot, PlotBounds};
use polars::prelude::DataFrame;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::logic::{
    AnalysisReceiver, BooleanStats, ColumnStats, ColumnSummary, FileHealth,
    NumericStats, TemporalStats, TextStats,
};
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
    pub cleaning_configs: std::collections::HashMap<String, super::logic::types::ColumnCleanConfig>,
    pub pg_url: String,
    pub pg_schema: String,
    pub pg_table: String,
    #[serde(skip)]
    pub is_pushing: bool,
    #[serde(skip)]
    pub push_start_time: Option<std::time::Instant>,
    pub push_last_duration: Option<std::time::Duration>,
    #[serde(skip)]
    pub was_cleaning: bool,
    #[serde(skip)]
    pub push_receiver: Option<crossbeam_channel::Receiver<anyhow::Result<()>>>,
    #[serde(skip)]
    pub should_scroll_to_top: bool,
}

impl App {
    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        self.handle_receivers();

        if self.is_loading || self.is_pushing {
            ctx.request_repaint();
        }

        let mut go_back = false;
        egui::TopBottomPanel::top("analyser_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();
                ui.heading("Analyse, Clean & Export Data");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_controls(ui, ctx);
            ui.add_space(4.0);
            self.render_db_config(ui, ctx);
            ui.label(&self.status);
            self.render_health_summary(ui);

            if !self.summary.is_empty() {
                ui.separator();
                self.render_summary_table(ui);
            }
        });

        go_back
    }

    fn handle_receivers(&mut self) {
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
                        self.should_scroll_to_top = true;

                        // Reset cleaning configs if we just applied them, otherwise preserve (for trim changes)
                        if self.was_cleaning {
                            self.cleaning_configs.clear();
                        }
                        for col in &self.summary {
                            self.cleaning_configs
                                .entry(col.name.clone())
                                .or_default();
                        }

                        let path_display = self.file_path.as_deref().unwrap_or("file");
                        let secs = duration.as_secs_f32();
                        self.status = format!("Loaded {path_display} in {secs:.2}s");
                    }
                    Err(e) => {
                        self.status = format!("Error: {e}");
                    }
                }
            }
        }

        if let Some(rx) = &self.push_receiver {
            if let Ok(result) = rx.try_recv() {
                let duration = self.push_start_time.take().map(|s| s.elapsed());
                self.push_last_duration = duration;
                self.is_pushing = false;
                self.push_receiver = None;
                match result {
                    Ok(_) => {
                        let secs = duration.map(|d| d.as_secs_f32()).unwrap_or(0.0);
                        self.status = format!("Successfully pushed to PostgreSQL in {secs:.2}s");
                    }
                    Err(e) => {
                        self.status = format!("PostgreSQL Push Error: {e}");
                    }
                }
            }
        }
    }

    fn render_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // --- Group: File ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("📁");
                    if ui.button("Open File").clicked() && !self.is_loading {
                        self.start_analysis(ctx.clone());
                    }
                });
            });

            // --- Group: Analysis Settings ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("📊 Analysis:");
                    ui.label("Trim %:").on_hover_text(
                        "Percentage to trim from EACH end of the data for the Trimmed Mean calculation.",
                    );
                    let slider = ui.add(
                        egui::Slider::new(&mut self.trim_pct, 0.0..=0.2)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                    );

                    if slider.changed() && !self.is_loading && self.df.is_some() {
                        self.trigger_reanalysis(ctx.clone());
                    }

                    ui.separator();
                    ui.checkbox(&mut self.show_full_range, "Full Range")
                        .on_hover_text("If unchecked, the histogram zooms into the 5th-95th percentile to avoid being 'crushed' by extreme outliers.");
                });
            });

            // --- Group: View Controls ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🗄 View:");
                    if ui.button("Expand All").clicked() {
                        for col in &self.summary {
                            self.expanded_rows.insert(col.name.clone());
                        }
                    }
                    if ui.button("Collapse All").clicked() {
                        self.expanded_rows.clear();
                    }
                });
            });

            // --- Group: Data Cleaning ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🧼 Cleaning:");
                    if ui
                        .button("Apply Cleaning")
                        .on_hover_text("Apply the selected cleaning actions to the data and re-analyse.")
                        .clicked()
                        && !self.is_loading
                        && self.df.is_some()
                    {
                        self.start_cleaning(ctx.clone());
                    }
                });
            });

            // --- Status & Timing ---
            if self.is_loading {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    let bytes = self.progress_counter.load(Ordering::Relaxed);
                    if bytes > 0 {
                        let mb = bytes as f64 / 1_000_000.0;
                        ui.label(format!("Loading... ({mb:.1} MB)"));
                    } else {
                        ui.label("Processing...");
                    }

                    if let Some(start) = self.start_time {
                        let elapsed = start.elapsed().as_secs_f32();
                        ui.label(format!("({elapsed:.1}s)"));
                    }
                });
            } else if let Some(duration) = self.last_duration {
                ui.add_space(4.0);
                let secs = duration.as_secs_f32();
                ui.label(egui::RichText::new(format!("⏱ {secs:.2}s")).weak());
            }
        });
    }

    fn render_db_config(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // --- Group: Database Export ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🚀 Database Export:");

                    ui.label("URL:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pg_url)
                            .hint_text("postgres://user:pass@host/db")
                            .desired_width(200.0),
                    );

                    ui.separator();
                    ui.label("Schema:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pg_schema)
                            .hint_text("public")
                            .desired_width(60.0),
                    );

                    ui.separator();
                    ui.label("Table:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pg_table)
                            .hint_text("data_123")
                            .desired_width(100.0),
                    );

                    ui.separator();
                    if self.is_pushing {
                        ui.spinner();
                        let mut label = "Pushing...".to_owned();
                        if let Some(start) = self.push_start_time {
                            label.push_str(&format!(" ({:.1}s)", start.elapsed().as_secs_f32()));
                        }
                        ui.label(label);
                    } else if ui
                        .button("Push to DB")
                        .on_hover_text(
                            "Initialize schema and push analysis results + data to PostgreSQL",
                        )
                        .clicked()
                    {
                        if let (Some(df), Some(health), Some(path)) =
                            (self.df.as_ref(), self.health.as_ref(), self.file_path.as_ref())
                        {
                            self.start_push_to_db(
                                ctx.clone(),
                                path.clone(),
                                self.file_size,
                                health.clone(),
                                self.summary.clone(),
                                df.clone(),
                            );
                        }
                    }
                });
            });
        });
    }

    fn render_health_summary(&self, ui: &mut egui::Ui) {
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
    }

    fn render_summary_table(&mut self, ui: &mut egui::Ui) {
        let mut scroll_area = egui::ScrollArea::horizontal();
        if self.should_scroll_to_top {
            scroll_area = scroll_area.scroll_offset(egui::Vec2::ZERO);
        }

        scroll_area.show(ui, |ui| {
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Min))
                .column(Column::auto().at_least(30.0)) // Expand
                .column(Column::initial(120.0).at_least(100.0)) // Name
                .column(Column::initial(100.0).at_least(80.0)) // Type
                .column(Column::auto().at_least(80.0)) // Nulls
                .column(Column::auto().at_least(80.0)) // Special
                .column(Column::initial(250.0).at_least(150.0)) // Stats & Samples
                .column(Column::initial(250.0).at_least(150.0)) // Technical Summary
                .column(Column::initial(250.0).at_least(150.0)) // Stakeholder Insight
                .column(Column::initial(100.0).at_least(100.0)) // Histogram
                .column(Column::remainder()) // Spacer
                .min_scrolled_height(0.0);

            if self.should_scroll_to_top {
                table = table.scroll_to_row(0, None);
                self.should_scroll_to_top = false;
            }

            table
                .header(25.0, |mut header| {
                    header.col(|_| {}); // Expand
                    header.col(|ui| {
                        let header_color = if ui.visuals().dark_mode {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        };
                        ui.label(egui::RichText::new("Column").strong().color(header_color));
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
                        ui.strong("Stats & Samples").on_hover_text("Core technical stats and a small selection of sample values.");
                    });
                    header.col(|ui| {
                        ui.strong("Technical Summary").on_hover_text("Detailed technical interpretation of the data patterns.");
                    });
                    header.col(|ui| {
                        ui.strong("Stakeholder Insight").on_hover_text("Business-friendly explanation of what the data means.");
                    });
                    header.col(|ui| {
                        ui.strong("Histogram");
                    });
                    header.col(|_| {}); // Spacer
                })
                .body(|mut body| {
                    let summary = self.summary.clone();
                    for col in &summary {
                        let is_expanded = self.expanded_rows.contains(&col.name);
                        let row_height = if is_expanded {
                            300.0f32
                        } else {
                            match &col.stats {
                                ColumnStats::Numeric(_) => 200.0f32,
                                ColumnStats::Text(_) | ColumnStats::Boolean(_) => 140.0f32,
                                ColumnStats::Categorical(_) | ColumnStats::Temporal(_) => 180.0f32,
                            }
                        }
                        .max(35.0);

                        body.row(row_height, |row| {
                            self.render_column_row(row, col, is_expanded);
                        });
                    }
                });
        });
    }

    fn render_column_row(&mut self, mut row: egui_extras::TableRow<'_, '_>, col: &ColumnSummary, is_expanded: bool) {
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
            self.render_name_editor(ui, col);
        });
        row.col(|ui| {
            self.render_type_editor(ui, col);
        });
        row.col(|ui| {
            let null_pct = (col.nulls as f64 / col.count as f64) * 100.0;
            ui.label(format!("{} ({null_pct:.1}%)", col.nulls));
        });
        row.col(|ui| {
            if col.has_special {
                ui.colored_label(egui::Color32::RED, "⚠ Yes");
            } else {
                ui.label("No");
            }
        });
        row.col(|ui| {
            ui.vertical(|ui| {
                // 1. Core Stats
                ui.scope(|ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    match &col.stats {
                        ColumnStats::Numeric(s) => self.render_numeric_stats_info(ui, s),
                        ColumnStats::Text(s) => Self::render_text_stats_info(ui, s),
                        ColumnStats::Categorical(freq) => {
                            Self::render_categorical_stats_info(ui, freq, is_expanded, col.count);
                        }
                        ColumnStats::Temporal(s) => Self::render_temporal_stats_info(ui, s),
                        ColumnStats::Boolean(s) => Self::render_boolean_stats_info(ui, s, col.count),
                    }
                });

                // 2. Sample Values (Now always shown under stats)
                if !col.samples.is_empty() {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Samples:").strong().size(11.0));
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(col.samples.join(", "))
                                .italics()
                                .size(11.0),
                        )
                        .wrap_mode(egui::TextWrapMode::Wrap),
                    );
                }

                // 3. Advanced Metrics
                Self::render_advanced_metrics(ui, &col.stats);

                // 4. Advanced Cleaning (Only shown when expanded)
                if is_expanded {
                    self.render_cleaning_controls(ui, col);
                }
            });
        });
        row.col(|ui| {
            // Technical Summary Column
            ui.vertical(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for line in &col.interpretation {
                    ui.label(format!("• {line}"));
                }
            });
        });
        row.col(|ui| {
            // Stakeholder Insight Column
            ui.vertical(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for line in &col.business_summary {
                    ui.label(format!("• {line}"));
                }
            });
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
    }

    fn render_cleaning_controls(&mut self, ui: &mut egui::Ui, col: &ColumnSummary) {
        ui.add_space(8.0);
        ui.separator();
        ui.strong("🧽 Advanced Cleaning:");
        if let Some(config) = self.cleaning_configs.get_mut(&col.name) {
            ui.horizontal(|ui| {
                ui.checkbox(&mut config.trim_whitespace, "Trim Whitespace");
                ui.checkbox(&mut config.remove_special_chars, "Remove Special Chars");
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("🧪 ML Preprocessing:").on_hover_text("Prepare data for Machine Learning models.");

                let effective_kind = config.target_dtype.unwrap_or(col.kind);

                // --- Imputation (Depends on ORIGINAL data type) ---
                egui::ComboBox::from_id_salt(format!("impute_{}", col.name))
                    .selected_text(match config.impute_mode {
                        super::logic::types::ImputeMode::None => "No Imputation",
                        super::logic::types::ImputeMode::Mean => "Fill with Mean",
                        super::logic::types::ImputeMode::Median => "Fill with Median",
                        super::logic::types::ImputeMode::Zero => "Fill with Zero",
                        super::logic::types::ImputeMode::Mode => "Fill with Mode",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.impute_mode, super::logic::types::ImputeMode::None, "No Imputation");
                        
                        if col.kind == super::logic::types::ColumnKind::Numeric {
                            ui.selectable_value(&mut config.impute_mode, super::logic::types::ImputeMode::Mean, "Fill with Mean");
                            ui.selectable_value(&mut config.impute_mode, super::logic::types::ImputeMode::Median, "Fill with Median");
                            ui.selectable_value(&mut config.impute_mode, super::logic::types::ImputeMode::Zero, "Fill with Zero");
                        }
                        
                        if col.kind == super::logic::types::ColumnKind::Categorical {
                            ui.selectable_value(&mut config.impute_mode, super::logic::types::ImputeMode::Mode, "Fill with Mode");
                        }
                    });

                // Auto-reset invalid imputation modes
                match config.impute_mode {
                    super::logic::types::ImputeMode::Mean | super::logic::types::ImputeMode::Median | super::logic::types::ImputeMode::Zero 
                        if col.kind != super::logic::types::ColumnKind::Numeric => {
                            config.impute_mode = super::logic::types::ImputeMode::None;
                        }
                    super::logic::types::ImputeMode::Mode if col.kind != super::logic::types::ColumnKind::Categorical => {
                         config.impute_mode = super::logic::types::ImputeMode::None;
                    }
                    _ => {}
                }

                // --- Normalization (Depends on EFFECTIVE data type) ---
                if effective_kind == super::logic::types::ColumnKind::Numeric {
                    ui.separator();
                    egui::ComboBox::from_id_salt(format!("norm_{}", col.name))
                        .selected_text(match config.normalization {
                            super::logic::types::NormalizationMethod::None => "No Scaling",
                            super::logic::types::NormalizationMethod::ZScore => "Z-Score (Std)",
                            super::logic::types::NormalizationMethod::MinMax => "Min-Max (0-1)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut config.normalization, super::logic::types::NormalizationMethod::None, "No Scaling");
                            ui.selectable_value(&mut config.normalization, super::logic::types::NormalizationMethod::ZScore, "Z-Score (Std)");
                            ui.selectable_value(&mut config.normalization, super::logic::types::NormalizationMethod::MinMax, "Min-Max (0-1)");
                        });
                } else {
                    config.normalization = super::logic::types::NormalizationMethod::None;
                }

                // --- One-Hot Encoding (Depends on EFFECTIVE data type) ---
                if effective_kind == super::logic::types::ColumnKind::Categorical {
                    ui.separator();
                    ui.checkbox(&mut config.one_hot_encode, "One-Hot Encode")
                        .on_hover_text("Convert categorical values into multiple binary columns.");
                } else {
                    config.one_hot_encode = false;
                }
            });
        }
    }

    fn render_name_editor(&mut self, ui: &mut egui::Ui, col: &ColumnSummary) {
        let text_color = if ui.visuals().dark_mode {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::BLACK
        };

        if let Some(config) = self.cleaning_configs.get_mut(&col.name) {
            ui.add(
                egui::TextEdit::singleline(&mut config.new_name)
                    .hint_text(egui::RichText::new(&col.name).strong().color(text_color))
                    .desired_width(f32::INFINITY),
            );
        } else {
            ui.label(egui::RichText::new(&col.name).strong().color(text_color));
        }
    }

    fn render_type_editor(&mut self, ui: &mut egui::Ui, col: &ColumnSummary) {
        if let Some(config) = self.cleaning_configs.get_mut(&col.name) {
            egui::ComboBox::from_id_salt(format!("row_dtype_{}", col.name))
                .selected_text(
                    config
                        .target_dtype
                        .map(|k| k.as_str())
                        .unwrap_or(col.kind.as_str()),
                )
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut config.target_dtype, None, col.kind.as_str());

                    let targets = [
                        super::logic::types::ColumnKind::Numeric,
                        super::logic::types::ColumnKind::Text,
                        super::logic::types::ColumnKind::Boolean,
                        super::logic::types::ColumnKind::Temporal,
                        super::logic::types::ColumnKind::Categorical,
                    ];

                    for target in targets {
                        if target != col.kind && col.is_compatible_with(target) {
                            ui.selectable_value(&mut config.target_dtype, Some(target), target.as_str());
                        }
                    }
                });
        } else {
            ui.label(col.kind.as_str());
        }
    }

    fn render_numeric_stats_info(&self, ui: &mut egui::Ui, s: &NumericStats) {
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
                let pct = self.trim_pct * 100.0;
                ui.label(format!("Trimmed Mean ({pct:.0}%):"))
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

    fn render_text_stats_info(ui: &mut egui::Ui, s: &TextStats) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Distinct:").on_hover_text("The number of unique values in this column.");
                ui.label(s.distinct.to_string());
            });
            if let Some((val, count)) = &s.top_value {
                ui.label(format!("Top: \"{val}\" ({count})"));
            }
        });
    }

    fn render_categorical_stats_info(
        ui: &mut egui::Ui,
        freq: &std::collections::HashMap<String, usize>,
        is_expanded: bool,
        total_count: usize,
    ) {
        let total = total_count as f64;
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
                ui.label(format!("- {val}: {count} ({pct:.1}%)"));
            }
            if sorted.len() > limit {
                ui.label("...");
            }
        });
    }

    fn render_temporal_stats_info(ui: &mut egui::Ui, s: &TemporalStats) {
        ui.vertical(|ui| {
            ui.label(format!("Min: {}", s.min.as_deref().unwrap_or("—")));
            ui.label(format!("Max: {}", s.max.as_deref().unwrap_or("—")));
        });
    }

    fn render_boolean_stats_info(ui: &mut egui::Ui, s: &BooleanStats, total_count: usize) {
        let total = total_count as f64;
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
            ui.label(format!("True: {} ({true_pct:.1}%)", s.true_count));
            ui.label(format!("False: {} ({false_pct:.1}%)", s.false_count));
        });
    }

    fn render_advanced_metrics(ui: &mut egui::Ui, stats: &ColumnStats) {
        match stats {
            ColumnStats::Numeric(s) => {
                ui.horizontal(|ui| {
                    let mut first = true;
                    if s.is_sorted {
                        ui.label(egui::RichText::new("✓ Sorted").color(egui::Color32::from_rgb(0, 150, 0)))
                            .on_hover_text("Values are strictly increasing.");
                        first = false;
                    } else if s.is_sorted_rev {
                        ui.label(egui::RichText::new("✓ Sorted (Rev)").color(egui::Color32::from_rgb(0, 150, 0)))
                            .on_hover_text("Values are strictly decreasing.");
                        first = false;
                    }

                    if s.is_integer {
                        if !first { ui.label("|"); }
                        ui.label("Integers").on_hover_text("All values are whole numbers.");
                        first = false;
                    }

                    if s.zero_count > 0 {
                        if !first { ui.label("|"); }
                        ui.label(format!("Zeros: {}", s.zero_count));
                        first = false;
                    }

                    if s.negative_count > 0 {
                        if !first { ui.label("|"); }
                        ui.label(format!("Negatives: {}", s.negative_count));
                    }
                });
            }
            ColumnStats::Text(s) => {
                ui.label(format!("Lengths: {} / {} / {:.1}", s.min_length, s.max_length, s.avg_length))
                    .on_hover_text("Character length of the text entries (Min / Max / Avg).");
            }
            ColumnStats::Temporal(s) => {
                if s.is_sorted {
                    ui.label(egui::RichText::new("✓ Chronological").color(egui::Color32::from_rgb(0, 150, 0)));
                } else if s.is_sorted_rev {
                    ui.label(egui::RichText::new("✓ Reverse Chronological").color(egui::Color32::from_rgb(0, 150, 0)));
                }
            }
            _ => {}
        }
    }

    pub fn start_analysis(&mut self, ctx: egui::Context) {
        let path = FileDialog::new()
            .add_filter("Data Files", &["csv", "json", "jsonl", "ndjson", "parquet"])
            .pick_file();

        let Some(path) = path else { return };

        self.is_loading = true;
        self.was_cleaning = false;
        self.status = format!("Loading: {}", path.display());
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());
        self.expanded_rows.clear();
        self.cleaning_configs.clear();

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();
        let trim_pct = self.trim_pct;

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let df = super::logic::load_df(&path, &progress)?;
                let file_size = std::fs::metadata(&path)?.len();
                let summary = super::logic::analyse_df(&df, trim_pct)?;
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
        self.was_cleaning = false;
        self.status = format!("Re-analysing with {:.0}% trim...", trim_pct * 100.0);
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let summary = super::logic::analyse_df(&df, trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send re-analysis result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_cleaning(&mut self, ctx: egui::Context) {
        let Some(df) = self.df.clone() else { return };
        let configs = self.cleaning_configs.clone();
        let trim_pct = self.trim_pct;
        let path_str = self.file_path.clone().unwrap_or_else(|| "cleaned_file".to_owned());
        let file_size = self.file_size;

        self.is_loading = true;
        self.was_cleaning = true;
        self.status = "Applying cleaning steps...".to_owned();
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let cleaned_df = super::logic::analysis::clean_df(df, &configs)?;
                let summary = super::logic::analyse_df(&cleaned_df, trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), cleaned_df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send cleaning result");
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
            self.status = "Error: PostgreSQL URL is required".to_owned();
            return;
        }

        self.is_pushing = true;
        self.status = "Connecting to PostgreSQL...".to_owned();
        self.push_start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.push_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let client = super::db::DbClient::connect(&pg_url).await?;
                    client.init_schema().await?;
                    
                    let schema_opt = if pg_schema.is_empty() { None } else { Some(pg_schema.as_str()) };
                    let table_opt = if pg_table.is_empty() { None } else { Some(pg_table.as_str()) };

                    client.push_analysis(super::db::AnalysisPush {
                        file_path: &file_path,
                        file_size,
                        health: &health,
                        summaries: &summary,
                        df: &df,
                        schema_name: schema_opt,
                        table_name: table_opt,
                    }).await?;
                    Ok(())
                })
            })();

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
            ColumnStats::Numeric(s) => render_numeric_plot(ui, name, s, show_full_range, plot_height),
            ColumnStats::Categorical(freq) => render_categorical_plot(ui, name, freq, plot_height),
            ColumnStats::Temporal(s) => render_temporal_plot(ui, name, s, show_full_range, plot_height),
            ColumnStats::Text(_) => { ui.label("—"); }
            ColumnStats::Boolean(s) => render_boolean_plot(ui, name, s, plot_height),
        }
    });
}

fn render_numeric_plot(ui: &mut egui::Ui, name: &str, s: &NumericStats, show_full_range: bool, plot_height: f32) {
    if s.histogram.is_empty() {
        ui.label("—");
        return;
    }

    let (view_min, view_max) = calculate_view_bounds(s, show_full_range);
    let range = view_max - view_min;
    let margin = if range > 0.0 { range * 0.05 } else { 1.0 };
    let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

    let chart = BarChart::new("Histogram", create_histogram_bars(s, view_min, view_max, margin, show_full_range))
        .color(egui::Color32::from_rgb(140, 180, 240))
        .element_formatter(Box::new(|bar, _| {
            format!("Value: {:.4}\nCount: {}", bar.argument, bar.value)
        }));

    let curve_points = create_gaussian_points(s, view_min, view_max, margin);
    let has_curve = !curve_points.is_empty();
    let curve = Line::new("", curve_points)
        .color(egui::Color32::from_rgb(250, 150, 100))
        .width(1.5);

    let box_lines = create_box_plot_lines(s, max_count, view_min, view_max, margin, show_full_range);

    ui.vertical(|ui| {
        Plot::new(format!("plot_num_{name}"))
            .show_axes([false, false])
            .show_grid([false, false])
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
            render_zoom_info(ui, s, view_min, view_max);
        }
    });
}

fn calculate_view_bounds(s: &NumericStats, show_full_range: bool) -> (f64, f64) {
    let min = s.min.unwrap_or(0.0);
    let max = s.max.unwrap_or(1.0);
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
}

fn create_histogram_bars(s: &NumericStats, view_min: f64, view_max: f64, margin: f64, show_full_range: bool) -> Vec<Bar> {
    s.histogram
        .iter()
        .filter(|&&(val, _)| {
            show_full_range || (val >= view_min - margin && val <= view_max + margin)
        })
        .map(|&(val, count)| {
            Bar::new(val, count as f64)
                .width(s.bin_width)
                .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(100, 140, 240)))
        })
        .collect()
}

fn create_gaussian_points(s: &NumericStats, view_min: f64, view_max: f64, margin: f64) -> Vec<[f64; 2]> {
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
    curve_points
}

fn create_box_plot_lines(s: &NumericStats, max_count: f64, view_min: f64, view_max: f64, margin: f64, show_full_range: bool) -> Vec<Line<'_>> {
    let mut box_lines = Vec::new();
    if let (Some(q1), Some(median), Some(q3), Some(min), Some(max)) =
        (s.q1, s.median, s.q3, s.min, s.max)
    {
        if max > min {
            let y_pos = -max_count * 0.15;
            let y_h = max_count * 0.08;
            box_lines.push(Line::new("Box", vec![[q1, y_pos - y_h], [q3, y_pos - y_h], [q3, y_pos + y_h], [q1, y_pos + y_h], [q1, y_pos - y_h]]));
            box_lines.push(Line::new("Median", vec![[median, y_pos - y_h], [median, y_pos + y_h]]));
            let w_min = if show_full_range { min } else { min.max(view_min - margin) };
            let w_max = if show_full_range { max } else { max.min(view_max + margin) };
            if w_min < q1 { box_lines.push(Line::new("Whisker1", vec![[w_min, y_pos], [q1, y_pos]])); }
            if w_max > q3 { box_lines.push(Line::new("Whisker2", vec![[q3, y_pos], [w_max, y_pos]])); }
        }
    }
    box_lines
}

fn render_zoom_info(ui: &mut egui::Ui, s: &NumericStats, view_min: f64, view_max: f64) {
    let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
    let in_view_count: usize = s.histogram.iter()
            .filter(|&&(v, _)| v >= view_min - s.bin_width/2.0 && v <= view_max + s.bin_width/2.0)
            .map(|&(_, c)| c)
            .sum();
    let out_pct = (1.0 - (in_view_count as f64 / total_count as f64)) * 100.0;
    if out_pct > 0.5 {
        ui.label(egui::RichText::new(format!("Zoomed to 5th-95th ({out_pct:.1}% hidden)")).size(9.0).color(egui::Color32::GRAY));
    }
}

fn render_categorical_plot(ui: &mut egui::Ui, name: &str, freq: &std::collections::HashMap<String, usize>, plot_height: f32) {
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

    let num_bars = sorted.len().min(10);
    Plot::new(format!("plot_cat_{name}"))
        .show_axes([false, false])
        .show_grid([false, false])
        .show_x(false)
        .show_y(false)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .include_y(0.0)
        .include_x(-0.5)
        .include_x(num_bars as f64 - 0.5)
        .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
        .height(plot_height)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
}

fn render_temporal_plot(ui: &mut egui::Ui, name: &str, s: &TemporalStats, show_full_range: bool, plot_height: f32) {
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
            .show_grid([false, false])
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
                        "Zoomed to 5th-95th ({out_pct:.1}% hidden)"
                    ))
                    .size(9.0)
                    .color(egui::Color32::GRAY),
                );
            }
        }
    });
}

fn render_boolean_plot(ui: &mut egui::Ui, name: &str, s: &BooleanStats, plot_height: f32) {
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
        .show_grid([false, false])
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
