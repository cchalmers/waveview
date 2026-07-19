use crate::vcd;
use crate::wave_dispatch;
use eframe::egui;
use eframe::egui::NumExt;
use egui::*;
use waveview_model::search::{SearchHistory, SearchMatcher};
use waveview_model::viewer::{
    DisplayedItem, EffectRequest, FocusPlacement, ViewerCommand, ViewerState,
};
use waveview_model::vim::{VimInput, VimState, NORMAL_BINDINGS};
use waveview_model::waveform::Waveform;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use std::future::Future;
use std::task::Poll;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    viewer: ViewerState,
    y_offset: f32,
    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,
    // a_future: Option<std::pin::Pin<Box<dyn Future<Output = Option<rfd::FileHandle>>>>>,
    #[serde(skip)]
    a_future: Option<std::pin::Pin<Box<dyn Future<Output = Option<OpenedVcd>>>>>,
    #[serde(skip)]
    open_file_ctx: Option<OpenFileCtx>,
    #[serde(skip)]
    pub download: Arc<Mutex<Download>>,
    #[serde(skip)]
    url_window: UrlWindow,
    #[serde(skip)]
    err_window: ErrWindow,
    #[serde(skip)]
    live: crate::live::LiveVcd,
    row_height: f32,
    side_panel: SidePanel,
    info: Info,
    #[serde(skip)]
    vim: VimState,
    #[serde(skip)]
    show_key_help: bool,
    #[serde(skip)]
    status_message: Option<String>,
    #[serde(skip)]
    status_expires_at: f64,
    search_history: SearchHistory,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            viewer: ViewerState::default(),
            y_offset: 0.0,
            dropped_files: vec![],
            a_future: None,
            open_file_ctx: None,
            download: Arc::new(Mutex::new(Download::None)),
            url_window: UrlWindow {
                url: "".to_owned(),
                open: false,
            },
            err_window: ErrWindow {
                msg: String::new(),
                open: false,
            },
            live: crate::live::LiveVcd::default(),

            row_height: 32.0,

            side_panel: if cfg!(debug_assertions) {
                SidePanel::Samples
            } else {
                SidePanel::None
            },
            info: Info {
                rect: Rect::NOTHING,
                min_rect: Rect::NOTHING,
                max_rect: Rect::NOTHING,
                viewport: Rect::NOTHING,
                pixels_per_tick: 0.0,
            },
            vim: VimState::default(),
            show_key_help: false,
            status_message: None,
            status_expires_at: 0.0,
            search_history: SearchHistory::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
enum SidePanel {
    None,
    Info,
    Samples,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Info {
    rect: Rect,
    min_rect: Rect,
    max_rect: Rect,
    viewport: Rect,
    pixels_per_tick: f32,
}

impl Info {
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let Self {
            rect,
            min_rect,
            max_rect,
            viewport,
            pixels_per_tick,
        } = self;
        ui.label(RichText::new("rect").strong());
        ui.monospace(format!("{:+04?}", rect.min));
        ui.monospace(format!("{:+04?}", rect.max));
        ui.separator();
        ui.label(RichText::new("min_rect").strong());
        ui.monospace(format!("{:?}", min_rect.min));
        ui.monospace(format!("{:?}", min_rect.max));
        ui.separator();
        ui.label(RichText::new("max_rect").strong());
        ui.monospace(format!("{:?}", max_rect.min));
        ui.monospace(format!("{:?}", max_rect.max));
        ui.separator();
        ui.label(RichText::new("viewport").strong());
        ui.monospace(format!("{:?}", viewport.min));
        ui.monospace(format!("{:?}", viewport.max));
        ui.separator();
        ui.label(RichText::new("pixels_per_tick").strong());
        ui.monospace(format!("{:?}", pixels_per_tick));
        ui.separator();
        ui.monospace(RichText::new("mouse_pos").strong());
        if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            ui.label(format!("{:?}", mouse_pos));
        } else {
            ui.label("None");
        }
        ui.separator();
        ui.label(RichText::new("scroll_delta").strong());
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
        let scroll_x = scroll_delta.x;
        let scroll_y = scroll_delta.y;
        ui.monospace(format!("{scroll_x:+03} {scroll_y:+03}"));
        ui.separator();
        ui.label(RichText::new("content_rect").strong());
        ui.label(format!("{:?}", ctx.content_rect()));
        ui.separator();
    }
}

impl TemplateApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        sigs: Vec<(vcd::ScopedVar, vcd::Signal)>,
        final_time: u64,
    ) -> TemplateApp {
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value(storage, eframe::APP_KEY) {
                return app;
            }
        }

        Self {
            viewer: ViewerState::with_waveform(mk_waveform(sigs, final_time)),
            y_offset: 0.0,
            dropped_files: vec![],
            a_future: None,
            open_file_ctx: None,
            download: Arc::new(Mutex::new(Download::None)),
            url_window: UrlWindow {
                url: "https://raw.githubusercontent.com/Mohammad-Heydariii/Digital-Systems-Lab-Course/main/Lab_project4/modelsim_files/clkdiv2n_tb.vcd".to_owned(),
                open: false,
            },
            err_window: ErrWindow { msg: String::new(), open: false },
            live: crate::live::LiveVcd::default(),

            row_height: 32.0,

            side_panel: if cfg!(debug_assertions) { SidePanel::Samples } else { SidePanel::None },
            info: Info { rect: Rect::NOTHING, min_rect: Rect::NOTHING, max_rect: Rect::NOTHING, viewport: Rect::NOTHING, pixels_per_tick: 0.0 },
            vim: VimState::default(),
            show_key_help: false,
            status_message: None,
            status_expires_at: 0.0,
            search_history: SearchHistory::default(),
        }
    }
}

