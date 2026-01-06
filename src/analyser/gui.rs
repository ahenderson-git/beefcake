//! This module implements the Graphical User Interface for the File Analyser.
//!
//! It uses `egui` for the UI and manages the state of file analysis,
//! data cleaning configurations, and database export workflows.

use super::controller::AnalysisController;
use super::logic::ColumnSummary;
use super::model::AnalysisModel;
use eframe::egui;
use egui_phosphor::regular as icons;
use polars::prelude::*;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

mod controls;
mod error_view;
mod heatmap;
mod plots;
mod summary_table;

pub use controls::{render_controls, render_db_config, render_ml_details_window, render_ml_panel};
pub use error_view::render_error_diagnostics_window;
use heatmap::render_correlation_heatmap;
use summary_table::render_summary_table;

#[derive(Default, Deserialize, Serialize)]
pub struct App {
    pub model: AnalysisModel,
    #[serde(skip)]
    pub controller: AnalysisController,
    pub status: String,
    pub load_summary: Option<String>,
    pub summary_minimized: bool,
    pub analysis_minimized: bool,
    #[serde(skip)]
    pub expanded_rows: std::collections::HashSet<String>,
    #[serde(skip)]
    pub should_scroll_to_top: bool,
    #[serde(skip)]
    pub show_ml_details: bool,
    #[serde(skip)]
    pub audit_log: Vec<crate::utils::AuditEntry>,
    #[serde(skip)]
    pub error_log: Vec<crate::utils::DetailedError>,
    #[serde(skip)]
    pub show_error_diagnostics: bool,
}

impl App {
    pub fn log_action(&mut self, action: &str, details: &str) {
        crate::utils::push_audit_log(&mut self.audit_log, action, details);
    }

