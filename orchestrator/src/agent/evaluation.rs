use crate::StepExecutionResult;

pub enum EvaluationDecision {
    Continue,
    Wait,
    Replan,
    Finish,
}

pub struct EvaluationStep {
    pub step_id: String,
    pub decision: EvaluationDecision,
}

impl EvaluationStep {
    pub async fn evaluate(step_id: String, result: &StepExecutionResult) -> EvaluationStep {
        let decision = if result.success {
            EvaluationDecision::Continue
        } else if result.waiting {
            EvaluationDecision::Wait
        } else if result.replan {
            EvaluationDecision::Replan
        } else {
            EvaluationDecision::Finish
        };

        EvaluationStep { step_id, decision }
    }
}
