use anyhow::Result;
use rmcp::{RoleClient, 
    ServiceExt,
    model::{CallToolRequestParams, Tool},
    service::RunningService,
    transport::{StreamableHttpClientTransport},
};
use serde_json::Value;

use crate::ServerConfig;


pub struct McpClient {
    pub server_name: String,
    pub peer: RunningService<RoleClient, ()>,
    pub tools: Vec<Tool>,
}

impl McpClient {
    pub async fn connect(server: &ServerConfig) -> Result<Self> {
        let transport = StreamableHttpClientTransport::from_uri(server.url.as_str());
        let peer = ().serve(transport).await?;
        let list = peer.list_tools(Default::default()).await?;
        let server_name = server.name.clone();

        Ok(Self {
            server_name,
            peer,
            tools: list.tools,
        })
    }

    pub async fn call_tool(&mut self, tool_name: &str, params: Value) -> Result<Value> {
        let args = match params {
            Value::Object(obj) => Some(obj),
            _ => None
        };
        let mut tool_params = CallToolRequestParams::new(tool_name.to_string());
        tool_params.arguments = args;
        let result = self.peer.call_tool(tool_params.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to call tool: {} - {}", tool_params.name, e);
            e
        })?;

        Ok(serde_json::to_value(result.content)?)
    }
}