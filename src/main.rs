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
    let native_options = eframe::NativeOptions {
        window_builder: Some(Box::new(|viewport| {
            viewport
                .with_title("waveview")
                .with_inner_size((1536.0, 768.0))
        })),
        ..Default::default()
    };
    eframe::run_native(
        "waveview",
        native_options,
        Box::new(move |cc| {
            waveview::install_reload_repaint(&cc.egui_ctx);
            let (signals, time) = if let Some(path) = &opt.starting_file {
                let file = std::fs::File::open(path).unwrap();
                let mut buf_file = std::io::BufReader::new(file);
                waveview::vcd::read_clocked_vcd(&mut buf_file).unwrap()
            } else {
                (vec![], 1)
            };
            Ok(Box::new(waveview::TemplateApp::new(cc, signals, time)))
        }),
    )
    .unwrap();
}

/// Cancel the browser's default action for the Ctrl/Cmd chords that waveview's Vim engine
/// binds in normal mode, so `Ctrl-D` (bookmark), `Ctrl-U` (view-source), `Ctrl-F` (find), etc.
/// drive the viewer instead of the browser.
///
/// eframe 0.35 only calls `preventDefault` for a fixed set of keys (`Ctrl-O/P/S`, arrows, …)
/// and offers no hook to extend it for keydown, so we install our own listener. egui still
/// receives the key — we only suppress the browser's default action.
///
/// Notes:
/// * `Ctrl-R` here becomes Vim "redo" rather than reload; `F5` still reloads.
/// * `Ctrl-N` / `Ctrl-T` / `Ctrl-W` cannot be intercepted by a web page, so they are not listed.
#[cfg(target_arch = "wasm32")]
fn install_browser_key_guard() {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast as _;

    // Chords handled in normal mode (search-mode Ctrl-P/N only fire while a text field is
    // focused, where we intentionally defer to the browser).
    const GUARDED: &[&str] = &["b", "d", "e", "f", "i", "o", "r", "u", "v", "y"];

    let Some(window) = web_sys::window() else {
        return;
    };

    let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            // Only a lone Ctrl/Cmd chord (no Alt) maps to a Vim command.
            if !(event.ctrl_key() || event.meta_key()) || event.alt_key() {
                return;
            }
            // While the user is typing in a text field (egui's hidden input, the URL box, …)
            // leave the browser alone so copy/paste/select-all keep working.
            let editing = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
                .is_some_and(|el| {
                    matches!(el.tag_name().to_ascii_lowercase().as_str(), "input" | "textarea")
                });
            if editing {
                return;
            }
            let key = event.key().to_ascii_lowercase();
            if GUARDED.contains(&key.as_str()) {
                event.prevent_default();
            }
        },
    );

    let options = web_sys::AddEventListenerOptions::new();
    // Capture phase: run before egui's canvas handler, which stops propagation.
    options.set_capture(true);
    if let Err(err) = window.add_event_listener_with_callback_and_add_event_listener_options(
        "keydown",
        closure.as_ref().unchecked_ref(),
        &options,
    ) {
        log::error!("failed to install browser key guard: {err:?}");
    }
    // Keep the listener alive for the lifetime of the page.
    closure.forget();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    install_browser_key_guard();

    let signals = vec![];
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("missing canvas element #the_canvas_id");

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let app = waveview::TemplateApp::new(cc, signals, 1);
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
                    Ok(Box::new(app))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