pub enum Download {
    None,
    InProgress,
    Done(ehttp::Result<ehttp::Response>),
}

// Custom waker
//
// egui doesn't have native support for futures but something simple like opening a file it's easy
// enough to make one that triggers a redraw on wake. It assumes the app is always alive so it
// doesn't have to deal with reference counting.

const RAW_WAKER_VTABLE: std::task::RawWakerVTable =
    std::task::RawWakerVTable::new(my_clone, my_wake_by_ref, my_wake_by_ref, my_drop);

struct OpenedVcd {
    waveform: Waveform,
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

struct ErrWindow {
    msg: String,
    open: bool,
}

impl ErrWindow {
    fn show(&mut self, ctx: &egui::Context) {
        let window = egui::Window::new("Error")
            .id(egui::Id::new("error_window"))
            .resizable(true)
            .collapsible(false)
            .title_bar(true)
            .open(&mut self.open);
        // .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.msg);
            });
        });
    }
}

struct UrlWindow {
    url: String,
    open: bool,
}

impl UrlWindow {
    fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut requested_url = None;
        if self.open {
            let window = egui::Window::new("Open URL")
                .id(egui::Id::new("open_url"))
                .resizable(true)
                .collapsible(false)
                .title_bar(true)
                .open(&mut self.open)
                .default_size(egui::vec2(600.0, 100.0))
                .default_pos(egui::pos2(0.0, 0.0));
            let mut close = false;
            window.show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.label("url:");
                egui::TextEdit::singleline(&mut self.url)
                    .desired_width(ui.available_width())
                    .show(ui);
                ui.horizontal(|ui| {
                    if ui.button("fetch").clicked() {
                        requested_url = Some(self.url.clone());
                        close = true;
                    }
                });
            });
            if close {
                self.open = false;
            }
        }
        requested_url
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eprintln!("save");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();
        let Self {
            viewer,
            y_offset,
            dropped_files: _,
            a_future,
            open_file_ctx,
            download,
            url_window,
            err_window,
            live,
            row_height,
            side_panel,
            info,
            vim,
            show_key_help,
            status_message,
            status_expires_at,
            search_history,
        } = self;
        let now = ctx.input(|input| input.time);
        if status_message.is_some() && now >= *status_expires_at {
            *status_message = None;
        }

        {
            let mut dl = download.lock().unwrap();
            match &*dl {
                Download::None => (),
                Download::InProgress => eprintln!("in progress"),
                Download::Done(Err(err)) => {
                    tracing::event!(tracing::Level::ERROR, "error: {err}");
                    err_window.msg = format!("url download failed:\n{err}");
                    err_window.open = true;
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

                    if res.status == 200 {
                        match vcd::read_clocked_vcd(&mut cursor) {
                            Ok((signals, time)) => {
                                viewer.apply(ViewerCommand::ReplaceWaveform {
                                    waveform: mk_waveform(signals, time),
                                    preserve_view: false,
                                });
                            }
                            Err(err) => {
                                err_window.msg =
                                    format!("url {} failed to parse as vcd:\n{err}", res.url);
                                err_window.open = true;
                            }
                        }
                    } else {
                        err_window.msg =
                            format!("url {} fetch gave status:\n{}", res.url, res.status);
                        err_window.open = true;
                    }
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
                    if let Some(handle) = shandle {
                        viewer.apply(ViewerCommand::ReplaceWaveform {
                            waveform: handle.waveform,
                            preserve_view: false,
                        });
                    }
                    *a_future = None;
                    *open_file_ctx = None;
                }
            }
        }

        let search_has_focus = ctx.memory(|memory| memory.has_focus(signal_search_id()));
        let keyboard_captured =
            search_has_focus || url_window.open || err_window.open || live.open || *show_key_help;
        let search_inputs = take_search_inputs(&ctx, search_has_focus);
        let mut reveal_keyboard_focus = false;
        let mut keyboard_half_page_scroll = 0_isize;
        let mut keyboard_focus_placement = None;
        let mut keyboard_select_visible = None;
        let mut force_vertical_scroll = false;
        for input in search_inputs {
            match input {
                SearchInput::Previous => {
                    if let Some(query) = search_history.previous(viewer.search()) {
                        viewer.apply(ViewerCommand::SetSearch(query));
                        if focus_first_search_match(viewer) {
                            reveal_keyboard_focus = true;
                            force_vertical_scroll = true;
                        }
                    }
                }
                SearchInput::Next => {
                    if let Some(query) = search_history.newer() {
                        viewer.apply(ViewerCommand::SetSearch(query));
                        if focus_first_search_match(viewer) {
                            reveal_keyboard_focus = true;
                            force_vertical_scroll = true;
                        }
                    }
                }
                SearchInput::Accept => {
                    search_history.accept(viewer.search());
                    if let Some(focused) = ctx.memory(|memory| memory.focused()) {
                        ctx.memory_mut(|memory| memory.surrender_focus(focused));
                    }
                }
            }
        }
        for input in take_vim_inputs(&ctx, keyboard_captured) {
            if input == VimInput::Escape {
                if *show_key_help {
                    *show_key_help = false;
                } else if url_window.open {
                    url_window.open = false;
                } else if err_window.open {
                    err_window.open = false;
                } else if live.open {
                    live.open = false;
                } else if search_has_focus {
                    viewer.apply(ViewerCommand::SetSearch(String::new()));
                }
                if let Some(focused) = ctx.memory(|memory| memory.focused()) {
                    ctx.memory_mut(|memory| memory.surrender_focus(focused));
                }
            }
            for command in vim.handle(input, keyboard_captured, viewer) {
                reveal_keyboard_focus |= matches!(
                    command,
                    ViewerCommand::SetFocusedItem(_)
                        | ViewerCommand::MoveDisplayedItem { .. }
                        | ViewerCommand::RemoveDisplayedItem(_)
                );
                for effect in viewer.apply(command) {
                    match effect {
                        EffectRequest::ScrollDisplayedRows(rows) => {
                            let row_span = *row_height + root_ui.spacing().item_spacing.y;
                            *y_offset = (*y_offset + rows as f32 * row_span).max(0.0);
                            force_vertical_scroll = true;
                        }
                        EffectRequest::ScrollDisplayedHalfPages(half_pages) => {
                            keyboard_half_page_scroll += half_pages;
                            force_vertical_scroll = true;
                        }
                        EffectRequest::FocusSearch => {
                            search_history.reset_navigation();
                            ctx.memory_mut(|memory| memory.request_focus(signal_search_id()));
                        }
                        EffectRequest::RevealFocusedItem(placement) => {
                            keyboard_focus_placement = Some(placement);
                            force_vertical_scroll = true;
                        }
                        EffectRequest::SelectVisibleRow { placement, count } => {
                            keyboard_select_visible = Some((placement, count));
                        }
                        EffectRequest::CopyText(text) => {
                            ctx.copy_text(text.clone());
                            *status_message = Some(format!("copied {text}"));
                            *status_expires_at = now + 2.0;
                            ctx.request_repaint();
                        }
                        EffectRequest::OpenFile
                        | EffectRequest::OpenUrl(_)
                        | EffectRequest::ConnectLive(_) => {}
                    }
                }
            }
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(root_ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                ui.separator();
                ui.menu_button("File", |ui| {
                    if ui.button("Open File…").clicked() {
                        for effect in viewer.apply(ViewerCommand::RequestOpenFile) {
                            if effect == EffectRequest::OpenFile {
                                *a_future = Some(Box::pin(async {
                                    let handle = rfd::AsyncFileDialog::new().pick_file().await;
                                    if let Some(h) = &handle {
                                        let bytes = h.read().await;
                                        let mut cursor = std::io::Cursor::new(&bytes);
                                        let (signals, time) =
                                            vcd::read_clocked_vcd(&mut cursor).unwrap();
                                        Some(OpenedVcd {
                                            waveform: mk_waveform(signals, time),
                                        })
                                    } else {
                                        None
                                    }
                                }));
                            }
                        }
                        ui.close();
                        ctx.request_repaint();
                    }
                    if ui.button("Open URL…").clicked() {
                        url_window.open = true;
                        ui.close();
                    }
                    if ui.button("Connect live…").clicked() {
                        live.open = true;
                        ui.close();
                    }
                    if ui.button("Reset").clicked() {
                        viewer.apply(ViewerCommand::ReplaceWaveform {
                            waveform: Waveform::empty(),
                            preserve_view: false,
                        });
                        *y_offset = 0.0;
                        ui.close();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo display change").clicked() {
                        viewer.apply(ViewerCommand::UndoDisplayChange);
                        ui.close();
                    }
                    if ui.button("Redo display change").clicked() {
                        viewer.apply(ViewerCommand::RedoDisplayChange);
                        ui.close();
                    }
                    if let Some(id) = viewer.cursor_state().focused_item() {
                        if ui.button("Remove selected signal").clicked() {
                            viewer.apply(ViewerCommand::RemoveDisplayedItem(id));
                            ui.close();
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Fit time").clicked() {
                        viewer.apply(ViewerCommand::FitTime);
                        ui.close();
                    }
                    ui.separator();
                    ui.add(egui::Slider::new(row_height, 25.0..=128.0).text("height"));
                    // if *show_info {
                    //     if ui.button("Hide info").clicked() {
                    //         *show_info = false;
                    //         ui.close_menu();
                    //     }
                    // } else if ui.button("Show info").clicked() {
                    //     *show_info = true;
                    //     ui.close_menu();
                    // }
                    match side_panel {
                        SidePanel::None => {
                            if ui.button("Show info").clicked() {
                                *side_panel = SidePanel::Info;
                                ui.close();
                            }
                            if ui.button("Show samples").clicked() {
                                *side_panel = SidePanel::Samples;
                                ui.close();
                            }
                        }
                        SidePanel::Info => {
                            if ui.button("Hide info").clicked() {
                                *side_panel = SidePanel::None;
                                ui.close();
                            }
                            if ui.button("Show samples").clicked() {
                                *side_panel = SidePanel::Samples;
                                ui.close();
                            }
                        }
                        SidePanel::Samples => {
                            if ui.button("Show info").clicked() {
                                *side_panel = SidePanel::Info;
                                ui.close();
                            }
                            if ui.button("Hide samples").clicked() {
                                *side_panel = SidePanel::None;
                                ui.close();
                            }
                        }
                    }

                    // ui.button("
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Keyboard shortcuts").clicked() {
                        *show_key_help = true;
                        ui.close();
                    }
                });
            });
        });

        egui::Window::new("Keyboard shortcuts")
            .open(show_key_help)
            .resizable(true)
            .show(&ctx, |ui| {
                egui::Grid::new("vim_key_help")
                    .num_columns(2)
                    .spacing(egui::vec2(24.0, 6.0))
                    .striped(true)
                    .show(ui, |ui| {
                        for binding in NORMAL_BINDINGS {
                            ui.monospace(binding.keys);
                            ui.label(binding.description);
                            ui.end_row();
                        }
                    });
            });

        // if *show_info {
        //     egui::SidePanel::right("inspection_panel").show(ctx, |ui| {
        //         let scroll_area = egui::ScrollArea::both().auto_shrink([false; 2]);
        //         scroll_area.show(ui, |ui| info.show(ctx, ui));
        //     });
        // }
        match side_panel {
            SidePanel::None => (),
            SidePanel::Info => {
                egui::Panel::right("inspection_panel").show(root_ui, |ui| {
                    let scroll_area = egui::ScrollArea::both().auto_shrink([false; 2]);
                    scroll_area.show(ui, |ui| info.show(&ctx, ui));
                });
            }
            SidePanel::Samples => {
                egui::Panel::right("inspection_panel").show(root_ui, |ui| {
                    let scroll_area = egui::ScrollArea::both().auto_shrink([false; 2]);
                    let resp = scroll_area.show(ui, crate::samples::show_samples);
                    if let Some(url) = resp.inner {
                        eprintln!("url: {}", url);
                        let request = ehttp::Request::get(url);
                        let dl = download.clone();
                        *dl.lock().unwrap() = Download::InProgress;
                        let ctx2 = ctx.clone();
                        ehttp::fetch(request, move |response| {
                            *dl.lock().unwrap() = Download::Done(response);
                            ctx2.request_repaint();
                        });
                        ctx.request_repaint();
                    }
                });
            }
        }

        let mut search_text = viewer.search().to_owned();
        let original_order: Vec<_> = viewer
            .displayed_items()
            .iter()
            .map(DisplayedItem::id)
            .collect();
        let mut displayed_items = viewer.displayed_items().to_vec();
        let mut requested_focus = None;
        let timeline_height = wave_dispatch::timeline_height();

        egui::Panel::bottom("vim_status").show(root_ui, |ui| {
            ui.horizontal(|ui| {
                ui.monospace(vim.mode().label());
                let pending = vim.pending_display();
                if !pending.is_empty() {
                    ui.separator();
                    ui.monospace(pending);
                }
                if let Some(message) = status_message.as_deref() {
                    ui.separator();
                    ui.label(message);
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_secs_f64(
                            (*status_expires_at - now).max(0.0),
                        ));
                }
            });
        });

        if let Some((placement, count)) = keyboard_select_visible {
            let row_span = *row_height + root_ui.spacing().item_spacing.y;
            let list_height = (root_ui.available_height() - timeline_height).max(row_span);
            let ids = viewer
                .displayed_items()
                .iter()
                .map(DisplayedItem::id)
                .collect::<Vec<_>>();
            if !ids.is_empty() {
                let first = (*y_offset / row_span).ceil() as usize;
                let last = ((*y_offset + list_height - *row_height) / row_span).floor() as usize;
                let middle = ((*y_offset + list_height * 0.5) / row_span).floor() as usize;
                let count_offset = count.saturating_sub(1);
                let index = match placement {
                    FocusPlacement::Top => first.saturating_add(count_offset),
                    FocusPlacement::Center => middle,
                    FocusPlacement::Bottom => last.saturating_sub(count_offset),
                }
                .min(ids.len() - 1);
                viewer.apply(ViewerCommand::SetFocusedItem(ids[index]));
            }
        }

        if keyboard_half_page_scroll != 0 {
            let row_span = *row_height + root_ui.spacing().item_spacing.y;
            let visible_ids = viewer
                .displayed_items()
                .iter()
                .map(DisplayedItem::id)
                .collect::<Vec<_>>();
            if let Some(current) = viewer
                .cursor_state()
                .focused_item()
                .and_then(|focused| visible_ids.iter().position(|id| *id == focused))
            {
                let list_height = (root_ui.available_height() - timeline_height).max(row_span);
                let half_page_rows = (list_height / row_span / 2.0).floor().max(1.0) as isize;
                let row_delta = keyboard_half_page_scroll.saturating_mul(half_page_rows);
                let target = current
                    .saturating_add_signed(row_delta)
                    .min(visible_ids.len().saturating_sub(1));
                viewer.apply(ViewerCommand::SetFocusedItem(visible_ids[target]));
                *y_offset = (*y_offset + row_delta as f32 * row_span).max(0.0);
                reveal_keyboard_focus = true;
            }
        }
        if let Some(placement) = keyboard_focus_placement {
            let row_span = *row_height + root_ui.spacing().item_spacing.y;
            let visible_ids = viewer
                .displayed_items()
                .iter()
                .map(DisplayedItem::id)
                .collect::<Vec<_>>();
            if let Some(index) = viewer
                .cursor_state()
                .focused_item()
                .and_then(|focused| visible_ids.iter().position(|id| *id == focused))
            {
                let list_height = (root_ui.available_height() - timeline_height).max(row_span);
                let row_top = index as f32 * row_span;
                let row_bottom = row_top + *row_height;
                *y_offset = match placement {
                    FocusPlacement::Top => row_top,
                    FocusPlacement::Center => (row_top + row_bottom - list_height) * 0.5,
                    FocusPlacement::Bottom => row_bottom - list_height,
                }
                .max(0.0);
                reveal_keyboard_focus = true;
            }
        }
        let focused_item = viewer.cursor_state().focused_item();

        egui::Panel::left("side_panel")
            .default_size(180.0)
            .min_size(120.0)
            .max_size(600.0)
            .resizable(true)
            .show(root_ui, |ui| {
                let max_rect = ui.max_rect();

                let (header_rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), timeline_height),
                    egui::Sense::hover(),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(header_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.label("🔎");
                        ui.add(egui::TextEdit::singleline(&mut search_text).id(signal_search_id()));
                    },
                );
                let rows_top = ui.next_widget_position().y;
                let search_matcher = SearchMatcher::new(&search_text);

                let spacing = ui.spacing().item_spacing;
                let row_height_with_spacing = *row_height + spacing.y;
                let name_row_y_offset = spacing.y * 1.75;

                use egui::*;

                let viewport = Rect::from_min_size(
                    egui::pos2(max_rect.left(), rows_top + name_row_y_offset - *y_offset),
                    egui::vec2(max_rect.width(), max_rect.height() + *y_offset),
                );

                let mut ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(viewport)
                        .layout(*ui.layout()),
                );

                let mut content_clip_rect = max_rect.expand(ui.visuals().clip_rect_margin);

                content_clip_rect.min.y = rows_top;
                ui.set_clip_rect(content_clip_rect);
                let num_rows = displayed_items.len();
                ui.set_height(
                    (row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0),
                );
                // let min_row = (viewport.min.y / row_height_with_spacing);
                let min_row = (*y_offset / row_height_with_spacing).floor().at_least(0.0) as usize;
                // let max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
                let max_row = ((*y_offset + max_rect.bottom() - rows_top) / row_height_with_spacing)
                    .ceil() as usize
                    + 1;
                let max_row = max_row.at_most(num_rows);

                ui.set_height(
                    (row_height_with_spacing * num_rows as f32 + spacing.y).at_least(0.0),
                );
                let max_row = max_row.at_most(num_rows);

                // let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
                // let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

                // let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=y_max);

                let response = egui_dnd::dnd(&mut ui, "dnd").show_custom(|ui, iter| {
                    if min_row > 0 {
                        ui.add_space(row_height_with_spacing * min_row as f32);
                    }
                    for (i, item) in displayed_items
                        .iter()
                        .enumerate()
                        .take(max_row)
                        .skip(min_row)
                    {
                        iter.next(ui, egui::Id::new(item.id()), i, true, |ui, item_handle| {
                            item_handle.ui(ui, |ui, handle, _state| {
                                ui.horizontal(|ui| {
                                    ui.set_height(*row_height);
                                    handle.ui(ui, |ui| {
                                        if let Some(signal) = item
                                            .signal_id()
                                            .and_then(|id| viewer.waveform().signal(id))
                                        {
                                            let size =
                                                egui::vec2(ui.available_width(), *row_height);
                                            let text = highlighted_signal_name(
                                                ui,
                                                signal.name(),
                                                search_matcher.as_ref(),
                                            );
                                            if ui
                                                .add_sized(
                                                    size,
                                                    egui::Button::selectable(
                                                        focused_item == Some(item.id()),
                                                        text,
                                                    )
                                                    .truncate(),
                                                )
                                                .clicked()
                                            {
                                                requested_focus = Some(item.id());
                                            }
                                        }
                                    });
                                });
                            })
                        });
                    }
                });
                if let Some(update) = response.final_update() {
                    egui_dnd::utils::shift_vec(update.from, update.to, &mut displayed_items);
                }
            });

        if search_text != viewer.search() {
            search_history.reset_navigation();
            viewer.apply(ViewerCommand::SetSearch(search_text));
            if focus_first_search_match(viewer) {
                reveal_keyboard_focus = true;
                force_vertical_scroll = true;
            }
        }
        let new_order: Vec<_> = displayed_items.iter().map(DisplayedItem::id).collect();
        if new_order != original_order {
            viewer.apply(ViewerCommand::SetDisplayedOrder(new_order));
        }
        if let Some(id) = requested_focus {
            viewer.apply(ViewerCommand::SetFocusedItem(id));
        }

        let mut pending_commands = Vec::new();
        let mut active_measurement_start = viewer.cursor_state().measurement_start();
        let persistent_cursor = viewer.cursor();

        egui::CentralPanel::default().show(root_ui, |ui| {
            let time_viewport = viewer.viewport();
            wave_dispatch::render_timeline(
                ui,
                viewer.capture_end(),
                time_viewport.start().floor().max(0.0) as u64,
                time_viewport.end().ceil().max(1.0) as u64,
                viewer.cursor(),
                viewer.cursor_state().measurement_start(),
                &mut pending_commands,
            );

            let min_rect = ui.min_rect();
            let max_rect = ui.max_rect();

            let spacing = ui.spacing().item_spacing;
            let row_height_with_spacing = *row_height + spacing.y;

            let filtered = viewer
                .displayed_items()
                .iter()
                .filter_map(|item| item.signal_id().and_then(|id| viewer.waveform().signal(id)))
                .collect::<Vec<_>>();

            let num_rows = filtered.len();
            if reveal_keyboard_focus {
                let focused_index = viewer.cursor_state().focused_item().and_then(|focused| {
                    viewer
                        .displayed_items()
                        .iter()
                        .position(|item| item.id() == focused)
                });
                if let Some(index) = focused_index {
                    let row_top = index as f32 * row_height_with_spacing;
                    let row_bottom = row_top + *row_height;
                    let visible_height = ui.available_height().max(*row_height);
                    if row_top < *y_offset {
                        *y_offset = row_top;
                    } else if row_bottom > *y_offset + visible_height {
                        *y_offset = row_bottom - visible_height;
                    }
                }
            }

            let content_height = (row_height_with_spacing * num_rows as f32 - spacing.y).max(0.0);
            let max_offset = (content_height - ui.available_height()).max(0.0);
            *y_offset = (*y_offset).clamp(0.0, max_offset);

            let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false; 2]);
            if force_vertical_scroll || reveal_keyboard_focus {
                scroll_area = scroll_area.vertical_scroll_offset(*y_offset);
            }

            scroll_area.show_viewport(ui, |ui, viewport| {
                // this is kinda nasty because you end up with a 1 frame lag between the waves and
                // the labels. Maybe having 2 separate scroll areas would one? One hoirzontal only
                // for the wave and a vertical only for waves and labels? I feel like I tried this
                // and it didn't work out properly.
                *y_offset = viewport.min.y;
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

                let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=y_max);

                let view_start = time_viewport.start() as f32;
                let view_end = time_viewport.end() as f32;
                let pixels_per_tick = rect.width() / time_viewport.span() as f32;

                info.rect = rect;
                info.min_rect = min_rect;
                info.max_rect = max_rect;
                info.viewport = viewport;
                info.pixels_per_tick = pixels_per_tick;

                let wave_resp = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
                        let resp = ui.interact(
                            max_rect,
                            egui::Id::new("ui_hover"),
                            // if I change this to hover, I can click and drag to move waves around
                            egui::Sense::click_and_drag(),
                            // egui::Sense::hover(),
                            // egui::Sense::
                        );
                        let hover_pos = resp.hover_pos();

                        let x_frac = hover_pos
                            .map(|hover_pos| (hover_pos.x - min_rect.min.x) / min_rect.width());
                        // let x_val = x_frac.map(|x_frac| {
                        //     (viewport.min.x + x_frac * (viewport.max.x - viewport.min.x))
                        //         / 32.0
                        //         / *x_scale
                        // });
                        ui.vertical(|ui| {
                            for d in filtered.iter().take(max_row).skip(min_row) {
                                wave_dispatch::render_wave(
                                    ui,
                                    d.name(),
                                    pixels_per_tick,
                                    view_start,
                                    view_end,
                                    *row_height,
                                    d.signal(),
                                );
                            }
                        });

                        if ui.rect_contains_pointer(egui::Rect::EVERYTHING) {
                            let zoom = ui.input(|i| i.zoom_delta());
                            if zoom != 1.0 {
                                if let Some(x_frac) = x_frac {
                                    let anchor = time_viewport.start()
                                        + f64::from(x_frac) * time_viewport.span();
                                    pending_commands.push(ViewerCommand::ZoomTime {
                                        anchor,
                                        factor: f64::from(zoom),
                                    });
                                }
                            }

                            let scroll_x = ui.input(|i| i.smooth_scroll_delta.x);
                            if scroll_x != 0.0 {
                                pending_commands.push(ViewerCommand::PanTime(f64::from(
                                    -scroll_x / pixels_per_tick,
                                )));
                            }
                        }
                        resp
                    })
                    .inner;

                let yellow = egui::Color32::from_rgb(0xd2, 0x99, 0x1d);

                if let Some(pos) = &wave_resp.hover_pos() {
                    use egui::*;
                    let mut shapes = vec![];
                    // let color = Color32::from_additive_luminance(196);

                    let x = pos.x;
                    let t = view_start + (x - rect.min.x) / pixels_per_tick;
                    let t_rounded = t.round();
                    let hover_t = t_rounded as u64;

                    if wave_resp.drag_started() {
                        active_measurement_start = Some(hover_t);
                        pending_commands.push(ViewerCommand::BeginMeasurement(hover_t));
                    } else if wave_resp.dragged() {
                        pending_commands.push(ViewerCommand::UpdateMeasurement(hover_t));
                    } else if wave_resp.clicked() {
                        pending_commands.push(ViewerCommand::SetCursor(hover_t));
                    }

                    let rounded_x = rect.min.x + (t_rounded - view_start) * pixels_per_tick;
                    let p0 = pos2(rounded_x, max_rect.min.y + 0.0);
                    let p1 = pos2(rounded_x, max_rect.max.y);
                    let stroke = Stroke::new(2.0_f32, yellow);
                    shapes.push(Shape::line_segment([p0, p1], stroke));

                    if let Some(start_t) = active_measurement_start {
                        let rounded_x =
                            rect.min.x + (start_t as f32 - view_start) * pixels_per_tick;
                        let sp0 = pos2(rounded_x, max_rect.min.y + 0.0);
                        let sp1 = pos2(rounded_x, max_rect.max.y);
                        let stroke = Stroke::new(2.0_f32, yellow);
                        shapes.push(Shape::line_segment([sp0, sp1], stroke));
                        let pp0 = pos2(rounded_x, max_rect.min.y);
                        shapes.push(Shape::rect_filled(
                            egui::Rect::from_two_pos(pp0, p1),
                            egui::CornerRadius::ZERO,
                            yellow.linear_multiply(0.1),
                        ));
                    }
                    ui.painter().extend(shapes);
                }

                if let Some(cursor_time) = persistent_cursor {
                    let cursor_x = rect.min.x + (cursor_time as f32 - view_start) * pixels_per_tick;
                    if rect.x_range().contains(cursor_x) {
                        ui.painter().line_segment(
                            [
                                pos2(cursor_x, max_rect.min.y),
                                pos2(cursor_x, max_rect.max.y),
                            ],
                            Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                        );
                    }
                }

                if wave_resp.drag_stopped() {
                    pending_commands.push(ViewerCommand::EndMeasurement);
                }
            });
        });

        for command in pending_commands {
            viewer.apply(command);
        }

        if let Some(url) = url_window.show(&ctx) {
            for effect in viewer.apply(ViewerCommand::RequestOpenUrl(url)) {
                if let EffectRequest::OpenUrl(url) = effect {
                    let request = ehttp::Request::get(&url);
                    let dl = download.clone();
                    *dl.lock().unwrap() = Download::InProgress;
                    let ctx2 = ctx.clone();
                    ehttp::fetch(request, move |response| {
                        *dl.lock().unwrap() = Download::Done(response);
                        ctx2.request_repaint();
                    });
                    ctx.request_repaint();
                }
            }
        }
        err_window.show(&ctx);
        if let Some(url) = live.show(&ctx) {
            for effect in viewer.apply(ViewerCommand::RequestLiveConnection(url)) {
                if let EffectRequest::ConnectLive(url) = effect {
                    live.connect(url, &ctx);
                }
            }
        }
        if let Some(result) = live.poll() {
            match result {
                Ok((signals, time)) => {
                    let preserve_view = !viewer.waveform().signals().is_empty();
                    viewer.apply(ViewerCommand::ReplaceWaveform {
                        waveform: mk_waveform(signals, time.max(1)),
                        preserve_view,
                    });
                }
                Err(error) => {
                    err_window.msg = error;
                    err_window.open = true;
                }
            }
        }

        self.ui_file_drag_and_drop(&ctx);

        if false {
            egui::Window::new("Window").show(&ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

fn mk_waveform(sigs: Vec<(vcd::ScopedVar, vcd::Signal)>, end_time: u64) -> Waveform {
    let signals = sigs
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
    Waveform::new(signals, end_time)
}

fn highlighted_signal_name(
    ui: &egui::Ui,
    name: &str,
    matcher: Option<&SearchMatcher>,
) -> egui::WidgetText {
    let Some(matcher) = matcher else {
        return egui::WidgetText::from(name.to_owned());
    };
    let ranges = matcher.ranges(name).collect::<Vec<_>>();
    if ranges.is_empty() {
        return egui::WidgetText::from(name.to_owned());
    }

    let normal = egui::TextFormat {
        font_id: egui::TextStyle::Button.resolve(ui.style()),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    let matched = egui::TextFormat {
        background: egui::Color32::DARK_GREEN,
        ..normal.clone()
    };
    let mut job = egui::text::LayoutJob::default();
    let mut end = 0;
    for range in ranges {
        job.append(&name[end..range.start], 0.0, normal.clone());
        job.append(&name[range.clone()], 0.0, matched.clone());
        end = range.end;
    }
    job.append(&name[end..], 0.0, normal);
    job.into()
}

fn focus_first_search_match(viewer: &mut ViewerState) -> bool {
    let Some(matcher) = SearchMatcher::new(viewer.search()) else {
        return false;
    };
    let first = viewer.displayed_items().iter().find_map(|item| {
        let signal = item
            .signal_id()
            .and_then(|id| viewer.waveform().signal(id))?;
        matcher.is_match(signal.name()).then_some(item.id())
    });
    if let Some(id) = first {
        viewer.apply(ViewerCommand::SetFocusedItem(id));
        true
    } else {
        false
    }
}

fn take_vim_inputs(ctx: &egui::Context, keyboard_captured: bool) -> Vec<VimInput> {
    ctx.input_mut(|input| {
        let modifiers = input.modifiers;
        let mut vim_inputs = Vec::new();
        input.events.retain(|event| {
            let Some(vim_input) = vim_input_for_event(event, modifiers) else {
                return true;
            };
            if keyboard_captured && vim_input != VimInput::Escape {
                return true;
            }
            vim_inputs.push(vim_input);
            false
        });
        vim_inputs
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchInput {
    Previous,
    Next,
    Accept,
}

fn take_search_inputs(ctx: &egui::Context, search_has_focus: bool) -> Vec<SearchInput> {
    if !search_has_focus {
        return Vec::new();
    }
    ctx.input_mut(|input| {
        let mut search_inputs = Vec::new();
        input.events.retain(|event| {
            let search_input = match event {
                egui::Event::Key {
                    key: egui::Key::ArrowUp,
                    pressed: true,
                    ..
                } => Some(SearchInput::Previous),
                egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    ..
                } => Some(SearchInput::Next),
                egui::Event::Key {
                    key: egui::Key::P,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.ctrl => Some(SearchInput::Previous),
                egui::Event::Key {
                    key: egui::Key::N,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.ctrl => Some(SearchInput::Next),
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => Some(SearchInput::Accept),
                _ => None,
            };
            if let Some(search_input) = search_input {
                search_inputs.push(search_input);
                false
            } else {
                true
            }
        });
        search_inputs
    })
}

fn vim_input_for_event(event: &egui::Event, modifiers: egui::Modifiers) -> Option<VimInput> {
    match event {
        egui::Event::Text(text) if !modifiers.ctrl && !modifiers.command => {
            let mut characters = text.chars();
            let character = characters.next()?;
            characters
                .next()
                .is_none()
                .then_some(VimInput::Char(character))
        }
        egui::Event::Key {
            key: egui::Key::Escape,
            pressed: true,
            ..
        } => Some(VimInput::Escape),
        egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.ctrl => ctrl_key_character(*key).map(VimInput::Ctrl),
        _ => None,
    }
}

fn signal_search_id() -> egui::Id {
    egui::Id::new("signal_search")
}

fn ctrl_key_character(key: egui::Key) -> Option<char> {
    match key {
        egui::Key::B => Some('b'),
        egui::Key::D => Some('d'),
        egui::Key::E => Some('e'),
        egui::Key::F => Some('f'),
        egui::Key::R => Some('r'),
        egui::Key::U => Some('u'),
        egui::Key::Y => Some('y'),
        _ => None,
    }
}

impl TemplateApp {
    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let mut text = "Dropping files:\n".to_owned();
            ctx.input(|i| {
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        text += &format!("\n{}", path.display());
                    } else if !file.mime.is_empty() {
                        text += &format!("\n{}", file.mime);
                    } else {
                        text += "\n???";
                    }
                }
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            self.dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        }

        // Show dropped files (if any):
        if !self.dropped_files.is_empty() {
            if let Some(path) = &self.dropped_files[0].path {
                let mut file = std::fs::File::open(path).unwrap();
                let mut buf_file = std::io::BufReader::new(&mut file);
                let (sigs, time) = vcd::read_clocked_vcd(&mut buf_file).unwrap();
                self.viewer.apply(ViewerCommand::ReplaceWaveform {
                    waveform: mk_waveform(sigs, time),
                    preserve_view: false,
                });
            } else if let Some(bytes) = &self.dropped_files[0].bytes {
                let mut cursor = std::io::Cursor::new(&bytes);
                let (sigs, time) = vcd::read_clocked_vcd(&mut cursor).unwrap();
                self.viewer.apply(ViewerCommand::ReplaceWaveform {
                    waveform: mk_waveform(sigs, time),
                    preserve_view: false,
                });
            }
        }
        self.dropped_files.clear();
    }
}
