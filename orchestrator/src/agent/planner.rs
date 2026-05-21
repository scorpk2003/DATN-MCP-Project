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

impl PlanStep {
    // pub fn calling_tool()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum InputResolver {
    Context{ keys: Vec<ContextKey> },
    LlmResolved,
    Static { value: Value },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OutputTarget{
    Field{ name: String },
    Scratchpad{ name: String },
    FieldAndScratchpad { field: String, scratchpad: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextKey {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StepActions {
    ToolCall {
        server: String,
        tool: String,
    },
    Reasoning,
    HumanApproval,
}
