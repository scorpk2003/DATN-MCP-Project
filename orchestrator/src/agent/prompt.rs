use async_openai::types::chat::{ChatCompletionRequestSystemMessage, 
    ChatCompletionRequestSystemMessageArgs};
use tokio::fs;


pub struct PromptBuilder {
    pub identity: String,
    pub tool_rules: String,
    pub phase_rules: Option<String>,
    pub phasing: Option<Phase>,
}

pub enum Phase {
    Planning,
    Binding,
    Execution,
    Failure,
    Testing,
}

impl PromptBuilder {
    pub async fn new() -> Self {
        let identity = fs::read_to_string("src/prompt/agent.md").await.expect("Failed to load Agent Prompt");
        let tool_rules = String::from("");
        let phase_rules = None;
        Self { identity, tool_rules, phase_rules, phasing: None }
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
            Some(Phase::Failure) => "failure",
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

    pub async fn build_planning_phase(&mut self) {
        let planning_phase_rules = fs::read_to_string("src/prompt/plan.md").await.expect("Failed to load Planning Phase Prompt");
        self.phase_rules = Some(planning_phase_rules);
        self.phasing = Some(Phase::Planning);
    }
    
    pub async fn build_binding_phase(&mut self) {
        let binding_phase_rules = fs::read_to_string("src/prompt/resolver.md").await.expect("Failed to load Binding Phase Prompt");
        self.phase_rules = Some(binding_phase_rules);
        self.phasing = Some(Phase::Binding);
    }

    pub async fn build_failure_phase(&mut self) {
        let failure_phase_rules = fs::read_to_string("src/prompt/failure.md").await.expect("Failed to load Failure Phase Prompt");
        self.phase_rules = Some(failure_phase_rules);
        self.phasing = Some(Phase::Failure);
    }

    pub async fn build_testing_phase(&mut self) {
        let testing_phase_rules = fs::read_to_string("src/prompt/test.md").await.expect("Failed to load Testing Phase Prompt");
        let testing_phase = self.phase_rules.clone().unwrap_or_default() + "\n\n" + &testing_phase_rules;
        self.phase_rules = Some(testing_phase);
        self.phasing = Some(Phase::Testing);
    }

}

mod test {
    #[tokio::test]
    async fn test_constructor() {
        use super::*;
        let prompt = PromptBuilder::new().await;
        println!("{}", prompt.identity);
        println!("{}", prompt.tool_rules);
        println!("{}", prompt.phase_rules.unwrap_or_default());
    }

    #[tokio::test]
    async fn test_build_system_prompt() {
        use super::*;
        let mut prompt = PromptBuilder::new().await;
        prompt.build_planning_phase().await;
        let system_prompt = prompt.build_system_prompt();
        println!("{:?}", system_prompt);
    }
}