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