use async_openai::types::chat::{ChatCompletionRequestSystemMessage, 
    ChatCompletionRequestSystemMessageArgs};
use tokio::fs;


pub struct PromptBuilder {
    pub identity: String,
    pub tool_rules: String,
    pub planning_rules: String,
}

impl PromptBuilder {
    pub async fn new() -> Self {
        let identity = fs::read_to_string("src/prompt/agent.md").await.expect("Failed to load Agent Prompt");
        let tool_rules = String::from("value");
        let planning_rules = fs::read_to_string("src/prompt/plan.md").await.expect("Failed to load Planning Prompt");
        Self { identity, tool_rules, planning_rules }
    }

    pub fn build_system_prompt(&self) -> ChatCompletionRequestSystemMessage {
        let mut prompt = Vec::new();
        prompt.push(self.identity.clone());
        prompt.push(self.planning_rules.clone());
        prompt.push(self.tool_rules.clone());
        let system_prompt = prompt.join("\n\n");
        let message = ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()
            .unwrap();
        message
    }

    pub async fn build_system_test_prompt(&self) -> ChatCompletionRequestSystemMessage {
        let testing_rules = fs::read_to_string("src/prompt/test.md").await.expect("Failed to load Testing Prompt");
        let mut prompt = Vec::new();
        prompt.push(self.identity.clone());
        prompt.push(self.planning_rules.clone());
        prompt.push(self.tool_rules.clone());
        prompt.push(testing_rules);
        let system_prompt = prompt.join("\n\n");
        let message = ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()
            .unwrap();
        message
    }
}

mod test {
    #[tokio::test]
    async fn test_constructor() {
        use super::*;
        let prompt = PromptBuilder::new().await;
        println!("{}", prompt.identity);
        println!("{}", prompt.tool_rules);
        println!("{}", prompt.planning_rules);
    }

    #[tokio::test]
    async fn test_build_system_prompt() {
        use super::*;
        let prompt = PromptBuilder::new().await;
        let system_prompt = prompt.build_system_prompt();
        println!("{:?}", system_prompt);
    }
}