#![warn(clippy::all, rust_2018_idioms)]

mod wave;

use eframe::egui;
use waveview_model::vcd;

/// Paint one signal row. This deliberately small boundary is also the native hot-reload entrypoint.
pub fn render_wave(
    ui: &mut egui::Ui,
    name: &str,
    scale: f32,
    view_start: f32,
    view_end: f32,
    height: f32,
    signal: &vcd::Signal,
) {
    let mut wave = wave::Wave::new(name, scale, view_start..=view_end, signal);
    wave.height = height;
    wave.ui(ui);
}
