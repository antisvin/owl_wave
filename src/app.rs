use crate::audio_devices::AudioHandler;
use crate::owl_control::OwlCommandProcessor;
use crate::{
    grid::Grid,
    midi_devices::{MidiDeviceSelection, MidiInputHandle, MidiOutputHandle},
};
use cpal::traits::DeviceTrait;
use cpal::HostId;
use eframe::egui::{
    self,
    plot::{Bar, BarChart},
};
use eframe::epaint::text::LayoutJob;
use eframe::epaint::{Color32, FontId};
use egui::plot::{Plot, Points, Value, Values};
use itertools::{EitherOrBoth::Both, EitherOrBoth::Left, EitherOrBoth::Right, Itertools};
use owl_midi::OpenWareMidiSysexCommand;
use std::io::Cursor;
use std::sync::Mutex;
use std::{fs::File, sync::Arc};
use wavetable::WavHandler;
use wmidi::MidiMessage;

enum MenuPage {
    Parameters,
    Patches,
    Resources,
    Settings,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct OwlWaveApp {
    active_wave_id: usize,
    midi_log: Arc<Mutex<Vec<u8>>>,

    #[serde(skip)]
    midi_devices: MidiDeviceSelection,
    #[serde(skip)]
    midi_input: MidiInputHandle<Arc<Mutex<Vec<u8>>>>,
    #[serde(skip)]
    owl_command_processor: OwlCommandProcessor,
    #[serde(skip)]
    midi_output: MidiOutputHandle,
    #[serde(skip)]
    midi_loaded: bool,
    #[serde(skip)]
    show_about: bool,
    #[serde(skip)]
    grid: Grid,
    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,
    #[serde(skip)]
    audio_handler: AudioHandler,
    #[serde(skip)]
    selected_audio_host: Option<HostId>,
    selected_audio_input: Option<usize>,
    selected_audio_output: Option<usize>,
    #[serde(skip)]
    menu_page: MenuPage,
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
            //log: String::new(),
            dropped_files: Vec::<egui::DroppedFile>::new(),
            audio_handler: AudioHandler::new(),
            selected_audio_host: None,
            selected_audio_input: None,
            selected_audio_output: None,
            menu_page: MenuPage::Parameters,
        }
    }
}

