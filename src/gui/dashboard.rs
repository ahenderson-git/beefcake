use crate::gui::TemplateApp;
use eframe::egui;
use egui_phosphor::regular as icons;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct ListItem {
    pub text: String,
    pub completed: bool,
}

impl TemplateApp {
    pub fn standard_todo_items() -> Vec<ListItem> {
        vec![
            ListItem {
                text: "Analyse high-value customer churn data".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Clean product inventory CSV for DB push".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Train linear regression on sales trends".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Verify PostgreSQL connection settings".to_owned(),
                completed: true,
            },
            ListItem {
                text: "Export cleaned transaction logs to Parquet".to_owned(),
                completed: false,
            },
        ]
    }

    pub fn standard_idea_items() -> Vec<ListItem> {
        vec![
            ListItem {
                text: "Implement automated outlier detection alerts".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Add support for AWS S3 bucket exports".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Develop a 'Quick View' dashboard for ML results".to_owned(),
                completed: false,
            },
            ListItem {
                text: "Support multi-file join operations in analyser".to_owned(),
                completed: true,
            },
            ListItem {
                text: "Create a dark mode theme for the GUI".to_owned(),
                completed: false,
            },
        ]
    }

    pub fn render_audit_log_panel(&mut self, ui: &mut egui::Ui) {
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading(format!("{} Activity Log", icons::CLOCK_COUNTER_CLOCKWISE));
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_salt("audit_log_scroll")
                .max_height(200.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if self.audit_log.is_empty() {
                        ui.label(egui::RichText::new("No activity yet.").weak());
                    } else {
                        for entry in self.audit_log.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(
                                        entry.timestamp.format("%H:%M:%S").to_string(),
                                    )
                                    .weak()
                                    .small(),
                                );
                                ui.label(egui::RichText::new(&entry.action).strong());
                                ui.label(&entry.details);
                            });
                        }
                    }
                });
            ui.add_space(8.0);
            if ui.button("Clear Log").clicked() {
                self.audit_log.clear();
            }
        });
    }

    pub fn render_dashboard_lists(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.columns(2, |columns| {
                if let [left, right] = columns {
                    self.render_todo_list(left);
                    self.render_ideas_list(right);
                }
            });
        });
    }

    pub fn render_todo_list(&mut self, ui: &mut egui::Ui) {
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading(format!("{} My TODOs", icons::CHECK_SQUARE));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.todo_input);
                if ((res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Add").clicked())
                    && !self.todo_input.is_empty()
                {
                    self.todo_list.push(ListItem {
                        text: self.todo_input.clone(),
                        completed: false,
                    });
                    self.log_action("Todo", &format!("Added: {}", self.todo_input));
                    self.todo_input.clear();
                }
            });

            ui.add_space(8.0);

            let mut to_remove = None;
            let mut log_entry = None;

            egui::ScrollArea::vertical()
                .id_salt("todo_scroll")
                .max_height(300.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (i, item) in self.todo_list.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut item.completed, "").changed() {
                                log_entry = Some((
                                    "Todo",
                                    format!(
                                        "Marked '{}' as {}",
                                        item.text,
                                        if item.completed { "done" } else { "pending" }
                                    ),
                                ));
                            }
                            let text_color = if item.completed {
                                ui.visuals().weak_text_color()
                            } else {
                                ui.visuals().text_color()
                            };
                            ui.label(egui::RichText::new(&item.text).color(text_color));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(icons::TRASH).clicked() {
                                        to_remove = Some(i);
                                    }
                                },
                            );
                        });
                    }
                });

            if let Some((action, details)) = log_entry {
                self.log_action(action, &details);
            }
            if let Some(i) = to_remove {
                let removed = self.todo_list.remove(i);
                self.log_action("Todo", &format!("Removed: {}", removed.text));
            }
        });
    }

    pub fn render_ideas_list(&mut self, ui: &mut egui::Ui) {
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading(format!("{} Future Ideas", icons::LIGHTBULB));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.idea_input);
                if ((res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Add").clicked())
                    && !self.idea_input.is_empty()
                {
                    self.ideas_list.push(ListItem {
                        text: self.idea_input.clone(),
                        completed: false,
                    });
                    self.log_action("Idea", &format!("Added: {}", self.idea_input));
                    self.idea_input.clear();
                }
            });

            ui.add_space(8.0);

            let mut to_remove = None;
            let mut log_entry = None;

            egui::ScrollArea::vertical()
                .id_salt("idea_scroll")
                .max_height(300.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (i, item) in self.ideas_list.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut item.completed, "").changed() {
                                log_entry = Some((
                                    "Idea",
                                    format!(
                                        "Marked '{}' as {}",
                                        item.text,
                                        if item.completed { "done" } else { "pending" }
                                    ),
                                ));
                            }
                            let text_color = if item.completed {
                                ui.visuals().weak_text_color()
                            } else {
                                ui.visuals().text_color()
                            };
                            ui.label(egui::RichText::new(&item.text).color(text_color));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(icons::TRASH).clicked() {
                                        to_remove = Some(i);
                                    }
                                },
                            );
                        });
                    }
                });

            if let Some((action, details)) = log_entry {
                self.log_action(action, &details);
            }
            if let Some(i) = to_remove {
                let removed = self.ideas_list.remove(i);
                self.log_action("Idea", &format!("Removed: {}", removed.text));
            }
        });
    }
}
