use crate::search::SearchMatcher;
use crate::viewer::{FocusPlacement, ViewerCommand, ViewerState};
use crate::vim_types::RepeatableChange;
pub use crate::vim_types::{VimInput, VimMode, VimState};

impl VimMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
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
        self.repeating_zoom = None;
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
        if let Some(zoom_in) = self.repeating_zoom.take() {
            let repeated_key = if zoom_in { 'i' } else { 'o' };
            if character == repeated_key {
                self.repeating_zoom = Some(zoom_in);
                return self.zoom(viewer, zoom_in);
            }
        }
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
            'H' => self.select_visible_row(FocusPlacement::Top),
            'M' => self.select_visible_row(FocusPlacement::Center),
            'L' => self.select_visible_row(FocusPlacement::Bottom),
            'h' => self.cursor_step(viewer, false),
            'l' => self.cursor_step(viewer, true),
            'b' => self.transition(viewer, false),
            'w' => self.transition(viewer, true),
            'n' => {
                let count = self.take_count() as isize;
                self.focus_relative_wrapped(viewer, count)
            }
            'N' => {
                let count = self.take_count() as isize;
                self.focus_relative_wrapped(viewer, -count)
            }
            '*' => self.search_focused_name(viewer),
            'y' => {
                self.cancel();
                vec![ViewerCommand::CopyFocusedName]
            }
            '/' => {
                self.cancel();
                vec![ViewerCommand::BeginSearch]
            }
            ':' => {
                self.cancel();
                vec![ViewerCommand::BeginCommandPrompt]
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
                    RepeatableChange::RemoveFocused(change_count) => {
                        RepeatableChange::RemoveFocused(change_count.saturating_mul(count))
                    }
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
        self.repeating_zoom = None;
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
            ("d", 'd') => {
                let count = self.take_count();
                self.repeatable(viewer, RepeatableChange::RemoveFocused(count))
            }
            ("z", 'i') => {
                let commands = self.zoom(viewer, true);
                self.repeating_zoom = Some(true);
                commands
            }
            ("z", 'o') => {
                let commands = self.zoom(viewer, false);
                self.repeating_zoom = Some(false);
                commands
            }
            ("z", 'f') => {
                self.count = None;
                vec![ViewerCommand::FitTime]
            }
            ("z", 't') => self.reveal_focused(FocusPlacement::Top),
            ("z", 'z') => self.reveal_focused(FocusPlacement::Center),
            ("z", 'b') => self.reveal_focused(FocusPlacement::Bottom),
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
        let items = navigable_items(viewer);
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
        let items = navigable_items(viewer);
        let item = if first {
            items.first().copied()
        } else {
            items.last().copied()
        };
        item.map_or_else(Vec::new, |item| {
            vec![ViewerCommand::SetFocusedItem(item.id())]
        })
    }

    fn focus_relative_wrapped(&mut self, viewer: &ViewerState, delta: isize) -> Vec<ViewerCommand> {
        let items = search_matches(viewer);
        if items.is_empty() {
            return Vec::new();
        }
        let current = viewer
            .cursor_state()
            .focused_item()
            .and_then(|id| items.iter().position(|item| item.id() == id));
        let target = current.map_or_else(
            || if delta < 0 { items.len() - 1 } else { 0 },
            |current| (current as isize + delta).rem_euclid(items.len() as isize) as usize,
        );
        vec![ViewerCommand::SetFocusedItem(items[target].id())]
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

    fn reveal_focused(&mut self, placement: FocusPlacement) -> Vec<ViewerCommand> {
        self.count = None;
        vec![ViewerCommand::RevealFocusedItem(placement)]
    }

    fn select_visible_row(&mut self, placement: FocusPlacement) -> Vec<ViewerCommand> {
        let count = self.take_count();
        vec![ViewerCommand::SelectVisibleRow { placement, count }]
    }

    fn search_focused_name(&mut self, viewer: &ViewerState) -> Vec<ViewerCommand> {
        self.cancel();
        viewer
            .cursor_state()
            .focused_item()
            .and_then(|focused| {
                viewer
                    .displayed_items()
                    .iter()
                    .find(|item| item.id() == focused)
            })
            .and_then(|item| item.signal_id())
            .and_then(|id| viewer.waveform().signal(id))
            .map_or_else(Vec::new, |signal| {
                let leaf_name = signal.name().rsplit('.').next().unwrap_or(signal.name());
                vec![ViewerCommand::SetSearch(leaf_name.to_owned())]
            })
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
            RepeatableChange::RemoveFocused(count) => {
                let Some(start) = viewer
                    .displayed_items()
                    .iter()
                    .position(|item| item.id() == id)
                else {
                    return Vec::new();
                };
                let ids = viewer.displayed_items()[start..]
                    .iter()
                    .take(count)
                    .map(|item| item.id())
                    .collect();
                vec![ViewerCommand::RemoveDisplayedItems(ids)]
            }
        }
    }
}

fn navigable_items(viewer: &ViewerState) -> Vec<&crate::viewer::DisplayedItem> {
    viewer.displayed_items().iter().collect()
}

