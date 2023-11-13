use crate::fifo_queue::FifoQueue;
use crate::spectrum_queue::SpectrumQueue;
use crate::stft::WindowType;
use crate::stft::STFT;
use ringbuf::Consumer;
use ringbuf::SharedRb;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

type ConsumerRbf32 = Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>;

enum FftMode {
    RustFFT,
    RealFFT,
}

pub struct StftHandler {
    ringbuffer_out: Option<ConsumerRbf32>,
    spectrum_queue: Arc<Mutex<SpectrumQueue>>,
    window_size: usize,
    step_size: usize,
    time: f32,
    stft: STFT<f32>,
    fft_mode: FftMode,
}

impl Default for StftHandler {
    fn default() -> Self {
        Self {
            ringbuffer_out: None,
            spectrum_queue: Arc::new(Mutex::new(SpectrumQueue::new(2048))),
            window_size: 512,
            step_size: 256,
            time: 0.0,
            stft: STFT::new(WindowType::Hanning, 1024, 1024),
            fft_mode: FftMode::RustFFT,
        }
    }
}

impl StftHandler {
    // ToDo improove in future
    pub fn new(ringbuffer_out: ConsumerRbf32) -> Self {
        Self {
            ringbuffer_out: Some(ringbuffer_out),
            spectrum_queue: Arc::new(Mutex::new(SpectrumQueue::new(2048))),
            window_size: 512,
            step_size: 256,
            time: 0.0,
            stft: STFT::new(WindowType::Hanning, 1024, 512),
            fft_mode: FftMode::RustFFT,
        }
    }

    pub fn run(&mut self) {
        match &mut self.ringbuffer_out {
            Some(ringbuffer_out) => {
                while ringbuffer_out.len() > 512 {
                    let mut values: Vec<f32> = vec![0.0; 512];
                    let mut tmp_vec: Vec<f32> = vec![0.0; 1024];
                    if ringbuffer_out.len() >= self.window_size {
                        let (older_audio, newer_audio) = ringbuffer_out.as_slices();
                        if older_audio.len() >= self.window_size {
                            tmp_vec[..self.window_size]
                                .copy_from_slice(&older_audio[..self.window_size]);
                        } else {
                            tmp_vec[..older_audio.len()].copy_from_slice(&older_audio[..]);
                            tmp_vec[older_audio.len()..self.window_size].copy_from_slice(
                                &newer_audio[..self.window_size - older_audio.len()],
                            );
                        }
                        ringbuffer_out.skip(self.step_size);
                        match self.fft_mode {
                            FftMode::RustFFT => {
                                self.stft.compute_column(&mut tmp_vec, &mut values);
                            }
                            FftMode::RealFFT => {
                                // dummy implementation
                                // ToDo call RealFFT
                                self.stft.compute_column(&mut tmp_vec, &mut values);
                            }
                        }

                        self.spectrum_queue.lock().expect("Unlock").push(values);
                    }
                }
            }
            None => (),
        }
    }

    pub fn get_spectrum(&mut self) -> Vec<Vec<f32>> {
        // ToDo: return tuble
        let mut spec_vec = Vec::new();
        while let Some(spectrum) = self.spectrum_queue.lock().expect("Unlock").pop() {
            spec_vec.push(spectrum);
        }
        return spec_vec;
    }
}
