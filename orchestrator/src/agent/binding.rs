use anyhow::{Result, anyhow};
use async_openai::{Client, config::OpenAIConfig, types::chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, ResponseFormat::JsonObject}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::{AgentContext, PlanStep, PromptBuilder, StepActions};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepBinding {
    pub step_id: String,
    pub input: InputResolver,
    pub output: OutputTarget,
    pub expected_schema: Option<Value>,
}

impl Default for InputResolver {
    fn default() -> Self {
        InputResolver::Context { keys: Vec::new() }
    }
}

impl Default for OutputTarget {
    fn default() -> Self {
        OutputTarget::Scratchpad { name: String::new() }
    }
}

impl Default for StepBinding {
    fn default() -> Self {
        let step_id = String::from("default");
        let input = InputResolver::default();
        let output = OutputTarget::default();
        Self { step_id, input, output, expected_schema: None }
    }
}

impl StepBinding {
    pub async fn resolve_binding(
        &mut self,
        step: &PlanStep,
        execution: &Client<OpenAIConfig>,
        context: &AgentContext,
    ) -> Result<Self>
    {
        // Build system prompt for binding phase
        let mut prompt_build = PromptBuilder::new().await;
        prompt_build.build_binding_phase(true).await; // === Testing On  ===
        let system_prompt = prompt_build.build_system_prompt();

        // Get Dependencies and last observation
        let obs_value = context.last_obs().cloned().unwrap_or(Value::String("No last observation".to_string()));
        let last_obs = serde_json::to_string(&obs_value).unwrap_or("Failed to serialize last observation".to_string());

        let dependencies = step.dependencies.clone();
        let dept_val = dependencies.iter().map(|d| {
            let dependency_obs = context.scratchpad.get(&format!("debug:step_{d}")).cloned().unwrap_or(Value::String("No observation for this dependency".to_string()));
            let dep_obs_str = serde_json::to_string(&dependency_obs).unwrap_or("Failed to serialize dependency observation".to_string());
            format!("Dependency ID: {d}, Observation: {dep_obs_str}")
        }).collect::<Vec<String>>().join("\n\n");

        let obs = format!("Last Observation: {last_obs}\n\n{dept_val}");

        let mut binding_prompt = match &step.action {
            StepActions::ToolCall { server, tool } => {
                let step_goal = step.step_goal.clone().and_then(|f| Some(format!("step goal: {f}"))).unwrap();
                format!("Use tool: {tool} in server: {server} for this step: {:?}\n{step_goal}", step.id.clone())
            },
            StepActions::Reasoning => {
                format!("Response to user")
            },
            StepActions::HumanApproval => {
                format!("Need User approval")
            },
        };
        binding_prompt.push_str(obs.as_str());

        let prompt = ChatCompletionRequestUserMessageArgs::default().content(binding_prompt).build().unwrap();
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(prompt),
            ],
            model: "openai/gpt-oss-20b:free".to_string(),
            response_format: Some(JsonObject),
            ..Default::default()
        };

        let response = match execution.chat().create(request).await {
            Ok(res) => {
                info!("Generate Binding success!!!\n\tBind:{:?}", res);
                println!("Generate Binding success!!!\n\t{:?}", res);
                res
            },
            Err(e) => {
                info!("Binding Generation failed!!!\n\tfail: {:?}", e);
                println!("Binding Generation failed!!!\n\tfail: {:?}", e);
                return Err(e.into());
            }
        };
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .ok_or_else(|| anyhow!("No content in binding response"))?;

        let mut binding = StepBinding::default();
        Ok(binding)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum InputResolver {
    Context{ keys: Vec<ContextKey> },
    LlmResolved { instruction: String, context_keys: Vec<String> },
    Static { value: Value },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OutputTarget{
    Field{ name: String },
    Scratchpad{ name: String },
    FieldAndScratchpad { field: String, scratchpad: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextKey {
    pub from: String,
    pub to: String,
}