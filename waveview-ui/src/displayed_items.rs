use eframe::egui;
use waveview_model::search::SearchMatcher;
use waveview_model::ui_types::{SignalMenuAction, SignalPresentation};
use waveview_model::vcd::Value;
use waveview_model::viewer::{DisplayColor, ValueFormat};

pub fn render_signal_button(
    ui: &mut egui::Ui,
    name: &str,
    value: Option<&[Value]>,
    presentation: SignalPresentation,
    height: f32,
    selected: bool,
    matcher: Option<&SearchMatcher>,
) -> egui::Response {
    let text_color = crate::display_color::resolve(presentation.color, ui.visuals().text_color());
    let text = highlighted_signal_name(ui, name, matcher, text_color);
    let mut button = egui::Button::selectable(selected, text)
        .min_size(egui::vec2(ui.available_width(), height))
        .truncate();
    if let Some(value) = value {
        let text = egui::RichText::new(crate::value::format(value, presentation.value_format))
            .monospace()
            .color(text_color);
        button = button.right_text(if selected { text.strong() } else { text });
    }
    ui.add_sized(egui::vec2(ui.available_width(), height), button)
}

pub fn render_signal_context_menu(
    ui: &mut egui::Ui,
    has_alias: bool,
    current_format: ValueFormat,
    current_color: DisplayColor,
) -> Option<SignalMenuAction> {
    let mut selected = None;
    if ui.button("Set alias…").clicked() {
        selected = Some(SignalMenuAction::EditAlias);
        ui.close();
    }
    if has_alias && ui.button("Clear alias").clicked() {
        selected = Some(SignalMenuAction::ClearAlias);
        ui.close();
    }
    ui.separator();
    ui.label("Value format");
    for (format, label) in [
        (ValueFormat::Binary, "Binary"),
        (ValueFormat::Hexadecimal, "Hexadecimal"),
        (ValueFormat::Unsigned, "Unsigned decimal"),
        (ValueFormat::Signed, "Signed decimal"),
        (ValueFormat::Ascii, "ASCII"),
    ] {
        if ui
            .selectable_label(current_format == format, label)
            .clicked()
        {
            selected = Some(SignalMenuAction::SetFormat(format));
            ui.close();
        }
    }
    ui.separator();
    ui.label("Color");
    ui.horizontal(|ui| {
        for color in DisplayColor::ALL {
            let fallback = ui.visuals().text_color();
            let swatch = crate::display_color::resolve(color, fallback);
            let stroke = if current_color == color {
                egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)
            } else {
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
            };
            let response = ui
                .add(
                    egui::Button::new("")
                        .min_size(egui::vec2(18.0, 18.0))
                        .fill(swatch)
                        .stroke(stroke),
                )
                .on_hover_text(color.name());
            if response.clicked() {
                selected = Some(SignalMenuAction::SetColor(color));
                ui.close();
            }
        }
    });
    selected
}

fn highlighted_signal_name(
    ui: &egui::Ui,
    name: &str,
    matcher: Option<&SearchMatcher>,
    color: egui::Color32,
) -> egui::WidgetText {
    let Some(matcher) = matcher else {
        return egui::RichText::new(name.to_owned()).color(color).into();
    };
    let ranges = matcher.ranges(name).collect::<Vec<_>>();
    if ranges.is_empty() {
        return egui::RichText::new(name.to_owned()).color(color).into();
    }

    let normal = egui::TextFormat {
        font_id: egui::TextStyle::Button.resolve(ui.style()),
        color,
        ..Default::default()
    };
    let matched = egui::TextFormat {
        background: egui::Color32::DARK_GREEN,
        ..normal.clone()
    };
    let mut job = egui::text::LayoutJob::default();
    let mut end = 0;
    for range in ranges {
        job.append(&name[end..range.start], 0.0, normal.clone());
        job.append(&name[range.clone()], 0.0, matched.clone());
        end = range.end;
    }
    job.append(&name[end..], 0.0, normal);
    job.into()
}
