use eframe::egui;
use ringbuf::Consumer;
use ringbuf::SharedRb;
use std::f32;
use std::mem::MaybeUninit;
use std::sync::Arc;

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use egui::plot::{GridInput, GridMark};
use egui::*;
use plot::{
    Arrows, Bar, BarChart, CoordinatesFormatter, Corner, HLine, Legend, Line, LineStyle,
    MarkerShape, Plot, PlotImage, Points, Polygon, Text, VLine,
};

fn fm_data_signal_generator(num_samples: usize, phase_fm: f32) -> Vec<f32> {
    let freq: f32 = 1000.0;
    let fsample_rate: f32 = 48000.0;
    let phase: f32 = 0.0;
    let modulator_hub: f32 = 400.0;
    let modulator_freq: f32 = 10.0;
    let modulator_index: f32 = modulator_hub / modulator_freq;
    let shift = |t: f32, fmod: f32, fs: f32| -> f32 {
        (2.0 * f32::consts::PI * t * fmod / fs + phase_fm).cos()
    };
    let values_data = (0..num_samples)
        .map(|i| {
            ((2.0 * f32::consts::PI * (freq / fsample_rate) * (i as f32)
                + modulator_index * shift(i as f32, modulator_freq, fsample_rate)
                + phase)
                .sin())
        })
        .collect();

    return values_data;
}

#[derive(PartialEq)]
struct Spectrum {
    values: Vec<f32>,
    window_size: usize,
    step_size: usize,
    time: f32,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            window_size: 512,
            step_size: 512,
            time: 0.0,
        }
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.ctx().request_repaint();
        self.time += ui.input().unstable_dt.at_most(1.0 / 30.0) as f32 * 0.1;
        self.bar_plot(ui)
    }

    fn bar_plot(&mut self, ui: &mut Ui) -> Response {
        let phase_fm = self.time;
        self.values = fm_data_signal_generator(512, phase_fm);
        let mut chart = BarChart::new(
            (0..512)
                .step_by(1)
                .map(|x| (x as f64, self.values[x] as f64))
                .map(|(x, f)| Bar::new(x, f.abs()).width(0.01))
                .collect(),
        )
        .color(Color32::LIGHT_BLUE)
        .horizontal();

        Plot::new("Spectrum Demo")
            .legend(Legend::default())
            //            .clamp_grid(true)
            .show(ui, |plot_ui| plot_ui.bar_chart(chart))
            .response
    }
}

pub struct SpectrogramGui {
    spectrum: Spectrum,
    ringbuffer_left_out: Option<Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>>,
    ringbuffer_right_out:
        Option<Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>>,
}

impl Default for SpectrogramGui {
    fn default() -> Self {
        Self {
            ringbuffer_left_out: None,
            ringbuffer_right_out: None,
            spectrum: Spectrum::default(),
        }
    }
}
impl SpectrogramGui {
    pub fn set_ringbuffer(
        &mut self,
        mut ringbuffer_left_out: Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>,
        mut ringbuffer_right_out: Consumer<
            f32,
            Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>,
        >,
    ) {
        self.ringbuffer_left_out = Some(ringbuffer_left_out);
        self.ringbuffer_right_out = Some(ringbuffer_right_out);
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
