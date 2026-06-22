use crate::domain::{LessonDraft, ValidationCheck, ValidationPolicy};

#[derive(Debug, Clone)]
pub struct LessonValidationResult {
    pub passed: bool,
    pub quality_score: f32,
    pub checks: Vec<ValidationCheck>,
    pub fix_suggestions: Vec<String>,
}

pub fn validate_draft(
    draft: &LessonDraft,
    policy: Option<ValidationPolicy>,
) -> LessonValidationResult {
    let policy = policy.unwrap_or(ValidationPolicy {
        require_objectives: Some(true),
        require_resources: Some(true),
        require_exercises: Some(true),
        require_quiz: Some(false),
        min_resource_quality_score: Some(0.65),
        min_content_blocks: Some(4),
    });

    let mut checks = vec![
        ValidationCheck {
            name: "objectives_present".to_string(),
            passed: !policy.require_objectives.unwrap_or(true) || !draft.objectives.is_empty(),
            message: "Lesson has at least one objective.".to_string(),
        },
        ValidationCheck {
            name: "resources_present".to_string(),
            passed: !policy.require_resources.unwrap_or(true) || !draft.resources.is_empty(),
            message: "Lesson has at least one selected resource.".to_string(),
        },
        ValidationCheck {
            name: "resource_quality".to_string(),
            passed: !draft.resources.is_empty()
                && draft.resources.iter().all(|resource| {
                    resource.quality_score.unwrap_or(0.0)
                        >= policy.min_resource_quality_score.unwrap_or(0.65)
                }),
            message: "Selected resources meet minimum quality threshold.".to_string(),
        },
        ValidationCheck {
            name: "exercises_present".to_string(),
            passed: !policy.require_exercises.unwrap_or(true) || !draft.exercises.is_empty(),
            message: "Lesson has at least one exercise.".to_string(),
        },
        ValidationCheck {
            name: "quiz_present".to_string(),
            passed: !policy.require_quiz.unwrap_or(false) || !draft.quizzes.is_empty(),
            message: "Lesson has a quiz when policy requires it.".to_string(),
        },
        ValidationCheck {
            name: "content_block_count".to_string(),
            passed: draft.content_blocks.len() as u32 >= policy.min_content_blocks.unwrap_or(4),
            message: "Lesson has enough content blocks for a guided lesson.".to_string(),
        },
    ];

    checks.push(ValidationCheck {
        name: "content_blocks_not_empty".to_string(),
        passed: !draft.content_blocks.is_empty()
            && draft
                .content_blocks
                .iter()
                .all(|block| !block.title.trim().is_empty() && !block.content.trim().is_empty()),
        message: "Every content block has title and content.".to_string(),
    });
    checks.push(ValidationCheck {
        name: "source_refs_present".to_string(),
        passed: !draft.content_blocks.is_empty()
            && draft.content_blocks.iter().all(|block| {
                block
                    .source_refs
                    .as_ref()
                    .map(|refs| !refs.is_empty())
                    .unwrap_or(false)
            }),
        message: "Every content block has source references.".to_string(),
    });
    checks.push(ValidationCheck {
        name: "exercise_rubric_present".to_string(),
        passed: !draft.exercises.is_empty()
            && draft
                .exercises
                .iter()
                .all(|exercise| !exercise.rubric.is_empty()),
        message: "Every exercise has a rubric.".to_string(),
    });

    let passed = checks.iter().all(|check| check.passed);
    let quality_score =
        checks.iter().filter(|check| check.passed).count() as f32 / checks.len().max(1) as f32;
    let fix_suggestions = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| format!("Fix failed check: {}", check.name))
        .collect::<Vec<_>>();

    LessonValidationResult {
        passed,
        quality_score,
        checks,
        fix_suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AssessmentRubric, LessonLevel};

    #[test]
    fn empty_draft_fails_validation() {
        let draft = LessonDraft {
            title: "Empty".to_string(),
            topic: "Empty".to_string(),
            level: LessonLevel::Beginner,
            objectives: vec![],
            prerequisites: vec![],
            estimated_minutes: 30,
            content_blocks: vec![],
            resources: vec![],
            exercises: vec![],
            quizzes: vec![],
            assessment_rubric: AssessmentRubric {
                items: vec![],
                passing_score: 0.8,
            },
        };

        let result = validate_draft(&draft, None);
        assert!(!result.passed);
        assert!(result.quality_score < 0.5);
    }
}
