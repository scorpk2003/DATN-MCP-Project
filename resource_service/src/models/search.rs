use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchFilters {
    pub language: Option<String>,
    pub difficulty: Option<String>,
    #[serde(rename = "resourceTypes")]
    pub resource_types: Option<Vec<String>>,
    #[serde(rename = "minQualityScore")]
    pub min_quality_score: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub limit: Option<i64>,
    #[serde(rename = "maxChunksPerResource")]
    pub max_chunks_per_resource: Option<usize>,
    #[serde(rename = "includeCoverage")]
    pub include_coverage: Option<bool>,
    #[serde(rename = "createGapOnLowConfidence")]
    pub create_gap_on_low_confidence: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    pub coverage: CoverageStatus,
    #[serde(rename = "queryInfo")]
    pub query_info: QueryInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryInfo {
    #[serde(rename = "normalizedQuery")]
    pub normalized_query: String,
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "resourceVersionId")]
    pub version_id: Uuid,
    #[serde(rename = "chunkId")]
    pub chunk_id: Uuid,
    pub title: String,
    pub url: String,
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
    pub snippet: String,
    #[serde(rename = "contentKind")]
    pub content_kind: String,
    pub scores: ScoreBreakdown,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreBreakdown {
    pub keyword: f64,
    pub vector: f64,
    pub quality: f64,
    pub freshness: f64,
    #[serde(rename = "difficultyMatch")]
    pub difficulty_match: f64,
    #[serde(rename = "final")]
    pub final_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageStatus {
    pub status: String,
    #[serde(rename = "lowConfidence")]
    pub low_confidence: bool,
    #[serde(rename = "missingTypes")]
    pub missing_types: Vec<String>,
    #[serde(rename = "resultCount")]
    pub result_count: i64,
    #[serde(rename = "bestScore")]
    pub best_score: f64,
    #[serde(rename = "gapId")]
    pub gap_id: Option<Uuid>,
}
