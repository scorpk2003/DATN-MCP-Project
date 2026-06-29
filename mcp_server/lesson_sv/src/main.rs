use std::env;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod contract;
mod domain;
mod error;
mod server;
mod services;

use crate::server::LessonServer;

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    match fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_timer(fmt::time::time())
        .try_init()
    {
        Ok(_) => info!("Tracing initialized successfully!!!"),
        Err(e) => {
            tracing::error!("Failed to initialize tracing: {}", e);
            eprint!("Failed to initialize tracing: {}", e);
        }
    };
}

fn load_env() {
    dotenv::dotenv().ok();
    for path in ["../.env", "../../.env", "../../../.env"] {
        dotenv::from_path(path).ok();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    load_env();
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    init_tracing(&log_level);

    LessonServer::new().run().await
}
