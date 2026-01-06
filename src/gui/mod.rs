use eframe::egui;
use egui_phosphor::regular as icons;
use secrecy::SecretString;

mod dashboard;
mod db_settings;
mod error_view_bridge {
    pub use crate::analyser::gui::render_error_diagnostics_window;
}
mod powershell;
mod reference;

pub use dashboard::ListItem;
pub use db_settings::DbConfig;
pub use powershell::PsScript;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Analyser(Box<crate::analyser::App>),
    ReferenceMaterial,
    DatabaseSettings,
    PowerShellModule,
    Settings,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BeefcakeApp {
    #[serde(skip)]
    pub state: AppState,
    #[serde(skip)]
    pub status: String,

    // Database Connection Info
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
    #[serde(skip)]
    pub show_db_save_ui: bool,

    // Dashboard
    pub todo_list: Vec<ListItem>,
    pub ideas_list: Vec<ListItem>,
    #[serde(skip)]
    pub todo_input: String,
    #[serde(skip)]
    pub idea_input: String,

    // PowerShell
    pub ps_script: String,
    pub saved_ps_scripts: Vec<PsScript>,
    #[serde(skip)]
    pub ps_script_name_input: String,
    #[serde(skip)]
    pub show_ps_save_ui: bool,
    #[serde(skip)]
    pub is_running_ps: bool,
    #[serde(skip)]
    pub ps_rx: Option<crossbeam_channel::Receiver<anyhow::Result<(i32, String)>>>,
    #[serde(skip)]
    pub ps_last_output: String,
    #[serde(skip)]
    pub running_ps_script_name: Option<String>,

    #[serde(skip)]
    pub is_testing: bool,
    #[serde(skip)]
    pub test_rx: Option<crossbeam_channel::Receiver<anyhow::Result<()>>>,

    pub audit_log: Vec<crate::utils::AuditEntry>,
    #[serde(skip)]
    pub error_log: Vec<crate::utils::DetailedError>,
    #[serde(skip)]
    pub show_error_diagnostics: bool,

    #[serde(skip)]
    pub toasts: egui_notify::Toasts,

    pub font_size: f32,
    pub sidebar_font_size: f32,
}

impl Default for BeefcakeApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu,
            status: String::new(),
            pg_type: "postgres".to_owned(),
            pg_host: "localhost".to_owned(),
            pg_port: "5432".to_owned(),
            pg_user: "postgres".to_owned(),
            pg_password: SecretString::default(),
            pg_database: String::new(),
            pg_schema: "public".to_owned(),
            pg_table: String::new(),
            save_password: false,
            saved_configs: Vec::new(),
            db_name_input: String::new(),
            show_db_save_ui: false,
            todo_list: Self::standard_todo_items(),
            ideas_list: Self::standard_idea_items(),
            todo_input: String::new(),
            idea_input: String::new(),
            ps_script: String::new(),
            saved_ps_scripts: Self::default_scripts(),
            ps_script_name_input: String::new(),
            show_ps_save_ui: false,
            is_running_ps: false,
            ps_rx: None,
            ps_last_output: String::new(),
            running_ps_script_name: None,
            is_testing: false,
            test_rx: None,
            audit_log: Vec::new(),
            error_log: Vec::new(),
            show_error_diagnostics: false,
            toasts: egui_notify::Toasts::default(),
            font_size: 14.0,
            sidebar_font_size: 14.0,
        }
    }
}

