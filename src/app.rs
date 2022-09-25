use crate::vcd;
use crate::wave;
use eframe::egui;
use eframe::egui::NumExt;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use std::future::Future;
use std::task::Poll;

pub struct TemplateApp {
    // wave_data: Vec<(String, Vec<bool>)>,
    wave_data: Vec<(String, vcd::Signal)>,
    x_scale: Option<f32>,
    final_time: u64,
    x_offset: Option<f32>,
    y_offset: f32,
    dropped_files: Vec<egui::DroppedFile>,
    main_viewport: egui::Rect,
    // a_future: Option<std::pin::Pin<Box<dyn Future<Output = Option<rfd::FileHandle>>>>>,
    a_future: Option<std::pin::Pin<Box<dyn Future<Output = Option<OpenedVcd>>>>>,
    open_file_ctx: Option<OpenFileCtx>,
    download: Arc<Mutex<Download>>,
    url_window: UrlWindow,
}

impl TemplateApp {
    pub fn new(sigs: Vec<(vcd::ScopedVar, vcd::Signal)>, final_time: u64) -> TemplateApp {
        let wave_data = sigs
            .into_iter()
            .map(|(var, sig)| {
                let mut name: String =
                    itertools::intersperse(var.scopes.iter().map(|x| x.1.as_str()), ".").collect();
                if !name.is_empty() {
                    name.push('.');
                }
                name.push_str(&var.var.reference);
                // let bools = sig.scalars().map(|(_, v)| v == vcd::Value::V1).collect();
                // eprintln!("bools = {bools:?}");
                (name, sig)
            })
            .collect();
        Self {
            wave_data,
            final_time,
            x_scale: None, // 3.0,
            x_offset: None,
            y_offset: 0.0,
            dropped_files: vec![],
            main_viewport: egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(100.0, 800.0),
            ),
            a_future: None,
            open_file_ctx: None,
            download: Arc::new(Mutex::new(Download::None)),
            url_window: UrlWindow {
                url: "https://raw.githubusercontent.com/emilk/ehttp/master/README.md".to_owned(),
                open: false,
            },
        }
    }
}

enum Download {
    None,
    InProgress,
    Done(ehttp::Result<ehttp::Response>),
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

// Custom waker
//
// egui doesn't have native support for futures but something simple like opening a file it's easy
// enough to make one that triggers a redraw on wake. It assumes the app is always alive so it
// doesn't have to deal with reference counting.

const RAW_WAKER_VTABLE: std::task::RawWakerVTable =
    std::task::RawWakerVTable::new(my_clone, my_wake_by_ref, my_wake_by_ref, my_drop);

struct OpenedVcd {
    // filename: String,
    wave_data: Vec<(String, vcd::Signal)>,
    time: u64,
}

struct OpenFileCtx {
    awoken: Arc<AtomicBool>,
    egui_ctx: egui::Context,
}

unsafe fn my_clone(ctx: *const ()) -> std::task::RawWaker {
    std::task::RawWaker::new(ctx, &RAW_WAKER_VTABLE)
}

unsafe fn my_wake_by_ref(ctx: *const ()) {
    let ctx: &OpenFileCtx = &*(ctx as *const OpenFileCtx);
    ctx.awoken.store(true, std::sync::atomic::Ordering::Release);
    ctx.egui_ctx.request_repaint();
}

unsafe fn my_drop(_: *const ()) {}

fn new_waker(ctx: &OpenFileCtx) -> std::task::RawWaker {
    std::task::RawWaker::new(ctx as *const OpenFileCtx as *const (), &RAW_WAKER_VTABLE)
}

struct UrlWindow {
    url: String,
    open: bool,
}

impl UrlWindow {
    fn show(&mut self, ctx: &egui::Context, download: &Arc<Mutex<Download>>) {
        if self.open {
            let window = egui::Window::new("Open URL")
                .id(egui::Id::new("open_url"))
                .resizable(false)
                .collapsible(false)
                .title_bar(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
            window.show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("url:");
                    ui.text_edit_singleline(&mut self.url);
                });
                ui.horizontal(|ui| {
                    if ui.button("fetch").clicked() {
                        let request = ehttp::Request::get(&self.url);
                        let dl = download.clone();
                        *dl.lock().unwrap() = Download::InProgress;
                        let ctx2 = ctx.clone();
                        ehttp::fetch(request, move |response| {
                            *dl.lock().unwrap() = Download::Done(response);
                            ctx2.request_repaint();
                        });
                        ctx.request_repaint();
                        self.open = false;
                    }
                });
            });
        }
    }
}

