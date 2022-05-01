#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio_devices;
mod grid;
mod midi_devices;
mod owl_control;
mod wave;
//mod effects;
pub use app::OwlWaveApp;

//#[macro_use]
//extern crate lazy_static;

// ----------------------------------------------------------------------------
// When compiling for web:
#[cfg(target_arch = "wasm32")]
pub fn log(s: String) {
    web_sys::console::log(&js_sys::Array::of1(&s.into()));
}
/*
macro_rules! println {
    ()              => (log("".to_owned()));
    ($($arg:tt)*)   => (log(format!($($arg)*)));
}
 */
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();
    eframe::start_web(canvas_id, Box::new(|cc| Box::new(OwlWaveApp::new(cc))))
}
