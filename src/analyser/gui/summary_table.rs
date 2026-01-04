use super::plots::render_distribution;
use crate::analyser::gui::App;
use crate::analyser::logic::types::ColumnKind;
use crate::analyser::logic::{
    BooleanStats, ColumnStats, ColumnSummary, NumericStats, TemporalStats, TextStats,
};
use crate::utils::fmt_opt;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icons;

pub fn render_summary_table(app: &mut App, ui: &mut egui::Ui) {
    let mut scroll_area = egui::ScrollArea::horizontal();
    if app.should_scroll_to_top {
        scroll_area = scroll_area.scroll_offset(egui::Vec2::ZERO);
    }

    scroll_area.show(ui, |ui| {
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Min))
            .column(Column::auto().at_least(30.0)) // Expand
            .column(Column::auto().at_least(30.0)) // Active/Exclude
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

        if app.should_scroll_to_top {
            table = table.scroll_to_row(0, None);
            app.should_scroll_to_top = false;
        }

        table
            .header(25.0, |mut header| {
                header.col(|ui| {
                    ui.strong(icons::CARET_UP_DOWN);
                });
                header.col(|ui| {
                    ui.strong(icons::CHECK_CIRCLE);
                });
                header.col(|ui| {
                    ui.strong("Name");
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
                    ui.strong("Stats & Samples");
                });
                header.col(|ui| {
                    ui.strong("Technical Summary");
                });
                header.col(|ui| {
                    ui.strong("Stakeholder Insight");
                });
                header.col(|ui| {
                    ui.strong("Histogram");
                });
                header.col(|_| {});
            })
            .body(|mut body| {
                for col in &app.model.summary.clone() {
                    let is_expanded = app.expanded_rows.contains(&col.name);
                    let row_height = (if is_expanded { 280.0 } else { 85.0 } as f32).max(35.0);

                    body.row(row_height, |row| {
                        render_column_row(app, row, col, is_expanded);
                    });
                }
            });
    });
}

fn render_column_row(
    app: &mut App,
    mut row: egui_extras::TableRow<'_, '_>,
    col: &ColumnSummary,
    is_expanded: bool,
) {
    let is_active = app
        .model
        .cleaning_configs
        .get(&col.name)
        .map(|c| c.active)
        .unwrap_or(true);

    row.col(|ui| {
        let icon = if is_expanded {
            icons::CARET_DOWN
        } else {
            icons::CARET_RIGHT
        };
        if ui
            .add(egui::Button::new(egui::RichText::new(icon).size(16.0)).frame(false))
            .clicked()
        {
            if is_expanded {
                app.expanded_rows.remove(&col.name);
            } else {
                app.expanded_rows.insert(col.name.clone());
            }
        }
    });
    row.col(|ui| {
        if let Some(config) = app.model.cleaning_configs.get_mut(&col.name) {
            ui.add_space(4.0);
            ui.checkbox(&mut config.active, "");
        }
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            render_name_editor(app, ui, col);
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            render_type_editor(app, ui, col);
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            let null_pct = if col.count > 0 {
                (col.nulls as f64 / col.count as f64) * 100.0
            } else {
                0.0
            };
            ui.label(format!("{} ({null_pct:.1}%)", col.nulls));
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            if col.has_special {
                ui.colored_label(egui::Color32::RED, format!("{} Yes", icons::WARNING));
            } else {
                ui.label("No");
            }
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            render_column_stats_cell(app, ui, col, is_expanded);
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            // Technical Summary Column
            ui.vertical(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for line in &col.interpretation {
                    ui.label(format!("• {line}"));
                }
            });
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            // Business Summary Column
            ui.vertical(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for line in &col.business_summary {
                    ui.label(format!("• {line}"));
                }
            });
        });
    });
    row.col(|ui| {
        ui.add_enabled_ui(is_active, |ui| {
            render_distribution(
                ui,
                &col.name,
                &col.stats,
                app.model.show_full_range,
                app.model.categorical_as_pie,
                is_expanded,
            );
        });
    });
    row.col(|_| {}); // Spacer
}

