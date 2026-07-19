use eframe::egui;
use egui::*;
use std::ops::{Range, RangeInclusive};
use waveview_model::vcd;
use waveview_model::viewer::{DisplayColor, ValueFormat};

pub struct Wave<'a> {
    view_range: RangeInclusive<f32>,
    pub height: f32,
    name: &'a str,
    wave_data: &'a vcd::Signal,
    value_format: ValueFormat,
    color: DisplayColor,
}

impl<'a> Wave<'a> {
    pub fn new(
        name: &'a str,
        view_range: RangeInclusive<f32>,
        wave_data: &'a vcd::Signal,
        value_format: ValueFormat,
        color: DisplayColor,
    ) -> Self {
        Self {
            view_range,
            height: 32.0,
            wave_data,
            name,
            value_format,
            color,
        }
    }

    pub fn ui(self, ui: &mut Ui) {
        let Self {
            view_range,
            height,
            wave_data,
            name,
            value_format,
            color,
        } = self;
        log::trace!("Wave::new({name})");

        let (rect, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), height),
            Sense::focusable_noninteractive(),
        );
        if wave_data.is_empty() {
            return;
        }

        let first_time = view_range.start().floor().max(0.0) as u64;
        let last_time = view_range.end().ceil().max(first_time as f32 + 1.0) as u64;
        if last_time <= first_time {
            return;
        }

        let mut shapes = Vec::new();
        let default_stroke = ui.visuals().widgets.active.bg_stroke;
        let style = WaveStyle {
            stroke: Stroke::new(
                default_stroke.width,
                crate::display_color::resolve(color, default_stroke.color),
            ),
            value_format,
        };
        if wave_data.width() == 1 {
            render_scalar(
                ui,
                &mut shapes,
                wave_data,
                first_time..last_time,
                rect,
                &view_range,
                style,
            );
        } else {
            render_vector(
                ui,
                &mut shapes,
                wave_data,
                first_time..last_time,
                rect,
                &view_range,
                style,
            );
        }
        ui.painter().with_clip_rect(rect).extend(shapes);
    }
}

#[derive(Clone, Copy)]
struct WaveStyle {
    stroke: Stroke,
    value_format: ValueFormat,
}

fn render_scalar(
    ui: &Ui,
    shapes: &mut Vec<Shape>,
    signal: &vcd::Signal,
    time_range: Range<u64>,
    rect: Rect,
    view_range: &RangeInclusive<f32>,
    style: WaveStyle,
) {
    let first_time = time_range.start;
    let last_time = time_range.end;
    let Some(&initial) = signal.value_at(first_time).and_then(|value| value.first()) else {
        return;
    };
    let stroke = style.stroke;
    let mut previous = initial;
    let mut segment_start = (*view_range.start()).max(first_time as f32);

    for (time, value) in signal.range(first_time.saturating_add(1)..last_time) {
        if time <= first_time {
            continue;
        }
        if time >= last_time || time as f32 >= *view_range.end() {
            break;
        }
        let segment_end = time as f32;
        add_scalar_segment(
            ui,
            shapes,
            segment_start..segment_end,
            previous,
            rect,
            view_range,
            stroke,
        );
        let next = value[0];
        if is_defined(previous) && is_defined(next) {
            let x = wave_pos(time as f32, 0.5, rect, view_range).x;
            shapes.push(Shape::line_segment(
                [
                    pos2(x, scalar_y(previous, rect)),
                    pos2(x, scalar_y(next, rect)),
                ],
                stroke,
            ));
        }
        previous = next;
        segment_start = segment_end;
    }

    add_scalar_segment(
        ui,
        shapes,
        segment_start..*view_range.end(),
        previous,
        rect,
        view_range,
        stroke,
    );
}

fn add_scalar_segment(
    ui: &Ui,
    shapes: &mut Vec<Shape>,
    segment: Range<f32>,
    value: vcd::Value,
    rect: Rect,
    view_range: &RangeInclusive<f32>,
    stroke: Stroke,
) {
    let left = wave_pos(segment.start, 0.5, rect, view_range).x;
    let right = wave_pos(segment.end, 0.5, rect, view_range).x;
    if right <= left {
        return;
    }
    match value {
        vcd::Value::V0 | vcd::Value::V1 => shapes.push(Shape::line_segment(
            [
                pos2(left, scalar_y(value, rect)),
                pos2(right, scalar_y(value, rect)),
            ],
            stroke,
        )),
        vcd::Value::X => add_unknown_horizontal(
            shapes,
            Rect::from_min_max(
                pos2(left, scalar_y(vcd::Value::V1, rect)),
                pos2(right, scalar_y(vcd::Value::V0, rect)),
            ),
            unknown_color(ui),
        ),
        vcd::Value::Z => add_dashed_horizontal(
            shapes,
            left,
            right,
            rect.center().y,
            ui.visuals().weak_text_color(),
            ui.visuals().extreme_bg_color,
        ),
    }
}

