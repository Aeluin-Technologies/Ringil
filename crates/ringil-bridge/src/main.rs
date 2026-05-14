mod config;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
use ringil_mavlink::{FlightDirector, MavlinkController};
use ringil_perception::InstinctEvent;
use tokio::sync::mpsc;
use tracing_subscriber::{
    EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
    let level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("ringil_daemon={level},ringil_perception={level},ringil_mavlink={level}").into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::load_from_file("config.toml")?;
    tracing::info!("configuration loaded");

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
        tracing::debug!(?event, "received event");
        director.process_event(event).await?;
    }

    Ok(())
}