impl BeefcakeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        crate::theme::apply_beefcake_theme(&cc.egui_ctx, app.font_size);
        app
    }

    pub fn log_action(&mut self, action: &str, details: &str) {
        crate::utils::push_audit_log(&mut self.audit_log, action, details);
    }

    pub fn render_main_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("main_menu_top")
            .frame(crate::theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(format!("{} Beefcake Data Suite", icons::CHART_BAR))
                            .size(24.0)
                            .strong()
                            .color(crate::theme::PRIMARY_COLOR),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(
                            "Professional Grade Analysis & Automation",
                        ));
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                crate::theme::central_panel_frame().inner_margin(egui::Margin {
                    left: crate::theme::PANEL_LEFT as i8,
                    right: crate::theme::PANEL_RIGHT as i8,
                    top: crate::theme::SPACING_LARGE as i8,
                    bottom: crate::theme::SPACING_LARGE as i8,
                }),
            )
            .show(ctx, |ui| {
                let spacing = crate::theme::SPACING_LARGE;
                let available_width = ui.available_width();
                let available_height = ui.available_height();
                let row_height = (available_height - spacing) / 2.0;
                let col_width = (available_width - spacing) / 2.0;

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_width, row_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.render_todo_list(ui);
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_width, row_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.render_ideas_list(ui);
                            },
                        );
                    });
                    ui.add_space(spacing);
                    ui.allocate_ui_with_layout(
                        egui::vec2(available_width, row_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.render_audit_log_panel(ui);
                        },
                    );
                });
            });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(crate::theme::MARGIN_SIDEBAR);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("BEEFCAKE")
                    .strong()
                    .size(self.sidebar_font_size + 6.0)
                    .color(crate::theme::PRIMARY_COLOR),
            );
            ui.add_space(crate::theme::SPACING_LARGE);
        });

        let mut next_state = None;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = crate::theme::SPACING_SMALL;

            if Self::sidebar_button(
                ui,
                icons::HOUSE,
                "Dashboard",
                matches!(self.state, AppState::MainMenu),
                self.sidebar_font_size,
            )
            .clicked()
            {
                next_state = Some(AppState::MainMenu);
            }
            if Self::sidebar_button(
                ui,
                icons::CHART_BAR,
                "Analyser",
                matches!(self.state, AppState::Analyser(_)),
                self.sidebar_font_size,
            )
            .clicked()
                && !matches!(self.state, AppState::Analyser(_))
            {
                next_state = Some(AppState::Analyser(Box::default()));
            }
            if Self::sidebar_button(
                ui,
                icons::DATABASE,
                "Database",
                matches!(self.state, AppState::DatabaseSettings),
                self.sidebar_font_size,
            )
            .clicked()
            {
                next_state = Some(AppState::DatabaseSettings);
            }
            if Self::sidebar_button(
                ui,
                icons::TERMINAL_WINDOW,
                "PowerShell",
                matches!(self.state, AppState::PowerShellModule),
                self.sidebar_font_size,
            )
            .clicked()
            {
                next_state = Some(AppState::PowerShellModule);
            }
            if Self::sidebar_button(
                ui,
                icons::BOOK_OPEN,
                "Reference",
                matches!(self.state, AppState::ReferenceMaterial),
                self.sidebar_font_size,
            )
            .clicked()
            {
                next_state = Some(AppState::ReferenceMaterial);
            }
            if Self::sidebar_button(
                ui,
                icons::GEAR,
                "Settings",
                matches!(self.state, AppState::Settings),
                self.sidebar_font_size,
            )
            .clicked()
            {
                next_state = Some(AppState::Settings);
            }

            if !self.error_log.is_empty() {
                ui.add_space(crate::theme::SPACING_LARGE);
                ui.vertical_centered(|ui| {
                    if ui
                        .button(
                            egui::RichText::new(format!("{} Diagnostics", icons::SHIELD_WARNING))
                                .size(self.sidebar_font_size)
                                .color(crate::theme::PRIMARY_COLOR),
                        )
                        .clicked()
                    {
                        self.show_error_diagnostics = true;
                    }
                });
            }
        });

        if let Some(state) = next_state {
            self.state = state;
        }
    }

    fn sidebar_button(
        ui: &mut egui::Ui,
        icon: &str,
        text: &str,
        active: bool,
        font_size: f32,
    ) -> egui::Response {
        let text = egui::RichText::new(format!("{icon}  {text}"))
            .size(font_size)
            .strong();

        let (rect, response) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), font_size + 26.0),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let bg_fill = if active {
                crate::theme::ACCENT_COLOR
            } else if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().rect_filled(rect, 6.0, bg_fill);

            let text_color = if active {
                egui::Color32::WHITE
            } else if response.hovered() {
                crate::theme::PRIMARY_COLOR
            } else {
                ui.visuals().weak_text_color()
            };

            ui.painter().text(
                rect.left_center() + egui::vec2(15.0, 0.0),
                egui::Align2::LEFT_CENTER,
                text.text(),
                egui::FontId::proportional(font_size),
                text_color,
            );
        }

        response
    }

    fn handle_receivers(&mut self) {
        if let Some(rx) = &self.ps_rx
            && let Ok(result) = rx.try_recv()
        {
            self.is_running_ps = false;
            match result {
                Ok((code, output)) => {
                    self.ps_last_output = output;
                    let script_info = self
                        .running_ps_script_name
                        .as_ref()
                        .map(|n| format!(" '{n}'"))
                        .unwrap_or_default();

                    if code == 0 {
                        self.status = format!(
                            "{} PowerShell script{script_info} finished successfully.",
                            icons::CHECK_CIRCLE
                        );
                        self.toasts.success(format!(
                            "PowerShell script{script_info} finished successfully."
                        ));
                        self.log_action("PowerShell", &format!("Execution success{script_info}"));
                    } else {
                        self.status = format!(
                            "{} PowerShell script{script_info} failed with exit code: {code}",
                            icons::X_CIRCLE
                        );
                        self.toasts.error(format!(
                            "PowerShell script{script_info} failed with exit code: {code}"
                        ));
                        self.log_action(
                            "PowerShell",
                            &format!("Execution failure{script_info} (code: {code})"),
                        );
                    }
                }
                Err(e) => {
                    let script_info = self
                        .running_ps_script_name
                        .as_ref()
                        .map(|n| format!(" '{n}'"))
                        .unwrap_or_default();

                    self.status = format!(
                        "{} PowerShell{script_info} execution error: {e}",
                        icons::X_CIRCLE
                    );
                    self.toasts
                        .error(format!("PowerShell{script_info} execution error: {e}"));
                    self.log_action("PowerShell", &format!("Execution error{script_info}: {e}"));
                    crate::utils::push_error_log(&mut self.error_log, &e, "PowerShell");
                }
            }
            self.ps_rx = None;
            self.running_ps_script_name = None;
        }

        if let Some(rx) = &self.test_rx
            && let Ok(result) = rx.try_recv()
        {
            self.is_testing = false;
            match result {
                Ok(_) => {
                    self.status = format!(
                        "{} Database connection test successful!",
                        icons::CHECK_CIRCLE
                    );
                    self.toasts.success("Database connection test successful!");
                    self.log_action("Database", "Test connection success");
                }
                Err(e) => {
                    self.status =
                        format!("{} Database connection test failed: {e}", icons::X_CIRCLE);
                    self.toasts
                        .error(format!("Database connection test failed: {e}"));
                    self.log_action("Database", "Test connection failure");
                    crate::utils::push_error_log(&mut self.error_log, &e, "Database Test");
                }
            }
            self.test_rx = None;
        }
    }

    pub fn render_settings(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;
        egui::TopBottomPanel::top("settings_top")
            .frame(crate::theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("{} Back", icons::ARROW_LEFT)).clicked() {
                        go_back = true;
                    }
                    ui.separator();
                    ui.heading(format!("{} Application Settings", icons::GEAR));
                });
            });

        egui::CentralPanel::default()
            .frame(crate::theme::central_panel_frame().inner_margin(egui::Margin::same(20)))
            .show(ctx, |ui| {
                ui.add_space(crate::theme::SPACING_LARGE);
                ui.heading("Appearance");
                ui.add_space(crate::theme::SPACING_SMALL);

                ui.horizontal(|ui| {
                    ui.label("Main Font Size:");
                    let response = ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0));
                    if response.changed() {
                        crate::theme::apply_beefcake_theme(ctx, self.font_size);
                    }
                    if ui.button("Reset").clicked() {
                        self.font_size = 14.0;
                        crate::theme::apply_beefcake_theme(ctx, self.font_size);
                    }
                });

                ui.add_space(crate::theme::SPACING_SMALL);
                ui.horizontal(|ui| {
                    ui.label("Sidebar Font Size:");
                    ui.add(egui::Slider::new(&mut self.sidebar_font_size, 10.0..=24.0));
                    if ui.button("Reset").clicked() {
                        self.sidebar_font_size = 14.0;
                    }
                });
            });

        go_back
    }

    fn render_footer(ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("App by Anthony Henderson")
                        .small()
                        .weak(),
                );
                ui.label(egui::RichText::new(" • ").small().weak());
                ui.label(egui::RichText::new("v0.1.0").small().weak());
                ui.label(egui::RichText::new(" • ").small().weak());
                ui.label(egui::RichText::new("Created in Rust").small().weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    powered_by_egui_and_eframe(ui);
                });
            });
        });
    }
}

