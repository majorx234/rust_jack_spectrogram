mod spectrogram_gui;
use spectrogram_gui::SpectrogramGui;
mod jackprocess;
use jackprocess::start_jack_thread;
use ringbuf::HeapRb;

fn main() {
    let ringbuffer_left = HeapRb::<f32>::new(96000);
    let ringbuffer_right = HeapRb::<f32>::new(96000);

    let (mut ringbuffer_left_in, mut ringbuffer_left_out) = ringbuffer_left.split();
    let (mut ringbuffer_right_in, mut ringbuffer_right_out) = ringbuffer_right.split();
    let jack_thread = start_jack_thread(ringbuffer_left_in, ringbuffer_right_in);

    let mut spectrogram_app = SpectrogramGui::default();
    spectrogram_app.set_ringbuffer(ringbuffer_left_out, ringbuffer_right_out);
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(600.0, 300.0);
    options.initial_window_size = Some(window_size);

    eframe::run_native(
        "SpectrogramGui",
        options,
        Box::new(|_cc| Box::new(spectrogram_app)),
    );
    jack_thread.join().unwrap();
}
