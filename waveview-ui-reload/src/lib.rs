#![warn(clippy::all, rust_2018_idioms)]

use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::ui_types::{MenuAction, PromptOutput};
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::vim::{VimInput, VimState};
use waveview_model::waveform::Waveform;
use waveview_model::SignalId;
use waveview_ui::PromptUiConfig;

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
    value: Option<&[waveview_model::vcd::Value]>,
    presentation: waveview_model::ui_types::SignalPresentation,
    height: f32,
    matcher: Option<&SearchMatcher>,
) -> egui::Response {
    waveview_ui::render_signal_button(ui, name, value, presentation, height, matcher)
}

#[unsafe(no_mangle)]
pub fn render_signal_context_menu(
    ui: &mut egui::Ui,
    has_alias: bool,
    current_format: waveview_model::viewer::ValueFormat,
    current_color: waveview_model::viewer::DisplayColor,
) -> Option<waveview_model::ui_types::SignalMenuAction> {
    waveview_ui::render_signal_context_menu(ui, has_alias, current_format, current_color)
}

#[unsafe(no_mangle)]
pub fn render_signal_activity_button(ui: &mut egui::Ui, selected: bool) -> bool {
    waveview_ui::render_signal_activity_button(ui, selected)
}

#[unsafe(no_mangle)]
pub fn render_file_menu(ui: &mut egui::Ui, native: bool) -> Option<MenuAction> {
    waveview_ui::render_file_menu(ui, native)
}

#[unsafe(no_mangle)]
pub fn render_edit_menu(ui: &mut egui::Ui, has_focused_item: bool) -> Option<MenuAction> {
    waveview_ui::render_edit_menu(ui, has_focused_item)
}

#[unsafe(no_mangle)]
pub fn render_view_menu(
    ui: &mut egui::Ui,
    signal_browser_open: bool,
    info_open: bool,
    samples_open: bool,
    marks_visible: bool,
    row_height: &mut f32,
) -> Option<MenuAction> {
    waveview_ui::render_view_menu(
        ui,
        signal_browser_open,
        info_open,
        samples_open,
        marks_visible,
        row_height,
    )
}

#[unsafe(no_mangle)]
pub fn render_help_menu(ui: &mut egui::Ui) -> Option<MenuAction> {
    waveview_ui::render_help_menu(ui)
}

#[unsafe(no_mangle)]
pub fn prompt_config() -> PromptUiConfig {
    waveview_ui::prompt_config()
}

#[unsafe(no_mangle)]
pub fn prompt_layout(
    body_rect: egui::Rect,
    input_height: f32,
    separator_gap: f32,
) -> (egui::Rect, egui::Rect, f32) {
    waveview_ui::prompt_layout(body_rect, input_height, separator_gap)
}

#[unsafe(no_mangle)]
pub fn render_prompt_header(ui: &mut egui::Ui) {
    waveview_ui::render_prompt_header(ui);
}

#[unsafe(no_mangle)]
pub fn render_prompt_transcript(
    ui: &mut egui::Ui,
    output: &[PromptOutput],
    viewport: egui::Rect,
    scroll_to_bottom: bool,
) {
    waveview_ui::render_prompt_transcript(ui, output, viewport, scroll_to_bottom);
}

#[unsafe(no_mangle)]
pub fn render_prompt_separator(ui: &egui::Ui, x_range: egui::Rangef, y: f32) {
    waveview_ui::render_prompt_separator(ui, x_range, y);
}

#[unsafe(no_mangle)]
pub fn render_prompt_prefix(ui: &mut egui::Ui) {
    waveview_ui::render_prompt_prefix(ui);
}

#[unsafe(no_mangle)]
pub fn prepare_prompt_input(ui: &mut egui::Ui) {
    waveview_ui::prepare_prompt_input(ui);
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
pub fn render_wave_canvas(
    ui: &mut egui::Ui,
    viewer: &ViewerState,
    canvas_rect: egui::Rect,
    interaction_rect: egui::Rect,
    visible_rows: std::ops::Range<usize>,
    row_height: f32,
    commands: &mut Vec<ViewerCommand>,
) -> (egui::Response, Option<waveview_model::DisplayedItemId>) {
    waveview_ui::render_wave_canvas(
        ui,
        viewer,
        canvas_rect,
        interaction_rect,
        visible_rows,
        row_height,
        commands,
    )
}

#[unsafe(no_mangle)]
pub fn render_timeline(
    ui: &mut egui::Ui,
    presentation: waveview_model::ui_types::TimelinePresentation,
    marks: &[waveview_model::ui_types::MarkPresentation],
    commands: &mut Vec<ViewerCommand>,
) {
    waveview_ui::render_timeline(ui, presentation, marks, commands);
}
