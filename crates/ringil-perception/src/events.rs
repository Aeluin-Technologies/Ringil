use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ObjectClass {
    Person = 0,
    Bicycle = 1,
    Car = 2,
    Motorcycle = 3,
    Bus = 4,
    Truck = 5,
    Train = 6,
    Animal = 7,
    TrafficLight = 10,
    FireHydrant = 11,
    StopSign = 12,
    ParkingMeter = 13,
    Bench = 14,
    Tree = 15,
    Building = 16,
    Fence = 17,
    Pole = 20,
    PowerLine = 21,
    TrafficSign = 22,
    Wall = 23,
    Backpack = 30,
    Umbrella = 31,
    Handbag = 32,
    Suitcase = 33,
    TrashCan = 34,
    Unknown = 255,
}

impl From<i64> for ObjectClass {
    fn from(val: i64) -> Self {
        match val {
            0 => ObjectClass::Person,
            1 => ObjectClass::Bicycle,
            2 => ObjectClass::Car,
            3 => ObjectClass::Motorcycle,
            4 => ObjectClass::Bus,
            5 => ObjectClass::Truck,
            6 => ObjectClass::Train,
            7 => ObjectClass::Animal,
            10 => ObjectClass::TrafficLight,
            11 => ObjectClass::FireHydrant,
            12 => ObjectClass::StopSign,
            13 => ObjectClass::ParkingMeter,
            14 => ObjectClass::Bench,
            15 => ObjectClass::Tree,
            16 => ObjectClass::Building,
            17 => ObjectClass::Fence,
            20 => ObjectClass::Pole,
            21 => ObjectClass::PowerLine,
            22 => ObjectClass::TrafficSign,
            23 => ObjectClass::Wall,
            30 => ObjectClass::Backpack,
            31 => ObjectClass::Umbrella,
            32 => ObjectClass::Handbag,
            33 => ObjectClass::Suitcase,
            34 => ObjectClass::TrashCan,
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
