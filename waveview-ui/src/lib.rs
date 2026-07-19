#![warn(clippy::all, rust_2018_idioms)]

mod activity;
mod canvas;
mod displayed_items;
mod signal_browser;
mod timeline_adapter;
mod vim_ui;
mod wave;

use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::vim::{VimInput, VimState};
use waveview_model::waveform::Waveform;
use waveview_model::SignalId;

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
    displayed_items::render_signal_button(ui, name, height, selected, matcher)
}

pub fn render_signal_activity_button(ui: &mut egui::Ui, selected: bool) -> bool {
    activity::render_signal_button(ui, selected)
}

pub fn render_signal_browser_header(ui: &mut egui::Ui, all_signals_displayed: bool) -> bool {
    signal_browser::render_header(ui, all_signals_displayed)
}

pub fn render_signal_browser_tree(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    query: &str,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    signal_browser::render_tree(ui, waveform, query, expanded, displayed, additions);
}

pub fn render_vim_status(ui: &mut egui::Ui, vim: &VimState, message: Option<&str>) {
    vim_ui::render_status(ui, vim, message);
}

pub fn render_key_help(ui: &mut egui::Ui) {
    vim_ui::render_key_help(ui);
}

pub fn timeline_height() -> f32 {
    timeline_adapter::height()
}

pub fn render_wave_canvas(
    ui: &mut egui::Ui,
    viewer: &ViewerState,
    canvas_rect: egui::Rect,
    interaction_rect: egui::Rect,
    visible_rows: std::ops::Range<usize>,
    row_height: f32,
    commands: &mut Vec<ViewerCommand>,
) {
    canvas::render(
        ui,
        viewer,
        canvas_rect,
        interaction_rect,
        visible_rows,
        row_height,
        commands,
    );
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
    timeline_adapter::render(
        ui,
        capture_end,
        view_start,
        view_end,
        cursor,
        measurement_start,
        commands,
    );
}
