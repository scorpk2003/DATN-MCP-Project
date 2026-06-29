use anyhow::{Result, anyhow};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, ResponseFormat::JsonObject,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::info;

use crate::{
    AgentContext, PlanStep, PromptBuilder, StepActions, agent_testing_enabled, executor_model,
    parse_llm_json_value,
};

#[derive(Debug, Deserialize)]
struct BindingResponse {
    binding: BindingPayload,
}

#[derive(Debug, Deserialize)]
struct BindingPayload {
    step_id: String,
    input: InputResolver,
    output: OutputTarget,
    expected_schema: Option<Value>,
}

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
        OutputTarget::Scratchpad {
            name: String::new(),
        }
    }
}

impl Default for StepBinding {
    fn default() -> Self {
        let step_id = String::from("default");
        let input = InputResolver::default();
        let output = OutputTarget::default();
        Self {
            step_id,
            input,
            output,
            expected_schema: None,
        }
    }
}

impl StepBinding {
    pub async fn resolve_binding(
        step: &PlanStep,
        execution: &Client<OpenAIConfig>,
        context: &AgentContext,
        prompt: &PromptBuilder,
        selected_tool_schema: Option<&Value>,
    ) -> Result<Self> {
        // Build system prompt for binding phase
        let mut prompt_build = prompt.clone();
        prompt_build
            .build_binding_phase(agent_testing_enabled())
            .await;
        let system_prompt = prompt_build.build_system_prompt();

        // Get Dependencies and last observation
        let obs_value = context
            .last_obs()
            .cloned()
            .unwrap_or(Value::String("No last observation".to_string()));
        let last_obs = serde_json::to_string(&obs_value)
            .unwrap_or("Failed to serialize last observation".to_string());

        let dependencies = step.dependencies.clone();
        let dept_val = dependencies
            .iter()
            .map(|d| {
                let dependency_obs = context
                    .scratchpad
                    .get(&format!("debug:step_{d}"))
                    .cloned()
                    .unwrap_or(Value::String(
                        "No observation for this dependency".to_string(),
                    ));
                let dep_obs_str = serde_json::to_string(&dependency_obs)
                    .unwrap_or("Failed to serialize dependency observation".to_string());
                format!("Dependency ID: {d}, Observation: {dep_obs_str}")
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let tool_schema = selected_tool_schema
            .map(|schema| {
                serde_json::to_string_pretty(schema)
                    .unwrap_or_else(|_| "Failed to serialize selected tool schema".to_string())
            })
            .unwrap_or_else(|| "No selected tool schema for this step.".to_string());
        let context_snapshot = serde_json::to_string(context)
            .unwrap_or_else(|_| "Failed to serialize agent context".to_string());
        let obs = format!(
            "Selected Tool Input Schema:\n{tool_schema}\n\nAgent Context JSON:\n{context_snapshot}\n\nLast Observation: {last_obs}\n\nDependency Observations:\n{dept_val}"
        );

        let mut binding_prompt = match &step.action {
            StepActions::ToolCall { server, tool } => {
                let step_goal = step
                    .step_goal
                    .clone()
                    .map(|f| format!("step goal: {f}"))
                    .unwrap_or_else(|| "step goal: use the selected tool safely".to_string());
                format!(
                    "Use tool: {tool} in server: {server} for this step: {:?}\n{step_goal}",
                    step.id.clone()
                )
            }
            StepActions::Reasoning => "Response to user".to_string(),
            StepActions::HumanApproval => "Need User approval".to_string(),
        };
        binding_prompt.push_str("\n\n");
        binding_prompt.push_str(obs.as_str());

        let prompt = ChatCompletionRequestUserMessageArgs::default()
            .content(binding_prompt)
            .build()
            .map_err(|e| anyhow!("Failed to build binding prompt: {e}"))?;
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(prompt),
            ],
            model: executor_model(),
            response_format: Some(JsonObject),
            ..Default::default()
        };

        let response = match execution.chat().create(request).await {
            Ok(res) => {
                info!("Generate Binding success!!!\n\tBind:{:?}", res);
                println!("Generate Binding success!!!\n\t{:?}", res);
                res
            }
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
        let binding = Self::parse_binding_response(content)?;

        Ok(binding)
    }

    pub fn parse_binding_response(content: &str) -> Result<Self> {
        let value = parse_llm_json_value(content, "BINDING_PARSE_ERROR")?;
        let response = serde_json::from_value::<BindingResponse>(value)
            .map_err(|e| anyhow!("BINDING_PARSE_ERROR: invalid binding contract: {e}"))?;

        Ok(StepBinding {
            step_id: response.binding.step_id,
            input: response.binding.input,
            output: response.binding.output,
            expected_schema: response.binding.expected_schema,
        })
    }

    pub async fn resolve_params(
        &self,
        context: &AgentContext,
        executor: &Client<OpenAIConfig>,
        prompt: &PromptBuilder,
    ) -> Result<Value> {
        match &self.input {
            InputResolver::Context { keys } => {
                let ctx = serde_json::to_value(context)?;
                let mut params = Map::new();

                for key in keys {
                    let (from, to) = key.extract_key();

                    let value = Self::resolve_path(&ctx, &from)?;
                    Self::insert_nested(&mut params, &to, value)?;
                }
                Ok(Value::Object(params))
            }
            InputResolver::LlmResolved {
                instruction,
                context_keys,
            } => {
                let keys = context_keys.join(", ");
                let ctx = serde_json::to_value(context)?;
                let user_prompt =
                    format!("Instruction: {instruction}\nContext Keys: {keys}\nContext: {ctx}");
                let system_prompt = prompt.build_system_prompt();
                let request = CreateChatCompletionRequest {
                    messages: vec![
                        ChatCompletionRequestMessage::System(system_prompt),
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(user_prompt)
                                .build()
                                .map_err(|e| anyhow!("Failed to build LLM resolved prompt: {e}"))?,
                        ),
                    ],
                    model: executor_model(),
                    response_format: Some(JsonObject),
                    ..Default::default()
                };
                let response = match executor.chat().create(request).await {
                    Ok(res) => {
                        info!("LLM Resolved input success!!!\n\tResolved Input:{:?}", res);
                        println!("LLM Resolved input success!!!\n\tResolved Input:{:?}", res);
                        res
                    }
                    Err(e) => {
                        info!("LLM Resolved input failed!!!\n\tfail: {:?}", e);
                        println!("LLM Resolved input failed!!!\n\tfail: {:?}", e);
                        return Err(e.into());
                    }
                };
                let content = response
                    .choices
                    .first()
                    .and_then(|c| c.message.content.as_deref())
                    .ok_or_else(|| anyhow!("No content in LLM resolved input response"))?;
                let value = parse_llm_json_value(content, "LLM_RESOLVED_INPUT_PARSE_ERROR")
                    .map_err(|e| anyhow!("Failed to parse LLM resolved input response: {}", e))?;
                Ok(value)
            }
            InputResolver::Static { value } => Ok(value.clone()),
        }
    }

