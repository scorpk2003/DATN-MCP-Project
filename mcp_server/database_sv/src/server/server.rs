use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use rmcp::{ErrorData, ServerHandler, handler::server::{tool::ToolRouter, wrapper::Parameters}, model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo}, tool, tool_handler, tool_router, transport::{StreamableHttpServerConfig, StreamableHttpService, streamable_http_server::session::local::LocalSessionManager}};
use serde_json::Value;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{VALID_TABLES, provider::SchemaProvider, schemas::{GetMultiTableSchema, GetTableSchema}, server::ServerConfig};


#[derive(Debug, Clone)]
pub struct DbServer {
    pub config: ServerConfig,
    pub provider: Arc<Mutex<SchemaProvider>>,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DbServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let provider = Arc::new(Mutex::new(SchemaProvider::default()));

        Self { config, provider, tool_router: Self::tool_router() }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Database Server at: {addr}");
        
        let config = StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new().nest_service("/mcp", service);

        let listener = TcpListener::bind(&addr).await?;

        info!("\tDatabase MCP Server Endpoint: http://{}/mcp", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    #[tool(description = "Infomation of multi schemas in database")]
    pub async fn get_multi_schema(&self, Parameters(params): Parameters<GetMultiTableSchema>)
    -> Result<CallToolResult, rmcp::ErrorData>
    {
        info!("\tCALL TOOL: [GET MULTI SCHEMA]");
        let invalid_tb: Option<String> = Some(
            params.table_names
                .iter()
                .filter(|tb| !VALID_TABLES.contains(&tb.as_str()))
                .cloned()
                .collect()
        );

        if let Some(ivl) = invalid_tb {
            warn!("\tInvalid Table: {ivl}");
        }

        let mut provider_locking = self.provider.lock().await;
        let result = provider_locking.get_multi_table_schema(params.table_names).await;
        info!("\tTOOL [GET MULTI SCHEMA] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Infomation of schema in Database")]
    pub async fn get_schema(&self, Parameters(param): Parameters<GetTableSchema>)
    -> Result<CallToolResult, ErrorData>
    {
        info!("\tCALL TOOL: [GET SCHEMA]");
        if !VALID_TABLES.contains(&param.table_name.as_str()) {
            return Ok(CallToolResult::error(vec![Content::text("Invalid schema")]));
        };

        let mut provider_locking = self.provider.lock().await;
        let schema = match provider_locking.get_table_schema(&param.table_name).await {
            Ok(db) => {
                info!("\tGet Table Information Success!!!!");
                Value::from(db)
            },
            Err(e) => {
                error!("\tGet Table Information Failed!!!");
                error!("\tErr: {e}");
                return Ok(CallToolResult::error(vec![Content::text(format!("Get table information failed: {e}"))]));
            }
        };
        info!("\tTOOL: [GET SCHEMA] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&schema).unwrap_or_default())]))
    }

    #[tool(description = "Health check database")]
    pub async fn get_health_check(&self) -> Result<CallToolResult, ErrorData>
    {
        info!("\tCALL TOOL: [HEALTH CHECK]");
        let mut provider_locking = self.provider.lock().await;
        let health = match provider_locking.health_check().await {
            Ok(result) => {
                info!("\tHealth check success!!!");
                Value::Object(result)
            },
            Err(e) => {
                error!("\thealth check failed: {e}");
                return Ok(CallToolResult::error(vec![Content::text(format!("Health check failed: {e}"))]));
            }
        };
        info!("\tTOOL: [HEALTH CHECK] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&health).unwrap_or_default())]))
    }
}

impl Drop for DbServer {
    fn drop(&mut self) {
        info!("\tShutting down Database Server!!!!");
        let mut provider = self.provider.blocking_lock();
        let _ = provider.close_pool();
        info!("\tDatabase Server shutdown is completed!!!");
    }
}

#[tool_handler]
impl ServerHandler for DbServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let ins = format!("Access Data Store in Database with valid schemas: {}", VALID_TABLES.join(" "));
        let sv = Implementation::new("Database Server".to_string(), "1.0".to_string())
            .with_website_url(self.config.url.clone())
            .with_description("Access and Store data of roadmap, lesson, user, progress, messages, AI conservations".to_string())
            .with_title("Data Access Server".to_string());

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();

        let info = ServerInfo::new(capabilities)
            .with_instructions(ins)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(sv);
        info
    }
}