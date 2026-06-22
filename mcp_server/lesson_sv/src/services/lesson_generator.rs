use crate::domain::{
    AssessmentRubric, Exercise, ExerciseDifficulty, ExerciseType, LessonBlockType,
    LessonContentBlock, LessonDraft, LessonLevel, LessonRequirement, PackedLessonEvidence,
    QuizQuestion, RubricItem,
};

pub fn generate_draft(
    requirement: &LessonRequirement,
    evidence: &PackedLessonEvidence,
) -> LessonDraft {
    let resources = evidence
        .primary_sources
        .iter()
        .chain(evidence.supporting_sources.iter())
        .chain(evidence.code_sources.iter())
        .cloned()
        .collect::<Vec<_>>();
    let source_refs = resources
        .iter()
        .map(|resource| resource.id.clone())
        .collect::<Vec<_>>();
    let chunk_summary = evidence
        .selected_chunks
        .iter()
        .take(3)
        .map(|chunk| trim_text(&chunk.text, 240))
        .collect::<Vec<_>>();

    let content_blocks = vec![
        LessonContentBlock {
            id: "block-why-this-matters".to_string(),
            block_type: LessonBlockType::Concept,
            title: format!("Why {} matters", requirement.topic),
            content: format!(
                "{} is part of this roadmap because it supports these outcomes: {}.",
                requirement.topic,
                requirement.objectives.join("; ")
            ),
            source_refs: Some(source_refs.clone()),
            order: 1,
        },
        LessonContentBlock {
            id: "block-core-concept".to_string(),
            block_type: LessonBlockType::Explanation,
            title: "Core concept".to_string(),
            content: format!(
                "Focus on the core idea of {} at {:?} level. Evidence highlights: {}",
                requirement.topic,
                requirement.level,
                chunk_summary.join(" ")
            ),
            source_refs: Some(source_refs.clone()),
            order: 2,
        },
        LessonContentBlock {
            id: "block-worked-example".to_string(),
            block_type: if has_code_signal(requirement) {
                LessonBlockType::CodeExample
            } else {
                LessonBlockType::Example
            },
            title: "Worked example".to_string(),
            content: format!(
                "Use one selected source to walk through a concrete example of {}. Start from the problem, identify the relevant concept, then apply it step by step.",
                requirement.topic
            ),
            source_refs: Some(source_refs.clone()),
            order: 3,
        },
        LessonContentBlock {
            id: "block-common-mistakes".to_string(),
            block_type: LessonBlockType::CommonMistake,
            title: "Common mistakes".to_string(),
            content: format!(
                "Common mistakes for {} usually come from skipping prerequisites, using the concept outside its constraints, or memorizing syntax without explaining why it works.",
                requirement.topic
            ),
            source_refs: Some(source_refs.clone()),
            order: 4,
        },
        LessonContentBlock {
            id: "block-checkpoint".to_string(),
            block_type: LessonBlockType::Checkpoint,
            title: "Checkpoint".to_string(),
            content: format!(
                "In your own words, explain how {} helps achieve one lesson objective.",
                requirement.topic
            ),
            source_refs: Some(source_refs.clone()),
            order: 5,
        },
        LessonContentBlock {
            id: "block-summary".to_string(),
            block_type: LessonBlockType::Summary,
            title: "Summary".to_string(),
            content: format!(
                "You should now be able to connect {} to these objectives: {}.",
                requirement.topic,
                requirement.objectives.join("; ")
            ),
            source_refs: Some(source_refs.clone()),
            order: 6,
        },
    ];

    let exercises = vec![Exercise {
        id: "exercise-main".to_string(),
        exercise_type: requirement
            .recommended_exercise_types
            .first()
            .cloned()
            .unwrap_or(ExerciseType::ShortAnswer),
        title: format!("Practice {}", requirement.topic),
        prompt: build_exercise_prompt(requirement),
        expected_output: None,
        hints: vec![
            "Start by naming the concept being applied.".to_string(),
            "Tie your answer back to at least one objective.".to_string(),
            "Use the selected resources as evidence instead of guessing.".to_string(),
        ],
        rubric: default_rubric(),
        difficulty: match requirement.level {
            LessonLevel::Beginner => ExerciseDifficulty::Easy,
            LessonLevel::Intermediate => ExerciseDifficulty::Medium,
            LessonLevel::Advanced => ExerciseDifficulty::Hard,
        },
        source_refs: Some(source_refs.clone()),
    }];

    let quizzes = vec![QuizQuestion {
        id: "quiz-core-concept".to_string(),
        question: format!(
            "What is the best next step when you cannot explain {} using the selected evidence?",
            requirement.topic
        ),
        choices: vec![
            "Continue without checking sources.".to_string(),
            "Review the relevant source chunks and identify the missing concept.".to_string(),
            "Skip the lesson objective.".to_string(),
            "Memorize a definition only.".to_string(),
        ],
        correct_choice_index: 1,
        explanation: "Evidence-backed review is required before continuing.".to_string(),
        source_refs: Some(source_refs),
    }];

    LessonDraft {
        title: format!("{} lesson", requirement.topic),
        topic: requirement.topic.clone(),
        level: requirement.level.clone(),
        objectives: requirement.objectives.clone(),
        prerequisites: requirement.prerequisite_gaps.clone(),
        estimated_minutes: requirement.estimated_minutes,
        content_blocks,
        resources,
        exercises,
        quizzes,
        assessment_rubric: AssessmentRubric {
            items: default_rubric(),
            passing_score: 0.8,
        },
    }
}