    pub async fn repair_params(
        step: &PlanStep,
        params: &Value,
        schema: &Value,
        validation_error: &str,
        context: &AgentContext,
        executor: &Client<OpenAIConfig>,
        prompt: &PromptBuilder,
    ) -> Result<Value> {
        let step_goal = step.step_goal.clone().unwrap_or_default();
        let context_json = serde_json::to_string(context)?;
        let params_json = serde_json::to_string(params)?;
        let schema_json = serde_json::to_string_pretty(schema)?;
        let user_prompt = format!(
            "Repair tool parameters only.\n\nStep goal: {step_goal}\n\nValidation error: {validation_error}\n\nCurrent params JSON:\n{params_json}\n\nTool input schema:\n{schema_json}\n\nAgent context JSON:\n{context_json}\n\nReturn only the repaired JSON params object. Do not wrap it in binding. Do not include markdown. Do not invent trusted auth fields."
        );
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(prompt.build_system_prompt()),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(user_prompt)
                        .build()
                        .map_err(|e| anyhow!("Failed to build param repair prompt: {e}"))?,
                ),
            ],
            model: executor_model(),
            response_format: Some(JsonObject),
            ..Default::default()
        };
        let response = executor.chat().create(request).await?;
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_deref())
            .ok_or_else(|| anyhow!("No content in param repair response"))?;
        let value = parse_llm_json_value(content, "PARAM_REPAIR_PARSE_ERROR")?;
        if !value.is_object() {
            return Err(anyhow!(
                "PARAM_REPAIR_PARSE_ERROR: repaired params must be a JSON object"
            ));
        }
        Ok(value)
    }

    pub fn apply_output(&self, context: &mut AgentContext, value: &Value) {
        match &self.output {
            OutputTarget::Field { name } => {
                context.write_field(name, value);
            }
            OutputTarget::Scratchpad { name } => {
                context.write_obs(name, value);
            }
            OutputTarget::FieldAndScratchpad { field, scratchpad } => {
                context.write_obs(scratchpad, value);
                context.write_field(field, value);
            }
        }
    }

    fn resolve_path(root: &Value, path: &[String]) -> Result<Value> {
        let mut current = root;
        for key in path {
            current = current
                .get(key)
                .ok_or_else(|| anyhow!("Key '{}' not found in context", key))?;
        }
        Ok(current.clone())
    }
    fn insert_nested(obj: &mut Map<String, Value>, path: &[String], value: Value) -> Result<()> {
        if path.is_empty() {
            return Ok(());
        }

        let mut current = obj;

        for key in &path[..path.len() - 1] {
            current = current
                .entry(key.clone())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| anyhow!("Target path '{}' is not an object", key))?;
        }

        let Some(last) = path.last() else {
            return Ok(());
        };
        current.insert(last.clone(), value);
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum InputResolver {
    Context {
        keys: Vec<ContextKey>,
    },
    LlmResolved {
        instruction: String,
        context_keys: Vec<String>,
    },
    Static {
        value: Value,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OutputTarget {
    Field { name: String },
    Scratchpad { name: String },
    FieldAndScratchpad { field: String, scratchpad: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextKey {
    pub from: String,
    pub to: String,
}
impl ContextKey {
    pub fn extract_key(&self) -> (Vec<String>, Vec<String>) {
        let split_val = self
            .from
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let split_key = self
            .to
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        (split_val, split_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wrapped_binding_response() {
        let content = r#"{
            "binding": {
                "step_id": "step 1",
                "input": {"type": "Static", "value": {"goal": "learn rust"}},
                "output": {"type": "Scratchpad", "name": "roadmap"},
                "expected_schema": {"ok": true}
            }
        }"#;

        let binding = StepBinding::parse_binding_response(content).unwrap();
        assert_eq!(binding.step_id, "step 1");
        assert!(matches!(binding.input, InputResolver::Static { .. }));
    }

    #[test]
    fn parses_fenced_binding_response() {
        let content = r#"```json
        {
            "binding": {
                "step_id": "step 1",
                "input": {"type": "Static", "value": {"goal": "learn rust"}},
                "output": {"type": "Scratchpad", "name": "roadmap"},
                "expected_schema": {"ok": true}
            }
        }
        ```"#;

        let binding = StepBinding::parse_binding_response(content).unwrap();
        assert_eq!(binding.step_id, "step 1");
        assert!(matches!(binding.input, InputResolver::Static { .. }));
    }

    #[test]
    fn parses_inline_fenced_binding_response() {
        let content = r#"```json{"binding":{"step_id":"step 1","input":{"type":"Static","value":{}},"output":{"type":"Scratchpad","name":"x"},"expected_schema":null}}```"#;

        let binding = StepBinding::parse_binding_response(content).unwrap();
        assert_eq!(binding.step_id, "step 1");
    }

    #[test]
    fn rejects_unwrapped_binding_response() {
        let content = r#"{
            "input": {"type": "Static", "value": {}},
            "output": {"type": "Scratchpad", "name": "x"},
            "expected_schema": {}
        }"#;

        let error = StepBinding::parse_binding_response(content).unwrap_err();
        assert!(error.to_string().contains("BINDING_PARSE_ERROR"));
    }
}
