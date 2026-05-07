
use std::{env, time::Duration};
use tokio::net::TcpListener;

use anyhow::{Result};
use axum::Router;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

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

    // Config ENV
    dotenv::from_path("../.env").ok();

    let log_level = &std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    let host = env::var("ORCHESTRATOR_HOST").unwrap_or("0.0.0.1".into());
    let port = env::var("ORCHESTRATOR_PORT").unwrap_or("3001".into());
    let addr = format!("{}:{}", host, port);

    // Init Logging
    init_tracing(log_level);

    println!("Hello, world!");

    // let app = Router::new();
    info!("Orchestrator Server is running on: {}", &addr);
    // let listener = TcpListener::bind(addr).await?;
    // axum::serve(listener, app).await?;

    let config = rmcp::transport::sse_server::SseServerConfig {
        bind: addr.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/mcp".to_string(),
        ct: CancellationToken::new(),
        sse_keep_alive: Some(Duration::from_secs(60))
    };
    let transport = rmcp::transport::SseServer::new(config);

    Ok(())
}
