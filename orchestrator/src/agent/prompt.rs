use async_openai::types::chat::{
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageArgs,
};
use tokio::fs;

use crate::McpClient;

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    pub identity: String,
    pub tool_rules: String,
    pub phase_rules: Option<String>,
    pub phasing: Option<Phase>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Planning,
    Binding,
    Execution,
    Testing,
}

impl PromptBuilder {
    pub async fn new(clients: &Vec<McpClient>) -> Self {
        let identity = fs::read_to_string("src/prompt/agent.md")
            .await
            .expect("Failed to load Agent Prompt");
        let tool_rules = clients
            .iter()
            .map(|client| client.build_tool_prompt().join("\n"))
            .collect::<Vec<_>>()
            .join("\n\n");
        let phase_rules = None;
        Self {
            identity,
            tool_rules,
            phase_rules,
            phasing: None,
        }
    }

    pub fn build_system_prompt(&self) -> ChatCompletionRequestSystemMessage {
        let mut prompt = Vec::new();
        prompt.push(self.identity.clone());
        prompt.push(self.phase_rules.clone().unwrap_or_default());
        prompt.push(self.tool_rules.clone());
        let name = match self.phasing {
            Some(Phase::Planning) => "planning",
            Some(Phase::Binding) => "binding",
            Some(Phase::Execution) => "execution",
            Some(Phase::Testing) => "testing",
            None => "general",
        };
        let system_prompt = prompt.join("\n\n");
        let message = ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .name(name.to_string())
            .build()
            .unwrap();
        message
    }

    pub async fn build_planning_phase(&mut self, is_test: bool) {
        let mut planning_phase_rules = fs::read_to_string("src/prompt/plan.md")
            .await
            .expect("Failed to load Planning Phase Prompt");
        match is_test {
            true => {
                let test = fs::read_to_string("src/prompt/test.md")
                    .await
                    .expect("Fail to load Test file for Planning Phase");
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
        let mut binding_phase_rules = fs::read_to_string("src/prompt/resolver.md")
            .await
            .expect("Failed to load Binding Phase Prompt");
        match is_test {
            true => {
                let test = fs::read_to_string("src/prompt/test.md")
                    .await
                    .expect("Fail to load Test file for Binding Phase");
                binding_phase_rules.push_str(test.as_str());
                self.phasing = Some(Phase::Testing)
            }
            false => {
                self.phasing = Some(Phase::Binding);
            }
        }
        self.phase_rules = Some(binding_phase_rules);
    }
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
