use std::collections::HashMap;

use crate::{ExecutionState, ExecutionStatus, McpClient, PlanStep, PromptBuilder, ServerConfig};
use async_openai::{Client, config::OpenAIConfig, types::chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionResponse}};
use anyhow::Result;

pub struct AgentKernel {
    pub planner: Client<OpenAIConfig>,
    pub executor: Client<OpenAIConfig>,
    pub clients: HashMap<String, McpClient>,
    pub state: ExecutionState,
}

impl Default for AgentKernel {
    fn default() -> Self {
        let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENAI_API_KEY must be set");
        let config = OpenAIConfig::new()
        .with_api_base("https://openrouter.ai/api/v1")
        .with_api_key(api_key);
        let planner = Client::with_config(config.clone());
        let executor = Client::with_config(config);
        Self {
            planner,
            executor,
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

    // async fn plan(&mut self, goal: String) -> Vec<PlanStep> {
    //     let system_prompt = PromptBuilder::new().await.build_system_prompt();
    //     let user_prompt = ChatCompletionRequestUserMessageArgs::default().content(goal).build().unwrap();
    //     let request = CreateChatCompletionRequest {
    //         messages: vec![
    //             ChatCompletionRequestMessage::System(system_prompt),
    //             ChatCompletionRequestMessage::User(user_prompt),
    //         ],
    //         model: "gpt-4o-mini".to_string(),
    //         ..Default::default()
    //     };
    //     let response = self.planner.chat().create_byot::<CreateChatCompletionRequest, Vec<PlanStep>>(request).await.unwrap();
    //     response
    // }

    async fn plan(&mut self, goal: String) -> Result<Vec<PlanStep>> {
        let system_prompt = PromptBuilder::new().await.build_system_prompt();
        let user_prompt = ChatCompletionRequestUserMessageArgs::default().content(goal).build().unwrap();
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(user_prompt),
            ],
            model: "openai/gpt-4o-mini".to_string(),
            ..Default::default()
        };
        let response = match self.planner.chat().create_byot::<CreateChatCompletionRequest, Vec<PlanStep>>(request).await {
            Ok(res) => {
                println!("Plan generated successfully: {:#?}", res);
                res   
            },
            Err(e) => {
                println!("Failed to generate plan: {}", e);
                return Err(anyhow::anyhow!("Failed to generate plan: {}", e));
            }
        };
        Ok(response)
    }
}

mod test {
    #[tokio::test]
    async fn test_generate_plan() {
        use super::*;
        dotenv::from_path("../.env").ok();
        let mut kernel = AgentKernel::default();
        let goal = "Testing: Learn Rust programming language".to_string();
        let plan = kernel.plan(goal).await;
        match plan {
            Ok(steps) => {
                println!("Plan generated successfully:");
                for (i, step) in steps.iter().enumerate() {
                    println!("\n\n===============================");
                    println!("\t\tStep {}: {:#?}", i + 1, step);
                    println!("===============================\n\n");
                }
                println!("Total steps: {}", steps.len());
                println!("Plan generation test completed successfully!!!!");
            },
            Err(e) => {
                println!("Error generating plan: {}", e);
            }
        }
    }
}
