use eframe::egui;
use waveview_model::search::SearchMatcher;
use waveview_model::vcd::Value;
use waveview_model::viewer::ValueFormat;

pub fn render_signal_button(
    ui: &mut egui::Ui,
    name: &str,
    value: Option<&[Value]>,
    value_format: ValueFormat,
    height: f32,
    selected: bool,
    matcher: Option<&SearchMatcher>,
) -> egui::Response {
    let text = highlighted_signal_name(ui, name, matcher);
    let mut button = egui::Button::selectable(selected, text)
        .min_size(egui::vec2(ui.available_width(), height))
        .truncate();
    if let Some(value) = value {
        let text = egui::RichText::new(crate::value::format(value, value_format)).monospace();
        button = button.right_text(if selected { text.strong() } else { text });
    }
    ui.add_sized(egui::vec2(ui.available_width(), height), button)
}

pub fn render_signal_format_menu(ui: &mut egui::Ui, current: ValueFormat) -> Option<ValueFormat> {
    let mut selected = None;
    ui.label("Value format");
    ui.separator();
    for (format, label) in [
        (ValueFormat::Binary, "Binary"),
        (ValueFormat::Hexadecimal, "Hexadecimal"),
        (ValueFormat::Unsigned, "Unsigned decimal"),
        (ValueFormat::Signed, "Signed decimal"),
        (ValueFormat::Ascii, "ASCII"),
    ] {
        if ui.selectable_label(current == format, label).clicked() {
            selected = Some(format);
            ui.close();
        }
    }
    selected
}

fn highlighted_signal_name(
    ui: &egui::Ui,
    name: &str,
    matcher: Option<&SearchMatcher>,
) -> egui::WidgetText {
    let Some(matcher) = matcher else {
        return egui::WidgetText::from(name.to_owned());
    };
    let ranges = matcher.ranges(name).collect::<Vec<_>>();
    if ranges.is_empty() {
        return egui::WidgetText::from(name.to_owned());
    }

    let normal = egui::TextFormat {
        font_id: egui::TextStyle::Button.resolve(ui.style()),
        color: ui.visuals().text_color(),
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
