use std::collections::HashMap;

use crate::{ExecutionState, ExecutionStatus, McpClient, PlanStep, PromptBuilder, ServerConfig};
use async_openai::{Client, config::OpenAIConfig, types::{self, chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest}}};
use anyhow::Result;
use tracing::info;

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

        // Planning Phase
        info!("Planning started!!!");
        (self.state.plan, self.state.context.goal) = self.plan(goal).await?;
        info!("Planning completed!!!");

        // Execution
        while self.state.current_step < self.state.plan.len() {
            let step = &self.state.plan[self.state.current_step].clone();

            if step.waitting {
                // Suspend here
            }

            // Resolver Phase
        }

        info!("Done!!!");
        Ok(())
    }

    async fn plan(&mut self, goal: String) -> Result<(Vec<PlanStep>, Option<String>)> {

        // Build system prompt
        let mut prompt_build = PromptBuilder::new().await;
        prompt_build.build_planning_phase().await;
        prompt_build.build_testing_phase().await;
        let system_prompt = prompt_build.build_system_prompt();

        // Build user prompt
        let user_prompt = ChatCompletionRequestUserMessageArgs::default().content(goal).build().unwrap();
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(user_prompt),
            ],
            model: "openai/gpt-oss-120b:free".to_string(),
            response_format: Some(types::chat::ResponseFormat::JsonObject),
            ..Default::default()
        };

        // Planning
        let response_content = match self.planner.chat().create(request).await {
            Ok(res) => {
                println!("\t\tPlan generated successfully: \n{:#?}\n\n", res);
                res
            },
            Err(e) => {
                println!("Failed to generate plan: {}", e);
                return Err(anyhow::anyhow!("Failed to generate plan: {}", e));
            }
        };
        let content = response_content
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No content in planner response"))?;
        let step: serde_json::Value = serde_json::from_str(content).map_err(|f| anyhow::anyhow!("Failed to parse plan response: {}", f))?;
        
        // Parsing
        let response = match step.get("steps") {
            Some(steps) => serde_json::from_value(steps.clone()).map_err(|f| anyhow::anyhow!("Failed to parse steps: {}", f))?,
            None => {
                println!("No steps found in planner response: {}", content);
                Vec::new()
            }
        };
        let step_goal = match step.get("goal") {
            Some(goals) => serde_json::from_value::<String>(goals.clone()).map_err(|f| anyhow::anyhow!("Failed to parse step goals: {}", f))?,
            None => {
                println!("No step goals found in planner response: {}", content);
                String::new()
            }
        };

        Ok((response, Some(step_goal)))
    }

    // async fn execute_step(&mut self, step: &PlanStep) -> Result<String> {

    // }
}

mod test {
    #[tokio::test]
    async fn test_generate_plan() {
        use super::*;
        dotenv::from_path("../.env").ok();
        let mut kernel = AgentKernel::default();
        let goal = "Testing: Learn C# programming language".to_string();
        let (plan, step_goal) = kernel.plan(goal).await.unwrap();
        for (idx, step) in plan.iter().enumerate() {
            println!("Step {}: {:?}", idx + 1, step);
        }
        println!("Step Goal: {:?}", step_goal);
    }
}
