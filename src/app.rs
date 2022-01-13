use std::io::Cursor;
use std::sync::Mutex;
use std::{fs::File, sync::Arc};

use crate::owl_control::OwlCommandProcessor;
//use crate::midi_handler::handle_message;
use crate::{
    grid::Grid,
    midi_devices::{MidiDeviceSelection, MidiInputHandle, MidiOutputHandle},
};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::HostId;
use eframe::{
    egui::{
        self,
        plot::{Bar, BarChart},
    },
    epi,
};
use egui::plot::{Plot, Points, Value, Values};
use itertools::{EitherOrBoth::Both, EitherOrBoth::Left, EitherOrBoth::Right, Itertools};
use owl_midi::OpenWareMidiSysexCommand;
//use once_cell::sync::Lazy;
use wavetable::WavHandler;
use wmidi::MidiMessage;
/*
lazy_static! {
    static ref MIDI_HANDLER: Mutex<RefCell<Option<MidiHandler<'static>>>> =
        Mutex::new(RefCell::new(None));
} */

//static MIDI_HANDLER: SyncLazy<Mutex<RefCell<Option<MidiHandler<'static>>>>> =
//    SyncLazy::new(|| Mutex::new(RefCell::new(Some(MidiHandler::new()))));

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct OwlWaveApp {
    label: String,
    active_wave_id: usize,
    midi_log: Arc<Mutex<Vec<u8>>>,

    midi_devices: MidiDeviceSelection,
    #[cfg_attr(feature = "persistence", serde(skip))]
    midi_input: MidiInputHandle<Arc<Mutex<Vec<u8>>>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    owl_command_processor: OwlCommandProcessor,
    #[cfg_attr(feature = "persistence", serde(skip))]
    midi_output: MidiOutputHandle,
    #[cfg_attr(feature = "persistence", serde(skip))]
    midi_loaded: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    show_about: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    grid: Grid,
    #[cfg_attr(feature = "persistence", serde(skip))]
    log: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dropped_files: Vec<egui::DroppedFile>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Default for OwlWaveApp {
    fn default() -> Self {
        let midi_log = Arc::new(Mutex::new(Vec::new()));
        let midi_input = MidiInputHandle::new(
            "OWL wave",
            0,
            |stamp, message, log| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
                log.lock().unwrap().extend_from_slice(message)
                //log.extend_from_slice(message);
            },
            //move |timestamp, data, _| midi_handler.handle_message(timestamp, data),
            midi_log.clone(),
        );
        Self {
            label: format!("OWL Wave {}", VERSION),
            active_wave_id: 0,
            midi_log,
            owl_command_processor: OwlCommandProcessor::new(),
            midi_devices: MidiDeviceSelection::Owl,
            //midi_in_ports: Arc::new(MidiInputPorts::new()),
            midi_input,
            midi_output: MidiOutputHandle::new("OWL Wave", 0),
            midi_loaded: false,
            show_about: false,
            grid: Grid::new(8, 8, 256),
            log: String::new(),
            dropped_files: Vec::<egui::DroppedFile>::new(),
        }
    }
}

