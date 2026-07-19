use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::waveform::Waveform;
use crate::{DisplayedItemId, SignalId};

const MIN_VIEW_SPAN: f64 = 1.0;

/// The visible interval in capture-time ticks.
///
/// Time stays in model space here. Conversion to egui points belongs in the timeline/UI crate, so
/// viewport behavior can be tested without constructing an egui context.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct TimeViewport {
    start: f64,
    span: f64,
}

impl TimeViewport {
    pub fn fit(capture_end: u64) -> Self {
        Self {
            start: 0.0,
            span: capture_span(capture_end),
        }
    }

    pub fn new(start: f64, span: f64, capture_end: u64) -> Self {
        let mut viewport = Self {
            start: finite_or(start, 0.0),
            span: finite_or(span, capture_span(capture_end)).max(MIN_VIEW_SPAN),
        };
        viewport.clamp_to_capture(capture_end);
        viewport
    }

    pub fn start(self) -> f64 {
        self.start
    }

    pub fn span(self) -> f64 {
        self.span
    }

    pub fn end(self) -> f64 {
        self.start + self.span
    }

    pub fn contains(self, time: f64) -> bool {
        self.start <= time && time <= self.end()
    }

    /// Move the visible interval by capture-time ticks.
    pub fn pan_by(&mut self, delta: f64, capture_end: u64) {
        if delta.is_finite() {
            self.start += delta;
            self.clamp_to_capture(capture_end);
        }
    }

    /// Zoom around `anchor`, where factors above one zoom in and factors below one zoom out.
    pub fn zoom_at(&mut self, anchor: f64, factor: f64, capture_end: u64) {
        if !anchor.is_finite() || !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let anchor = anchor.clamp(self.start, self.end());
        let anchor_fraction = (anchor - self.start) / self.span;
        let full_span = capture_span(capture_end);
        let new_span = (self.span / factor).clamp(MIN_VIEW_SPAN, full_span);
        self.start = anchor - anchor_fraction * new_span;
        self.span = new_span;
        self.clamp_to_capture(capture_end);
    }

    /// Pan only as far as needed to make `time` visible.
    pub fn reveal(&mut self, time: f64, capture_end: u64) {
        if !time.is_finite() {
            return;
        }

        let time = time.clamp(0.0, capture_end as f64);
        if time < self.start {
            self.start = time;
        } else if self.end() < time {
            self.start = time - self.span;
        }
        self.clamp_to_capture(capture_end);
    }

    pub fn clamp_to_capture(&mut self, capture_end: u64) {
        let full_span = capture_span(capture_end);
        self.span = finite_or(self.span, full_span).clamp(MIN_VIEW_SPAN, full_span);
        let max_start = (full_span - self.span).max(0.0);
        self.start = finite_or(self.start, 0.0).clamp(0.0, max_start);
    }
}

