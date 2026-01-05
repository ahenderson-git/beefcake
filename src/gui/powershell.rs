use crate::gui::BeefcakeApp;
use eframe::egui;
use egui_phosphor::regular as icons;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize, Serialize, Clone)]
pub struct PsScript {
    pub name: String,
    pub content: String,
}

impl BeefcakeApp {
    pub fn default_scripts() -> Vec<PsScript> {
        vec![
            PsScript {
                name: "Get System Info".to_owned(),
                content: "Get-ComputerInfo | Select-Object WindowsVersion, OsName, CsProcessors".to_owned(),
            },
            PsScript {
                name: "List Large Files".to_owned(),
                content: "Get-ChildItem -Path C:\\ -File -Recurse -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 10 Name, @{Name=\"Size(GB)\";Expression={$_.Length / 1GB}}".to_owned(),
            },
            PsScript {
                name: "Watch Real-time Logs".to_owned(),
                content: "Get-Content -Path \".\\.output.txt\" -Wait -Tail 10".to_owned(),
            },
            PsScript {
                name: "Check Postgres Service".to_owned(),
                content: "Get-Service -Name postgres*".to_owned(),
            },
            PsScript {
                name: "Network Diagnostics".to_owned(),
                content: "Test-NetConnection -ComputerName google.com -Port 443".to_owned(),
            },
        ]
    }

