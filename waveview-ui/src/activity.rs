use eframe::egui;

pub fn render_signal_button(ui: &mut egui::Ui, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
    let visuals = ui.style().interact_selectable(&response, selected);
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let icon = rect.shrink2(egui::vec2(7.0, 9.0));
    let low = icon.bottom();
    let high = icon.top();
    let x0 = icon.left();
    let x1 = egui::lerp(icon.x_range(), 0.28);
    let x2 = egui::lerp(icon.x_range(), 0.62);
    let x3 = icon.right();
    ui.painter().add(egui::Shape::line(
        vec![
            egui::pos2(x0, low),
            egui::pos2(x1, low),
            egui::pos2(x1, high),
            egui::pos2(x2, high),
            egui::pos2(x2, low),
            egui::pos2(x3, low),
        ],
        visuals.fg_stroke,
    ));

    response.on_hover_text("Signals").clicked()
}
