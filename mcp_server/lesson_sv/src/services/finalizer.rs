use serde_json::{Value, json};

use crate::domain::{DatabaseMcpToolCall, LessonDraft};

pub fn build_lesson_payload(draft: LessonDraft, status: String) -> Value {
    let idempotency_key = build_idempotency_key(&draft, &status);
    let calls = build_database_calls(&draft, &status, &idempotency_key);
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
            "executionPolicy": "Execute in order. Treat create_lesson as the parent record and resolve ${lesson.lessonId} for child rows.",
        }
    })
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
    }
}
