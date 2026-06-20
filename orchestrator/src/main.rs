use std::{env, sync::Arc};
use tokio::net::TcpListener;

use anyhow::Result;
use axum::{Router, routing::post};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod agent;
mod api;
mod mcp;

pub use agent::*;
pub use api::*;
pub use mcp::*;

const AGENT_TESTING: bool = true;

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
    // Config ENV
    dotenv::from_path("../.env").ok();

    let log_level = &std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    let host = env::var("ORCHESTRATOR_HOST").unwrap_or("0.0.0.1".into());
    let port = env::var("ORCHESTRATOR_PORT").unwrap_or("3001".into());
    let addr = format!("{}:{}", host, port);

    // Init Logging
    init_tracing(log_level);

    // Connect MCP Server
    let roadmap_sv = ServerConfig::new("roadmap");
    let lesson_sv = ServerConfig::new("lesson");
    let github_sv = ServerConfig::new("github");
    let figma_sv = ServerConfig::new("figma");

    let clients = vec![roadmap_sv, lesson_sv, github_sv, figma_sv];
    let state = Arc::new(AppState { clients });

    // Config Server
    let app = Router::new()
        .route("/agent/run", post(agent_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());
    let listener = TcpListener::bind(addr.clone()).await?;

    // Server Running
    info!("Orchestrator Server is running on: {}", &addr);
    axum::serve(listener, app).await?;

    Ok(())
}
