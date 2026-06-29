use crate::StepExecutionResult;

#[derive(Debug, Clone)]
pub enum EvaluationDecision {
    Continue,
    Wait,
    Replan,
    Finish,
    Failed,
}

#[derive(Debug, Clone)]
pub struct EvaluationStep {
    pub step_id: String,
    pub decision: EvaluationDecision,
}

impl EvaluationStep {
    pub async fn evaluate(step_id: String, result: &StepExecutionResult) -> EvaluationStep {
        let decision = if result.waiting {
            EvaluationDecision::Wait
        } else if result.success {
            EvaluationDecision::Continue
        } else if result.replan {
            EvaluationDecision::Replan
        } else if !result.success {
            EvaluationDecision::Failed
        } else {
            EvaluationDecision::Finish
        };

        EvaluationStep { step_id, decision }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[tokio::test]
    async fn waiting_takes_priority_over_success() {
        let result = StepExecutionResult {
            success: true,
            output: Value::Null,
            observation: Some("approval required".to_string()),
            waiting: true,
            replan: false,
        };

        let evaluation = EvaluationStep::evaluate("step 1".to_string(), &result).await;
        assert!(matches!(evaluation.decision, EvaluationDecision::Wait));
    }

    #[tokio::test]
    async fn non_retriable_failure_does_not_finish() {
        let result = StepExecutionResult {
            success: false,
            output: Value::Null,
            observation: Some("auth missing".to_string()),
            waiting: false,
            replan: false,
        };

        let evaluation = EvaluationStep::evaluate("step 1".to_string(), &result).await;
        assert!(matches!(evaluation.decision, EvaluationDecision::Failed));
    }
}
