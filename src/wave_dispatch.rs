use eframe::egui;
use std::collections::HashSet;
use waveview_model::search::SearchMatcher;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;
use waveview_model::viewer::ViewerState;
use waveview_model::waveform::Waveform;
use waveview_model::SignalId;

#[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
static RELOAD_STATUS: OnceLock<Arc<Mutex<Option<String>>>> = OnceLock::new();
use waveview_model::vim::{VimInput, VimState};

#[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
#[hot_lib_reloader::hot_module(dylib = "waveview_ui_reload", file_watch_debounce = 100)]
mod hot_ui {
    use eframe::egui;
    use std::collections::HashSet;
    use waveview_model::search::SearchMatcher;
    use waveview_model::vcd;
    use waveview_model::viewer::ViewerCommand;
    use waveview_model::viewer::ViewerState;
    use waveview_model::vim::{VimInput, VimState};
    use waveview_model::waveform::Waveform;
    use waveview_model::SignalId;

    hot_functions_from_file!("waveview-ui-reload/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

pub fn handle_vim_input(
    vim: &mut VimState,
    input: VimInput,
    keyboard_captured: bool,
    viewer: &ViewerState,
    commands: &mut Vec<ViewerCommand>,
) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::handle_vim_input(vim, input, keyboard_captured, viewer, commands);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::handle_vim_input(vim, input, keyboard_captured, viewer, commands);
}

pub fn render_signal_button(
    ui: &mut egui::Ui,
    name: &str,
    height: f32,
    selected: bool,
    matcher: Option<&SearchMatcher>,
) -> bool {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    return hot_ui::render_signal_button(ui, name, height, selected, matcher);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_signal_button(ui, name, height, selected, matcher)
}

pub fn render_signal_activity_button(ui: &mut egui::Ui, selected: bool) -> bool {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    return hot_ui::render_signal_activity_button(ui, selected);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_signal_activity_button(ui, selected)
}

pub fn render_signal_browser_header(ui: &mut egui::Ui, all_signals_displayed: bool) -> bool {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    return hot_ui::render_signal_browser_header(ui, all_signals_displayed);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_signal_browser_header(ui, all_signals_displayed)
}

pub fn render_signal_browser_tree(
    ui: &mut egui::Ui,
    waveform: &Waveform,
    query: &str,
    expanded: &mut HashSet<String>,
    displayed: &HashSet<SignalId>,
    additions: &mut Vec<SignalId>,
) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_signal_browser_tree(ui, waveform, query, expanded, displayed, additions);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_signal_browser_tree(ui, waveform, query, expanded, displayed, additions);
}

pub fn render_vim_status(ui: &mut egui::Ui, vim: &VimState, message: Option<&str>) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_vim_status(ui, vim, message);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_vim_status(ui, vim, message);
}

pub fn render_key_help(ui: &mut egui::Ui) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_key_help(ui);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_key_help(ui);
}

pub fn reload_status() -> Option<String> {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    return RELOAD_STATUS
        .get()
        .and_then(|status| status.lock().ok()?.clone());

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    None
}

pub fn timeline_height() -> f32 {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    return hot_ui::timeline_height();

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::timeline_height()
}

pub fn render_timeline(
    ui: &mut egui::Ui,
    capture_end: u64,
    view_start: u64,
    view_end: u64,
    cursor: Option<u64>,
    measurement_start: Option<u64>,
    commands: &mut Vec<ViewerCommand>,
) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_timeline(
        ui,
        capture_end,
        view_start,
        view_end,
        cursor,
        measurement_start,
        commands,
    );

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_timeline(
        ui,
        capture_end,
        view_start,
        view_end,
        cursor,
        measurement_start,
        commands,
    );
}

pub fn render_wave(
    ui: &mut egui::Ui,
    name: &str,
    pixels_per_tick: f32,
    view_start: f32,
    view_end: f32,
    height: f32,
    signal: &vcd::Signal,
) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_wave(
        ui,
        name,
        pixels_per_tick,
        view_start,
        view_end,
        height,
        signal,
    );

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_wave(
        ui,
        name,
        pixels_per_tick,
        view_start,
        view_end,
        height,
        signal,
    );
}

pub fn install_reload_repaint(ctx: &egui::Context) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    {
        // egui stores its label-selection plugin by Rust TypeId. Types compiled into a newly
        // loaded dylib get different TypeIds, so selectable labels can panic after a reload.
        // Label selection is not useful enough in this development-only mode to justify that
        // risk; explicit Waveview copy commands remain available.
        ctx.all_styles_mut(|style| style.interaction.selectable_labels = false);

        let observer = hot_ui::subscribe();
        let observer_ctx = ctx.clone();
        std::thread::spawn(move || loop {
            observer.wait_for_reload();
            observer_ctx.request_repaint();
        });

        if let Ok(path) = std::env::var("WAVEVIEW_RELOAD_STATUS_FILE") {
            let status = RELOAD_STATUS
                .get_or_init(|| Arc::new(Mutex::new(None)))
                .clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let mut previous = String::new();
                loop {
                    if let Ok(current) = std::fs::read_to_string(&path) {
                        if current != previous {
                            previous.clone_from(&current);
                            let message = match current.trim() {
                                "building" => Some("rebuilding UI…".to_owned()),
                                "failed" => {
                                    Some("UI reload failed; previous UI still active".to_owned())
                                }
                                "restart" => {
                                    Some("restart required; host or shared code changed".to_owned())
                                }
                                _ => None,
                            };
                            if let Ok(mut shared) = status.lock() {
                                *shared = message;
                            }
                            ctx.request_repaint();
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
            });
        }
    }

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    let _ = ctx;
}