fn search_matches(viewer: &ViewerState) -> Vec<&crate::viewer::DisplayedItem> {
    let Some(matcher) = SearchMatcher::new(viewer.search()) else {
        return Vec::new();
    };
    viewer
        .displayed_items()
        .iter()
        .filter(|item| {
            item.signal_id()
                .and_then(|id| viewer.waveform().signal(id))
                .is_some_and(|signal| matcher.is_match(signal.name()))
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
        description: "select next / previous signal",
    },
    Binding {
        keys: "gg / G",
        description: "select first / last signal",
    },
    Binding {
        keys: "[count] H / M / L",
        description: "select top / middle / bottom visible signal",
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
        keys: "n / N / *",
        description: "select next / previous search match / search selected name",
    },
    Binding {
        keys: "y",
        description: "copy selected signal name",
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
        keys: "zt / zz / zb",
        description: "place selected signal at top / center / bottom",
    },
    Binding {
        keys: "dd",
        description: "remove selected signal",
    },
    Binding {
        keys: "[count] J / K",
        description: "move selected signal down / up",
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
        let mut viewer = ViewerState::with_waveform(Waveform::new(
            std::iter::once(("top.s0".to_owned(), transitioning))
                .chain((1..5).map(|index| (format!("top.s{index}"), Signal::new(1))))
                .collect(),
            100,
        ));
        viewer.apply(ViewerCommand::AddDisplayedSignals(
            viewer
                .waveform()
                .signals()
                .iter()
                .map(|signal| signal.id())
                .collect(),
        ));
        viewer
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
    fn counted_dd_removes_the_focused_row_and_following_rows() {
        let viewer = viewer();
        let mut vim = VimState::default();

        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[
                    VimInput::Char('3'),
                    VimInput::Char('d'),
                    VimInput::Char('d'),
                ],
            ),
            vec![ViewerCommand::RemoveDisplayedItems(
                viewer.displayed_items()[..3]
                    .iter()
                    .map(|item| item.id())
                    .collect()
            )]
        );
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
            vec![ViewerCommand::RemoveDisplayedItems(vec![viewer
                .displayed_items()[0]
                .id()])]
        );
        for command in commands {
            viewer.apply(command);
        }
        assert_eq!(
            vim.handle(VimInput::Char('.'), false, &viewer),
            vec![ViewerCommand::RemoveDisplayedItems(vec![viewer
                .displayed_items()[0]
                .id()])]
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
    fn row_motions_ignore_search_highlighting() {
        let mut viewer = viewer();
        viewer.apply(ViewerCommand::SetSearch("s3".to_owned()));
        let mut vim = VimState::default();

        assert_eq!(
            vim.handle(VimInput::Char('j'), false, &viewer),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[1].id()
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
    fn normal_search_copy_and_row_placement_use_commands() {
        let mut viewer = viewer();
        viewer.apply(ViewerCommand::SetSearch(r"s[0-4]".to_owned()));
        let mut vim = VimState::default();
        assert_eq!(
            vim.handle(VimInput::Char('N'), false, &viewer),
            vec![ViewerCommand::SetFocusedItem(
                viewer.displayed_items()[4].id()
            )]
        );
        assert_eq!(
            vim.handle(VimInput::Char('*'), false, &viewer),
            vec![ViewerCommand::SetSearch("s0".to_owned())]
        );
        assert_eq!(
            vim.handle(VimInput::Char('y'), false, &viewer),
            vec![ViewerCommand::CopyFocusedName]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('z'), VimInput::Char('z')]
            ),
            vec![ViewerCommand::RevealFocusedItem(FocusPlacement::Center)]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[VimInput::Char('2'), VimInput::Char('H')]
            ),
            vec![ViewerCommand::SelectVisibleRow {
                placement: FocusPlacement::Top,
                count: 2,
            }]
        );
    }

    #[test]
    fn colon_opens_the_command_prompt() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            vim.handle(VimInput::Char(':'), false, &viewer),
            vec![ViewerCommand::BeginCommandPrompt]
        );
    }

    #[test]
    fn repeated_i_and_o_continue_a_zoom_command() {
        let viewer = viewer();
        let mut vim = VimState::default();
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[
                    VimInput::Char('z'),
                    VimInput::Char('i'),
                    VimInput::Char('i'),
                    VimInput::Char('i'),
                ]
            ),
            vec![
                ViewerCommand::ZoomTime {
                    anchor: 50.0,
                    factor: 2.0,
                },
                ViewerCommand::ZoomTime {
                    anchor: 50.0,
                    factor: 2.0,
                },
                ViewerCommand::ZoomTime {
                    anchor: 50.0,
                    factor: 2.0,
                },
            ]
        );
        assert_eq!(
            keys(
                &mut vim,
                &viewer,
                &[
                    VimInput::Char('z'),
                    VimInput::Char('o'),
                    VimInput::Char('o'),
                ]
            ),
            vec![
                ViewerCommand::ZoomTime {
                    anchor: 50.0,
                    factor: 0.5,
                },
                ViewerCommand::ZoomTime {
                    anchor: 50.0,
                    factor: 0.5,
                },
            ]
        );
    }
}
