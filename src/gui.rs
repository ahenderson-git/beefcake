use eframe::egui;
use secrecy::SecretString;

// This defines the possible "Pages" or "Views" in our application.
pub enum AppState {
    MainMenu,
    Analyser(Box<crate::analyser::App>),
    ReferenceMaterial,
    DatabaseSettings,
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

    // Database Connection Info (Split)
    pub pg_type: String,
    pub pg_host: String,
    pub pg_port: String,
    pub pg_user: String,
    #[serde(skip)]
    pub pg_password: SecretString,
    pub pg_database: String,

    pub pg_schema: String,
    pub pg_table: String,
    pub save_password: bool,
    pub saved_configs: Vec<DbConfig>,
    #[serde(skip)]
    pub db_name_input: String,
    pub todo_list: Vec<ListItem>,
    pub ideas_list: Vec<ListItem>,
    #[serde(skip)]
    pub todo_input: String,
    #[serde(skip)]
    pub idea_input: String,
    #[serde(skip)]
    pub is_testing: bool,
    #[serde(skip)]
    pub test_rx: Option<crossbeam_channel::Receiver<anyhow::Result<()>>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DbConfig {
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: String,
    pub user: String,
    #[serde(skip)]
    pub password: SecretString,
    pub database: String,
    pub schema: String,
    pub table: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            db_type: "postgres".to_string(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            user: "postgres".to_string(),
            password: SecretString::default(),
            database: String::new(),
            schema: "public".to_string(),
            table: String::new(),
        }
    }
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
            pg_type: "postgres".to_string(),
            pg_host: "localhost".to_string(),
            pg_port: "5432".to_string(),
            pg_user: "postgres".to_string(),
            pg_password: SecretString::default(),
            pg_database: String::new(),
            pg_schema: "public".to_string(),
            pg_table: String::new(),
            save_password: false,
            saved_configs: Vec::new(),
            db_name_input: String::new(),
            todo_list: Vec::new(),
            ideas_list: Vec::new(),
            todo_input: String::new(),
            idea_input: String::new(),
            is_testing: false,
            test_rx: None,
        }
    }
}

