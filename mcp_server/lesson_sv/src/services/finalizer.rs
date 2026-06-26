use serde_json::{Value, json};

use crate::domain::{DatabaseMcpToolCall, LessonDraft};

const CONTRACT_STATUS: &str = "missing_database_tools";

pub fn build_lesson_payload(draft: LessonDraft, status: String) -> Value {
    let idempotency_key = build_idempotency_key(&draft, &status);
    let calls = build_database_calls(&draft, &status, &idempotency_key);
    let steps = build_database_steps(&calls);
    let resource_links = calls
        .iter()
        .filter(|call| call.tool_name == "link_lesson_resource")
        .cloned()
        .collect::<Vec<_>>();
    let exercise_payloads = calls
        .iter()
        .filter(|call| call.tool_name == "create_lesson_exercise")
        .cloned()
        .collect::<Vec<_>>();
    let quiz_payloads = calls
        .iter()
        .filter(|call| call.tool_name == "create_lesson_quiz")
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "schemaVersion": "lesson_draft_v1",
        "notPersisted": true,
        "idempotencyKey": idempotency_key,
        "lesson": draft,
        "resourceLinks": resource_links,
        "exercisePayloads": exercise_payloads,
        "quizPayloads": quiz_payloads,
        "orchestratorPersistencePlan": {
            "databaseMcpCalls": calls,
            "databaseCallPlan": {
                "contractStatus": CONTRACT_STATUS,
                "transactionRequired": true,
                "idempotencyKey": idempotency_key,
                "steps": steps,
                "rollbackPolicy": {
                    "onFailure": "rollback_all",
                    "reason": "Lesson payload creates parent and child records that must remain consistent."
                },
                "executionPolicy": "Execute in order. Treat create_lesson as the parent record and resolve ${lesson.lessonId} for child rows.",
                "contractWarning": "Database MCP currently does not expose lesson-specific persistence tools; see lesson_sv/docs/database_mcp_contract_mapping.md."
            }
        }
    })
}

fn build_database_steps(calls: &[DatabaseMcpToolCall]) -> Vec<Value> {
    calls
        .iter()
        .enumerate()
        .map(|(index, call)| {
            json!({
                "stepId": call.result_alias.clone().unwrap_or_else(|| format!("step_{}", index + 1)),
                "tool": format!("database_mcp.{}", call.tool_name),
                "args": call.arguments,
                "dependsOn": call.depends_on,
                "expectedOutput": expected_output_for_tool(&call.tool_name),
                "errorCases": error_cases_for_tool(&call.tool_name),
            })
        })
        .collect()
}

fn expected_output_for_tool(tool_name: &str) -> Value {
    match tool_name {
        "create_lesson" => json!({"ok": true, "lessonId": "string"}),
        "create_lesson_block" => json!({"ok": true, "blockId": "string"}),
        "link_lesson_resource" => json!({"ok": true, "lessonResourceId": "string"}),
        "create_lesson_exercise" => json!({"ok": true, "exerciseId": "string"}),
        "create_lesson_quiz" => json!({"ok": true, "quizId": "string"}),
        _ => json!({"ok": true}),
    }
}

fn error_cases_for_tool(tool_name: &str) -> Vec<&'static str> {
    match tool_name {
        "create_lesson" => vec!["DUPLICATE_IDEMPOTENCY_KEY", "ROADMAP_NODE_NOT_FOUND"],
        "create_lesson_block" => vec!["LESSON_NOT_FOUND", "INVALID_BLOCK_PAYLOAD"],
        "link_lesson_resource" => vec!["LESSON_NOT_FOUND", "RESOURCE_NOT_FOUND"],
        "create_lesson_exercise" => vec!["LESSON_NOT_FOUND", "INVALID_EXERCISE_PAYLOAD"],
        "create_lesson_quiz" => vec!["LESSON_NOT_FOUND", "INVALID_QUIZ_PAYLOAD"],
        _ => vec!["DATABASE_ERROR"],
    }
}

