#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::OwlWaveApp;
mod audio_devices;
mod grid;
mod midi_devices;
mod owl_control;
mod wave;
//mod effects;
