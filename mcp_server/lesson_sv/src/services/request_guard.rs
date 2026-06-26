use serde_json::json;

use crate::{
    domain::{
        LessonCreateDraftParam, LessonDraft, LessonGenerateRemediationParam,
        LessonGradeAnswerParam, ResourceCandidateInput, SessionSummary,
    },
    error::{LessonErrorCode, LessonToolError},
};

const MAX_ANSWER_BYTES: usize = 8 * 1024;
const MAX_CODE_SUBMISSION_BYTES: usize = 64 * 1024;
const MAX_RESOURCE_PACK_BYTES: usize = 256 * 1024;
const MAX_SINGLE_CHUNK_BYTES: usize = 16 * 1024;
const MAX_RESOURCE_CHUNKS_PER_LESSON: usize = 20;
const MAX_DRAFT_BLOCKS: usize = 20;
const MAX_QUIZ_ITEMS: usize = 20;

pub fn require_context(
    user_id: &str,
    roadmap_id: &str,
    roadmap_node_id: &str,
) -> Result<(), LessonToolError> {
    if user_id.trim().is_empty()
        || roadmap_id.trim().is_empty()
        || roadmap_node_id.trim().is_empty()
    {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "userId, roadmapId, and roadmapNodeId are required.",
            json!({
                "userIdEmpty": user_id.trim().is_empty(),
                "roadmapIdEmpty": roadmap_id.trim().is_empty(),
                "roadmapNodeIdEmpty": roadmap_node_id.trim().is_empty(),
            }),
        ));
    }

    Ok(())
}

pub fn require_lesson_session_context(
    user_id: &str,
    lesson_id: &str,
    session_id: &str,
) -> Result<(), LessonToolError> {
    if user_id.trim().is_empty() || lesson_id.trim().is_empty() || session_id.trim().is_empty() {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "userId, lessonId, and sessionId are required.",
            json!({
                "userIdEmpty": user_id.trim().is_empty(),
                "lessonIdEmpty": lesson_id.trim().is_empty(),
                "sessionIdEmpty": session_id.trim().is_empty(),
            }),
        ));
    }

    Ok(())
}

pub fn validate_create_draft(param: &LessonCreateDraftParam) -> Result<(), LessonToolError> {
    validate_resource_pack(&param.resources)
}

pub fn validate_resource_pack(resources: &[ResourceCandidateInput]) -> Result<(), LessonToolError> {
    let chunk_count = resources
        .iter()
        .map(|resource| resource.chunks.as_ref().map(Vec::len).unwrap_or(0))
        .sum::<usize>();
    let total_bytes = resources
        .iter()
        .map(|resource| {
            resource.title.len()
                + resource.summary.as_ref().map(String::len).unwrap_or(0)
                + resource
                    .chunks
                    .as_ref()
                    .map(|chunks| chunks.iter().map(|chunk| chunk.text.len()).sum::<usize>())
                    .unwrap_or(0)
        })
        .sum::<usize>();

    if chunk_count > MAX_RESOURCE_CHUNKS_PER_LESSON {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "Too many resource chunks for one lesson.",
            json!({
                "chunkCount": chunk_count,
                "maxChunkCount": MAX_RESOURCE_CHUNKS_PER_LESSON,
            }),
        ));
    }
    if total_bytes > MAX_RESOURCE_PACK_BYTES {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "Resource pack input is too large.",
            json!({
                "resourcePackBytes": total_bytes,
                "maxResourcePackBytes": MAX_RESOURCE_PACK_BYTES,
            }),
        ));
    }

    for resource in resources {
        for chunk in resource.chunks.as_ref().into_iter().flatten() {
            if chunk.text.len() > MAX_SINGLE_CHUNK_BYTES {
                return Err(LessonToolError::new(
                    LessonErrorCode::InvalidInput,
                    "A resource chunk is too large.",
                    json!({
                        "resourceId": resource.id,
                        "chunkId": chunk.chunk_id,
                        "chunkBytes": chunk.text.len(),
                        "maxChunkBytes": MAX_SINGLE_CHUNK_BYTES,
                    }),
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_draft_size(draft: &LessonDraft) -> Result<(), LessonToolError> {
    if draft.content_blocks.len() > MAX_DRAFT_BLOCKS {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "Lesson draft has too many content blocks.",
            json!({
                "blockCount": draft.content_blocks.len(),
                "maxBlockCount": MAX_DRAFT_BLOCKS,
            }),
        ));
    }
    if draft.quizzes.len() > MAX_QUIZ_ITEMS {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "Lesson draft has too many quiz items.",
            json!({
                "quizCount": draft.quizzes.len(),
                "maxQuizCount": MAX_QUIZ_ITEMS,
            }),
        ));
    }

    Ok(())
}

pub fn validate_answer(param: &LessonGradeAnswerParam) -> Result<(), LessonToolError> {
    if param.answer.trim().is_empty() {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "answer is required.",
            json!({ "activity": param.activity }),
        ));
    }
    if param.answer.len() > MAX_ANSWER_BYTES {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "answer is too large.",
            json!({
                "answerBytes": param.answer.len(),
                "maxAnswerBytes": MAX_ANSWER_BYTES,
            }),
        ));
    }
    if param.answer.len() > MAX_CODE_SUBMISSION_BYTES {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "code submission is too large.",
            json!({
                "submissionBytes": param.answer.len(),
                "maxSubmissionBytes": MAX_CODE_SUBMISSION_BYTES,
            }),
        ));
    }

    Ok(())
}

