use eframe::egui;
use waveview_model::search::SearchMatcher;

pub fn render_signal_button(
    ui: &mut egui::Ui,
    name: &str,
    height: f32,
    selected: bool,
    matcher: Option<&SearchMatcher>,
) -> bool {
    let text = highlighted_signal_name(ui, name, matcher);
    ui.add_sized(
        egui::vec2(ui.available_width(), height),
        egui::Button::selectable(selected, text).truncate(),
    )
    .clicked()
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