fn render_column_stats_cell(
    app: &mut App,
    ui: &mut egui::Ui,
    col: &ColumnSummary,
    is_expanded: bool,
) {
    ui.vertical(|ui| {
        // 1. Core Stats
        ui.scope(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            match &col.stats {
                ColumnStats::Numeric(s) => render_numeric_stats_info(app, ui, s),
                ColumnStats::Text(s) => render_text_stats_info(ui, s),
                ColumnStats::Categorical(freq) => {
                    render_categorical_stats_info(ui, freq, is_expanded, col.count);
                }
                ColumnStats::Temporal(s) => render_temporal_stats_info(ui, s),
                ColumnStats::Boolean(s) => render_boolean_stats_info(ui, s, col.count),
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
        render_advanced_metrics(ui, &col.stats);

        // 4. Advanced Cleaning (Only shown when expanded)
        if is_expanded {
            render_cleaning_controls(app, ui, col);
        }
    });
}

#[expect(clippy::too_many_lines)]
pub fn render_cleaning_controls(app: &mut App, ui: &mut egui::Ui, col: &ColumnSummary) {
    ui.add_space(8.0);
    ui.separator();

    if let Some(config) = app.model.cleaning_configs.get_mut(&col.name) {
        // --- 1. Advanced Cleaning Section ---
        ui.horizontal(|ui| {
            ui.checkbox(&mut config.advanced_cleaning, "")
                .on_hover_text("Enable or disable all advanced cleaning steps for this column.");
            ui.strong(format!("{} Advanced Cleaning:", icons::SPARKLE));

            ui.add_enabled_ui(config.advanced_cleaning, |ui| {
                ui.checkbox(&mut config.trim_whitespace, "Trim");
                ui.checkbox(&mut config.remove_special_chars, "Special");
                ui.checkbox(&mut config.remove_non_ascii, "Non-ASCII");
                ui.checkbox(&mut config.standardize_nulls, "Std Nulls");

                egui::ComboBox::from_id_salt(format!("case_{}", col.name))
                    .selected_text(match config.text_case {
                        crate::analyser::logic::types::TextCase::None => "Original",
                        crate::analyser::logic::types::TextCase::Lowercase => "lower",
                        crate::analyser::logic::types::TextCase::Uppercase => "UPPER",
                        crate::analyser::logic::types::TextCase::TitleCase => "Title",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut config.text_case,
                            crate::analyser::logic::types::TextCase::None,
                            "Original",
                        );
                        ui.selectable_value(
                            &mut config.text_case,
                            crate::analyser::logic::types::TextCase::Lowercase,
                            "lowercase",
                        );
                        ui.selectable_value(
                            &mut config.text_case,
                            crate::analyser::logic::types::TextCase::Uppercase,
                            "UPPERCASE",
                        );
                    });

                ui.separator();
                ui.label("Regex:");
                ui.add(
                    egui::TextEdit::singleline(&mut config.regex_find)
                        .hint_text("Find")
                        .desired_width(60.0),
                );
                ui.label("->");
                ui.add(
                    egui::TextEdit::singleline(&mut config.regex_replace)
                        .hint_text("Replace")
                        .desired_width(60.0),
                );
            });
        });

        ui.add_space(4.0);

        // --- 2. ML Preprocessing Section ---
        ui.horizontal(|ui| {
            ui.checkbox(&mut config.ml_preprocessing, "")
                .on_hover_text("Enable or disable all ML preprocessing steps for this column.");
            ui.strong(format!("{} ML Preprocessing:", icons::BRAIN));

            ui.add_enabled_ui(config.ml_preprocessing, |ui| {
                let effective_kind = config.target_dtype.unwrap_or(col.kind);

                // --- Imputation ---
                let suggest_impute = col.ml_advice.iter().any(|a| a.contains("Mean or Median Imputation"));
                if suggest_impute && config.impute_mode == crate::analyser::logic::types::ImputeMode::Mean {
                    ui.label(egui::RichText::new(icons::WARNING).color(egui::Color32::YELLOW))
                        .on_hover_text("Auto-selected Mean imputation based on missing data advice. Please review.");
                }

                egui::ComboBox::from_id_salt(format!("impute_{}", col.name))
                    .selected_text(match config.impute_mode {
                        crate::analyser::logic::types::ImputeMode::None => "No Imputation",
                        crate::analyser::logic::types::ImputeMode::Mean => "Fill with Mean",
                        crate::analyser::logic::types::ImputeMode::Median => "Fill with Median",
                        crate::analyser::logic::types::ImputeMode::Zero => "Fill with Zero",
                        crate::analyser::logic::types::ImputeMode::Mode => "Fill with Mode",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut config.impute_mode,
                            crate::analyser::logic::types::ImputeMode::None,
                            "No Imputation",
                        );

                        if effective_kind == crate::analyser::logic::types::ColumnKind::Numeric {
                            ui.selectable_value(
                                &mut config.impute_mode,
                                crate::analyser::logic::types::ImputeMode::Mean,
                                "Fill with Mean",
                            );
                            ui.selectable_value(
                                &mut config.impute_mode,
                                crate::analyser::logic::types::ImputeMode::Median,
                                "Fill with Median",
                            );
                            ui.selectable_value(
                                &mut config.impute_mode,
                                crate::analyser::logic::types::ImputeMode::Zero,
                                "Fill with Zero",
                            );
                        }

                        if effective_kind == crate::analyser::logic::types::ColumnKind::Categorical {
                            ui.selectable_value(
                                &mut config.impute_mode,
                                crate::analyser::logic::types::ImputeMode::Mode,
                                "Fill with Mode",
                            );
                        }
                    });

                // Auto-reset invalid imputation modes
                match config.impute_mode {
                    crate::analyser::logic::types::ImputeMode::Mean
                    | crate::analyser::logic::types::ImputeMode::Median
                    | crate::analyser::logic::types::ImputeMode::Zero
                        if effective_kind != crate::analyser::logic::types::ColumnKind::Numeric =>
                    {
                        config.impute_mode = crate::analyser::logic::types::ImputeMode::None;
                    }
                    crate::analyser::logic::types::ImputeMode::Mode
                        if effective_kind != crate::analyser::logic::types::ColumnKind::Categorical =>
                    {
                        config.impute_mode = crate::analyser::logic::types::ImputeMode::None;
                    }
                    _ => {}
                }

                // --- Normalization & Numeric Refinement ---
                if effective_kind == crate::analyser::logic::types::ColumnKind::Numeric {
                    ui.separator();
                    egui::ComboBox::from_id_salt(format!("norm_{}", col.name))
                        .selected_text(match config.normalization {
                            crate::analyser::logic::types::NormalizationMethod::None => "No Scaling",
                            crate::analyser::logic::types::NormalizationMethod::ZScore => "Z-Score (Std)",
                            crate::analyser::logic::types::NormalizationMethod::MinMax => "Min-Max (0-1)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut config.normalization, crate::analyser::logic::types::NormalizationMethod::None, "No Scaling");
                            ui.selectable_value(&mut config.normalization, crate::analyser::logic::types::NormalizationMethod::ZScore, "Z-Score (Std)");
                            ui.selectable_value(&mut config.normalization, crate::analyser::logic::types::NormalizationMethod::MinMax, "Min-Max (0-1)");
                        });

                    ui.separator();
                    ui.checkbox(&mut config.extract_numbers, "Extract Num");
                    ui.checkbox(&mut config.clip_outliers, "Clip");

                    let mut rounding_active = config.rounding.is_some();
                    if ui.checkbox(&mut rounding_active, "Round").changed() {
                        config.rounding = if rounding_active { Some(2) } else { None };
                    }
                    if let Some(val) = &mut config.rounding {
                        ui.add(egui::DragValue::new(val).range(0..=10).suffix(" d"));
                    }
                } else {
                    config.normalization = crate::analyser::logic::types::NormalizationMethod::None;
                    config.extract_numbers = false;
                    config.clip_outliers = false;
                    config.rounding = None;
                }

                // --- One-Hot Encoding & Categorical Refinement ---
                if effective_kind == crate::analyser::logic::types::ColumnKind::Categorical {
                    ui.separator();
                    ui.checkbox(&mut config.one_hot_encode, "One-Hot Encode")
                        .on_hover_text("Convert categorical values into multiple binary columns.");

                    ui.separator();
                    let mut cap_active = config.freq_threshold.is_some();
                    if ui.checkbox(&mut cap_active, "Freq Cap")
                        .on_hover_text("Group rare categories (appearing less than the threshold) into an 'Other' category to prevent a 'curse of dimensionality' in ML models.")
                        .changed()
                    {
                        config.freq_threshold = if cap_active { Some(5) } else { None };
                    }
                    if let Some(val) = &mut config.freq_threshold {
                        ui.add(egui::DragValue::new(val).range(1..=1000).prefix("min:"));
                    }
                } else if effective_kind == crate::analyser::logic::types::ColumnKind::Boolean {
                    ui.separator();
                    ui.checkbox(&mut config.one_hot_encode, "One-Hot Encode");
                    config.freq_threshold = None;
                } else {
                    config.one_hot_encode = false;
                    config.freq_threshold = None;
                }

                // --- Temporal Format ---
                if effective_kind == crate::analyser::logic::types::ColumnKind::Temporal {
                    ui.separator();
                    ui.label("Format:");
                    ui.add(egui::TextEdit::singleline(&mut config.temporal_format).hint_text("%Y-%m-%d").desired_width(80.0));
                    ui.checkbox(&mut config.timezone_utc, "UTC").on_hover_text("Normalize to UTC timezone.");
                } else {
                    config.temporal_format = String::new();
                    config.timezone_utc = false;
                }
            });
        });

        ui.add_space(4.0);

        // --- 3. Smart ML Advice ---
        ui.horizontal(|ui| {
            ui.label(icons::LIGHTBULB);
            ui.strong("Smart ML Advice:");
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for advice in &col.ml_advice {
                    ui.label(
                        egui::RichText::new(format!("• {advice}"))
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                }
            });
        });
    }
}

