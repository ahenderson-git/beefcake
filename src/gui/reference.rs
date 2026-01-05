use eframe::egui;
use egui_phosphor::regular as icons;

pub fn render_reference_material(ctx: &egui::Context) -> bool {
    let mut go_back = false;
    egui::TopBottomPanel::top("ref_material_top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(format!("{} Back", icons::ARROW_LEFT)).clicked() {
                go_back = true;
            }
            ui.separator();
            ui.heading(format!("{} Analytical Reference Material", icons::BOOKS));
        });
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(egui::Margin {
            left: crate::theme::PANEL_LEFT as i8,
            right: crate::theme::PANEL_RIGHT as i8,
            top: 0,
            bottom: 0,
        }))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(crate::theme::SPACING_SMALL);

                crate::theme::card_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("Understanding Data Skewness");
                    ui.label("Skewness measures the asymmetry of the probability distribution of a real-valued random variable about its mean.");
                    ui.add_space(crate::theme::SPACING_TINY);
                    ui.label("• Right Skew (Positive): The mean is greater than the median. High-value outliers pull the average up.");
                    ui.label("• Left Skew (Negative): The mean is less than the median. Low-value outliers pull the average down.");
                });

                ui.add_space(crate::theme::SPACING_MEDIUM);

                crate::theme::card_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("Preprocessing for Machine Learning");
                    ui.add_space(crate::theme::SPACING_SMALL);
                    ui.collapsing("Normalization (Scaling)", |ui| {
                        ui.label("Required for many models (like Linear Regression or K-Means) to ensure features with different scales are treated equally.");
                        ui.label("• Min-Max: Rescales data to [0, 1]. Best when you know bounds and have few outliers.");
                        ui.label("• Z-Score: Rescales data to mean=0 and std_dev=1. More robust to outliers.");
                    });

                    ui.collapsing("Categorical Encoding", |ui| {
                        ui.label("ML models generally require numeric input. Encoding converts text categories to numbers.");
                        ui.label("• One-Hot: Creates binary columns for each category. Best for non-ordered data with low cardinality.");
                        ui.label("• Label: Assigns 1, 2, 3... to categories. Better for ordered data (Ordinal).");
                    });

                    ui.collapsing("Imputation (Handling Nulls)", |ui| {
                        ui.label("What to do with missing values?");
                        ui.label("• Mean/Median: Good for numeric fields with small % of missing values.");
                        ui.label("• Mode: Best for categorical fields.");
                        ui.label("• Constant (e.g., 0): Used when 'missing' has a business meaning (e.g., 0 sales).");
                    });
                });

                ui.add_space(crate::theme::SPACING_MEDIUM);

                crate::theme::card_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("PostgreSQL Export Guide");
                    ui.label("When pushing data to PostgreSQL, the application handles several steps automatically:");
                    ui.add_space(crate::theme::SPACING_TINY);
                    ui.label("1. Metadata: Saves file size, health score, and analysis timestamp.");
                    ui.label("2. Summaries: Saves column-level statistics and ML advice for later review.");
                    ui.label("3. Data: Creates a dedicated table for your cleaned dataset.");

                    ui.add_space(crate::theme::SPACING_MEDIUM);
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().faint_bg_color)
                        .show(ui, |ui| {
                            ui.label("Pro Tip: Always 'Test Connection' before attempting a full data push to verify your credentials and network path.");
                        });
                });

                ui.add_space(crate::theme::SPACING_MEDIUM);

                crate::theme::card_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading(format!("{} Useful Links & Resources", icons::GLOBE));
                    ui.add_space(crate::theme::SPACING_SMALL);

                    ui.columns(3, |columns| {
                        if let [c1, c2, c3] = columns {
                            c1.vertical(|ui| {
                                ui.strong(format!("{} Data Analysis", icons::CHART_BAR));
                                ui.hyperlink_to("Polars User Guide", "https://docs.pola.rs/user-guide/");
                                ui.hyperlink_to("Polars API (Rust)", "https://docs.rs/polars/latest/polars/");
                            });

                            c2.vertical(|ui| {
                                ui.strong(format!("{} UI Development", icons::PALETTE));
                                ui.hyperlink_to("egui Documentation", "https://docs.rs/egui/latest/egui/");
                                ui.hyperlink_to("egui Demo Gallery", "https://emilk.github.io/egui/");
                            });

                            c3.vertical(|ui| {
                                ui.strong(format!("{} Rust Programming", icons::GEAR));
                                ui.hyperlink_to("The Rust Book", "https://doc.rust-lang.org/book/");
                                ui.hyperlink_to("Rust by Example", "https://doc.rust-lang.org/rust-by-example/");
                            });
                        }
                    });

                    ui.add_space(crate::theme::SPACING_MEDIUM);
                    ui.separator();
                    ui.add_space(crate::theme::SPACING_SMALL);

                    render_training_and_ides(ui);
                });

                ui.add_space(crate::theme::SPACING_LARGE);
            });
        });

    go_back
}

fn render_training_and_ides(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.strong(format!("{} Training & Courses", icons::GRADUATION_CAP));
            ui.hyperlink_to(
                "Udemy: Learn to Code with Rust",
                "https://www.udemy.com/course/learn-to-code-with-rust/",
            );
        });

        ui.add_space(crate::theme::SPACING_HUGE);

        ui.vertical(|ui| {
            ui.strong(format!("{} IDEs & Tools", icons::WRENCH));
            ui.horizontal(|ui| {
                ui.hyperlink_to("RustRover", "https://www.jetbrains.com/rust/");
                ui.label("•");
                ui.hyperlink_to("VS Code", "https://code.visualstudio.com/");
            });
        });
    });
}