impl Default for TimeViewport {
    fn default() -> Self {
        Self::fit(1)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DisplayedItemKind {
    Signal(SignalId),
    Group(String),
    Divider(String),
    Timeline(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DisplayedItem {
    id: DisplayedItemId,
    kind: DisplayedItemKind,
}

impl DisplayedItem {
    pub fn id(&self) -> DisplayedItemId {
        self.id
    }

    pub fn kind(&self) -> &DisplayedItemKind {
        &self.kind
    }

    pub fn signal_id(&self) -> Option<SignalId> {
        match self.kind {
            DisplayedItemKind::Signal(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CursorState {
    time: Option<u64>,
    measurement_start: Option<u64>,
    focused_item: Option<DisplayedItemId>,
}

impl CursorState {
    pub fn time(&self) -> Option<u64> {
        self.time
    }

    pub fn measurement_start(&self) -> Option<u64> {
        self.measurement_start
    }

    pub fn focused_item(&self) -> Option<DisplayedItemId> {
        self.focused_item
    }
}

/// Durable state for navigation shared by every input frontend.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct ViewerState {
    waveform: Waveform,
    displayed_items: Vec<DisplayedItem>,
    viewport: TimeViewport,
    cursor: CursorState,
    search: String,
    display_undo: Vec<Vec<DisplayedItem>>,
    display_redo: Vec<Vec<DisplayedItem>>,
}

impl ViewerState {
    pub fn new(capture_end: u64) -> Self {
        Self::with_waveform(Waveform::new(Vec::new(), capture_end))
    }

    pub fn with_waveform(waveform: Waveform) -> Self {
        let displayed_items = displayed_items_for(&waveform);
        let focused_item = displayed_items.first().map(DisplayedItem::id);
        let capture_end = waveform.end_time();
        Self {
            waveform,
            displayed_items,
            viewport: TimeViewport::fit(capture_end),
            cursor: CursorState {
                focused_item,
                ..CursorState::default()
            },
            search: String::new(),
            display_undo: Vec::new(),
            display_redo: Vec::new(),
        }
    }

    pub fn capture_end(&self) -> u64 {
        self.waveform.end_time()
    }

    pub fn waveform(&self) -> &Waveform {
        &self.waveform
    }

    pub fn displayed_items(&self) -> &[DisplayedItem] {
        &self.displayed_items
    }

    pub fn viewport(&self) -> TimeViewport {
        self.viewport
    }

    pub fn cursor(&self) -> Option<u64> {
        self.cursor.time()
    }

    pub fn cursor_state(&self) -> &CursorState {
        &self.cursor
    }

    pub fn search(&self) -> &str {
        &self.search
    }

    /// Apply one command and return host work requested by that command.
    pub fn apply(&mut self, command: ViewerCommand) -> Vec<EffectRequest> {
        match command {
            ViewerCommand::ReplaceWaveform {
                waveform,
                preserve_view,
            } => {
                let end_time = waveform.end_time();
                self.waveform = waveform;
                self.displayed_items = displayed_items_for(&self.waveform);
                if preserve_view {
                    self.viewport.clamp_to_capture(end_time);
                } else {
                    self.viewport = TimeViewport::fit(end_time);
                }
                self.cursor = CursorState {
                    focused_item: self.displayed_items.first().map(DisplayedItem::id),
                    ..CursorState::default()
                };
                self.search.clear();
                self.display_undo.clear();
                self.display_redo.clear();
            }
            ViewerCommand::FitTime => self.viewport = TimeViewport::fit(self.capture_end()),
            ViewerCommand::PanTime(delta) => self.viewport.pan_by(delta, self.capture_end()),
            ViewerCommand::ZoomTime { anchor, factor } => {
                self.viewport.zoom_at(anchor, factor, self.capture_end());
            }
            ViewerCommand::RevealTime(time) => {
                self.viewport.reveal(time as f64, self.capture_end());
            }
            ViewerCommand::SetCursor(time) => {
                self.cursor.time = Some(time.min(self.capture_end()));
            }
            ViewerCommand::ClearCursor => {
                self.cursor.time = None;
                self.cursor.measurement_start = None;
            }
            ViewerCommand::BeginMeasurement(time) => {
                let time = time.min(self.capture_end());
                self.cursor.time = Some(time);
                self.cursor.measurement_start = Some(time);
            }
            ViewerCommand::UpdateMeasurement(time) => {
                if self.cursor.measurement_start.is_some() {
                    self.cursor.time = Some(time.min(self.capture_end()));
                }
            }
            ViewerCommand::EndMeasurement => self.cursor.measurement_start = None,
            ViewerCommand::SetFocusedItem(id) => {
                if self.displayed_items.iter().any(|item| item.id == id) {
                    self.cursor.focused_item = Some(id);
                }
            }
            ViewerCommand::SetSearch(search) => self.search = search,
            ViewerCommand::SetDisplayedOrder(order) => {
                if is_valid_order(&self.displayed_items, &order) {
                    let mut old = self.displayed_items.clone();
                    let reordered = order
                        .into_iter()
                        .map(|id| {
                            let index = old.iter().position(|item| item.id == id).unwrap();
                            old.swap_remove(index)
                        })
                        .collect();
                    self.commit_display_change(reordered);
                }
            }
            ViewerCommand::MoveDisplayedItem { id, delta } => {
                if let Some(from) = self.displayed_items.iter().position(|item| item.id == id) {
                    let mut items = self.displayed_items.clone();
                    let to = (from as isize + delta)
                        .clamp(0, items.len().saturating_sub(1) as isize)
                        as usize;
                    let item = items.remove(from);
                    items.insert(to, item);
                    self.commit_display_change(items);
                }
            }
            ViewerCommand::RemoveDisplayedItem(id) => {
                if let Some(removed_index) =
                    self.displayed_items.iter().position(|item| item.id == id)
                {
                    let mut items = self.displayed_items.clone();
                    items.retain(|item| item.id != id);
                    self.cursor.focused_item = items
                        .get(removed_index.min(items.len().saturating_sub(1)))
                        .map(DisplayedItem::id);
                    self.commit_display_change(items);
                }
            }
            ViewerCommand::UndoDisplayChange => {
                if let Some(previous) = self.display_undo.pop() {
                    self.display_redo
                        .push(std::mem::replace(&mut self.displayed_items, previous));
                    self.repair_focus();
                }
            }
            ViewerCommand::RedoDisplayChange => {
                if let Some(next) = self.display_redo.pop() {
                    self.display_undo
                        .push(std::mem::replace(&mut self.displayed_items, next));
                    self.repair_focus();
                }
            }
            ViewerCommand::ScrollDisplayedRows(delta) => {
                return vec![EffectRequest::ScrollDisplayedRows(delta)];
            }
            ViewerCommand::ScrollDisplayedHalfPages(half_pages) => {
                return vec![EffectRequest::ScrollDisplayedHalfPages(half_pages)];
            }
            ViewerCommand::BeginSearch => {
                self.search.clear();
                return vec![EffectRequest::FocusSearch];
            }
            ViewerCommand::RequestOpenFile => return vec![EffectRequest::OpenFile],
            ViewerCommand::RequestOpenUrl(url) => return vec![EffectRequest::OpenUrl(url)],
            ViewerCommand::RequestLiveConnection(url) => {
                return vec![EffectRequest::ConnectLive(url)];
            }
        }

        Vec::new()
    }

    fn commit_display_change(&mut self, items: Vec<DisplayedItem>) {
        if items != self.displayed_items {
            self.display_undo
                .push(std::mem::replace(&mut self.displayed_items, items));
            self.display_redo.clear();
            self.repair_focus();
        }
    }

    fn repair_focus(&mut self) {
        if self
            .cursor
            .focused_item
            .is_none_or(|id| !self.displayed_items.iter().any(|item| item.id == id))
        {
            self.cursor.focused_item = self.displayed_items.first().map(DisplayedItem::id);
        }
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self::new(1)
    }
}

/// An operation requested by keyboard, mouse, menus, Tcl, or remote control.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewerCommand {
    ReplaceWaveform {
        waveform: Waveform,
        preserve_view: bool,
    },
    FitTime,
    PanTime(f64),
    ZoomTime {
        anchor: f64,
        factor: f64,
    },
    RevealTime(u64),
    SetCursor(u64),
    ClearCursor,
    BeginMeasurement(u64),
    UpdateMeasurement(u64),
    EndMeasurement,
    SetFocusedItem(DisplayedItemId),
    SetSearch(String),
    SetDisplayedOrder(Vec<DisplayedItemId>),
    MoveDisplayedItem {
        id: DisplayedItemId,
        delta: isize,
    },
    RemoveDisplayedItem(DisplayedItemId),
    UndoDisplayChange,
    RedoDisplayChange,
    ScrollDisplayedRows(isize),
    ScrollDisplayedHalfPages(isize),
    BeginSearch,
    RequestOpenFile,
    RequestOpenUrl(String),
    RequestLiveConnection(String),
}

/// Imperative work performed by the stable application host rather than the reducer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EffectRequest {
    ScrollDisplayedRows(isize),
    ScrollDisplayedHalfPages(isize),
    FocusSearch,
    OpenFile,
    OpenUrl(String),
    ConnectLive(String),
}

fn capture_span(capture_end: u64) -> f64 {
    (capture_end as f64).max(MIN_VIEW_SPAN)
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn displayed_items_for(waveform: &Waveform) -> Vec<DisplayedItem> {
    waveform
        .signals()
        .iter()
        .enumerate()
        .map(|(index, signal)| DisplayedItem {
            id: DisplayedItemId::new(index as u64),
            kind: DisplayedItemKind::Signal(signal.id()),
        })
        .collect()
}

fn is_valid_order(items: &[DisplayedItem], order: &[DisplayedItemId]) -> bool {
    items.len() == order.len()
        && order.iter().copied().collect::<HashSet<_>>().len() == order.len()
        && order
            .iter()
            .all(|id| items.iter().any(|item| item.id == *id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_handles_empty_and_nonempty_captures() {
        assert_eq!(TimeViewport::fit(0), TimeViewport::new(0.0, 1.0, 0));
        assert_eq!(TimeViewport::fit(200), TimeViewport::new(0.0, 200.0, 200));
    }

    #[test]
    fn pan_is_clamped_to_the_capture() {
        let mut viewport = TimeViewport::new(20.0, 40.0, 100);

        viewport.pan_by(-30.0, 100);
        assert_eq!(viewport.start(), 0.0);
        viewport.pan_by(500.0, 100);
        assert_eq!(viewport.start(), 60.0);
        assert_eq!(viewport.end(), 100.0);
    }

    #[test]
    fn zoom_keeps_the_anchor_at_the_same_fraction() {
        let mut viewport = TimeViewport::new(20.0, 40.0, 100);

        viewport.zoom_at(30.0, 2.0, 100);

        assert_eq!(viewport, TimeViewport::new(25.0, 20.0, 100));
        assert_eq!((30.0 - viewport.start()) / viewport.span(), 0.25);
    }

    #[test]
    fn invalid_viewport_input_cannot_introduce_nan() {
        let mut viewport = TimeViewport::new(f64::NAN, f64::INFINITY, 100);
        assert_eq!(viewport, TimeViewport::fit(100));

        viewport.pan_by(f64::NAN, 100);
        viewport.zoom_at(f64::NAN, 2.0, 100);
        viewport.zoom_at(50.0, 0.0, 100);
        assert_eq!(viewport, TimeViewport::fit(100));
    }

    #[test]
    fn reveal_moves_only_when_time_is_outside_the_view() {
        let mut viewport = TimeViewport::new(20.0, 20.0, 100);

        viewport.reveal(30.0, 100);
        assert_eq!(viewport.start(), 20.0);
        viewport.reveal(70.0, 100);
        assert_eq!(viewport.start(), 50.0);
        viewport.reveal(10.0, 100);
        assert_eq!(viewport.start(), 10.0);
    }

    #[test]
    fn waveform_replacement_clamps_view_and_resets_session_state() {
        let mut state = ViewerState::new(100);
        state.apply(ViewerCommand::ZoomTime {
            anchor: 50.0,
            factor: 2.0,
        });
        state.apply(ViewerCommand::PanTime(50.0));
        state.apply(ViewerCommand::SetCursor(90));

        state.apply(ViewerCommand::SetSearch("clk".to_owned()));

        state.apply(ViewerCommand::ReplaceWaveform {
            waveform: Waveform::new(Vec::new(), 60),
            preserve_view: true,
        });

        assert_eq!(state.viewport(), TimeViewport::new(10.0, 50.0, 60));
        assert_eq!(state.cursor(), None);
        assert_eq!(state.search(), "");
    }

    #[test]
    fn capture_replacement_can_fit_and_reset_the_view() {
        let mut state = ViewerState::new(100);
        state.apply(ViewerCommand::ZoomTime {
            anchor: 50.0,
            factor: 4.0,
        });

        state.apply(ViewerCommand::ReplaceWaveform {
            waveform: Waveform::new(Vec::new(), 250),
            preserve_view: false,
        });

        assert_eq!(state.viewport(), TimeViewport::fit(250));
    }

    #[test]
    fn effects_do_not_mutate_viewer_state() {
        let mut state = ViewerState::new(100);
        let before = state.clone();

        let effects = state.apply(ViewerCommand::RequestOpenUrl(
            "https://example.test/capture.vcd".to_owned(),
        ));

        assert_eq!(state, before);
        assert_eq!(
            effects,
            vec![EffectRequest::OpenUrl(
                "https://example.test/capture.vcd".to_owned()
            )]
        );
    }

    #[test]
    fn beginning_a_search_clears_the_previous_query_and_requests_focus() {
        let mut state = ViewerState::new(100);
        state.apply(ViewerCommand::SetSearch("old query".to_owned()));

        let effects = state.apply(ViewerCommand::BeginSearch);

        assert_eq!(state.search(), "");
        assert_eq!(effects, vec![EffectRequest::FocusSearch]);
    }

    fn waveform_with_signals(count: usize) -> Waveform {
        Waveform::new(
            (0..count)
                .map(|index| (format!("signal_{index}"), crate::vcd::Signal::new(1)))
                .collect(),
            100,
        )
    }

    #[test]
    fn displayed_order_accepts_only_complete_permutations() {
        let mut state = ViewerState::with_waveform(waveform_with_signals(3));
        let original: Vec<_> = state
            .displayed_items()
            .iter()
            .map(DisplayedItem::id)
            .collect();
        let reversed = original.iter().copied().rev().collect();

        state.apply(ViewerCommand::SetDisplayedOrder(reversed));
        assert_eq!(
            state
                .displayed_items()
                .iter()
                .map(DisplayedItem::id)
                .collect::<Vec<_>>(),
            vec![original[2], original[1], original[0]]
        );

        state.apply(ViewerCommand::SetDisplayedOrder(vec![
            original[0],
            original[0],
            original[1],
        ]));
        assert_eq!(state.displayed_items()[0].id(), original[2]);
    }

    #[test]
    fn removing_focused_item_cannot_leave_dangling_focus() {
        let mut state = ViewerState::with_waveform(waveform_with_signals(2));
        let first = state.displayed_items()[0].id();
        let second = state.displayed_items()[1].id();
        assert_eq!(state.cursor_state().focused_item(), Some(first));

        state.apply(ViewerCommand::RemoveDisplayedItem(first));

        assert_eq!(state.cursor_state().focused_item(), Some(second));
        assert_eq!(state.displayed_items().len(), 1);

        state.apply(ViewerCommand::SetFocusedItem(DisplayedItemId::new(999)));
        assert_eq!(state.cursor_state().focused_item(), Some(second));

        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(state.displayed_items().len(), 2);
        state.apply(ViewerCommand::RedoDisplayChange);
        assert_eq!(state.displayed_items().len(), 1);
    }

    #[test]
    fn removing_a_middle_item_focuses_the_item_that_followed_it() {
        let mut state = ViewerState::with_waveform(waveform_with_signals(4));
        let removed = state.displayed_items()[1].id();
        let following = state.displayed_items()[2].id();
        state.apply(ViewerCommand::SetFocusedItem(removed));

        state.apply(ViewerCommand::RemoveDisplayedItem(removed));

        assert_eq!(state.cursor_state().focused_item(), Some(following));
    }

    #[test]
    fn measurement_commands_clamp_times_and_preserve_result() {
        let mut state = ViewerState::new(100);

        state.apply(ViewerCommand::BeginMeasurement(20));
        state.apply(ViewerCommand::UpdateMeasurement(150));
        assert_eq!(state.cursor_state().measurement_start(), Some(20));
        assert_eq!(state.cursor(), Some(100));

        state.apply(ViewerCommand::EndMeasurement);
        assert_eq!(state.cursor_state().measurement_start(), None);
        assert_eq!(state.cursor(), Some(100));
    }
}
