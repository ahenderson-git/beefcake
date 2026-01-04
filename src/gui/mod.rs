use eframe::egui;
use egui_phosphor::regular as icons;
use secrecy::SecretString;

pub mod dashboard;
pub mod db_settings;
pub mod powershell;
pub mod reference;

pub use dashboard::ListItem;
pub use db_settings::DbConfig;
pub use powershell::PsScript;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum AppState {
    MainMenu,
    Analyser(Box<crate::analyser::App>),
    ReferenceMaterial,
    DatabaseSettings,
    PowerShellModule,
}

impl Default for AppState {
    fn default() -> Self {
        Self::MainMenu
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
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
    pub is_running_ps: bool,
    #[serde(skip)]
    pub ps_rx: Option<crossbeam_channel::Receiver<anyhow::Result<(i32, String)>>>,
    #[serde(skip)]
    pub ps_last_output: String,

    #[serde(skip)]
    pub is_testing: bool,
    #[serde(skip)]
    pub test_rx: Option<crossbeam_channel::Receiver<anyhow::Result<()>>>,

    pub audit_log: Vec<crate::utils::AuditEntry>,

    #[serde(skip)]
    pub toasts: egui_notify::Toasts,
}

impl Default for TemplateApp {
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
            todo_list: Self::standard_todo_items(),
            ideas_list: Self::standard_idea_items(),
            todo_input: String::new(),
            idea_input: String::new(),
            ps_script: String::new(),
            saved_ps_scripts: Self::default_scripts(),
            ps_script_name_input: String::new(),
            is_running_ps: false,
            ps_rx: None,
            ps_last_output: String::new(),
            is_testing: false,
            test_rx: None,
            audit_log: Vec::new(),
            toasts: egui_notify::Toasts::default(),
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::theme::apply_beefcake_theme(&cc.egui_ctx);
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self::default()
    }

    pub fn log_action(&mut self, action: &str, details: &str) {
        crate::utils::push_audit_log(&mut self.audit_log, action, details);
    }

    pub fn render_main_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::theme::top_bar_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(format!("{} Beefcake Data Suite", icons::CHART_BAR))
                            .size(24.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Professional Grade Analysis & Automation").weak(),
                        );
                    });
                });
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(20.0);
                ui.columns(2, |columns| {
                    if let [left, right] = columns {
                        left.vertical(|ui| {
                            self.render_dashboard_lists(ui);
                        });

                        right.vertical(|ui| {
                            self.render_audit_log_panel(ui);
                        });
                    }
                });
            });
        });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("BEEFCAKE")
                    .strong()
                    .size(20.0)
                    .color(crate::theme::ACCENT_COLOR),
            );
            ui.add_space(20.0);
        });

        let mut next_state = None;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            if Self::sidebar_button(
                ui,
                icons::HOUSE,
                "Dashboard",
                matches!(self.state, AppState::MainMenu),
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
            )
            .clicked()
            {
                next_state = Some(AppState::ReferenceMaterial);
            }
        });

        if let Some(state) = next_state {
            self.state = state;
        }
    }

    fn sidebar_button(ui: &mut egui::Ui, icon: &str, text: &str, active: bool) -> egui::Response {
        let text = egui::RichText::new(format!("{icon}  {text}"))
            .size(14.0)
            .strong();

        let (rect, response) =
            ui.allocate_at_least(egui::vec2(ui.available_width(), 40.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg_fill = if active {
                crate::theme::ACCENT_COLOR
            } else if response.hovered() {
                ui.visuals().faint_bg_color
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().rect_filled(rect, 6.0, bg_fill);

            let text_color = if active {
                egui::Color32::WHITE
            } else if response.hovered() {
                ui.visuals().strong_text_color()
            } else {
                ui.visuals().weak_text_color()
            };

            ui.painter().text(
                rect.left_center() + egui::vec2(15.0, 0.0),
                egui::Align2::LEFT_CENTER,
                text.text(),
                egui::FontId::proportional(14.0),
                text_color,
            );
        }

        response
    }

    fn handle_receivers(&mut self) {
        if let Some(rx) = &self.ps_rx {
            if let Ok(result) = rx.try_recv() {
                self.is_running_ps = false;
                match result {
                    Ok((code, output)) => {
                        self.ps_last_output = output;
                        if code == 0 {
                            self.status = format!(
                                "{} PowerShell script finished successfully.",
                                icons::CHECK_CIRCLE
                            );
                            self.toasts
                                .success("PowerShell script finished successfully.");
                            self.log_action("PowerShell", "Execution success");
                        } else {
                            self.status = format!(
                                "{} PowerShell script failed with exit code: {code}",
                                icons::X_CIRCLE
                            );
                            self.toasts.error(format!(
                                "PowerShell script failed with exit code: {code}"
                            ));
                            self.log_action("PowerShell", &format!("Execution failure (code: {code})"));
                        }
                    }
                    Err(e) => {
                        self.status =
                            format!("{} PowerShell execution error: {e}", icons::X_CIRCLE);
                        self.toasts
                            .error(format!("PowerShell execution error: {e}"));
                        self.log_action("PowerShell", &format!("Execution error: {e}"));
                    }
                }
                self.ps_rx = None;
            }
        }

        if let Some(rx) = &self.test_rx {
            if let Ok(result) = rx.try_recv() {
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
                    }
                }
                self.test_rx = None;
            }
        }
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

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_receivers();
        self.toasts.show(ctx);

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
