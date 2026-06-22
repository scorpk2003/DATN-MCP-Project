#![allow(dead_code)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LessonLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LessonStatus {
    Draft,
    Ready,
    Active,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LessonBlockType {
    Concept,
    Explanation,
    Example,
    CodeExample,
    DiagramInstruction,
    CommonMistake,
    Summary,
    Checkpoint,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseType {
    ShortAnswer,
    MultipleChoice,
    Coding,
    Debugging,
    Design,
    MiniProject,
    Reflection,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Docs,
    Article,
    Book,
    Paper,
    Video,
    Github,
    Course,
    InternalNote,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    ContentBlock,
    Checkpoint,
    Exercise,
    Quiz,
    Review,
    Finish,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MistakeType {
    Conceptual,
    Syntax,
    Reasoning,
    MissingDetail,
    DesignIssue,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonContentBlock {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: LessonBlockType,
    pub title: String,
    pub content: String,
    #[serde(rename = "sourceRefs")]
    pub source_refs: Option<Vec<String>>,
    pub order: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonResource {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: SourceType,
    pub difficulty: LessonLevel,
    #[serde(rename = "relevanceScore")]
    pub relevance_score: f32,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f32>,
    #[serde(rename = "reasonSelected")]
    pub reason_selected: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RubricItem {
    pub criterion: String,
    #[serde(rename = "maxScore")]
    pub max_score: f32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Exercise {
    pub id: String,
    #[serde(rename = "type")]
    pub exercise_type: ExerciseType,
    pub title: String,
    pub prompt: String,
    #[serde(rename = "expectedOutput")]
    pub expected_output: Option<String>,
    pub hints: Vec<String>,
    pub rubric: Vec<RubricItem>,
    pub difficulty: ExerciseDifficulty,
    #[serde(rename = "sourceRefs")]
    pub source_refs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct QuizQuestion {
    pub id: String,
    pub question: String,
    pub choices: Vec<String>,
    #[serde(rename = "correctChoiceIndex")]
    pub correct_choice_index: u32,
    pub explanation: String,
    #[serde(rename = "sourceRefs")]
    pub source_refs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AssessmentRubric {
    pub items: Vec<RubricItem>,
    #[serde(rename = "passingScore")]
    pub passing_score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonDraft {
    pub title: String,
    pub topic: String,
    pub level: LessonLevel,
    pub objectives: Vec<String>,
    pub prerequisites: Vec<String>,
    #[serde(rename = "estimatedMinutes")]
    pub estimated_minutes: u32,
    #[serde(rename = "contentBlocks")]
    pub content_blocks: Vec<LessonContentBlock>,
    pub resources: Vec<LessonResource>,
    pub exercises: Vec<Exercise>,
    pub quizzes: Vec<QuizQuestion>,
    #[serde(rename = "assessmentRubric")]
    pub assessment_rubric: AssessmentRubric,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RoadmapNodeInput {
    pub title: String,
    pub topic: String,
    pub description: String,
    pub level: LessonLevel,
    pub prerequisites: Vec<String>,
    #[serde(rename = "expectedOutcomes")]
    pub expected_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UserContextInput {
    #[serde(rename = "skillLevel")]
    pub skill_level: Option<String>,
    #[serde(rename = "knownTopics")]
    pub known_topics: Option<Vec<String>>,
    #[serde(rename = "weakTopics")]
    pub weak_topics: Option<Vec<String>>,
    #[serde(rename = "learningGoal")]
    pub learning_goal: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResourceQuery {
    pub query: String,
    #[serde(rename = "resourceTypes")]
    pub resource_types: Vec<SourceType>,
    pub difficulty: LessonLevel,
    #[serde(rename = "mustHave")]
    pub must_have: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonRequirement {
    pub topic: String,
    pub level: LessonLevel,
    pub objectives: Vec<String>,
    #[serde(rename = "prerequisiteGaps")]
    pub prerequisite_gaps: Vec<String>,
    #[serde(rename = "resourceQueries")]
    pub resource_queries: Vec<ResourceQuery>,
    #[serde(rename = "recommendedExerciseTypes")]
    pub recommended_exercise_types: Vec<ExerciseType>,
    #[serde(rename = "estimatedMinutes")]
    pub estimated_minutes: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResourceChunkInput {
    #[serde(rename = "chunkId")]
    pub chunk_id: String,
    pub text: String,
    pub score: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResourceCandidateInput {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: SourceType,
    pub summary: Option<String>,
    pub chunks: Option<Vec<ResourceChunkInput>>,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f32>,
    #[serde(rename = "relevanceScore")]
    pub relevance_score: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PackedSelectedChunk {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "chunkId")]
    pub chunk_id: String,
    pub text: String,
    #[serde(rename = "relevanceScore")]
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EvidenceCoverage {
    #[serde(rename = "objectivesCovered")]
    pub objectives_covered: Vec<String>,
    #[serde(rename = "objectivesMissing")]
    pub objectives_missing: Vec<String>,
    #[serde(rename = "coverageScore")]
    pub coverage_score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PackedLessonEvidence {
    #[serde(rename = "primarySources")]
    pub primary_sources: Vec<LessonResource>,
    #[serde(rename = "supportingSources")]
    pub supporting_sources: Vec<LessonResource>,
    #[serde(rename = "codeSources")]
    pub code_sources: Vec<LessonResource>,
    #[serde(rename = "selectedChunks")]
    pub selected_chunks: Vec<PackedSelectedChunk>,
    pub coverage: EvidenceCoverage,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonIssue {
    #[serde(rename = "type")]
    pub issue_type: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ValidationPolicy {
    #[serde(rename = "requireObjectives")]
    pub require_objectives: Option<bool>,
    #[serde(rename = "requireResources")]
    pub require_resources: Option<bool>,
    #[serde(rename = "requireExercises")]
    pub require_exercises: Option<bool>,
    #[serde(rename = "requireQuiz")]
    pub require_quiz: Option<bool>,
    #[serde(rename = "minResourceQualityScore")]
    pub min_resource_quality_score: Option<f32>,
    #[serde(rename = "minContentBlocks")]
    pub min_content_blocks: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GradingMistake {
    #[serde(rename = "type")]
    pub mistake_type: MistakeType,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NextRecommendation {
    pub action: String,
    #[serde(rename = "targetId")]
    pub target_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GradingResult {
    pub status: String,
    pub score: f32,
    pub passed: bool,
    pub feedback: String,
    pub mistakes: Vec<GradingMistake>,
    #[serde(rename = "improvementSuggestions")]
    pub improvement_suggestions: Vec<String>,
    #[serde(rename = "nextRecommendation")]
    pub next_recommendation: NextRecommendation,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SessionSummary {
    #[serde(rename = "completedBlocks")]
    pub completed_blocks: Vec<String>,
    #[serde(rename = "exerciseScores")]
    pub exercise_scores: Vec<ExerciseScore>,
    #[serde(rename = "quizScore")]
    pub quiz_score: Option<f32>,
    #[serde(rename = "checkpointScore")]
    pub checkpoint_score: Option<f32>,
    #[serde(rename = "timeSpentMinutes")]
    pub time_spent_minutes: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ProgressPayload {
    #[serde(rename = "lessonId")]
    pub lesson_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "completionStatus")]
    pub completion_status: String,
    #[serde(rename = "masteryScore")]
    pub mastery_score: f32,
    #[serde(rename = "weakTopics")]
    pub weak_topics: Vec<String>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LessonCompletionResult {
    pub status: String,
    #[serde(rename = "masteryScore")]
    pub mastery_score: f32,
    #[serde(rename = "progressPayload")]
    pub progress_payload: ProgressPayload,
    #[serde(rename = "nextAction")]
    pub next_action: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExerciseScore {
    #[serde(rename = "exerciseId")]
    pub exercise_id: String,
    pub score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DatabaseMcpToolCall {
    #[serde(rename = "toolName")]
    pub tool_name: String,
    pub arguments: Value,
    #[serde(rename = "resultAlias")]
    pub result_alias: Option<String>,
    #[serde(rename = "dependsOn")]
    pub depends_on: Vec<String>,
}
