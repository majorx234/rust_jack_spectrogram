use std::{collections::VecDeque, default};

pub struct SpectrumQueue {
    pub data: VecDeque<Vec<f32>>,
    pub size: usize,
}

impl SpectrumQueue {
    pub fn new(size: usize) -> Self {
        SpectrumQueue {
            data: VecDeque::new(),
            size,
        }
    }

    pub fn push(&mut self, spectrum: Vec<f32>) {
        self.data.push_back(spectrum);

        while self.data.len() > self.size {
            self.data.pop_front();
        }
    }

    pub fn pop(&mut self) -> Option<Vec<f32>> {
        self.data.pop_front()
    }
}
