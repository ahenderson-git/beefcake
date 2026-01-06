use crate::analyser::logic::{ColumnStats, NumericStats, TemporalStats};
use eframe::egui;
use egui_plot::{Bar, BarChart, Line, Plot, PlotBounds};

pub fn render_distribution(
    ui: &mut egui::Ui,
    name: &str,
    stats: &ColumnStats,
    show_full_range: bool,
    as_pie: bool,
    is_expanded: bool,
) {
    ui.scope(|ui| {
        let plot_height = if is_expanded { 220.0 } else { 80.0 };

        match stats {
            ColumnStats::Numeric(s) => {
                render_numeric_plot(ui, name, s, show_full_range, plot_height);
            }
            ColumnStats::Categorical(freq) => {
                render_categorical_plot(ui, name, freq, as_pie, plot_height);
            }
            ColumnStats::Temporal(s) => {
                render_temporal_plot(ui, name, s, show_full_range, plot_height);
            }
            ColumnStats::Text(_) => {
                ui.label("—");
            }
            ColumnStats::Boolean(s) => {
                let mut freq = std::collections::HashMap::new();
                freq.insert("True".to_owned(), s.true_count);
                freq.insert("False".to_owned(), s.false_count);
                render_categorical_plot(ui, name, &freq, as_pie, plot_height);
            }
        }
    });
}

fn render_numeric_plot(
    ui: &mut egui::Ui,
    name: &str,
    s: &NumericStats,
    show_full_range: bool,
    plot_height: f32,
) {
    if s.histogram.is_empty() {
        ui.label("—");
        return;
    }

    let (view_min, view_max) = calculate_view_bounds(s, show_full_range);
    let range = view_max - view_min;
    let margin = if range > 0.0 { range * 0.05 } else { 1.0 };
    let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

    let chart = BarChart::new(
        "Histogram",
        create_histogram_bars(s, view_min, view_max, margin, show_full_range),
    )
    .color(crate::theme::ACCENT_COLOR.linear_multiply(0.5))
    .element_formatter(Box::new(|bar, _| {
        format!("Value: {:.4}\nCount: {}", bar.argument, bar.value)
    }));

    let curve_points = create_gaussian_points(s, view_min, view_max, margin);
    let has_curve = !curve_points.is_empty();
    let curve = Line::new("Gaussian Distribution", curve_points)
        .color(egui::Color32::from_rgb(255, 179, 102))
        .width(2.0);

    let box_lines =
        create_box_plot_lines(s, max_count, view_min, view_max, margin, show_full_range);

    ui.vertical(|ui| {
        Plot::new(format!("plot_num_{name}"))
            .show_axes([false, false])
            .show_grid([false, false])
            .show_x(false)
            .show_y(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .include_y(max_count)
            .include_y(-max_count * 0.3)
            .include_x(view_min - margin)
            .include_x(view_max + margin)
            .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
            .height(plot_height)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [view_min - margin, -max_count * 0.3],
                    [view_max + margin, max_count * 1.1],
                ));
                plot_ui.bar_chart(chart);
                if has_curve {
                    plot_ui.line(curve);
                }
                for line in box_lines {
                    plot_ui.line(line.color(egui::Color32::from_gray(180)).width(1.0));
                }
            });

        if !show_full_range {
            render_zoom_info(ui, &s.histogram, s.bin_width, view_min, view_max);
        }
    });
}

fn calculate_view_bounds(s: &NumericStats, show_full_range: bool) -> (f64, f64) {
    let min = s.min.unwrap_or(0.0);
    let max = s.max.unwrap_or(1.0);
    let p05 = s.p05.unwrap_or(min);
    let p95 = s.p95.unwrap_or(max);

    if show_full_range {
        (min, max)
    } else {
        let full_range = max - min;
        let zoom_range = p95 - p05;
        if full_range > 3.0 * zoom_range && zoom_range > 0.0 {
            (p05, p95)
        } else {
            (min, max)
        }
    }
}