fn build_database_calls(
    draft: &LessonDraft,
    status: &str,
    idempotency_key: &str,
) -> Vec<DatabaseMcpToolCall> {
    let mut calls = vec![DatabaseMcpToolCall {
        tool_name: "create_lesson".to_string(),
        arguments: json!({
            "idempotencyKey": idempotency_key,
            "title": &draft.title,
            "topic": &draft.topic,
            "level": &draft.level,
            "objectives": &draft.objectives,
            "prerequisites": &draft.prerequisites,
            "estimatedMinutes": draft.estimated_minutes,
            "status": status,
            "assessmentRubric": &draft.assessment_rubric,
        }),
        result_alias: Some("lesson".to_string()),
        depends_on: vec![],
    }];

    for block in &draft.content_blocks {
        calls.push(DatabaseMcpToolCall {
            tool_name: "create_lesson_block".to_string(),
            arguments: json!({
                "lessonId": "${lesson.lessonId}",
                "block": block,
            }),
            result_alias: Some(format!("block:{}", block.id)),
            depends_on: vec!["lesson".to_string()],
        });
    }

    for resource in &draft.resources {
        calls.push(DatabaseMcpToolCall {
            tool_name: "link_lesson_resource".to_string(),
            arguments: json!({
                "lessonId": "${lesson.lessonId}",
                "resource": resource,
            }),
            result_alias: Some(format!("resource:{}", resource.id)),
            depends_on: vec!["lesson".to_string()],
        });
    }

    for exercise in &draft.exercises {
        calls.push(DatabaseMcpToolCall {
            tool_name: "create_lesson_exercise".to_string(),
            arguments: json!({
                "lessonId": "${lesson.lessonId}",
                "exercise": exercise,
            }),
            result_alias: Some(format!("exercise:{}", exercise.id)),
            depends_on: vec!["lesson".to_string()],
        });
    }

    for quiz in &draft.quizzes {
        calls.push(DatabaseMcpToolCall {
            tool_name: "create_lesson_quiz".to_string(),
            arguments: json!({
                "lessonId": "${lesson.lessonId}",
                "quiz": quiz,
            }),
            result_alias: Some(format!("quiz:{}", quiz.id)),
            depends_on: vec!["lesson".to_string()],
        });
    }

    calls
}

fn build_idempotency_key(draft: &LessonDraft, status: &str) -> String {
    format!(
        "lesson:{}:{}:{}:{}",
        slug(&draft.topic),
        format!("{:?}", draft.level).to_lowercase(),
        draft.content_blocks.len(),
        status
    )
}

fn slug(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AssessmentRubric, LessonLevel};
    use serde::Deserialize;

    #[test]
    fn splits_draft_into_ordered_database_calls() {
        let draft = LessonDraft {
            title: "SQL joins lesson".to_string(),
            topic: "SQL joins".to_string(),
            level: LessonLevel::Beginner,
            objectives: vec!["Write joins".to_string()],
            prerequisites: vec![],
            estimated_minutes: 35,
            content_blocks: vec![],
            resources: vec![],
            exercises: vec![],
            quizzes: vec![],
            assessment_rubric: AssessmentRubric {
                items: vec![],
                passing_score: 0.8,
            },
        };

        let payload = build_lesson_payload(draft, "ready".to_string());
        assert_eq!(payload["notPersisted"], true);
        assert_eq!(
            payload["orchestratorPersistencePlan"]["databaseMcpCalls"][0]["toolName"],
            "create_lesson"
        );
        assert_eq!(
            payload["orchestratorPersistencePlan"]["databaseCallPlan"]["transactionRequired"],
            true
        );
        assert_eq!(
            payload["orchestratorPersistencePlan"]["databaseCallPlan"]["contractStatus"],
            "missing_database_tools"
        );
    }

    #[derive(Debug, Deserialize)]
    struct DatabaseContractFixture {
        #[serde(rename = "contractStatus")]
        contract_status: String,
        #[serde(rename = "currentDatabaseMcpTools")]
        current_database_mcp_tools: Vec<String>,
        #[serde(rename = "requiredLessonTools")]
        required_lesson_tools: Vec<String>,
    }

    #[test]
    fn finalizer_contract_fixture_marks_missing_database_tools() {
        let fixture: DatabaseContractFixture = serde_json::from_str(include_str!(
            "../../tests/fixtures/database_mcp_contract.json"
        ))
        .expect("database contract fixture should parse");

        assert_eq!(fixture.contract_status, CONTRACT_STATUS);
        for required_tool in &fixture.required_lesson_tools {
            assert!(
                !fixture
                    .current_database_mcp_tools
                    .iter()
                    .any(|tool| tool == required_tool),
                "{required_tool} unexpectedly exists in current Database MCP fixture"
            );
        }
    }

    #[test]
    fn finalizer_output_declares_required_lesson_tools() {
        let draft = LessonDraft {
            title: "SQL joins lesson".to_string(),
            topic: "SQL joins".to_string(),
            level: LessonLevel::Beginner,
            objectives: vec!["Write joins".to_string()],
            prerequisites: vec![],
            estimated_minutes: 35,
            content_blocks: vec![],
            resources: vec![],
            exercises: vec![],
            quizzes: vec![],
            assessment_rubric: AssessmentRubric {
                items: vec![],
                passing_score: 0.8,
            },
        };
        let payload = build_lesson_payload(draft, "ready".to_string());
        let tool_name =
            payload["orchestratorPersistencePlan"]["databaseCallPlan"]["steps"][0]["tool"]
                .as_str()
                .unwrap_or_default();

        assert_eq!(tool_name, "database_mcp.create_lesson");
    }
}
