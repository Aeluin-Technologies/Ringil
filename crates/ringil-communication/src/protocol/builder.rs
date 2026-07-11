use std::time::{SystemTime, UNIX_EPOCH};

use passerelle::ringil::swarm::v1::{
    ConsensusNode, GeoPoint, MappingUpdate, PerceptionEvent, RingilFrame,
    UnitStatus, ringil_frame::Payload,
};

pub struct FrameBuilder {
    frame: RingilFrame,
}

impl FrameBuilder {
    /// Initializes a new [`FrameBuilder`] with the current node_id.
    pub fn new(source_node_id: u32) -> Self {
        let timestamp_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        Self {
            frame: RingilFrame {
                source_node_id,
                timestamp_us,
                hop_limit: 3,
                payload: None,
            },
        }
    }

    pub fn with_hop_limit(mut self, limit: u32) -> Self {
        self.frame.hop_limit = limit;
        self
    }

    pub fn perception(mut self, event: PerceptionEvent) -> Self {
        self.frame.payload = Some(Payload::Perception(event));
        self
    }

    pub fn mapping(mut self, update: MappingUpdate) -> Self {
        self.frame.payload = Some(Payload::Mapping(update));
        self
    }

    pub fn consensus(mut self, node: ConsensusNode) -> Self {
        self.frame.payload = Some(Payload::Consensus(node));
        self
    }

    pub fn status(mut self, status: UnitStatus) -> Self {
        self.frame.payload = Some(Payload::Status(status));
        self
    }

    pub fn nav_goal(mut self, goal: GeoPoint) -> Self {
        self.frame.payload = Some(Payload::NavGoal(goal));
        self
    }

    pub fn build(self) -> RingilFrame {
        self.frame
    }
}
