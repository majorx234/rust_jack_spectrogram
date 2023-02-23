use crate::fifo_queue::FifoQueue;
use crate::spectrum_queue;
use crate::spectrum_queue::SpectrumQueue;
use crate::stft;
use crate::stft::WindowType;
use crate::stft::STFT;
use ringbuf::Consumer;
use ringbuf::SharedRb;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

type ConsumerRbf32 = Consumer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>;

struct StftHandler {
    ringbuffer_left_out: Option<ConsumerRbf32>,
    ringbuffer_right_out: Option<ConsumerRbf32>,
    spectrum_queue: Arc<Mutex<SpectrumQueue>>,
    window_size: usize,
    step_size: usize,
    time: f32,
    stft: STFT<f32>,
}

impl Default for StftHandler {
    fn default() -> Self {
        Self {
            ringbuffer_left_out: None,
            ringbuffer_right_out: None,
            spectrum_queue: Arc::new(Mutex::new(SpectrumQueue::new(2048))),
            window_size: 512,
            step_size: 256,
            time: 0.0,
            stft: STFT::new(WindowType::Hanning, 1024, 1024),
        }
    }
}

impl StftHandler {
    pub fn set_ringbuffer(
        &mut self,
        mut ringbuffer_left_out: ConsumerRbf32,
        mut ringbuffer_right_out: ConsumerRbf32,
    ) {
        self.ringbuffer_left_out = Some(ringbuffer_left_out);
        self.ringbuffer_right_out = Some(ringbuffer_right_out);
    }

    pub fn run(&mut self) {
        match &mut self.ringbuffer_left_out {
            Some(ringbuffer_left_out) => {
                while ringbuffer_left_out.len() > 512 {
                    let mut values: Vec<f32> = vec![0.0; 512];
                    let mut tmp_vec: Vec<f32> = vec![0.0; 512];
                    if ringbuffer_left_out.len() >= self.window_size {
                        let (older_audio, newer_audio) = ringbuffer_left_out.as_slices();
                        if older_audio.len() >= self.window_size {
                            tmp_vec[..self.window_size]
                                .copy_from_slice(&older_audio[..self.window_size]);
                        } else {
                            tmp_vec[..older_audio.len()].copy_from_slice(&older_audio[..]);
                            tmp_vec[older_audio.len()..self.window_size].copy_from_slice(
                                &newer_audio[..self.window_size - older_audio.len()],
                            );
                        }
                        ringbuffer_left_out.skip(self.step_size);
                        self.stft.compute_column(&mut tmp_vec, &mut values);
                        self.spectrum_queue.lock().expect("Unlock").push(values);
                    }
                }
            }
            None => (),
        }
        match &mut self.ringbuffer_right_out {
            Some(ringbuffer_right_out) => {
                ringbuffer_right_out.skip(self.step_size);
            }
            None => (),
        }
    }
}
