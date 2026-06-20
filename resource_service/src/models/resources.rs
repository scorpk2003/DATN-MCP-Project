use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateResourceRequest {
    pub title: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    #[serde(rename = "sourceSiteId")]
    pub source_site_id: Option<Uuid>,
    pub language: Option<String>,
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,
    #[serde(rename = "resourceFormat")]
    pub resource_format: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateResourceRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub difficulty: Option<String>,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateResourceVersionRequest {
    pub title: Option<String>,
    pub content: String,
    pub markdown: Option<String>,
    #[serde(rename = "fetchArtifactId")]
    pub fetch_artifact_id: Option<Uuid>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManualIngestRequest {
    pub canonical_url: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub format: Option<String>,
    pub language_code: Option<String>,
    pub primary_domain: Option<String>,
    pub is_official: Option<bool>,
    pub source_id: Option<Uuid>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateResourceResponse {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    pub status: String,
    #[serde(rename = "processingStatus")]
    pub processing_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestResourceResponse {
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "versionId")]
    pub version_id: Uuid,
    #[serde(rename = "chunkCount")]
    pub chunk_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceSummary {
    #[serde(rename = "resourceId")]
    pub id: Uuid,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    pub title: String,
    pub summary: Option<String>,
    #[serde(rename = "resourceType")]
    pub kind: String,
    pub format: String,
    pub status: String,
    pub language: String,
    pub difficulty: String,
    #[serde(rename = "qualityScore")]
    pub quality_score: f64,
    #[serde(rename = "isOfficial")]
    pub is_official: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceDetail {
    pub resource: ResourceSummary,
    #[serde(rename = "sourceName")]
    pub source_name: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: Option<String>,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<ResourceVersionSummary>,
    #[serde(rename = "chunkCount")]
    pub chunk_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceVersionSummary {
    #[serde(rename = "versionId")]
    pub id: Uuid,
    #[serde(rename = "versionNo")]
    pub version_no: i32,
    pub title: Option<String>,
    #[serde(rename = "extractedAt")]
    pub extracted_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceChunk {
    #[serde(rename = "chunkId")]
    pub id: Uuid,
    #[serde(rename = "resourceVersionId")]
    pub version_id: Uuid,
    #[serde(rename = "chunkIndex")]
    pub chunk_index: i32,
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
    pub content: String,
    #[serde(rename = "tokenCount")]
    pub content_tokens: Option<i32>,
    #[serde(rename = "contentKind")]
    pub content_kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceChunksQuery {
    pub version_id: Option<Uuid>,
    #[serde(rename = "maxChunks")]
    pub max_chunks: Option<i64>,
}
