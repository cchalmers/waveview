use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueFormat {
    Binary,
    #[default]
    Hexadecimal,
    Unsigned,
    Signed,
    Ascii,
}

impl ValueFormat {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "binary" | "bin" => Some(Self::Binary),
            "hexadecimal" | "hex" => Some(Self::Hexadecimal),
            "unsigned" | "uint" => Some(Self::Unsigned),
            "signed" | "int" => Some(Self::Signed),
            "ascii" => Some(Self::Ascii),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Gray,
}

impl DisplayColor {
    pub const ALL: [Self; 9] = [
        Self::Default,
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Cyan,
        Self::Blue,
        Self::Purple,
        Self::Gray,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
            Self::Purple => "purple",
            Self::Gray => "gray",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|color| color.name() == name)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DisplayedItem {
    id: DisplayedItemId,
    kind: DisplayedItemKind,
    #[serde(default)]
    value_format: ValueFormat,
    #[serde(default)]
    alias: Option<String>,
    #[serde(default)]
    color: DisplayColor,
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

    pub fn value_format(&self) -> ValueFormat {
        self.value_format
    }

    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    pub fn color(&self) -> DisplayColor {
        self.color
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CursorState {
    time: Option<u64>,
    measurement_start: Option<u64>,
    focused_item: Option<DisplayedItemId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarkPosition {
    item: DisplayedItemId,
    time: u64,
}

impl MarkPosition {
    pub fn item(self) -> DisplayedItemId {
        self.item
    }

    pub fn time(self) -> u64 {
        self.time
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FocusPlacement {
    Top,
    Center,
    Bottom,
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
    marks: BTreeMap<char, MarkPosition>,
    previous_jump: Option<MarkPosition>,
    jump_list: Vec<MarkPosition>,
    jump_index: Option<usize>,
    display_undo: Vec<Vec<DisplayedItem>>,
    display_redo: Vec<Vec<DisplayedItem>>,
    next_displayed_item_id: u64,
}

impl ViewerState {
    pub fn new(capture_end: u64) -> Self {
        Self::with_waveform(Waveform::new(Vec::new(), capture_end))
    }

    pub fn with_waveform(waveform: Waveform) -> Self {
        let displayed_items = displayed_items_for(&waveform);
        let next_displayed_item_id = displayed_items.len() as u64;
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
            marks: BTreeMap::new(),
            previous_jump: None,
            jump_list: Vec::new(),
            jump_index: None,
            display_undo: Vec::new(),
            display_redo: Vec::new(),
            next_displayed_item_id,
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

    pub fn marks(&self) -> &BTreeMap<char, MarkPosition> {
        &self.marks
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
                self.marks.clear();
                self.previous_jump = None;
                self.jump_list.clear();
                self.jump_index = None;
                self.display_undo.clear();
                self.display_redo.clear();
                self.next_displayed_item_id = self.displayed_items.len() as u64;
            }
            ViewerCommand::FitTime => self.viewport = TimeViewport::fit(self.capture_end()),
            ViewerCommand::PanTime(delta) => self.viewport.pan_by(delta, self.capture_end()),
            ViewerCommand::ZoomTime { anchor, factor } => {
                self.viewport.zoom_at(anchor, factor, self.capture_end());
            }
            ViewerCommand::RevealTime(time) => {
                self.viewport.reveal(time as f64, self.capture_end());
            }
            ViewerCommand::JumpToTime(time) => {
                let time = time.min(self.capture_end());
                if self.cursor.time != Some(time) {
                    let origin = self.current_position();
                    self.cursor.time = Some(time);
                    self.viewport.reveal(time as f64, self.capture_end());
                    if let Some(origin) = origin {
                        self.record_jump(origin);
                    }
                }
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
            ViewerCommand::FocusDisplayedRelative(delta) => {
                let navigable = self
                    .displayed_items
                    .iter()
                    .filter(|item| item.signal_id().is_some())
                    .collect::<Vec<_>>();
                if !navigable.is_empty() {
                    let current = self
                        .cursor
                        .focused_item
                        .and_then(|focused| navigable.iter().position(|item| item.id == focused));
                    let target = current.map_or_else(
                        || if delta < 0 { navigable.len() - 1 } else { 0 },
                        |current| {
                            current
                                .saturating_add_signed(delta)
                                .min(navigable.len() - 1)
                        },
                    );
                    self.cursor.focused_item = Some(navigable[target].id);
                }
            }
            ViewerCommand::SetSearch(search) => self.search = search,
            ViewerCommand::AddDisplayedSignals(signal_ids) => {
                let mut existing = self
                    .displayed_items
                    .iter()
                    .filter_map(DisplayedItem::signal_id)
                    .collect::<HashSet<_>>();
                let mut items = self.displayed_items.clone();
                for signal_id in signal_ids {
                    if self.waveform.signal(signal_id).is_some() && existing.insert(signal_id) {
                        items.push(DisplayedItem {
                            id: DisplayedItemId::new(self.next_displayed_item_id),
                            kind: DisplayedItemKind::Signal(signal_id),
                            value_format: ValueFormat::default(),
                            alias: None,
                            color: DisplayColor::default(),
                        });
                        self.next_displayed_item_id += 1;
                    }
                }
                self.commit_display_change(items);
            }
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
                self.remove_displayed_items(&HashSet::from([id]));
            }
            ViewerCommand::RemoveDisplayedItems(ids) => {
                self.remove_displayed_items(&ids.into_iter().collect());
            }
            ViewerCommand::SetFocusedValueFormat(format) => {
                if let Some(focused) = self.cursor.focused_item {
                    self.set_displayed_value_format(focused, format);
                }
            }
            ViewerCommand::SetDisplayedValueFormat { id, format } => {
                self.set_displayed_value_format(id, format);
            }
            ViewerCommand::SetFocusedAlias(alias) => {
                if let Some(focused) = self.cursor.focused_item {
                    self.set_displayed_alias(focused, alias);
                }
            }
            ViewerCommand::SetDisplayedAlias { id, alias } => {
                self.set_displayed_alias(id, alias);
            }
            ViewerCommand::SetFocusedColor(color) => {
                if let Some(focused) = self.cursor.focused_item {
                    self.set_displayed_color(focused, color);
                }
            }
            ViewerCommand::SetDisplayedColor { id, color } => {
                self.set_displayed_color(id, color);
            }
            ViewerCommand::SetMark(name) => {
                if name.is_ascii_lowercase() {
                    if let Some(position) = self.current_position() {
                        self.marks.insert(name, position);
                    }
                }
            }
            ViewerCommand::JumpToMark { name, exact } => {
                if let Some(position) = self.marks.get(&name).copied() {
                    return self.jump_to_position(position, exact, true);
                }
            }
            ViewerCommand::JumpToPrevious { exact } => {
                if let Some(position) = self.previous_jump {
                    return self.jump_to_position(position, exact, true);
                }
            }
            ViewerCommand::TraverseJumpList(delta) => {
                return self.traverse_jump_list(delta);
            }
            ViewerCommand::DeleteMarks(names) => {
                for name in names {
                    self.marks.remove(&name);
                }
            }
            ViewerCommand::ClearMarks => self.marks.clear(),
            ViewerCommand::RequestMarkList => {
                return vec![EffectRequest::PromptText(self.mark_list())];
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
            ViewerCommand::BeginCommandPrompt => {
                return vec![EffectRequest::FocusCommandPrompt];
            }
            ViewerCommand::RevealFocusedItem(placement) => {
                return vec![EffectRequest::RevealFocusedItem(placement)];
            }
            ViewerCommand::SelectVisibleRow { placement, count } => {
                return vec![EffectRequest::SelectVisibleRow { placement, count }];
            }
            ViewerCommand::CopyFocusedName => {
                let name = self
                    .cursor
                    .focused_item
                    .and_then(|focused| self.displayed_items.iter().find(|item| item.id == focused))
                    .and_then(|item| {
                        item.alias().map(str::to_owned).or_else(|| {
                            item.signal_id()
                                .and_then(|id| self.waveform.signal(id))
                                .map(|signal| signal.name().to_owned())
                        })
                    });
                return name.map_or_else(Vec::new, |name| vec![EffectRequest::CopyText(name)]);
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

    fn set_displayed_value_format(&mut self, id: DisplayedItemId, format: ValueFormat) {
        let mut items = self.displayed_items.clone();
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            item.value_format = format;
        }
        self.commit_display_change(items);
    }

    fn set_displayed_alias(&mut self, id: DisplayedItemId, alias: Option<String>) {
        let alias = alias.and_then(|alias| {
            let alias = alias.trim().to_owned();
            (!alias.is_empty()).then_some(alias)
        });
        let mut items = self.displayed_items.clone();
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            item.alias = alias;
        }
        self.commit_display_change(items);
    }

    fn set_displayed_color(&mut self, id: DisplayedItemId, color: DisplayColor) {
        let mut items = self.displayed_items.clone();
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            item.color = color;
        }
        self.commit_display_change(items);
    }

    fn current_position(&self) -> Option<MarkPosition> {
        Some(MarkPosition {
            item: self.cursor.focused_item?,
            time: self.cursor.time?,
        })
    }

    fn jump_to_position(
        &mut self,
        position: MarkPosition,
        exact: bool,
        record_previous: bool,
    ) -> Vec<EffectRequest> {
        if !self
            .displayed_items
            .iter()
            .any(|item| item.id == position.item)
        {
            return Vec::new();
        }
        let previous = self.current_position();
        self.cursor.focused_item = Some(position.item);
        if exact {
            let time = position.time.min(self.capture_end());
            self.cursor.time = Some(time);
            self.viewport.reveal(time as f64, self.capture_end());
        }
        if record_previous {
            if let Some(previous) = previous {
                self.record_jump(previous);
            }
        }
        vec![EffectRequest::RevealFocusedItem(FocusPlacement::Center)]
    }

    fn record_jump(&mut self, origin: MarkPosition) {
        if let Some(index) = self.jump_index.take() {
            self.jump_list.truncate(index.saturating_add(1));
        }
        if self.jump_list.last() != Some(&origin) {
            self.jump_list.push(origin);
        }
        const MAX_JUMPS: usize = 100;
        if self.jump_list.len() > MAX_JUMPS {
            self.jump_list.remove(0);
        }
        self.previous_jump = Some(origin);
    }

    fn traverse_jump_list(&mut self, delta: isize) -> Vec<EffectRequest> {
        let Some(current) = self.current_position() else {
            return Vec::new();
        };
        if self.jump_index.is_none() {
            if self.jump_list.last() != Some(&current) {
                self.jump_list.push(current);
            }
            self.jump_index = self.jump_list.len().checked_sub(1);
        }
        let Some(index) = self.jump_index else {
            return Vec::new();
        };
        let target = index
            .saturating_add_signed(delta)
            .min(self.jump_list.len().saturating_sub(1));
        if target == index {
            return Vec::new();
        }
        self.jump_index = Some(target);
        self.jump_to_position(self.jump_list[target], true, false)
    }

    fn mark_list(&self) -> String {
        if self.marks.is_empty() {
            return "No marks set".to_owned();
        }
        let mut result = String::from("mark  time  signal");
        for (&name, &position) in &self.marks {
            let signal = self
                .displayed_items
                .iter()
                .find(|item| item.id == position.item)
                .and_then(|item| {
                    item.alias().map(str::to_owned).or_else(|| {
                        item.signal_id()
                            .and_then(|id| self.waveform.signal(id))
                            .map(|signal| signal.name().to_owned())
                    })
                })
                .unwrap_or_else(|| "[unresolved]".to_owned());
            result.push_str(&format!("\n {name}    {}  {signal}", position.time));
        }
        result
    }

    fn remove_displayed_items(&mut self, ids: &HashSet<DisplayedItemId>) {
        let focused_index = self.cursor.focused_item.and_then(|focused| {
            ids.contains(&focused)
                .then(|| {
                    self.displayed_items
                        .iter()
                        .position(|item| item.id == focused)
                })
                .flatten()
        });
        let mut items = self.displayed_items.clone();
        items.retain(|item| !ids.contains(&item.id));
        if let Some(index) = focused_index {
            self.cursor.focused_item = items
                .get(index.min(items.len().saturating_sub(1)))
                .map(DisplayedItem::id);
        }
        self.commit_display_change(items);
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
    JumpToTime(u64),
    SetCursor(u64),
    ClearCursor,
    BeginMeasurement(u64),
    UpdateMeasurement(u64),
    EndMeasurement,
    SetFocusedItem(DisplayedItemId),
    FocusDisplayedRelative(isize),
    SetSearch(String),
    AddDisplayedSignals(Vec<SignalId>),
    SetDisplayedOrder(Vec<DisplayedItemId>),
    MoveDisplayedItem {
        id: DisplayedItemId,
        delta: isize,
    },
    RemoveDisplayedItem(DisplayedItemId),
    RemoveDisplayedItems(Vec<DisplayedItemId>),
    SetFocusedValueFormat(ValueFormat),
    SetDisplayedValueFormat {
        id: DisplayedItemId,
        format: ValueFormat,
    },
    SetFocusedAlias(Option<String>),
    SetDisplayedAlias {
        id: DisplayedItemId,
        alias: Option<String>,
    },
    SetFocusedColor(DisplayColor),
    SetDisplayedColor {
        id: DisplayedItemId,
        color: DisplayColor,
    },
    SetMark(char),
    JumpToMark {
        name: char,
        exact: bool,
    },
    JumpToPrevious {
        exact: bool,
    },
    TraverseJumpList(isize),
    DeleteMarks(Vec<char>),
    ClearMarks,
    RequestMarkList,
    UndoDisplayChange,
    RedoDisplayChange,
    ScrollDisplayedRows(isize),
    ScrollDisplayedHalfPages(isize),
    BeginSearch,
    BeginCommandPrompt,
    RevealFocusedItem(FocusPlacement),
    SelectVisibleRow {
        placement: FocusPlacement,
        count: usize,
    },
    CopyFocusedName,
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
    FocusCommandPrompt,
    RevealFocusedItem(FocusPlacement),
    SelectVisibleRow {
        placement: FocusPlacement,
        count: usize,
    },
    CopyText(String),
    PromptText(String),
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

fn is_valid_order(items: &[DisplayedItem], order: &[DisplayedItemId]) -> bool {
    items.len() == order.len()
        && order.iter().copied().collect::<HashSet<_>>().len() == order.len()
        && order
            .iter()
            .all(|id| items.iter().any(|item| item.id == *id))
}

fn displayed_items_for(waveform: &Waveform) -> Vec<DisplayedItem> {
    waveform
        .signals()
        .iter()
        .enumerate()
        .map(|(index, signal)| DisplayedItem {
            id: DisplayedItemId::new(index as u64),
            kind: DisplayedItemKind::Signal(signal.id()),
            value_format: ValueFormat::default(),
            alias: None,
            color: DisplayColor::default(),
        })
        .collect()
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
    fn loaded_signals_are_displayed_and_can_be_removed_then_readded() {
        let mut state = ViewerState::with_waveform(waveform_with_signals(3));
        assert_eq!(state.displayed_items().len(), 3);
        let removed_item = state.displayed_items()[1].id();
        state.apply(ViewerCommand::RemoveDisplayedItem(removed_item));

        state.apply(ViewerCommand::AddDisplayedSignals(vec![
            SignalId::new(1),
            SignalId::new(1),
            SignalId::new(99),
            SignalId::new(2),
        ]));

        assert_eq!(
            state
                .displayed_items()
                .iter()
                .filter_map(DisplayedItem::signal_id)
                .collect::<Vec<_>>(),
            vec![SignalId::new(0), SignalId::new(2), SignalId::new(1)]
        );

        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(state.displayed_items().len(), 2);
        state.apply(ViewerCommand::RedoDisplayChange);
        assert_eq!(state.displayed_items().len(), 3);
    }

    #[test]
    fn focused_value_format_is_durable_and_undoable() {
        let mut state = state_with_signals(2);
        assert_eq!(
            state.displayed_items()[0].value_format(),
            ValueFormat::Hexadecimal
        );

        state.apply(ViewerCommand::SetFocusedValueFormat(ValueFormat::Signed));
        assert_eq!(
            state.displayed_items()[0].value_format(),
            ValueFormat::Signed
        );

        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(
            state.displayed_items()[0].value_format(),
            ValueFormat::Hexadecimal
        );
        state.apply(ViewerCommand::RedoDisplayChange);
        assert_eq!(
            state.displayed_items()[0].value_format(),
            ValueFormat::Signed
        );
    }

    #[test]
    fn focused_alias_and_color_are_durable_and_undoable() {
        let mut state = state_with_signals(2);
        let first = state.displayed_items()[0].id();

        state.apply(ViewerCommand::SetFocusedAlias(Some(
            "  instruction  ".to_owned(),
        )));
        assert_eq!(state.displayed_items()[0].alias(), Some("instruction"));
        state.apply(ViewerCommand::SetDisplayedColor {
            id: first,
            color: DisplayColor::Cyan,
        });
        assert_eq!(state.displayed_items()[0].color(), DisplayColor::Cyan);

        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(state.displayed_items()[0].color(), DisplayColor::Default);
        assert_eq!(state.displayed_items()[0].alias(), Some("instruction"));
        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(state.displayed_items()[0].alias(), None);

        state.apply(ViewerCommand::RedoDisplayChange);
        state.apply(ViewerCommand::RedoDisplayChange);
        assert_eq!(state.displayed_items()[0].alias(), Some("instruction"));
        assert_eq!(state.displayed_items()[0].color(), DisplayColor::Cyan);
    }

    #[test]
    fn marks_jump_exactly_or_linewise_and_track_the_previous_position() {
        let mut state = state_with_signals(2);
        let first = state.displayed_items()[0].id();
        let second = state.displayed_items()[1].id();
        state.apply(ViewerCommand::SetCursor(20));
        state.apply(ViewerCommand::SetMark('a'));

        state.apply(ViewerCommand::SetFocusedItem(second));
        state.apply(ViewerCommand::SetCursor(80));
        assert_eq!(
            state.apply(ViewerCommand::JumpToMark {
                name: 'a',
                exact: true,
            }),
            vec![EffectRequest::RevealFocusedItem(FocusPlacement::Center)]
        );
        assert_eq!(state.cursor_state().focused_item(), Some(first));
        assert_eq!(state.cursor(), Some(20));

        state.apply(ViewerCommand::JumpToPrevious { exact: true });
        assert_eq!(state.cursor_state().focused_item(), Some(second));
        assert_eq!(state.cursor(), Some(80));

        state.apply(ViewerCommand::SetCursor(60));
        state.apply(ViewerCommand::JumpToMark {
            name: 'a',
            exact: false,
        });
        assert_eq!(state.cursor_state().focused_item(), Some(first));
        assert_eq!(state.cursor(), Some(60));
    }

    #[test]
    fn mark_listing_and_deletion_are_command_driven() {
        let mut state = state_with_signals(1);
        state.apply(ViewerCommand::SetCursor(12));
        state.apply(ViewerCommand::SetMark('b'));

        let effects = state.apply(ViewerCommand::RequestMarkList);
        assert!(matches!(
            effects.as_slice(),
            [EffectRequest::PromptText(text)] if text.contains("b    12")
        ));
        state.apply(ViewerCommand::DeleteMarks(vec!['b']));
        assert!(state.marks().is_empty());
    }

    #[test]
    fn jump_list_traverses_older_and_newer_positions() {
        let mut state = state_with_signals(3);
        let first = state.displayed_items()[0].id();
        let second = state.displayed_items()[1].id();
        let third = state.displayed_items()[2].id();

        state.apply(ViewerCommand::SetCursor(10));
        state.apply(ViewerCommand::SetMark('a'));
        state.apply(ViewerCommand::SetFocusedItem(second));
        state.apply(ViewerCommand::SetCursor(20));
        state.apply(ViewerCommand::SetMark('b'));
        state.apply(ViewerCommand::SetFocusedItem(third));
        state.apply(ViewerCommand::SetCursor(30));

        state.apply(ViewerCommand::JumpToMark {
            name: 'a',
            exact: true,
        });
        state.apply(ViewerCommand::JumpToMark {
            name: 'b',
            exact: true,
        });
        state.apply(ViewerCommand::TraverseJumpList(-1));
        assert_eq!(state.cursor_state().focused_item(), Some(first));
        assert_eq!(state.cursor(), Some(10));
        state.apply(ViewerCommand::TraverseJumpList(-1));
        assert_eq!(state.cursor_state().focused_item(), Some(third));
        assert_eq!(state.cursor(), Some(30));
        state.apply(ViewerCommand::TraverseJumpList(2));
        assert_eq!(state.cursor_state().focused_item(), Some(second));
        assert_eq!(state.cursor(), Some(20));
    }

    #[test]
    fn capture_start_and_end_are_jump_list_positions() {
        let mut state = state_with_signals(1);
        state.apply(ViewerCommand::SetCursor(50));
        state.apply(ViewerCommand::JumpToTime(0));
        state.apply(ViewerCommand::JumpToTime(100));

        state.apply(ViewerCommand::TraverseJumpList(-1));
        assert_eq!(state.cursor(), Some(0));
        state.apply(ViewerCommand::TraverseJumpList(-1));
        assert_eq!(state.cursor(), Some(50));
        state.apply(ViewerCommand::TraverseJumpList(2));
        assert_eq!(state.cursor(), Some(100));
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

    #[test]
    fn beginning_a_command_prompt_is_a_host_effect() {
        let mut viewer = ViewerState::default();
        assert_eq!(
            viewer.apply(ViewerCommand::BeginCommandPrompt),
            vec![EffectRequest::FocusCommandPrompt]
        );
    }

    #[test]
    fn focused_name_copy_and_row_placement_are_ui_effects() {
        let mut state = state_with_signals(2);

        assert_eq!(
            state.apply(ViewerCommand::CopyFocusedName),
            vec![EffectRequest::CopyText("signal_0".to_owned())]
        );
        assert_eq!(
            state.apply(ViewerCommand::RevealFocusedItem(FocusPlacement::Center)),
            vec![EffectRequest::RevealFocusedItem(FocusPlacement::Center)]
        );
        assert_eq!(
            state.apply(ViewerCommand::SelectVisibleRow {
                placement: FocusPlacement::Top,
                count: 2,
            }),
            vec![EffectRequest::SelectVisibleRow {
                placement: FocusPlacement::Top,
                count: 2,
            }]
        );
    }

    fn waveform_with_signals(count: usize) -> Waveform {
        Waveform::new(
            (0..count)
                .map(|index| (format!("signal_{index}"), crate::vcd::Signal::new(1)))
                .collect(),
            100,
        )
    }

    fn state_with_signals(count: usize) -> ViewerState {
        let mut state = ViewerState::with_waveform(waveform_with_signals(count));
        let signal_ids = state
            .waveform()
            .signals()
            .iter()
            .map(|signal| signal.id())
            .collect();
        state.apply(ViewerCommand::AddDisplayedSignals(signal_ids));
        state
    }

    #[test]
    fn displayed_order_accepts_only_complete_permutations() {
        let mut state = state_with_signals(3);
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
        let mut state = state_with_signals(2);
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
    fn relative_focus_commands_clamp_to_displayed_signals() {
        let mut state = state_with_signals(3);
        let first = state.displayed_items()[0].id();
        let last = state.displayed_items()[2].id();

        state.apply(ViewerCommand::FocusDisplayedRelative(20));
        assert_eq!(state.cursor_state().focused_item(), Some(last));
        state.apply(ViewerCommand::FocusDisplayedRelative(-20));
        assert_eq!(state.cursor_state().focused_item(), Some(first));
    }

    #[test]
    fn removing_a_middle_item_focuses_the_item_that_followed_it() {
        let mut state = state_with_signals(4);
        let removed = state.displayed_items()[1].id();
        let following = state.displayed_items()[2].id();
        state.apply(ViewerCommand::SetFocusedItem(removed));

        state.apply(ViewerCommand::RemoveDisplayedItem(removed));

        assert_eq!(state.cursor_state().focused_item(), Some(following));
    }

    #[test]
    fn removing_multiple_items_is_one_undoable_display_change() {
        let mut state = state_with_signals(4);
        let removed = [
            state.displayed_items()[1].id(),
            state.displayed_items()[2].id(),
        ];
        let following = state.displayed_items()[3].id();
        state.apply(ViewerCommand::SetFocusedItem(removed[0]));

        state.apply(ViewerCommand::RemoveDisplayedItems(removed.to_vec()));
        assert_eq!(state.displayed_items().len(), 2);
        assert_eq!(state.cursor_state().focused_item(), Some(following));

        state.apply(ViewerCommand::UndoDisplayChange);
        assert_eq!(state.displayed_items().len(), 4);
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
