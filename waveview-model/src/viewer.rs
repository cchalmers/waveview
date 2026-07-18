use serde::{Deserialize, Serialize};

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

/// Durable state for navigation shared by every input frontend.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct ViewerState {
    capture_end: u64,
    viewport: TimeViewport,
    cursor: Option<u64>,
}

impl ViewerState {
    pub fn new(capture_end: u64) -> Self {
        Self {
            capture_end,
            viewport: TimeViewport::fit(capture_end),
            cursor: None,
        }
    }

    pub fn capture_end(&self) -> u64 {
        self.capture_end
    }

    pub fn viewport(&self) -> TimeViewport {
        self.viewport
    }

    pub fn cursor(&self) -> Option<u64> {
        self.cursor
    }

    /// Apply one command and return host work requested by that command.
    pub fn apply(&mut self, command: ViewerCommand) -> Vec<EffectRequest> {
        match command {
            ViewerCommand::ReplaceCapture {
                end_time,
                preserve_view,
            } => {
                self.capture_end = end_time;
                if preserve_view {
                    self.viewport.clamp_to_capture(end_time);
                } else {
                    self.viewport = TimeViewport::fit(end_time);
                }
                self.cursor = self.cursor.map(|cursor| cursor.min(end_time));
            }
            ViewerCommand::FitTime => self.viewport = TimeViewport::fit(self.capture_end),
            ViewerCommand::PanTime(delta) => self.viewport.pan_by(delta, self.capture_end),
            ViewerCommand::ZoomTime { anchor, factor } => {
                self.viewport.zoom_at(anchor, factor, self.capture_end);
            }
            ViewerCommand::RevealTime(time) => {
                self.viewport.reveal(time as f64, self.capture_end);
            }
            ViewerCommand::SetCursor(time) => self.cursor = Some(time.min(self.capture_end)),
            ViewerCommand::ClearCursor => self.cursor = None,
            ViewerCommand::RequestOpenFile => return vec![EffectRequest::OpenFile],
            ViewerCommand::RequestOpenUrl(url) => return vec![EffectRequest::OpenUrl(url)],
            ViewerCommand::RequestLiveConnection(url) => {
                return vec![EffectRequest::ConnectLive(url)];
            }
        }

        Vec::new()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self::new(1)
    }
}

/// An operation requested by keyboard, mouse, menus, Tcl, or remote control.
#[derive(Clone, Debug, PartialEq)]
pub enum ViewerCommand {
    ReplaceCapture { end_time: u64, preserve_view: bool },
    FitTime,
    PanTime(f64),
    ZoomTime { anchor: f64, factor: f64 },
    RevealTime(u64),
    SetCursor(u64),
    ClearCursor,
    RequestOpenFile,
    RequestOpenUrl(String),
    RequestLiveConnection(String),
}

/// Imperative work performed by the stable application host rather than the reducer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectRequest {
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
    fn capture_replacement_clamps_view_and_cursor() {
        let mut state = ViewerState::new(100);
        state.apply(ViewerCommand::ZoomTime {
            anchor: 50.0,
            factor: 2.0,
        });
        state.apply(ViewerCommand::PanTime(50.0));
        state.apply(ViewerCommand::SetCursor(90));

        state.apply(ViewerCommand::ReplaceCapture {
            end_time: 60,
            preserve_view: true,
        });

        assert_eq!(state.viewport(), TimeViewport::new(10.0, 50.0, 60));
        assert_eq!(state.cursor(), Some(60));
    }

    #[test]
    fn capture_replacement_can_fit_and_reset_the_view() {
        let mut state = ViewerState::new(100);
        state.apply(ViewerCommand::ZoomTime {
            anchor: 50.0,
            factor: 4.0,
        });

        state.apply(ViewerCommand::ReplaceCapture {
            end_time: 250,
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
}
