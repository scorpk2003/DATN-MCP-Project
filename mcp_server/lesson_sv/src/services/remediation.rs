use serde_json::json;

use crate::{
    domain::{
        GradingMistake, HintItem, LessonGenerateRemediationParam, MistakeType, RemedialBlock,
        RemediationResourceRef, RemediationResult, RetryActivity, RubricItem,
    },
    error::{LessonErrorCode, LessonToolError},
};

pub fn generate_remediation(
    param: &LessonGenerateRemediationParam,
) -> Result<RemediationResult, LessonToolError> {
    if param.grading_result.passed || param.grading_result.score >= 0.8 {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "Remediation is only generated for failed or weak answers.",
            json!({
                "score": param.grading_result.score,
                "passed": param.grading_result.passed,
            }),
        ));
    }
    if param.grading_result.mistakes.is_empty() {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "gradingResult.mistakes is required for remediation.",
            json!({ "activityId": param.activity_id }),
        ));
    }
    if param.resource_refs.is_empty() {
        return Err(LessonToolError::new(
            LessonErrorCode::InsufficientResources,
            "Remediation requires at least one resource reference.",
            json!({ "lessonId": param.lesson_id, "activityId": param.activity_id }),
        ));
    }

    let max_blocks = param
        .constraints
        .as_ref()
        .and_then(|constraints| constraints.max_blocks)
        .unwrap_or(5)
        .clamp(1, 5) as usize;
    let mastery_gap = param
        .grading_result
        .mastery_gap
        .clone()
        .filter(|gaps| !gaps.is_empty())
        .unwrap_or_else(|| infer_mastery_gap(&param.grading_result.mistakes));
    let resource_refs = format_resource_refs(&param.resource_refs);

    let mut remedial_blocks = param
        .grading_result
        .mistakes
        .iter()
        .map(|mistake| block_for_mistake(mistake, &resource_refs))
        .collect::<Vec<_>>();
    remedial_blocks.truncate(max_blocks);

    Ok(RemediationResult {
        reason: format!(
            "Learner needs remediation because score {:.2} is below the passing threshold and {} mistake(s) were detected.",
            param.grading_result.score,
            param.grading_result.mistakes.len()
        ),
        mastery_gap,
        hint_ladder: build_hint_ladder(&param.grading_result.mistakes),
        retry_activity: build_retry_activity(param, &resource_refs),
        remedial_blocks,
        next_action: json!({
            "type": "retry_activity",
            "targetId": format!("retry-{}", param.activity_id),
            "reason": "Review remedial blocks, then retry a focused activity."
        }),
    })
}

fn block_for_mistake(mistake: &GradingMistake, resource_refs: &[String]) -> RemedialBlock {
    let (block_type, title, content) = match mistake.mistake_type {
        MistakeType::Conceptual => (
            "explanation",
            "Rebuild the core concept",
            "Restate the concept in simpler terms, then compare it with the learner's mistaken interpretation.",
        ),
        MistakeType::Syntax => (
            "mini_practice",
            "Fix the implementation detail",
            "Focus on the smallest runnable or checkable step, then explain the expected behavior before retrying.",
        ),
        MistakeType::Reasoning => (
            "comparison",
            "Break down the reasoning",
            "Split the answer into claim, evidence, and conclusion so each reasoning step can be checked.",
        ),
        MistakeType::MissingDetail => (
            "warning",
            "Fill in the missing details",
            "Add the omitted condition, example, or explanation needed to prove understanding.",
        ),
        MistakeType::DesignIssue => (
            "example",
            "Review the design tradeoff",
            "Identify the tradeoff, name the constraint, then justify why the chosen approach fits.",
        ),
    };

    RemedialBlock {
        block_type: block_type.to_string(),
        title: title.to_string(),
        content: format!("{content} Detected issue: {}", mistake.message),
        resource_refs: resource_refs.to_vec(),
    }
}

fn build_hint_ladder(mistakes: &[GradingMistake]) -> Vec<HintItem> {
    let first = mistakes.first();
    vec![
        HintItem {
            level: 1,
            hint: "Name the exact concept or step that the question is testing.".to_string(),
        },
        HintItem {
            level: 2,
            hint: first
                .map(|mistake| format!("Address this issue directly: {}", mistake.message))
                .unwrap_or_else(|| {
                    "Compare your answer with the resource-backed explanation.".to_string()
                }),
        },
        HintItem {
            level: 3,
            hint: "Use one concrete example and explain why it satisfies the rubric.".to_string(),
        },
    ]
}

