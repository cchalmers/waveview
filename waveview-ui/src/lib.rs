#![warn(clippy::all, rust_2018_idioms)]

mod wave;

use eframe::egui;
use waveview_model::search::SearchMatcher;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::vim::{VimInput, VimState, NORMAL_BINDINGS};

pub fn handle_vim_input(
    vim: &mut VimState,
    input: VimInput,
    keyboard_captured: bool,
    viewer: &ViewerState,
    commands: &mut Vec<ViewerCommand>,
) {
    commands.extend(vim.handle(input, keyboard_captured, viewer));
}

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

pub fn render_vim_status(ui: &mut egui::Ui, vim: &VimState, message: Option<&str>) {
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

pub fn timeline_height() -> f32 {
    timeline::DEFAULT_HEIGHT
}

/// Paint one signal row. This deliberately small boundary is also the native hot-reload entrypoint.
pub fn render_wave(
    ui: &mut egui::Ui,
    name: &str,
    pixels_per_tick: f32,
    view_start: f32,
    view_end: f32,
    height: f32,
    signal: &vcd::Signal,
) {
    let mut wave = wave::Wave::new(name, pixels_per_tick, view_start..=view_end, signal);
    wave.height = height;
    wave.ui(ui);
}

/// Render the generic timeline and translate its backend-free actions into viewer commands.
pub fn render_timeline(
    ui: &mut egui::Ui,
    capture_end: u64,
    view_start: u64,
    view_end: u64,
    cursor: Option<u64>,
    measurement_start: Option<u64>,
    commands: &mut Vec<ViewerCommand>,
) {
    let full = timeline::TimeRange::new(0, capture_end.max(1));
    let visible = timeline::TimeRange::new(view_start, view_end).clamp_to(full);
    let selection = measurement_start
        .zip(cursor)
        .map(|(start, end)| timeline::TimeRange::new(start, end));
    let response = timeline::Timeline::new(full, visible, &|time| time.to_string())
        .cursor(cursor)
        .selection(selection)
        .show(ui);

    commands.extend(response.actions.into_iter().map(|action| match action {
        timeline::Action::Pan(delta) => ViewerCommand::PanTime(delta),
        timeline::Action::Zoom { anchor, factor } => ViewerCommand::ZoomTime {
            anchor: anchor as f64,
            factor,
        },
        timeline::Action::Fit => ViewerCommand::FitTime,
        timeline::Action::SetCursor(time) => ViewerCommand::SetCursor(time),
        timeline::Action::BeginSelection(time) => ViewerCommand::BeginMeasurement(time),
        timeline::Action::UpdateSelection(time) => ViewerCommand::UpdateMeasurement(time),
        timeline::Action::EndSelection => ViewerCommand::EndMeasurement,
    }));
}
