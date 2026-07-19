use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::waveform::{ScopeNode, SignalHierarchy, Waveform};
use waveview_model::SignalId;

pub fn render_header(ui: &mut egui::Ui, all_signals_displayed: bool) -> bool {
    let mut add_all = false;
    ui.horizontal(|ui| {
        ui.strong("Available signals");
        add_all = ui
            .add_enabled(!all_signals_displayed, egui::Button::new("Add all"))
            .clicked();
    });
    add_all
}

pub fn render_tree(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    query: &str,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    let matcher = SearchMatcher::new(query);
    render_hierarchy(
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
fn render_hierarchy(
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
        render_signal(
            ui, waveform, signal_id, matcher, false, displayed, additions,
        );
    }
    for scope in hierarchy.scopes() {
        render_scope(
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
fn render_scope(
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
                render_signal(
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
                render_scope(
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

fn render_signal(
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
