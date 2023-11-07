use eframe::egui;
use eframe::egui::{lerp, Color32, Rgba, TextureHandle};
use egui::plot::{GridInput, GridMark};
use egui::*;
use itertools::izip;
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
    pub texture_ids: Vec<Option<(egui::Vec2, egui::TextureId)>>,
}

impl Default for Spectrum {
    fn default() -> Self {
        // TODO review this, maybe size of channels is important so no Default is possible
        let texture_queue = vec![Color32::from_rgb(255, 255, 255); 1024 * 512];
        let mut texture_queues = Vec::new();
        let texture_ids = vec![None; 1];
        texture_queues.push(texture_queue);
        Self {
            last_vec: vec![0.0; 512],
            tex_mngr: TextureManager(texture_queues, texture_ids),
            texture_ids: Vec::new(),
        }
    }
}

impl Spectrum {
    fn ui(&mut self, ui: &mut Ui, spectrum_data: Vec<Vec<Vec<f32>>>) {
        self.set_values(ui.ctx(), spectrum_data);
        for texture_id in &self.texture_ids {
            if let Some((size, texture_id)) = *texture_id {
                ui.add(egui::Image::new(texture_id, size));
                ui.ctx().request_repaint();
            }
        }
        // ui.horizontal(|ui| {});
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

    fn set_values(&mut self, ctx: &egui::Context, specs_channels: Vec<Vec<Vec<f32>>>) {
        let mut int_specs_channels: Vec<Vec<Vec<u8>>> = Vec::new();
        for specs_channel in specs_channels.iter() {
            let mut int_specs: Vec<Vec<u8>> = Vec::new();
            for spec in specs_channel {
                let int_spec = spec
                    .iter()
                    .map(|&value| (255.0 * value.abs()) as u8)
                    .collect();
                int_specs.push(int_spec);
            }
            int_specs_channels.push(int_specs);
        }
        /* TODO last_vec need to be vec of vectors, not in use (needed only for barplot)
                for int_specs in int_specs_channels.iter().zip() {
                        if specs_channel.len() > 0 {
                                for (last, new) in self.last_vec.iter_mut().zip(&specs[specs.len() - 1]) {
                *last = *new;
            }
        }
        */
        self.tex_mngr
            .update_spectrogram_texture(ctx, int_specs_channels, 512, 512);
        for (texture_option, texture_id) in
            self.tex_mngr.1.iter_mut().zip(self.texture_ids.iter_mut())
        {
            if let Some(ref texture) = *texture_option {
                *texture_id = Some((egui::Vec2::new(512.0, 512.0), texture.into()));
            }
        }
    }
}

fn value_to_rgb(value: u8) -> egui::epaint::Color32 {
    let min_value = 0.0;
    let max_value = 255.0;
    Color32::from_rgb(0, 0, 0)
}

//#[derive(Default)]
struct TextureManager(Vec<Vec<egui::epaint::Color32>>, Vec<Option<TextureHandle>>);

impl TextureManager {
    pub fn update_spectrogram_texture(
        &mut self,
        ctx: &egui::Context,
        specs_channel: Vec<Vec<Vec<u8>>>,
        width: usize,
        height: usize,
    ) {
        for (specs, mut texture_queue, mut textures) in
            izip!(&specs_channel, &mut self.0, &mut self.1)
        {
            let mut new_cols = specs
                .into_iter()
                .flatten()
                .map(|x| egui::epaint::Color32::from_gray(*x))
                .collect::<Vec<Color32>>();
            texture_queue.append(&mut new_cols);
            let current_length = texture_queue.len();
            if current_length > width * height {
                let drain_count = current_length - width * height;
                texture_queue.drain(0..drain_count);
            }
            let pixels: Vec<egui::epaint::Color32> = texture_queue.clone();
            *textures = Some(ctx.load_texture(
                "color_test_gradient",
                egui::ColorImage {
                    size: [width, height],
                    pixels,
                },
            ));
        }

        // TODO handling of multiple texture handles
        // maybe return an option
        // or handle if pixels.len() < width*height
    }
}

pub struct SpectrogramGui {
    spectrum: Spectrum,
    stft_handler: Option<Vec<StftHandler>>,
}

impl SpectrogramGui {
    pub fn new(stft_handler: Vec<StftHandler>) -> Self {
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
                for stft_handle in stft_handler {
                    stft_handle.run();
                    spectrum.push(stft_handle.get_spectrum());
                }
            };
            self.spectrum.ui(ui, spectrum);
        });
    }
}
