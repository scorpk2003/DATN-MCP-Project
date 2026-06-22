use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct GapSummary {
    #[serde(rename = "gapId")]
    pub id: Uuid,
    pub topic: String,
    #[serde(rename = "normalizedTopic")]
    pub normalized_query: String,
    pub status: String,
    pub priority: i32,
    #[serde(rename = "minRequiredResources")]
    pub min_required_resources: i32,
    pub context: Value,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportGapRequest {
    pub topic: String,
    pub level: Option<String>,
    #[serde(rename = "missingTypes")]
    pub missing_types: Option<Vec<String>>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportGapResponse {
    #[serde(rename = "gapId")]
    pub gap_id: Option<Uuid>,
    pub created: bool,
    pub status: String,
    #[serde(rename = "researchTaskId")]
    pub research_task_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchTaskRequest {
    pub topic: String,
    #[serde(rename = "gapId")]
    pub gap_id: Option<Uuid>,
    pub language: Option<String>,
    pub priority: Option<i32>,
    #[serde(rename = "targetResourceTypes")]
    pub target_resource_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchTaskSummary {
    #[serde(rename = "researchTaskId")]
    pub id: Uuid,
    #[serde(rename = "gapId")]
    pub gap_id: Option<Uuid>,
    pub query: String,
    pub status: String,
    pub priority: i32,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandidateRequest {
    #[serde(rename = "researchTaskId")]
    pub research_task_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateScoringDetails {
    #[serde(rename = "relevanceScore")]
    pub relevance_score: f64,
    #[serde(rename = "authorityScore")]
    pub authority_score: f64,
    #[serde(rename = "freshnessScore")]
    pub freshness_score: f64,
    #[serde(rename = "contentDepthScore")]
    pub content_depth_score: f64,
    #[serde(rename = "duplicatePenalty")]
    pub duplicate_penalty: f64,
    #[serde(rename = "languageMatchScore")]
    pub language_match_score: f64,
    #[serde(rename = "finalScore")]
    pub final_score: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateSummary {
    #[serde(rename = "candidateId")]
    pub id: Uuid,
    #[serde(rename = "researchTaskId")]
    pub task_id: Uuid,
    pub url: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    pub title: Option<String>,
    pub selected: bool,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,
    pub metadata: Value,
    #[serde(rename = "candidateType")]
    pub candidate_type: String,
    pub score: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubDiscoveryRequest {
    pub query: Option<String>,
    pub language: Option<String>,
    #[serde(rename = "minStars")]
    pub min_stars: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitHubDiscoveryResponse {
    #[serde(rename = "researchTaskId")]
    pub research_task_id: Uuid,
    pub provider: String,
    pub query: String,
    #[serde(rename = "createdCandidateCount")]
    pub created_candidate_count: usize,
    pub candidates: Vec<CandidateSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApproveCandidateResponse {
    pub candidate: CandidateSummary,
    #[serde(rename = "createdCrawlSeedId")]
    pub created_crawl_seed_id: Option<Uuid>,
    #[serde(rename = "createdCrawlJobId")]
    pub created_crawl_job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RejectCandidateRequest {
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminResourceActionRequest {
    pub reason: Option<String>,
    #[serde(rename = "actorId")]
    pub actor_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardSummary {
    #[serde(rename = "failedJobs")]
    pub failed_jobs: i64,
    #[serde(rename = "openGaps")]
    pub open_gaps: i64,
    #[serde(rename = "pendingCandidates")]
    pub pending_candidates: i64,
    #[serde(rename = "resourcesNeedReview")]
    pub resources_need_review: i64,
    #[serde(rename = "outdatedResources")]
    pub outdated_resources: i64,
    #[serde(rename = "lastCrawlRunStatus")]
    pub last_crawl_run_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminResourceActionResponse {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    pub action: String,
    pub status: String,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f64>,
}
