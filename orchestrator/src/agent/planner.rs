use serde::{Deserialize, Serialize};
use serde_json::{Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanStep {
    pub id: String,
    pub action: StepActions,
    pub input: InputResolver,
    pub output: OutputTarget,
    pub waitting: bool,
    pub re_plan: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputResolver {
    Context(Vec<ContextKey>),
    LlmResolved,
    Static(Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OutputTarget{
    Field(String),
    Scratchpad(String),
    FieldAndScratchpad { field: String, scratchpad: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextKey {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StepActions {
    ToolCall {
        server: String,
        tool: String,
    },
    Reasoning,
    HumanApproval,
}
