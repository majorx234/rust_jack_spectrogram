mod spectrogram_gui;
use spectrogram_gui::SpectrogramGui;
mod jackprocess;
use jackprocess::start_jack_thread;

fn main() {
    let spectrogram_app = SpectrogramGui::default();
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(600.0, 300.0);
    options.initial_window_size = Some(window_size);
    let jack_thread = start_jack_thread();
    eframe::run_native(
        "SpectrogramGui",
        options,
        Box::new(|_cc| Box::new(spectrogram_app)),
    );
    jack_thread.join().unwrap();
}