impl epi::App for OwlWaveApp {
    fn name(&self) -> &str {
        self.label.as_str()
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        //self.fft_context = start_fft_thread();
        //let mut planner = get_fft_planner();

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        //self.grid = Grid::new(8, 8, 256)
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        //let Self { label, grid } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.menu_button("File", |ui| {
                    if !frame.is_web() {
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Open").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Ok(open_file) = File::open(path) {
                                    if let Ok(wav_content) = WavHandler::read_content(open_file) {
                                        self.grid.load_waves(&wav_content).unwrap_or(0);
                                    }
                                }
                            }
                        }
                    }

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                    };
                });
            });
        });
        egui::Window::new("about")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.label(format!("Version: {}", VERSION));
            });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Wavetables");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..self.grid.get_waves() {
                    //ui.label(i.to_string());
                    let samples = self.grid.get_samples() as f64;
                    let points = Points::new(Values::from_values(
                        self.grid
                            .get_wave_by_id(i)
                            .iter()
                            .enumerate()
                            .map(|(i, &v)| Value::new(i as f64 / samples, v))
                            .collect(),
                    ))
                    .stems(-1.5)
                    .radius(1.0);
                    //ui.points(points.name("Points with stems"));
                    let plot = Plot::new("Points")
                        .view_aspect(1.0)
                        .allow_drag(false)
                        .show_axes([false, true]);
                    //ui.add(plot);
                    let plot = plot.show_background(self.active_wave_id == i);
                    let response = plot.show(ui, |plot_ui| plot_ui.points(points)).response;
                    if response.clicked() {
                        self.active_wave_id = i
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("OWL wave");
            ui.hyperlink("https://github.com/antisvin/owl_wave");
            ui.add(egui::github_link_file!(
                "https://github.com/antisvin/owl_wave/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        egui::Window::new("Wavetable").show(ctx, |ui| {
            ui.vertical(|ui| {
                let samples = self.grid.get_samples();
                let points = Points::new(Values::from_values(
                    self.grid
                        .get_wave_by_id(self.active_wave_id)
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| Value::new(i as f64 / samples as f64, v))
                        .collect(),
                ))
                .stems(-1.5)
                .radius(1.0);
                //ui.points(points.name("Points with stems"));
                let plot = Plot::new("Points")
                    .view_aspect(1.0)
                    .allow_drag(false)
                    .show_axes([false, true]);
                plot.show(ui, |plot_ui| plot_ui.points(points));

                // Harmonics
                let harmonics = BarChart::new(
                    self.grid
                        .get_harmonics(self.active_wave_id)
                        .iter()
                        .take(samples / 2)
                        .enumerate()
                        .map(|(i, &v)| {
                            Bar::new(i as f64 / samples as f64, v.to_polar().0)
                                .width(1.0 / samples as f64)
                        })
                        .collect(),
                );
                let harm_plot = Plot::new("Harmonics")
                    .view_aspect(4.0)
                    .allow_drag(false)
                    .show_axes([false, true]);
                harm_plot.show(ui, |plot_ui| plot_ui.bar_chart(harmonics));
            })
        });
        egui::Window::new("Grid").show(ctx, |ui| {
            //ui.label("Wavetables grid");
            egui::Grid::new("grid").show(ui, |ui| {
                let samples = self.grid.get_samples() as f64;
                let mut wave_id = 0;
                for _i in 0..self.grid.get_rows() {
                    for _j in 0..self.grid.get_cols() {
                        let points = Points::new(Values::from_values(
                            self.grid
                                .get_wave_by_id(wave_id)
                                .iter()
                                .enumerate()
                                .map(|(i, &v)| Value::new(i as f64 / samples, v))
                                .collect(),
                        ))
                        .stems(-1.5)
                        .radius(1.0);
                        //ui.points(points.name("Points with stems"));
                        let plot = Plot::new("Points")
                            .view_aspect(1.0)
                            .allow_drag(false)
                            .show_axes([false, true]);
                        //ui.add(plot);
                        let plot = plot.show_background(self.active_wave_id == wave_id);
                        let response = plot.show(ui, |plot_ui| plot_ui.points(points)).response;
                        if response.clicked() {
                            self.active_wave_id = wave_id;
                        }
                        wave_id += 1;
                    }
                    ui.end_row()
                }
            });
        });

        egui::Window::new("Audio Devices").show(ctx, |ui| {
            ui.vertical(|ui| {
                let input_hosts = cpal::available_hosts();
                let mut num_input_devices = 0;
                for host_id in input_hosts {
                    if let Ok(host) = cpal::host_from_id(host_id) {
                        ui.label(host_id.name());
                        let _default_in = host.default_input_device().map(|e| e.name().unwrap());
                        let _default_out = host.default_output_device().map(|e| e.name().unwrap());
                        let input_devices: Vec<(HostId, usize, String)> = host
                            .input_devices()
                            .map(|devices| {
                                devices.enumerate().map(|(device_index, device)| {
                                    (
                                        host_id,
                                        device_index + num_input_devices,
                                        device.name().unwrap_or_else(|_| "-".to_string()),
                                    )
                                })
                            })
                            .unwrap()
                            .collect();
                        num_input_devices += input_devices.len();
                        for (_host, _device_index, device_name) in input_devices.iter() {
                            ui.label(device_name);
                        }
                    }
                }
            });
        });

        // MIDI devices window
        egui::Window::new("MIDI Devices").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("🔃").clicked() {
                        self.reset_midi();
                    }
                    ui.selectable_value(&mut self.midi_devices, MidiDeviceSelection::All, "All");
                    ui.selectable_value(&mut self.midi_devices, MidiDeviceSelection::Owl, "OWL");
                });
                ui.separator();

                if !self.midi_loaded {
                    // Reconnect
                    self.update_midi_input();
                    self.update_midi_output();
                    self.midi_loaded = true;
                }

                egui::Grid::new("grid").show(ui, |ui| {
                    let mut selected_input_port = *self.midi_input.get_selected_port_mut();
                    let mut selected_output_port = *self.midi_output.get_selected_port_mut();
                    for pair in self
                        .midi_input
                        .names
                        .iter()
                        .enumerate()
                        .zip_longest(self.midi_output.names.iter().enumerate())
                    {
                        match pair {
                            Both((i, in_port_name), (j, out_port_name)) => {
                                let show_in = self.midi_devices.show_midi_device(in_port_name);
                                let show_out = self.midi_devices.show_midi_device(out_port_name);
                                if show_in || show_out {
                                    if show_in {
                                        ui.radio_value(&mut selected_input_port, i, in_port_name);
                                    } else {
                                        ui.label("");
                                    }
                                    if show_out {
                                        ui.radio_value(&mut selected_output_port, j, out_port_name);
                                    } else {
                                        ui.label("");
                                    }
                                    ui.end_row()
                                }
                            }
                            Left((i, in_port_name)) => {
                                if self.midi_devices.show_midi_device(in_port_name) {
                                    ui.radio_value(&mut selected_input_port, i, in_port_name);
                                    ui.label("");
                                    ui.end_row();
                                }
                            }
                            Right((j, out_port_name)) => {
                                if self.midi_devices.show_midi_device(out_port_name) {
                                    ui.label("");
                                    ui.radio_value(&mut selected_output_port, j, out_port_name);
                                    ui.end_row()
                                }
                            }
                        }
                    }
                    if selected_input_port != self.midi_input.selected_port {
                        // Connect to a different input
                        self.midi_input.selected_port = selected_input_port;
                        self.update_midi_input()
                    }
                    if selected_output_port != self.midi_output.selected_port {
                        // Connect to a different output
                        self.midi_output.selected_port = selected_output_port;
                        self.update_midi_output()
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Info").clicked() {
                        if let Some(connection) = &mut self.midi_output.connection {
                            self.owl_command_processor
                                .request_settings(
                                    connection,
                                    OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION,
                                )
                                .unwrap();
                        }
                    }
                });
                if self.midi_devices == MidiDeviceSelection::Owl {
                    if let Ok(mut data_guard) = self.midi_log.lock() {
                        //data_guard.iter(|| wmidi::MidiMessage::try_from().unwrap());
                        let bytes = data_guard.as_mut_slice();
                        let mut start = 0;
                        while start < bytes.len() {
                            let message = wmidi::MidiMessage::try_from(&bytes[start..]).unwrap();
                            start += message.bytes_size();
                            if let MidiMessage::SysEx(_data) = message {
                                let mut buf: [u8; 256] = [0; 256];
                                if let Ok(size) = message.copy_to_slice(buf.as_mut_slice()) {
                                    if size > 0 {
                                        if let Ok(_result) =
                                            self.owl_command_processor.handle_response(&buf, size)
                                        {
                                            self.log += format!(
                                                "{}\n",
                                                self.owl_command_processor.output.as_ref().unwrap()
                                            )
                                            .as_str()
                                        }
                                    }
                                    //for byte in buf.iter().take(size) {
                                    //    self.log += format!("{:x?}", byte).as_str();
                                    //}
                                }
                            }
                        }
                        data_guard.clear();
                        //data_guard.

                        //for byte in data_guard.iter_mut() {
                        //    if let MidiMessage::SysEx(bytes) = MidiMessage::try_from(byte) {}
                        //}

                        //midi_handler.messages.pop() {
                    }
                    ui.label(self.log.clone());
                }
            });
        });

        self.ui_file_drag_and_drop(ctx);
    }
}

