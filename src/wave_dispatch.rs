use eframe::egui;
use waveview_model::vcd;

#[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
#[hot_lib_reloader::hot_module(dylib = "waveview_ui_reload", file_watch_debounce = 100)]
mod hot_ui {
    use eframe::egui;
    use waveview_model::vcd;

    hot_functions_from_file!("waveview-ui-reload/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

pub fn render_wave(
    ui: &mut egui::Ui,
    name: &str,
    scale: f32,
    view_start: f32,
    view_end: f32,
    height: f32,
    signal: &vcd::Signal,
) {
    #[cfg(all(feature = "reload", not(target_arch = "wasm32")))]
    hot_ui::render_wave(ui, name, scale, view_start, view_end, height, signal);

    #[cfg(not(all(feature = "reload", not(target_arch = "wasm32"))))]
    waveview_ui::render_wave(ui, name, scale, view_start, view_end, height, signal);
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
