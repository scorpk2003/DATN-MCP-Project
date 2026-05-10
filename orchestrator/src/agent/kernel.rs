use crate::{ExecutionState, PlanStep};


pub struct AgentKernel {
    pub planner: Vec<PlanStep>,
    pub executor: ExecutionState,
}