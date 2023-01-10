use eframe::egui;

pub struct SpectrogramGui {}

impl Default for SpectrogramGui {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for SpectrogramGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SpectrogramGui");
            ui.vertical(|ui| {
                ui.label("Test ");
            });
        });
    }
}
