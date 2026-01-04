use crate::analyser::gui::App;
use crate::analyser::logic::types::{ColumnKind, MlModelKind};
use eframe::egui;
use egui_phosphor::regular as icons;
use std::sync::atomic::Ordering;

pub fn render_controls(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // --- Group: File ---
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icons::FILE).size(18.0).color(crate::theme::ACCENT_COLOR));
                if ui.button("Open File").clicked() && !app.controller.is_loading {
                    app.start_analysis(ctx.clone());
                }
            });
        });

        // --- Group: Analysis Settings ---
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icons::CHART_LINE).size(18.0).color(crate::theme::ACCENT_COLOR));
                ui.label("Analysis:");
                ui.label("Trim %:").on_hover_text(
                    "Percentage to trim from EACH end of the data for the Trimmed Mean calculation.",
                );
                let slider = ui.add(
                    egui::Slider::new(&mut app.model.trim_pct, 0.0..=0.2)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                );

                if slider.changed() && !app.controller.is_loading && app.model.df.is_some() {
                    app.trigger_reanalysis(ctx.clone());
                }

                ui.separator();
                ui.checkbox(&mut app.model.show_full_range, "Full Range")
                    .on_hover_text("If unchecked, the histogram zooms into the 5th-95th percentile to avoid being 'crushed' by extreme outliers.");
                ui.separator();
                ui.checkbox(&mut app.model.categorical_as_pie, "Pie Charts")
                    .on_hover_text("Show categorical data as Pie charts instead of Bar charts.");
            });
        });

        // --- Group: View Controls ---
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icons::EYE).size(18.0).color(crate::theme::ACCENT_COLOR));
                ui.label("View:");
                if ui.button("Expand All").clicked() {
                    for col in &app.model.summary {
                        app.expanded_rows.insert(col.name.clone());
                    }
                }
                if ui.button("Collapse All").clicked() {
                    app.expanded_rows.clear();
                }
            });
        });

        // --- Group: Data Cleaning ---
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icons::SPARKLE).size(18.0).color(crate::theme::ACCENT_COLOR));
                ui.label("Cleaning:");
                if ui
                    .button("Apply Cleaning")
                    .on_hover_text("Apply the selected cleaning actions to the data and re-analyse.")
                    .clicked()
                    && !app.controller.is_loading
                    && app.model.df.is_some()
                {
                    app.start_cleaning(ctx.clone());
                }

                if ui
                    .button(format!("{} Export to File", icons::FLOPPY_DISK))
                    .on_hover_text("Export the currently cleaned and preprocessed data to CSV or Parquet.")
                    .clicked()
                    && !app.controller.is_exporting
                    && app.model.df.is_some()
                {
                    app.start_export(ctx.clone());
                }
            });
        });

        render_joins_group(app, ui, ctx);
        render_status_and_timing(app, ui);
    });
}

fn render_joins_group(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    // --- Group: Data Joins ---
    crate::theme::card_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(icons::LINK)
                    .size(18.0)
                    .color(crate::theme::ACCENT_COLOR),
            );
            ui.label("Joins:");
            if ui
                .button(format!("{} Load 2nd File", icons::FOLDER_OPEN))
                .on_hover_text("Load another file to join with the current data.")
                .clicked()
            {
                app.start_secondary_analysis(ctx.clone());
            }

            if app.model.secondary_df.is_some() {
                ui.separator();
                ui.label("Join on:");
                // Primary key selection
                egui::ComboBox::from_id_salt("primary_key")
                    .selected_text(if app.model.join_key_primary.is_empty() {
                        "Primary Key"
                    } else {
                        &app.model.join_key_primary
                    })
                    .show_ui(ui, |ui| {
                        for col in &app.model.summary {
                            ui.selectable_value(
                                &mut app.model.join_key_primary,
                                col.name.clone(),
                                &col.name,
                            );
                        }
                    });

                ui.label("=");

                // Secondary key selection
                egui::ComboBox::from_id_salt("secondary_key")
                    .selected_text(if app.model.join_key_secondary.is_empty() {
                        "Secondary Key"
                    } else {
                        &app.model.join_key_secondary
                    })
                    .show_ui(ui, |ui| {
                        for col in &app.model.secondary_summary {
                            ui.selectable_value(
                                &mut app.model.join_key_secondary,
                                col.name.clone(),
                                &col.name,
                            );
                        }
                    });

                ui.separator();
                egui::ComboBox::from_id_salt("join_type")
                    .selected_text(format!("{:?}", app.model.join_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut app.model.join_type,
                            crate::analyser::model::MyJoinType::Inner,
                            "Inner Join",
                        );
                        ui.selectable_value(
                            &mut app.model.join_type,
                            crate::analyser::model::MyJoinType::Left,
                            "Left Join",
                        );
                        ui.selectable_value(
                            &mut app.model.join_type,
                            crate::analyser::model::MyJoinType::Outer,
                            "Outer (Full) Join",
                        );
                    });

                ui.separator();
                if ui
                    .add(
                        egui::Button::new(format!("{} Join Now", icons::LIGHTNING))
                            .fill(egui::Color32::from_rgb(60, 120, 200)),
                    )
                    .clicked()
                {
                    app.perform_join(ctx.clone());
                }
            } else if !app.model.secondary_summary.is_empty() {
                ui.label(" (Joining...)");
            }
        });
    });
}

