use eframe::{egui, epi};
use eframe::egui::NumExt;
use crate::wave;
use crate::vcd;

pub struct TemplateApp {
    // wave_data: Vec<(String, Vec<bool>)>,
    wave_data: Vec<(String, vcd::Signal)>,
    x_scale: f32,
    final_time: u64,
    x_offset: Option<f32>,
    y_offset: f32,
    main_viewport: egui::Rect,
}

impl TemplateApp {
    pub fn new(sigs: Vec<(vcd::ScopedVar, vcd::Signal)>, final_time: u64) -> TemplateApp {
        let wave_data = sigs.into_iter().map(|(var, sig)| {
            let mut name: String = itertools::intersperse(var.scopes.iter().map(|x| x.1.as_str()), ".").collect();
            if !name.is_empty() {
                name.push_str(".");
            }
            name.push_str(&var.var.reference);
            // let bools = sig.scalars().map(|(_, v)| v == vcd::Value::V1).collect();
            // eprintln!("bools = {bools:?}");
            (name, sig)
        }).collect();
        Self {
            wave_data,
            final_time,
            x_scale: 3.0,
            x_offset: None,
            y_offset: 0.0,
            main_viewport: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 800.0)),
        }
    }
}

// impl Default for TemplateApp {
//     fn default() -> Self {
//         let num_rows = 1000;
//         let mut wave_data = vec![];
//         let mut rng = rand::thread_rng();
//         for row in 0..num_rows {
//             let mut dat = vec![true];
//             for _ in 0..NUM_CYCLES {
//                 dat.push(rng.gen());
//             }
//             wave_data.push((format!("wave-{}", row), dat));
//         }
//         Self {
//             wave_data,
//             x_scale: 3.0,
//             x_offset: None,
//         }
//     }
// }

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // #[cfg(feature = "persistence")]
        // if let Some(storage) = _storage {
        //     *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        // }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        // let Self { label, value } = self;
        let Self { wave_data, final_time, x_scale, x_offset, y_offset, main_viewport } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        // let main_viewport = std::rc::Rc::new(std::cell::Cell::new(None));
        // let mut main_viewport = None;

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.set_width(180.0);
            let max_rect = ui.max_rect();
            // max_rect.max.x += 100.0;

            let row_height_sans_spacing = 32.0;
            let spacing = ui.spacing().item_spacing;
            let row_height_with_spacing = row_height_sans_spacing + spacing.y;

            use egui::*;

            let viewport = Rect::from_min_size(egui::pos2(8.0, 16.0 - *y_offset), egui::vec2(180.0, 800.0));

            // eprintln!("viewport = {viewport:?}");
            let mut ui = ui.child_ui(viewport, *ui.layout());
            // let viewport = main_viewport.unwrap();
            let mut content_clip_rect = max_rect.expand(ui.visuals().clip_rect_margin);
            // add clipping for the "timeline" bar
            content_clip_rect.min.y += 25.0;
            ui.set_clip_rect(content_clip_rect);
            let num_rows = wave_data.len();
            ui.set_height((row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0));
            // let min_row = (viewport.min.y / row_height_with_spacing)
            let min_row = (*y_offset / row_height_with_spacing)
                .floor()
                .at_least(0.0) as usize;
            // let max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
            let max_row = ((*y_offset + max_rect.size().y) / row_height_with_spacing).ceil() as usize + 1;
            let max_row = max_row.at_most(num_rows);

            ui.set_height((row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0));
            let max_row = max_row.at_most(num_rows);

            let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
            let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

            let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=y_max);

            ui.allocate_ui_at_rect(rect, |ui| {
                ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
                // ui.vertical(|ui| {
                // ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.scope(|ui| ui.set_height(16.0 + 24.0));
                    for i in min_row..max_row {
                        ui.scope(|ui| {
                            // ui.set_height(32.0 - 12.0);
                            ui.set_height(32.0);
                            ui.label(&wave_data[i].0);
                        });
                    }
                });

            })
            .inner
            // use egui::*;
            // let mut shapes = vec![];
            // let color = Color32::from_additive_luminance(196);
            // // ui.allocate_ui_at_rect(rect, |ui| {
            // //     ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
            // //     ui.vertical(|ui| {
            //         for i in min_row..max_row {
            // //             ui.group(|ui| {
            //                 let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
            //                 let txt = &wave_data[i].0;
            //                 let galley = ui.fonts().layout_no_wrap(txt.to_string(), font, color);
            //                 let pos = pos2(5.0, i as f32 * 32.0);
            //                 let anchor = Align2::CENTER_CENTER;
            //                 let rect = anchor.anchor_rect(Rect::from_min_size(pos, galley.size()));
            //                 shapes.push(Shape::galley(rect.min, galley));
            //                 // ui.set_height(32.0 - 12.0);
            //                 // ui.label(&wave_data[i].0);
            //             // });
            //         }
            //     // });
            // // })
            // // .inner;
            // ui.painter().extend(shapes);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("eframe template");

            let clip_rect = ui.clip_rect();
            let min_rect = ui.min_rect();
            let max_rect = ui.max_rect();

            let scroll_area = egui::ScrollArea::both()
                .auto_shrink([false; 2]);

            let scroll_area = if let Some(offset) = x_offset {
                scroll_area.horizontal_scroll_offset(*offset)
            } else {
                scroll_area
            };

            let num_rows = wave_data.len();

            let row_height_sans_spacing = 32.0;
            let spacing = ui.spacing().item_spacing;
            let row_height_with_spacing = row_height_sans_spacing + spacing.y;

            scroll_area.show_viewport(ui, |ui, viewport| {
                // this is kinda nasty because you end up with a 1 frame lag between the waves and
                // the labels. Maybe having 2 separate scroll areas would one? One hoirzontal only
                // for the wave and a vertical only for waves and labels? I feel like I tried this
                // and it didn't work out properly.
                *y_offset = viewport.min.y;
                *main_viewport = viewport;
                ui.set_height((row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0));
                let min_row = (viewport.min.y / row_height_with_spacing)
                    .floor()
                    .at_least(0.0) as usize;
                let max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
                let max_row = max_row.at_most(num_rows);

                let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
                let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

                let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=16.0);
                let v = egui::vec2(ui.max_rect().width(), 16.0);
                ui.allocate_ui(v, |ui| {
                    ui.set_height(16.0);
                    let x_min = (main_viewport.min.x / 32.0 / *x_scale).floor() as usize;
                    let x_max = (main_viewport.max.x / 32.0 / *x_scale).ceil() as usize;
                    let mut ticks = vec![];
                    let stroke = egui::Stroke::new(1.0, egui::Color32::YELLOW);
                    let num_ticks = std::cmp::max(1, (main_viewport.width() / 64.0).floor() as usize);
                    let gap = std::cmp::max(1, (main_viewport.width() / 32.0 / *x_scale / num_ticks as f32).round() as usize);
                    let mut i = (x_min + gap - 1) / gap * gap;
                    while i <= x_max { // in x_min..=x_max {
                        let p0 = egui::pos2(rect.min.x + *x_scale * 32.0 * i as f32, max_rect.min.y + 4.0);
                        let p1 = egui::pos2(rect.min.x + *x_scale * 32.0 * i as f32, max_rect.min.y + 10.0);
                        ticks.push(egui::Shape::line_segment([p0, p1], stroke));

                        use egui::*;
                        let anchor = Align2::LEFT_CENTER;
                        let font = epaint::text::FontId::new(12.0, text::FontFamily::Monospace);
                        let color = Color32::from_additive_luminance(196);

                        let galley = ui.fonts().layout_no_wrap(i.to_string(), font, color);
                        let rect = anchor.anchor_rect(Rect::from_min_size(p0 + vec2(4.0, 0.0), galley.size()));
                        ticks.push(Shape::galley(rect.min, galley));
                        i += gap;
                    }
                    ui.painter().extend(ticks);
                });

                let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min+16.0..=y_max);

                ui.allocate_ui_at_rect(rect, |ui| {
                    // let mut max_rect = ui.max_rect();
            // let mut content_clip_rect = max_rect.expand(ui.visuals().clip_rect_margin);
            // // add clipping for the "timeline" bar
            // content_clip_rect.
            // ui.set_clip_rect(content_clip_rect);
                    let mut clip_rect = ui.clip_rect();
                    clip_rect.min.y += 16.0;
                    ui.set_clip_rect(clip_rect);
                    ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
                    let resp = ui.interact(egui::Rect::EVERYTHING, egui::Id::new("ui_hover"), egui::Sense::drag());
                    let hover_pos = resp.hover_pos();

                    let x_frac = hover_pos.map(|hover_pos| (hover_pos.x - min_rect.min.x) / min_rect.width());
                    let x_val = x_frac.map(|x_frac| (viewport.min.x + x_frac * (viewport.max.x - viewport.min.x)) / 32.0 / *x_scale);
                    ui.vertical(|ui| {
                        for i in min_row..max_row {
                            let name = format!("{}
hover_pos: {hover_pos:?}
clip_rect: {clip_rect:?}
min_rect: {min_rect:?}
max_rect: {max_rect:?},
viewport: {viewport:?}
x_frac: {x_frac:?}
x_val: {x_val:?}
x_scale: {x_scale:?}",
    wave_data[i].0);
                            let wave = wave::Wave::new(&name, *x_scale, viewport.min.x..=viewport.max.x, &wave_data[i].1);
                            // wave.ui(ui, &ctx.fonts());
                            wave.ui(ui);
                        }
                    });

                    if let Some(pos) = &hover_pos {
                        use egui::*;
                        let mut shapes = vec![];
                        // let color = Color32::from_additive_luminance(196);
                        let p0 = pos2(pos.x, 0.0);
                        let p1 = pos2(pos.x, 800.0);
                        let stroke = Stroke::new(1.0, Color32::YELLOW);
                        shapes.push(Shape::line_segment([p0, p1], stroke));
                        ui.painter().extend(shapes);
                    }


                    let view_width = viewport.max.x - viewport.min.x;

                    // do the actual zoom on the next frame where we know the scroll position that
                    // will be needed
                    if ui.rect_contains_pointer(egui::Rect::EVERYTHING) {
                        let zoom = ui.input().zoom_delta();
                        if zoom != 1.0 {
                            *x_scale *= zoom;
                            if *x_scale < 0.001 {
                                *x_scale = 0.001;
                            } else if *x_scale > 100.0 {
                                *x_scale = 100.0;
                            } else {
                                if let Some(x_frac) = x_frac {
                                    let offset = zoom * viewport.min.x + (zoom - 1.0) * x_frac * view_width;
                                    // the scoll area doesn't like it a negative offset or a
                                    // positive offset when there's nothing to scroll
                                    if offset < 0.0 || (*final_time as f32) * *x_scale * 32.0 < view_width {
                                        *x_offset = Some(0.0);
                                    } else {
                                        *x_offset = Some(offset);
                                    }
                                }
                            }
                        } else {
                            // only set the offset correction when we're zooming, otherwise the
                            // scroller deal with the position
                            *x_offset = None;
                        }
                    }

                })
                .inner;
            });

        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