impl eframe::App for TemplateApp {
    // fn name(&self) -> &str {
    //     "eframe template"
    // }

    // /// Called once before the first frame.
    // fn setup(
    //     &mut self,
    //     _ctx: &egui::Context,
    //     _frame: &epi::Frame,
    //     _storage: Option<&dyn epi::Storage>,
    // ) {
    //     // #[cfg(feature = "persistence")]
    //     // if let Some(storage) = _storage {
    //     //     *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    //     // }
    // }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // let Self { label, value } = self;
        let Self {
            wave_data,
            final_time,
            x_scale,
            x_offset,
            y_offset,
            dropped_files: _,
            main_viewport,
            a_future,
            open_file_ctx,
            download,
            url_window,
        } = self;

        {
            let mut dl = download.lock().unwrap();
            match &*dl {
                Download::None => (),
                Download::InProgress => eprintln!("in progress"),
                Download::Done(Err(res)) => {
                    tracing::event!(tracing::Level::ERROR, "error: {res}");
                    *dl = Download::None;
                }
                Download::Done(Ok(res)) => {
                    tracing::event!(
                        tracing::Level::INFO,
                        "response: url = {}, status = {}, headers = {:?}",
                        res.url,
                        res.status,
                        res.headers
                    );
                    let bytes = &res.bytes;
                    let mut cursor = std::io::Cursor::new(bytes);
                    let (signals, time) = vcd::read_clocked_vcd(&mut cursor).unwrap();
                    *wave_data = mk_wave_data(signals);
                    *final_time = time;
                    *dl = Download::None;
                }
            }
        }

