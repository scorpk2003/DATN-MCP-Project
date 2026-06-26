use serde::Deserialize;

use crate::domain::{
    ExerciseScore, LessonAnalyzeNodeParam, LessonCompleteSessionParam, ResourceCandidateInput,
    SessionSummary,
};
use crate::services::{
    finalizer, grading, lesson_generator, lesson_validator, node_analyzer, progress_policy,
    resource_packer,
};

#[derive(Debug, Deserialize)]
struct LessonFixtureInput {
    #[serde(rename = "analyzeNode")]
    analyze_node: LessonAnalyzeNodeParam,
    resources: Vec<ResourceCandidateInput>,
}

#[derive(Debug, Deserialize)]
struct LessonFixtureOutput {
    status: String,
    #[serde(rename = "minimumExpectations")]
    minimum_expectations: MinimumExpectations,
}

#[derive(Debug, Deserialize)]
struct MinimumExpectations {
    topic: String,
    level: String,
    #[serde(rename = "minContentBlocks")]
    min_content_blocks: usize,
    #[serde(rename = "minResources")]
    min_resources: usize,
    #[serde(rename = "minExercises")]
    min_exercises: usize,
    #[serde(rename = "minQuizzes")]
    min_quizzes: usize,
    #[serde(rename = "validationStatus")]
    validation_status: String,
    #[serde(rename = "completionStatus")]
    completion_status: String,
}

#[test]
fn sql_joins_fixture_runs_through_lesson_pipeline() {
    let input: LessonFixtureInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/sql-joins.lesson-input.json"
    ))
    .expect("fixture input should match Lesson MCP schema");
    let output: LessonFixtureOutput = serde_json::from_str(include_str!(
        "../../tests/fixtures/sql-joins.lesson-output.json"
    ))
    .expect("fixture output should be valid JSON");

    let requirement = node_analyzer::analyze_node(&input.analyze_node);
    assert_eq!(requirement.topic, output.minimum_expectations.topic);
    assert!(
        format!("{:?}", requirement.level).eq_ignore_ascii_case(&output.minimum_expectations.level)
    );

    let evidence = resource_packer::pack_resources(&requirement, &input.resources);
    assert!(resource_packer::has_sufficient_evidence(&evidence));

    let draft = lesson_generator::generate_draft(&requirement, &evidence);
    assert_eq!(output.status, "ready");
    assert!(draft.content_blocks.len() >= output.minimum_expectations.min_content_blocks);
    assert!(draft.resources.len() >= output.minimum_expectations.min_resources);
    assert!(draft.exercises.len() >= output.minimum_expectations.min_exercises);
    assert!(draft.quizzes.len() >= output.minimum_expectations.min_quizzes);

    let validation = lesson_validator::validate_draft(&draft, None);
    assert_eq!(
        if validation.passed {
            "passed"
        } else {
            "failed"
        },
        output.minimum_expectations.validation_status
    );

    let payload = finalizer::build_lesson_payload(draft.clone(), "ready".to_string());
    assert_eq!(payload["notPersisted"], true);

    let grade = grading::grade_answer(
        "This answer explains concept accuracy with evidence and practical application because INNER JOIN combines matching rows and LEFT JOIN keeps unmatched left rows.",
        &draft.exercises[0].rubric,
    );
    assert!(grade.score >= 0.5);

    let completion = progress_policy::complete_session(LessonCompleteSessionParam {
        request_id: None,
        auth_context: None,
        user_id: input.analyze_node.user_id,
        lesson_id: "lesson-sql-joins".to_string(),
        session_id: "session-sql-joins".to_string(),
        session_summary: SessionSummary {
            completed_blocks: draft
                .content_blocks
                .iter()
                .map(|block| block.id.clone())
                .collect(),
            exercise_scores: vec![ExerciseScore {
                exercise_id: draft.exercises[0].id.clone(),
                score: 0.86,
            }],
            quiz_score: Some(0.85),
            checkpoint_score: Some(0.9),
            time_spent_minutes: Some(draft.estimated_minutes),
        },
    });
    assert_eq!(
        completion.status,
        output.minimum_expectations.completion_status
    );
}
