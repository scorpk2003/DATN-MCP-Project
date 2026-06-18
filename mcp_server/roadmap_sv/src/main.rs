mod server;
use std::env;

use anyhow::Result;
use server::*;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

use crate::server::server::RoadmapServer;

fn init_tracing(level: &str) {
    let filter = match EnvFilter::try_new(level) {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };

    match fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_timer(fmt::time::time())
        .try_init()
    {
        Ok(_) => {
            info!("Tracing initialized successfully!!!");
        }
        Err(e) => {
            tracing::error!("Failed to initialize tracing: {}", e);
            eprint!("Failed to initialize tracing: {}", e);
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_path("../../.env").ok();
    let log_level = &env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    init_tracing(log_level);

    let server = RoadmapServer::new();
    server.run().await?;

    Ok(())
}