fn build_exercise_prompt(requirement: &LessonRequirement) -> String {
    if has_code_signal(requirement) {
        format!(
            "Create a small solution that demonstrates {}. Then explain which objective it satisfies and why.",
            requirement.topic
        )
    } else {
        format!(
            "Explain {} with a concrete example, then map your explanation to one lesson objective.",
            requirement.topic
        )
    }
}

fn has_code_signal(requirement: &LessonRequirement) -> bool {
    requirement
        .recommended_exercise_types
        .iter()
        .any(|exercise_type| {
            matches!(
                exercise_type,
                ExerciseType::Coding | ExerciseType::Debugging
            )
        })
}

fn default_rubric() -> Vec<RubricItem> {
    vec![
        RubricItem {
            criterion: "concept_accuracy".to_string(),
            max_score: 0.4,
            description: "Uses the core concept correctly.".to_string(),
        },
        RubricItem {
            criterion: "evidence_alignment".to_string(),
            max_score: 0.3,
            description: "Grounds the answer in provided lesson evidence.".to_string(),
        },
        RubricItem {
            criterion: "practical_application".to_string(),
            max_score: 0.3,
            description: "Applies the concept to a concrete example or task.".to_string(),
        },
    ]
}

fn trim_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    text.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EvidenceCoverage, LessonResource, PackedSelectedChunk, SourceType};

    #[test]
    fn generates_non_empty_draft_from_evidence() {
        let requirement = LessonRequirement {
            topic: "SQL joins".to_string(),
            level: LessonLevel::Beginner,
            objectives: vec!["Write inner join queries".to_string()],
            prerequisite_gaps: vec![],
            resource_queries: vec![],
            recommended_exercise_types: vec![ExerciseType::Coding],
            estimated_minutes: 35,
        };
        let evidence = PackedLessonEvidence {
            primary_sources: vec![LessonResource {
                id: "res".to_string(),
                title: "SQL joins docs".to_string(),
                url: None,
                source_type: SourceType::Docs,
                difficulty: LessonLevel::Beginner,
                relevance_score: 0.9,
                quality_score: Some(0.9),
                reason_selected: "test".to_string(),
            }],
            supporting_sources: vec![],
            code_sources: vec![],
            selected_chunks: vec![PackedSelectedChunk {
                resource_id: "res".to_string(),
                chunk_id: "chunk".to_string(),
                text: "INNER JOIN returns rows matching both tables.".to_string(),
                relevance_score: 0.9,
            }],
            coverage: EvidenceCoverage {
                objectives_covered: vec!["Write inner join queries".to_string()],
                objectives_missing: vec![],
                coverage_score: 1.0,
            },
        };

        let draft = generate_draft(&requirement, &evidence);
        assert!(draft.content_blocks.len() >= 4);
        assert_eq!(draft.exercises.len(), 1);
        assert_eq!(draft.quizzes.len(), 1);
    }
}
