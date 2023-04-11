#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use clap::Parser;

#[derive(Parser)]
struct Opt {
    starting_file: Option<std::path::PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opt = Opt::parse();
    color_eyre::install().unwrap();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "waveview",
        native_options,
        Box::new(move |_cc| {
            let mut file = if let Some(path) = &opt.starting_file {
                std::fs::File::open(path).unwrap()
            } else {
                std::fs::File::open("/Users/chris/Dev/egui/waveview/mlp512b4c1.vcd").unwrap()
            };
            let (signals, time) = waveview::vcd::read_clocked_vcd(&mut file).unwrap();
            Box::new(waveview::TemplateApp::new(signals, time))
        }),
    )
    .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    tracing::event!(tracing::Level::INFO, "tracing says hi");

    let signals = vec![];

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| {
                let app = waveview::TemplateApp::new(signals, 1);
                if let Some(Ok(s)) = web_sys::window().map(|w| w.location().search()) {
                    if let Some(url) = s.strip_prefix('?') {
                        let request = ehttp::Request::get(url);
                        let dl = app.download.clone();
                        *dl.lock().unwrap() = waveview::app::Download::InProgress;
                        let ctx = cc.egui_ctx.clone();
                        ehttp::fetch(request, move |response| {
                            *dl.lock().unwrap() = waveview::app::Download::Done(response);
                            ctx.request_repaint();
                        });
                    }
                }
                Box::new(app)
            }),
        )
        .await
        .expect("failed to start eframe");
    });
}
