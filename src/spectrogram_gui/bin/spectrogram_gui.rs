use eframe::egui;
use ringbuf::Consumer;
use ringbuf::SharedRb;
use spectrogram_lib::stft;
use spectrogram_lib::stft::WindowType;
use spectrogram_lib::stft::STFT;
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

type ConsumerRbf32 = Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>;

//#[derive(PartialEq)]
struct Spectrum {
    values: Vec<f32>,
    window_size: usize,
    step_size: usize,
    time: f32,
    stft: STFT<f32>,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            values: vec![0.0; 512],
            window_size: 1024,
            step_size: 1024,
            time: 0.0,
            stft: STFT::new(WindowType::Hanning, 1024, 1024),
        }
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.ctx().request_repaint();
        self.bar_plot(ui)
    }

    fn bar_plot(&mut self, ui: &mut Ui) -> Response {
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
            .data_aspect(4.0 / 512.0)
            .include_x(2.0)
            .include_y(512.0)
            .width(100.0)
            .show(ui, |plot_ui| plot_ui.bar_chart(chart))
            .response
    }
    fn set_values(&mut self, new_values: &[f32]) {
        for (value, new_value) in self.values.iter_mut().zip(new_values.iter()) {
            *value = *new_value;
        }
    }
}

pub struct SpectrogramGui {
    spectrum: Spectrum,
    ringbuffer_left_out: Option<ConsumerRbf32>,
    ringbuffer_right_out: Option<ConsumerRbf32>,
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
        mut ringbuffer_left_out: ConsumerRbf32,
        mut ringbuffer_right_out: ConsumerRbf32,
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
                match &mut self.ringbuffer_left_out {
                    Some(ringbuffer_left_out) => {
                        //while
                        let mut tmp_vec: Vec<f32> = vec![0.0; 1024];
                        if ringbuffer_left_out.len() >= self.spectrum.window_size {
                            let (older_audio, newer_audio) = ringbuffer_left_out.as_slices();
                            if older_audio.len() >= self.spectrum.window_size {
                                tmp_vec[..self.spectrum.window_size]
                                    .copy_from_slice(&older_audio[..self.spectrum.window_size]);
                            } else {
                                tmp_vec[..older_audio.len()].copy_from_slice(&older_audio[..]);
                                tmp_vec[older_audio.len()..self.spectrum.window_size]
                                    .copy_from_slice(
                                        &newer_audio
                                            [..self.spectrum.window_size - older_audio.len()],
                                    );
                            }
                            ringbuffer_left_out.skip(self.spectrum.step_size);
                            self.spectrum
                                .stft
                                .compute_column(&mut tmp_vec, &mut self.spectrum.values);
                        }
                    }
                    None => (),
                }
                match &mut self.ringbuffer_right_out {
                    Some(ringbuffer_right_out) => {
                        ringbuffer_right_out.skip(self.spectrum.step_size);
                    }
                    None => (),
                }
                self.spectrum.ui(ui);
                ui.label("Test ");
            });
        });
    }
}
