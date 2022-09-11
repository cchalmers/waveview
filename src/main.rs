#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    color_eyre::install().unwrap();
    // let mut file = std::fs::File::open("/Users/chris/Dev/egui/eframe_template/clkdiv2n_tb.vcd").unwrap();
    // signals.iter().for_each(|sig| eprintln!("{:?}", sig.0.scopes));
    // eprintln!("{signals:?}");
    // let app = eframe_template::TemplateApp::new(signals, time);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "waveview",
        native_options,
        Box::new(|_cc| {
            let mut file =
                std::fs::File::open("/Users/chris/Dev/egui/eframe_template/mlp512b4c1.vcd")
                    .unwrap();
            let (signals, time) = eframe_template::vcd::read_clocked_vcd(&mut file).unwrap();
            Box::new(eframe_template::TemplateApp::new(signals, time))
        }),
    );
}
