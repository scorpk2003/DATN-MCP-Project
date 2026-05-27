use serde_json::Value;

use crate::{AgentContext, PlanStep, StepBinding};


pub enum ExecutionStatus {
    Init,
    Planning,
    Running,
    Waiting(String),
    RePlanning(String),
    Completed,
    Failed(String),
}

pub struct ExecutionState {
    pub session_id: String,
    pub status: ExecutionStatus,
    pub current_step: usize,
    pub plan: Vec<PlanStep>,
    pub context: AgentContext,
    pub resolver: Vec<StepBinding>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            status: ExecutionStatus::Init,
            current_step: 0,
            plan: Vec::new(),
            context: AgentContext::default(),
            resolver: Vec::new()
        }
    }
}

pub struct StepExecutionResult {
    pub success: bool,
    pub output: Value,
    pub observation: Option<String>,
    pub waiting: bool,
    pub replan: bool,
}