fn build_retry_activity(
    param: &LessonGenerateRemediationParam,
    resource_refs: &[String],
) -> RetryActivity {
    RetryActivity {
        activity_id: format!("retry-{}", param.activity_id),
        activity_type: if param.submission.submission_type == "code" {
            "debug".to_string()
        } else {
            "short_answer".to_string()
        },
        prompt: format!(
            "Retry the activity. Use the referenced evidence ({}) to correct the weak parts of your answer without copying a full solution.",
            resource_refs.join(", ")
        ),
        expected_answer: "A corrected answer should name the core concept, cite the relevant evidence, and explain the reasoning step by step.".to_string(),
        rubric: vec![
            RubricItem {
                criterion: "corrected_concept".to_string(),
                max_score: 0.4,
                description: "Corrects the mistake identified in feedback.".to_string(),
            },
            RubricItem {
                criterion: "evidence_use".to_string(),
                max_score: 0.3,
                description: "Uses the provided resource references.".to_string(),
            },
            RubricItem {
                criterion: "reasoning_clarity".to_string(),
                max_score: 0.3,
                description: "Explains why the corrected answer works.".to_string(),
            },
        ],
    }
}

fn infer_mastery_gap(mistakes: &[GradingMistake]) -> Vec<String> {
    mistakes
        .iter()
        .map(|mistake| match mistake.mistake_type {
            MistakeType::Conceptual => "conceptual_understanding",
            MistakeType::Syntax => "implementation_detail",
            MistakeType::Reasoning => "reasoning_steps",
            MistakeType::MissingDetail => "answer_completeness",
            MistakeType::DesignIssue => "design_tradeoff",
        })
        .map(ToString::to_string)
        .collect()
}

fn format_resource_refs(resource_refs: &[RemediationResourceRef]) -> Vec<String> {
    resource_refs
        .iter()
        .map(|resource| match &resource.chunk_id {
            Some(chunk_id) => format!("{}:{}", resource.resource_id, chunk_id),
            None => resource.resource_id.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AuthContext, GradingMistake, LessonGenerateRemediationParam, RemediationConstraints,
        RemediationGradingResultInput, RemediationResourceRef, RemediationSubmissionInput,
    };

    fn param_for(mistake_type: MistakeType) -> LessonGenerateRemediationParam {
        LessonGenerateRemediationParam {
            request_id: Some("req-remediate".to_string()),
            auth_context: Some(AuthContext {
                user_id: "user-a".to_string(),
                verified: true,
                scope: vec!["lesson:evaluate".to_string()],
                verified_by: Some("database_mcp".to_string()),
                verified_at: Some("2026-06-26T00:00:00Z".to_string()),
            }),
            user_id: "user-a".to_string(),
            roadmap_id: "roadmap-a".to_string(),
            roadmap_node_id: "node-a".to_string(),
            lesson_id: "lesson-a".to_string(),
            session_id: "session-a".to_string(),
            activity_id: "activity-a".to_string(),
            grading_result: RemediationGradingResultInput {
                score: 0.4,
                passed: false,
                mistakes: vec![GradingMistake {
                    mistake_type,
                    message: "The answer misses the key point.".to_string(),
                    severity: "high".to_string(),
                }],
                feedback: "Needs review.".to_string(),
                mastery_gap: None,
            },
            submission: RemediationSubmissionInput {
                submission_type: "short_answer".to_string(),
                content: "I am not sure.".to_string(),
            },
            resource_refs: vec![RemediationResourceRef {
                resource_id: "res-a".to_string(),
                chunk_id: Some("chunk-a".to_string()),
                title: "Resource".to_string(),
                source_url: Some("https://example.com".to_string()),
            }],
            constraints: Some(RemediationConstraints {
                max_blocks: Some(3),
                difficulty: Some("easier".to_string()),
                include_retry_activity: Some(true),
            }),
        }
    }

    #[test]
    fn conceptual_mistake_generates_explanation_and_retry() {
        let remediation = generate_remediation(&param_for(MistakeType::Conceptual)).unwrap();
        assert_eq!(remediation.remedial_blocks[0].block_type, "explanation");
        assert_eq!(remediation.retry_activity.activity_type, "short_answer");
        assert!(!remediation.hint_ladder.is_empty());
    }

    #[test]
    fn reasoning_mistake_generates_comparison_block() {
        let remediation = generate_remediation(&param_for(MistakeType::Reasoning)).unwrap();
        assert_eq!(remediation.remedial_blocks[0].block_type, "comparison");
    }

    #[test]
    fn missing_resource_refs_returns_insufficient_resources() {
        let mut param = param_for(MistakeType::MissingDetail);
        param.resource_refs = vec![];
        let error = generate_remediation(&param).unwrap_err();
        assert_eq!(error.code, LessonErrorCode::InsufficientResources);
    }
}
