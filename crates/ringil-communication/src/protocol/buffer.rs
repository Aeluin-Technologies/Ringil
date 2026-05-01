use std::collections::VecDeque;

use crate::protocol::pb::{PriorityLevel, RingilFrame, ringil_frame::Payload};

/// Priority-aware buffer.
pub struct PriorityFrameBuffer {
    critical_q: VecDeque<RingilFrame>,
    high_q: VecDeque<RingilFrame>,
    medium_q: VecDeque<RingilFrame>,
    low_q: VecDeque<RingilFrame>,
    max_len: usize,
    current_len: usize,
}

impl PriorityFrameBuffer {
    /// Create a new [`PriorityFrameBuffer`].
    pub fn new(max_len: usize) -> Self {
        Self {
            critical_q: VecDeque::new(),
            high_q: VecDeque::new(),
            medium_q: VecDeque::new(),
            low_q: VecDeque::new(),
            max_len,
            current_len: 0,
        }
    }

    /// Extract priority from frame payload type or threat level.
    fn get_priority(frame: &RingilFrame) -> PriorityLevel {
        match &frame.payload {
            Some(Payload::Perception(p)) => PriorityLevel::try_from(p.threat)
                .unwrap_or(PriorityLevel::Medium),
            Some(Payload::Consensus(_)) | Some(Payload::NavGoal(_)) => {
                PriorityLevel::High
            },
            Some(Payload::Mapping(_)) => PriorityLevel::Medium,
            Some(Payload::Status(_)) | None => PriorityLevel::Low,
        }
    }

    /// Push frame into the appropriate queue with overflow eviction.
    pub fn push(&mut self, frame: RingilFrame) {
        let priority = Self::get_priority(&frame);

        // Evict lower priority frames if buffer is full.
        if self.current_len >= self.max_len {
            if !self.low_q.is_empty() {
                self.low_q.pop_front();
            } else if !self.medium_q.is_empty()
                && priority >= PriorityLevel::High
            {
                self.medium_q.pop_front();
            } else if priority <= PriorityLevel::Medium {
                return; // Drop new frame: no lower priority to evict.
            } else {
                // Buffer saturated with High/Critical: evict oldest of same priority.
                match priority {
                    PriorityLevel::High => {
                        self.high_q.pop_front();
                    },
                    PriorityLevel::Critical => {
                        self.critical_q.pop_front();
                    },
                    _ => {},
                }
            }
            self.current_len -= 1;
        }

        match priority {
            PriorityLevel::Critical => self.critical_q.push_back(frame),
            PriorityLevel::High => self.high_q.push_back(frame),
            PriorityLevel::Medium => self.medium_q.push_back(frame),
            PriorityLevel::Low => self.low_q.push_back(frame),
        }
        self.current_len += 1;
    }

    /// Pop the highest priority frame available (Strict Priority Scheduling).
    pub fn pop(&mut self) -> Option<RingilFrame> {
        let frame = self
            .critical_q
            .pop_front()
            .or_else(|| self.high_q.pop_front())
            .or_else(|| self.medium_q.pop_front())
            .or_else(|| self.low_q.pop_front());

        if frame.is_some() {
            self.current_len -= 1;
        }
        frame
    }

    pub fn is_empty(&self) -> bool {
        self.current_len == 0
    }
}
