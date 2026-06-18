use std::{sync::Arc};
use tokio::net::TcpListener;
use anyhow::Result;
use axum::Router;
use rmcp::{ServerHandler, handler::server::tool::ToolRouter, model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo}, tool_handler, tool_router, transport::{StreamableHttpServerConfig, StreamableHttpService, streamable_http_server::session::local::LocalSessionManager}};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::server::config::ServerConfig;

#[derive(Debug, Clone)]
pub struct RoadmapServer {
    pub config: ServerConfig,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RoadmapServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let tool_router = Self::tool_router();
        Self { config, tool_router }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Roadmap Server at: {addr}");

        let config = StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()), 
            Arc::new(LocalSessionManager::default()), 
            config,
        );

        let app = Router::new().nest_service("/mcp", service);

        let listener = TcpListener::bind(&addr).await?;

        axum::serve(listener, app).await?;
        Ok(())
    }
}

impl Drop for RoadmapServer {
    fn drop(&mut self) {
        info!("\tShutting down Roadmap Server");
    }
}

#[tool_handler]
impl ServerHandler for RoadmapServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let ins = format!("Generating and Planning Roadmap for topic, skill");

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();

        let info = Implementation::default()
            .with_description("Roadmap for topics, skill")
            .with_title("Roadmap Server")
            .with_website_url(self.config.url.clone());

        let sv_info = ServerInfo::new(capabilities)
            .with_instructions(ins)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(info);

        sv_info
    }
}