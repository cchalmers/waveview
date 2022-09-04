use eframe::egui;
use egui::widgets::plot::Value;
use egui::*;
use std::ops::RangeInclusive;
// use std::ops::RangeInclusive;
use crate::vcd;

pub struct Wave<'a> {
    scale: f32,
    view_range: RangeInclusive<f32>,
    height: f32,
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
    pub fn new(
        name: &'a str,
        scale: f32,
        view_range: RangeInclusive<f32>,
        wave_data: &'a vcd::Signal,
    ) -> Self {
        Wave {
            scale,
            view_range,
            height: 32.0,
            wave_data,
            name,
        }
    }

    pub fn ui(self, ui: &mut Ui) {
        let Self {
            scale,
            view_range,
            height,
            wave_data,
            name,
        } = self;
        log::trace!("Wave::new({name})");

        let unscaled_unit_width = 32.0;

        // let width = range.end() - range.start();
        let total_wave_width = scale * wave_data.final_time() as f32;
        let (rect, _response) = ui.allocate_exact_size(
            vec2(total_wave_width * unscaled_unit_width, height),
            Sense::drag(),
        );
        // let response = response.on_hover_ui_at_pointer(|ui| {ui.add(egui::widgets::Label::new(name));});

        if wave_data.is_empty() {
            return;
        }
        let wave_painter = ui.painter().sub_region(rect);

        let show_background = true;
        if show_background {
            wave_painter.add(epaint::RectShape {
                rect,
                rounding: Rounding::same(2.0),
                fill: ui.visuals().extreme_bg_color,
                stroke: ui.visuals().widgets.noninteractive.bg_stroke,
            });
        }

        let mut wave_ui = ui.child_ui(rect, Layout::default());
        wave_ui.set_clip_rect(rect);
        let mut pts = vec![];
        // let mut last_high;
        // let dx = 1.0;
        // let dy = 0.9;
        let first_ix = (view_range.start() / 32.0 / scale).floor() as u64;
        let last_ix = (view_range.end() / 32.0 / scale).ceil() as u64;
        if last_ix <= first_ix {
            return;
        }
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
        pts.push(Value::new(x, y));

        for (t, v) in scalars {
            x = t as f32;
            pts.push(Value::new(x, y));
            if v == vcd::Value::V1 {
                y = 0.9
            } else {
                y = 0.1
            }
            pts.push(Value::new(x, y));
        }
        // pts.push(Value::new(x, dy));
        // for &h in &wave_data[std::cmp::min(first_data_ix + 1, wave_data.len() - 1)..std::cmp::min(last_data_ix + 1, wave_data.len() - 1)] {
        //     if h {
        //         if last_high {
        //             x += dx;
        //             pts.push(Value::new(x, dy));
        //         } else {
        //             pts.push(Value::new(x, dy));
        //             x += dx;
        //             pts.push(Value::new(x, dy));
        //         }
        //         last_high = true;
        //     } else {
        //         if last_high {
        //             pts.push(Value::new(x, 0.1));
        //             x += dx;
        //             pts.push(Value::new(x, 0.1));
        //         } else {
        //             x += dx;
        //             pts.push(Value::new(x, 0.1));
        //         }
        //         last_high = false;
        //     }
        // }
        // if last_high {
        //     pts.push(Value::new(last_view_ix as f32, dy));
        // } else {
        //     pts.push(Value::new(last_view_ix as f32, 0.1));
        // }

        fn pos_from_val(value: Value, rect: Rect, len: usize) -> egui::Pos2 {
            let x = remap(
                value.x as f32,
                // range,
                0.0..=(len as f32),
                rect.left()..=rect.right(),
                // 0.0..=(32.0),
            );
            let y = remap(
                value.y as f32,
                0.0..=1.0,
                rect.bottom()..=rect.top(), // negated y axis!
            );
            pos2(x as f32, y as f32)
        }

        let stroke = Stroke::new(2.0, Color32::from_additive_luminance(196));

        let mut shapes = vec![];
        shapes.push(Shape::line(
            pts.iter()
                .map(|v| pos_from_val(*v, rect, wave_data.final_time() as usize))
                .collect(),
            stroke,
        ));

        // if scale > 0.6 {
        //     let mut x = first_ix as f32 + 0.5;
        //     let mut prev = wave_data[first_ix];
        //     let mut prev_start_x = x;
        //     let last_x = last_ix as f32 - 2.0;
        //     for &h in &wave_data[first_ix+1..last_ix] {
        //         if h == prev && x < last_x {
        //             x += 1.0;
        //             continue;
        //         }

        //         let pos = pos_from_val(Value::new((prev_start_x + x)/2.0, 0.5), rect, wave_data.len());
        //         let txt = if prev {"0xde"} else {"0xbeef"};
        //         let anchor = Align2::CENTER_CENTER;
        //         // let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
        //         // let sty = TextStyle::Monospace;
        //         let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
        //         let color = Color32::from_additive_luminance(196);
        //         // let fill_color = Color32::from_additive_luminance(40);
        //         // let fill_color = if h { Color32::GREEN.linear_multiply(0.1) } else { Color32::RED.linear_multiply(0.1) };
        //         let fill_color = if prev {
        //             Color32::from(Rgba::GREEN.multiply(0.2) + Rgba::from_white_alpha(0.08))
        //         } else {
        //             // Color32::RED.linear_multiply(0.1)
        //             // Color32::from(Rgba::RED.multiply(0.2) + Rgba::from_white_alpha(0.08))
        //             Color32::from(Rgba::RED.multiply(0.3))
        //         };

        //         let galley = ui.fonts().layout_no_wrap(txt.to_string(), font, color);
        //         let rect = anchor.anchor_rect(Rect::from_min_size(pos, galley.size()));
        //         let fill_rect = rect.expand(2.0);
        //         if fill_rect.width() < scale * 32.0 {
        //             shapes.push(Shape::rect_filled(fill_rect, 2.0, fill_color));
        //             shapes.push(Shape::galley(rect.min, galley));
        //         }
        //         x += 1.0;
        //         prev = h;
        //         prev_start_x = x;
        //     }
        // }

        // ui.painter().sub_region(*transform.frame()).extend(shapes);
        ui.painter().extend(shapes);

        // response
    }
}
