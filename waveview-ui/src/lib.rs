#![warn(clippy::all, rust_2018_idioms)]

mod wave;

use eframe::egui;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;

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
