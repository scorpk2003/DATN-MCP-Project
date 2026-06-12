use anyhow::{Result, anyhow};
use async_openai::{Client, config::OpenAIConfig, types::chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, ResponseFormat::JsonObject}};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::info;

use crate::{AGENT_TESTING, AgentContext, PlanStep, PromptBuilder, StepActions};


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
        step: &PlanStep,
        execution: &Client<OpenAIConfig>,
        context: &AgentContext,
        prompt: &PromptBuilder
    ) -> Result<Self>
    {
        // Build system prompt for binding phase
        let mut prompt_build = prompt.clone();
        prompt_build.build_binding_phase(AGENT_TESTING).await;
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
        let content = serde_json::from_str::<Value>(content).expect("Convert to value fail!!!");

        let input = content.get("input").expect("Input no exist in binding response");
        let output = content.get("output").expect("Output no exist in binding response");
        let expected_schema = content.get("expected_schema").expect("Expected schema no exist in binding response");

        let input = serde_json::from_value::<InputResolver>(input.clone()).expect("Failed to parse input resolver");
        let output = serde_json::from_value::<OutputTarget>(output.clone()).expect("Failed to parse output target");
        let expected_schema = Some(expected_schema.clone());

        let binding = StepBinding { step_id: step.id.clone(), input, output, expected_schema };
        Ok(binding)
    }

    pub async fn resolve_params(&self, context: &AgentContext, executor: &Client<OpenAIConfig>, prompt: &PromptBuilder) -> Result<Value> {
        match &self.input {
            InputResolver::Context { keys } => {
                let ctx = serde_json::to_value(context)?;
                let mut params = Map::new();

                for key in keys {
                    let (from, to) = key.extract_key();

                    let value = Self::resolve_path(&ctx, &from)?;
                    Self::insert_nested(&mut params, &to, value);
                }
                Ok(Value::Object(params))
            },
            InputResolver::LlmResolved { instruction, context_keys } => {
                let keys = context_keys.join(", ");
                let ctx = serde_json::to_value(context)?;
                let user_prompt = format!("Instruction: {instruction}\nContext Keys: {keys}\nContext: {ctx}");
                let system_prompt = prompt.build_system_prompt();
                let request = CreateChatCompletionRequest {
                    messages: vec![
                        ChatCompletionRequestMessage::System(system_prompt),
                        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessageArgs::default().content(user_prompt).build().unwrap()),
                    ],
                    model: "openai/gpt-oss-20b:free".to_string(),
                    response_format: Some(JsonObject),
                    ..Default::default()
                };
                let response = match executor.chat().create(request).await {
                    Ok(res) => {
                        info!("LLM Resolved input success!!!\n\tResolved Input:{:?}", res);
                        println!("LLM Resolved input success!!!\n\tResolved Input:{:?}", res);
                        res
                    },
                    Err(e) => {
                        info!("LLM Resolved input failed!!!\n\tfail: {:?}", e);
                        println!("LLM Resolved input failed!!!\n\tfail: {:?}", e);
                        return Err(e.into());
                    }
                };
                let content = response.choices.first().and_then(|c| c.message.content.as_deref()).ok_or_else(|| anyhow!("No content in LLM resolved input response"))?;
                let value = serde_json::from_str(content).map_err(|e| anyhow!("Failed to parse LLM resolved input response: {}", e))?;
                Ok(value)
            },
            InputResolver::Static { value } => {
                Ok(value.clone())
            },
        }
    }

    pub fn apply_output(&self, context: &mut AgentContext, value: &Value) {
        match &self.output {
            OutputTarget::Field { name } => {
                context.write_field(name, value);
            },
            OutputTarget::Scratchpad { name } => {
                context.write_obs(name, value);
            },
            OutputTarget::FieldAndScratchpad { field, scratchpad } => {
                context.write_obs(scratchpad, value);
                context.write_field(field, value);
            },
        }
    }

    fn resolve_path(root: &Value, path: &[String]) -> Result<Value> {
        let mut current = root;
        for key in path {
            current = current.get(key).ok_or_else(|| anyhow!("Key '{}' not found in context", key))?;
        }
        Ok(current.clone())
    }
    fn insert_nested(obj: &mut Map<String, Value>, path: &[String], value: Value) {
        if path.is_empty() {
            return;
        }

        let mut current = obj;

        for key in &path[..path.len() - 1] {
            current = current.entry(key.clone()).or_insert_with(|| Value::Object(Map::new())).as_object_mut().expect("Expect Object during insert");
        }

        current.insert(path.last().unwrap().clone(), value);
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
impl ContextKey {
    pub fn extract_key(&self) -> (Vec<String>, Vec<String>) {
        let split_val = self.from.split(".").map(|s| s.to_string()).collect::<Vec<String>>();
        let split_key = self.to.split(".").map(|s| s.to_string()).collect::<Vec<String>>();
        (split_val, split_key)
    }
}