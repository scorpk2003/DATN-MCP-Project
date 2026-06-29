use async_openai::types::chat::ResponseFormat::JsonObject;
use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use crate::{
    AuthContext, EvaluationDecision, EvaluationStep, ExecutionState, ExecutionStatus, McpClient,
    PlanStep, PromptBuilder, ServerConfig, StepActions, StepBinding, StepExecutionResult,
    parse_llm_json_value,
};
use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest,
    },
};
use tracing::{error, info, warn};

pub struct AgentKernel {
    pub planner: Client<OpenAIConfig>,
    pub executor: Client<OpenAIConfig>,
    pub clients: Vec<McpClient>,
    pub state: ExecutionState,
    pub evaluation: Vec<EvaluationStep>,
    final_output: Map<String, Value>,
}

impl Default for AgentKernel {
    fn default() -> Self {
        let planner = llm_client_from_env().unwrap_or_else(|_| llm_client_with_key("test-key"));
        let executor = llm_client_from_env().unwrap_or_else(|_| llm_client_with_key("test-key"));
        let final_output = Map::new();
        Self {
            planner,
            executor,
            clients: Vec::new(),
            state: ExecutionState::default(),
            evaluation: Vec::new(),
            final_output,
        }
    }
}

impl AgentKernel {
    pub async fn new(
        server_configs: Vec<ServerConfig>,
        session_id: String,
        user_id: Option<String>,
        auth_context: Option<AuthContext>,
    ) -> Result<Self> {
        let mut clients = Vec::new();
        let mut required_failures = Vec::new();
        for server_config in server_configs {
            match McpClient::connect(&server_config).await {
                Ok(client) => clients.push(client),
                Err(e) => {
                    warn!(
                        "Failed to connect MCP server {} at {}: {}",
                        server_config.name, server_config.url, e
                    );
                    if server_config.required {
                        required_failures.push(format!("{} ({})", server_config.name, e));
                    }
                }
            }
        }
        if !required_failures.is_empty() {
            return Err(anyhow::anyhow!(
                "Required MCP servers unavailable: {}",
                required_failures.join(", ")
            ));
        }
        if clients.is_empty() {
            return Err(anyhow::anyhow!(
                "No MCP servers are available for orchestration"
            ));
        }
        let planner = llm_client_from_env()?;
        let executor = llm_client_from_env()?;
        let mut state = ExecutionState::new(session_id);
        state.context.session_id = state.session_id.clone();
        state.context.user_id = user_id;
        state.context.auth_context = auth_context;
        sync_authenticated_user(&clients, state.context.auth_context.as_ref()).await?;
        let evaluation = Vec::new();
        let final_output = Map::new();
        Ok(Self {
            clients,
            planner,
            executor,
            state,
            evaluation,
            final_output,
        })
    }

    pub async fn from_state(
        server_configs: Vec<ServerConfig>,
        state: ExecutionState,
    ) -> Result<Self> {
        let mut kernel = Self::new(
            server_configs,
            state.session_id.clone(),
            state.context.user_id.clone(),
            state.context.auth_context.clone(),
        )
        .await?;
        kernel.state = state;
        Ok(kernel)
    }

    pub async fn run(&mut self, goal: String) -> Result<Value> {
        self.state.status = ExecutionStatus::Planning;

        let prompt = PromptBuilder::new(&self.clients).await;

        // Planning Phase
        info!("Planning started!!!");
        (self.state.plan, self.state.context.goal) =
            PlanStep::plan(goal, &self.planner, &prompt).await?;
        info!("Planning completed!!!");

        self.continue_run(prompt).await
    }

    pub async fn continue_existing(&mut self) -> Result<Value> {
        let prompt = PromptBuilder::new(&self.clients).await;
        self.continue_run(prompt).await
    }

    pub async fn replan_existing(&mut self, observation: String) -> Result<Value> {
        let prompt = PromptBuilder::new(&self.clients).await;
        self.state.status = ExecutionStatus::RePlanning(observation.clone());
        self.state.plan =
            PlanStep::re_plan(&self.planner, &prompt, &self.state.context, observation).await?;
        self.state.current_step = 0;
        self.continue_run(prompt).await
    }

