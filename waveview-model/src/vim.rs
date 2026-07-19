use crate::viewer::{ViewerCommand, ViewerState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VimInput {
    Char(char),
    Ctrl(char),
    Escape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VimMode {
    Normal,
}

impl VimMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RepeatableChange {
    MoveFocused(isize),
    RemoveFocused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VimState {
    mode: VimMode,
    count: Option<usize>,
    pending: String,
    last_change: Option<RepeatableChange>,
    search_history: Vec<String>,
    search_history_cursor: Option<usize>,
    search_draft: String,
}

impl Default for VimState {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            count: None,
            pending: String::new(),
            last_change: None,
            search_history: Vec::new(),
            search_history_cursor: None,
            search_draft: String::new(),
        }
    }
}

impl VimState {
    pub fn mode(&self) -> VimMode {
        self.mode
    }

    pub fn pending_display(&self) -> String {
        let count = self
            .count
            .map_or_else(String::new, |count| count.to_string());
        format!("{count}{}", self.pending)
    }

    pub fn cancel(&mut self) {
        self.count = None;
        self.pending.clear();
    }

    pub fn accept_search(&mut self, query: &str) {
        if !query.is_empty() && self.search_history.last().is_none_or(|last| last != query) {
            self.search_history.push(query.to_owned());
        }
        self.reset_search_history_navigation();
    }

    pub fn search_history_previous(&mut self, current: &str) -> Option<String> {
        if self.search_history.is_empty() {
            return None;
        }
        let index = match self.search_history_cursor {
            Some(index) => index.saturating_sub(1),
            None => {
                self.search_draft = current.to_owned();
                self.search_history.len() - 1
            }
        };
        self.search_history_cursor = Some(index);
        Some(self.search_history[index].clone())
    }

    pub fn search_history_next(&mut self) -> Option<String> {
        let index = self.search_history_cursor?;
        if index + 1 < self.search_history.len() {
            let next = index + 1;
            self.search_history_cursor = Some(next);
            Some(self.search_history[next].clone())
        } else {
            self.search_history_cursor = None;
            Some(self.search_draft.clone())
        }
    }

    pub fn search_edited(&mut self) {
        self.reset_search_history_navigation();
    }

    fn reset_search_history_navigation(&mut self) {
        self.search_history_cursor = None;
        self.search_draft.clear();
    }

    /// Interpret one key without depending on egui. `keyboard_captured` is supplied by the UI
    /// routing layer when a text edit, dialog, or prompt owns keyboard input.
    pub fn handle(
        &mut self,
        input: VimInput,
        keyboard_captured: bool,
        viewer: &ViewerState,
    ) -> Vec<ViewerCommand> {
        if input == VimInput::Escape {
            self.cancel();
            return Vec::new();
        }
        if keyboard_captured {
            return Vec::new();
        }

        match input {
            VimInput::Char(character) => self.handle_char(character, viewer),
            VimInput::Ctrl(character) => self.handle_ctrl(character, viewer),
            VimInput::Escape => Vec::new(),
        }
    }

    fn handle_char(&mut self, character: char, viewer: &ViewerState) -> Vec<ViewerCommand> {
        if character.is_ascii_digit() && (character != '0' || self.count.is_some()) {
            let digit = character.to_digit(10).unwrap() as usize;
            self.count = Some(
                self.count
                    .unwrap_or_default()
                    .saturating_mul(10)
                    .saturating_add(digit),
            );
            return Vec::new();
        }

        if !self.pending.is_empty() {
            return self.finish_pending(character, viewer);
        }

        match character {
            'g' | 'd' | 'z' => {
                self.pending.push(character);
                Vec::new()
            }
            'j' => {
                let count = self.take_count() as isize;
                self.focus_relative(viewer, count)
            }
            'k' => {
                let count = self.take_count() as isize;
                self.focus_relative(viewer, -count)
            }
            'G' => self.focus_absolute(viewer, false),
            'h' => self.cursor_step(viewer, false),
            'l' => self.cursor_step(viewer, true),
            'b' => self.transition(viewer, false),
            'w' => self.transition(viewer, true),
            '/' => {
                self.cancel();
                self.reset_search_history_navigation();
                vec![ViewerCommand::BeginSearch]
            }
            '0' => {
                self.cancel();
                vec![ViewerCommand::SetCursor(0), ViewerCommand::RevealTime(0)]
            }
            '$' => {
                self.cancel();
                let end = viewer.capture_end();
                vec![
                    ViewerCommand::SetCursor(end),
                    ViewerCommand::RevealTime(end),
                ]
            }
            'J' => {
                let count = self.take_count() as isize;
                self.repeatable(viewer, RepeatableChange::MoveFocused(count))
            }
            'K' => {
                let count = self.take_count() as isize;
                self.repeatable(viewer, RepeatableChange::MoveFocused(-count))
            }
            'u' => {
                self.cancel();
                vec![ViewerCommand::UndoDisplayChange]
            }
            '.' => {
                let count = self.take_count();
                let Some(change) = self.last_change else {
                    return Vec::new();
                };
                let change = match change {
                    RepeatableChange::MoveFocused(delta) => {
                        RepeatableChange::MoveFocused(delta.saturating_mul(count as isize))
                    }
                    RepeatableChange::RemoveFocused => RepeatableChange::RemoveFocused,
                };
                self.commands_for_change(viewer, change)
            }
            _ => {
                self.cancel();
                Vec::new()
            }
        }
    }

    fn handle_ctrl(&mut self, character: char, viewer: &ViewerState) -> Vec<ViewerCommand> {
        let count = self.take_count();
        let span = viewer.viewport().span();
        match character.to_ascii_lowercase() {
            'f' => vec![ViewerCommand::PanTime(span * count as f64)],
            'b' => vec![ViewerCommand::PanTime(-span * count as f64)],
            'd' => vec![ViewerCommand::ScrollDisplayedHalfPages(count as isize)],
            'u' => vec![ViewerCommand::ScrollDisplayedHalfPages(-(count as isize))],
            'e' => vec![ViewerCommand::ScrollDisplayedRows(count as isize)],
            'y' => vec![ViewerCommand::ScrollDisplayedRows(-(count as isize))],
            'r' => vec![ViewerCommand::RedoDisplayChange],
            _ => Vec::new(),
        }
    }

    fn finish_pending(&mut self, character: char, viewer: &ViewerState) -> Vec<ViewerCommand> {
        let pending = std::mem::take(&mut self.pending);
        match (pending.as_str(), character) {
            ("g", 'g') => self.focus_absolute(viewer, true),
            ("d", 'd') => self.repeatable(viewer, RepeatableChange::RemoveFocused),
            ("z", 'i') => self.zoom(viewer, true),
            ("z", 'o') => self.zoom(viewer, false),
            ("z", 'f') => {
                self.count = None;
                vec![ViewerCommand::FitTime]
            }
            _ => {
                self.count = None;
                Vec::new()
            }
        }
    }

    fn take_count(&mut self) -> usize {
        self.count.take().unwrap_or(1).max(1)
    }

    fn focus_relative(&mut self, viewer: &ViewerState, delta: isize) -> Vec<ViewerCommand> {
        let items = visible_items(viewer);
        if items.is_empty() {
            return Vec::new();
        }
        let current = viewer
            .cursor_state()
            .focused_item()
            .and_then(|id| items.iter().position(|item| item.id() == id));
        let target = current.map_or_else(
            || if delta < 0 { items.len() - 1 } else { 0 },
            |current| current.saturating_add_signed(delta).min(items.len() - 1),
        );
        vec![ViewerCommand::SetFocusedItem(items[target].id())]
    }

    fn focus_absolute(&mut self, viewer: &ViewerState, first: bool) -> Vec<ViewerCommand> {
        self.count = None;
        let items = visible_items(viewer);
        let item = if first {
            items.first().copied()
        } else {
            items.last().copied()
        };
        item.map_or_else(Vec::new, |item| {
            vec![ViewerCommand::SetFocusedItem(item.id())]
        })
    }

    fn transition(&mut self, viewer: &ViewerState, forward: bool) -> Vec<ViewerCommand> {
        let count = self.take_count();
        let Some(item) = viewer
            .cursor_state()
            .focused_item()
            .and_then(|id| viewer.displayed_items().iter().find(|item| item.id() == id))
        else {
            return Vec::new();
        };
        let Some(signal) = item.signal_id().and_then(|id| viewer.waveform().signal(id)) else {
            return Vec::new();
        };

        let mut time = viewer.cursor().unwrap_or_else(|| {
            if forward {
                viewer.viewport().start().floor().max(0.0) as u64
            } else {
                viewer
                    .viewport()
                    .end()
                    .ceil()
                    .min(viewer.capture_end() as f64) as u64
            }
        });
        for _ in 0..count {
            let next = if forward {
                signal.signal().next_transition(time)
            } else {
                signal.signal().previous_transition(time)
            };
            let Some(next) = next else { break };
            time = next;
        }
        vec![
            ViewerCommand::SetCursor(time),
            ViewerCommand::RevealTime(time),
        ]
    }

    fn cursor_step(&mut self, viewer: &ViewerState, forward: bool) -> Vec<ViewerCommand> {
        let count = self.take_count() as f64;
        let viewport = viewer.viewport();
        let current = viewer
            .cursor()
            .map_or(viewport.start() + viewport.span() * 0.5, |time| time as f64);
        let delta = (viewport.span() / 20.0).max(1.0) * count;
        let time = if forward {
            current + delta
        } else {
            current - delta
        }
        .round()
        .clamp(0.0, viewer.capture_end() as f64) as u64;
        vec![
            ViewerCommand::SetCursor(time),
            ViewerCommand::RevealTime(time),
        ]
    }

    fn zoom(&mut self, viewer: &ViewerState, zoom_in: bool) -> Vec<ViewerCommand> {
        let count = self.take_count().min(16) as i32;
        let factor = if zoom_in {
            2_f64.powi(count)
        } else {
            0.5_f64.powi(count)
        };
        let anchor = viewer.cursor().map_or_else(
            || viewer.viewport().start() + viewer.viewport().span() * 0.5,
            |time| time as f64,
        );
        vec![ViewerCommand::ZoomTime { anchor, factor }]
    }

    fn repeatable(&mut self, viewer: &ViewerState, change: RepeatableChange) -> Vec<ViewerCommand> {
        self.last_change = Some(change);
        self.commands_for_change(viewer, change)
    }

    fn commands_for_change(
        &self,
        viewer: &ViewerState,
        change: RepeatableChange,
    ) -> Vec<ViewerCommand> {
        let Some(id) = viewer.cursor_state().focused_item() else {
            return Vec::new();
        };
        match change {
            RepeatableChange::MoveFocused(delta) => {
                vec![ViewerCommand::MoveDisplayedItem { id, delta }]
            }
            RepeatableChange::RemoveFocused => vec![ViewerCommand::RemoveDisplayedItem(id)],
        }
    }
}

fn visible_items(viewer: &ViewerState) -> Vec<&crate::viewer::DisplayedItem> {
    viewer
        .displayed_items()
        .iter()
        .filter(|item| {
            item.signal_id()
                .and_then(|id| viewer.waveform().signal(id))
                .is_some_and(|signal| signal.name().contains(viewer.search()))
        })
        .collect()
}

pub struct Binding {
    pub keys: &'static str,
    pub description: &'static str,
}

pub const NORMAL_BINDINGS: &[Binding] = &[
    Binding {
        keys: "[count] j / k",
        description: "focus next / previous signal",
    },
    Binding {
        keys: "gg / G",
        description: "focus first / last signal",
    },
    Binding {
        keys: "[count] h / l",
        description: "move cursor left / right by 5% of the viewport",
    },
    Binding {
        keys: "[count] b / w",
        description: "previous / next transition",
    },
    Binding {
        keys: "0 / $",
        description: "start / end of capture",
    },
    Binding {
        keys: "Ctrl-F / Ctrl-B",
        description: "pan one viewport forward / backward",
    },
    Binding {
        keys: "Ctrl-D / Ctrl-U",
        description: "scroll signals down / up half a page",
    },
    Binding {
        keys: "Ctrl-E / Ctrl-Y",
        description: "scroll signals down / up one row",
    },
    Binding {
        keys: "/",
        description: "focus signal search",
    },
    Binding {
        keys: "search: Enter",
        description: "accept search and return to Normal mode",
    },
    Binding {
        keys: "search: Ctrl-P/Up, Ctrl-N/Down",
        description: "older / newer search history",
    },
    Binding {
        keys: "zi / zo / zf",
        description: "zoom in / out / fit",
    },
    Binding {
        keys: "dd",
        description: "remove focused signal",
    },
    Binding {
        keys: "[count] J / K",
        description: "move focused signal down / up",
    },
    Binding {
        keys: "u / Ctrl-R",
        description: "undo / redo display change",
    },
    Binding {
        keys: ".",
        description: "repeat last display change",
    },
    Binding {
        keys: "Esc",
        description: "cancel pending keys",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vcd::Signal;
    use crate::waveform::Waveform;

    fn viewer() -> ViewerState {
        let mut transitioning = Signal::new(1);
        transitioning.insert_bit(10, crate::vcd::Value::V0);
        transitioning.insert_bit(30, crate::vcd::Value::V1);
        transitioning.insert_bit(70, crate::vcd::Value::V0);
        ViewerState::with_waveform(Waveform::new(
            std::iter::once(("top.s0".to_owned(), transitioning))
                .chain((1..5).map(|index| (format!("top.s{index}"), Signal::new(1))))
                .collect(),
            100,
        ))
    }

    fn keys(state: &mut VimState, viewer: &ViewerState, inputs: &[VimInput]) -> Vec<ViewerCommand> {
        inputs
            .iter()
            .flat_map(|input| state.handle(*input, false, viewer))
            .collect()
    }

    #[test]
    fn counts_and_multi_key_motions_are_interpreted() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('3'), VimInput::Char('j')]
            ),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[3].id()
            )]
        );
        assert_eq!(
            keys(&mut vim, &viewer, &[VimInput::Char('G')]),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[4].id()
            )]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('g'), VimInput::Char('g')]
            ),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[0].id()
            )]
        );
    }

    #[test]
    fn cancellation_and_keyboard_capture_clear_or_preserve_grammar() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert!(vim.handle(VimInput::Char('d'), false, &viewer).is_empty());
        assert_eq!(vim.pending_display(), "d");
        assert!(vim.handle(VimInput::Char('d'), true, &viewer).is_empty());
        assert_eq!(vim.pending_display(), "d");
        assert!(vim.handle(VimInput::Escape, true, &viewer).is_empty());
        assert_eq!(vim.pending_display(), "");
    }

    #[test]
    fn repeat_replays_the_last_display_change_against_current_focus() {
        let mut viewer = viewer();
        let mut vim = VimState::default();
        let commands = keys(
            &mut vim,
            &viewer,
            &[VimInput::Char('d'), VimInput::Char('d')],
        );
        assert_eq!(
            commands,
            vec![ViewerCommand::RemoveDisplayedItem(
                viewer.displayed_items()[0].id()
            )]
        );
        for command in commands {
            viewer.apply(command);
        }
        assert_eq!(
            vim.handle(VimInput::Char('.'), false, &viewer),
            vec![ViewerCommand::RemoveDisplayedItem(
                viewer.displayed_items()[0].id()
            )]
        );
    }

    #[test]
    fn transition_motion_moves_the_cursor_and_reveals_it() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('2'), VimInput::Char('w')]
            ),
            vec![ViewerCommand::SetCursor(30), ViewerCommand::RevealTime(30)]
        );
    }

    #[test]
    fn horizontal_motion_is_relative_to_the_visible_span() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            vim.handle(VimInput::Char('l'), false, &viewer),
            vec![ViewerCommand::SetCursor(55), ViewerCommand::RevealTime(55)]
        );
    }

    #[test]
    fn list_scrolling_and_search_are_commands_not_direct_ui_mutations() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            vim.handle(VimInput::Ctrl('d'), false, &viewer),
            vec![ViewerCommand::ScrollDisplayedHalfPages(1)]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('3'), VimInput::Ctrl('y')]
            ),
            vec![ViewerCommand::ScrollDisplayedRows(-3)]
        );
        assert_eq!(
            vim.handle(VimInput::Char('/'), false, &viewer),
            vec![ViewerCommand::BeginSearch]
        );
    }

    #[test]
    fn row_motions_only_visit_items_visible_through_search() {
        let mut viewer = viewer();
        viewer.apply(ViewerCommand::SetSearch("s3".to_owned()));
        let mut vim = VimState::default();

        assert_eq!(
            vim.handle(VimInput::Char('j'), false, &viewer),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[3].id()
            )]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('g'), VimInput::Char('g')]
            ),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[3].id()
            )]
        );
    }

    #[test]
    fn search_history_moves_backward_forward_and_restores_the_draft() {
        let mut vim = VimState::default();
        vim.accept_search("clock");
        vim.accept_search("reset");

        assert_eq!(vim.search_history_previous("dra"), Some("reset".to_owned()));
        assert_eq!(
            vim.search_history_previous("reset"),
            Some("clock".to_owned())
        );
        assert_eq!(
            vim.search_history_previous("clock"),
            Some("clock".to_owned())
        );
        assert_eq!(vim.search_history_next(), Some("reset".to_owned()));
        assert_eq!(vim.search_history_next(), Some("dra".to_owned()));
        assert_eq!(vim.search_history_next(), None);
    }

    #[test]
    fn search_history_ignores_empty_and_consecutive_duplicate_queries() {
        let mut vim = VimState::default();
        vim.accept_search("");
        vim.accept_search("clock");
        vim.accept_search("clock");

        assert_eq!(vim.search_history_previous(""), Some("clock".to_owned()));
        vim.search_edited();
        assert_eq!(vim.search_history_next(), None);
    }
}
