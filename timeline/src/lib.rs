#![warn(clippy::all, rust_2018_idioms)]

use egui::{pos2, Align2, Color32, FontFamily, FontId, Rect, Response, Sense, Stroke, Ui, Vec2};

pub const DEFAULT_HEIGHT: f32 = 38.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

impl TimeRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            start: start.min(end),
            end: end.max(start),
        }
    }

    pub fn span(self) -> u64 {
        self.end.saturating_sub(self.start).max(1)
    }

    pub fn clamp_to(self, full: Self) -> Self {
        let span = self.span().min(full.span());
        let max_start = full.end.saturating_sub(span);
        let start = self.start.clamp(full.start, max_start);
        Self::new(start, start.saturating_add(span))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Pan(f64),
    Zoom { anchor: u64, factor: f64 },
    Fit,
    SetCursor(u64),
    BeginSelection(u64),
    UpdateSelection(u64),
    EndSelection,
}

pub struct TimelineResponse {
    pub response: Response,
    pub actions: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Marker {
    pub time: u64,
    pub label: char,
}

pub struct Timeline<'a> {
    visible: TimeRange,
    cursor: Option<u64>,
    selection: Option<TimeRange>,
    marks: &'a [Marker],
    formatter: &'a dyn Fn(u64) -> String,
    height: f32,
}

impl<'a> Timeline<'a> {
    pub fn new(full: TimeRange, visible: TimeRange, formatter: &'a dyn Fn(u64) -> String) -> Self {
        Self {
            visible: visible.clamp_to(full),
            cursor: None,
            selection: None,
            marks: &[],
            formatter,
            height: DEFAULT_HEIGHT,
        }
    }

    pub fn cursor(mut self, cursor: Option<u64>) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn selection(mut self, selection: Option<TimeRange>) -> Self {
        self.selection = selection;
        self
    }

    pub fn marks(mut self, marks: &'a [Marker]) -> Self {
        self.marks = marks;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut Ui) -> TimelineResponse {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), self.height),
            Sense::click_and_drag(),
        );
        let mut actions = Vec::new();
        paint_background(ui, rect);
        paint_ticks(ui, rect, self.visible, self.formatter);

        if let Some(selection) = self.selection {
            let left = x_from_time(rect, self.visible, selection.start);
            let right = x_from_time(rect, self.visible, selection.end);
            ui.painter().rect_filled(
                Rect::from_x_y_ranges(left..=right, rect.y_range()),
                0.0,
                Color32::LIGHT_BLUE.linear_multiply(0.12),
            );
        }
        for mark in self.marks {
            paint_marker(ui, rect, self.visible, mark.time, Color32::LIGHT_GREEN, 1.0);
        }
        if let Some(cursor) = self.cursor {
            paint_marker(ui, rect, self.visible, cursor, Color32::LIGHT_BLUE, 2.0);
        }
        for (index, mark) in self.marks.iter().enumerate() {
            let total = self
                .marks
                .iter()
                .filter(|candidate| candidate.time == mark.time)
                .count();
            let ordinal = self.marks[..index]
                .iter()
                .filter(|candidate| candidate.time == mark.time)
                .count();
            let offset = (ordinal as f32 - (total.saturating_sub(1)) as f32 * 0.5) * 15.0;
            paint_marker_label(ui, rect, self.visible, *mark, offset);
        }
        paint_readout(
            ui,
            rect,
            self.visible,
            self.cursor,
            self.selection,
            self.formatter,
        );

        if let Some(pointer) = response.interact_pointer_pos() {
            let time = time_from_x(rect, self.visible, pointer.x);
            if response.drag_started() {
                actions.push(Action::BeginSelection(time));
            } else if response.dragged() {
                actions.push(Action::UpdateSelection(time));
            } else if response.clicked() {
                actions.push(Action::SetCursor(time));
            }
        }
        if response.drag_stopped() {
            actions.push(Action::EndSelection);
        }
        if response.hovered() {
            let zoom = ui.input(|input| input.zoom_delta());
            if zoom != 1.0 {
                let anchor = response.hover_pos().map_or(self.visible.start, |pos| {
                    time_from_x(rect, self.visible, pos.x)
                });
                actions.push(Action::Zoom {
                    anchor,
                    factor: f64::from(zoom),
                });
            }
            let pan_points = ui.input(|input| input.smooth_scroll_delta.x);
            if pan_points != 0.0 {
                actions.push(Action::Pan(
                    -f64::from(pan_points) * self.visible.span() as f64 / f64::from(rect.width()),
                ));
            }
        }
        if response.double_clicked() {
            actions.push(Action::Fit);
        }

        TimelineResponse { response, actions }
    }
}

pub fn x_from_time(rect: Rect, visible: TimeRange, time: u64) -> f32 {
    let relative = time.saturating_sub(visible.start) as f64;
    let fraction = relative / visible.span() as f64;
    rect.left() + (fraction * f64::from(rect.width())) as f32
}