    async fn continue_run(&mut self, prompt: PromptBuilder) -> Result<Value> {
        let max_steps = env_usize("AGENT_MAX_STEPS", 8);
        let max_replans = env_usize("AGENT_MAX_REPLANS", 2);
        let mut executed_steps = 0usize;
        let mut replan_count = 0usize;

        while self.state.current_step < self.state.plan.len() {
            if executed_steps >= max_steps {
                self.state.status = ExecutionStatus::Failed("AGENT_MAX_STEPS exceeded".into());
                return Err(anyhow::anyhow!("AGENT_MAX_STEPS exceeded"));
            }
            executed_steps += 1;
            let step: &PlanStep = &self.state.plan[self.state.current_step].clone();

            // Binding Phase
            let selected_tool_schema = self.selected_tool_schema(step);
            let binding = StepBinding::resolve_binding(
                step,
                &self.executor,
                &self.state.context,
                &prompt,
                selected_tool_schema.as_ref(),
            )
            .await?;
            self.state.resolver.push(binding.clone());

            // Execute Phase
            self.state.status = ExecutionStatus::Running;
            let step_result = self.execute_step(step, &binding, &prompt).await;
            binding.apply_output(&mut self.state.context, &step_result.output);

            // Evaluation Phase
            let evaluation = EvaluationStep::evaluate(step.id.clone(), &step_result).await;
            match evaluation.decision {
                EvaluationDecision::Continue => {
                    self.state.current_step += 1;
                    if let Some(final_output) = &step.final_output {
                        self.final_output
                            .insert(final_output.clone(), step_result.output.clone());
                    }
                }
                EvaluationDecision::Wait => {
                    let message = step_result
                        .observation
                        .clone()
                        .unwrap_or_else(|| "Waiting for user input".to_string());
                    self.state.status = ExecutionStatus::Waiting(message.clone());
                    return Ok(json!({
                        "ok": true,
                        "status": "waiting_for_user",
                        "session_id": self.state.session_id,
                        "approval": {
                            "step_id": step.id,
                            "question": message,
                            "options": ["approve", "reject", "revise"]
                        }
                    }));
                }
                EvaluationDecision::Replan => {
                    if replan_count >= max_replans {
                        self.state.status =
                            ExecutionStatus::Failed("AGENT_MAX_REPLANS exceeded".into());
                        return Err(anyhow::anyhow!("AGENT_MAX_REPLANS exceeded"));
                    }
                    replan_count += 1;
                    let observation = step_result
                        .observation
                        .clone()
                        .unwrap_or_else(|| "Step failed without observation".to_string());
                    self.state.status = ExecutionStatus::RePlanning(observation.clone());
                    self.state.plan =
                        PlanStep::re_plan(&self.planner, &prompt, &self.state.context, observation)
                            .await?;
                    self.state.current_step = 0;
                    continue;
                }
                EvaluationDecision::Finish => {
                    self.state.status = ExecutionStatus::Completed;
                    break;
                }
                EvaluationDecision::Failed => {
                    let observation = step_result
                        .observation
                        .clone()
                        .unwrap_or_else(|| "Step failed".to_string());
                    self.state.status = ExecutionStatus::Failed(observation.clone());
                    return Err(anyhow::anyhow!(observation));
                }
            }
            info!("Step {} execution completed!!!", step.id.clone());
        }

        info!("Done!!!");
        Ok(json!({
            "ok": true,
            "status": "completed",
            "session_id": self.state.session_id,
            "output": Value::Object(self.final_output.clone())
        }))
    }

