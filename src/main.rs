#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OWL Wave",
        native_options,
        Box::new(|cc| Box::new(owl_wave::OwlWaveApp::new(cc))),
    );
}
