use eframe::egui;

// This defines the possible "Pages" or "Views" in our application.
pub enum AppState {
    MainMenu,
    Analyser(Box<crate::analyser::App>),
    ReferenceMaterial,
}

// This is the main data structure for your app.
// It keeps track of which "State" (page) we are currently on.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)] // We don't save the current page to disk when closing the app
    pub state: AppState,
    #[serde(skip)]
    pub status: String,
    pub pg_url: String,
    pub pg_schema: String,
    pub pg_table: String,
    pub todo_list: Vec<ListItem>,
    pub ideas_list: Vec<ListItem>,
    #[serde(skip)]
    pub todo_input: String,
    #[serde(skip)]
    pub idea_input: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ListItem {
    pub text: String,
    pub completed: bool,
}

impl Default for ListItem {
    fn default() -> Self {
        Self {
            text: String::new(),
            completed: false,
        }
    }
}

// Logic for starting up the app for the very first time.
impl Default for AppState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu,
            status: String::new(),
            pg_url: String::new(),
            pg_schema: String::new(),
            pg_table: String::new(),
            todo_list: Vec::new(),
            ideas_list: Vec::new(),
            todo_input: String::new(),
            idea_input: String::new(),
        }
    }
}

impl TemplateApp {
    // This is called when the app first launches.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Here we try to load "saved state" from the previous session.
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value(storage, eframe::APP_KEY) {
                return app;
            }
        }
        Default::default()
    }

    // Logic for drawing the Main Menu screen.
    fn render_main_menu(&mut self, ctx: &egui::Context) {
        // Draw the top bar with the "File" menu.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // Draw the main area with buttons.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 6.0);

                ui.horizontal(|ui| {
                    let button_size = egui::vec2(280.0, 280.0);
                    let spacing = 20.0;
                    let total_width = button_size.x * 3.0 + spacing * 2.0;
                    let start_space = (ui.available_width() - total_width) / 2.0;
                    if start_space > 0.0 {
                        ui.add_space(start_space);
                    }

                    // --- Button 1: Analyse, Clean & Export ---
                    let analyse_text = egui::RichText::new("Analyse, Clean & Export")
                        .heading()
                        .strong();

                    if ui
                        .add_sized(button_size, egui::Button::new(analyse_text))
                        .clicked()
                    {
                        // When clicked, we change the app state to the Analyser view.
                        let mut analyser = crate::analyser::run_analyser();
                        analyser.pg_url = self.pg_url.clone();
                        analyser.pg_schema = self.pg_schema.clone();
                        analyser.pg_table = self.pg_table.clone();
                        self.state = AppState::Analyser(Box::new(analyser));
                    }

                    ui.add_space(spacing);

                    // --- Button 2: PowerShell ---
                    let ps_text = egui::RichText::new("🐚 PowerShell")
                        .heading()
                        .strong()
                        .color(egui::Color32::from_rgb(1, 158, 222)); // PowerShell Blue

                    if ui
                        .add_sized(button_size, egui::Button::new(ps_text))
                        .on_hover_text("Open PowerShell Terminal")
                        .clicked()
                    {
                        let mut cmd = std::process::Command::new("powershell.exe");
                        if let Ok(exe_path) = std::env::current_exe() {
                            if let Some(exe_dir) = exe_path.parent() {
                                cmd.current_dir(exe_dir);
                            }
                        }
                        match cmd.spawn() {
                            Ok(_) => {
                                self.status = "PowerShell terminal opened.".to_owned();
                            }
                            Err(e) => {
                                self.status = format!("Failed to launch PowerShell: {e}");
                            }
                        }
                    }

                    ui.add_space(spacing);

                    // --- Button 3: Reference Material ---
                    let ref_text = egui::RichText::new("📚 Reference Material")
                        .heading()
                        .strong();

                    if ui
                        .add_sized(button_size, egui::Button::new(ref_text))
                        .on_hover_text("Helpful links and documentation")
                        .clicked()
                    {
                        self.state = AppState::ReferenceMaterial;
                    }
                });

                if !self.status.is_empty() {
                    ui.add_space(20.0);
                    let color = if self.status.starts_with("Error") || self.status.contains("Failed") {
                        egui::Color32::RED
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.colored_label(color, &self.status);
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal_top(|ui| {
                let spacing = 40.0;
                let column_width = (ui.available_width() - spacing) / 2.0;

                // --- To-Do List ---
                ui.allocate_ui(egui::vec2(column_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("✅ To-Do List");
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let res = ui.add(
                                egui::TextEdit::singleline(&mut self.todo_input)
                                    .hint_text("Add a task...")
                                    .desired_width(column_width * 0.7),
                            );
                            if (ui.button("Add").clicked()
                                || (res.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                && !self.todo_input.trim().is_empty()
                            {
                                self.todo_list.push(ListItem {
                                    text: self.todo_input.trim().to_string(),
                                    completed: false,
                                });
                                self.todo_input.clear();
                            }
                        });

                        ui.add_space(8.0);
                        egui::ScrollArea::vertical()
                            .id_salt("todo_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                let mut to_remove = None;
                                for (i, item) in self.todo_list.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut item.completed, "");
                                        if item.completed {
                                            ui.label(
                                                egui::RichText::new(&item.text)
                                                    .strikethrough()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        } else {
                                            ui.label(&item.text);
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("🗑").on_hover_text("Delete").clicked()
                                                {
                                                    to_remove = Some(i);
                                                }
                                            },
                                        );
                                    });
                                }
                                if let Some(idx) = to_remove {
                                    self.todo_list.remove(idx);
                                }
                            });
                    });
                });

                ui.add_space(spacing);

                // --- Ideas List ---
                ui.allocate_ui(egui::vec2(column_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("💡 Ideas");
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let res = ui.add(
                                egui::TextEdit::singleline(&mut self.idea_input)
                                    .hint_text("New idea...")
                                    .desired_width(column_width * 0.7),
                            );
                            if (ui.button("Add").clicked()
                                || (res.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                && !self.idea_input.trim().is_empty()
                            {
                                self.ideas_list.push(ListItem {
                                    text: self.idea_input.trim().to_string(),
                                    completed: false,
                                });
                                self.idea_input.clear();
                            }
                        });

                        ui.add_space(8.0);
                        egui::ScrollArea::vertical()
                            .id_salt("ideas_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                let mut to_remove = None;
                                for (i, item) in self.ideas_list.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut item.completed, "");
                                        if item.completed {
                                            ui.label(
                                                egui::RichText::new(&item.text)
                                                    .strikethrough()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        } else {
                                            ui.label(&item.text);
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("🗑").on_hover_text("Delete").clicked()
                                                {
                                                    to_remove = Some(i);
                                                }
                                            },
                                        );
                                    });
                                }
                                if let Some(idx) = to_remove {
                                    self.ideas_list.remove(idx);
                                }
                            });
                    });
                });
            });
        });
    }

    // Logic for drawing the Reference Material page.
    fn render_reference_material(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;
        egui::TopBottomPanel::top("ref_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();
                ui.heading("Reference Material");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Here are some helpful links for data analysis and Rust development:");
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong("📊 Data Analysis (Polars)");
                        ui.hyperlink_to("Polars User Guide", "https://docs.pola.rs/user-guide/");
                        ui.hyperlink_to(
                            "Polars API Reference (Rust)",
                            "https://docs.rs/polars/latest/polars/",
                        );
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong("🖼 UI Development (egui)");
                        ui.hyperlink_to("egui Documentation", "https://docs.rs/egui/latest/egui/");
                        ui.hyperlink_to("egui Demo Gallery", "https://emilk.github.io/egui/");
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong("⚙ Rust Programming");
                        ui.hyperlink_to(
                            "The Rust Programming Language",
                            "https://doc.rust-lang.org/book/",
                        );
                        ui.hyperlink_to(
                            "Rust by Example",
                            "https://doc.rust-lang.org/rust-by-example/",
                        );
                        ui.hyperlink_to(
                            "Udemy: Learn to Code with Rust",
                            "https://www.udemy.com/course/learn-to-code-with-rust/",
                        );
                    });
                });
            });
        });
        go_back
    }

    // Logic for drawing the shared footer at the bottom of every page.
    fn render_footer(ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
                // Put the theme buttons (Dark/Light) on the far right.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });
    }
}

