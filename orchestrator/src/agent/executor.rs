use crate::{AgentContext, PlanStep};


pub enum ExecutionStatus {
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
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            status: ExecutionStatus::Planning,
            current_step: 0,
            plan: Vec::new(),
            context: AgentContext::default(),
        }
    }
}