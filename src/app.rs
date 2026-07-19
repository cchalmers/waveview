use crate::vcd;
use crate::wave_dispatch;
use eframe::egui;
use eframe::egui::NumExt;
use egui::*;
use waveview_model::search::{SearchHistory, SearchMatcher};
use waveview_model::ui_types::{MenuAction, SignalMenuAction, SignalPresentation};
use waveview_model::viewer::{
    DisplayedItem, EffectRequest, FocusPlacement, ViewerCommand, ViewerState,
};
use waveview_model::vim::{VimInput, VimState};
use waveview_model::waveform::Waveform;
use waveview_model::DisplayedItemId;

use std::collections::HashSet;
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
    prompt: crate::prompt::PromptRuntime,
    #[serde(skip)]
    status_message: Option<String>,
    #[serde(skip)]
    status_expires_at: f64,
    #[serde(skip)]
    alias_editor: AliasEditor,
    #[serde(skip)]
    wave_context_target: Option<DisplayedItemId>,
    search_history: SearchHistory,
    command_history: SearchHistory,
    selected_activity: Option<Activity>,
    signal_browser_search: String,
    expanded_signal_scopes: HashSet<String>,
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
            prompt: crate::prompt::PromptRuntime::default(),
            status_message: None,
            status_expires_at: 0.0,
            alias_editor: AliasEditor::default(),
            wave_context_target: None,
            search_history: SearchHistory::default(),
            command_history: SearchHistory::default(),
            selected_activity: Some(Activity::Signals),
            signal_browser_search: String::new(),
            expanded_signal_scopes: HashSet::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
enum SidePanel {
    None,
    Info,
    Samples,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum Activity {
    Signals,
}

#[derive(Default)]
struct AliasEditor {
    target: Option<DisplayedItemId>,
    input: String,
    request_focus: bool,
}

impl AliasEditor {
    fn open(&mut self, target: DisplayedItemId, current: Option<&str>) {
        self.target = Some(target);
        self.input = current.unwrap_or_default().to_owned();
        self.request_focus = true;
    }

    fn is_open(&self) -> bool {
        self.target.is_some()
    }

    fn close(&mut self) {
        self.target = None;
        self.request_focus = false;
    }

