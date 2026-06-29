use std::sync::Arc;

use anyhow::Result;
use resource_service::{
    AppConfig, ResourceService, create_pool,
    worker::{WorkerMode, WorkerRunOptions, run_worker},
};
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

    let options = parse_args();
    let pool = create_pool(&config)?;
    let service = Arc::new(ResourceService::new(pool, config));

    info!(
        mode = ?options.mode,
        once = options.once,
        "resource worker starting"
    );
    run_worker(service, options).await?;
    Ok(())
}

fn parse_args() -> WorkerRunOptions {
    let mut mode = WorkerMode::All;
    let mut once = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                if let Some(value) = args.next() {
                    mode = parse_mode(&value);
                }
            }
            "--once" => once = true,
            _ => {}
        }
    }

    WorkerRunOptions { mode, once }
}

fn parse_mode(value: &str) -> WorkerMode {
    match value {
        "fetcher" => WorkerMode::Fetcher,
        "extractor" => WorkerMode::Extractor,
        "embedding" => WorkerMode::Embedding,
        "all" => WorkerMode::All,
        _ => WorkerMode::All,
    }
}
