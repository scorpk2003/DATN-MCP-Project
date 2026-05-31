use serde::{Deserialize, Serialize};

use crate::{AGENT_TESTING, McpClient, PromptBuilder};
use async_openai::{Client, config::OpenAIConfig, types::{self, chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest}}};
use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanStep {
    pub id: String,
    pub action: StepActions,
    pub step_goal: Option<String>,
    pub dependencies: Vec<String>,
}

impl PlanStep {
    pub async fn plan(goal: String, planner: &Client<OpenAIConfig>, clients: &Vec<McpClient>) -> Result<(Vec<Self>, Option<String>)> {

        // Build system prompt
        let mut prompt_build = PromptBuilder::new(clients).await;
        prompt_build.build_planning_phase(AGENT_TESTING).await;
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
        let response_content = match planner.chat().create(request).await {
            Ok(res) => {
                println!("\t\tPlan generated successfully: \n{:#?}\n\n", res);
                info!("\t\tPlan generated successfully: \n{:#?}\n\n", res);
                res
            },
            Err(e) => {
                println!("Failed to generate plan: {}", e);
                info!("Failed to generate plan: {}", e);
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StepActions {
    ToolCall {
        server: String,
        tool: String,
    },
    Reasoning,
    HumanApproval,
}

mod test {
    #[tokio::test]
    async fn test_plan() {
        use crate::AgentKernel;
        use super::*;
        dotenv::from_path("../.env").ok();
        let kernel = AgentKernel::default();
        let goal = "Testing: Learn C# programming language".to_string();
        let (plan, step_goal) = PlanStep::plan(goal, &kernel.planner, &kernel.clients).await.unwrap();
        for (idx, step) in plan.iter().enumerate() {
            println!("Step {}: {:?}", idx + 1, step);
        }
        println!("Step Goal: {:?}", step_goal);
    }
}
