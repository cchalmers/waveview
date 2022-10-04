#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use clap::Parser;

#[derive(Parser)]
struct Opt {
    starting_file: Option<std::path::PathBuf>,
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opt = Opt::from_args();
    color_eyre::install().unwrap();
    // let mut file = std::fs::File::open("/Users/chris/Dev/egui/eframe_template/clkdiv2n_tb.vcd").unwrap();
    // signals.iter().for_each(|sig| eprintln!("{:?}", sig.0.scopes));
    // eprintln!("{signals:?}");
    // let app = eframe_template::TemplateApp::new(signals, time);
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "waveview",
        native_options,
        Box::new(move |_cc| {
            // cc.egui_ctx.set_visuals(eframe::egui::Visuals::light());
            // let path = opt.starting_file.().unwrap_or();
            let mut file = if let Some(path) = &opt.starting_file {
                std::fs::File::open(path).unwrap()
            } else {
                std::fs::File::open("/Users/chris/Dev/egui/waveview/mlp512b4c1.vcd").unwrap()
            };
            let (signals, time) = waveview::vcd::read_clocked_vcd(&mut file).unwrap();
            Box::new(waveview::TemplateApp::new(signals, time))
        }),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    tracing::event!(tracing::Level::INFO, "tracing says hi");

    let signals = vec![];

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // hardcode it
        web_options,
        Box::new(|_cc| Box::new(waveview::TemplateApp::new(signals, 1))),
    )
    .expect("failed to start eframe");
}
