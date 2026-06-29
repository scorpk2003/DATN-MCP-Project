use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    AgentContext, PromptBuilder, agent_testing_enabled, parse_llm_json_value, planner_model,
};
use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        self,
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequest,
        },
    },
};
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanStep {
    pub id: String,
    pub action: StepActions,
    pub step_goal: Option<String>,
    pub dependencies: Vec<String>,
    pub final_output: Option<String>,
}

impl PlanStep {
    pub async fn plan(
        goal: String,
        planner: &Client<OpenAIConfig>,
        prompt: &PromptBuilder,
    ) -> Result<(Vec<Self>, Option<String>)> {
        // Build system prompt
        let mut prompt_build = prompt.clone();
        prompt_build
            .build_planning_phase(agent_testing_enabled())
            .await;
        let system_prompt = prompt_build.build_system_prompt();

        // Build user prompt
        let user_prompt = ChatCompletionRequestUserMessageArgs::default()
            .content(goal)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build planner prompt: {e}"))?;
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(user_prompt),
            ],
            model: planner_model(),
            response_format: Some(types::chat::ResponseFormat::JsonObject),
            ..Default::default()
        };

        // Planning
        let response_content = match planner.chat().create(request).await {
            Ok(res) => {
                println!("\t\tPlan generated successfully: \n{:#?}\n\n", res);
                info!("\t\tPlan generated successfully: \n{:#?}\n\n", res);
                res
            }
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
        let step = parse_llm_json_value(content, "PLAN_PARSE_ERROR")
            .map_err(|f| anyhow::anyhow!("Failed to parse plan response: {}", f))?;

        // Parsing
        let response: Vec<Self> = match step.get("steps") {
            Some(steps) => serde_json::from_value(steps.clone())
                .map_err(|f| anyhow::anyhow!("Failed to parse steps: {}", f))?,
            None => {
                println!("No steps found in planner response: {}", content);
                Vec::new()
            }
        };
        validate_plan_tools(&response, prompt)?;
        let step_goal = match step.get("goal") {
            Some(goals) => serde_json::from_value::<String>(goals.clone())
                .map_err(|f| anyhow::anyhow!("Failed to parse step goals: {}", f))?,
            None => {
                println!("No step goals found in planner response: {}", content);
                String::new()
            }
        };

        Ok((response, Some(step_goal)))
    }

    pub async fn re_plan(
        planner: &Client<OpenAIConfig>,
        prompt: &PromptBuilder,
        context: &AgentContext,
        observation: String,
    ) -> Result<Vec<Self>> {
        let mut prompt_build = prompt.clone();
        prompt_build
            .build_planning_phase(agent_testing_enabled())
            .await;
        let system_prompt = prompt_build.build_system_prompt();
        let last_obs = [
            "Step execute failed, this is full observation!!!".to_string(),
            observation,
            context.last_obs().unwrap_or_default().to_string(),
        ];
        let user_prompt = ChatCompletionRequestUserMessageArgs::default()
            .content(last_obs.join("\n\n"))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build re-plan prompt: {e}"))?;
        let request = CreateChatCompletionRequest {
            messages: vec![
                ChatCompletionRequestMessage::System(system_prompt),
                ChatCompletionRequestMessage::User(user_prompt),
            ],
            model: planner_model(),
            response_format: Some(types::chat::ResponseFormat::JsonObject),
            ..Default::default()
        };
        let response_content = match planner.chat().create(request).await {
            Ok(res) => {
                println!("\t\tRe-plan generated successfully: \n{:#?}\n\n", res);
                info!("\t\tRe-plan generated successfully: \n{:#?}\n\n", res);
                res
            }
            Err(e) => {
                println!("Failed to generate re-plan: {}", e);
                info!("Failed to generate re-plan: {}", e);
                return Err(anyhow::anyhow!("Failed to generate re-plan: {}", e));
            }
        };
        let content = response_content
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No content in re-planner response"))?;
        let step = parse_llm_json_value(content, "REPLAN_PARSE_ERROR")
            .map_err(|f| anyhow::anyhow!("Failed to parse re-plan response: {}", f))?;
        let response: Vec<Self> = match step.get("steps") {
            Some(steps) => serde_json::from_value(steps.clone())
                .map_err(|f| anyhow::anyhow!("Failed to parse re-plan steps: {}", f))?,
            None => {
                println!("No steps found in re-planner response: {}", content);
                match step.get("cause") {
                    Some(step) => {
                        let cause = step.as_str().unwrap_or("unknown re-plan refusal");
                        return Err(anyhow::anyhow!(
                            "Failed to Re-plan due to dangerous cause: {}",
                            cause
                        ));
                    }
                    None => {
                        return Err(anyhow::anyhow!("Nothing in re-plan"));
                    }
                }
            }
        };
        validate_plan_tools(&response, prompt)?;
        Ok(response)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StepActions {
    ToolCall { server: String, tool: String },
    Reasoning,
    HumanApproval,
}

impl<'de> Deserialize<'de> for StepActions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("action must be a JSON object"))?;
        let action_type = object
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("action.type must be a string"))?;

        match action_type {
            "ToolCall" => {
                let server = object
                    .get("server")
                    .and_then(Value::as_str)
                    .ok_or_else(|| serde::de::Error::custom("ToolCall action requires server"))?;
                let tool = object
                    .get("tool")
                    .and_then(Value::as_str)
                    .ok_or_else(|| serde::de::Error::custom("ToolCall action requires tool"))?;
                Ok(Self::ToolCall {
                    server: server.to_string(),
                    tool: tool.to_string(),
                })
            }
            "Reasoning" => Ok(Self::Reasoning),
            "HumanApproval" => Ok(Self::HumanApproval),
            function_style if function_style.contains('.') => {
                let (server, tool) = function_style.split_once('.').ok_or_else(|| {
                    serde::de::Error::custom("function-style action must be server.tool")
                })?;
                if server.trim().is_empty() || tool.trim().is_empty() {
                    return Err(serde::de::Error::custom(
                        "function-style action must include non-empty server and tool",
                    ));
                }
                Ok(Self::ToolCall {
                    server: server.to_string(),
                    tool: tool.to_string(),
                })
            }
            other => Err(serde::de::Error::custom(format!(
                "unsupported action.type '{other}'"
            ))),
        }
    }
}