fn create_histogram_bars(
    s: &NumericStats,
    view_min: f64,
    view_max: f64,
    margin: f64,
    show_full_range: bool,
) -> Vec<Bar> {
    s.histogram
        .iter()
        .filter(|&&(val, _)| {
            show_full_range || (val >= view_min - margin && val <= view_max + margin)
        })
        .map(|&(val, count)| {
            Bar::new(val, count as f64)
                .width(s.bin_width)
                .stroke(egui::Stroke::new(0.5, crate::theme::ACCENT_COLOR))
        })
        .collect()
}

fn create_gaussian_points(
    s: &NumericStats,
    view_min: f64,
    view_max: f64,
    margin: f64,
) -> Vec<[f64; 2]> {
    let mut curve_points = Vec::new();
    if let (Some(mu), Some(sigma)) = (s.mean, s.std_dev)
        && sigma > 0.0
    {
        let total_count: usize = s.histogram.iter().map(|h| h.1).sum();
        let scale = total_count as f64 * s.bin_width;
        let plot_min = view_min - margin;
        let plot_max = view_max + margin;
        if plot_max > plot_min {
            let step = (plot_max - plot_min) / 100.0;
            for i in 0..=100 {
                let x = plot_min + i as f64 * step;
                let z = (x - mu) / sigma;
                let y = scale
                    * (1.0 / (sigma * (2.0 * std::f64::consts::PI).sqrt()))
                    * (-0.5 * z * z).exp();
                curve_points.push([x, y]);
            }
        }
    }
    curve_points
}

fn create_box_plot_lines(
    s: &NumericStats,
    max_count: f64,
    view_min: f64,
    view_max: f64,
    margin: f64,
    show_full_range: bool,
) -> Vec<Line<'static>> {
    let mut box_lines = Vec::new();
    if let (Some(q1), Some(median), Some(q3), Some(min), Some(max)) =
        (s.q1, s.median, s.q3, s.min, s.max)
        && max > min
    {
        let y_pos = -max_count * 0.15;
        let y_h = max_count * 0.08;
        box_lines.push(Line::new(
            "Box",
            vec![
                [q1, y_pos - y_h],
                [q3, y_pos - y_h],
                [q3, y_pos + y_h],
                [q1, y_pos + y_h],
                [q1, y_pos - y_h],
            ],
        ));
        box_lines.push(Line::new(
            "Median",
            vec![[median, y_pos - y_h], [median, y_pos + y_h]],
        ));
        let w_min = if show_full_range {
            min
        } else {
            min.max(view_min - margin)
        };
        let w_max = if show_full_range {
            max
        } else {
            max.min(view_max + margin)
        };
        if w_min < q1 {
            box_lines.push(Line::new("Whisker1", vec![[w_min, y_pos], [q1, y_pos]]));
        }
        if w_max > q3 {
            box_lines.push(Line::new("Whisker2", vec![[q3, y_pos], [w_max, y_pos]]));
        }
    }
    box_lines
}

fn render_zoom_info(
    ui: &mut egui::Ui,
    histogram: &[(f64, usize)],
    bin_width: f64,
    view_min: f64,
    view_max: f64,
) {
    let total_count: usize = histogram.iter().map(|h| h.1).sum();
    let in_view_count: usize = histogram
        .iter()
        .filter(|&&(v, _)| v >= view_min - bin_width / 2.0 && v <= view_max + bin_width / 2.0)
        .map(|&(_, c)| c)
        .sum();
    let out_pct = (1.0 - (in_view_count as f64 / total_count as f64)) * 100.0;
    if out_pct > 0.5 {
        ui.label(
            egui::RichText::new(format!("Zoomed to 5th-95th ({out_pct:.1}% hidden)"))
                .size(9.0)
                .color(egui::Color32::GRAY),
        );
    }
}

