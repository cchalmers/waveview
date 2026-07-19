// #![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub use waveview_model::vcd;
mod wave_dispatch;
pub use app::TemplateApp;
mod live;
mod prompt;
pub mod samples;

pub use wave_dispatch::install_reload_repaint;
