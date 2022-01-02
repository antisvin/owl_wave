use crate::grid::Grid;
use eframe::{egui, epi};
use egui::plot::{Plot, Points, Value, Values};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct OwlWaveApp {
    // Example stuff:
    label: String,
    active_wave_id: usize,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    grid: Grid,
    #[cfg_attr(feature = "serde", serde(skip))]
    dropped_files: Vec<egui::DroppedFile>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Default for OwlWaveApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: format!("Owl Wave {}", VERSION),
            active_wave_id: 0,
            grid: Grid::new(8, 8, 256),
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

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if !frame.is_web() {
                        //ui.output().open_url(format!("#{}", anchor));
                        //ui.button(tex)
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Open").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                let picked_path = Some(path.display().to_string());
                                if let Some(picked_path) = picked_path {
                                    ui.horizontal(|ui| {
                                        ui.label("Picked file:");
                                        ui.monospace(picked_path);
                                    });
                                }
                            }
                        }
                    }

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Wavetables");

            //egui::Grid::new("tables list").show(ui,|ui| {
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
                    //ui.add(plot);
                    //ui.end_row();
                }
            });

            //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            //if ui.button("Increment").clicked() {
            //    *value += 1.0;
            //}

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/antisvin/owl_wave");
            ui.add(egui::github_link_file!(
                "https://github.com/antisvin/owl_wave/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        egui::Window::new("Wavetable").show(ctx, |ui| {
            let samples = self.grid.get_samples() as f64;
            let points = Points::new(Values::from_values(
                self.grid
                    .get_wave_by_id(self.active_wave_id)
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
            let _response = plot.show(ui, |plot_ui| plot_ui.points(points)).response;
            //if response.clicked() {
            //    self.active_wave_id = i
            //}
        });
        egui::Window::new("Grid").show(ctx, |ui| {
            ui.label("Wavetables grid");
        });

        //self.backend_panel.end_of_frame(ctx);

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
            let mut open = true;
            egui::Window::new("Dropped files")
                .open(&mut open)
                .show(ctx, |ui| {
                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            info += &format!(" ({} bytes)", bytes.len());
                        }
                        ui.label(info);
                    }
                });
            if !open {
                self.dropped_files.clear();
            }
        }
    }
}
