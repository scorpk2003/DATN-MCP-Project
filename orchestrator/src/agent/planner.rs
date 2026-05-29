use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanStep {
    pub id: String,
    pub action: StepActions,
    pub step_goal: Option<String>,
    pub dependencies: Vec<String>,
}

impl PlanStep {
    // pub fn calling_tool()
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