fn render_numeric_stats_info(app: &App, ui: &mut egui::Ui, s: &NumericStats) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Range:").on_hover_text("The full spread of values from minimum to maximum.");
            ui.label(format!("{} to {}", fmt_opt(s.min), fmt_opt(s.max)));
            ui.label("|");
            ui.label("P05-P95:").on_hover_text("The range of the middle 90% of your data. Helps identify the typical range while ignoring extreme outliers.");
            ui.label(format!("{}-{}", fmt_opt(s.p05), fmt_opt(s.p95)));
        });
        ui.horizontal(|ui| {
            ui.label("Mean:").on_hover_text("The mathematical average. If significantly different from the median, the data is skewed.");
            ui.label(fmt_opt(s.mean));
            ui.label("|");
            ui.label("Median:").on_hover_text("The middle value. 50% of the data is above this, and 50% is below.");
            ui.label(fmt_opt(s.median));
            ui.label("|");
            ui.label("Skew:").on_hover_text("Measures lack of symmetry. Positive = tail on right, Negative = tail on left. |Skew| > 1 is high.");
            ui.label(fmt_opt(s.skew));

            if let (Some(mean), Some(median)) = (s.mean, s.median) {
                if median.abs() > 1e-9 && (mean - median).abs() / median.abs() > 0.1 {
                    ui.label(egui::RichText::new(icons::WARNING).color(egui::Color32::YELLOW))
                        .on_hover_text("Gap > 10%: Outliers likely influencing the average");
                }
            }
        });
        ui.horizontal(|ui| {
            let pct = app.model.trim_pct * 100.0;
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
                        ui.label(egui::RichText::new(icons::WARNING).color(egui::Color32::YELLOW))
                            .on_hover_text("Standard deviation may be less reliable because the mean is heavily influenced by outliers or skew.");
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
            ui.label("Distinct:")
                .on_hover_text("The number of unique values in this column.");
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
            ui.label("Categories:")
                .on_hover_text("The number of unique groups detected.");
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
        if let (Some(p05), Some(p95)) = (s.p05, s.p95) {
            ui.label(format!("90% Range (TS): {p05:.0}..{p95:.0}"))
                .on_hover_text("Range excluding the top and bottom 5% of timestamps.");
        }
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
                    ui.label(
                        egui::RichText::new(format!("{} Sorted", icons::CHECK_CIRCLE))
                            .color(egui::Color32::from_rgb(0, 150, 0)),
                    )
                    .on_hover_text("Values are strictly increasing.");
                    first = false;
                } else if s.is_sorted_rev {
                    ui.label(
                        egui::RichText::new(format!("{} Sorted (Rev)", icons::CHECK_CIRCLE))
                            .color(egui::Color32::from_rgb(0, 150, 0)),
                    )
                    .on_hover_text("Values are strictly decreasing.");
                    first = false;
                }

                if s.is_integer {
                    if !first {
                        ui.label("|");
                    }
                    ui.label("Integers")
                        .on_hover_text("All values are whole numbers.");
                    first = false;
                }

                if s.zero_count > 0 {
                    if !first {
                        ui.label("|");
                    }
                    ui.label(format!("Zeros: {}", s.zero_count));
                    first = false;
                }

                if s.negative_count > 0 {
                    if !first {
                        ui.label("|");
                    }
                    ui.label(format!("Negatives: {}", s.negative_count));
                }
            });
        }
        ColumnStats::Text(s) => {
            ui.label(format!(
                "Lengths: {} / {} / {:.1}",
                s.min_length, s.max_length, s.avg_length
            ))
            .on_hover_text("Character length of the text entries (Min / Max / Avg).");
        }
        ColumnStats::Temporal(s) => {
            if s.is_sorted {
                ui.label(
                    egui::RichText::new(format!("{} Chronological", icons::CHECK_CIRCLE))
                        .color(egui::Color32::from_rgb(0, 150, 0)),
                )
                .on_hover_text("Values are in strictly increasing chronological order.");
            } else if s.is_sorted_rev {
                ui.label(
                    egui::RichText::new(format!("{} Reverse Chronological", icons::CHECK_CIRCLE))
                        .color(egui::Color32::from_rgb(0, 150, 0)),
                )
                .on_hover_text("Values are in strictly decreasing chronological order.");
            }
        }
        _ => {}
    }
}