impl eframe::App for OwlWaveApp {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                    let plot = Plot::new(format!("points-{i}"))
                        .view_aspect(1.0)
                        .show_axes([false, true])
                        .allow_drag(false)
                        .allow_scroll(false)
                        .allow_zoom(false)
                        .allow_boxed_zoom(false);
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
                let plot = Plot::new("wavetable-main")
                    .view_aspect(1.0)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_boxed_zoom(false)
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
                let harm_plot = Plot::new("harmonics-main")
                    .view_aspect(4.0)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .show_axes([false, true]);
                harm_plot.show(ui, |plot_ui| plot_ui.bar_chart(harmonics));
            })
        });
        egui::Window::new("Grid").show(ctx, |ui| {
            //ui.label("Wavetables grid");
            egui::Grid::new("grid").show(ui, |ui| {
                let samples = self.grid.get_samples() as f64;
                let mut wave_id = 0;
                for i in 0..self.grid.get_rows() {
                    for j in 0..self.grid.get_cols() {
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
                        let plot = Plot::new(format!("plot-{i}-{j}"))
                            .view_aspect(1.0)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .allow_zoom(false)
                            .allow_boxed_zoom(false)
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
                ui.horizontal(|ui| {
                    if ui.button("🔃").clicked() || !self.audio_handler.audio_loaded {
                        self.audio_handler.scan();
                    }
                    let mut label = String::new();
                    let num_hosts = self.audio_handler.hosts.len();
                    if num_hosts == 1 {
                        label += "1 host";
                    } else {
                        label += format!("{} hosts", num_hosts).as_str();
                    }
                    ui.label(label);
                });
                ui.horizontal(|ui| {
                    for &host_id in self.audio_handler.hosts.keys() {
                        ui.selectable_value(
                            &mut self.selected_audio_host,
                            Some(host_id),
                            host_id.name(),
                        );
                    }
                });
                //let _default_in = host.default_input_device().map(|e| e.name().unwrap());
                //let _default_out = host.default_output_device().map(|e| e.name().unwrap());
                egui::Grid::new("audio-grid").show(ui, |ui| {
                    if let Some(host_id) = self.selected_audio_host {
                        let in_devices = self
                            .audio_handler
                            .input_devices
                            .get(&host_id)
                            .into_iter()
                            .flatten();
                        let out_devices = self
                            .audio_handler
                            .output_devices
                            .get(&host_id)
                            .into_iter()
                            .flatten();
                        let mut selected_audio_input = self.selected_audio_input;
                        let mut selected_audio_output = self.selected_audio_output;
                        for (i, pair) in in_devices.zip_longest(out_devices).enumerate() {
                            match pair {
                                Both(input_device, output_device) => {
                                    let input_name =
                                        input_device.name().unwrap_or_else(|_| " - ".to_string());
                                    ui.radio_value(&mut selected_audio_input, Some(i), input_name);
                                    let output_name =
                                        output_device.name().unwrap_or_else(|_| " - ".to_string());
                                    ui.radio_value(
                                        &mut selected_audio_output,
                                        Some(i),
                                        output_name,
                                    );
                                }
                                Right(output_device) => {
                                    let output_name =
                                        output_device.name().unwrap_or_else(|_| " - ".to_string());
                                    ui.radio_value(
                                        &mut selected_audio_output,
                                        Some(i),
                                        output_name,
                                    );
                                    ui.label("");
                                }
                                Left(input_device) => {
                                    ui.label("".to_string());
                                    let input_name =
                                        input_device.name().unwrap_or_else(|_| " - ".to_string());
                                    ui.radio_value(&mut selected_audio_input, Some(i), input_name);
                                }
                            }
                            ui.end_row()
                        }
                        if selected_audio_input != self.selected_audio_input {
                            // Connect to a different input
                            self.audio_handler
                                .select_input(self.selected_audio_host, selected_audio_input);
                            self.selected_audio_input = selected_audio_input
                        }
                        if selected_audio_output != self.selected_audio_output {
                            // Connect to a different input
                            self.audio_handler
                                .select_output(self.selected_audio_host, selected_audio_output)
                                .unwrap();
                            self.selected_audio_output = selected_audio_output
                        }
                    }
                });
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

                egui::Grid::new("midi-grid").show(ui, |ui| {
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
                            Both((i, out_port_name), (j, in_port_name)) => {
                                let show_in = self.midi_devices.show_midi_device(in_port_name);
                                let show_out = self.midi_devices.show_midi_device(out_port_name);
                                if show_in || show_out {
                                    if show_out {
                                        ui.radio_value(&mut selected_output_port, i, out_port_name);
                                    } else {
                                        ui.label("");
                                    }
                                    if show_in {
                                        ui.radio_value(&mut selected_input_port, j, in_port_name);
                                    } else {
                                        ui.label("");
                                    }
                                    ui.end_row()
                                }
                            }
                            Left((i, out_port_name)) => {
                                if self.midi_devices.show_midi_device(out_port_name) {
                                    ui.radio_value(&mut selected_output_port, i, out_port_name);
                                    ui.label("");
                                    ui.end_row();
                                }
                            }
                            Right((j, in_port_name)) => {
                                if self.midi_devices.show_midi_device(in_port_name) {
                                    ui.label("");
                                    ui.radio_value(&mut selected_input_port, j, in_port_name);
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

                if self.midi_devices == MidiDeviceSelection::Owl {
                    //ui.separator();

                    egui::TopBottomPanel::top("top-panel").show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.menu_button("Remote control", |ui| {
                                ui.menu_button("Request", |ui| {
                                    if ui.button("Firmware name").clicked() {
                                        if let Some(connection) = &mut self.midi_output.connection {
                                            self.owl_command_processor
                                            .request_settings(
                                                connection,
                                                OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION,
                                            )
                                            .unwrap();
                                        }
                                        ui.close_menu()
                                    };
                                    if ui.button("Patches").clicked() {
                                        if let Some(connection) = &mut self.midi_output.connection {
                                            self.owl_command_processor
                                            .request_settings(
                                                connection,
                                                OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND,
                                            )
                                            .unwrap();
                                        }
                                        ui.close_menu();
                                        self.menu_page = MenuPage::Patches
                                    };
                                    if ui.button("Resources").clicked() {
                                        if let Some(connection) = &mut self.midi_output.connection {
                                            self.owl_command_processor
                                    .request_settings(
                                        connection,
                                        OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND,
                                    )
                                    .unwrap();
                                        }
                                        ui.close_menu();
                                        self.menu_page = MenuPage::Resources
                                    };
                                    if ui.button("Program stats").clicked() {
                                        if let Some(connection) = &mut self.midi_output.connection {
                                            self.owl_command_processor
                                                .request_settings(
                                                    connection,
                                                    OpenWareMidiSysexCommand::SYSEX_PROGRAM_STATS,
                                                )
                                                .unwrap();
                                        }
                                        ui.close_menu()
                                    };
                                });
                                ui.menu_button("Device", |ui| {
                                    if ui.button("Reset").clicked() {
                                        if let Some(connection) = &mut self.midi_output.connection {
                                            self.owl_command_processor
                                    .send_sysex_command(
                                        connection,
                                        OpenWareMidiSysexCommand::SYSEX_DEVICE_RESET_COMMAND,
                                    )
                                    .unwrap();
                                        }
                                        ui.close_menu()
                                    }
                                })
                            });

                            let mut job = LayoutJob::default();
                            let first_row_indentation = 10.0;
                            let (default_color, strong_color) = if ui.visuals().dark_mode {
                                (Color32::LIGHT_GRAY, Color32::WHITE)
                            } else {
                                (Color32::DARK_GRAY, Color32::BLACK)
                            };
                            if let Some(patch_name) = &self.owl_command_processor.patch_name {
                                job.append(
                                    format!("{patch_name}\n").as_str(),
                                    first_row_indentation,
                                    egui::TextFormat {
                                        font_id: FontId::proportional(20.0),
                                        color: strong_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            if let Some(firmware_version) =
                                &self.owl_command_processor.firmware_version
                            {
                                job.append(
                                    format!("{firmware_version}\n").as_str(),
                                    0.0,
                                    egui::TextFormat {
                                        color: default_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            if let Some(program_stats) = &self.owl_command_processor.program_stats {
                                job.append(
                                    format!("{program_stats}\n").as_str(),
                                    0.0,
                                    egui::TextFormat {
                                        color: default_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            if let Some(program_message) =
                                &self.owl_command_processor.program_message
                            {
                                job.append(
                                    format!("{program_message}\n").as_str(),
                                    0.0,
                                    egui::TextFormat {
                                        color: default_color,
                                        ..Default::default()
                                    },
                                );
                                /*
                                job.append(
                                    "or error message ",
                                    0.0,
                                    egui::TextFormat {
                                        color: Color32::DARK_RED,
                                        ..Default::default()
                                    },
                                );
                                */
                            }
                            ui.label(job);
                        });
                    });

                    egui::SidePanel::left("left-panel")
                        .resizable(true)
                        .default_width(150.0)
                        .width_range(80.0..=200.0)
                        .show_inside(ui, |ui| {
                            ui.vertical_centered_justified(|ui| {
                                if ui.button("Parameters").clicked() {
                                    self.menu_page = MenuPage::Parameters
                                }
                                if ui.button("Patches").clicked() {
                                    self.menu_page = MenuPage::Patches
                                }
                                if ui.button("Resources").clicked() {
                                    self.menu_page = MenuPage::Resources
                                }
                                if ui.button("Settings").clicked() {
                                    self.menu_page = MenuPage::Settings
                                }
                            });
                        });

                    egui::TopBottomPanel::bottom("bottom-panel")
                        .resizable(false)
                        .min_height(0.0)
                        .show_inside(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("Log");
                            });
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.label(&self.owl_command_processor.log);
                                });
                        });
                    egui::CentralPanel::default().show_inside(ui, |ui| match self.menu_page {
                        MenuPage::Parameters => {}
                        MenuPage::Patches => {
                            ui.vertical_centered(|ui| {
                                ui.heading("Patches");
                            });
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, patch) in self
                                    .owl_command_processor
                                    .patch_names
                                    .iter()
                                    .skip(1)
                                    .enumerate()
                                {
                                    ui.label(format!("{:>2}. {}", i + 1, patch));
                                }
                            });
                        }
                        MenuPage::Resources => {
                            ui.vertical_centered(|ui| {
                                ui.heading("Resources");
                            });
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, patch) in
                                    self.owl_command_processor.resource_names.iter().enumerate()
                                {
                                    ui.label(format!("{:>2}. {}", i + 1, patch));
                                }
                            });
                        }
                        MenuPage::Settings => {}
                    });
                    //ui.horizontal(|ui|{});
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
                                            /*
                                            if let Some(output) = &self.owl_command_processor.output
                                            {
                                                let s = format!("{}\n", output);
                                                self.log += s.as_str()
                                            }
                                            */
                                        }
                                    }
                                }
                            }
                        }
                        data_guard.clear();
                    }
                }
            });
        });

        self.ui_file_drag_and_drop(ctx);
    }
}

impl OwlWaveApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
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
                FontId::monospace(14.0),
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