    pub fn render_powershell_module(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;
        egui::TopBottomPanel::top("ps_module_top")
            .frame(crate::theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("{} Back", icons::ARROW_LEFT)).clicked() {
                        go_back = true;
                    }
                    ui.separator();
                    ui.heading(format!("{} PowerShell Automation Module", icons::TERMINAL));
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
                egui::ScrollArea::vertical()
                    .id_salt("ps_module_scroll")
                    .show(ui, |ui| {
                        ui.columns(2, |columns| {
                            if let [left, right] = columns {
                                self.render_ps_editor(left);
                                self.render_ps_library(right);
                            }
                        });

                        ui.add_space(crate::theme::MARGIN_SIDEBAR);
                        crate::utils::render_status_message(ui, &self.status);

                        if !self.ps_last_output.is_empty() {
                            ui.add_space(crate::theme::MARGIN_SIDEBAR);
                            crate::theme::card_frame(ui).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_height(ui.available_height());
                                egui::CollapsingHeader::new(egui::RichText::new(format!("{} Last Execution Output", icons::LIST_DASHES)).strong())
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button(format!("{} Clear", icons::TRASH)).clicked() {
                                                    self.ps_last_output.clear();
                                                }
                                                if ui.small_button(format!("{} Copy", icons::COPY)).clicked() {
                                                    ui.ctx().copy_text(self.ps_last_output.clone());
                                                    self.toasts.success("Output copied to clipboard");
                                                }
                                            });
                                        });
                                        ui.add_space(crate::theme::SPACING_TINY);
                                        egui::ScrollArea::vertical()
                                            .id_salt("ps_output_scroll")
                                            .auto_shrink([false; 2])
                                            .show(ui, |ui| {
                                                let mut output = self.ps_last_output.clone();
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut output)
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_width(f32::INFINITY)
                                                        .desired_rows(10)
                                                        .lock_focus(true)
                                                        .interactive(true),
                                                );
                                            });
                                    });
                            });
                        }
                        ui.add_space(crate::theme::SPACING_LARGE);
                    });
            });

        go_back
    }

    pub fn render_ps_editor(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(format!("{} Script Editor", icons::NOTE_PENCIL));
            ui.add_space(crate::theme::SPACING_TINY);

            let theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
            let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                    ui.ctx(),
                    ui.style(),
                    &theme,
                    string.as_str(),
                    "ps1",
                );
                layout_job.wrap.max_width = wrap_width;
                ui.painter().layout_job(layout_job)
            };

            egui::ScrollArea::vertical()
                .id_salt("ps_editor_scroll")
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.ps_script)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .lock_focus(true)
                            .layouter(&mut layouter),
                    );
                });

            ui.add_space(crate::theme::SPACING_SMALL);
            self.render_ps_controls(ui);
        });
    }

    pub fn render_ps_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.is_running_ps {
                ui.add(egui::Spinner::new());
                ui.label("Running script...");
            } else if ui
                .button(format!("{} Run Script", icons::PLAY))
                .on_hover_text("Execute script and wait for result")
                .clicked()
            {
                self.start_ps_execution(ui.ctx().clone(), self.ps_script.clone(), None);
                self.log_action("PowerShell", "Started script execution");
            }

            ui.vertical(|ui| {
                if ui
                    .button(format!("{} Save to Library", icons::FLOPPY_DISK))
                    .clicked()
                {
                    self.show_ps_save_ui = !self.show_ps_save_ui;
                }

                if self.show_ps_save_ui {
                    ui.add_space(crate::theme::SPACING_TINY);
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().faint_bg_color)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label("Script Name:");
                                ui.text_edit_singleline(&mut self.ps_script_name_input);
                                ui.add_space(crate::theme::SPACING_TINY);
                                if ui.button("Confirm Save").clicked()
                                    && !self.ps_script_name_input.is_empty()
                                {
                                    self.saved_ps_scripts.push(PsScript {
                                        name: self.ps_script_name_input.clone(),
                                        content: self.ps_script.clone(),
                                    });
                                    self.log_action(
                                        "PowerShell",
                                        &format!("Saved script: {}", self.ps_script_name_input),
                                    );
                                    self.ps_script_name_input.clear();
                                    self.show_ps_save_ui = false;
                                }
                            });
                        });
                }
            });
        });
    }

    pub fn render_ps_library(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(format!("{} Script Library", icons::BOOKS));
            ui.add_space(crate::theme::SPACING_TINY);

            let mut to_remove = None;
            let mut to_load = None;
            let mut to_run = None;

            egui::ScrollArea::vertical()
                .id_salt("ps_library_scroll")
                .show(ui, |ui| {
                    for (i, script) in self.saved_ps_scripts.iter().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&script.name).strong());
                                    ui.label(
                                        egui::RichText::new(&script.content)
                                            .small()
                                            .weak()
                                            .line_height(Some(12.0)),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button(icons::TRASH).clicked() {
                                            to_remove = Some(i);
                                        }
                                        if ui.button("Load").clicked() {
                                            to_load = Some(i);
                                        }
                                        if ui.button(icons::PLAY).on_hover_text("Run Now").clicked()
                                        {
                                            to_run = Some(i);
                                        }
                                    },
                                );
                            });
                        });
                        ui.add_space(crate::theme::SPACING_TINY);
                    }
                });

            if let Some(i) = to_load {
                let script_data = self
                    .saved_ps_scripts
                    .get(i)
                    .map(|s| (s.content.clone(), s.name.clone()));
                if let Some((content, name)) = script_data {
                    self.ps_script = content;
                    self.status = format!("Loaded script: {name}");
                }
            }

            if let Some(i) = to_run {
                let script_data = self
                    .saved_ps_scripts
                    .get(i)
                    .map(|s| (s.content.clone(), s.name.clone()));
                if let Some((content, name)) = script_data {
                    self.start_ps_execution(ui.ctx().clone(), content, Some(name.clone()));
                    self.log_action("PowerShell", &format!("Executed: {name}"));
                }
            }

            if let Some(i) = to_remove {
                if i < self.saved_ps_scripts.len() {
                    let removed = self.saved_ps_scripts.remove(i);
                    self.log_action("PowerShell", &format!("Removed script: {}", removed.name));
                }
            }
        });
    }

    pub fn start_ps_execution(&mut self, ctx: egui::Context, script: String, name: Option<String>) {
        self.is_running_ps = true;
        self.ps_last_output.clear();
        self.running_ps_script_name = name;
        let (tx, rx) = crossbeam_channel::unbounded();
        self.ps_rx = Some(rx);

        std::thread::spawn(move || {
            let result = {
                let output = Self::prepare_powershell_cmd(&script, false).output();
                match output {
                    Ok(output) => {
                        let combined_output = format!(
                            "{}{}",
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        );
                        let cleaned_output = crate::utils::strip_ansi(&combined_output);
                        Ok((output.status.code().unwrap_or(-1), cleaned_output))
                    }
                    Err(e) => Err(anyhow::anyhow!(e)),
                }
            };

            if let Err(e) = tx.send(result) {
                log::error!("Failed to send PowerShell execution result: {e}");
            }
            ctx.request_repaint();
        });
    }

    pub fn prepare_powershell_cmd(command: &str, no_exit: bool) -> Command {
        let mut cmd = Command::new("powershell");
        if no_exit {
            cmd.arg("-NoExit");
        }
        cmd.arg("-Command").arg(command);
        cmd
    }
}
