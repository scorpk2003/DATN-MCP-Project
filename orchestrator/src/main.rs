use std::{env, sync::Arc};
use tokio::net::TcpListener;

use anyhow::Result;
use axum::{
    Router,
    http::HeaderValue,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod agent;
mod api;
mod mcp;

pub use agent::*;
pub use api::*;
pub use mcp::*;

pub fn agent_testing_enabled() -> bool {
    env_bool("AGENT_TESTING", false)
}

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

fn load_env() {
    dotenv::dotenv().ok();
    for path in ["../.env", "../../.env", "../../../.env"] {
        dotenv::from_path(path).ok();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Config ENV
    load_env();

    let log_level = &std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    let host = env::var("ORCHESTRATOR_HOST").unwrap_or("0.0.0.0".into());
    let port = env::var("ORCHESTRATOR_PORT").unwrap_or("3001".into());
    let addr = format!("{}:{}", host, port);

    // Init Logging
    init_tracing(log_level);

    // Load MCP server configuration and initial readiness snapshot.
    let clients = ServerConfig::all_from_env();
    info!("Loaded MCP servers: {:?}", clients);
    let registry = McpRegistry::build(&clients).await;
    let state = Arc::new(AppState {
        clients,
        registry,
        sessions: tokio::sync::Mutex::new(std::collections::HashMap::new()),
    });

    // Config Server
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/mcp/tools", get(mcp_tools_handler))
        .route("/agent/run", post(agent_handler))
        .route("/agent/resume", post(agent_resume_handler))
        .with_state(state)
        .layer(cors_layer_from_env());
    let listener = TcpListener::bind(addr.clone()).await?;

    // Server Running
    info!("Orchestrator Server is running on: {}", &addr);
    axum::serve(listener, app).await?;

    Ok(())
}

fn cors_layer_from_env() -> CorsLayer {
    let allowed_origins = env::var("ORCHESTRATOR_CORS_ORIGINS")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match allowed_origins.as_deref() {
        None | Some("*") => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
        Some(origins) => parse_cors_origins(origins)
            .map(|origins| {
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_methods(Any)
                    .allow_headers(Any)
            })
            .unwrap_or_else(|_| CorsLayer::permissive()),
    }
}

fn parse_cors_origins(
    origins: &str,
) -> Result<Vec<HeaderValue>, axum::http::header::InvalidHeaderValue> {
    origins
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(HeaderValue::from_str)
        .collect()
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}
