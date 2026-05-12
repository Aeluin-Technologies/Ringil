use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ObjectClass {
    Person = 0,
    Vehicle = 1,
    Tree = 2,
    Building = 3,
    Unknown = 255,
}

impl From<i64> for ObjectClass {
    fn from(val: i64) -> Self {
        match val {
            0 => ObjectClass::Person,
            1 => ObjectClass::Vehicle,
            2 => ObjectClass::Tree,
            3 => ObjectClass::Building,
            _ => ObjectClass::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct OrientedBoundingBox {
    pub cx: f32,
    pub cy: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
}

impl OrientedBoundingBox {
    pub fn to_tlwh(&self) -> [f32; 4] {
        let cos_a = self.angle.cos().abs();
        let sin_a = self.angle.sin().abs();
        let w = self.width * cos_a + self.height * sin_a;
        let h = self.width * sin_a + self.height * cos_a;
        [self.cx - w / 2.0, self.cy - h / 2.0, w, h]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstinctEvent {
    ObstacleDetected {
        id: u64,
        class: ObjectClass,
        obb: OrientedBoundingBox,
        confidence: f32,
    },
    PersonTracked {
        track_id: u64,
        obb: OrientedBoundingBox,
    },
    PersonIdentityExtracted {
        track_id: u64,
        embedding: Vec<f32>,
    },
    TrackLost(u64),
}
