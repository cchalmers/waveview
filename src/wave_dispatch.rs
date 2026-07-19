use eframe::egui;
use waveview_model::vcd;
use waveview_model::viewer::ViewerCommand;

#[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
#[hot_lib_reloader::hot_module(dylib = "waveview_ui_reload", file_watch_debounce = 100)]
mod hot_ui {
    use eframe::egui;
    use waveview_model::vcd;
    use waveview_model::viewer::ViewerCommand;

    hot_functions_from_file!("waveview-ui-reload/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
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
        let observer = hot_ui::subscribe();
        let ctx = ctx.clone();
        std::thread::spawn(move || loop {
            observer.wait_for_reload();
            ctx.request_repaint();
        });
    }

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    let _ = ctx;
}