// This is the "Engine" of the app. It runs many times per second to redraw the screen.
impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // CUSTOM THEME: If the user is in Light Mode, apply our "Muted/Soft" colors.
        if !ctx.style().visuals.dark_mode {
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = egui::Color32::from_rgb(220, 220, 215);
            visuals.window_fill = egui::Color32::from_rgb(230, 230, 225);
            ctx.set_visuals(visuals);
        }

        // Always show the footer.
        Self::render_footer(ctx);

        // Check our "State" to decide which main page content to draw.
        let mut switch_to_main = false;
        match &mut self.state {
            AppState::MainMenu => {
                self.render_main_menu(ctx);
            }
            AppState::Analyser(analyser) => {
                // If the Analyser's update returns 'true', it means the user clicked "Back".
                if analyser.update(ctx) {
                    switch_to_main = true;
                }

                // Keep our persistent URL in sync
                self.pg_url = analyser.pg_url.clone();
                self.pg_schema = analyser.pg_schema.clone();
                self.pg_table = analyser.pg_table.clone();
            }
            AppState::ReferenceMaterial => {
                if self.render_reference_material(ctx) {
                    switch_to_main = true;
                }
            }
        }

        if switch_to_main {
            self.state = AppState::MainMenu;
        }
    }

    // Save the app's data to disk before closing.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

// Small helper function to draw the credits/links in the footer.
fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.label("App by Anthony Henderson");
        ui.separator();
        ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
        ui.separator();
        ui.hyperlink_to("GitHub", "https://github.com/ahenderson-git/beefcake");
        ui.separator();
        ui.label(egui::RichText::new("⚙").color(egui::Color32::from_rgb(222, 54, 26)));
        ui.hyperlink_to("Built with Rust", "https://www.rust-lang.org/");
        ui.separator();
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
    });
}