    fn show(&mut self, ctx: &egui::Context) -> Option<(DisplayedItemId, Option<String>)> {
        let target = self.target?;
        let mut window_open = true;
        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("Signal alias")
            .id(egui::Id::new("signal_alias_window"))
            .open(&mut window_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Display name");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .id(signal_alias_id())
                        .desired_width(320.0),
                );
                if self.request_focus {
                    response.request_focus();
                    self.request_focus = false;
                }
                submit |= response.has_focus() && ui.input(|input| input.key_pressed(Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if ui.button("Apply").clicked() {
                        submit = true;
                    }
                });
            });
        if !window_open || cancel {
            self.close();
        } else if submit {
            let alias = (!self.input.trim().is_empty()).then(|| self.input.trim().to_owned());
            self.close();
            return Some((target, alias));
        }
        None
    }
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
            prompt: crate::prompt::PromptRuntime::default(),
            status_message: None,
            status_expires_at: 0.0,
            alias_editor: AliasEditor::default(),
            wave_context_target: None,
            search_history: SearchHistory::default(),
            command_history: SearchHistory::default(),
            selected_activity: Some(Activity::Signals),
            signal_browser_search: String::new(),
            expanded_signal_scopes: HashSet::new(),
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
            prompt,
            status_message,
            status_expires_at,
            alias_editor,
            wave_context_target,
            search_history,
            command_history,
            selected_activity,
            signal_browser_search,
            expanded_signal_scopes,
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
        let browser_search_has_focus =
            ctx.memory(|memory| memory.has_focus(signal_browser_search_id()));
        let prompt_has_focus = ctx.memory(|memory| memory.has_focus(command_prompt_id()));
        let keyboard_captured = search_has_focus
            || browser_search_has_focus
            || prompt_has_focus
            || url_window.open
            || err_window.open
            || live.open
            || alias_editor.is_open()
            || *show_key_help;
        let search_inputs = take_search_inputs(&ctx, search_has_focus);
        let prompt_inputs = take_search_inputs(&ctx, prompt_has_focus);
        let mut reveal_keyboard_focus = false;
        let mut keyboard_half_page_scroll = 0_isize;
        let mut keyboard_focus_placement = None;
        let mut keyboard_select_visible = None;
        let mut force_vertical_scroll = false;
        let mut prompt_history_moved = false;
        let mut prompt_scroll_to_bottom = false;
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
        for input in prompt_inputs {
            match input {
                SearchInput::Previous => {
                    if let Some(command) = command_history.previous(prompt.input()) {
                        prompt.set_input(command);
                        prompt_history_moved = true;
                    }
                }
                SearchInput::Next => {
                    if let Some(command) = command_history.newer() {
                        prompt.set_input(command);
                        prompt_history_moved = true;
                    }
                }
                SearchInput::Accept => {
                    let submitted = !prompt.input().trim().is_empty();
                    command_history.accept(prompt.input());
                    for command in prompt.submit() {
                        viewer.apply(command);
                    }
                    prompt_scroll_to_bottom = submitted;
                }
            }
        }
        for input in take_vim_inputs(&ctx, keyboard_captured) {
            if input == VimInput::Escape {
                if prompt.is_open() {
                    prompt.close();
                } else if alias_editor.is_open() {
                    alias_editor.close();
                } else if *show_key_help {
                    *show_key_help = false;
                } else if url_window.open {
                    url_window.open = false;
                } else if err_window.open {
                    err_window.open = false;
                } else if live.open {
                    live.open = false;
                } else if search_has_focus {
                    viewer.apply(ViewerCommand::SetSearch(String::new()));
                } else if browser_search_has_focus {
                    signal_browser_search.clear();
                }
                if let Some(focused) = ctx.memory(|memory| memory.focused()) {
                    ctx.memory_mut(|memory| memory.surrender_focus(focused));
                }
            }
            let mut vim_commands = Vec::new();
            wave_dispatch::handle_vim_input(
                vim,
                input,
                keyboard_captured,
                viewer,
                &mut vim_commands,
            );
            for command in vim_commands {
                reveal_keyboard_focus |= matches!(
                    command,
                    ViewerCommand::SetFocusedItem(_)
                        | ViewerCommand::MoveDisplayedItem { .. }
                        | ViewerCommand::RemoveDisplayedItem(_)
                        | ViewerCommand::RemoveDisplayedItems(_)
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
                        EffectRequest::FocusCommandPrompt => {
                            command_history.reset_navigation();
                            prompt.open();
                            ctx.memory_mut(|memory| memory.request_focus(command_prompt_id()));
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

        let mut menu_action = None;
        egui::Panel::top("top_panel").show(root_ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                ui.separator();
                ui.menu_button("File", |ui| {
                    menu_action =
                        wave_dispatch::render_file_menu(ui, cfg!(not(target_arch = "wasm32")));
                });
                ui.menu_button("Edit", |ui| {
                    menu_action = wave_dispatch::render_edit_menu(
                        ui,
                        viewer.cursor_state().focused_item().is_some(),
                    );
                });
                ui.menu_button("View", |ui| {
                    menu_action = wave_dispatch::render_view_menu(
                        ui,
                        *selected_activity == Some(Activity::Signals),
                        matches!(*side_panel, SidePanel::Info),
                        matches!(*side_panel, SidePanel::Samples),
                        row_height,
                    );
                });
                ui.menu_button("Help", |ui| {
                    menu_action = wave_dispatch::render_help_menu(ui);
                });
            });
        });

        match menu_action {
            Some(MenuAction::OpenFile) => {
                for effect in viewer.apply(ViewerCommand::RequestOpenFile) {
                    if effect == EffectRequest::OpenFile {
                        *a_future = Some(Box::pin(async {
                            let handle = rfd::AsyncFileDialog::new().pick_file().await;
                            if let Some(h) = &handle {
                                let bytes = h.read().await;
                                let mut cursor = std::io::Cursor::new(&bytes);
                                let (signals, time) = vcd::read_clocked_vcd(&mut cursor).unwrap();
                                Some(OpenedVcd {
                                    waveform: mk_waveform(signals, time),
                                })
                            } else {
                                None
                            }
                        }));
                    }
                }
                ctx.request_repaint();
            }
            Some(MenuAction::OpenUrl) => url_window.open = true,
            Some(MenuAction::ConnectLive) => live.open = true,
            Some(MenuAction::Reset) => {
                viewer.apply(ViewerCommand::ReplaceWaveform {
                    waveform: Waveform::empty(),
                    preserve_view: false,
                });
                *y_offset = 0.0;
            }
            Some(MenuAction::Quit) => ctx.send_viewport_cmd(ViewportCommand::Close),
            Some(MenuAction::UndoDisplayChange) => {
                viewer.apply(ViewerCommand::UndoDisplayChange);
            }
            Some(MenuAction::RedoDisplayChange) => {
                viewer.apply(ViewerCommand::RedoDisplayChange);
            }
            Some(MenuAction::RemoveFocusedItem) => {
                if let Some(id) = viewer.cursor_state().focused_item() {
                    viewer.apply(ViewerCommand::RemoveDisplayedItem(id));
                }
            }
            Some(MenuAction::ToggleSignalBrowser) => {
                *selected_activity =
                    (*selected_activity != Some(Activity::Signals)).then_some(Activity::Signals);
            }
            Some(MenuAction::FitTime) => {
                viewer.apply(ViewerCommand::FitTime);
            }
            Some(MenuAction::ShowInfo) => *side_panel = SidePanel::Info,
            Some(MenuAction::ShowSamples) => *side_panel = SidePanel::Samples,
            Some(MenuAction::HideInspector) => *side_panel = SidePanel::None,
            Some(MenuAction::ShowKeyHelp) => *show_key_help = true,
            None => {}
        }

        egui::Window::new("Keyboard shortcuts")
            .open(show_key_help)
            .resizable(true)
            .show(&ctx, |ui| {
                wave_dispatch::render_key_help(ui);
            });

        if let Some((id, alias)) = alias_editor.show(&ctx) {
            viewer.apply(ViewerCommand::SetDisplayedAlias { id, alias });
        }

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

        egui::Panel::left("activity_bar")
            .exact_size(38.0)
            .resizable(false)
            .show(root_ui, |ui| {
                let signals_selected = *selected_activity == Some(Activity::Signals);
                if wave_dispatch::render_signal_activity_button(ui, signals_selected) {
                    *selected_activity = if signals_selected {
                        None
                    } else {
                        Some(Activity::Signals)
                    };
                }
            });

        let mut signals_to_add = Vec::new();
        if *selected_activity == Some(Activity::Signals) {
            let displayed_signal_ids = viewer
                .displayed_items()
                .iter()
                .filter_map(DisplayedItem::signal_id)
                .collect::<HashSet<_>>();
            let all_signals_displayed = viewer
                .waveform()
                .signals()
                .iter()
                .all(|signal| displayed_signal_ids.contains(&signal.id()));
            egui::Panel::left("signal_browser")
                .default_size(240.0)
                .min_size(150.0)
                .max_size(500.0)
                .resizable(true)
                .show(root_ui, |ui| {
                    if wave_dispatch::render_signal_browser_header(ui, all_signals_displayed) {
                        signals_to_add
                            .extend(viewer.waveform().signals().iter().map(|signal| signal.id()));
                    }
                    ui.add(
                        egui::TextEdit::singleline(signal_browser_search)
                            .hint_text("regex search")
                            .id(signal_browser_search_id()),
                    );
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("available_signal_tree")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            wave_dispatch::render_signal_browser_tree(
                                ui,
                                viewer.waveform(),
                                signal_browser_search,
                                expanded_signal_scopes,
                                &displayed_signal_ids,
                                &mut signals_to_add,
                            );
                        });
                });
        }
        if !signals_to_add.is_empty() {
            viewer.apply(ViewerCommand::AddDisplayedSignals(signals_to_add));
        }

        let mut search_text = viewer.search().to_owned();
        let original_order: Vec<_> = viewer
            .displayed_items()
            .iter()
            .map(DisplayedItem::id)
            .collect();
        let mut displayed_items = viewer.displayed_items().to_vec();
        let mut requested_focus = None;
        let mut requested_signal_action = None;
        let timeline_height = wave_dispatch::timeline_height();
        let reload_status = wave_dispatch::reload_status();

        egui::Panel::bottom("vim_status").show(root_ui, |ui| {
            let message = reload_status.as_deref().or(status_message.as_deref());
            wave_dispatch::render_vim_status(ui, vim, message);
            if status_message.is_some() {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs_f64(
                        (*status_expires_at - now).max(0.0),
                    ));
            }
        });

        if prompt.is_open() {
            let prompt_config = wave_dispatch::prompt_config();
            egui::Panel::bottom("command_prompt")
                .default_size(prompt_config.panel_default_height)
                .min_size(prompt_config.panel_minimum_height)
                .max_size(prompt_config.panel_maximum_height)
                .resizable(prompt_config.panel_resizable)
                .show_separator_line(prompt_config.panel_show_separator_line)
                .show(root_ui, |ui| {
                    wave_dispatch::render_prompt_header(ui);
                    let body_rect = ui.available_rect_before_wrap();
                    let (transcript_rect, input_rect, separator_y) = wave_dispatch::prompt_layout(
                        body_rect,
                        ui.spacing().interact_size.y,
                        ui.spacing().item_spacing.y,
                    );

                    let mut transcript_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("command_prompt_transcript")
                            .max_rect(transcript_rect)
                            .layout(egui::Layout::top_down(
                                prompt_config.transcript_horizontal_align,
                            )),
                    );
                    transcript_ui.set_clip_rect(transcript_rect);
                    transcript_ui.set_min_size(transcript_rect.size());
                    egui::ScrollArea::vertical()
                        .id_salt("command_prompt_output")
                        .stick_to_bottom(prompt_config.scroll_stick_to_bottom)
                        .auto_shrink(prompt_config.scroll_auto_shrink)
                        .animated(prompt_config.scroll_animated)
                        .show(&mut transcript_ui, |ui| {
                            wave_dispatch::render_prompt_transcript(
                                ui,
                                prompt.output(),
                                transcript_rect,
                                prompt_scroll_to_bottom,
                            );
                        });

                    wave_dispatch::render_prompt_separator(ui, body_rect.x_range(), separator_y);

                    let mut input_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("command_prompt_input")
                            .max_rect(input_rect)
                            .layout(egui::Layout::left_to_right(
                                prompt_config.input_vertical_align,
                            )),
                    );
                    input_ui.set_clip_rect(input_rect);
                    input_ui.set_min_size(input_rect.size());
                    input_ui.horizontal(|ui| {
                        wave_dispatch::prepare_prompt_input(ui);
                        wave_dispatch::render_prompt_prefix(ui);
                        let cursor_end = prompt.input().chars().count();
                        let mut edit = egui::TextEdit::singleline(prompt.input_mut())
                            .id(command_prompt_id())
                            .margin(prompt_config.input_margin)
                            .desired_width(prompt_config.input_desired_width)
                            .min_size(prompt_config.input_min_size)
                            .clip_text(prompt_config.input_clip_text);
                        if !prompt_config.input_frame {
                            edit = edit.frame(egui::Frame::NONE);
                        }
                        let mut output = edit.show(ui);
                        let response = output.response;
                        if prompt_history_moved {
                            output.state.cursor.set_char_range(Some(
                                egui::text::CCursorRange::one(egui::text::CCursor::new(cursor_end)),
                            ));
                            output.state.store(ui.ctx(), command_prompt_id());
                        }
                        if prompt.take_focus_request() {
                            response.request_focus();
                        }
                    });

                    ui.allocate_rect(body_rect, egui::Sense::hover());
                });
        }

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
                                            let signal_response =
                                                wave_dispatch::render_signal_button(
                                                    ui,
                                                    item.alias().unwrap_or_else(|| signal.name()),
                                                    viewer.cursor().and_then(|time| {
                                                        signal.signal().value_at(time)
                                                    }),
                                                    SignalPresentation {
                                                        value_format: item.value_format(),
                                                        color: item.color(),
                                                    },
                                                    *row_height,
                                                    focused_item == Some(item.id()),
                                                    search_matcher.as_ref(),
                                                );
                                            if signal_response.clicked()
                                                || signal_response.secondary_clicked()
                                            {
                                                requested_focus = Some(item.id());
                                            }
                                            signal_response.context_menu(|ui| {
                                                if let Some(action) =
                                                    wave_dispatch::render_signal_context_menu(
                                                        ui,
                                                        item.alias().is_some(),
                                                        item.value_format(),
                                                        item.color(),
                                                    )
                                                {
                                                    requested_signal_action =
                                                        Some((item.id(), action));
                                                }
                                            });
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
        if let Some((id, action)) = requested_signal_action {
            match action {
                SignalMenuAction::EditAlias => {
                    let alias = displayed_items
                        .iter()
                        .find(|item| item.id() == id)
                        .and_then(DisplayedItem::alias);
                    alias_editor.open(id, alias);
                }
                SignalMenuAction::ClearAlias => {
                    viewer.apply(ViewerCommand::SetDisplayedAlias { id, alias: None });
                }
                SignalMenuAction::SetFormat(format) => {
                    viewer.apply(ViewerCommand::SetDisplayedValueFormat { id, format });
                }
                SignalMenuAction::SetColor(color) => {
                    viewer.apply(ViewerCommand::SetDisplayedColor { id, color });
                }
            }
        }

        let mut pending_commands = Vec::new();
        let mut requested_wave_focus = None;
        let mut requested_wave_action = None;

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

                let pixels_per_tick = rect.width() / time_viewport.span() as f32;

                info.rect = rect;
                info.min_rect = min_rect;
                info.max_rect = max_rect;
                info.viewport = viewport;
                info.pixels_per_tick = pixels_per_tick;

                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    let (wave_response, context_target) = wave_dispatch::render_wave_canvas(
                        ui,
                        viewer,
                        rect,
                        max_rect,
                        min_row..max_row,
                        *row_height,
                        &mut pending_commands,
                    );
                    if let Some(id) = context_target {
                        *wave_context_target = Some(id);
                        requested_wave_focus = Some(id);
                    }
                    let context_target = *wave_context_target;
                    wave_response.context_menu(|ui| {
                        let Some(item) = context_target.and_then(|id| {
                            viewer.displayed_items().iter().find(|item| item.id() == id)
                        }) else {
                            ui.close();
                            return;
                        };
                        if let Some(action) = wave_dispatch::render_signal_context_menu(
                            ui,
                            item.alias().is_some(),
                            item.value_format(),
                            item.color(),
                        ) {
                            requested_wave_action = Some((item.id(), action));
                        }
                    });
                });
            });
        });

        if let Some(id) = requested_wave_focus {
            viewer.apply(ViewerCommand::SetFocusedItem(id));
        }
        if let Some((id, action)) = requested_wave_action {
            match action {
                SignalMenuAction::EditAlias => {
                    let alias = viewer
                        .displayed_items()
                        .iter()
                        .find(|item| item.id() == id)
                        .and_then(DisplayedItem::alias);
                    alias_editor.open(id, alias);
                }
                SignalMenuAction::ClearAlias => {
                    viewer.apply(ViewerCommand::SetDisplayedAlias { id, alias: None });
                }
                SignalMenuAction::SetFormat(format) => {
                    viewer.apply(ViewerCommand::SetDisplayedValueFormat { id, format });
                }
                SignalMenuAction::SetColor(color) => {
                    viewer.apply(ViewerCommand::SetDisplayedColor { id, color });
                }
            }
        }

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
    Waveform::from_vcd(sigs, end_time)
}

fn focus_first_search_match(viewer: &mut ViewerState) -> bool {
    let Some(matcher) = SearchMatcher::new(viewer.search()) else {
        return false;
    };
    let first = viewer.displayed_items().iter().find_map(|item| {
        let signal = item
            .signal_id()
            .and_then(|id| viewer.waveform().signal(id))?;
        matcher
            .is_match(item.alias().unwrap_or_else(|| signal.name()))
            .then_some(item.id())
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
                egui::Event::Key {
                    key: egui::Key::J,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.ctrl => Some(SearchInput::Accept),
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

fn signal_alias_id() -> egui::Id {
    egui::Id::new("signal_alias")
}

fn signal_browser_search_id() -> egui::Id {
    egui::Id::new("signal_browser_search")
}

fn command_prompt_id() -> egui::Id {
    egui::Id::new("command_prompt_input")
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
