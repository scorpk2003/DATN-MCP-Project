use serde::Serialize;
use serde_json::{Value, json};
use tracing::warn;

use crate::{McpClient, ServerConfig};

#[derive(Debug, Clone, Serialize)]
pub struct McpServerStatus {
    pub name: String,
    pub url: String,
    pub required: bool,
    pub connected: bool,
    pub tool_count: usize,
    pub tools: Vec<String>,
    pub last_error: Option<String>,
    pub readiness_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpRegistry {
    pub servers: Vec<McpServerStatus>,
}

impl McpRegistry {
    pub async fn build(configs: &[ServerConfig]) -> Self {
        let mut servers = Vec::new();
        for config in configs {
            let status = match McpClient::connect(config).await {
                Ok(client) => {
                    let readiness_status = readiness_status(&client).await;
                    McpServerStatus {
                        name: config.name.clone(),
                        url: config.url.clone(),
                        required: config.required,
                        connected: true,
                        tool_count: client.tools.len(),
                        tools: client.tools.keys().cloned().collect(),
                        last_error: None,
                        readiness_status,
                    }
                }
                Err(error) => {
                    warn!(
                        server = %config.name,
                        url = %config.url,
                        required = config.required,
                        "MCP registry failed to connect: {error}"
                    );
                    McpServerStatus {
                        name: config.name.clone(),
                        url: config.url.clone(),
                        required: config.required,
                        connected: false,
                        tool_count: 0,
                        tools: Vec::new(),
                        last_error: Some(error.to_string()),
                        readiness_status: "unavailable".to_string(),
                    }
                }
            };
            servers.push(status);
        }
        Self { servers }
    }

    pub fn ready(&self) -> bool {
        self.servers
            .iter()
            .filter(|server| server.required)
            .all(|server| {
                server.connected
                    && server.last_error.is_none()
                    && !server.readiness_status.starts_with("not_ready")
            })
    }

    pub fn ready_response(&self) -> Value {
        json!({
            "ok": self.ready(),
            "status": if self.ready() { "ready" } else { "not_ready" },
            "servers": self.servers,
        })
    }

    pub fn tools_response(&self) -> Value {
        json!({
            "ok": true,
            "servers": self.servers,
        })
    }
}

async fn readiness_status(client: &McpClient) -> String {
    let candidates = match client.server_name.as_str() {
        "roadmap" => vec!["get_readiness_check", "get_health_check"],
        "lesson" => vec!["lesson_readiness", "lesson_health"],
        "resource" => vec!["get_integration_contract"],
        "database" => vec!["get_health_check"],
        _ => Vec::new(),
    };

    for tool in candidates {
        if client.tools.contains_key(tool) {
            return match client.call_tool(tool, json!({})).await {
                Ok(result) if result.ok => "ready".to_string(),
                Ok(result) => result
                    .error
                    .map(|error| format!("not_ready:{}", error.code))
                    .unwrap_or_else(|| "not_ready".to_string()),
                Err(error) => format!("not_ready:{error}"),
            };
        }
    }
    "tool_catalog_ready".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_not_ready_server_makes_registry_not_ready() {
        let registry = McpRegistry {
            servers: vec![McpServerStatus {
                name: "lesson".to_string(),
                url: "http://lesson/mcp".to_string(),
                required: true,
                connected: true,
                tool_count: 1,
                tools: vec!["lesson_readiness".to_string()],
                last_error: None,
                readiness_status: "not_ready:LESSON_MCP_NOT_READY".to_string(),
            }],
        };

        assert!(!registry.ready());
    }
}