    async fn execute_step(
        &mut self,
        step: &PlanStep,
        binding: &StepBinding,
        prompt: &PromptBuilder,
    ) -> StepExecutionResult {
        let params = match binding
            .resolve_params(&self.state.context, &self.executor, prompt)
            .await
        {
            Ok(p) => {
                info!(
                    "Resolved Params success for action: {:?}",
                    step.action.clone()
                );
                info!("Params: {:?}", p);
                p
            }
            Err(e) => {
                error!("Failed to resolve params for action: {:?}", e);
                return StepExecutionResult {
                    success: false,
                    output: Value::Null,
                    observation: Some(format!(
                        "Failed to resolve params for action: {:?}. Error: {}",
                        step.action.clone(),
                        e
                    )),
                    waiting: false,
                    replan: true,
                };
            }
        };

        match &step.action {
            StepActions::ToolCall { server, tool } => {
                if server == "database" {
                    return StepExecutionResult {
                        success: false,
                        output: Value::Null,
                        observation: Some(format!(
                            "DIRECT_DATABASE_TOOL_FORBIDDEN: {}.{} is an internal persistence tool. Use roadmap/lesson/resource MCP tools; the orchestrator will execute Database MCP persistence plans with DB UUIDs.",
                            server, tool
                        )),
                        waiting: false,
                        replan: true,
                    };
                }
                let client = match self
                    .clients
                    .iter()
                    .find(|c| c.server_name == *server)
                    .ok_or_else(|| anyhow::anyhow!("No client found for server: {}", server))
                {
                    Ok(c) => {
                        info!("Found client success!!!");
                        c
                    }
                    Err(e) => {
                        error!("Failed to find client, error: {}", e);
                        return StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!(
                                "Failed to find client for server: {}. Error: {}",
                                server, e
                            )),
                            waiting: false,
                            replan: true,
                        };
                    }
                };
                let mut params = params;
                if client.tool_requires_auth(tool) {
                    params = match inject_trusted_auth(params, &self.state.context) {
                        Ok(value) => value,
                        Err(e) => {
                            return StepExecutionResult {
                                success: false,
                                output: Value::Null,
                                observation: Some(format!(
                                    "AUTH_CONTEXT_MISSING for protected tool {}.{}: {}",
                                    server, tool, e
                                )),
                                waiting: false,
                                replan: false,
                            };
                        }
                    };
                }
                params = sanitize_tool_params(params);

                match client.tool_validation(tool, &params) {
                    Ok(_) => info!("Tool validation success!!!"),
                    Err(e) => {
                        let validation_error = e.to_string();
                        error!("Tool validation failed, error: {}", validation_error);
                        let max_repairs = env_usize("AGENT_MAX_PARAM_REPAIRS", 1);
                        let mut repaired = false;
                        for repair_attempt in 0..max_repairs {
                            let Some(schema) = client.tool_schema(tool) else {
                                break;
                            };
                            match StepBinding::repair_params(
                                step,
                                &params,
                                &schema,
                                &validation_error,
                                &self.state.context,
                                &self.executor,
                                prompt,
                            )
                            .await
                            {
                                Ok(candidate) => match client.tool_validation(tool, &candidate) {
                                    Ok(_) => {
                                        info!(
                                            repair_attempt,
                                            "Tool params repaired and validated successfully"
                                        );
                                        params = candidate;
                                        repaired = true;
                                        break;
                                    }
                                    Err(error) => {
                                        error!(
                                            repair_attempt,
                                            "Repaired params still failed validation: {}", error
                                        );
                                    }
                                },
                                Err(error) => {
                                    error!(repair_attempt, "Tool params repair failed: {}", error);
                                }
                            }
                        }
                        if !repaired {
                            return StepExecutionResult {
                                success: false,
                                output: Value::Null,
                                observation: Some(format!(
                                    "Tool validation failed for server: {}.{}. Error: {}",
                                    server, tool, validation_error
                                )),
                                waiting: false,
                                replan: true,
                            };
                        }
                    }
                };
                match client.call_tool(tool, params).await {
                    Ok(response) => {
                        let mut output = serde_json::to_value(response).unwrap_or(Value::Null);
                        let persistence_result =
                            self.execute_database_call_plans(&mut output).await;
                        match persistence_result {
                            Ok(executed_count) => StepExecutionResult {
                                success: output.get("ok").and_then(Value::as_bool).unwrap_or(true),
                                output,
                                observation: Some(if executed_count > 0 {
                                    format!(
                                        "Call Tool success: {}.{} and executed {executed_count} persistence calls",
                                        server, tool
                                    )
                                } else {
                                    format!("Call Tool success: {}.{}", server, tool)
                                }),
                                waiting: false,
                                replan: false,
                            },
                            Err(error) => StepExecutionResult {
                                success: false,
                                output,
                                observation: Some(format!(
                                    "Persistence plan execution failed after {}.{}: {}",
                                    server, tool, error
                                )),
                                waiting: false,
                                replan: true,
                            },
                        }
                    }
                    Err(e) => StepExecutionResult {
                        success: false,
                        output: Value::Null,
                        observation: Some(format!(
                            "Call Tool failed: {}.{}. Error: {}",
                            server, tool, e
                        )),
                        waiting: false,
                        replan: true,
                    },
                }
            }
            StepActions::Reasoning => {
                let user_prompt = ChatCompletionRequestUserMessageArgs::default()
                    .content(format!(
                        "Reasoning step with instruction: {:?}\n\nContext: {}",
                        step.step_goal.clone().unwrap_or_default(),
                        serde_json::to_string(&self.state.context).unwrap_or_default()
                    ))
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build reasoning prompt: {e}"));
                let user_prompt = match user_prompt {
                    Ok(prompt) => prompt,
                    Err(e) => {
                        return StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(e.to_string()),
                            waiting: false,
                            replan: true,
                        };
                    }
                };
                let system = prompt.build_system_prompt();
                let request = CreateChatCompletionRequest {
                    messages: vec![
                        ChatCompletionRequestMessage::System(system),
                        ChatCompletionRequestMessage::User(user_prompt),
                    ],
                    model: executor_model(),
                    response_format: Some(JsonObject),
                    ..Default::default()
                };
                let response = match self.executor.chat().create(request).await {
                    Ok(resp) => {
                        info!("Reasoning step completed successfully!!!");
                        resp
                    }
                    Err(e) => {
                        error!("Reasoning step failed, error: {}", e);
                        return StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!("Reasoning step failed. Error: {}", e)),
                            waiting: false,
                            replan: true,
                        };
                    }
                };
                let content = match response
                    .choices
                    .first()
                    .and_then(|c| c.message.content.as_deref())
                    .ok_or_else(|| anyhow::anyhow!("No content in reasoning response"))
                {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Failed to extract content from reasoning response: {}", e);
                        return StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!(
                                "Failed to extract content from reasoning response. Error: {}",
                                e
                            )),
                            waiting: false,
                            replan: true,
                        };
                    }
                };
                match parse_llm_json_value(content, "REASONING_PARSE_ERROR") {
                    Ok(json) => StepExecutionResult {
                        success: true,
                        output: json,
                        observation: Some(format!(
                            "Reasoning step completed: {:?}",
                            step.step_goal.clone().unwrap_or_default()
                        )),
                        waiting: false,
                        replan: false,
                    },
                    Err(e) => {
                        error!("Failed to parse reasoning response as JSON: {}", e);
                        StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!(
                                "Failed to parse reasoning response as JSON. Error: {}",
                                e
                            )),
                            waiting: false,
                            replan: true,
                        }
                    }
                }
            }
            StepActions::HumanApproval => {
                let output = json!({});
                StepExecutionResult {
                    success: true,
                    output,
                    observation: Some(format!(
                        "Human approval needed for step: {:?}",
                        step.step_goal.clone().unwrap_or_default()
                    )),
                    waiting: true,
                    replan: false,
                }
            }
        }
    }

    fn selected_tool_schema(&self, step: &PlanStep) -> Option<Value> {
        let StepActions::ToolCall { server, tool } = &step.action else {
            return None;
        };
        self.clients
            .iter()
            .find(|client| client.server_name == *server)
            .and_then(|client| client.tool_schema(tool))
    }

    async fn execute_database_call_plans(&self, output: &mut Value) -> Result<usize> {
        let calls = collect_database_calls(output);
        if calls.is_empty() {
            return Ok(0);
        }

        let Some(database_client) = self
            .clients
            .iter()
            .find(|client| client.server_name == "database")
        else {
            return Err(anyhow::anyhow!(
                "Database MCP client is not available for persistence plan"
            ));
        };

        let mut alias_outputs: HashMap<String, Value> = HashMap::new();
        let mut executed = Vec::new();

        for call in calls {
            let tool_name = call
                .get("toolName")
                .or_else(|| call.get("tool_name"))
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Persistence call is missing toolName"))?;
            let args = call
                .get("arguments")
                .or_else(|| call.get("args"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let resolved_args =
                sanitize_tool_params(resolve_persistence_placeholders(args, &alias_outputs));
            if let Some(unresolved) = find_unresolved_placeholder(&resolved_args) {
                return Err(anyhow::anyhow!(
                    "Persistence placeholder {} could not be resolved before Database MCP call {}",
                    unresolved,
                    tool_name
                ));
            }
            database_client.tool_validation(tool_name, &resolved_args)?;
            let result = database_client.call_tool(tool_name, resolved_args).await?;
            let result_value = serde_json::to_value(&result)?;
            if !result.ok {
                return Err(anyhow::anyhow!(
                    "Database MCP call {} failed: {}",
                    tool_name,
                    result
                        .error
                        .as_ref()
                        .map(|error| error.message.as_str())
                        .unwrap_or("unknown error")
                ));
            }
            if let Some(alias) = call
                .get("resultAlias")
                .or_else(|| call.get("result_alias"))
                .and_then(Value::as_str)
            {
                alias_outputs.insert(alias.to_string(), result.data.clone());
            }
            executed.push(json!({
                "toolName": tool_name,
                "result": result_value
            }));
        }

        if let Some(object) = output.as_object_mut() {
            object.insert(
                "executedPersistenceCalls".to_string(),
                Value::Array(executed),
            );
        }
        Ok(alias_outputs.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolves_placeholder_with_colon_alias_and_suffix_path() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "phase:orientation".to_string(),
            json!({ "id": "11111111-1111-1111-1111-111111111111" }),
        );

        assert_eq!(
            resolve_placeholder_string("${phase:orientation}.id", &aliases),
            Some("11111111-1111-1111-1111-111111111111".to_string())
        );
    }

    #[test]
    fn resolves_placeholder_with_colon_alias_and_inner_path() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "phase:orientation".to_string(),
            json!({ "id": "11111111-1111-1111-1111-111111111111" }),
        );

        assert_eq!(
            resolve_placeholder_string("${phase:orientation.id}", &aliases),
            Some("11111111-1111-1111-1111-111111111111".to_string())
        );
    }

    #[test]
    fn trusted_auth_injection_omits_null_optional_strings_and_repairs_user_id() {
        let mut context = crate::AgentContext::default();
        context.auth_context = Some(AuthContext {
            user_id: "firebase-user".to_string(),
            roles: vec![],
            scopes: vec!["lesson:write".to_string()],
            verified: true,
            verified_by: None,
            verified_at: None,
        });

        let params = inject_trusted_auth(
            json!({
                "requestId": Value::Null,
                "authContext": Value::Null,
                "userId": Value::Null,
                "lessonId": "lesson-a"
            }),
            &context,
        )
        .unwrap();
        let sanitized = sanitize_tool_params(params);

        assert_eq!(sanitized["userId"], "firebase-user");
        assert!(sanitized.get("requestId").is_none());
        assert!(sanitized["authContext"].get("verifiedBy").is_none());
        assert!(sanitized["authContext"].get("verifiedAt").is_none());
    }
}

