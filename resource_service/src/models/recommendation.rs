use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::CoverageStatus;

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendRequest {
    pub topic: String,
    pub level: Option<String>,
    pub goal: Option<String>,
    #[serde(rename = "requiredTypes")]
    pub required_types: Option<Vec<String>>,
    #[serde(rename = "maxResources")]
    pub max_resources: Option<i64>,
    #[serde(rename = "includeChunks")]
    pub include_chunks: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicCoverageRequest {
    pub topic: String,
    pub level: Option<String>,
    #[serde(rename = "requiredTypes")]
    pub required_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicCoverageResponse {
    pub topic: String,
    #[serde(rename = "normalizedTopic")]
    pub normalized_topic: String,
    pub coverage: CoverageStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendResponse {
    pub topic: String,
    #[serde(rename = "normalizedTopic")]
    pub normalized_topic: String,
    pub level: Option<String>,
    pub resources: Vec<RecommendedResource>,
    pub coverage: CoverageStatus,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendedResource {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    pub title: String,
    pub url: String,
    pub role: String,
    pub difficulty: String,
    pub reason: String,
    pub score: f64,
    #[serde(rename = "chunkIds")]
    pub chunk_ids: Vec<Uuid>,
}