fn render_categorical_plot(
    ui: &mut egui::Ui,
    name: &str,
    freq: &std::collections::HashMap<String, usize>,
    as_pie: bool,
    plot_height: f32,
) {
    if freq.is_empty() {
        ui.label("—");
        return;
    }

    if as_pie {
        render_pie_chart(ui, name, freq, plot_height);
        return;
    }

    render_horizontal_bar_chart(ui, freq, plot_height);
}

fn render_horizontal_bar_chart(
    ui: &mut egui::Ui,
    freq: &std::collections::HashMap<String, usize>,
    plot_height: f32,
) {
    let total: usize = freq.values().sum();
    if total == 0 {
        ui.label("—");
        return;
    }

    let mut sorted: Vec<_> = freq.iter().collect();
    // Sort by count descending, then by label ascending
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    let max_count = sorted.first().map(|s| *s.1 as f32).unwrap_or(1.0);

    let row_height = 16.0;
    let spacing = 4.0;
    // Calculate how many bars we can fit in the allotted plot_height
    let limit = ((plot_height + spacing) / (row_height + spacing)).floor() as usize;
    let limit = limit.min(sorted.len()).max(1);

    ui.allocate_ui(egui::vec2(ui.available_width(), plot_height), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = spacing;
            for (label, count) in sorted.iter().take(limit) {
                let fraction = **count as f32 / max_count;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // 1. Label (with fixed width for alignment)
                    let label_width = 70.0;
                    let label_text = egui::RichText::new(*label).small();
                    ui.add_sized(
                        [label_width, row_height],
                        egui::Label::new(label_text).truncate(),
                    );

                    // 2. Bar & Value area
                    let available_for_bar_and_val = ui.available_width();
                    let val_text = count.to_string();

                    // Estimate value width
                    let val_width = 35.0;
                    let bar_width_max = (available_for_bar_and_val - val_width - 10.0).max(20.0);

                    let (rect, response) = ui.allocate_at_least(
                        egui::vec2(bar_width_max, row_height),
                        egui::Sense::hover(),
                    );

                    let fill_width = fraction * rect.width();
                    let painter = ui.painter();

                    // Bar background
                    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);

                    // Bar fill
                    if fill_width > 0.0 {
                        let fill_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(fill_width, rect.height()),
                        );
                        painter.rect_filled(fill_rect, 2.0, crate::theme::ACCENT_COLOR);
                    }

                    response.on_hover_ui(|ui| {
                        ui.label(format!(
                            "{label}: {count} ({:.1}%)",
                            (**count as f32 / total as f32) * 100.0
                        ));
                    });

                    // 3. Value
                    ui.add_sized(
                        [val_width, row_height],
                        egui::Label::new(egui::RichText::new(val_text).small().weak()),
                    );
                });
            }
        });
    });
}

