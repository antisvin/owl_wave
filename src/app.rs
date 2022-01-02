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
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Default for OwlWaveApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: format!("Owl Wave {}", VERSION),
            active_wave_id: 0,
            grid: Grid::new(8, 8, 256),
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
    }
}
