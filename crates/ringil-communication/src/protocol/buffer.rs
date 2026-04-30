use std::collections::VecDeque;

use crate::protocol::pb::RingilFrame;

/// Bufferizer for LoRa/mesh-constrained links.
pub struct FrameBuffer {
    queue: VecDeque<RingilFrame>,
    max_len: usize,
}

impl FrameBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push(&mut self, frame: RingilFrame) {
        if self.queue.len() >= self.max_len {
            self.queue.pop_front(); // Discard oldest on overflow.
        }
        self.queue.push_back(frame);
    }

    pub fn pop(&mut self) -> Option<RingilFrame> {
        self.queue.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