async fn sync_authenticated_user(
    clients: &[McpClient],
    auth_context: Option<&AuthContext>,
) -> Result<()> {
    let Some(auth_context) = auth_context else {
        return Ok(());
    };
    if !auth_context.verified || auth_context.user_id.trim().is_empty() {
        return Ok(());
    }

    let Some(database_client) = clients
        .iter()
        .find(|client| client.server_name == "database")
    else {
        return Ok(());
    };

    let params = json!({
        "firebase_id": auth_context.user_id,
        "display_name": Value::Null,
        "email": Value::Null,
    });
    database_client.tool_validation("upsert_user", &params)?;
    let result = database_client.call_tool("upsert_user", params).await?;
    if !result.ok {
        return Err(anyhow::anyhow!(
            "AUTH_USER_SYNC_FAILED: {}",
            result
                .error
                .as_ref()
                .map(|error| error.message.as_str())
                .unwrap_or("database.upsert_user returned an error")
        ));
    }

    Ok(())
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn llm_client_from_env() -> Result<Client<OpenAIConfig>> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "LLM_API_KEY_MISSING: set OPENROUTER_API_KEY in the runtime environment"
            )
        })?;

    Ok(llm_client_with_key(&api_key))
}

fn llm_client_with_key(api_key: &str) -> Client<OpenAIConfig> {
    let api_base = std::env::var("OPENROUTER_API_BASE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
    let config = OpenAIConfig::new()
        .with_api_base(api_base)
        .with_api_key(api_key);

    Client::with_config(config)
}

pub fn planner_model() -> String {
    std::env::var("AGENT_PLANNER_MODEL")
        .or_else(|_| std::env::var("AGENT_MODEL"))
        .unwrap_or_else(|_| "openai/gpt-oss-120b:free".to_string())
}

pub fn executor_model() -> String {
    std::env::var("AGENT_EXECUTOR_MODEL")
        .or_else(|_| std::env::var("AGENT_MODEL"))
        .unwrap_or_else(|_| "openai/gpt-oss-120b:free".to_string())
}

fn inject_trusted_auth(params: Value, context: &crate::AgentContext) -> Result<Value> {
    let Some(auth_context) = &context.auth_context else {
        return Err(anyhow::anyhow!("trusted auth_context is not present"));
    };
    if !auth_context.verified {
        return Err(anyhow::anyhow!("trusted auth_context is not verified"));
    }
    let mut object = params.as_object().cloned().unwrap_or_default();
    if !object
        .get("authContext")
        .is_some_and(|value| !value.is_null())
    {
        object.insert("authContext".to_string(), trusted_auth_json(auth_context));
    }
    let trusted_user_id = context
        .user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(auth_context.user_id.as_str());
    if !object
        .get("userId")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        object.insert(
            "userId".to_string(),
            Value::String(trusted_user_id.to_string()),
        );
    }
    Ok(Value::Object(object))
}

