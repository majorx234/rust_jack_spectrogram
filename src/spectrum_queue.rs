use crate::fifo_queue::FifoQueue;
use std::{collections::VecDeque, default};

pub struct SpectrumQueue {
    pub data: VecDeque<Vec<f32>>,
    pub size: usize,
}

impl FifoQueue<Vec<f32>> for SpectrumQueue {
    fn new(size: usize) -> Self {
        SpectrumQueue {
            data: VecDeque::new(),
            size,
        }
    }

    fn push(&mut self, new_data: Vec<f32>) {
        self.data.push_back(new_data);
        while self.data.len() > self.size {
            self.data.pop_front();
        }
    }

    fn pop(&mut self) -> Option<Vec<f32>> {
        self.data.pop_front()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
