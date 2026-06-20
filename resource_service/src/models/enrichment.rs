use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct EnrichResourceRequest {
    #[serde(rename = "resourceVersionId")]
    pub resource_version_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichmentMatch {
    pub slug: String,
    pub name: String,
    pub score: f64,
    #[serde(rename = "evidenceChunkIds")]
    pub evidence_chunk_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichResourceResponse {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "resourceVersionId")]
    pub resource_version_id: Uuid,
    pub summary: String,
    pub difficulty: String,
    pub topics: Vec<EnrichmentMatch>,
    pub concepts: Vec<EnrichmentMatch>,
    pub prerequisites: Vec<String>,
    #[serde(rename = "learningOutcomes")]
    pub learning_outcomes: Vec<String>,
    #[serde(rename = "resourceRoles")]
    pub resource_roles: Vec<String>,
    pub confidence: f64,
}
