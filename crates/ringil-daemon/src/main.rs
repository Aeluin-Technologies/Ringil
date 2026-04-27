mod config;

use anyhow::Result;
use ringil_instinct::InstinctEvent;
use ringil_mavlink::{FlightDirector, MavlinkController};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::AppConfig::load_from_file("config.toml")?;

    let mavlink =
        MavlinkController::connect(&config.communication.mavlink_url)?;
    let director = FlightDirector::new(
        mavlink,
        config.avoidance.person_safe_distance,
        config.avoidance.safe_distance,
        config.avoidance.max_yaw_rate,
        config.avoidance.repulse_gain,
        config.controller.p_gain_advance,
        config.controller.p_gain_yaw,
        config.vision.resolution,
    );

    let (_event_tx, mut event_rx) = mpsc::channel::<InstinctEvent>(32);

    while let Some(event) = event_rx.recv().await {
        director.process_event(event).await?;
    }

    Ok(())
}