fn trusted_auth_json(auth_context: &AuthContext) -> Value {
    let mut object = json!({
        "userId": auth_context.user_id,
        "verified": auth_context.verified,
        "scope": auth_context.scopes,
    })
    .as_object()
    .cloned()
    .unwrap_or_default();
    if let Some(verified_by) = &auth_context.verified_by {
        object.insert(
            "verifiedBy".to_string(),
            Value::String(verified_by.to_string()),
        );
    }
    if let Some(verified_at) = &auth_context.verified_at {
        object.insert(
            "verifiedAt".to_string(),
            Value::String(verified_at.to_string()),
        );
    }
    Value::Object(object)
}

fn sanitize_tool_params(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter_map(|(key, value)| {
                    if value.is_null() {
                        None
                    } else {
                        Some((key, sanitize_tool_params(value)))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize_tool_params).collect()),
        other => other,
    }
}

fn collect_database_calls(value: &Value) -> Vec<Value> {
    let mut calls = Vec::new();
    collect_database_calls_inner(value, &mut calls);
    let mut seen = HashSet::new();
    calls
        .into_iter()
        .filter(|call| seen.insert(serde_json::to_string(call).unwrap_or_default()))
        .collect()
}

fn collect_database_calls_inner(value: &Value, calls: &mut Vec<Value>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::Array(items)) = object.get("databaseMcpCalls") {
                calls.extend(items.iter().cloned());
            }
            if let Some(Value::Array(items)) = object.get("database_mcp_calls") {
                calls.extend(items.iter().cloned());
            }
            for child in object.values() {
                collect_database_calls_inner(child, calls);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_database_calls_inner(item, calls);
            }
        }
        _ => {}
    }
}

