use eframe::egui;
use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use egui::plot::{GridInput, GridMark};
use egui::*;
use plot::{
    Arrows, Bar, BarChart, CoordinatesFormatter, Corner, HLine, Legend, Line, LineStyle,
    MarkerShape, Plot, PlotImage, Points, Polygon, Text, VLine,
};

#[derive(PartialEq)]
struct Spectrum {
    window_size: usize,
    step_size: usize,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            window_size: 512,
            step_size: 512,
        }
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        self.bar_plot(ui)
    }

    fn bar_plot(&self, ui: &mut Ui) -> Response {
        let mut chart = BarChart::new(
            (-395..=395)
                .step_by(10)
                .map(|x| x as f64 * 0.01)
                .map(|x| {
                    (
                        x,
                        (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt(),
                    )
                })
                // The 10 factor here is purely for a nice 1:1 aspect ratio
                .map(|(x, f)| Bar::new(x, f * 1.0).width(0.1))
                .collect(),
        )
        .color(Color32::LIGHT_BLUE)
        .horizontal();

        Plot::new("Normal Distribution Demo")
            .legend(Legend::default())
            //            .clamp_grid(true)
            .show(ui, |plot_ui| plot_ui.bar_chart(chart))
            .response
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
