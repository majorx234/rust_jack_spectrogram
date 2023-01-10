mod spectrogram_gui;
use spectrogram_gui::SpectrogramGui;

fn main() {
    let spectrogram_app = SpectrogramGui::default();
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(600.0, 300.0);
    options.initial_window_size = Some(window_size);
    eframe::run_native(
        "SpectrogramGui",
        options,
        Box::new(|_cc| Box::new(spectrogram_app)),
    );
}