impl eframe::App for BeefcakeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_receivers();
        self.toasts.show(ctx);

        if self.show_error_diagnostics {
            error_view_bridge::render_error_diagnostics_window(
                &mut self.error_log,
                &mut self.show_error_diagnostics,
                ctx,
            );
        }

        egui::SidePanel::left("main_sidebar")
            .frame(crate::theme::sidebar_frame())
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        match &mut self.state {
            AppState::MainMenu => {
                self.render_main_menu(ctx);
            }
            AppState::Analyser(analyser_app) => {
                if analyser_app.update(ctx, &mut self.toasts) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::DatabaseSettings => {
                if self.render_database_settings(ctx) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::PowerShellModule => {
                if self.render_powershell_module(ctx) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::ReferenceMaterial => {
                if reference::render_reference_material(ctx) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Settings => {
                if self.render_settings(ctx) {
                    self.state = AppState::MainMenu;
                }
            }
        }

        Self::render_footer(ctx);

        if self.is_testing || self.is_running_ps {
            ctx.request_repaint();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(egui::RichText::new("Powered by ").small().weak());
        ui.add(egui::Hyperlink::from_label_and_url(
            egui::RichText::new("egui").small(),
            "https://github.com/emilk/egui",
        ));
        ui.label(egui::RichText::new(" and ").small().weak());
        ui.add(egui::Hyperlink::from_label_and_url(
            egui::RichText::new("eframe").small(),
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        ));
        ui.label(egui::RichText::new(".").small().weak());
    });
}