fn render_status_and_timing(app: &App, ui: &mut egui::Ui) {
    // --- Group: Status & Timing ---
    crate::theme::card_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            if app.controller.is_loading {
                ui.add(egui::Spinner::new());
                let elapsed = app
                    .controller
                    .start_time
                    .map(|s| s.elapsed().as_secs_f32())
                    .unwrap_or(0.0);
                ui.label(format!("Analysing... ({elapsed:.1}s)"));

                let progress = app
                    .controller
                    .progress_counter
                    .load(std::sync::atomic::Ordering::SeqCst);
                if progress > 0 {
                    ui.label(format!("- {}", crate::utils::fmt_bytes(progress)));
                }
            } else if app.controller.is_pushing {
                ui.add(egui::Spinner::new());
                ui.label("Pushing to DB...");
            } else if app.controller.is_exporting {
                ui.add(egui::Spinner::new());
                ui.label("Exporting...");
            } else if let Some(duration) = app.model.last_duration {
                ui.label(
                    egui::RichText::new(format!("{} Ready", icons::CHECK_CIRCLE))
                        .color(egui::Color32::from_rgb(102, 187, 106)),
                );
                ui.separator();
                ui.label(egui::RichText::new(format!("{:.2}s", duration.as_secs_f32())).weak());
            } else {
                ui.label(egui::RichText::new(format!("{} Ready", icons::INFO)).weak());
            }
        });
    });
}

pub fn render_db_config(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    crate::theme::card_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::CollapsingHeader::new(
            egui::RichText::new(format!("{} Database Export (PostgreSQL)", icons::DATABASE))
                .strong(),
        )
        .default_open(false)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Schema:");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.model.pg_schema).desired_width(100.0),
                    );
                    ui.add_space(10.0);
                    ui.label("Table:");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.model.pg_table).desired_width(150.0),
                    );

                    ui.add_space(20.0);
                    let can_push = app.model.df.is_some() && !app.model.pg_table.is_empty();
                    let push_btn = ui.add_enabled(
                        can_push,
                        egui::Button::new(format!("{} Push to DB", icons::ROCKET)),
                    );
                    if push_btn.clicked() {
                        app.start_push_to_db(ctx.clone());
                    }

                    if app.controller.is_pushing {
                        ui.add(egui::Spinner::new());
                    }
                });

                if let Some(dur) = app.model.push_last_duration {
                    ui.label(
                        egui::RichText::new(format!(
                            "Last Export Success: {:.2}s",
                            dur.as_secs_f32()
                        ))
                        .weak()
                        .size(10.0),
                    );
                }
            });
        });
    });
}

