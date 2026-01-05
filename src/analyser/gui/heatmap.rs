use crate::analyser::logic::CorrelationMatrix;
use eframe::egui;
use egui_phosphor::regular as icons;

#[expect(clippy::indexing_slicing)]
pub fn render_correlation_heatmap(ui: &mut egui::Ui, matrix: &CorrelationMatrix) {
    ui.vertical(|ui| {
        ui.heading(format!(
            "{} Correlation Matrix (Numeric Features)",
            icons::LINK
        ));
        ui.add_space(crate::theme::SPACING_TINY);
        ui.label(
            egui::RichText::new("Shows how numeric columns relate to each other (-1.0 to 1.0).")
                .weak()
                .size(12.0),
        );
        ui.add_space(crate::theme::MARGIN_SIDEBAR);

        let n = matrix.columns.len();
        let cell_size = 42.0;
        let label_width = 180.0;
        let header_height = 100.0;

        egui::ScrollArea::both()
            .id_salt("correlation_heatmap_scroll")
            .show(ui, |ui| {
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(
                        label_width + n as f32 * cell_size + 20.0,
                        header_height + n as f32 * cell_size + 20.0,
                    ),
                    egui::Sense::hover(),
                );

                let painter = ui.painter();

                // Render columns labels (diagonal/vertical)
                for (j, name) in matrix.columns.iter().enumerate() {
                    let pos = rect.min
                        + egui::vec2(
                            label_width + j as f32 * cell_size + cell_size / 2.0,
                            header_height - 10.0,
                        );
                    painter.text(
                        pos,
                        egui::Align2::CENTER_BOTTOM,
                        name.chars().take(24).collect::<String>(),
                        egui::FontId::proportional(10.0),
                        ui.visuals().text_color(),
                    );
                }

                for (i, row_name) in matrix.columns.iter().enumerate() {
                    // Render row label
                    painter.text(
                        rect.min
                            + egui::vec2(
                                label_width - 10.0,
                                header_height + i as f32 * cell_size + cell_size / 2.0,
                            ),
                        egui::Align2::RIGHT_CENTER,
                        row_name.chars().take(32).collect::<String>(),
                        egui::FontId::proportional(11.0),
                        ui.visuals().text_color(),
                    );

                    for (j, &val) in matrix.data[i].iter().enumerate() {
                        let cell_rect = egui::Rect::from_min_size(
                            rect.min
                                + egui::vec2(
                                    label_width + j as f32 * cell_size,
                                    header_height + i as f32 * cell_size,
                                ),
                            egui::vec2(cell_size, cell_size),
                        );

                        // Color based on correlation: -1 (Blue) -> 0 (Faint) -> 1 (Beef Red)
                        let color = if val > 0.0 {
                            crate::theme::ACCENT_COLOR
                                .linear_multiply(val.abs() as f32)
                                .gamma_multiply(0.8)
                        } else {
                            egui::Color32::from_rgb(41, 121, 255)
                                .linear_multiply(val.abs() as f32)
                                .gamma_multiply(0.8)
                        };

                        let cell_bg = ui.visuals().extreme_bg_color;
                        painter.rect_filled(cell_rect.shrink(1.0), 4.0, cell_bg);
                        painter.rect_filled(cell_rect.shrink(1.0), 4.0, color);

                        // Text value
                        if val.abs() > 0.2 {
                            painter.text(
                                cell_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{val:.1}"),
                                egui::FontId::proportional(11.0),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            });
    });
}
