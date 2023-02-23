use eframe::egui;
use std::f32;

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use egui::plot::{GridInput, GridMark};
use egui::*;
use plot::{
    Arrows, Bar, BarChart, CoordinatesFormatter, Corner, HLine, Legend, Line, LineStyle,
    MarkerShape, Plot, PlotImage, Points, Polygon, Text, VLine,
};

//#[derive(PartialEq)]
struct Spectrum {}

impl Default for Spectrum {
    fn default() -> Self {
        Self {}
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.ctx().request_repaint();
        self.bar_plot(ui)
    }

    fn bar_plot(&mut self, ui: &mut Ui) -> Response {
        let values = vec![0.0; 10]; // ToDo get values from Queue object
        let mut chart = BarChart::new(
            (0..512)
                .step_by(1)
                .map(|x| (x as f64, values[x] as f64))
                .map(|(x, f)| Bar::new(x, f.abs()).width(0.01))
                .collect(),
        )
        .color(Color32::LIGHT_BLUE)
        .horizontal();

        Plot::new("Spectrum Demo")
            .legend(Legend::default())
            .data_aspect(4.0 / 512.0)
            .include_x(2.0)
            .include_y(512.0)
            .width(100.0)
            .show(ui, |plot_ui| plot_ui.bar_chart(chart))
            .response
    }

    fn set_values(&mut self, new_values: &[f32]) {
        // Todo: maybe delete this function
        // prupose is calling from outer to set values,
        // maybe set Spectrogramm or texures now
        let mut values = vec![0.0; 10];
        for (value, new_value) in values.iter_mut().zip(new_values.iter()) {
            *value = *new_value;
        }
    }
}

pub struct SpectrogramGui {
    spectrum: Spectrum,
}

impl Default for SpectrogramGui {
    fn default() -> Self {
        Self {
            spectrum: Spectrum::default(),
        }
    }
}

impl eframe::App for SpectrogramGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SpectrogramGui");
            ui.vertical(|ui| {
                self.spectrum.ui(ui);
                ui.label("Test ");
            });
        });
    }
}
