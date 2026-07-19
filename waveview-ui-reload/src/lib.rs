#![warn(clippy::all, rust_2018_idioms)]

use eframe::egui;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;

#[unsafe(no_mangle)]
pub fn timeline_height() -> f32 {
    waveview_ui::timeline_height()
}

#[unsafe(no_mangle)]
pub fn render_wave(
    ui: &mut egui::Ui,
    name: &str,
    pixels_per_tick: f32,
    view_start: f32,
    view_end: f32,
    height: f32,
    signal: &vcd::Signal,
) {
    waveview_ui::render_wave(
        ui,
        name,
        pixels_per_tick,
        view_start,
        view_end,
        height,
        signal,
    );
}

#[unsafe(no_mangle)]
pub fn render_timeline(
    ui: &mut egui::Ui,
    capture_end: u64,
    view_start: u64,
    view_end: u64,
    cursor: Option<u64>,
    measurement_start: Option<u64>,
    commands: &mut Vec<ViewerCommand>,
) {
    waveview_ui::render_timeline(
        ui,
        capture_end,
        view_start,
        view_end,
        cursor,
        measurement_start,
        commands,
    );
}
