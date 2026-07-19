use eframe::egui;
use eprompt::{OutputEntry, OutputKind};
use waveview_model::ui_types::{PromptOutput, PromptOutputKind};

pub fn default_height() -> f32 {
    220.0
}

pub fn render_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.strong("Command console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.weak("Esc close · Enter evaluate · ↑↓ history");
        });
    });
}

pub fn render_output(ui: &mut egui::Ui, output: &[PromptOutput]) {
    for entry in output.iter().rev() {
        eprompt::render_output_entry(ui, &OutputEntry::new(&entry.text, output_kind(entry.kind)));
    }
}

pub fn render_prefix(ui: &mut egui::Ui) {
    eprompt::render_prompt_label(ui, ":");
}

fn output_kind(kind: PromptOutputKind) -> OutputKind {
    match kind {
        PromptOutputKind::Normal => OutputKind::Normal,
        PromptOutputKind::Command => OutputKind::Command,
        PromptOutputKind::Result => OutputKind::Success,
        PromptOutputKind::Error => OutputKind::Error,
        PromptOutputKind::Muted => OutputKind::Muted,
    }
}
