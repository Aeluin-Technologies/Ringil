mod config;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
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

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!("ringil_daemon={level},ringil_perception={level}").into()
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let _config = config::AppConfig::load_from_file("config.toml")?;
    tracing::info!("configuration loaded");

    let (_event_tx, mut event_rx) = mpsc::channel::<InstinctEvent>(32);

    while let Some(event) = event_rx.recv().await {
        tracing::debug!(?event, "received event");
    }

    Ok(())
}
