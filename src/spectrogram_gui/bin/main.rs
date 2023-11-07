mod spectrogram_gui;
use spectrogram_gui::SpectrogramGui;
mod jackprocess;
use jackprocess::start_jack_thread;
use ringbuf::HeapRb;
use spectrogram_lib::stft_handler::StftHandler;

fn main() {
    let ringbuffer_left = HeapRb::<f32>::new(96000);
    let ringbuffer_right = HeapRb::<f32>::new(96000);

    let (mut ringbuffer_left_in, mut ringbuffer_left_out) = ringbuffer_left.split();
    let (mut ringbuffer_right_in, mut ringbuffer_right_out) = ringbuffer_right.split();
    let stft_handler = StftHandler::new(ringbuffer_left_out);
    let jack_thread = start_jack_thread(ringbuffer_left_in, ringbuffer_right_in);

    let mut stft_handlers = Vec::new();
    stft_handlers.push(stft_handler);
    let mut spectrogram_app = SpectrogramGui::new(stft_handlers);
    //    spectrogram_app.set_ringbuffer(ringbuffer_left_out, ringbuffer_right_out);
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(525.0, 530.0);
    options.initial_window_size = Some(window_size);

    eframe::run_native(
        "SpectrogramGui",
        options,
        Box::new(|_cc| Box::new(spectrogram_app)),
    );
    jack_thread.join().unwrap();
}
