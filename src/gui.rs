use eframe::egui;

pub enum AppState {
    MainMenu,
    Analyser(crate::analyser::App),
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    pub state: AppState,
}

impl Default for AppState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu,
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value(storage, eframe::APP_KEY) {
                return app;
            }
        }
        Default::default()
    }

    fn render_main_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("UTILITIES");
            if ui.button("Analyse File").clicked() {
                self.state = AppState::Analyser(crate::analyser::run_analyser());
            }
        });
    }

    fn render_footer(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Muted/Darkened Light Theme
        if !ctx.style().visuals.dark_mode {
            let mut visuals = egui::Visuals::light();
            
            // Darken the overall backgrounds (Greyer, less white)
            visuals.panel_fill = egui::Color32::from_rgb(220, 220, 215);  // Deeper grey-tan
            visuals.window_fill = egui::Color32::from_rgb(230, 230, 225); // Main background
            
            // Soften the widgets (buttons, etc) so they aren't stark white
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(210, 210, 205);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(200, 200, 195);
            
            // Darken the text slightly (Deep Charcoal instead of Black)
            visuals.override_text_color = Some(egui::Color32::from_rgb(50, 50, 55));
            
            ctx.set_visuals(visuals);
        }

        self.render_footer(ctx);

        match &mut self.state {
            AppState::MainMenu => {
                self.render_main_menu(ctx);
            }
            AppState::Analyser(analyser) => {
                if analyser.update(ctx) {
                    self.state = AppState::MainMenu;
                }
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
