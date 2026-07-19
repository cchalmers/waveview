use eframe::egui;
use waveview_model::vim::{VimState, NORMAL_BINDINGS};

pub fn render_status(ui: &mut egui::Ui, vim: &VimState, message: Option<&str>) {
    ui.horizontal(|ui| {
        ui.monospace(vim.mode().label());
        let pending = vim.pending_display();
        if !pending.is_empty() {
            ui.separator();
            ui.monospace(pending);
        }
        if let Some(message) = message {
            ui.separator();
            if message.starts_with("restart required") {
                ui.colored_label(egui::Color32::YELLOW, message);
            } else if message.starts_with("UI reload failed") {
                ui.colored_label(egui::Color32::LIGHT_RED, message);
            } else {
                ui.label(message);
            }
        }
    });
}

pub fn render_key_help(ui: &mut egui::Ui) {
    egui::Grid::new("vim_key_help")
        .num_columns(2)
        .spacing(egui::vec2(24.0, 6.0))
        .striped(true)
        .show(ui, |ui| {
            for binding in NORMAL_BINDINGS {
                ui.monospace(binding.keys);
                ui.label(binding.description);
                ui.end_row();
            }
        });
}