    pub fn update(&mut self, ctx: &egui::Context, toasts: &mut egui_notify::Toasts) -> bool {
        self.handle_receivers(toasts);

        if self.model.ml_enabled && self.show_ml_details {
            render_ml_details_window(self, ctx);
        }

        if self.show_error_diagnostics {
            render_error_diagnostics_window(
                &mut self.error_log,
                &mut self.show_error_diagnostics,
                ctx,
            );
        }

        if self.controller.is_loading || self.controller.is_pushing || self.controller.is_training {
            ctx.request_repaint();
        }

        let mut go_back = false;
        egui::TopBottomPanel::top("analyser_top")
            .frame(crate::theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("{} Back", icons::ARROW_LEFT)).clicked() {
                        go_back = true;
                    }
                    ui.separator();
                    ui.heading("Analyse, Clean & Export Data");
                });
            });

        egui::CentralPanel::default()
            .frame(crate::theme::central_panel_frame().inner_margin(egui::Margin {
                left: crate::theme::PANEL_LEFT as i8,
                right: crate::theme::PANEL_RIGHT as i8,
                top: crate::theme::SPACING_LARGE as i8,
                bottom: 0,
            }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("analyser_scroll")
                    .show(ui, |ui| {
                        render_controls(self, ui, ctx);
                        ui.add_space(crate::theme::SPACING_TINY);
                        render_db_config(self, ui, ctx);
                        ui.add_space(crate::theme::SPACING_TINY);
                        if self.model.ml_enabled {
                            render_ml_panel(self, ui, ctx);
                        }

                        self.render_load_summary(ui);

                        ui.add_space(crate::theme::SPACING_TINY);
                        ui.horizontal(|ui| {
                            crate::utils::render_status_message(ui, &self.status);
                            if !self.error_log.is_empty()
                                && (self.status.contains("failed")
                                    || self.status.contains("Error")
                                    || self.status.contains(icons::X_CIRCLE))
                            {
                                ui.add_space(crate::theme::SPACING_SMALL);
                                if ui
                                    .button(format!("{} View Diagnostics", icons::SHIELD_WARNING))
                                    .clicked()
                                {
                                    self.show_error_diagnostics = true;
                                }
                            }
                        });

                        if !self.model.summary.is_empty() {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{} Column Analysis", icons::TABLE))
                                        .strong()
                                        .size(14.0),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let btn_text = if self.analysis_minimized {
                                            "Expand"
                                        } else {
                                            "Minimise"
                                        };
                                        if ui.button(btn_text).clicked() {
                                            self.analysis_minimized = !self.analysis_minimized;
                                        }
                                    },
                                );
                            });

                            if !self.analysis_minimized {
                                ui.add_space(crate::theme::SPACING_TINY);
                                render_summary_table(self, ui);
                            }
                        }

                        if let Some(matrix) = &self.model.correlation_matrix {
                            ui.add_space(crate::theme::SPACING_LARGE);
                            ui.separator();
                            render_correlation_heatmap(ui, matrix);
                        }
                        ui.add_space(crate::theme::SPACING_LARGE);
                    });
            });

        go_back
    }

    fn render_load_summary(&mut self, ui: &mut egui::Ui) {
        if let Some(summary) = &self.load_summary {
            ui.add_space(crate::theme::SPACING_SMALL);
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} File Summary",
                                    icons::CLIPBOARD_TEXT
                                ))
                                .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let btn_text = if self.summary_minimized {
                                    "Expand"
                                } else {
                                    "Minimise"
                                };
                                if ui.button(btn_text).clicked() {
                                    self.summary_minimized = !self.summary_minimized;
                                }
                            });
                        });
                        if !self.summary_minimized {
                            ui.separator();
                            ui.label(summary);
                        }
                    });
                });
        }
    }

    fn handle_receivers(&mut self, toasts: &mut egui_notify::Toasts) {
        self.handle_analysis_receiver(toasts);
        self.handle_secondary_receiver(toasts);
        self.handle_push_receiver(toasts);
        self.handle_test_receiver(toasts);
        self.handle_training_receiver(toasts);
        self.handle_export_receiver(toasts);
    }

    fn process_result<T>(
        &mut self,
        toasts: &mut egui_notify::Toasts,
        result: anyhow::Result<T>,
        action_label: &str,
        success_handler: impl FnOnce(&mut Self, T) -> (String, String), // (Success Message, Log Detail)
    ) {
        match result {
            Ok(data) => {
                let (msg, log_detail) = success_handler(self, data);
                self.status = format!("{} {}", icons::CHECK_CIRCLE, msg);
                toasts.success(&msg);
                self.log_action(&format!("{action_label} Success"), &log_detail);
            }
            Err(e) => {
                let err_msg = format!("{action_label} failed: {e}");
                self.status = format!("{} {err_msg}", icons::X_CIRCLE);
                toasts.error(&err_msg);
                self.log_action(&format!("{action_label} Failed"), &e.to_string());
                crate::utils::push_error_log(&mut self.error_log, &e, action_label);
            }
        }
    }

    fn handle_secondary_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .secondary_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.secondary_receiver = None;
            self.process_result(toasts, result, "Secondary Load", |this, resp| {
                this.model.secondary_df = Some(resp.df);
                this.model.secondary_summary = resp.summary;
                let name = resp.file_path;
                this.model.secondary_file_name = Some(name.clone());
                (format!("Loaded secondary file: {name}"), name)
            });
        }
    }

    fn handle_analysis_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.receiver = None;
            self.process_result(toasts, result, "Analysis", |this, resp| {
                this.controller.is_loading = false;
                this.controller.start_time = None; // Reset timer

                this.model.file_path = Some(resp.file_path);
                this.model.file_size = resp.file_size;
                this.model.summary = resp.summary;
                this.model.health = Some(resp.health);
                this.model.last_duration = Some(resp.duration);
                this.model.correlation_matrix = resp.correlation_matrix;
                this.model.df = Some(resp.df);
                this.should_scroll_to_top = true;

                // Reset cleaning configs if we just applied them, otherwise preserve (for trim changes)
                if this.controller.was_cleaning {
                    this.model.cleaning_configs.clear();
                }
                for col in &this.model.summary {
                    let is_new = !this.model.cleaning_configs.contains_key(&col.name);
                    let config = this
                        .model
                        .cleaning_configs
                        .entry(col.name.clone())
                        .or_default();

                    if is_new {
                        col.apply_advice_to_config(config);
                    }
                }

                // Auto-select first suitable ML target if none set
                if this.model.ml_target.is_none() {
                    this.model.ml_target = this
                        .model
                        .summary
                        .iter()
                        .find(|c| this.model.ml_model_kind.is_suitable_target(c.kind))
                        .map(|c| c.name.clone());
                }

                let path_display = this.model.file_path.as_deref().unwrap_or("file").to_owned();
                let secs = resp.duration.as_secs_f32();

                // Generate load summary
                let mut breakdown = std::collections::HashMap::new();
                for col in &this.model.summary {
                    *breakdown.entry(col.kind.as_str()).or_insert(0) += 1;
                }

                let mut breakdown_str = String::new();
                let mut keys: Vec<_> = breakdown.keys().collect();
                keys.sort();
                for key in keys {
                    let count = breakdown.get(key).unwrap_or(&0);
                    breakdown_str.push_str(&format!("\n - {key}: {count}"));
                }

                let row_count = this.model.summary.first().map(|c| c.count).unwrap_or(0);
                let row_count_fmt = crate::utils::fmt_num_human(row_count);
                let col_count = this.model.summary.len();
                let size_fmt = crate::utils::fmt_bytes(resp.file_size);

                let mut msg = format!(
                    "{row_count} ({row_count_fmt}) rows\n{col_count} columns\nBreakdown by Type:{breakdown_str}"
                );
                msg.push_str(&format!("\n\nFile Size: {size_fmt}"));

                if let Some(h) = &this.model.health {
                    msg.push_str(&format!("\nHealth Score: {:.0}%", h.score * 100.0));

                    if !h.risks.is_empty() {
                        msg.push_str("\n\nIdentified Risks:");
                        for risk in h.risks.iter().take(15) {
                            msg.push_str(&format!("\n • {risk}"));
                        }
                        if h.risks.len() > 15 {
                            msg.push_str("\n ... and more");
                        }
                    }
                }
                this.load_summary = Some(msg);
                this.summary_minimized = false;

                (format!("Analysed {path_display} in {secs:.2}s"), path_display)
            });
        }
    }

    fn handle_push_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .push_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.push_receiver = None;
            self.process_result(toasts, result, "Export", |this, _| {
                let duration = this.controller.push_start_time.take().map(|s| s.elapsed());
                this.model.push_last_duration = duration;
                this.controller.is_pushing = false;

                let secs = duration.map(|d| d.as_secs_f32()).unwrap_or(0.0);
                let table = this.model.pg_table.clone();
                (
                    format!("Successfully pushed to PostgreSQL in {secs:.2}s"),
                    format!("Table: {table}"),
                )
            });
        }
    }

    fn handle_test_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .test_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.test_receiver = None;
            self.process_result(toasts, result, "DB Test", |this, _| {
                this.controller.is_testing = false;
                let host = this.model.pg_host.clone();
                ("Database connection test successful!".to_owned(), host)
            });
        }
    }

    fn handle_training_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .training_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.training_receiver = None;
            self.process_result(toasts, result, "ML Training", |this, res| {
                this.controller.is_training = false;
                this.controller.training_start_time = None;
                let target = res.target_column.clone();
                this.model.ml_results = Some(res);
                (
                    "ML Training successful!".to_owned(),
                    format!("Target: {target}"),
                )
            });
        }
    }

    fn handle_export_receiver(&mut self, toasts: &mut egui_notify::Toasts) {
        if let Some(result) = self
            .controller
            .export_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.controller.export_receiver = None;
            self.process_result(toasts, result, "File Export", |this, _| {
                this.controller.is_exporting = false;
                (
                    "File exported successfully!".to_owned(),
                    "Cleaned data saved to disk".to_owned(),
                )
            });
        }
    }

    fn get_filtered_data(&self) -> (DataFrame, Vec<ColumnSummary>) {
        let Some(df) = &self.model.df else {
            return (DataFrame::default(), Vec::new());
        };

        let mut inactive_names: Vec<_> = self
            .model
            .cleaning_configs
            .iter()
            .filter(|(_, c)| !c.active)
            .map(|(n, _)| n)
            .collect();
        inactive_names.sort();

        if inactive_names.is_empty() {
            return (df.clone(), self.model.summary.clone());
        }

        let mut filtered_df = df.clone();
        for name in &inactive_names {
            if filtered_df.column(name).is_ok() {
                let _res = filtered_df.drop_in_place(name);
            }
        }

        let filtered_summary = self
            .model
            .summary
            .iter()
            .filter(|c| !inactive_names.contains(&&c.name))
            .cloned()
            .collect();

        (filtered_df, filtered_summary)
    }

    pub fn start_analysis(&mut self, ctx: egui::Context) {
        let path = FileDialog::new()
            .add_filter("Data Files", &["csv", "json", "jsonl", "ndjson", "parquet"])
            .pick_file();

        let Some(path) = path else { return };

        self.expanded_rows.clear();
        self.model.cleaning_configs.clear();
        self.status = format!("Loading: {}", path.display());
        self.log_action("Analysis Started", &path.display().to_string());
        self.controller
            .start_analysis(ctx, path, self.model.trim_pct);
    }

    pub fn trigger_reanalysis(&mut self, ctx: egui::Context) {
        let Some(df) = self.model.df.clone() else {
            return;
        };
        let Some(path_str) = self.model.file_path.clone() else {
            return;
        };

        self.status = format!(
            "Re-analysing with {:.0}% trim...",
            self.model.trim_pct * 100.0
        );
        self.log_action(
            "Re-analysis Started",
            &format!("{:.0}% trim", self.model.trim_pct * 100.0),
        );
        self.controller.trigger_reanalysis(
            ctx,
            df,
            path_str,
            self.model.file_size,
            self.model.trim_pct,
        );
    }

    pub fn start_cleaning(&mut self, ctx: egui::Context) {
        let Some(df) = self.model.df.clone() else {
            return;
        };
        let configs = self.model.cleaning_configs.clone();
        let trim_pct = self.model.trim_pct;
        let path_str = self.model.file_path.clone();
        let file_size = self.model.file_size;

        self.status = "Applying cleaning steps...".to_owned();
        self.log_action("Cleaning Started", "Applying custom rules");
        self.controller
            .start_cleaning(ctx, df, configs, trim_pct, path_str, file_size);
    }

    pub fn start_push_to_db(&mut self, ctx: egui::Context) {
        use secrecy::ExposeSecret as _;
        use sqlx::postgres::PgConnectOptions;

        let Some(file_path) = self.model.file_path.clone() else {
            return;
        };
        let file_size = self.model.file_size;
        let Some(health) = self.model.health.clone() else {
            return;
        };
        let (df, summary) = self.get_filtered_data();

        if self.model.pg_host.is_empty() || self.model.pg_database.is_empty() {
            self.status = "Error: Database connection not configured".to_owned();
            return;
        }

        let port: u16 = self.model.pg_port.parse().unwrap_or(5432);
        let pg_options = PgConnectOptions::new()
            .host(&self.model.pg_host)
            .port(port)
            .username(&self.model.pg_user)
            .password(self.model.pg_password.expose_secret())
            .database(&self.model.pg_database);

        let pg_schema = self.model.pg_schema.clone();
        let pg_table = self.model.pg_table.clone();

        self.status = "Connecting to PostgreSQL...".to_owned();
        self.log_action("Export Started", &format!("Table: {pg_table}"));
        self.controller.start_push_to_db(
            ctx, file_path, file_size, health, summary, df, pg_options, pg_schema, pg_table,
        );
    }

    pub fn start_test_connection(&mut self, ctx: egui::Context) {
        use secrecy::ExposeSecret as _;
        use sqlx::postgres::PgConnectOptions;

        let port: u16 = self.model.pg_port.parse().unwrap_or(5432);
        let pg_options = PgConnectOptions::new()
            .host(&self.model.pg_host)
            .port(port)
            .username(&self.model.pg_user)
            .password(self.model.pg_password.expose_secret())
            .database(&self.model.pg_database);

        self.status = "Testing database connection...".to_owned();
        let host = self.model.pg_host.clone();
        self.log_action("DB Test Started", &host);
        self.controller.start_test_connection(ctx, pg_options);
    }

    pub fn start_export(&mut self, ctx: egui::Context) {
        let path = FileDialog::new()
            .add_filter("CSV File", &["csv"])
            .add_filter("Parquet File", &["parquet"])
            .set_file_name("cleaned_data")
            .save_file();

        let Some(path) = path else { return };

        let (df, _) = self.get_filtered_data();

        self.status = format!("Exporting to: {}", path.display());
        self.log_action("Export Started", &path.display().to_string());
        self.controller.start_export(ctx, df, path);
    }

    pub fn start_secondary_analysis(&mut self, ctx: egui::Context) {
        let path = FileDialog::new()
            .add_filter("Data Files", &["csv", "json", "jsonl", "ndjson", "parquet"])
            .pick_file();

        let Some(path) = path else { return };

        // Clear previous secondary results before starting new load
        self.model.secondary_summary.clear();
        self.model.secondary_file_name = None;

        self.status = format!("Loading secondary: {}", path.display());
        self.log_action("Secondary Load Started", &path.display().to_string());
        self.controller
            .start_secondary_analysis(ctx, path, self.model.trim_pct);
    }

    pub fn perform_join(&mut self, ctx: egui::Context) {
        let (df1, _) = self.get_filtered_data();
        let Some(df2) = self.model.secondary_df.clone() else {
            return;
        };

        let primary_key = &self.model.join_key_primary;
        let secondary_key = &self.model.join_key_secondary;

        if primary_key.is_empty() || secondary_key.is_empty() {
            self.status = "Please select join keys for both files.".to_owned();
            return;
        }

        self.status = "Joining dataframes...".to_owned();

        let join_type = match self.model.join_type {
            super::model::MyJoinType::Inner => polars::prelude::JoinType::Inner,
            super::model::MyJoinType::Left => polars::prelude::JoinType::Left,
            super::model::MyJoinType::Outer => polars::prelude::JoinType::Full,
        };

        let result = df1.join(
            &df2,
            [primary_key],
            [secondary_key],
            polars::prelude::JoinArgs::new(join_type),
        );

        match result {
            Ok(joined_df) => {
                self.model.df = Some(joined_df);
                self.model.secondary_df = None;
                self.status = "Successfully joined dataframes.".to_owned();
                self.log_action(
                    "Data Join Performed",
                    &format!("Type: {:?}", self.model.join_type),
                );
                self.trigger_reanalysis(ctx);
            }
            Err(e) => {
                self.status = format!("Join Error: {e}");
                self.log_action("Join Failed", &e.to_string());
            }
        }
    }
}
