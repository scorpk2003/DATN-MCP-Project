use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{
    GradingMistake, LessonDraft, LessonRequirement, RemediationResourceRef, ResourceCandidateInput,
    RoadmapNodeInput, RubricItem, SessionSummary, UserContextInput, ValidationPolicy,
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AuthContext {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub verified: bool,
    pub scope: Vec<String>,
    #[serde(rename = "verifiedBy")]
    pub verified_by: Option<String>,
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonAnalyzeNodeParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "roadmapId")]
    pub roadmap_id: String,
    #[serde(rename = "roadmapNodeId")]
    pub roadmap_node_id: String,
    pub node: RoadmapNodeInput,
    #[serde(rename = "userContext")]
    pub user_context: Option<UserContextInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonCreateDraftConstraints {
    #[serde(rename = "maxMinutes")]
    pub max_minutes: Option<u32>,
    #[serde(rename = "includeCode")]
    pub include_code: Option<bool>,
    #[serde(rename = "includeQuiz")]
    pub include_quiz: Option<bool>,
    #[serde(rename = "includeMiniProject")]
    pub include_mini_project: Option<bool>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonCreateDraftParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "roadmapId")]
    pub roadmap_id: String,
    #[serde(rename = "roadmapNodeId")]
    pub roadmap_node_id: String,
    #[serde(rename = "lessonRequirement")]
    pub lesson_requirement: LessonRequirement,
    pub resources: Vec<ResourceCandidateInput>,
    pub constraints: Option<LessonCreateDraftConstraints>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonValidateDraftParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "lessonDraft")]
    pub lesson_draft: LessonDraft,
    #[serde(rename = "validationPolicy")]
    pub validation_policy: Option<ValidationPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonFinalizeSavePolicy {
    pub status: Option<String>,
    #[serde(rename = "dedupeByRoadmapNode")]
    pub dedupe_by_roadmap_node: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonFinalizeParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "lessonDraft")]
    pub lesson_draft: LessonDraft,
    #[serde(rename = "savePolicy")]
    pub save_policy: Option<LessonFinalizeSavePolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonGradeAnswerParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "lessonId")]
    pub lesson_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub activity: Value,
    pub answer: String,
    pub rubric: Vec<RubricItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonCompleteSessionParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "lessonId")]
    pub lesson_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "sessionSummary")]
    pub session_summary: SessionSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemediationGradingResultInput {
    pub score: f32,
    pub passed: bool,
    pub mistakes: Vec<GradingMistake>,
    pub feedback: String,
    #[serde(rename = "masteryGap")]
    pub mastery_gap: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemediationSubmissionInput {
    #[serde(rename = "type")]
    pub submission_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemediationConstraints {
    #[serde(rename = "maxBlocks")]
    pub max_blocks: Option<u32>,
    pub difficulty: Option<String>,
    #[serde(rename = "includeRetryActivity")]
    pub include_retry_activity: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonGenerateRemediationParam {
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "authContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "roadmapId")]
    pub roadmap_id: String,
    #[serde(rename = "roadmapNodeId")]
    pub roadmap_node_id: String,
    #[serde(rename = "lessonId")]
    pub lesson_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "activityId")]
    pub activity_id: String,
    #[serde(rename = "gradingResult")]
    pub grading_result: RemediationGradingResultInput,
    pub submission: RemediationSubmissionInput,
    #[serde(rename = "resourceRefs")]
    pub resource_refs: Vec<RemediationResourceRef>,
    pub constraints: Option<RemediationConstraints>,
}