fn resolve_persistence_placeholders(value: Value, aliases: &HashMap<String, Value>) -> Value {
    match value {
        Value::String(text) => resolve_placeholder_string(&text, aliases)
            .map(Value::String)
            .unwrap_or(Value::String(text)),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| resolve_persistence_placeholders(item, aliases))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, item)| (key, resolve_persistence_placeholders(item, aliases)))
                .collect(),
        ),
        other => other,
    }
}

fn resolve_placeholder_string(text: &str, aliases: &HashMap<String, Value>) -> Option<String> {
    let text = text.strip_prefix("${")?;
    let (inner, suffix_path) = match text.split_once('}') {
        Some((inner, suffix)) => (inner, suffix.strip_prefix('.')),
        None => (text.strip_suffix('}')?, None),
    };
    let mut parts = inner.split('.');
    let alias = parts.next()?;
    let mut current = aliases.get(alias)?;
    for part in parts {
        current = current.get(part)?;
    }
    if let Some(suffix_path) = suffix_path {
        for part in suffix_path.split('.') {
            if part.is_empty() {
                continue;
            }
            current = current.get(part)?;
        }
    }
    current
        .as_str()
        .map(ToString::to_string)
        .or_else(|| Some(current.to_string()))
}

fn find_unresolved_placeholder(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            if text.contains("${") {
                Some(text.clone())
            } else {
                None
            }
        }
        Value::Array(items) => items.iter().find_map(find_unresolved_placeholder),
        Value::Object(object) => object.values().find_map(find_unresolved_placeholder),
        _ => None,
    }
}