fn render_name_editor(app: &mut App, ui: &mut egui::Ui, col: &ColumnSummary) {
    let text_color = if ui.visuals().dark_mode {
        egui::Color32::LIGHT_GRAY
    } else {
        egui::Color32::BLACK
    };

    if let Some(config) = app.model.cleaning_configs.get_mut(&col.name) {
        ui.add(
            egui::TextEdit::singleline(&mut config.new_name)
                .hint_text(egui::RichText::new(&col.name).strong().color(text_color))
                .desired_width(f32::INFINITY),
        );
    } else {
        ui.label(egui::RichText::new(&col.name).strong().color(text_color));
    }
}

fn render_type_editor(app: &mut App, ui: &mut egui::Ui, col: &ColumnSummary) {
    if let Some(config) = app.model.cleaning_configs.get_mut(&col.name) {
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
                    ColumnKind::Numeric,
                    ColumnKind::Text,
                    ColumnKind::Boolean,
                    ColumnKind::Temporal,
                    ColumnKind::Categorical,
                ];

                for target in targets {
                    if target != col.kind && col.is_compatible_with(target) {
                        ui.selectable_value(
                            &mut config.target_dtype,
                            Some(target),
                            target.as_str(),
                        );
                    }
                }
            });
    } else {
        ui.label(col.kind.as_str());
    }
}
