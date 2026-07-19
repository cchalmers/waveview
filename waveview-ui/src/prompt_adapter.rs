use eframe::egui;
use eprompt::{OutputEntry, OutputKind};
use std::borrow::Cow;
use waveview_model::ui_types::{PromptOutput, PromptOutputKind};

const WELCOME_TEXT: &str = "Molt command console";

#[derive(Clone, Copy, Debug)]
pub struct PromptUiConfig {
    pub panel_default_height: f32,
    pub panel_minimum_height: f32,
    pub panel_maximum_height: f32,
    pub panel_resizable: bool,
    pub panel_show_separator_line: bool,
    pub transcript_horizontal_align: egui::Align,
    pub scroll_stick_to_bottom: bool,
    pub scroll_auto_shrink: [bool; 2],
    pub scroll_animated: bool,
    pub input_vertical_align: egui::Align,
    pub input_frame: bool,
    pub input_margin: egui::Margin,
    pub input_desired_width: f32,
    pub input_min_size: egui::Vec2,
    pub input_clip_text: bool,
}

pub fn config() -> PromptUiConfig {
    PromptUiConfig {
        panel_default_height: 220.0,
        panel_minimum_height: 100.0,
        panel_maximum_height: f32::INFINITY,
        panel_resizable: true,
        panel_show_separator_line: true,
        transcript_horizontal_align: egui::Align::Min,
        scroll_stick_to_bottom: true,
        scroll_auto_shrink: [false, false],
        scroll_animated: false,
        input_vertical_align: egui::Align::Center,
        input_frame: false,
        input_margin: egui::Margin::symmetric(4, 2),
        input_desired_width: f32::INFINITY,
        input_min_size: egui::Vec2::ZERO,
        input_clip_text: true,
    }
}

pub fn layout(
    body_rect: egui::Rect,
    input_height: f32,
    separator_gap: f32,
) -> (egui::Rect, egui::Rect, f32) {
    let input_rect = egui::Rect::from_min_max(
        egui::pos2(body_rect.left(), body_rect.bottom() - input_height),
        body_rect.right_bottom(),
    );
    let transcript_rect = egui::Rect::from_min_max(
        body_rect.left_top(),
        egui::pos2(body_rect.right(), input_rect.top() - separator_gap),
    );
    let separator_y = input_rect.top() - separator_gap * 0.5;
    (transcript_rect, input_rect, separator_y)
}

pub fn render_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.strong("Command console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.weak("Esc close · Enter evaluate · ↑↓ history");
        });
    });
    ui.separator();
}

pub fn render_transcript(
    ui: &mut egui::Ui,
    output: &[PromptOutput],
    viewport: egui::Rect,
    scroll_to_bottom: bool,
) {
    let output_height = output_height(ui, output, viewport.width());
    ui.add_space((viewport.height() - output_height).max(0.0));
    eprompt::render_output_entry(ui, &OutputEntry::new(WELCOME_TEXT, OutputKind::Muted));
    for entry in output {
        eprompt::render_output_entry(
            ui,
            &OutputEntry::new(display_text(entry), output_kind(entry.kind)),
        );
    }
    if scroll_to_bottom {
        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
    }
}

pub fn render_separator(ui: &egui::Ui, x_range: egui::Rangef, y: f32) {
    ui.painter()
        .hline(x_range, y, ui.visuals().widgets.noninteractive.bg_stroke);
}

pub fn render_prefix(ui: &mut egui::Ui) {
    eprompt::render_prompt_label(ui, ":");
}

pub fn prepare_input(ui: &mut egui::Ui) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
}

fn output_height(ui: &egui::Ui, output: &[PromptOutput], width: f32) -> f32 {
    let welcome_height = text_height(ui, WELCOME_TEXT, width);
    let output_height = output
        .iter()
        .map(|entry| text_height(ui, &display_text(entry), width))
        .sum::<f32>();
    let gaps = output.len() as f32 * ui.spacing().item_spacing.y;
    welcome_height + output_height + gaps
}

fn text_height(ui: &egui::Ui, text: &str, width: f32) -> f32 {
    egui::WidgetText::from(egui::RichText::new(text).monospace())
        .into_galley(ui, None, width, egui::TextStyle::Body)
        .size()
        .y
}

fn display_text(entry: &PromptOutput) -> Cow<'_, str> {
    match entry.kind {
        PromptOutputKind::Command => Cow::Owned(format!(": {}", entry.text)),
        _ => Cow::Borrowed(&entry.text),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_decoration_is_presentation_only() {
        let entry = PromptOutput::new("zoom fit", PromptOutputKind::Command);
        assert_eq!(display_text(&entry), ": zoom fit");
        assert_eq!(entry.text, "zoom fit");
    }
}