        if let Some(future) = a_future {
            if open_file_ctx.is_none() {
                let awoken = Arc::new(AtomicBool::new(false));
                *open_file_ctx = Some(OpenFileCtx {
                    awoken,
                    egui_ctx: ctx.clone(),
                });
            }
            let waker =
                unsafe { std::task::Waker::from_raw(new_waker(open_file_ctx.as_ref().unwrap())) };
            let mut my_ctx = std::task::Context::from_waker(&waker);
            match Future::poll(future.as_mut(), &mut my_ctx) {
                Poll::Pending => (),
                Poll::Ready(shandle) => {
                    match shandle {
                        Some(handle) => {
                            *wave_data = handle.wave_data;
                            *final_time = handle.time;
                        }
                        None => (),
                    }
                    *a_future = None;
                    *open_file_ctx = None;
                }
            }
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File…").clicked() {
                        *a_future = Some(Box::pin(async {
                            let handle = rfd::AsyncFileDialog::new().pick_file().await;
                            if let Some(h) = &handle {
                                let bytes = h.read().await;
                                let mut cursor = std::io::Cursor::new(&bytes);
                                let (signals, time) = vcd::read_clocked_vcd(&mut cursor).unwrap();
                                let wave_data = mk_wave_data(signals);
                                Some(OpenedVcd {
                                    // filename: h.file_name(),
                                    wave_data,
                                    time,
                                })
                            } else {
                                None
                            }
                        }));
                        ui.close_menu();
                        ctx.request_repaint();
                    }
                    if ui.button("Open URL…").clicked() {
                        url_window.open = true;
                        ui.close_menu();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                    // annoying warning
                    #[cfg(target_arch = "wasm32")]
                    let _ = &frame;
                });
            });
        });

        url_window.show(ctx, download);

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

            let viewport =
                Rect::from_min_size(egui::pos2(8.0, 16.0 - *y_offset), egui::vec2(180.0, 800.0));

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
            let min_row = (*y_offset / row_height_with_spacing).floor().at_least(0.0) as usize;
            // let max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
            let max_row =
                ((*y_offset + max_rect.size().y) / row_height_with_spacing).ceil() as usize + 1;
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
                    for d in wave_data.iter().take(max_row).skip(min_row) {
                        ui.scope(|ui| {
                            // ui.set_height(32.0 - 12.0);
                            ui.set_height(32.0);
                            ui.label(&d.0);
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

            // let clip_rect = ui.clip_rect();
            let min_rect = ui.min_rect();
            let max_rect = ui.max_rect();

            let scroll_area = egui::ScrollArea::both().auto_shrink([false; 2]);

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
                ui.set_height(
                    (row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0),
                );
                let min_row = (viewport.min.y / row_height_with_spacing)
                    .floor()
                    .at_least(0.0) as usize;
                let max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
                let max_row = max_row.at_most(num_rows);

                let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
                let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

                let rect =
                    egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min + 16.0..=y_max);

                // hackily make initial scale to fit everything (0.95 to handle scroll bar (major
                // hack))
                if x_scale.is_none() {
                    *x_scale = Some(
                        0.95 * (viewport.max.x - viewport.min.x) / (*final_time as f32 * 32.0),
                    );
                }
                let x_scale = x_scale.as_mut().unwrap();

                let hover_pos = ui
                    .allocate_ui_at_rect(rect, |ui| {
                        // let mut max_rect = ui.max_rect();
                        // let mut content_clip_rect = max_rect.expand(ui.visuals().clip_rect_margin);
                        // // add clipping for the "timeline" bar
                        // content_clip_rect.
                        // ui.set_clip_rect(content_clip_rect);
                        let mut clip_rect = ui.clip_rect();
                        clip_rect.min.y += 16.0;
                        ui.set_clip_rect(clip_rect);
                        ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
                        let resp = ui.interact(
                            egui::Rect::EVERYTHING,
                            egui::Id::new("ui_hover"),
                            egui::Sense::drag(),
                        );
                        let hover_pos = resp.hover_pos();

                        let x_frac = hover_pos
                            .map(|hover_pos| (hover_pos.x - min_rect.min.x) / min_rect.width());
                        let x_val = x_frac.map(|x_frac| {
                            (viewport.min.x + x_frac * (viewport.max.x - viewport.min.x))
                                / 32.0
                                / *x_scale
                        });
                        ui.vertical(|ui| {
                            for d in wave_data.iter().take(max_row).skip(min_row) {
                                let name = format!(
                                    "{}
hover_pos: {hover_pos:?}
clip_rect: {clip_rect:?}
min_rect: {min_rect:?}
max_rect: {max_rect:?},
viewport: {viewport:?}
x_frac: {x_frac:?}
x_val: {x_val:?}
x_scale: {x_scale:?}",
                                    d.0
                                );
                                let wave = wave::Wave::new(
                                    &name,
                                    *x_scale,
                                    viewport.min.x..=viewport.max.x,
                                    &d.1,
                                );
                                // wave.ui(ui, &ctx.fonts());
                                wave.ui(ui);
                            }
                        });

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
                                } else if let Some(x_frac) = x_frac {
                                    let offset =
                                        zoom * viewport.min.x + (zoom - 1.0) * x_frac * view_width;
                                    // the scoll area doesn't like it a negative offset or a
                                    // positive offset when there's nothing to scroll
                                    if offset < 0.0
                                        || (*final_time as f32) * *x_scale * 32.0 < view_width
                                    {
                                        *x_offset = Some(0.0);
                                    } else {
                                        *x_offset = Some(offset);
                                    }
                                }
                            } else {
                                // only set the offset correction when we're zooming, otherwise the
                                // scroller deal with the position
                                *x_offset = None;
                            }
                        }
                        hover_pos
                    })
                    .inner;

                let mut hover_t = None;
                if let Some(pos) = &hover_pos {
                    use egui::*;
                    let mut shapes = vec![];
                    // let color = Color32::from_additive_luminance(196);

                    let x = pos.x;
                    let t = (x - rect.min.x) / 32.0 / *x_scale;
                    let t_rounded = t.round();
                    hover_t = Some(t_rounded as usize);

                    let rounded_x = rect.min.x + t_rounded * *x_scale * 32.0;
                    let p0 = pos2(rounded_x, max_rect.min.y + 0.0);
                    let p1 = pos2(rounded_x, max_rect.max.y);
                    let stroke = Stroke::new(1.0, Color32::YELLOW);
                    shapes.push(Shape::line_segment([p0, p1], stroke));
                    ui.painter().extend(shapes);
                }

                let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=16.0);
                let x_min = (main_viewport.min.x / 32.0 / *x_scale).floor() as usize;
                let x_max = (main_viewport.max.x / 32.0 / *x_scale).ceil() as usize;
                let mut ticks = vec![];
                let stroke = egui::Stroke::new(1.0, egui::Color32::YELLOW);
                let num_ticks = std::cmp::max(1, (main_viewport.width() / 64.0).floor() as usize);
                let gap = std::cmp::max(
                    1,
                    (main_viewport.width() / 32.0 / *x_scale / num_ticks as f32).round() as usize,
                );
                // render the previous tick because part of it is still visible
                let mut i = (std::cmp::max(1, x_min) - 1) / gap * gap;
                while i <= x_max {
                    // in x_min..=x_max {
                    let mut used_i = i;
                    let mut highlight = false;
                    if let Some(t) = hover_t {
                        // kinda ugly since we'll render twice if midway
                        if i.abs_diff(t) <= gap / 2 {
                            used_i = t;
                            highlight = true;
                        }
                    }
                    let p0 = egui::pos2(
                        rect.min.x + *x_scale * 32.0 * used_i as f32,
                        max_rect.min.y + 4.0,
                    );
                    let p1 = egui::pos2(
                        rect.min.x + *x_scale * 32.0 * used_i as f32,
                        max_rect.min.y + 10.0,
                    );
                    ticks.push(egui::Shape::line_segment([p0, p1], stroke));

                    use egui::*;
                    let anchor = Align2::LEFT_CENTER;
                    let font_size = if highlight { 13.0 } else { 12.0 };
                    let font = epaint::text::FontId::new(font_size, text::FontFamily::Monospace);
                    let mut color = Color32::from_additive_luminance(196);
                    if highlight {
                        color = Color32::from_additive_luminance(255);
                    }

                    let galley = ui.fonts().layout_no_wrap(used_i.to_string(), font, color);
                    let rect =
                        anchor.anchor_rect(Rect::from_min_size(p0 + vec2(4.0, 0.0), galley.size()));
                    ticks.push(Shape::galley(rect.min, galley));
                    i += gap;
                }
                ui.painter().extend(ticks);
            });
        });

        self.ui_file_drag_and_drop(ctx);

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

