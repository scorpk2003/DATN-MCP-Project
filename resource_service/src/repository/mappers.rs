use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio_postgres::Row;

use crate::models::{
    CandidateSummary, CrawlJob, CrawlSeed, FetchArtifact, GapSummary, Page, PaginationMeta,
    RecommendedResource, ResourceChunk, ResourceSummary, ResourceVersionSummary, ScoreBreakdown,
    SearchResult, SourceSite,
};

pub(crate) fn page<T: serde::Serialize>(
    items: Vec<T>,
    limit: i64,
    offset: i64,
    total: i64,
) -> Page<T> {
    let has_more = offset + (items.len() as i64) < total;
    Page {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
            has_more,
        },
    }
}

pub(crate) fn row_to_resource_summary(row: &Row) -> ResourceSummary {
    ResourceSummary {
        id: row.get("id"),
        canonical_url: row.get("canonical_url"),
        title: row.get("title"),
        summary: row.get("summary"),
        kind: row.get("kind"),
        format: row.get("format"),
        status: row.get("status"),
        language: row.get("language_code"),
        difficulty: row.get("difficulty"),
        quality_score: row.get("quality_score"),
        is_official: row.get("is_official"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn row_to_version_summary(row: Row) -> ResourceVersionSummary {
    ResourceVersionSummary {
        id: row.get("id"),
        version_no: row.get("version_no"),
        title: row.get("title"),
        extracted_at: row.get("extracted_at"),
    }
}

pub(crate) fn row_to_chunk(row: &Row) -> ResourceChunk {
    let metadata: Value = row.get("metadata");
    ResourceChunk {
        id: row.get("id"),
        version_id: row.get("version_id"),
        chunk_index: row.get("chunk_index"),
        heading_path: row.get("heading_path"),
        content: row.get("content"),
        content_tokens: row.get("content_tokens"),
        content_kind: metadata
            .get("content_kind")
            .and_then(Value::as_str)
            .unwrap_or("mixed")
            .to_string(),
    }
}

pub(crate) fn row_to_search_result(row: &Row) -> SearchResult {
    let content: String = row.get("content");
    let score: f64 = row.get("score");
    SearchResult {
        resource_id: row.get("resource_id"),
        version_id: row.get("version_id"),
        chunk_id: row.get("chunk_id"),
        title: row.get("title"),
        url: row.get("canonical_url"),
        heading_path: row.get("heading_path"),
        snippet: snippet(&content),
        content_kind: row.get("content_kind"),
        scores: ScoreBreakdown {
            keyword: row.get::<_, Option<f64>>("text_score").unwrap_or(0.0),
            vector: row.get::<_, Option<f64>>("vector_score").unwrap_or(0.0),
            quality: row.get("quality_score"),
            freshness: row.get("freshness_score"),
            difficulty_match: 1.0,
            final_score: score,
        },
    }
}

pub(crate) fn row_to_source(row: &Row) -> SourceSite {
    SourceSite {
        id: row.get("id"),
        name: row.get("name"),
        kind: row.get("kind"),
        base_url: row.get("base_url"),
        host: row.get("host"),
        trust_tier: row.get("trust_tier"),
        language_hint: row.get("language_hint"),
        enabled: row.get("enabled"),
        is_official: row.get("is_official"),
        crawl_policy: row.get("crawl_policy"),
        allowed_paths: row.get("allowed_paths"),
        blocked_paths: row.get("blocked_paths"),
    }
}

pub(crate) fn row_to_seed(row: &Row) -> CrawlSeed {
    CrawlSeed {
        id: row.get("id"),
        source_id: row.get("source_id"),
        kind: row.get("kind"),
        seed_value: row.get("seed_value"),
        priority: row.get("priority"),
        enabled: row.get("enabled"),
        metadata: row.get("metadata"),
    }
}

pub(crate) fn row_to_crawl_job(row: &Row) -> CrawlJob {
    CrawlJob {
        id: row.get("id"),
        run_id: row.get("run_id"),
        source_id: row.get("source_id"),
        url: row.get("url"),
        canonical_url: row.get("canonical_url"),
        status: row.get("status"),
        priority: row.get("priority"),
        attempts: row.get("attempts"),
        max_attempts: row.get("max_attempts"),
        last_error: row.get("last_error"),
    }
}

pub(crate) fn row_to_fetch_artifact(row: &Row) -> FetchArtifact {
    let checksum = row
        .get::<_, Option<Vec<u8>>>("body_sha256")
        .map(|bytes| format!("sha256:{}", hex_lower(&bytes)));
    FetchArtifact {
        id: row.get("id"),
        crawl_job_id: row.get("job_id"),
        url: row.get("url"),
        final_url: row.get("final_url"),
        http_status: row.get("http_status"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum,
    }
}

pub(crate) fn row_to_recommended_resource(row: &Row) -> RecommendedResource {
    let is_official: bool = row.get("is_official");
    let quality: f64 = row.get("quality_score");
    let authority: f64 = row.get("authority_score");
    let role = row.try_get::<_, String>("role").unwrap_or_else(|_| {
        if is_official {
            "official_reference".to_string()
        } else {
            "primary_learning".to_string()
        }
    });
    RecommendedResource {
        resource_id: row.get("resource_id"),
        title: row.get("title"),
        url: row.get("canonical_url"),
        role: role.clone(),
        difficulty: row.get("difficulty"),
        reason: recommendation_reason(&role, is_official),
        score: (quality * 0.7) + (authority * 0.3),
        chunk_ids: row.get("chunk_ids"),
    }
}

fn recommendation_reason(role: &str, is_official: bool) -> String {
    match role {
        "official_reference" => "Official or high-authority reference for this topic.".to_string(),
        "practice" => "Practice-oriented resource matched to the requested topic.".to_string(),
        "project" => "Project-oriented resource useful for applying this topic.".to_string(),
        "deep_dive" => "Deep-dive resource for advanced understanding.".to_string(),
        "troubleshooting" => "Troubleshooting resource for debugging common issues.".to_string(),
        _ if is_official => "Official source suitable as a learning reference.".to_string(),
        _ => "Relevant learning resource matched by topic, title, or content.".to_string(),
    }
}

pub(crate) fn row_to_gap(row: &Row) -> GapSummary {
    GapSummary {
        id: row.get("id"),
        topic: row.get("topic_text"),
        normalized_query: row.get("normalized_query"),
        status: row.get("status"),
        priority: row.get("priority"),
        min_required_resources: row.get("min_required_resources"),
        context: row.get("context"),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_research_task(row: &Row) -> crate::models::ResearchTaskSummary {
    crate::models::ResearchTaskSummary {
        id: row.get("id"),
        gap_id: row.get("gap_id"),
        query: row.get("query_text"),
        status: row.get("status"),
        priority: row.get("priority"),
        metadata: row.get("metadata"),
    }
}

pub(crate) fn row_to_candidate(row: &Row) -> CandidateSummary {
    let metadata: Value = row.get("metadata");
    let candidate_type = metadata
        .get("candidateType")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let score = metadata.get("score").and_then(Value::as_f64).unwrap_or(0.0);
    CandidateSummary {
        id: row.get("id"),
        task_id: row.get("task_id"),
        url: row.get("url"),
        canonical_url: row.get("canonical_url"),
        title: row.get("title"),
        selected: row.get("selected"),
        reject_reason: row.get("reject_reason"),
        metadata,
        candidate_type,
        score,
    }
}

pub(crate) fn sha256_bytes(text: &str) -> Vec<u8> {
    Sha256::digest(text.as_bytes()).to_vec()
}

pub(crate) fn extract_domain(raw_url: &str) -> Option<String> {
    url::Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
}

pub(crate) fn normalize_url(raw_url: &str) -> String {
    match url::Url::parse(raw_url) {
        Ok(mut url) => {
            url.set_fragment(None);
            url.to_string()
        }
        Err(_) => raw_url.to_string(),
    }
}

fn snippet(content: &str) -> String {
    let text = content.split_whitespace().collect::<Vec<_>>().join(" ");
    text.chars().take(320).collect()
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