impl TemplateApp {
    // This is called when the app first launches.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Here we try to load "saved state" from the previous session.
        if let Some(storage) = cc.storage {
            if let Some(mut app) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                // Try to load password from keyring if enabled
                if app.save_password {
                    // Try new key first
                    if let Ok(entry) = keyring::Entry::new("beefcake-app", "database-password") {
                        if let Ok(pass) = entry.get_password() {
                            app.pg_password = pass.into();
                        } else {
                            // Fallback to old full URL key and try to extract password
                            if let Ok(entry_old) = keyring::Entry::new("beefcake-app", "database-url") {
                                if let Ok(url) = entry_old.get_password() {
                                    if let Some(pass) = extract_pass_from_url(&url) {
                                        app.pg_password = pass.into();
                                    }
                                }
                            }
                        }
                    }
                }

                // Load saved connection passwords from keyring
                for config in &mut app.saved_configs {
                    let key = format!("db-pass-{}", config.name);
                    if let Ok(entry) = keyring::Entry::new("beefcake-app", &key) {
                        if let Ok(pass) = entry.get_password() {
                            config.password = pass.into();
                        } else {
                            // Fallback to old full URL key
                            let key_old = format!("db-profile-{}", config.name);
                            if let Ok(entry_old) = keyring::Entry::new("beefcake-app", &key_old) {
                                if let Ok(url) = entry_old.get_password() {
                                    if let Some(pass) = extract_pass_from_url(&url) {
                                        config.password = pass.into();
                                    }
                                }
                            }
                        }
                    }
                }
                
                app.is_testing = false;
                app.test_rx = None;
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
                ui.add_space(ui.available_height() / 10.0);

                let button_size = egui::vec2(280.0, 280.0);
                let spacing = 20.0;

                // --- Row 1 ---
                ui.horizontal(|ui| {
                    let total_width = button_size.x * 2.0 + spacing;
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
                        analyser.model.pg_type = self.pg_type.clone();
                        analyser.model.pg_host = self.pg_host.clone();
                        analyser.model.pg_port = self.pg_port.clone();
                        analyser.model.pg_user = self.pg_user.clone();
                        analyser.model.pg_password = self.pg_password.clone();
                        analyser.model.pg_database = self.pg_database.clone();
                        analyser.model.pg_schema = self.pg_schema.clone();
                        analyser.model.pg_table = self.pg_table.clone();
                        analyser.model.save_password = self.save_password;
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
                });

                ui.add_space(spacing);

                // --- Row 2 ---
                ui.horizontal(|ui| {
                    let total_width = button_size.x * 2.0 + spacing;
                    let start_space = (ui.available_width() - total_width) / 2.0;
                    if start_space > 0.0 {
                        ui.add_space(start_space);
                    }

                    // --- Button 3: Database Settings ---
                    let db_text = egui::RichText::new("⚙ Database Settings")
                        .heading()
                        .strong();

                    if ui
                        .add_sized(button_size, egui::Button::new(db_text))
                        .on_hover_text("Manage database credentials and connections")
                        .clicked()
                    {
                        self.state = AppState::DatabaseSettings;
                    }

                    ui.add_space(spacing);

                    // --- Button 4: Reference Material ---
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

    // Logic for drawing the Database Settings page.
    fn render_database_settings(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;
        egui::TopBottomPanel::top("db_settings_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    go_back = true;
                }
                ui.separator();
                ui.heading("Database & Credentials Settings");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Manage your database connections and credentials securely.");
                ui.add_space(10.0);

                egui::Grid::new("db_settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Type:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_type).hint_text("postgres"));
                        ui.end_row();

                        ui.label("Host:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_host).hint_text("localhost"));
                        ui.end_row();

                        ui.label("Port:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_port).hint_text("5432"));
                        ui.end_row();

                        ui.label("User:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_user).hint_text("postgres"));
                        ui.end_row();

                        ui.label("Password:");
                        use secrecy::ExposeSecret;
                        let mut temp_pass = self.pg_password.expose_secret().to_string();
                        if ui.add(egui::TextEdit::singleline(&mut temp_pass).password(true)).changed() {
                            self.pg_password = temp_pass.into();
                        }
                        ui.end_row();

                        ui.label("Database:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_database).hint_text("my_database"));
                        ui.end_row();

                        ui.label("");
                        if ui.checkbox(&mut self.save_password, "Save Password securely in OS Keyring")
                            .on_hover_text("If enabled, only the password is stored in the OS's native secure storage. Other fields are saved in the app config.")
                            .changed()
                        {
                            if self.save_password {
                                if let Ok(entry) = keyring::Entry::new("beefcake-app", "database-password") {
                                    let _ = entry.set_password(self.pg_password.expose_secret());
                                }
                            } else {
                                if let Ok(entry) = keyring::Entry::new("beefcake-app", "database-password") {
                                    let _ = entry.delete_credential();
                                }
                            }
                        }
                        // Also update keyring if password changed while "save_password" is on
                        if self.save_password {
                             if let Ok(entry) = keyring::Entry::new("beefcake-app", "database-password") {
                                let _ = entry.set_password(self.pg_password.expose_secret());
                            }
                        }
                        ui.end_row();

                        ui.label("Default Schema:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_schema).hint_text("public"));
                        ui.end_row();

                        ui.label("Default Table:");
                        ui.add(egui::TextEdit::singleline(&mut self.pg_table).hint_text("my_table"));
                        ui.end_row();

                        ui.label("");
                        ui.horizontal(|ui| {
                            if self.is_testing {
                                ui.spinner();
                                ui.label("Testing...");
                            } else {
                                if ui.button("🔌 Test Connection").clicked() {
                                    self.start_test_connection(ctx.clone());
                                }
                            }

                            if !self.status.is_empty() && (self.status.contains("Connection successful") || self.status.contains("Connection failed") || self.status.contains("Testing connection")) {
                                let color = if self.status.contains("failed") || self.status.contains("Error") {
                                    egui::Color32::RED
                                } else if self.status.contains("successful") {
                                    egui::Color32::from_rgb(0, 150, 0)
                                } else {
                                    ui.visuals().text_color()
                                };
                                ui.label(egui::RichText::new(&self.status).color(color));
                            }
                        });
                        ui.end_row();

                        ui.separator();
                        ui.end_row();

                        ui.label("Profile Name:");
                        ui.horizontal(|ui| {
                            let res = ui.add(egui::TextEdit::singleline(&mut self.db_name_input).hint_text("Production / Local"));
                            let name_not_empty = !self.db_name_input.trim().is_empty();
                            let save_btn = ui.add_enabled(name_not_empty, egui::Button::new("💾 Save to List"));
                            
                            if (save_btn.clicked() || (res.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && name_not_empty {
                                let name = self.db_name_input.trim().to_string();
                                let config = DbConfig {
                                    name: name.clone(),
                                    db_type: self.pg_type.clone(),
                                    host: self.pg_host.clone(),
                                    port: self.pg_port.clone(),
                                    user: self.pg_user.clone(),
                                    password: self.pg_password.clone(),
                                    database: self.pg_database.clone(),
                                    schema: self.pg_schema.clone(),
                                    table: self.pg_table.clone(),
                                };
                                
                                // Save Password to keyring
                                if let Ok(entry) = keyring::Entry::new("beefcake-app", &format!("db-pass-{}", name)) {
                                    use secrecy::ExposeSecret;
                                    if let Err(e) = entry.set_password(self.pg_password.expose_secret()) {
                                        self.status = format!("Error saving password to keyring: {}", e);
                                    } else {
                                        // Update existing or add new
                                        if let Some(idx) = self.saved_configs.iter().position(|c| c.name == name) {
                                            self.saved_configs[idx] = config;
                                        } else {
                                            self.saved_configs.push(config);
                                        }
                                        self.status = format!("Profile '{}' saved to list.", name);
                                        self.db_name_input.clear();
                                    }
                                } else {
                                    self.status = "Error: Could not access OS keyring".to_owned();
                                }
                            }
                        });
                        ui.end_row();
                    });
                
                ui.add_space(20.0);
                ui.heading("📋 Saved Connections");
                ui.add_space(8.0);
                
                if self.saved_configs.is_empty() {
                    ui.label("No saved connections yet.");
                } else {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        let mut to_delete = None;
                        let mut to_load = None;

                        egui::Grid::new("saved_connections_grid")
                            .num_columns(5)
                            .spacing([10.0, 10.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Name");
                                ui.strong("URL (Masked)");
                                ui.strong("Schema");
                                ui.strong("Table");
                                ui.label("");
                                ui.end_row();

                                for (i, config) in self.saved_configs.iter().enumerate() {
                                    ui.label(&config.name);
                                    
                                    let port_str = if config.port.is_empty() { String::new() } else { format!(":{}", config.port) };
                                    let display_url = format!("{}://{}@{}{}/{}", config.db_type, config.user, config.host, port_str, config.database);
                                    ui.label(egui::RichText::new(display_url).monospace());
                                    
                                    ui.label(&config.schema);
                                    ui.label(&config.table);
                                    
                                    ui.horizontal(|ui| {
                                        if ui.button("📥 Load").clicked() {
                                            to_load = Some(i);
                                        }
                                        if ui.button("🗑").clicked() {
                                            to_delete = Some(i);
                                        }
                                    });
                                    ui.end_row();
                                }
                            });

                        if let Some(idx) = to_load {
                            let config = &self.saved_configs[idx];
                            self.pg_type = config.db_type.clone();
                            self.pg_host = config.host.clone();
                            self.pg_port = config.port.clone();
                            self.pg_user = config.user.clone();
                            self.pg_password = config.password.clone();
                            self.pg_database = config.database.clone();
                            self.pg_schema = config.schema.clone();
                            self.pg_table = config.table.clone();
                            self.db_name_input = config.name.clone();
                            self.status = format!("Profile '{}' loaded.", config.name);
                        }
                        if let Some(idx) = to_delete {
                            let name = &self.saved_configs[idx].name;
                            if let Ok(entry) = keyring::Entry::new("beefcake-app", &format!("db-pass-{}", name)) {
                                let _ = entry.delete_credential();
                            }
                            self.status = format!("Profile '{}' deleted.", self.saved_configs[idx].name);
                            self.saved_configs.remove(idx);
                        }
                    });
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Security Note:").strong());
                ui.label("Passwords are encrypted at rest using your operating system's native keychain (Windows Credential Manager, macOS Keychain, or Linux Secret Service).");
            });
        });
        go_back
    }

    fn handle_receivers(&mut self) {
        if let Some(rx) = &self.test_rx {
            if let Ok(result) = rx.try_recv() {
                self.is_testing = false;
                self.test_rx = None;
                match result {
                    Ok(_) => {
                        self.status = "✅ Connection successful!".to_owned();
                    }
                    Err(e) => {
                        self.status = format!("❌ Connection failed: {}", e);
                    }
                }
            }
        }
    }

    fn start_test_connection(&mut self, ctx: egui::Context) {
        use secrecy::ExposeSecret;
        use sqlx::postgres::PgConnectOptions;

        if self.pg_host.is_empty() || self.pg_database.is_empty() {
            self.status = "❌ Error: Host and Database are required to test connection".to_owned();
            return;
        }

        self.is_testing = true;
        self.status = "Testing connection...".to_owned();

        let port: u16 = self.pg_port.parse().unwrap_or(5432);
        let pg_options = PgConnectOptions::new()
            .host(&self.pg_host)
            .port(port)
            .username(&self.pg_user)
            .password(self.pg_password.expose_secret())
            .database(&self.pg_database);

        let (tx, rx) = crossbeam_channel::unbounded();
        self.test_rx = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let result = rt.block_on(async {
                crate::analyser::db::DbClient::connect(pg_options).await?;
                Ok(())
            });

            let _ = tx.send(result);
            ctx.request_repaint();
        });
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
        self.handle_receivers();

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
                self.pg_type = analyser.model.pg_type.clone();
                self.pg_host = analyser.model.pg_host.clone();
                self.pg_port = analyser.model.pg_port.clone();
                self.pg_user = analyser.model.pg_user.clone();
                self.pg_password = analyser.model.pg_password.clone();
                self.pg_database = analyser.model.pg_database.clone();
                self.pg_schema = analyser.model.pg_schema.clone();
                self.pg_table = analyser.model.pg_table.clone();
                self.save_password = analyser.model.save_password;
            }
            AppState::ReferenceMaterial => {
                if self.render_reference_material(ctx) {
                    switch_to_main = true;
                }
            }
            AppState::DatabaseSettings => {
                if self.render_database_settings(ctx) {
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

fn extract_pass_from_url(url: &str) -> Option<String> {
    if let Some(at_idx) = url.find('@') {
        if let Some(proto_idx) = url.find("://") {
            let authority = &url[proto_idx + 3..at_idx];
            if let Some(colon_idx) = authority.find(':') {
                return Some(authority[colon_idx + 1..].to_string());
            }
        }
    }
    None
}

