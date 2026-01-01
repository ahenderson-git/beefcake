use eframe::egui;

// This defines the possible "Pages" or "Views" in our application.
pub enum AppState {
    MainMenu,
    Analyser(Box<crate::analyser::App>),
}

// This is the main data structure for your app.
// It keeps track of which "State" (page) we are currently on.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)] // We don't save the current page to disk when closing the app
    pub state: AppState,
    pub pg_url: String,
    pub pg_schema: String,
    pub pg_table: String,
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
            pg_url: String::new(),
            pg_schema: String::new(),
            pg_table: String::new(),
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

        // Draw the main area with the "Analyse File" button.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("UTILITIES");
            if ui.button("Analyse File").clicked() {
                // When clicked, we change the app state to the Analyser view.
                let mut analyser = crate::analyser::run_analyser();
                analyser.pg_url = self.pg_url.clone();
                analyser.pg_schema = self.pg_schema.clone();
                analyser.pg_table = self.pg_table.clone();
                self.state = AppState::Analyser(Box::new(analyser));
            }
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
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
    });
}
