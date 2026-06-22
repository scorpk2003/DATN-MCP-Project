use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRequest {
    pub name: String,
    pub kind: Option<String>,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "trustTier")]
    pub trust_tier: Option<i16>,
    #[serde(rename = "languageHint")]
    pub language_hint: Option<String>,
    pub enabled: Option<bool>,
    #[serde(rename = "isOfficial")]
    pub is_official: Option<bool>,
    #[serde(rename = "crawlPolicy")]
    pub crawl_policy: Option<Value>,
    #[serde(rename = "allowedPaths")]
    pub allowed_paths: Option<Vec<String>>,
    #[serde(rename = "blockedPaths")]
    pub blocked_paths: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourcePatchRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    #[serde(rename = "crawlPolicy")]
    pub crawl_policy: Option<Value>,
    #[serde(rename = "allowedPaths")]
    pub allowed_paths: Option<Vec<String>>,
    #[serde(rename = "blockedPaths")]
    pub blocked_paths: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceSite {
    #[serde(rename = "sourceSiteId")]
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub host: String,
    #[serde(rename = "trustTier")]
    pub trust_tier: i16,
    #[serde(rename = "languageHint")]
    pub language_hint: String,
    pub enabled: bool,
    #[serde(rename = "isOfficial")]
    pub is_official: bool,
    #[serde(rename = "crawlPolicy")]
    pub crawl_policy: Value,
    #[serde(rename = "allowedPaths")]
    pub allowed_paths: Vec<String>,
    #[serde(rename = "blockedPaths")]
    pub blocked_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlSeedRequest {
    #[serde(rename = "sourceSiteId")]
    pub source_site_id: Option<Uuid>,
    #[serde(rename = "seedUrl")]
    pub seed_url: String,
    #[serde(rename = "seedType")]
    pub seed_type: Option<String>,
    #[serde(rename = "maxDepth")]
    pub max_depth: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlSeed {
    #[serde(rename = "crawlSeedId")]
    pub id: Uuid,
    #[serde(rename = "sourceSiteId")]
    pub source_id: Option<Uuid>,
    #[serde(rename = "seedUrl")]
    pub seed_value: String,
    #[serde(rename = "seedType")]
    pub kind: String,
    pub priority: i32,
    pub enabled: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrawlJobRequest {
    #[serde(rename = "sourceSiteId")]
    pub source_site_id: Option<Uuid>,
    #[serde(rename = "crawlSeedId")]
    pub seed_id: Option<Uuid>,
    #[serde(rename = "crawlRunId")]
    pub run_id: Option<Uuid>,
    pub url: String,
    pub priority: Option<i32>,
    pub depth: Option<i32>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrawlJob {
    #[serde(rename = "crawlJobId")]
    pub id: Uuid,
    #[serde(rename = "crawlRunId")]
    pub run_id: Option<Uuid>,
    #[serde(rename = "sourceSiteId")]
    pub source_id: Option<Uuid>,
    pub url: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    pub status: String,
    pub priority: i32,
    pub depth: i32,
    pub attempts: i32,
    #[serde(rename = "maxAttempts")]
    pub max_attempts: i32,
    #[serde(rename = "lastError")]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimJobsRequest {
    #[serde(rename = "workerId")]
    pub worker_id: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompleteJobRequest {
    pub succeeded: bool,
    #[serde(rename = "httpStatus")]
    pub http_status: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleCrawlRequest {
    #[serde(rename = "sourceSiteId")]
    pub source_site_id: Option<Uuid>,
    pub limit: Option<i64>,
    #[serde(rename = "requestedBy")]
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleCrawlResponse {
    #[serde(rename = "crawlRunId")]
    pub crawl_run_id: Uuid,
    #[serde(rename = "createdJobIds")]
    pub created_job_ids: Vec<Uuid>,
    #[serde(rename = "createdJobCount")]
    pub created_job_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FetchArtifactRequest {
    #[serde(rename = "crawlJobId")]
    pub crawl_job_id: Uuid,
    #[serde(rename = "sourceSiteId")]
    pub source_site_id: Option<Uuid>,
    pub url: String,
    #[serde(rename = "finalUrl")]
    pub final_url: Option<String>,
    #[serde(rename = "httpStatus")]
    pub http_status: Option<i32>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "contentLength")]
    pub content_length: Option<i64>,
    pub etag: Option<String>,
    #[serde(rename = "rawObjectKey")]
    pub raw_object_key: Option<String>,
    #[serde(rename = "rawBody")]
    pub raw_body: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FetchArtifact {
    #[serde(rename = "fetchArtifactId")]
    pub id: Uuid,
    #[serde(rename = "crawlJobId")]
    pub crawl_job_id: Uuid,
    pub url: String,
    #[serde(rename = "finalUrl")]
    pub final_url: Option<String>,
    #[serde(rename = "httpStatus")]
    pub http_status: Option<i32>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "contentLength")]
    pub content_length: Option<i64>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessFetchArtifactRequest {
    #[serde(rename = "fetchArtifactId")]
    pub fetch_artifact_id: Uuid,
    #[serde(rename = "activateResource")]
    pub activate_resource: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessFetchArtifactResponse {
    #[serde(rename = "fetchArtifactId")]
    pub fetch_artifact_id: Uuid,
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "versionId")]
    pub version_id: Uuid,
    #[serde(rename = "chunkCount")]
    pub chunk_count: i32,
    pub action: String,
    pub title: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
}
