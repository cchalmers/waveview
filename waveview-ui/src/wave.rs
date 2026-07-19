use eframe::egui;
use egui::*;
use egui_plot::PlotPoint;
use std::ops::RangeInclusive;
// use std::ops::RangeInclusive;
use waveview_model::vcd;

pub struct Wave<'a> {
    view_range: RangeInclusive<f32>,
    pub height: f32,
    name: &'a str,
    // wave_data: &'a [bool],
    wave_data: &'a vcd::Signal,
}

// fn signal_points(signal: &crate::vcd::Signal, range: RangeInclusive<f32>) -> Vec<Value> {
//     let mut pts = vec![];
//     let start = range.start().floor() as u64;
//     let end = range.end().ceil() as u64;
//     let dy = 0.9;
//     let mut last_high = false;
//     for (&t, val) in signal.range(start..end) {
//         let t = t as f32;
//         if val[0] == vcd::Value::V1 {
//             if last_high {
//                 pts.push(Value::new(t, dy));
//             } else {
//                 pts.push(Value::new(t, dy));
//                 pts.push(Value::new(t, dy));
//             }
//             last_high = true;
//         } else if val[0] == vcd::Value::V0 {
//             if last_high {
//                 pts.push(Value::new(t, 0.1));
//                 pts.push(Value::new(t, 0.1));
//             } else {
//                 pts.push(Value::new(t, 0.1));
//             }
//             last_high = false;
//         }
//     }
//     pts
// }

impl<'a> Wave<'a> {
    // pub fn new(name: &'a str, scale: f32, view_range: RangeInclusive<f32>, wave_data: &'a [bool]) -> Self {
    pub fn new(name: &'a str, view_range: RangeInclusive<f32>, wave_data: &'a vcd::Signal) -> Self {
        Wave {
            view_range,
            height: 32.0,
            wave_data,
            name,
        }
    }

