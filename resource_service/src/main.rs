use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use resource_service::{AppConfig, ResourceService, api, create_pool};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));
    if let Err(err) = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .try_init()
    {
        eprintln!("failed to initialize tracing: {err}");
    }
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

    let config = AppConfig::from_env();
    init_tracing(&config.log_level);

    let pool = create_pool(&config)?;
    let service = Arc::new(ResourceService::new(pool, config.clone()));
    let app: Router = api::router(service).layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;

    info!("resource_service listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
