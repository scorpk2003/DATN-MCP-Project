use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use jsonschema::Validator;
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, Tool},
    service::RunningService,
    transport::StreamableHttpClientTransport,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep, timeout};

use crate::{ServerConfig, parse_llm_json_value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedToolError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedToolMetadata {
    pub duration_ms: u128,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedToolResult {
    pub ok: bool,
    pub server: String,
    pub tool: String,
    pub data: Value,
    pub raw: Value,
    pub error: Option<NormalizedToolError>,
    pub metadata: NormalizedToolMetadata,
}

pub struct McpClient {
    pub server_name: String,
    pub peer: RunningService<RoleClient, ()>,
    pub tools: HashMap<String, Tool>,
}

impl McpClient {
    pub async fn connect(server: &ServerConfig) -> Result<Self> {
        let transport = StreamableHttpClientTransport::from_uri(server.url.as_str());
        let connect_timeout = Duration::from_millis(env_ms("MCP_CONNECT_TIMEOUT_MS", 3000));
        let list_timeout = Duration::from_millis(env_ms("MCP_LIST_TOOLS_TIMEOUT_MS", 3000));
        let peer = timeout(connect_timeout, ().serve(transport))
            .await
            .map_err(|_| anyhow::anyhow!("MCP_TIMEOUT connecting to {}", server.name))??;
        let list = timeout(list_timeout, peer.list_tools(Default::default()))
            .await
            .map_err(|_| anyhow::anyhow!("MCP_TIMEOUT listing tools for {}", server.name))??;
        let server_name = server.name.clone();

        let list_tools = list.tools.clone();
        let tools = list_tools
            .iter()
            .map(|tool| {
                let name = tool.name.clone().into();
                (name, tool.clone())
            })
            .collect();

        Ok(Self {
            server_name,
            peer,
            tools,
        })
    }

    pub async fn call_tool(&self, tool_name: &str, params: Value) -> Result<NormalizedToolResult> {
        let args = match params {
            Value::Object(obj) => Some(obj),
            _ => None,
        };
        let mut tool_params = CallToolRequestParams::new(tool_name.to_string());
        tool_params.arguments = args;
        let started = Instant::now();
        let call_timeout = Duration::from_millis(env_ms("MCP_CALL_TOOL_TIMEOUT_MS", 15000));
        let retries = env_usize("MCP_CALL_RETRIES", 1);
        let backoff = Duration::from_millis(env_ms("MCP_RETRY_BACKOFF_MS", 150));
        let mut attempt = 0usize;
        let result = loop {
            let call = timeout(call_timeout, self.peer.call_tool(tool_params.clone())).await;
            match call {
                Ok(Ok(result)) => break result,
                Ok(Err(error)) => {
                    tracing::error!(
                        attempt,
                        "Failed to call tool: {} - {}",
                        tool_params.name,
                        error
                    );
                    if attempt >= retries {
                        return Err(error.into());
                    }
                }
                Err(_) => {
                    let error =
                        anyhow::anyhow!("MCP_TIMEOUT calling {}.{}", self.server_name, tool_name);
                    tracing::error!(attempt, "{error}");
                    if attempt >= retries {
                        return Err(error);
                    }
                }
            }
            attempt += 1;
            sleep(backoff).await;
        };
        let duration_ms = started.elapsed().as_millis();
        let is_error = result.is_error.unwrap_or(false);
        let raw = serde_json::to_value(&result.content)?;
        let data = normalize_content_data(&raw);

        Ok(NormalizedToolResult {
            ok: !is_error,
            server: self.server_name.clone(),
            tool: tool_name.to_string(),
            data: if is_error { Value::Null } else { data.clone() },
            raw,
            error: is_error.then(|| NormalizedToolError {
                code: "MCP_TOOL_ERROR".to_string(),
                message: summarize_tool_error(&data),
                recoverable: true,
            }),
            metadata: NormalizedToolMetadata {
                duration_ms,
                is_error,
            },
        })
    }

    pub fn build_tool_prompt(&self) -> Vec<String> {
        self.tools
            .values()
            .map(|tool| {
                format!(
                    "{}.{}: {}",
                    self.server_name.clone(),
                    tool.name.clone(),
                    tool.description
                        .clone()
                        .unwrap_or_else(|| "No description available".into())
                )
            })
            .collect()
    }

    pub fn tool_validation(&self, tool_name: &str, params: &Value) -> Result<()> {
        let tool = self.tools.get(tool_name);
        match tool {
            Some(t) => {
                let schema = t.schema_as_json_value();
                let validator = Validator::new(&schema)?;
                validator.validate(params).map_err(|e| {
                    anyhow::anyhow!("Validation failed for tool {}: {}", tool_name, e)
                })?;
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Tool {} not found in server {}",
                tool_name,
                self.server_name
            )),
        }
    }

    pub fn tool_schema(&self, tool_name: &str) -> Option<Value> {
        self.tools
            .get(tool_name)
            .map(|tool| tool.schema_as_json_value())
    }

    pub fn tool_description(&self) -> String {
        self.tools.values().map(|tool| {
            format!("\n\tTool Name: {},\n\tTool Description: {},\n\tInput Schema: {:?}, Output Schema: {:?}",
                tool.name.clone(), tool.description.clone().unwrap_or_else(|| "No description available".into()), tool.input_schema, tool.output_schema
            )
        }).collect::<Vec<String>>().join(format!("\nAll Tool Above Exist in Server: {}", self.server_name.clone()).as_str())
    }

    pub fn tool_requires_auth(&self, tool_name: &str) -> bool {
        self.tools
            .get(tool_name)
            .map(|tool| {
                let schema = tool.schema_as_json_value().to_string();
                schema.contains("auth_context") || schema.contains("authContext")
            })
            .unwrap_or(false)
    }
}

fn env_ms(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn normalize_content_data(raw: &Value) -> Value {
    let Some(items) = raw.as_array() else {
        return raw.clone();
    };
    for item in items {
        if let Some(text) = extract_text(item) {
            return parse_llm_json_value(&text, "MCP_CONTENT_PARSE")
                .unwrap_or_else(|_| json!({ "text": text }));
        }
    }
    raw.clone()
}

fn extract_text(value: &Value) -> Option<String> {
    value
        .pointer("/raw/text")
        .or_else(|| value.pointer("/text"))
        .or_else(|| value.pointer("/raw/Text/text"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn summarize_tool_error(data: &Value) -> String {
    data.pointer("/error/message")
        .or_else(|| data.pointer("/message"))
        .or_else(|| data.pointer("/text"))
        .and_then(Value::as_str)
        .unwrap_or("MCP tool returned an error")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_json_text_content() {
        let raw = json!([{ "raw": { "text": "{\"ok\":true,\"data\":{\"x\":1}}" } }]);
        assert_eq!(normalize_content_data(&raw)["data"]["x"], 1);
    }

    #[test]
    fn normalizes_fenced_json_text_content() {
        let raw = json!([{ "raw": { "text": "```json\n{\"ok\":true,\"data\":{\"x\":1}}\n```" } }]);
        assert_eq!(normalize_content_data(&raw)["data"]["x"], 1);
    }

    #[test]
    fn normalizes_plain_text_content() {
        let raw = json!([{ "raw": { "text": "hello" } }]);
        assert_eq!(normalize_content_data(&raw)["text"], "hello");
    }
}