pub fn validate_remediation_request(
    param: &LessonGenerateRemediationParam,
) -> Result<(), LessonToolError> {
    if param.activity_id.trim().is_empty() {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "activityId is required.",
            json!({ "activityIdEmpty": true }),
        ));
    }
    if param.submission.content.len() > MAX_ANSWER_BYTES {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "submission content is too large.",
            json!({
                "submissionBytes": param.submission.content.len(),
                "maxSubmissionBytes": MAX_ANSWER_BYTES,
            }),
        ));
    }
    if !(0.0..=1.0).contains(&param.grading_result.score) {
        return Err(LessonToolError::new(
            LessonErrorCode::InvalidInput,
            "gradingResult.score must be between 0 and 1.",
            json!({ "score": param.grading_result.score }),
        ));
    }

    Ok(())
}

pub fn validate_session_scores(summary: &SessionSummary) -> Result<(), LessonToolError> {
    for score in &summary.exercise_scores {
        if !(0.0..=1.0).contains(&score.score) {
            return Err(LessonToolError::new(
                LessonErrorCode::InvalidInput,
                "exercise score must be between 0 and 1.",
                json!({
                    "exerciseId": score.exercise_id,
                    "score": score.score,
                }),
            ));
        }
    }

    for (name, score) in [
        ("quizScore", summary.quiz_score),
        ("checkpointScore", summary.checkpoint_score),
    ] {
        if let Some(score) = score {
            if !(0.0..=1.0).contains(&score) {
                return Err(LessonToolError::new(
                    LessonErrorCode::InvalidInput,
                    format!("{name} must be between 0 and 1."),
                    json!({
                        "field": name,
                        "score": score,
                    }),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ResourceCandidateInput, ResourceChunkInput, SourceType};

    #[test]
    fn rejects_missing_context() {
        let error = require_context("", "roadmap", "node").unwrap_err();
        assert_eq!(error.code, LessonErrorCode::InvalidInput);
    }

    #[test]
    fn rejects_oversized_resource_chunk() {
        let resources = vec![ResourceCandidateInput {
            id: "res".to_string(),
            title: "Resource".to_string(),
            url: None,
            source_type: SourceType::Docs,
            summary: None,
            chunks: Some(vec![ResourceChunkInput {
                chunk_id: "chunk".to_string(),
                text: "x".repeat(MAX_SINGLE_CHUNK_BYTES + 1),
                score: Some(0.9),
            }]),
            quality_score: Some(0.9),
            relevance_score: Some(0.9),
        }];

        let error = validate_resource_pack(&resources).unwrap_err();
        assert_eq!(error.code, LessonErrorCode::InvalidInput);
    }
}
