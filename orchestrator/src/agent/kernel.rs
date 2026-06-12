use async_openai::types::chat::ResponseFormat::JsonObject;
use serde_json::{Value, json};

use crate::{AGENT_TESTING, EvaluationDecision, EvaluationStep, ExecutionState, ExecutionStatus, McpClient, PlanStep, PromptBuilder, ServerConfig, StepActions, StepBinding, StepExecutionResult};
use async_openai::{Client, config::OpenAIConfig, types::{self, chat::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest}}};
use anyhow::Result;
use tracing::{error, info};

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
            let step_result = self.execute_step(step, &binding, &prompt).await;
            binding.apply_output(&mut self.state.context, &step_result.output);

            // Evaluation Phase
            let evaluation = EvaluationStep::evaluate(step.id.clone(), &step_result).await;
            match evaluation.decision {
                EvaluationDecision::Continue => {
                    self.state.current_step += 1;
                },
                EvaluationDecision::Wait => {
                    self.state.status = ExecutionStatus::Waiting(step_result.observation.unwrap().clone());
                },
                EvaluationDecision::Replan => {
                    self.state.status = ExecutionStatus::RePlanning(step_result.observation.as_ref().unwrap().clone());
                    self.state.plan = PlanStep::re_plan(&self.planner, &prompt, &self.state.context, step_result.observation.unwrap().clone()).await?;
                    continue;
                },
                EvaluationDecision::Finish => {
                    self.state.status = ExecutionStatus::Completed;
                    break;
                }
            }
            info!("Step {} execution completed!!!", step.id.clone());
        }

        info!("Done!!!");
        Ok(())
    }

    async fn execute_step(&mut self, step: &PlanStep, binding: &StepBinding, prompt: &PromptBuilder) -> StepExecutionResult {
        
        let params = match binding.resolve_params(&self.state.context, &self.executor, &prompt).await{
            Ok(p) => {
                info!("Resolved Params success for action: {:?}", step.action.clone());
                p
            },
            Err(e) => {
                error!("Failed to resolve params for action: {:?}", e);
                return StepExecutionResult {
                    success: false,
                    output: Value::Null,
                    observation: Some(format!("Failed to resolve params for action: {:?}. Error: {}", step.action.clone(), e)),
                    waiting: false,
                    replan: true,
                }
            }
        };
        
        match &step.action {
            StepActions::ToolCall { server, tool } => {
                let client = match self.clients.iter().find(|c| c.server_name == *server)
                    .ok_or_else(|| anyhow::anyhow!("No client found for server: {}", server)) {
                        Ok(c) => {
                            info!("Found client success!!!");
                            c
                        },
                        Err(e) => {
                            error!("Failed to find client, error: {}", e);
                            return StepExecutionResult {
                                success: false,
                                output: Value::Null,
                                observation: Some(format!("Failed to find client for server: {}. Error: {}", server, e)),
                                waiting: false,
                                replan: true,
                            };
                        }
                    };
                match client.tool_validation(tool, &params){
                    Ok(_) => info!("Tool validation success!!!"),
                    Err(e) => {
                        error!("Tool validation failed, error: {}", e);
                        return StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!("Tool validation failed for server: {}.{}. Error: {}", server, tool, e)),
                            waiting: false,
                            replan: true,
                        };
                    }
                };
                match client.call_tool(tool, params).await {
                    Ok(response) => {
                        StepExecutionResult {
                            success: true,
                            output: response,
                            observation: Some(format!("Call Tool success: {}.{}", server, tool)),
                            waiting: false,
                            replan: false,
                        }
                    },
                    Err(e) => {
                        StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!("Call Tool failed: {}.{}. Error: {}", server, tool, e)),
                            waiting: false,
                            replan: true,
                        }
                    }
                }
            },
            StepActions::Reasoning => {
                let user_prompt = ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Reasoning step with instruction: {:?}", step.step_goal.clone().unwrap_or_default()))
                    .build()
                    .unwrap();
                let system = prompt.build_system_prompt();
                let request = CreateChatCompletionRequest {
                    messages: vec![
                        ChatCompletionRequestMessage::System(system),
                        ChatCompletionRequestMessage::User(user_prompt),
                    ],
                    model: "openai/gpt-oss-20b:free".to_string(),
                    response_format: Some(JsonObject),
                    ..Default::default()
                };
                let response = match self.executor.chat().create(request).await {
                    Ok(resp) => {
                        info!("Reasoning step completed successfully!!!");
                        resp
                    },
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
                let content = match response.choices.first()
                    .and_then(|c| c.message.content.as_deref())
                    .ok_or_else(|| anyhow::anyhow!("No content in reasoning response")) {
                        Ok(content) => content,
                        Err(e) => {
                            error!("Failed to extract content from reasoning response: {}", e);
                            return StepExecutionResult {
                                success: false,
                                output: Value::Null,
                                observation: Some(format!("Failed to extract content from reasoning response. Error: {}", e)),
                                waiting: false,
                                replan: true,
                            };
                        }
                    };
                match serde_json::from_str::<Value>(content){
                    Ok(json) => {
                        StepExecutionResult {
                            success: true,
                            output: json,
                            observation: Some(format!("Reasoning step completed: {:?}", step.step_goal.clone().unwrap_or_default())),
                            waiting: false,
                            replan: false,
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse reasoning response as JSON: {}", e);
                        StepExecutionResult {
                            success: false,
                            output: Value::Null,
                            observation: Some(format!("Failed to parse reasoning response as JSON. Error: {}", e)),
                            waiting: false,
                            replan: true,
                        }
                    }
                }
                
            },
            StepActions::HumanApproval => {
                let output = json!({});
                StepExecutionResult {
                    success: true,
                    output,
                    observation: Some(format!("Human approval needed for step: {:?}", step.step_goal.clone().unwrap_or_default())),
                    waiting: true,
                    replan: false,
                }
            },
        }
    }
}

mod test {
}