fn mk_wave_data(sigs: Vec<(vcd::ScopedVar, vcd::Signal)>) -> Vec<(String, vcd::Signal)> {
    sigs.into_iter()
        .map(|(var, sig)| {
            let mut name: String =
                itertools::intersperse(var.scopes.iter().map(|x| x.1.as_str()), ".").collect();
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&var.var.reference);
            // let bools = sig.scalars().map(|(_, v)| v == vcd::Value::V1).collect();
            // eprintln!("bools = {bools:?}");
            (name, sig)
        })
        .collect()
}

impl TemplateApp {
    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    text += &format!("\n{}", path.display());
                } else if !file.mime.is_empty() {
                    text += &format!("\n{}", file.mime);
                } else {
                    text += "\n???";
                }
            }

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }

        // Show dropped files (if any):
        if !self.dropped_files.is_empty() {
            if let Some(path) = &self.dropped_files[0].path {
                let mut file = std::fs::File::open(path).unwrap();
                let (sigs, time) = vcd::read_clocked_vcd(&mut file).unwrap();
                self.final_time = time;
                self.wave_data = mk_wave_data(sigs);
            } else if let Some(bytes) = &self.dropped_files[0].bytes {
                let mut cursor = std::io::Cursor::new(&bytes);
                let (sigs, time) = vcd::read_clocked_vcd(&mut cursor).unwrap();
                self.final_time = time;
                self.wave_data = mk_wave_data(sigs);
            }
        }
    }
}
