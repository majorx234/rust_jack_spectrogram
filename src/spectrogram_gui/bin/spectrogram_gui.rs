use eframe::egui;
use eframe::egui::{lerp, Color32, Rgba, TextureHandle};
use egui::plot::{GridInput, GridMark};
use egui::*;
use plot::{
    Arrows, Bar, BarChart, CoordinatesFormatter, Corner, HLine, Legend, Line, LineStyle,
    MarkerShape, Plot, PlotImage, Points, Polygon, Text, VLine,
};
use spectrogram_lib::stft_handler::StftHandler;
use std::f32;
use std::f64::consts::TAU;
use std::ops::RangeInclusive;

//#[derive(PartialEq)]
struct Spectrum {
    pub last_vec: Vec<f32>,
    pub tex_mngr: TextureManager,
    pub texture_id: Option<(egui::Vec2, egui::TextureId)>,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            last_vec: vec![0.0; 512],
            tex_mngr: TextureManager(vec![Color32::from_rgb(255, 255, 255); 1024 * 512]),
            texture_id: None,
        }
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui) {

        //        ui.horizontal(|ui| {});
        // self.bar_plot(ui);
    }

    fn bar_plot(&mut self, ui: &mut Ui) -> Response {
        let mut chart = BarChart::new(
            (0..512)
                .step_by(1)
                .map(|x| (x as f64, self.last_vec[x] as f64))
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

    fn set_values(&mut self, ctx: &egui::Context, specs: Vec<Vec<f32>>) {
        let mut int_specs: Vec<Vec<u8>> = Vec::new();
        for spec in specs.iter() {
            let int_spec = spec
                .iter()
                .map(|&value| (255.0 * value.abs()) as u8)
                .collect();
            int_specs.push(int_spec);
        }
        if specs.len() > 0 {
            for (last, new) in self.last_vec.iter_mut().zip(&specs[specs.len() - 1]) {
                *last = *new;
            }
        }
        self.texture_id = Some((
            egui::Vec2::new(512.0, 512.0),
            (&(self
                .tex_mngr
                .get_spectrogram_texture(ctx, int_specs, 512, 512)))
                .into(),
        ));
    }
}

fn value_to_rgb(value: u8) -> egui::epaint::Color32 {
    let min_value = 0.0;
    let max_value = 255.0;
    Color32::from_rgb(0, 0, 0)
}

#[derive(Default)]
struct TextureManager(Vec<egui::epaint::Color32>);

impl TextureManager {
    pub fn get_spectrogram_texture(
        &mut self,
        ctx: &egui::Context,
        specs: Vec<Vec<u8>>,
        width: usize,
        height: usize,
    ) -> TextureHandle {
        let mut new_cols = specs
            .into_iter()
            .flatten()
            .map(|x| egui::epaint::Color32::from_gray(x))
            .collect::<Vec<Color32>>();
        self.0.append(&mut new_cols);
        let current_length = self.0.len();
        if current_length > width * height {
            let drain_count = current_length - width * height;
            self.0.drain(0..drain_count);
        }
        // maybe return an option
        // or handle if pixels.len() < width*height
        let pixels: Vec<egui::epaint::Color32> = self.0.clone();
        ctx.load_texture(
            "color_test_gradient",
            egui::ColorImage {
                size: [width, height],
                pixels,
            },
        )
    }
}

pub struct SpectrogramGui {
    spectrum: Spectrum,
    stft_handler: Option<StftHandler>,
}

impl SpectrogramGui {
    pub fn new(stft_handler: StftHandler) -> Self {
        Self {
            spectrum: Spectrum::default(),
            stft_handler: Some(stft_handler),
        }
    }
}
impl Default for SpectrogramGui {
    fn default() -> Self {
        Self {
            spectrum: Spectrum::default(),
            stft_handler: None,
        }
    }
}

impl eframe::App for SpectrogramGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // get data
            let mut spectrum = Vec::new();
            if let Some(stft_handler) = &mut self.stft_handler {
                stft_handler.run();
                spectrum = stft_handler.get_spectrum();
            }

            self.spectrum.set_values(ctx, spectrum);

            if let Some((size, texture_id)) = self.spectrum.texture_id {
                ui.add(egui::Image::new(texture_id, size));
                ctx.request_repaint();
            }
            //self.spectrum.ui(ui);
            //ui.vertical(|ui| {
        });
    }
}
