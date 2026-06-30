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

const CRITICAL_IDENTIFIER_FIELDS: &[&str] = &[
    "userId",
    "roadmapId",
    "roadmapNodeId",
    "lessonId",
    "sessionId",
    "activityId",
    "exerciseId",
    "topicId",
];

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
        intent_context: Option<Value>,
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
        let mut state = ExecutionState::new(session_id);
        state.context.session_id = state.session_id.clone();
        state.context.user_id = user_id;
        state.context.auth_context = auth_context;
        state.context.intent_context = intent_context;
        sync_authenticated_user(&clients, state.context.auth_context.as_ref()).await?;
        let planner = llm_client_from_env()?;
        let executor = llm_client_from_env()?;
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
            state.context.intent_context.clone(),
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
        repair_approval_only_review_plan(
            &mut self.state.plan,
            self.state.context.intent_context.as_ref(),
        );
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
                if server == "lesson" && tool == "lesson_finalize" {
                    params = hydrate_lesson_finalize_level(params, &self.state.context);
                }
                let selected_schema = client.tool_schema(tool);
                if let Some(schema) = selected_schema.as_ref() {
                    params = hydrate_required_identifiers_from_context(
                        params,
                        schema,
                        &self.state.context,
                    );
                    let missing_identifiers = missing_required_identifiers(&params, schema);
                    if !missing_identifiers.is_empty() {
                        let missing_message = missing_identifiers.join(", ");
                        return StepExecutionResult {
                            success: false,
                            output: json!({
                                "code": "MISSING_REQUIRED_CONTEXT",
                                "server": server,
                                "tool": tool,
                                "missing": missing_identifiers,
                            }),
                            observation: Some(format!(
                                "MISSING_REQUIRED_CONTEXT: {}.{} requires {} from existing context before tool execution.",
                                server, tool, missing_message
                            )),
                            waiting: false,
                            replan: true,
                        };
                    }
                }
                

                match client.tool_validation(tool, &params) {
                    Ok(_) => info!("Tool validation success!!!"),
                    Err(e) => {
                        let validation_error = e.to_string();
                        error!("Tool validation failed, error: {}", validation_error);
                        let max_repairs = env_usize("AGENT_MAX_PARAM_REPAIRS", 1);
                        let mut repaired = false;
                        for repair_attempt in 0..max_repairs {
                            let Some(schema) = selected_schema.as_ref() else {
                                break;
                            };
                            match StepBinding::repair_params(
                                step,
                                &params,
                                schema,
                                &validation_error,
                                &self.state.context,
                                &self.executor,
                                prompt,
                            )
                            .await
                            {
                                Ok(candidate) => {
                                    let candidate = preserve_required_identifiers(
                                        &params,
                                        sanitize_tool_params(candidate),
                                        schema,
                                    );
                                    if !missing_required_identifiers(&candidate, schema).is_empty()
                                    {
                                        error!(
                                            repair_attempt,
                                            "Repaired params removed required identity fields"
                                        );
                                        continue;
                                    }
                                    match client.tool_validation(tool, &candidate) {
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
                                                "Repaired params still failed validation: {}",
                                                error
                                            );
                                        }
                                    }
                                }
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

fn repair_approval_only_review_plan(plan: &mut Vec<PlanStep>, intent_context: Option<&Value>) {
    if plan.len() != 1 || !matches!(plan[0].action, StepActions::HumanApproval) {
        return;
    }

    let intent_type = intent_context
        .and_then(Value::as_object)
        .and_then(|context| context.get("type"))
        .and_then(Value::as_str);
    if !matches!(
        intent_type,
        Some("review.task.selected") | Some("note.review.requested")
    ) {
        return;
    }

    plan.push(PlanStep {
        id: "step 2".to_string(),
        action: StepActions::Reasoning,
        step_goal: Some(
            "After approval, produce a concise JSON lessonDraft for the requested review. Include title, topic, objectives, contentBlocks, resources, exercises, and status. Use intent_context taskId/noteId/concept/title as the review subject; do not return an empty object."
                .to_string(),
        ),
        dependencies: vec![plan[0].id.clone()],
        final_output: Some("lesson".to_string()),
    });
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
    if object
        .get("authContext")
        .is_none_or(|value| value.is_null())
    {
        object.insert("authContext".to_string(), trusted_auth_json(auth_context));
    }
    let trusted_user_id = context
        .user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(auth_context.user_id.as_str());
    if object
        .get("userId")
        .and_then(Value::as_str)
        .is_none_or(|value| value.trim().is_empty())
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

fn hydrate_required_identifiers_from_context(
    params: Value,
    schema: &Value,
    context: &crate::AgentContext,
) -> Value {
    let Value::Object(mut object) = params else {
        return params;
    };

    for field in required_critical_identifier_fields(schema) {
        if has_non_empty_value(object.get(field)) {
            continue;
        }
        if let Some(value) = context_identifier_value(context, field) {
            object.insert(field.to_string(), value);
        }
    }

    Value::Object(object)
}

fn preserve_required_identifiers(original: &Value, candidate: Value, schema: &Value) -> Value {
    let Some(original_object) = original.as_object() else {
        return candidate;
    };
    let mut candidate_object = match candidate {
        Value::Object(object) => object,
        other => return other,
    };

    for field in required_critical_identifier_fields(schema) {
        if let Some(value) = original_object
            .get(field)
            .filter(|value| has_non_empty_value(Some(value)))
        {
            candidate_object.insert(field.to_string(), value.clone());
        }
    }

    Value::Object(candidate_object)
}

fn missing_required_identifiers(params: &Value, schema: &Value) -> Vec<String> {
    let object = params.as_object();
    required_critical_identifier_fields(schema)
        .into_iter()
        .filter(|field| !has_non_empty_value(object.and_then(|params| params.get(*field))))
        .map(str::to_string)
        .collect()
}

fn required_critical_identifier_fields(schema: &Value) -> Vec<&'static str> {
    let Some(required) = schema.get("required").and_then(Value::as_array) else {
        return Vec::new();
    };

    CRITICAL_IDENTIFIER_FIELDS
        .iter()
        .copied()
        .filter(|field| {
            required
                .iter()
                .any(|required_field| required_field.as_str() == Some(*field))
        })
        .collect()
}

fn context_identifier_value(context: &crate::AgentContext, field: &str) -> Option<Value> {
    match field {
        "userId" => context
            .user_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                context
                    .auth_context
                    .as_ref()
                    .map(|auth_context| auth_context.user_id.as_str())
                    .filter(|value| !value.trim().is_empty())
            })
            .map(|value| Value::String(value.to_string())),
        "sessionId" => (!context.session_id.trim().is_empty())
            .then(|| Value::String(context.session_id.clone())),
        _ => context
            .intent_context
            .as_ref()
            .and_then(|intent_context| lookup_intent_identifier(intent_context, field)),
    }
}

