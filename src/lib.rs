// #![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod vcd;
mod wave;
pub use app::TemplateApp;
pub mod samples;
pub mod timeline;

#[cfg(target_arch = "wasm32")]
mod ws_wasm;
