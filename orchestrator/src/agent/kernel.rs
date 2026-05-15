use std::collections::HashMap;

use crate::{ExecutionState, ExecutionStatus, McpClient, PlanStep, ServerConfig};
use async_openai::{Client, config::OpenAIConfig, types::chat::CreateChatCompletionRequest};
use anyhow::Result;
use rmcp::model::PromptMessage;
use serde_json::{Value, json};

pub struct AgentKernel {
    pub planner: Client<OpenAIConfig>,
    pub executor: Client<OpenAIConfig>,
    pub clients: HashMap<String, McpClient>,
    pub state: ExecutionState,
}

impl Default for AgentKernel {
    fn default() -> Self {
        Self {
            planner: Client::new(),
            executor: Client::new(),
            clients: HashMap::new(),
            state: ExecutionState::default(),
        }
    }
}

impl AgentKernel {
    pub async fn new(server_configs: Vec<ServerConfig>) -> Result<Self> {
        let mut clients = HashMap::new();
        for server_config in server_configs {
            let client = McpClient::connect(&server_config).await?;
            clients.insert(server_config.name, client);
        }
        let planner = Client::new();
        let executor = Client::new();
        let state = ExecutionState::default();
        Ok(Self { clients, planner, executor, state })
    }
    pub async fn run(&mut self, goal: String) -> Result<()> {
        self.state.status = ExecutionStatus::Planning;
        Ok(())
    }

    fn plan(&mut self) -> Vec<PlanStep> {
        let mut plan = Vec::new();
        // let mess = PromptMessage::new(role, content)

        plan
    }
}
