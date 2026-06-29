use async_openai::types::chat::{
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageArgs,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::McpClient;

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    pub identity: String,
    pub tool_rules: String,
    pub tool_catalog: Vec<ToolCatalogServer>,
    pub phase_rules: Option<String>,
    pub phasing: Option<Phase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCatalogServer {
    pub server: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Planning,
    Binding,
    Execution,
    Testing,
}

impl PromptBuilder {
    pub async fn new(clients: &[McpClient]) -> Self {
        let identity = read_prompt("agent.md")
            .await
            .unwrap_or_else(|_| "Self-Learn Orchestrator Agent".to_string());
        let tool_rules = clients
            .iter()
            .filter_map(public_tool_prompt)
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut tool_catalog = clients
            .iter()
            .filter_map(|client| {
                let mut tools = client
                    .tools
                    .keys()
                    .filter(|tool| planner_visible_tool(&client.server_name, tool))
                    .cloned()
                    .collect::<Vec<_>>();
                tools.sort();
                if tools.is_empty() {
                    return None;
                }
                Some(ToolCatalogServer {
                    server: client.server_name.clone(),
                    tools,
                })
            })
            .collect::<Vec<_>>();
        tool_catalog.sort_by(|left, right| left.server.cmp(&right.server));
        let phase_rules = None;
        Self {
            identity,
            tool_rules,
            tool_catalog,
            phase_rules,
            phasing: None,
        }
    }

    pub fn from_catalog_for_test(tool_catalog: Vec<ToolCatalogServer>) -> Self {
        Self {
            identity: String::new(),
            tool_rules: String::new(),
            tool_catalog,
            phase_rules: None,
            phasing: None,
        }
    }

    pub fn build_system_prompt(&self) -> ChatCompletionRequestSystemMessage {
        let prompt = [
            self.identity.clone(),
            self.phase_rules.clone().unwrap_or_default(),
            self.canonical_tool_catalog_prompt(),
            self.tool_rules.clone(),
        ];
        let name = match self.phasing {
            Some(Phase::Planning) => "planning",
            Some(Phase::Binding) => "binding",
            Some(Phase::Execution) => "execution",
            Some(Phase::Testing) => "testing",
            None => "general",
        };
        let system_prompt = prompt.join("\n\n");
        ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .name(name.to_string())
            .build()
            .unwrap_or_else(|_| ChatCompletionRequestSystemMessage {
                content: async_openai::types::chat::ChatCompletionRequestSystemMessageContent::Text(
                    "Self-Learn Orchestrator Agent".to_string(),
                ),
                name: Some(name.to_string()),
            })
    }

    pub async fn build_planning_phase(&mut self, is_test: bool) {
        let mut planning_phase_rules = read_prompt("plan.md")
            .await
            .unwrap_or_else(|_| "Return a JSON plan with steps and goal.".to_string());
        match is_test {
            true => {
                let test = read_prompt("test.md").await.unwrap_or_default();
                planning_phase_rules.push_str(test.as_str());
                self.phasing = Some(Phase::Testing);
            }
            false => {
                self.phasing = Some(Phase::Planning);
            }
        }
        self.phase_rules = Some(planning_phase_rules);
    }

    pub async fn build_binding_phase(&mut self, is_test: bool) {
        let mut binding_phase_rules = read_prompt("resolver.md")
            .await
            .unwrap_or_else(|_| "Return a JSON object with a binding wrapper.".to_string());
        match is_test {
            true => {
                let test = read_prompt("test.md").await.unwrap_or_default();
                binding_phase_rules.push_str(test.as_str());
                self.phasing = Some(Phase::Testing)
            }
            false => {
                self.phasing = Some(Phase::Binding);
            }
        }
        self.phase_rules = Some(binding_phase_rules);
    }

    pub fn has_tool(&self, server: &str, tool: &str) -> bool {
        self.tool_catalog
            .iter()
            .any(|entry| entry.server == server && entry.tools.iter().any(|name| name == tool))
    }

    pub fn catalog_summary(&self) -> String {
        self.tool_catalog
            .iter()
            .map(|entry| format!("{}: {}", entry.server, entry.tools.join(", ")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn canonical_tool_catalog_prompt(&self) -> String {
        let catalog = serde_json::to_string(&self.tool_catalog).unwrap_or_else(|_| "[]".into());
        format!(
            "Canonical Tool Catalog JSON. ToolCall actions must use only these exact server/tool pairs:\n{catalog}"
        )
    }
}

fn public_tool_prompt(client: &McpClient) -> Option<String> {
    let lines = client
        .build_tool_prompt()
        .into_iter()
        .filter(|line| {
            let prefix = format!("{}.", client.server_name);
            let Some(rest) = line.strip_prefix(&prefix) else {
                return false;
            };
            let Some((tool, _)) = rest.split_once(':') else {
                return false;
            };
            planner_visible_tool(&client.server_name, tool)
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn planner_visible_tool(server: &str, _tool: &str) -> bool {
    server != "database"
}

async fn read_prompt(file_name: &str) -> std::io::Result<String> {
    let prompt_dir =
        std::env::var("ORCHESTRATOR_PROMPT_DIR").unwrap_or_else(|_| "src/prompt".to_string());
    fs::read_to_string(std::path::Path::new(&prompt_dir).join(file_name)).await
}

mod test {
    #[allow(unused)]
    use crate::{AgentKernel, kernel};

    #[tokio::test]
    async fn test_constructor() {
        use super::*;
        let kernel = AgentKernel::default();
        let prompt = PromptBuilder::new(&kernel.clients).await;
        println!("{}", prompt.identity);
        println!("{}", prompt.tool_rules);
        println!("{}", prompt.phase_rules.unwrap_or_default());
    }

    #[tokio::test]
    async fn test_build_system_prompt() {
        use super::*;
        let kernel = AgentKernel::default();
        let mut prompt = PromptBuilder::new(&kernel.clients).await;
        prompt.build_planning_phase(false).await;
        let system_prompt = prompt.build_system_prompt();
        println!("{:?}", system_prompt);
    }
}
