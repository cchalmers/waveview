#![warn(clippy::all, rust_2018_idioms)]

use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::vim::{VimInput, VimState};
use waveview_model::waveform::Waveform;
use waveview_model::SignalId;

#[unsafe(no_mangle)]
pub fn handle_vim_input(
    vim: &mut VimState,
    input: VimInput,
    keyboard_captured: bool,
    viewer: &ViewerState,
    commands: &mut Vec<ViewerCommand>,
) {
    waveview_ui::handle_vim_input(vim, input, keyboard_captured, viewer, commands);
}

#[unsafe(no_mangle)]
pub fn render_signal_button(
    ui: &mut egui::Ui,
    name: &str,
    height: f32,
    selected: bool,
    matcher: Option<&SearchMatcher>,
) -> bool {
    waveview_ui::render_signal_button(ui, name, height, selected, matcher)
}

#[unsafe(no_mangle)]
pub fn render_signal_activity_button(ui: &mut egui::Ui, selected: bool) -> bool {
    waveview_ui::render_signal_activity_button(ui, selected)
}

#[unsafe(no_mangle)]
pub fn render_signal_browser_header(ui: &mut egui::Ui, all_signals_displayed: bool) -> bool {
    waveview_ui::render_signal_browser_header(ui, all_signals_displayed)
}

#[unsafe(no_mangle)]
pub fn render_signal_browser_tree(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    query: &str,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    waveview_ui::render_signal_browser_tree(ui, waveform, query, expanded, displayed, additions);
}

#[unsafe(no_mangle)]
pub fn render_vim_status(ui: &mut egui::Ui, vim: &VimState, message: Option<&str>) {
    waveview_ui::render_vim_status(ui, vim, message);
}

#[unsafe(no_mangle)]
pub fn render_key_help(ui: &mut egui::Ui) {
    waveview_ui::render_key_help(ui);
}

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
