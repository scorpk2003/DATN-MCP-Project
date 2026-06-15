use std::env;

use anyhow::Result;
use axum::{Router, routing::post};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod server;
use server::*;

mod provider;
use provider::*;

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
        .try_init() {
            Ok(_) => {
                info!("Tracing initialized successfully!!!");
            },
            Err(e) => {
                tracing::error!("Failed to initialize tracing: {}", e);
                eprint!("Failed to initialize tracing: {}", e);
            }
        };
}

#[tokio::main]
async fn main() -> Result<()> {
    // Start
    dotenv::from_path("../../.env").ok();
    let log_level = &env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    init_tracing(log_level);

    Ok(())
}
