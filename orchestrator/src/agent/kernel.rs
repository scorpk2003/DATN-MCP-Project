use std::collections::HashMap;

use crate::{AGENT_TESTING, EvaluationStep, ExecutionState, ExecutionStatus, McpClient, PlanStep, PromptBuilder, ServerConfig, StepActions, StepBinding};
use async_openai::{Client, config::OpenAIConfig, types::{self, chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest}}};
use anyhow::Result;
use tracing::info;

pub struct AgentKernel {
    pub planner: Client<OpenAIConfig>,
    pub executor: Client<OpenAIConfig>,
    pub clients: Vec<McpClient>,
    pub state: ExecutionState,
    pub evaluation: Vec<EvaluationStep>,
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
            clients: Vec::new(),
            state: ExecutionState::default(),
            evaluation: Vec::new()
        }
    }
}

impl AgentKernel {
    pub async fn new(server_configs: Vec<ServerConfig>) -> Result<Self> {
        let mut clients = Vec::new();
        for server_config in server_configs {
            let client = McpClient::connect(&server_config).await?;
            clients.push(client);
        }
        let planner = Client::new();
        let executor = Client::new();
        let state = ExecutionState::default();
        let evaluation = Vec::new();
        Ok(Self { clients, planner, executor, state, evaluation })
    }
    pub async fn run(&mut self, goal: String) -> Result<()> {

        self.state.status = ExecutionStatus::Planning;

        let prompt = PromptBuilder::new(&self.clients).await;
        
        // Planning Phase
        info!("Planning started!!!");
        (self.state.plan, self.state.context.goal) = PlanStep::plan(goal, &self.planner, &prompt).await?;
        info!("Planning completed!!!");


        // Execution
        while self.state.current_step < self.state.plan.len() {
            let step: &PlanStep = &self.state.plan[self.state.current_step].clone();

            // Binding Phase
            let binding = StepBinding::resolve_binding(step, &self.executor, &self.state.context, &prompt).await?;
            self.state.resolver.push(binding.clone());

            // Execute Phase
            self.state.status = ExecutionStatus::Running;
            let step_result = self.execute_step(step, &binding, &prompt).await?;
        }

        info!("Done!!!");
        Ok(())
    }

    async fn execute_step(&mut self, step: &PlanStep, binding: &StepBinding, prompt: &PromptBuilder) -> Result<String> {
        
        let params = binding.resolve_params(&self.state.context, &self.executor, &prompt).await?;
        
        match &step.action {
            StepActions::ToolCall { server, tool } => {
                let client = self.clients.iter().find(|c| c.server_name == *server)
                    .ok_or_else(|| anyhow::anyhow!("No client found for server: {}", server))?;
                let response = client.call_tool(tool, params).await?;
            },
            StepActions::Reasoning => {},
            StepActions::HumanApproval => {},
        }
        Ok(String::new())
    }
}

mod test {
}