pub fn render_ml_panel(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    crate::theme::card_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::CollapsingHeader::new(egui::RichText::new(format!("{} Machine Learning (Beta)", icons::BRAIN)).strong())
            .default_open(false)
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label("Train basic models on your cleaned data. Categorical columns must be One-Hot encoded first.");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        // 1. Select Model Kind
                        ui.label("Model:");
                        let old_kind = app.model.ml_model_kind;
                        egui::ComboBox::from_id_salt("ml_model_kind")
                            .selected_text(app.model.ml_model_kind.as_str())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.model.ml_model_kind, MlModelKind::LinearRegression, "Linear Regression");
                                ui.selectable_value(&mut app.model.ml_model_kind, MlModelKind::LogisticRegression, "Logistic Regression");
                                ui.selectable_value(&mut app.model.ml_model_kind, MlModelKind::DecisionTree, "Decision Tree");
                            });

                        if app.model.ml_model_kind != old_kind {
                            app.model.ml_target = app.model.summary.iter()
                                .find(|c| app.model.ml_model_kind.is_suitable_target(c.kind))
                                .map(|c| c.name.clone());
                        }

                        ui.add_space(10.0);

                        // 2. Select Target Column
                        ui.label("Target:");
                        egui::ComboBox::from_id_salt("ml_target_col")
                            .selected_text(app.model.ml_target.as_deref().unwrap_or("Select Target"))
                            .show_ui(ui, |ui| {
                                for col in &app.model.summary {
                                    if app.model.ml_model_kind.is_suitable_target(col.kind) {
                                        ui.selectable_value(&mut app.model.ml_target, Some(col.name.clone()), &col.name);
                                    }
                                }
                            });

                        ui.add_space(10.0);

                        // 3. Train Button
                        if app.controller.is_training {
                            ui.horizontal(|ui| {
                                let pct = app.controller.training_progress.load(Ordering::SeqCst) as f32 / 100.0;
                                ui.add(egui::ProgressBar::new(pct)
                                    .show_percentage()
                                    .animate(true)
                                    .text("Training Model..."));

                                if let Some(start) = app.controller.training_start_time {
                                    ui.label(egui::RichText::new(format!("{:.1}s", start.elapsed().as_secs_f32())).strong());
                                }
                            });
                        } else {
                            let n_features = app.model.summary.iter()
                                .filter(|c| c.name != app.model.ml_target.as_deref().unwrap_or(""))
                                .filter(|c| matches!(c.kind, ColumnKind::Numeric | ColumnKind::Boolean))
                                .count();

                            let can_train = app.model.df.is_some() && app.model.ml_target.is_some() && n_features > 0;

                            ui.horizontal(|ui| {
                                let btn = ui.add_enabled(can_train, egui::Button::new(format!("{} Train Model", icons::ROCKET_LAUNCH)));
                                if btn.clicked() {
                                    if let Some(target) = app.model.ml_target.clone() {
                                        let (filtered_df, _) = app.get_filtered_data();
                                        app.controller.start_training(ctx.clone(), filtered_df, target.clone(), app.model.ml_model_kind);
                                        app.log_action("ML Training Started", &format!("Target: {target}"));
                                    }
                                }

                                if n_features == 0 && app.model.ml_target.is_some() {
                                    ui.label(egui::RichText::new(format!("{} No features", icons::WARNING)).color(egui::Color32::RED))
                                        .on_hover_text("No numeric or boolean columns found to use as features. Try using 'Apply Cleaning' to convert categories to One-Hot encoding first.");
                                } else {
                                    ui.label(format!("({n_features} features)"))
                                        .on_hover_text("Number of numeric/boolean columns that will be used to predict the target.");
                                }
                            });
                        }

                        // 4. Show Results if any
                        if let Some(res) = &app.model.ml_results {
                            ui.separator();
                            ui.label("Last Result:");
                            if let Some(r2) = res.r2_score {
                                ui.label(format!("R²: {r2:.4}")).on_hover_text("Coefficient of Determination: 1.0 is perfect prediction.");
                            }
                            if let Some(acc) = res.accuracy {
                                ui.label(format!("Acc: {:.2}%", acc * 100.0));
                            }
                            if let Some(mse) = res.mse {
                                ui.label(format!("MSE: {mse:.4}")).on_hover_text("Mean Squared Error: Lower is better.");
                            }

                            if ui.button(format!("{} View Details", icons::EYE)).clicked() {
                                app.show_ml_details = true;
                            }
                        }
                    });
                });
            });
    });
}

pub fn render_ml_details_window(app: &mut App, ctx: &egui::Context) {
    let Some(res) = &app.model.ml_results else {
        return;
    };

    egui::Window::new(format!("{} ML Model Details", icons::BRAIN))
        .open(&mut app.show_ml_details)
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading(format!("Model: {}", res.model_kind.as_str()));
                ui.label(format!("Target Column: {}", res.target_column));
                ui.label(format!(
                    "Training Duration: {:.2}s",
                    res.duration.as_secs_f32()
                ));

                ui.separator();
                ui.strong("Metrics:");
                if let Some(r2) = res.r2_score {
                    ui.label(format!("• R² Score: {r2:.6}"));
                }
                if let Some(acc) = res.accuracy {
                    ui.label(format!("• Accuracy: {:.2}%", acc * 100.0));
                }
                if let Some(mse) = res.mse {
                    ui.label(format!("• Mean Squared Error: {mse:.6}"));
                }

                if !res.interpretation.is_empty() {
                    ui.separator();
                    ui.strong("Interpretation:");
                    for line in &res.interpretation {
                        ui.label(format!("• {line}"));
                    }
                }

                ui.separator();
                ui.strong("Features:");
                ui.label(res.feature_columns.join(", "));

                if let Some(coeffs) = &res.coefficients {
                    ui.separator();
                    ui.strong("Coefficients:");
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            egui::Grid::new("coeffs_grid").striped(true).show(ui, |ui| {
                                let mut sorted_coeffs: Vec<_> = coeffs.iter().collect();
                                sorted_coeffs.sort_by(|a, b| {
                                    b.1.abs()
                                        .partial_cmp(&a.1.abs())
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });

                                for (name, val) in sorted_coeffs {
                                    ui.label(name);
                                    ui.label(format!("{val:.6}"));
                                    ui.end_row();
                                }
                            });
                        });
                    if let Some(intercept) = res.intercept {
                        ui.label(format!("Intercept: {intercept:.6}"));
                    }
                }
            });
        });
}
