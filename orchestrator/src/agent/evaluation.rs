
pub enum EvaluationDecision {
    Continue,
    Retry,
    Wait,
    Replan,
    Finish,
}

pub struct EvaluationStep {
    pub step_id: String,
    pub decision: EvaluationDecision,
}