fn lookup_intent_identifier(intent_context: &Value, field: &str) -> Option<Value> {
    let object = intent_context.as_object()?;
    let direct = object
        .get(field)
        .filter(|value| has_non_empty_value(Some(value)));
    if direct.is_some() {
        return direct.cloned();
    }

    let alias = match field {
        "roadmapNodeId" => Some("nodeId"),
        "activityId" => Some("exerciseId"),
        _ => None,
    };
    alias.and_then(|alias| {
        object
            .get(alias)
            .filter(|value| has_non_empty_value(Some(value)))
            .cloned()
    })
}

fn has_non_empty_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(Value::Null) | None => false,
        Some(_) => true,
    }
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

fn hydrate_lesson_finalize_level(params: Value, context: &crate::AgentContext) -> Value {
    let Value::Object(mut object) = params else {
        return params;
    };
    let Some(draft) = object.get_mut("lessonDraft").and_then(Value::as_object_mut) else {
        return Value::Object(object);
    };
    if has_non_empty_value(draft.get("level")) {
        return Value::Object(object);
    }

    let level = lesson_level_from_context(context).unwrap_or("beginner");
    draft.insert("level".to_string(), Value::String(level.to_string()));
    Value::Object(object)
}

fn lesson_level_from_context(context: &crate::AgentContext) -> Option<&'static str> {
    let intent = context.intent_context.as_ref()?.as_object()?;
    intent
        .get("level")
        .or_else(|| intent.get("difficulty"))
        .and_then(Value::as_str)
        .and_then(normalize_lesson_level)
}

