use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, RawContent},
};
use serde::Deserialize;
use serde_json::Value;

use super::LessonServer;
use crate::domain::{
    AuthContext, ExerciseScore, LessonAnalyzeNodeParam, LessonCompleteSessionParam,
    LessonCreateDraftParam, LessonGenerateRemediationParam, LessonRequirement, MistakeType,
    RemediationGradingResultInput, RemediationResourceRef, RemediationSubmissionInput,
    ResourceCandidateInput, SessionSummary,
};

#[derive(Debug, Deserialize)]
struct LessonFixtureInput {
    #[serde(rename = "analyzeNode")]
    analyze_node: LessonAnalyzeNodeParam,
    resources: Vec<ResourceCandidateInput>,
}

fn fixture_input() -> LessonFixtureInput {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/sql-joins.lesson-input.json"
    ))
    .expect("fixture input should match Lesson MCP schema")
}

fn auth(user_id: &str, scopes: &[&str]) -> AuthContext {
    AuthContext {
        user_id: user_id.to_string(),
        verified: true,
        scope: scopes.iter().map(|scope| scope.to_string()).collect(),
        verified_by: Some("database_mcp".to_string()),
        verified_at: Some("2026-01-01T00:00:00Z".to_string()),
    }
}

fn envelope(result: CallToolResult) -> Value {
    assert_ne!(result.is_error, Some(true));
    let text = result
        .content
        .first()
        .and_then(|content| match &content.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .expect("tool result should contain text JSON");

    serde_json::from_str(text).expect("tool result text should be valid JSON")
}

fn assert_tool_error(value: &Value, code: &str, request_id: &str) {
    assert_eq!(value["ok"], false);
    assert_eq!(value["success"], false);
    assert_eq!(value["error"]["code"], code);
    assert_eq!(value["request_id"], request_id);
    assert!(value["error"]["message"].is_string());
    assert!(value["error"]["suggested_action"].is_string());
}

#[tokio::test]
async fn mcp_contract_and_readiness_expose_supported_tool_surface() {
    let server = LessonServer::new();

    let info = server.get_info();
    assert!(info.capabilities.tools.is_some());

    let contract = envelope(server.get_lesson_contract().await.unwrap());
    assert_eq!(contract["ok"], true);
    let tools = contract["data"]["tools"]
        .as_array()
        .expect("contract tools should be an array");
    for expected_tool in [
        "lesson_analyze_node",
        "lesson_create_draft",
        "lesson_validate_draft",
        "lesson_finalize",
        "lesson_grade_answer",
        "lesson_generate_remediation",
        "lesson_complete_session",
    ] {
        assert!(
            tools.iter().any(|tool| tool == expected_tool),
            "missing contract tool {expected_tool}"
        );
    }

    let readiness = envelope(server.lesson_readiness().await.unwrap());
    assert_eq!(readiness["ok"], true);
    assert_eq!(readiness["data"]["remediation"]["status"], "ready");
    assert_eq!(readiness["data"]["observability"]["status"], "ready");
    assert_eq!(
        readiness["data"]["checks"]["telemetry"]["runtimeVerified"],
        true
    );
    assert_eq!(
        readiness["data"]["hardeningStatus"]["remainingPhases"]
            .as_array()
            .expect("remainingPhases should be an array")
            .len(),
        0
    );
    assert!(readiness["data"]["nextBuildStep"].is_null());
}

#[tokio::test]
async fn mcp_client_happy_path_can_analyze_and_create_draft() {
    let server = LessonServer::new();
    let fixture = fixture_input();
    let mut analyze = fixture.analyze_node;
    analyze.request_id = Some("req-analyze-ok".to_string());
    analyze.auth_context = Some(auth(&analyze.user_id, &["roadmap:read"]));

    let analyze_response = envelope(
        server
            .lesson_analyze_node(Parameters(analyze.clone()))
            .await
            .unwrap(),
    );
    assert_eq!(analyze_response["ok"], true);
    assert_eq!(analyze_response["data"]["status"], "ok");

    let requirement: LessonRequirement =
        serde_json::from_value(analyze_response["data"]["lessonRequirement"].clone())
            .expect("lessonRequirement should match public schema");
    let create = LessonCreateDraftParam {
        request_id: Some("req-create-ok".to_string()),
        auth_context: Some(auth(&analyze.user_id, &["lesson:write"])),
        user_id: analyze.user_id,
        roadmap_id: analyze.roadmap_id,
        roadmap_node_id: analyze.roadmap_node_id,
        lesson_requirement: requirement,
        resources: fixture.resources,
        constraints: None,
    };

    let create_response = envelope(
        server
            .lesson_create_draft(Parameters(create))
            .await
            .unwrap(),
    );
    assert_eq!(create_response["ok"], true);
    assert_eq!(create_response["data"]["status"], "ready");
    assert!(
        create_response["data"]["lessonDraft"]["contentBlocks"]
            .as_array()
            .expect("contentBlocks should be an array")
            .len()
            >= 2
    );
    assert_eq!(
        create_response["data"]["implementationStatus"],
        "lesson_generator_v1"
    );
}

#[tokio::test]
async fn mcp_client_receives_permission_denied_envelope_without_verified_auth() {
    let server = LessonServer::new();
    let mut analyze = fixture_input().analyze_node;
    analyze.request_id = Some("req-permission-denied".to_string());
    analyze.auth_context = None;

    let response = envelope(
        server
            .lesson_analyze_node(Parameters(analyze))
            .await
            .unwrap(),
    );
    assert_tool_error(&response, "PERMISSION_DENIED", "req-permission-denied");

    let health = envelope(server.lesson_health().await.unwrap());
    assert_eq!(health["data"]["observability"]["totalToolErrors"], 1);
    assert_eq!(
        health["data"]["observability"]["errorCodes"]["PERMISSION_DENIED"],
        1
    );
}

#[tokio::test]
async fn mcp_client_receives_insufficient_resources_for_empty_evidence() {
    let server = LessonServer::new();
    let fixture = fixture_input();
    let mut analyze = fixture.analyze_node;
    analyze.auth_context = Some(auth(&analyze.user_id, &["roadmap:read"]));

    let analyze_response = envelope(
        server
            .lesson_analyze_node(Parameters(analyze.clone()))
            .await
            .unwrap(),
    );
    let requirement: LessonRequirement =
        serde_json::from_value(analyze_response["data"]["lessonRequirement"].clone())
            .expect("lessonRequirement should match public schema");
    let create = LessonCreateDraftParam {
        request_id: Some("req-insufficient-resources".to_string()),
        auth_context: Some(auth(&analyze.user_id, &["lesson:write"])),
        user_id: analyze.user_id,
        roadmap_id: analyze.roadmap_id,
        roadmap_node_id: analyze.roadmap_node_id,
        lesson_requirement: requirement,
        resources: vec![],
        constraints: None,
    };

    let response = envelope(
        server
            .lesson_create_draft(Parameters(create))
            .await
            .unwrap(),
    );
    assert_tool_error(
        &response,
        "INSUFFICIENT_RESOURCES",
        "req-insufficient-resources",
    );
}

#[tokio::test]
async fn mcp_client_receives_invalid_input_for_bad_progress_score() {
    let server = LessonServer::new();
    let param = LessonCompleteSessionParam {
        request_id: Some("req-invalid-score".to_string()),
        auth_context: Some(auth("user-1", &["lesson:progress"])),
        user_id: "user-1".to_string(),
        lesson_id: "lesson-1".to_string(),
        session_id: "session-1".to_string(),
        session_summary: SessionSummary {
            completed_blocks: vec!["block-1".to_string()],
            exercise_scores: vec![ExerciseScore {
                exercise_id: "exercise-1".to_string(),
                score: 1.2,
            }],
            quiz_score: Some(0.8),
            checkpoint_score: Some(0.7),
            time_spent_minutes: Some(20),
        },
    };

    let response = envelope(
        server
            .lesson_complete_session(Parameters(param))
            .await
            .unwrap(),
    );
    assert_tool_error(&response, "INVALID_INPUT", "req-invalid-score");
}

#[tokio::test]
async fn mcp_client_can_generate_grounded_remediation() {
    let server = LessonServer::new();
    let param = LessonGenerateRemediationParam {
        request_id: Some("req-remediation-ok".to_string()),
        auth_context: Some(auth("user-1", &["lesson:evaluate"])),
        user_id: "user-1".to_string(),
        roadmap_id: "roadmap-1".to_string(),
        roadmap_node_id: "node-1".to_string(),
        lesson_id: "lesson-1".to_string(),
        session_id: "session-1".to_string(),
        activity_id: "exercise-main".to_string(),
        grading_result: RemediationGradingResultInput {
            score: 0.35,
            passed: false,
            mistakes: vec![crate::domain::GradingMistake {
                mistake_type: MistakeType::Conceptual,
                message: "Does not explain why the join matches rows.".to_string(),
                severity: "medium".to_string(),
            }],
            feedback: "The answer needs a clearer concept explanation.".to_string(),
            mastery_gap: Some(vec!["SQL join matching semantics".to_string()]),
        },
        submission: RemediationSubmissionInput {
            submission_type: "short_answer".to_string(),
            content: "INNER JOIN joins tables.".to_string(),
        },
        resource_refs: vec![RemediationResourceRef {
            resource_id: "resource-sql-joins-docs".to_string(),
            chunk_id: Some("chunk-inner-join".to_string()),
            title: "SQL joins documentation".to_string(),
            source_url: Some("https://example.com/sql-joins".to_string()),
        }],
        constraints: None,
    };

    let response = envelope(
        server
            .lesson_generate_remediation(Parameters(param))
            .await
            .unwrap(),
    );
    assert_eq!(response["ok"], true);
    assert_eq!(response["data"]["status"], "ok");
    assert_eq!(
        response["data"]["remediation"]["nextAction"]["type"],
        "retry_activity"
    );
    assert!(
        response["data"]["remediation"]["remedialBlocks"]
            .as_array()
            .expect("remedialBlocks should be an array")
            .iter()
            .all(|block| !block["resourceRefs"].as_array().unwrap().is_empty())
    );
}

#[tokio::test]
async fn mcp_client_observability_counters_track_successes_and_errors() {
    let server = LessonServer::new();

    let health = envelope(server.lesson_health().await.unwrap());
    assert_eq!(health["data"]["observability"]["totalToolCalls"], 1);
    assert_eq!(health["data"]["observability"]["totalToolSuccesses"], 1);

    let mut analyze = fixture_input().analyze_node;
    analyze.request_id = Some("req-observe-error".to_string());
    analyze.auth_context = None;
    let error_response = envelope(
        server
            .lesson_analyze_node(Parameters(analyze))
            .await
            .unwrap(),
    );
    assert_tool_error(&error_response, "PERMISSION_DENIED", "req-observe-error");

    let readiness = envelope(server.lesson_readiness().await.unwrap());
    let counters = &readiness["data"]["observability"]["counters"];
    assert_eq!(counters["totalToolCalls"], 3);
    assert_eq!(counters["totalToolSuccesses"], 2);
    assert_eq!(counters["totalToolErrors"], 1);
    assert_eq!(counters["toolCalls"]["lesson_health"], 1);
    assert_eq!(counters["toolCalls"]["lesson_analyze_node"], 1);
    assert_eq!(counters["errorCodes"]["PERMISSION_DENIED"], 1);
}