fn render_vector(
    ui: &Ui,
    shapes: &mut Vec<Shape>,
    signal: &vcd::Signal,
    time_range: Range<u64>,
    rect: Rect,
    view_range: &RangeInclusive<f32>,
    style: WaveStyle,
) {
    let first_time = time_range.start;
    let last_time = time_range.end;
    let stroke = style.stroke;
    for (time, _) in signal.range(first_time.saturating_add(1)..last_time) {
        if time <= first_time || time >= last_time {
            continue;
        }
        let time = time as f32;
        shapes.push(Shape::line_segment(
            [
                wave_pos(time, 0.25, rect, view_range),
                wave_pos(time, 0.75, rect, view_range),
            ],
            stroke,
        ));
    }

    let Some(mut previous) = signal.value_at(first_time) else {
        return;
    };
    let mut segment_start = (*view_range.start()).max(first_time as f32);
    for (time, value) in signal.range(first_time.saturating_add(1)..last_time) {
        if time <= first_time {
            continue;
        }
        if time >= last_time || time as f32 >= *view_range.end() {
            break;
        }
        let segment_end = time as f32;
        add_vector_segment(
            ui,
            shapes,
            segment_start..segment_end,
            previous,
            rect,
            view_range,
            style,
        );
        previous = value;
        segment_start = segment_end;
    }
    add_vector_segment(
        ui,
        shapes,
        segment_start..*view_range.end(),
        previous,
        rect,
        view_range,
        style,
    );
}

fn add_vector_segment(
    ui: &Ui,
    shapes: &mut Vec<Shape>,
    segment: Range<f32>,
    value: &[vcd::Value],
    rect: Rect,
    view_range: &RangeInclusive<f32>,
    style: WaveStyle,
) {
    let start_pos = wave_pos(segment.start, 0.5, rect, view_range);
    let end_pos = wave_pos(segment.end, 0.5, rect, view_range);
    let available_width = (end_pos.x - start_pos.x).max(0.0);
    if available_width <= 0.0 {
        return;
    }

    if value.contains(&vcd::Value::X) {
        add_unknown_horizontal(
            shapes,
            Rect::from_min_max(
                pos2(
                    start_pos.x,
                    wave_pos(segment.start, 0.75, rect, view_range).y,
                ),
                pos2(end_pos.x, wave_pos(segment.end, 0.25, rect, view_range).y),
            ),
            unknown_color(ui),
        );
    } else if value.contains(&vcd::Value::Z) {
        add_dashed_horizontal(
            shapes,
            start_pos.x,
            end_pos.x,
            rect.center().y,
            ui.visuals().weak_text_color(),
            ui.visuals().extreme_bg_color,
        );
    } else {
        shapes.push(Shape::line_segment([start_pos, end_pos], style.stroke));
    }

    let text = crate::value::format(value, style.value_format);
    let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
    let color = style.stroke.color;
    let mut galley = ui.fonts_mut(|fonts| fonts.layout_no_wrap(text, font.clone(), color));
    let mut label_color = color;
    let mut horizontal_padding = 4.0;
    if galley.size().x + horizontal_padding * 2.0 > available_width {
        label_color = ui.visuals().weak_text_color();
        horizontal_padding = 2.0;
        galley = ui.fonts_mut(|fonts| fonts.layout_no_wrap("…".to_owned(), font, label_color));
    }
    let text_rect = Align2::CENTER_CENTER.anchor_size(
        pos2((start_pos.x + end_pos.x) * 0.5, rect.center().y),
        galley.size(),
    );
    let fill_rect = text_rect.expand2(vec2(horizontal_padding, 2.0));
    if fill_rect.width() <= available_width {
        shapes.push(Shape::rect_filled(
            fill_rect,
            CornerRadius::same(2),
            ui.visuals().extreme_bg_color,
        ));
        shapes.push(Shape::galley(text_rect.min, galley, label_color));
    }
}

fn add_unknown_horizontal(shapes: &mut Vec<Shape>, rect: Rect, color: Color32) {
    if rect.width() <= 0.0 {
        return;
    }
    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 16);
    let line = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 170);
    shapes.push(Shape::rect_filled(rect, CornerRadius::ZERO, fill));
    shapes.push(Shape::line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(1.25, line),
    ));
}

fn unknown_color(ui: &Ui) -> Color32 {
    let error = ui.visuals().error_fg_color;
    let weak = ui.visuals().weak_text_color();
    Color32::from_rgb(
        ((u16::from(error.r()) + u16::from(weak.r()) * 2) / 3) as u8,
        ((u16::from(error.g()) + u16::from(weak.g()) * 2) / 3) as u8,
        ((u16::from(error.b()) + u16::from(weak.b()) * 2) / 3) as u8,
    )
}

fn is_defined(value: vcd::Value) -> bool {
    matches!(value, vcd::Value::V0 | vcd::Value::V1)
}

fn add_dashed_horizontal(
    shapes: &mut Vec<Shape>,
    left: f32,
    right: f32,
    y: f32,
    color: Color32,
    background: Color32,
) {
    shapes.push(Shape::line_segment(
        [pos2(left, y), pos2(right, y)],
        Stroke::new(3.0, background),
    ));
    let mut dash_start = left;
    while dash_start < right {
        let dash_end = (dash_start + 6.0).min(right);
        shapes.push(Shape::line_segment(
            [pos2(dash_start, y), pos2(dash_end, y)],
            Stroke::new(1.0, color),
        ));
        dash_start += 10.0;
    }
}

fn scalar_y(value: vcd::Value, rect: Rect) -> f32 {
    let level = match value {
        vcd::Value::V0 => 0.1,
        vcd::Value::V1 => 0.9,
        vcd::Value::X | vcd::Value::Z => 0.5,
    };
    remap(level, 0.0..=1.0, rect.bottom()..=rect.top())
}

fn wave_pos(time: f32, level: f32, rect: Rect, view_range: &RangeInclusive<f32>) -> Pos2 {
    pos2(
        remap(time, view_range.clone(), rect.left()..=rect.right()),
        remap(level, 0.0..=1.0, rect.bottom()..=rect.top()),
    )
}
