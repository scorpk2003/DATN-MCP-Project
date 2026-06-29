use crate::domain::{
    ExerciseScore, LessonAnalyzeNodeParam, LessonGenerateRemediationParam, LessonLevel,
    RemediationConstraints, RemediationGradingResultInput, RemediationResourceRef,
    RemediationSubmissionInput, ResourceCandidateInput, ResourceChunkInput, RoadmapNodeInput,
    SessionSummary, SourceType, UserContextInput,
};
use crate::services::{
    finalizer, grading, lesson_generator, lesson_validator, node_analyzer, progress_policy,
    remediation, resource_packer,
};

#[test]
fn happy_path_generates_validates_finalizes_grades_and_completes() {
    let analyze_param = LessonAnalyzeNodeParam {
        request_id: None,
        auth_context: None,
        user_id: "user-1".to_string(),
        roadmap_id: "roadmap-1".to_string(),
        roadmap_node_id: "node-1".to_string(),
        node: RoadmapNodeInput {
            title: "SQL joins".to_string(),
            topic: "SQL joins".to_string(),
            description: "Combine rows from multiple tables".to_string(),
            level: LessonLevel::Beginner,
            prerequisites: vec!["SQL basics".to_string()],
            expected_outcomes: vec!["Write inner join queries".to_string()],
        },
        user_context: Some(UserContextInput {
            skill_level: Some("beginner".to_string()),
            known_topics: Some(vec!["SQL basics".to_string()]),
            weak_topics: Some(vec![]),
            learning_goal: Some("Backend development".to_string()),
        }),
    };

    let requirement = node_analyzer::analyze_node(&analyze_param);
    assert!(requirement.prerequisite_gaps.is_empty());

    let resources = vec![ResourceCandidateInput {
        id: "res-sql-joins".to_string(),
        title: "SQL joins documentation".to_string(),
        url: Some("https://example.com/sql-joins".to_string()),
        source_type: SourceType::Docs,
        summary: Some("INNER JOIN combines matching rows from two tables.".to_string()),
        chunks: Some(vec![ResourceChunkInput {
            chunk_id: "chunk-1".to_string(),
            text: "Use INNER JOIN to write queries that combine matching rows across tables."
                .to_string(),
            score: Some(0.95),
        }]),
        quality_score: Some(0.9),
        relevance_score: Some(0.9),
    }];

    let evidence = resource_packer::pack_resources(&requirement, &resources);
    assert!(resource_packer::has_sufficient_evidence(&evidence));

    let draft = lesson_generator::generate_draft(&requirement, &evidence);
    let validation = lesson_validator::validate_draft(&draft, None);
    assert!(validation.passed);

    let payload = finalizer::build_lesson_payload(draft.clone(), "ready".to_string());
    assert_eq!(payload["notPersisted"], false);

    let grading = grading::grade_answer(
        "This answer explains concept accuracy with evidence and practical application because INNER JOIN combines matching rows from both tables.",
        &draft.exercises[0].rubric,
    );
    assert!(grading.score >= 0.5);

    let completion = progress_policy::complete_session(crate::domain::LessonCompleteSessionParam {
        request_id: None,
        auth_context: None,
        user_id: "user-1".to_string(),
        lesson_id: "lesson-1".to_string(),
        session_id: "session-1".to_string(),
        session_summary: SessionSummary {
            completed_blocks: draft
                .content_blocks
                .iter()
                .map(|block| block.id.clone())
                .collect(),
            exercise_scores: vec![ExerciseScore {
                exercise_id: "exercise-main".to_string(),
                score: 0.85,
            }],
            quiz_score: Some(0.8),
            checkpoint_score: Some(0.9),
            time_spent_minutes: Some(35),
        },
    });

    assert_eq!(completion.status, "completed");
}

#[test]
fn weak_answer_can_generate_grounded_remediation() {
    let rubric = vec![crate::domain::RubricItem {
        criterion: "concept accuracy".to_string(),
        max_score: 1.0,
        description: "Uses evidence and explains reasoning".to_string(),
    }];
    let grade = grading::grade_answer("not sure", &rubric);
    assert!(!grade.passed);

    let remediation = remediation::generate_remediation(&LessonGenerateRemediationParam {
        request_id: Some("req-remediation-flow".to_string()),
        auth_context: None,
        user_id: "user-1".to_string(),
        roadmap_id: "roadmap-1".to_string(),
        roadmap_node_id: "node-1".to_string(),
        lesson_id: "lesson-1".to_string(),
        session_id: "session-1".to_string(),
        activity_id: "exercise-main".to_string(),
        grading_result: RemediationGradingResultInput {
            score: grade.score,
            passed: grade.passed,
            mistakes: grade.mistakes,
            feedback: grade.feedback,
            mastery_gap: None,
        },
        submission: RemediationSubmissionInput {
            submission_type: "short_answer".to_string(),
            content: "not sure".to_string(),
        },
        resource_refs: vec![RemediationResourceRef {
            resource_id: "res-sql-joins".to_string(),
            chunk_id: Some("chunk-1".to_string()),
            title: "SQL joins documentation".to_string(),
            source_url: Some("https://example.com/sql-joins".to_string()),
        }],
        constraints: Some(RemediationConstraints {
            max_blocks: Some(3),
            difficulty: Some("easier".to_string()),
            include_retry_activity: Some(true),
        }),
    })
    .expect("weak answer should generate remediation");

    assert!(!remediation.remedial_blocks.is_empty());
    assert_eq!(
        remediation.retry_activity.activity_id,
        "retry-exercise-main"
    );
    assert_eq!(remediation.next_action["type"], "retry_activity");
}