pub fn time_from_x(rect: Rect, visible: TimeRange, x: f32) -> u64 {
    let fraction = ((x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    visible
        .start
        .saturating_add((fraction * visible.span() as f64).round() as u64)
        .min(visible.end)
}

fn paint_background(ui: &Ui, rect: Rect) {
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );
}

fn paint_ticks(ui: &Ui, rect: Rect, visible: TimeRange, formatter: &dyn Fn(u64) -> String) {
    let count = (rect.width() / 96.0).floor().max(1.0) as u64;
    let raw_gap = (visible.span() / count).max(1);
    let gap = nice_gap(raw_gap);
    let mut time = visible.start / gap * gap;
    if time < visible.start {
        time = time.saturating_add(gap);
    }
    let font = FontId::new(11.0, FontFamily::Monospace);
    while time <= visible.end {
        let x = x_from_time(rect, visible, time);
        ui.painter().line_segment(
            [pos2(x, rect.bottom() - 6.0), pos2(x, rect.bottom())],
            Stroke::new(1.0, ui.visuals().text_color()),
        );
        ui.painter().text(
            pos2(x + 3.0, rect.bottom() - 2.0),
            Align2::LEFT_BOTTOM,
            formatter(time),
            font.clone(),
            ui.visuals().text_color(),
        );
        let Some(next) = time.checked_add(gap) else {
            break;
        };
        time = next;
    }
}

fn paint_marker(ui: &Ui, rect: Rect, visible: TimeRange, time: u64, color: Color32, width: f32) {
    if visible.start <= time && time <= visible.end {
        let x = x_from_time(rect, visible, time);
        ui.painter()
            .vline(x, rect.y_range(), Stroke::new(width, color));
    }
}

fn paint_marker_label(ui: &Ui, rect: Rect, visible: TimeRange, marker: Marker, offset: f32) {
    if !(visible.start <= marker.time && marker.time <= visible.end) {
        return;
    }
    let x = (x_from_time(rect, visible, marker.time) + offset)
        .clamp(rect.left() + 7.0, rect.right() - 7.0);
    let badge = Rect::from_center_size(pos2(x, rect.center().y), Vec2::splat(14.0));
    ui.painter()
        .rect_filled(badge, 2.0, ui.visuals().extreme_bg_color);
    ui.painter().rect_stroke(
        badge,
        2.0,
        Stroke::new(1.0, Color32::LIGHT_GREEN),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        badge.center(),
        Align2::CENTER_CENTER,
        marker.label,
        FontId::new(10.0, FontFamily::Monospace),
        Color32::LIGHT_GREEN,
    );
}

fn paint_readout(
    ui: &Ui,
    rect: Rect,
    visible: TimeRange,
    cursor: Option<u64>,
    selection: Option<TimeRange>,
    formatter: &dyn Fn(u64) -> String,
) {
    let font = FontId::new(12.0, FontFamily::Monospace);
    let color = Color32::LIGHT_BLUE;
    if let Some(selection) = selection {
        let duration = selection.end.saturating_sub(selection.start);
        let text = format!(
            "{} → {}   Δ {}",
            formatter(selection.start),
            formatter(selection.end),
            formatter(duration)
        );
        let center_time = selection.start.saturating_add(duration / 2);
        let x = x_from_time(rect, visible, center_time).clamp(rect.left(), rect.right());
        ui.painter().text(
            pos2(x, rect.top() + 3.0),
            Align2::CENTER_TOP,
            text,
            font,
            color,
        );
    } else if let Some(cursor) = cursor {
        let x = x_from_time(rect, visible, cursor).clamp(rect.left(), rect.right());
        let align = if x > rect.center().x {
            Align2::RIGHT_TOP
        } else {
            Align2::LEFT_TOP
        };
        let offset = if x > rect.center().x { -4.0 } else { 4.0 };
        ui.painter().text(
            pos2(x + offset, rect.top() + 3.0),
            align,
            formatter(cursor),
            font,
            color,
        );
    }
}

fn nice_gap(raw: u64) -> u64 {
    let magnitude = 10_u64.saturating_pow(raw.ilog10());
    let normalized = raw / magnitude;
    let step = match normalized {
        0 | 1 => 1,
        2..=4 => 5,
        _ => 10,
    };
    magnitude.saturating_mul(step).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_preserves_precision_near_u64_max() {
        let start = u64::MAX - 10_000;
        let visible = TimeRange::new(start, start + 1_000);
        let rect = Rect::from_min_size(pos2(20.0, 0.0), Vec2::new(1_000.0, 20.0));

        assert_eq!(x_from_time(rect, visible, start + 1), 21.0);
        assert_eq!(time_from_x(rect, visible, 21.0), start + 1);
        assert_eq!(time_from_x(rect, visible, 520.0), start + 500);
    }

    #[test]
    fn range_clamping_preserves_span_at_capture_end() {
        assert_eq!(
            TimeRange::new(80, 120).clamp_to(TimeRange::new(0, 100)),
            TimeRange::new(60, 100)
        );
    }

    #[test]
    fn gaps_use_readable_one_five_ten_steps() {
        assert_eq!(nice_gap(1), 1);
        assert_eq!(nice_gap(2), 5);
        assert_eq!(nice_gap(49), 50);
        assert_eq!(nice_gap(51), 100);
    }
}
