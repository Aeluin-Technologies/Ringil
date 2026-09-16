use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "configuration is parsed but not yet consumed by the bridge"
)]
pub struct AppConfig {
    pub target: TargetConfig,
    pub avoidance: AvoidanceConfig,
    pub controller: ControllerConfig,
    pub vision: VisionConfig,
    pub communication: CommConfig,
    pub simulation: SimConfig,
    pub offboard: OffboardConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "target settings are deserialized for the bridge"
)]
pub struct TargetConfig {
    pub r#type: String,
    pub embedding: Vec<f32>,
    pub threshold: f32,
    pub coordinates: Option<Coordinates>,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "target coordinates are deserialized for the bridge"
)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "avoidance settings are deserialized for the bridge"
)]
pub struct AvoidanceConfig {
    pub person_safe_distance: f32,
    pub safe_distance: f32,
    pub max_yaw_rate: f32,
    pub repulse_gain: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "controller settings are deserialized for the bridge"
)]
pub struct ControllerConfig {
    pub p_gain_advance: f32,
    pub p_gain_yaw: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "vision settings are deserialized for the bridge"
)]
pub struct VisionConfig {
    pub video_src: String,
    pub resolution: [u32; 2],
    pub frame_rate: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "communication settings are deserialized for the bridge"
)]
pub struct CommConfig {
    pub mavlink_url: String,
    pub heartbeat_rate: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "simulation settings are deserialized for the bridge"
)]
pub struct SimConfig {
    pub use_sim_time: bool,
    pub timeout_connect: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[expect(
    dead_code,
    reason = "offboard settings are deserialized for the bridge"
)]
pub struct OffboardConfig {
    pub failsafe_on_loss: bool,
    pub command_freq: f32,
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        config.try_deserialize()
    }
}
