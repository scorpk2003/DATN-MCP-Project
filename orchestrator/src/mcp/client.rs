use std::collections::HashMap;

use anyhow::Result;
use jsonschema::Validator;
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, Tool},
    service::RunningService,
    transport::StreamableHttpClientTransport,
};
use serde_json::Value;

use crate::ServerConfig;

pub struct McpClient {
    pub server_name: String,
    pub peer: RunningService<RoleClient, ()>,
    pub tools: HashMap<String, Tool>,
}

impl McpClient {
    pub async fn connect(server: &ServerConfig) -> Result<Self> {
        let transport = StreamableHttpClientTransport::from_uri(server.url.as_str());
        let peer = ().serve(transport).await?;
        let list = peer.list_tools(Default::default()).await?;
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

    pub async fn call_tool(&self, tool_name: &str, params: Value) -> Result<Value> {
        let args = match params {
            Value::Object(obj) => Some(obj),
            _ => None,
        };
        let mut tool_params = CallToolRequestParams::new(tool_name.to_string());
        tool_params.arguments = args;
        let result = self
            .peer
            .call_tool(tool_params.clone())
            .await
            .map_err(|e| {
                tracing::error!("Failed to call tool: {} - {}", tool_params.name, e);
                e
            })?;

        Ok(serde_json::to_value(result.content)?)
    }

    pub fn build_tool_prompt(&self) -> Vec<String> {
        self.tools
            .iter()
            .map(|(_, tool)| {
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
            None => {
                return Err(anyhow::anyhow!(
                    "Tool {} not found in server {}",
                    tool_name,
                    self.server_name
                ));
            }
        }
    }

    pub fn tool_description(&self) -> String {
        self.tools.iter().map(|(_, tool)| {
            format!("\n\tTool Name: {},\n\tTool Description: {},\n\tInput Schema: {:?}, Output Schema: {:?}",
                tool.name.clone(), tool.description.clone().unwrap(), tool.input_schema, tool.output_schema
            )
        }).collect::<Vec<String>>().join(format!("\nAll Tool Above Exist in Server: {}", self.server_name.clone()).as_str())
    }
}