impl OwlWaveApp {
    fn ui_file_drag_and_drop(&mut self, ctx: &egui::CtxRef) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    text += &format!("\n{}", path.display());
                } else if !file.mime.is_empty() {
                    text += &format!("\n{}", file.mime);
                } else {
                    text += "\n???";
                }
            }

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading,
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }

        // Show dropped files (if any):
        if !self.dropped_files.is_empty() {
            for file in &self.dropped_files {
                let mut info = if let Some(path) = &file.path {
                    path.display().to_string()
                } else if !file.name.is_empty() {
                    file.name.clone()
                } else {
                    "???".to_owned()
                };
                if let Some(path) = &file.path {
                    if let Ok(open_file) = File::open(path) {
                        if let Ok(wav_content) = WavHandler::read_content(open_file) {
                            self.grid.load_waves(&wav_content).unwrap_or(0);
                        }
                    }
                } else if let Some(bytes) = &file.bytes {
                    info += &format!(" ({} bytes)", bytes.len());
                    let reader = Cursor::new(bytes);
                    if let Ok(wav_content) = WavHandler::read_content(reader) {
                        self.grid.load_waves(&wav_content).unwrap_or(0);
                    }
                }
            }
            self.dropped_files.clear();
        }
    }
    fn reset_midi(&mut self) -> &mut Self {
        self.midi_loaded = false;
        self
    }
    fn update_midi_input(&mut self) {
        self.midi_input = MidiInputHandle::new(
            "OWL wave",
            self.midi_input.selected_port,
            |stamp, message, log| {
                println!("{}: {:x?} (len = {})", stamp, message, message.len());
                log.lock().unwrap().extend_from_slice(message)
            },
            self.midi_log.clone(),
        );
    }
    fn update_midi_output(&mut self) {
        self.midi_output = MidiOutputHandle::new("OWL wave", self.midi_output.selected_port);
    }
}