    pub fn ui(self, ui: &mut Ui) {
        let Self {
            view_range,
            height,
            wave_data,
            name,
        } = self;
        log::trace!("Wave::new({name})");

        let (rect, _response) = ui.allocate_exact_size(
            vec2(ui.available_width(), height),
            // Sense::hover()
            Sense::focusable_noninteractive(),
        );
        // let _response = response.on_hover_ui_at_pointer(|ui| {
        //     ui.add(egui::widgets::Label::new(name));
        // });

        let wave_painter = ui.painter().with_clip_rect(rect);

        if wave_data.is_empty() {
            return;
        }

        // let mut last_high;
        // let dx = 1.0;
        // let dy = 0.9;
        let first_ix = view_range.start().floor().max(0.0) as u64;
        let last_ix = view_range.end().ceil().max(first_ix as f32 + 1.0) as u64;
        if last_ix <= first_ix {
            return;
        }
        // TODO
        //
        // - undefined values should be visible
        // - the last signal at the end of the simulation should be visible (currently it gets cut
        //   off)
        if wave_data.width() == 1 {
            let mut pts = vec![];
            let mut scalars = wave_data.bit_range(first_ix..last_ix).into_iter();
            let (t0, v0) = scalars.next().unwrap();
            let mut x = t0 as f32;
            let mut y;
            // let last_data_ix = std::cmp::min(wave_data.final_time(), last_view_ix);
            // let mut x = t as f32;
            if v0 == vcd::Value::V1 {
                // last_high = true;
                y = 0.9;
                // pts.push(Value::new(x, dy));
                // x += dx;
                // pts.push(Value::new(x, dy));
            } else {
                // last_high = false;
                y = 0.1;
                // x += dx;
                // pts.push(Value::new(x, 0.1));
            }
            pts.push(PlotPoint::new(x, y));

            for (t, v) in scalars {
                x = t as f32;
                pts.push(PlotPoint::new(x, y));
                if v == vcd::Value::V1 {
                    y = 0.9
                } else {
                    y = 0.1
                }
                pts.push(PlotPoint::new(x, y));
            }
            // pts.push(PlotPoint::new(x, dy));
            // for &h in &wave_data[std::cmp::min(first_data_ix + 1, wave_data.len() - 1)..std::cmp::min(last_data_ix + 1, wave_data.len() - 1)] {
            //     if h {
            //         if last_high {
            //             x += dx;
            //             pts.push(PlotPoint::new(x, dy));
            //         } else {
            //             pts.push(PlotPoint::new(x, dy));
            //             x += dx;
            //             pts.push(PlotPoint::new(x, dy));
            //         }
            //         last_high = true;
            //     } else {
            //         if last_high {
            //             pts.push(PlotPoint::new(x, 0.1));
            //             x += dx;
            //             pts.push(PlotPoint::new(x, 0.1));
            //         } else {
            //             x += dx;
            //             pts.push(PlotPoint::new(x, 0.1));
            //         }
            //         last_high = false;
            //     }
            // }
            // if last_high {
            //     pts.push(PlotPoint::new(last_view_ix as f32, dy));
            // } else {
            //     pts.push(PlotPoint::new(last_view_ix as f32, 0.1));
            // }

            fn pos_from_val(
                value: PlotPoint,
                rect: Rect,
                view_range: &RangeInclusive<f32>,
            ) -> egui::Pos2 {
                let x = remap(
                    value.x as f32,
                    view_range.clone(),
                    rect.left()..=rect.right(),
                );
                let y = remap(
                    value.y as f32,
                    0.0..=1.0,
                    rect.bottom()..=rect.top(), // negated y axis!
                );
                pos2(x as f32, y as f32)
            }

            let stroke = ui.style().visuals.widgets.active.bg_stroke;

            let shapes = vec![Shape::line(
                pts.iter()
                    .map(|v| pos_from_val(*v, rect, &view_range))
                    .collect(),
                stroke,
            )];
            wave_painter.extend(shapes);
        } else {
            fn pos_from_val(
                value: PlotPoint,
                rect: Rect,
                view_range: &RangeInclusive<f32>,
            ) -> egui::Pos2 {
                let x = remap(
                    value.x as f32,
                    view_range.clone(),
                    rect.left()..=rect.right(),
                );
                let y = remap(
                    value.y as f32,
                    0.0..=1.0,
                    rect.bottom()..=rect.top(), // negated y axis!
                );
                pos2(x as f32, y as f32)
            }

            let stroke = ui.style().visuals.widgets.active.bg_stroke;
            let mut shapes = vec![Shape::line_segment(
                [
                    pos_from_val(PlotPoint::new(*view_range.start(), 0.5), rect, &view_range),
                    pos_from_val(PlotPoint::new(*view_range.end(), 0.5), rect, &view_range),
                ],
                stroke,
            )];
            for (time, _) in wave_data.range(first_ix.saturating_add(1)..last_ix) {
                let time = time as f32;
                shapes.push(Shape::line_segment(
                    [
                        pos_from_val(PlotPoint::new(time, 0.25), rect, &view_range),
                        pos_from_val(PlotPoint::new(time, 0.75), rect, &view_range),
                    ],
                    stroke,
                ));
            }
            let mut add_value_label = |start: f32, end: f32, value: &[vcd::Value]| {
                let start_pos = pos_from_val(PlotPoint::new(start, 0.5), rect, &view_range);
                let end_pos = pos_from_val(PlotPoint::new(end, 0.5), rect, &view_range);
                let available_width = (end_pos.x - start_pos.x).abs();
                let text = format_vector_value(value);
                let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
                let color = ui.visuals().text_color();
                let mut galley =
                    ui.fonts_mut(|fonts| fonts.layout_no_wrap(text, font.clone(), color));
                let mut label_color = color;
                let mut horizontal_padding = 4.0;
                if galley.size().x + horizontal_padding * 2.0 > available_width {
                    label_color = ui.visuals().weak_text_color();
                    horizontal_padding = 2.0;
                    galley = ui
                        .fonts_mut(|fonts| fonts.layout_no_wrap("…".to_owned(), font, label_color));
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
            };

            let mut previous = &wave_data[first_ix];
            let mut segment_start = (*view_range.start()).max(first_ix as f32);
            for (time, value) in wave_data.range(first_ix.saturating_add(1)..last_ix) {
                let segment_end = time as f32;
                add_value_label(segment_start, segment_end, previous);
                previous = value;
                segment_start = segment_end;
            }
            add_value_label(
                segment_start,
                (*view_range.end()).min(last_ix as f32),
                previous,
            );
            wave_painter.extend(shapes);
        }
    }
}

fn format_vector_value(value: &[vcd::Value]) -> String {
    let padding = (4 - value.len() % 4) % 4;
    let mut digits = String::with_capacity(value.len().div_ceil(4));
    for nibble in std::iter::repeat_n(vcd::Value::V0, padding)
        .chain(value.iter().copied())
        .collect::<Vec<_>>()
        .chunks_exact(4)
    {
        let digit = if nibble.contains(&vcd::Value::X) {
            'x'
        } else if nibble.contains(&vcd::Value::Z) {
            'z'
        } else {
            let number = nibble.iter().fold(0_u8, |number, bit| {
                (number << 1) | u8::from(*bit == vcd::Value::V1)
            });
            char::from_digit(u32::from(number), 16).unwrap()
        };
        digits.push(digit);
    }
    format!("0x{digits}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_values_are_compact_and_preserve_unknowns() {
        assert_eq!(
            format_vector_value(&[
                vcd::Value::V1,
                vcd::Value::V0,
                vcd::Value::V1,
                vcd::Value::V0,
            ]),
            "0xa"
        );
        assert_eq!(
            format_vector_value(&[
                vcd::Value::X,
                vcd::Value::X,
                vcd::Value::X,
                vcd::Value::X,
                vcd::Value::Z,
                vcd::Value::Z,
                vcd::Value::Z,
                vcd::Value::Z,
            ]),
            "0xxz"
        );
    }
}