#[expect(clippy::indexing_slicing)]
fn render_pie_chart(
    ui: &mut egui::Ui,
    _name: &str,
    freq: &std::collections::HashMap<String, usize>,
    plot_height: f32,
) {
    let total: usize = freq.values().sum();
    if total == 0 {
        ui.label("—");
        return;
    }

    let radius = plot_height / 2.0;
    let (rect, _response) =
        ui.allocate_exact_size(egui::vec2(radius * 2.0, radius * 2.0), egui::Sense::hover());
    let center = rect.center();

    let mut start_angle = -std::f32::consts::FRAC_PI_2;
    let colors = [
        egui::Color32::from_rgb(100, 200, 100),
        egui::Color32::from_rgb(100, 100, 200),
        egui::Color32::from_rgb(200, 100, 100),
        egui::Color32::from_rgb(200, 200, 100),
        egui::Color32::from_rgb(100, 200, 200),
        egui::Color32::from_rgb(200, 100, 200),
    ];

    let mut sorted: Vec<_> = freq.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let painter = ui.painter();

    for (i, pair) in sorted.iter().take(10).enumerate() {
        let count = *pair.1;
        let sweep_angle = (count as f32 / total as f32) * 2.0 * std::f32::consts::PI;
        if sweep_angle < 0.001 {
            continue;
        }
        let _end_angle = start_angle + sweep_angle;
        let color = colors[i % colors.len()];

        let mut points = vec![center];
        let n_points = (sweep_angle / (std::f32::consts::PI / 32.0)).ceil() as usize;
        let n_points = n_points.max(3);
        for j in 0..=n_points {
            let angle = start_angle + (j as f32 / n_points as f32) * sweep_angle;
            points.push(center + egui::vec2(angle.cos(), angle.sin()) * radius);
        }

        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::new(1.0, color.gamma_multiply(0.5)),
        ));

        start_angle = _end_angle;
    }

    // Draw center hole for Donut chart
    painter.add(egui::Shape::circle_filled(
        center,
        radius * 0.4,
        ui.visuals().extreme_bg_color,
    ));

    // Draw "Other" if there are more than 10 categories
    if sorted.len() > 10 {
        let other_count: usize = sorted.iter().skip(10).map(|s| s.1).sum();
        if other_count > 0 {
            let sweep_angle = (other_count as f32 / total as f32) * 2.0 * std::f32::consts::PI;
            let _end_angle = start_angle + sweep_angle;
            let color = egui::Color32::GRAY;

            let mut points = vec![center];
            let n_points = (sweep_angle / (std::f32::consts::PI / 32.0)).ceil() as usize;
            let n_points = n_points.max(3);
            for j in 0..=n_points {
                let angle = start_angle + (j as f32 / n_points as f32) * sweep_angle;
                points.push(center + egui::vec2(angle.cos(), angle.sin()) * radius);
            }

            painter.add(egui::Shape::convex_polygon(
                points,
                color,
                egui::Stroke::new(1.0, color.gamma_multiply(0.5)),
            ));
        }
    }
}

fn render_temporal_plot(
    ui: &mut egui::Ui,
    name: &str,
    s: &TemporalStats,
    show_full_range: bool,
    plot_height: f32,
) {
    if s.histogram.is_empty() {
        ui.label("—");
        return;
    }

    let (view_min, view_max) = {
        let min = s.histogram.first().map(|h| h.0).unwrap_or(0.0);
        let max = s.histogram.last().map(|h| h.0).unwrap_or(1.0);
        let p05 = s.p05.unwrap_or(min);
        let p95 = s.p95.unwrap_or(max);

        if show_full_range {
            (min, max)
        } else {
            let full_range = max - min;
            let zoom_range = p95 - p05;

            if full_range > 3.0 * zoom_range && zoom_range > 0.0 {
                (p05, p95)
            } else {
                (min, max)
            }
        }
    };

    let range = view_max - view_min;
    let margin = if range > 0.0 { range * 0.05 } else { 1.0 };

    let bars: Vec<egui_plot::Bar> = s
        .histogram
        .iter()
        .filter(|&&(ts, _)| show_full_range || (ts >= view_min - margin && ts <= view_max + margin))
        .map(|&(ts, count)| {
            Bar::new(ts, count as f64)
                .width(s.bin_width)
                .stroke(egui::Stroke::new(
                    0.5,
                    egui::Color32::from_rgb(217, 187, 157),
                ))
        })
        .collect();

    let chart = BarChart::new("Temporal", bars).color(egui::Color32::from_rgb(238, 218, 198));

    let max_count = s.histogram.iter().map(|h| h.1).max().unwrap_or(1) as f64;

    ui.vertical(|ui| {
        Plot::new(format!("plot_temp_{name}"))
            .show_axes([false, false])
            .show_grid([false, false])
            .show_x(false)
            .show_y(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .include_y(max_count)
            .include_y(0.0)
            .include_x(view_min - margin)
            .include_x(view_max + margin)
            .set_margin_fraction(egui::Vec2::new(0.0, 0.1))
            .height(plot_height)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [view_min - margin, -max_count * 0.1],
                    [view_max + margin, max_count * 1.1],
                ));
                plot_ui.bar_chart(chart);
            });

        if !show_full_range {
            render_zoom_info(ui, &s.histogram, s.bin_width, view_min, view_max);
        }
    });
}
