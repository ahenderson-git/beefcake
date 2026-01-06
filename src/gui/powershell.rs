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
            PsScript {
                name: "Beefcake CLI Wrappers".to_owned(),
                content: r#"# Use these functions to interact with the Beefcake CLI
function Import-BeefcakeData {
    param($Path, $Table, $Schema="public", [switch]$Clean)
    $Args = @("import", "--file", $Path, "--table", $Table, "--schema", $Schema, "--db-url", $env:DATABASE_URL)
    if ($Clean) { $Args += "--clean" }
    & ./beefcake.exe @Args
}

function Convert-BeefcakeData {
    param($Input, $Output, [switch]$Clean)
    $Args = @("export", "--input", $Input, "--output", $Output)
    if ($Clean) { $Args += "--clean" }
    & ./beefcake.exe @Args
}

function Clean-BeefcakeData {
    param($Path, $Output)
    & ./beefcake.exe clean --file $Path --output $Output
}

# Example:
# Clean-BeefcakeData -Path "dirty.csv" -Output "clean.parquet"
"Beefcake CLI functions loaded. Usage: Clean-BeefcakeData -Path 'dirty.csv' -Output 'clean.parquet'"#.to_owned(),
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
            .frame(crate::theme::central_panel_frame().inner_margin(egui::Margin {
                left: crate::theme::PANEL_LEFT as i8,
                right: crate::theme::PANEL_RIGHT as i8,
                top: crate::theme::SPACING_LARGE as i8,
                bottom: 0,
            }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("ps_module_scroll")
                    .show(ui, |ui| {
                        ui.columns(2, |columns| {
                            if let [left, right] = columns {
                                left.vertical(|ui| {
                                    self.render_ps_editor(ui);

                                    ui.add_space(crate::theme::MARGIN_SIDEBAR);
                                    crate::utils::render_status_message(ui, &self.status);

                                    if !self.ps_last_output.is_empty() {
                                        ui.add_space(crate::theme::MARGIN_SIDEBAR);
                                        crate::theme::card_frame(ui).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
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
                                });
                                self.render_ps_library(right);
                            }
                        });

                        ui.add_space(crate::theme::SPACING_LARGE);
                        self.render_cli_reference(ui);
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
            } else {
                if ui
                    .button(format!("{} Run Script", icons::PLAY))
                    .on_hover_text("Execute script and wait for result")
                    .clicked()
                {
                    self.start_ps_execution(ui.ctx().clone(), self.ps_script.clone(), None);
                    self.log_action("PowerShell", "Started script execution");
                }

                if ui
                    .button(format!("{} Format Code", icons::CODE))
                    .on_hover_text("Auto-format PowerShell script")
                    .clicked()
                {
                    self.format_ps_code();
                }
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
                        crate::theme::card_frame(ui).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&script.name)
                                            .strong()
                                            .color(crate::theme::ACCENT_COLOR),
                                    );
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
                                        if ui
                                            .button(icons::TRASH)
                                            .on_hover_text("Delete script")
                                            .clicked()
                                        {
                                            to_remove = Some(i);
                                        }
                                        if ui
                                            .button("Load")
                                            .on_hover_text("Load to editor")
                                            .clicked()
                                        {
                                            to_load = Some(i);
                                        }
                                        if ui.button(icons::PLAY).on_hover_text("Run now").clicked()
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

            if let Some(i) = to_remove
                && i < self.saved_ps_scripts.len()
            {
                let removed = self.saved_ps_scripts.remove(i);
                self.log_action("PowerShell", &format!("Removed script: {}", removed.name));
            }
        });
    }

    pub fn render_cli_reference(&self, ui: &mut egui::Ui) {
        crate::theme::card_frame(ui).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(format!(
                            "{} Beefcake CLI Reference",
                            icons::TERMINAL_WINDOW
                        ))
                        .color(crate::theme::PRIMARY_COLOR),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("v0.1.0").weak().small());
                    });
                });
                ui.add_space(crate::theme::SPACING_SMALL);

                ui.label("Beefcake can be run directly from the terminal for high-performance data operations.");

                ui.add_space(crate::theme::SPACING_MEDIUM);

                egui::Grid::new("cli_commands_grid")
                    .num_columns(2)
                    .spacing([crate::theme::SPACING_LARGE, crate::theme::SPACING_SMALL])
                    .min_col_width(200.0)
                    .show(ui, |ui| {
                        // Import Command
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("import")
                                    .strong()
                                    .color(crate::theme::PRIMARY_COLOR),
                            );
                            ui.label(egui::RichText::new("Load files into database").small().weak());
                        });
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("beefcake import --file <PATH> --table <NAME>").monospace());
                            ui.label(egui::RichText::new("Options: --schema, --db-url, --clean").small().weak());
                        });
                        ui.end_row();

                        // Export Command
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("export")
                                    .strong()
                                    .color(crate::theme::PRIMARY_COLOR),
                            );
                            ui.label(egui::RichText::new("Fast file format conversion").small().weak());
                        });
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("beefcake export --input <FILE> --output <FILE>").monospace());
                            ui.label(egui::RichText::new("Options: --clean (Supports CSV, Parquet, JSON)").small().weak());
                        });
                        ui.end_row();

                        // Clean Command
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("clean")
                                    .strong()
                                    .color(crate::theme::PRIMARY_COLOR),
                            );
                            ui.label(egui::RichText::new("Heuristic-based auto-cleaning").small().weak());
                        });
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("beefcake clean --file <FILE> --output <FILE>").monospace());
                            ui.label(egui::RichText::new("Applies advised cleaning transformations").small().weak());
                        });
                        ui.end_row();
                    });

                ui.add_space(crate::theme::SPACING_MEDIUM);
                ui.horizontal(|ui| {
                    ui.label(format!("{} ", icons::INFO));
                    ui.label(egui::RichText::new("Tip: Use '--help' with any command to see all available flags and options.").small().weak());
                });
            });
        });
    }

    pub fn format_ps_code(&mut self) {
        let mut formatted = String::new();
        let mut indent_level: usize = 0;
        let indent_size: usize = 4;

        for line in self.ps_script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !formatted.is_empty() && !formatted.ends_with("\n\n") {
                    formatted.push('\n');
                }
                continue;
            }

            let open_braces = trimmed.chars().filter(|&c| c == '{').count();
            let close_braces = trimmed.chars().filter(|&c| c == '}').count();

            // Adjust indent BEFORE line for any starting '}'
            let mut current_line_indent = indent_level;
            if trimmed.starts_with('}') {
                current_line_indent = current_line_indent.saturating_sub(1);
            }

            formatted.push_str(&" ".repeat(current_line_indent * indent_size));
            formatted.push_str(trimmed);
            formatted.push('\n');

            // Update indent level for NEXT line
            indent_level =
                (indent_level as i32 + open_braces as i32 - close_braces as i32).max(0) as usize;
        }
        self.ps_script = formatted.trim().to_owned();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ps_code() {
        let mut app = BeefcakeApp {
            ps_script: "if ($true) {\ncommand\n}".to_owned(),
            ..BeefcakeApp::default()
        };
        app.format_ps_code();
        // The formatter adds 4 spaces and a newline at the end of the last line
        // trim() is called at the end of format_ps_code
        assert_eq!(app.ps_script, "if ($true) {\n    command\n}");

        app.ps_script = "} else {".to_owned();
        app.format_ps_code();
        assert_eq!(app.ps_script, "} else {");
    }
}