fn validate_plan_tools(steps: &[PlanStep], prompt: &PromptBuilder) -> Result<()> {
    for step in steps {
        let StepActions::ToolCall { server, tool } = &step.action else {
            continue;
        };
        if !prompt.has_tool(server, tool) {
            return Err(anyhow::anyhow!(
                "PLAN_TOOL_NOT_FOUND: step '{}' selected invalid tool '{}.{}'. Available exact tools:\n{}",
                step.id,
                server,
                tool,
                prompt.catalog_summary()
            ));
        }
    }
    Ok(())
}

mod test {
    use crate::agent::prompt::ToolCatalogServer;

    use super::*;

    fn prompt_with_catalog() -> PromptBuilder {
        PromptBuilder::from_catalog_for_test(vec![
            ToolCatalogServer {
                server: "roadmap".to_string(),
                tools: vec!["generate_roadmap_from_goal".to_string()],
            },
            ToolCatalogServer {
                server: "lesson".to_string(),
                tools: vec![
                    "lesson_analyze_node".to_string(),
                    "lesson_create_draft".to_string(),
                    "lesson_finalize".to_string(),
                ],
            },
        ])
    }

    #[test]
    fn rejects_hallucinated_tool_name_in_plan() {
        let steps = vec![PlanStep {
            id: "step 1".to_string(),
            action: StepActions::ToolCall {
                server: "roadmap".to_string(),
                tool: "create_roadmap".to_string(),
            },
            step_goal: None,
            dependencies: vec![],
            final_output: None,
        }];

        let error = validate_plan_tools(&steps, &prompt_with_catalog()).unwrap_err();
        assert!(error.to_string().contains("PLAN_TOOL_NOT_FOUND"));
    }

    #[test]
    fn accepts_exact_tool_name_in_plan() {
        let steps = vec![PlanStep {
            id: "step 1".to_string(),
            action: StepActions::ToolCall {
                server: "roadmap".to_string(),
                tool: "generate_roadmap_from_goal".to_string(),
            },
            step_goal: None,
            dependencies: vec![],
            final_output: None,
        }];

        validate_plan_tools(&steps, &prompt_with_catalog()).unwrap();
    }

    #[test]
    fn rejects_database_tool_even_when_llm_wants_raw_uuid_lookup() {
        let steps = vec![PlanStep {
            id: "step 1".to_string(),
            action: StepActions::ToolCall {
                server: "database".to_string(),
                tool: "get_roadmap".to_string(),
            },
            step_goal: Some("Load roadmap by roadmapId from UI".to_string()),
            dependencies: vec![],
            final_output: None,
        }];

        let error = validate_plan_tools(&steps, &prompt_with_catalog()).unwrap_err();
        assert!(error.to_string().contains("PLAN_TOOL_NOT_FOUND"));
    }

    #[test]
    fn normalizes_function_style_action_type_from_llm_plan() {
        let content = r#"{
            "id": "step 1",
            "action": {"type": "lesson.lesson_analyze_node"},
            "step_goal": "Analyze the roadmap node.",
            "dependencies": [],
            "final_output": "lesson_requirements"
        }"#;

        let step = serde_json::from_str::<PlanStep>(content).unwrap();
        match &step.action {
            StepActions::ToolCall { server, tool } => {
                assert_eq!(server, "lesson");
                assert_eq!(tool, "lesson_analyze_node");
            }
            _ => panic!("expected normalized ToolCall"),
        }
        validate_plan_tools(&[step], &prompt_with_catalog()).unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live LLM credentials and can return non-deterministic plans"]
    async fn test_plan() {
        use super::*;
        use crate::AgentKernel;
        dotenv::from_path("../.env").ok();
        let kernel = AgentKernel::default();
        let goal = "Testing: Learn C# programming language".to_string();
        let prompt = PromptBuilder::new(&kernel.clients).await;
        let (plan, step_goal) = PlanStep::plan(goal, &kernel.planner, &prompt)
            .await
            .unwrap();
        for (idx, step) in plan.iter().enumerate() {
            println!("Step {}: {:?}", idx + 1, step);
        }
        println!("Step Goal: {:?}", step_goal);
    }
}
