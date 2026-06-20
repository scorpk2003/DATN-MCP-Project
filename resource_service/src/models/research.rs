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
}

#[derive(Debug, Clone, Deserialize)]
pub struct RejectCandidateRequest {
    pub reason: String,
}
