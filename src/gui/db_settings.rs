use crate::gui::BeefcakeApp;
use eframe::egui;
use egui_phosphor::regular as icons;
use secrecy::{ExposeSecret as _, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
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
            db_type: "postgres".to_owned(),
            host: "localhost".to_owned(),
            port: "5432".to_owned(),
            user: "postgres".to_owned(),
            password: SecretString::default(),
            database: String::new(),
            schema: "public".to_owned(),
            table: String::new(),
        }
    }
}

impl BeefcakeApp {
    pub fn render_database_settings(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;
        egui::TopBottomPanel::top("db_settings_top")
            .frame(crate::theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("{} Back", icons::ARROW_LEFT)).clicked() {
                        go_back = true;
                    }
                    ui.separator();
                    ui.heading(format!("{} Database Connection Settings", icons::DATABASE));
                });
            });

        egui::CentralPanel::default()
            .frame(
                crate::theme::central_panel_frame().inner_margin(egui::Margin {
                    left: crate::theme::PANEL_LEFT as i8,
                    right: crate::theme::PANEL_RIGHT as i8,
                    top: crate::theme::SPACING_LARGE as i8,
                    bottom: 0,
                }),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_db_grid(ui, ctx);

                    ui.add_space(crate::theme::SPACING_LARGE);
                    ui.separator();
                    ui.add_space(crate::theme::SPACING_LARGE);

                    self.render_db_profiles(ui);
                    ui.add_space(crate::theme::SPACING_LARGE);
                });
            });

        go_back
    }

    pub fn render_db_grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Current Configuration");
        ui.add_space(crate::theme::SPACING_SMALL);

        egui::Grid::new("db_settings_grid")
            .num_columns(2)
            .spacing([40.0, crate::theme::SPACING_MEDIUM])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Database Type:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.pg_type, "postgres".to_owned(), "PostgreSQL");
                    ui.add_enabled_ui(false, |ui| {
                        ui.selectable_value(
                            &mut self.pg_type,
                            "mysql".to_owned(),
                            "MySQL (Coming Soon)",
                        );
                    });
                });
                ui.end_row();

                ui.label("Host / Server:");
                ui.text_edit_singleline(&mut self.pg_host);
                ui.end_row();

                ui.label("Port:");
                ui.text_edit_singleline(&mut self.pg_port);
                ui.end_row();

                ui.label("Username:");
                ui.text_edit_singleline(&mut self.pg_user);
                ui.end_row();

                ui.label("Password:");
                ui.horizontal(|ui| {
                    let mut pass = self.pg_password.expose_secret().to_owned();
                    let response = ui.add(egui::TextEdit::singleline(&mut pass).password(true));
                    if response.changed() {
                        self.pg_password = SecretString::from(pass);
                    }
                    ui.checkbox(&mut self.save_password, "Save to profile");
                });
                ui.end_row();

                ui.label("Database Name:");
                ui.text_edit_singleline(&mut self.pg_database);
                ui.end_row();

                ui.label("Default Schema:");
                ui.text_edit_singleline(&mut self.pg_schema);
                ui.end_row();

                ui.label("Default Table:");
                ui.text_edit_singleline(&mut self.pg_table);
                ui.end_row();
            });

        ui.add_space(crate::theme::SPACING_LARGE);

        ui.horizontal(|ui| {
            if self.is_testing {
                ui.add(egui::Spinner::new());
                ui.label("Testing connection...");
            } else if ui
                .button(format!("{} Test Connection", icons::LIGHTNING))
                .on_hover_text("Try to connect with current settings")
                .clicked()
            {
                self.start_test_connection(ctx.clone());
            }

            self.render_save_profile_controls(ui);
        });

        ui.add_space(crate::theme::SPACING_MEDIUM);
        crate::utils::render_status_message(ui, &self.status);
    }

    fn render_save_profile_controls(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .button(format!("{} Save as New Profile", icons::FLOPPY_DISK))
                .clicked()
            {
                self.show_db_save_ui = !self.show_db_save_ui;
            }

            if self.show_db_save_ui {
                ui.add_space(crate::theme::SPACING_TINY);
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label("Profile Name:");
                            ui.text_edit_singleline(&mut self.db_name_input);
                            ui.add_space(crate::theme::SPACING_TINY);
                            if ui.button("Confirm Save").clicked() && !self.db_name_input.is_empty()
                            {
                                let config = DbConfig {
                                    name: self.db_name_input.clone(),
                                    db_type: self.pg_type.clone(),
                                    host: self.pg_host.clone(),
                                    port: self.pg_port.clone(),
                                    user: self.pg_user.clone(),
                                    password: if self.save_password {
                                        self.pg_password.clone()
                                    } else {
                                        SecretString::default()
                                    },
                                    database: self.pg_database.clone(),
                                    schema: self.pg_schema.clone(),
                                    table: self.pg_table.clone(),
                                };
                                self.saved_configs.push(config);
                                self.log_action(
                                    "Database",
                                    &format!("Saved profile: {}", self.db_name_input),
                                );
                                self.db_name_input.clear();
                                self.show_db_save_ui = false;
                            }
                        });
                    });
            }
        });
    }

    pub fn render_db_profiles(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("{} Saved Profiles", icons::FLOPPY_DISK));
        ui.add_space(crate::theme::SPACING_SMALL);

        if self.saved_configs.is_empty() {
            ui.label(egui::RichText::new("No profiles saved yet.").weak());
        } else {
            let mut to_remove = None;
            let mut to_load = None;

            egui::Grid::new("db_profiles_grid")
                .num_columns(3)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    for (i, config) in self.saved_configs.iter().enumerate() {
                        ui.label(egui::RichText::new(&config.name).strong());
                        ui.label(format!(
                            "{}@{}/{}",
                            config.user, config.host, config.database
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("Load").clicked() {
                                to_load = Some(i);
                            }
                            if ui.button(icons::TRASH).clicked() {
                                to_remove = Some(i);
                            }
                        });
                        ui.end_row();
                    }
                });

            if let Some(i) = to_load
                && let Some(config) = self.saved_configs.get(i)
            {
                self.pg_type = config.db_type.clone();
                self.pg_host = config.host.clone();
                self.pg_port = config.port.clone();
                self.pg_user = config.user.clone();
                self.pg_password = config.password.clone();
                self.pg_database = config.database.clone();
                self.pg_schema = config.schema.clone();
                self.pg_table = config.table.clone();
                self.save_password = !config.password.expose_secret().is_empty();
                self.status = format!("Loaded profile: {}", config.name);
                self.log_action("Database", &format!("Loaded profile: {}", config.name));
            }

            if let Some(i) = to_remove
                && i < self.saved_configs.len()
            {
                let removed = self.saved_configs.remove(i);
                self.log_action("Database", &format!("Removed profile: {}", removed.name));
            }
        }
    }

    pub fn start_test_connection(&mut self, ctx: egui::Context) {
        use sqlx::postgres::PgConnectOptions;

        let port: u16 = self.pg_port.parse().unwrap_or(5432);
        let pg_options = PgConnectOptions::new()
            .host(&self.pg_host)
            .port(port)
            .username(&self.pg_user)
            .password(self.pg_password.expose_secret())
            .database(&self.pg_database);

        self.is_testing = true;
        self.status = "Testing database connection...".to_owned();
        let (tx, rx) = crossbeam_channel::unbounded();
        self.test_rx = Some(rx);

        std::thread::spawn(move || {
            let result = crate::utils::TOKIO_RUNTIME.block_on(async {
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(1)
                    .acquire_timeout(std::time::Duration::from_secs(3))
                    .connect_with(pg_options)
                    .await?;
                Ok(())
            });

            if let Err(e) = tx.send(result) {
                log::error!("Failed to send test connection result: {e}");
            }
            ctx.request_repaint();
        });
    }
}
