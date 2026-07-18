#![warn(clippy::all, rust_2018_idioms)]

use eframe::egui;
use waveview_model::vcd;

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
