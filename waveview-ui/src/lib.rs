#![warn(clippy::all, rust_2018_idioms)]

mod wave;

use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::vim::{VimInput, VimState, NORMAL_BINDINGS};
use waveview_model::waveform::{ScopeNode, SignalHierarchy, Waveform};
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
    let text = highlighted_signal_name(ui, name, matcher);
    ui.add_sized(
        egui::vec2(ui.available_width(), height),
        egui::Button::selectable(selected, text).truncate(),
    )
    .clicked()
}

pub fn render_signal_activity_button(ui: &mut egui::Ui, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
    let visuals = ui.style().interact_selectable(&response, selected);
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let icon = rect.shrink2(egui::vec2(7.0, 9.0));
    let low = icon.bottom();
    let high = icon.top();
    let x0 = icon.left();
    let x1 = egui::lerp(icon.x_range(), 0.28);
    let x2 = egui::lerp(icon.x_range(), 0.62);
    let x3 = icon.right();
    ui.painter().add(egui::Shape::line(
        vec![
            egui::pos2(x0, low),
            egui::pos2(x1, low),
            egui::pos2(x1, high),
            egui::pos2(x2, high),
            egui::pos2(x2, low),
            egui::pos2(x3, low),
        ],
        visuals.fg_stroke,
    ));

    response.on_hover_text("Signals").clicked()
}

pub fn render_signal_browser_header(ui: &mut egui::Ui, all_signals_displayed: bool) -> bool {
    let mut add_all = false;
    ui.horizontal(|ui| {
        ui.strong("Available signals");
        add_all = ui
            .add_enabled(!all_signals_displayed, egui::Button::new("Add all"))
            .clicked();
    });
    add_all
}

pub fn render_signal_browser_tree(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    query: &str,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    let matcher = SearchMatcher::new(query);
    render_signal_hierarchy(
        ui,
        waveform,
        waveform.hierarchy(),
        query,
        matcher.as_ref(),
        expanded,
        displayed,
        additions,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_signal_hierarchy(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    hierarchy: &SignalHierarchy,
    query: &str,
    matcher: Option<&SearchMatcher>,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    if !query.is_empty() && matcher.is_none() {
        ui.colored_label(egui::Color32::LIGHT_RED, "Invalid regular expression");
        return;
    }
    for &signal_id in hierarchy.signals() {
        render_available_signal(
            ui, waveform, signal_id, matcher, false, displayed, additions,
        );
    }
    for scope in hierarchy.scopes() {
        render_available_scope(
            ui,
            waveform,
            scope,
            "",
            matcher,
            !query.is_empty(),
            false,
            expanded,
            displayed,
            additions,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_available_scope(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    scope: &ScopeNode,
    parent_path: &str,
    matcher: Option<&SearchMatcher>,
    searching: bool,
    ancestor_matched: bool,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    let path = if parent_path.is_empty() {
        scope.metadata().name().to_owned()
    } else {
        format!("{parent_path}.{}", scope.metadata().name())
    };
    let scope_matched = ancestor_matched
        || matcher.is_some_and(|matcher| {
            matcher.is_match(scope.metadata().name()) || matcher.is_match(&path)
        });
    if searching && !scope_matched && !scope_contains_match(waveform, scope, matcher) {
        return;
    }

    let is_open = searching || expanded.contains(&path);
    let scope_signal_ids = scope.signal_ids_recursive();
    let scope_is_displayed = scope_signal_ids
        .iter()
        .all(|signal_id| displayed.contains(signal_id));
    ui.horizontal(|ui| {
        if ui.small_button(if is_open { "▾" } else { "▸" }).clicked() && !searching {
            if is_open {
                expanded.remove(&path);
            } else {
                expanded.insert(path.clone());
            }
        }
        ui.label(scope.metadata().name())
            .on_hover_text(format!("{:?} scope", scope.metadata().kind()));
        if ui
            .add_enabled(!scope_is_displayed, egui::Button::new("+"))
            .on_hover_text("Add scope recursively")
            .clicked()
        {
            additions.extend(scope_signal_ids);
        }
    });

    if is_open {
        ui.indent((&path, "scope"), |ui| {
            for &signal_id in scope.signals() {
                render_available_signal(
                    ui,
                    waveform,
                    signal_id,
                    matcher,
                    scope_matched,
                    displayed,
                    additions,
                );
            }
            for child in scope.scopes() {
                render_available_scope(
                    ui,
                    waveform,
                    child,
                    &path,
                    matcher,
                    searching,
                    scope_matched,
                    expanded,
                    displayed,
                    additions,
                );
            }
        });
    }
}

fn scope_contains_match(
    waveform: &Waveform,
    scope: &ScopeNode,
    matcher: Option<&SearchMatcher>,
) -> bool {
    let Some(matcher) = matcher else {
        return true;
    };
    scope.signals().iter().any(|&id| {
        waveform
            .signal(id)
            .is_some_and(|signal| matcher.is_match(signal.name()))
    }) || scope
        .scopes()
        .iter()
        .any(|child| scope_contains_match(waveform, child, Some(matcher)))
}

fn render_available_signal(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    signal_id: SignalId,
    matcher: Option<&SearchMatcher>,
    ancestor_matched: bool,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    let Some(signal) = waveform.signal(signal_id) else {
        return;
    };
    if !ancestor_matched && matcher.is_some_and(|matcher| !matcher.is_match(signal.name())) {
        return;
    }
    let metadata = signal.metadata();
    ui.horizontal(|ui| {
        let already_displayed = displayed.contains(&signal_id);
        if ui
            .add_enabled(!already_displayed, egui::Button::new("+"))
            .on_hover_text(if already_displayed {
                "Already displayed"
            } else {
                "Add signal"
            })
            .clicked()
        {
            additions.push(signal_id);
        }
        ui.label(metadata.reference()).on_hover_text(format!(
            "{} · {:?} · {} bit{}{}",
            signal.name(),
            metadata.kind(),
            metadata.width(),
            if metadata.width() == 1 { "" } else { "s" },
            metadata
                .index()
                .map_or_else(String::new, |index| format!(" · {index:?}"))
        ));
    });
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
