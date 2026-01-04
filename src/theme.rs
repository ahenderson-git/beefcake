use eframe::egui;
use egui::{Color32, CornerRadius, Margin, Stroke};

pub const ACCENT_COLOR: Color32 = Color32::from_rgb(211, 47, 47);

pub fn apply_beefcake_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Custom palette
    visuals.widgets.active.bg_fill = ACCENT_COLOR;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(183, 28, 28);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 30);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = ACCENT_COLOR.linear_multiply(0.4);

    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.window_shadow.blur = 15;
    visuals.window_shadow.color = Color32::from_rgba_premultiplied(0, 0, 0, 150);

    visuals.faint_bg_color = Color32::from_rgb(35, 35, 35);
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 20);

    ctx.set_visuals(visuals);

    // Setup Phosphor icons
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

pub fn card_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(15))
        .stroke(Stroke::new(
            1.0,
            ui.visuals()
                .widgets
                .noninteractive
                .bg_stroke
                .color
                .linear_multiply(0.5),
        ))
}

pub fn sidebar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(25, 25, 25))
        .inner_margin(Margin::same(10))
}

pub fn top_bar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(30, 30, 30))
        .inner_margin(Margin::symmetric(20, 10))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 45, 45)))
}