fn normalize_lesson_level(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "beginner" | "easy" => Some("beginner"),
        "intermediate" | "medium" => Some("intermediate"),
        "advanced" | "hard" | "expert" => Some("advanced"),
        _ => None,
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
        let context = crate::AgentContext {
            auth_context: Some(AuthContext {
                user_id: "firebase-user".to_string(),
                roles: vec![],
                scopes: vec!["lesson:write".to_string()],
                verified: true,
                verified_by: None,
                verified_at: None,
            }),
            ..crate::AgentContext::default()
        };

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

    #[test]
    fn hydrates_required_identifiers_from_intent_context() {
        let context = crate::AgentContext {
            session_id: "session-a".to_string(),
            user_id: Some("user-a".to_string()),
            intent_context: Some(json!({
                "type": "roadmap.node.selected",
                "roadmapId": "roadmap-a",
                "nodeId": "node-a"
            })),
            ..crate::AgentContext::default()
        };
        let schema = json!({
            "type": "object",
            "required": ["userId", "roadmapId", "roadmapNodeId", "node"]
        });

        let params = hydrate_required_identifiers_from_context(
            json!({
                "roadmapId": Value::Null,
                "node": {"title": "Intro"}
            }),
            &schema,
            &context,
        );

        assert_eq!(params["userId"], "user-a");
        assert_eq!(params["roadmapId"], "roadmap-a");
        assert_eq!(params["roadmapNodeId"], "node-a");
        assert!(missing_required_identifiers(&params, &schema).is_empty());
    }

    #[test]
    fn repairs_review_approval_only_plan_with_followup_reasoning() {
        let mut plan = vec![PlanStep {
            id: "step 1".to_string(),
            action: StepActions::HumanApproval,
            step_goal: Some("Approve review".to_string()),
            dependencies: Vec::new(),
            final_output: None,
        }];

        repair_approval_only_review_plan(
            &mut plan,
            Some(&json!({
                "type": "review.task.selected",
                "taskId": "task-a",
                "concept": "Core concept"
            })),
        );

        assert_eq!(plan.len(), 2);
        assert!(matches!(plan[1].action, StepActions::Reasoning));
        assert_eq!(plan[1].dependencies, vec!["step 1".to_string()]);
        assert_eq!(plan[1].final_output.as_deref(), Some("lesson"));
    }

    #[test]
    fn detects_missing_required_identifiers_after_sanitize() {
        let schema = json!({
            "type": "object",
            "required": ["roadmapId", "roadmapNodeId", "node"]
        });
        let params = sanitize_tool_params(json!({
            "roadmapId": Value::Null,
            "roadmapNodeId": "   ",
            "node": {"title": "Intro"}
        }));

        assert_eq!(
            missing_required_identifiers(&params, &schema),
            vec!["roadmapId".to_string(), "roadmapNodeId".to_string()]
        );
    }

    #[test]
    fn repair_candidate_preserves_existing_required_identifiers() {
        let schema = json!({
            "type": "object",
            "required": ["userId", "roadmapId", "roadmapNodeId", "node"]
        });
        let original = json!({
            "userId": "user-a",
            "roadmapId": "roadmap-a",
            "roadmapNodeId": "node-a",
            "node": {"title": "Intro"}
        });
        let candidate = preserve_required_identifiers(
            &original,
            json!({
                "userId": "other-user",
                "roadmapId": "other-roadmap",
                "node": {"title": "Intro", "topic": "Rust"}
            }),
            &schema,
        );

        assert_eq!(candidate["userId"], "user-a");
        assert_eq!(candidate["roadmapId"], "roadmap-a");
        assert_eq!(candidate["roadmapNodeId"], "node-a");
        assert_eq!(candidate["node"]["topic"], "Rust");
    }

    #[test]
    fn hydrates_missing_lesson_finalize_level_from_intent_context() {
        let context = crate::AgentContext {
            intent_context: Some(json!({
                "type": "roadmap.task.selected",
                "level": "medium"
            })),
            ..crate::AgentContext::default()
        };

        let params = hydrate_lesson_finalize_level(
            json!({
                "lessonDraft": {
                    "title": "SQL joins",
                    "topic": "SQL joins"
                }
            }),
            &context,
        );

        assert_eq!(params["lessonDraft"]["level"], "intermediate");
    }

    #[test]
    fn defaults_missing_lesson_finalize_level_to_beginner() {
        let params = hydrate_lesson_finalize_level(
            json!({
                "lessonDraft": {
                    "title": "SQL joins",
                    "topic": "SQL joins"
                }
            }),
            &crate::AgentContext::default(),
        );

        assert_eq!(params["lessonDraft"]["level"], "beginner");
    }

}
