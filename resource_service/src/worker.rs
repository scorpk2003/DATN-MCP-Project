use std::{str, sync::Arc, time::Duration};

use reqwest::{Client, redirect::Policy};
use serde_json::json;
use tokio::time::sleep;
use tracing::{error, info, warn};
use url::Url;

use crate::{
    AppError, AppResult, ResourceService,
    embedding_provider::deterministic_embedding,
    models::{
        ClaimJobsRequest, CompleteJobRequest, FetchArtifactRequest, PendingEmbeddingChunksQuery,
        ProcessFetchArtifactRequest, SourceSite,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerMode {
    Fetcher,
    Extractor,
    Embedding,
    All,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkerRunOptions {
    pub mode: WorkerMode,
    pub once: bool,
}

pub async fn run_worker(service: Arc<ResourceService>, options: WorkerRunOptions) -> AppResult<()> {
    loop {
        let mut processed = 0_usize;
        if matches!(options.mode, WorkerMode::Fetcher | WorkerMode::All) {
            processed += run_fetcher_once(service.clone()).await?;
        }
        if matches!(options.mode, WorkerMode::Extractor | WorkerMode::All) {
            processed += run_extractor_once(service.clone()).await?;
        }
        if matches!(options.mode, WorkerMode::Embedding | WorkerMode::All) {
            processed += run_embedding_once(service.clone()).await?;
        }

        if options.once {
            return Ok(());
        }
        if processed == 0 {
            sleep(Duration::from_millis(
                service.config.worker.poll_interval_ms,
            ))
            .await;
        }
    }
}

pub async fn run_embedding_once(service: Arc<ResourceService>) -> AppResult<usize> {
    let model = service.repository.get_default_embedding_model().await?;
    if !model.supports_inline_pgvector() {
        return Err(AppError::Validation(
            "embedding worker currently supports 1536-dimensional pgvector storage".to_string(),
        ));
    }
    let pending = service
        .list_pending_embedding_chunks(PendingEmbeddingChunksQuery {
            embedding_model_id: Some(model.id),
            limit: Some(service.config.worker.batch_size),
        })
        .await?;
    let mut processed = 0_usize;

    for chunk in pending {
        let vector = deterministic_embedding(&chunk.input_text, model.dimensions as usize)?;
        let metadata = json!({
            "provider": model.provider,
            "model": model.name,
            "workerId": service.config.worker.worker_id,
            "embeddingProvider": "deterministic_local_v1",
            "inputTokenEstimate": chunk.token_count
        });
        service
            .repository
            .upsert_chunk_embedding(chunk.chunk_id, model.id, &vector, metadata)
            .await?;
        processed += 1;
        info!(
            chunk_id = %chunk.chunk_id,
            resource_id = %chunk.resource_id,
            model_id = %model.id,
            "chunk embedding persisted"
        );
    }

    Ok(processed)
}

pub async fn run_fetcher_once(service: Arc<ResourceService>) -> AppResult<usize> {
    let request = ClaimJobsRequest {
        worker_id: service.config.worker.worker_id.clone(),
        limit: Some(service.config.worker.batch_size),
    };
    let jobs = service.claim_crawl_jobs(request).await?;
    let mut processed = 0_usize;

    for job in jobs {
        let job_id = job.id;
        let url = job.url.clone();
        let source_id = job.source_id;
        info!(%job_id, %url, "fetcher claimed crawl job");

        let result = async {
            let source = if let Some(source_id) = source_id {
                let source = service.repository.get_source_policy(source_id).await?;
                validate_fetch_policy(&source, &url, job.depth)?;
                Some(source)
            } else {
                None
            };
            let response = fetch_url(
                &url,
                Duration::from_millis(service.config.worker.http_timeout_ms),
                service.config.worker.max_body_bytes,
            )
            .await?;
            if let Some(source) = &source {
                validate_fetch_policy(source, &response.final_url, job.depth)?;
            }
            Ok::<_, AppError>(response)
        }
        .await;

        match result {
            Ok(response) => {
                let succeeded = (200..400).contains(&response.status);
                let artifact = FetchArtifactRequest {
                    crawl_job_id: job_id,
                    source_site_id: source_id,
                    url,
                    final_url: Some(response.final_url),
                    http_status: Some(response.status),
                    content_type: response.content_type,
                    content_length: Some(response.body.len() as i64),
                    etag: response.etag,
                    raw_object_key: None,
                    raw_body: Some(response.body),
                    metadata: Some(json!({
                        "workerId": service.config.worker.worker_id,
                        "fetcher": "resource_worker_http_v1"
                    })),
                };

                match service.create_fetch_artifact(artifact).await {
                    Ok(artifact) => {
                        processed += 1;
                        info!(
                            %job_id,
                            fetch_artifact_id = %artifact.id,
                            succeeded,
                            "fetch artifact stored"
                        );
                    }
                    Err(err) => {
                        warn!(%job_id, error = %err, "failed to store fetch artifact");
                        fail_job(
                            service.as_ref(),
                            job_id,
                            Some(response.status),
                            err.to_string(),
                        )
                        .await?;
                    }
                }
            }
            Err(err) => {
                warn!(%job_id, error = %err, "fetch failed");
                fail_job(service.as_ref(), job_id, None, err.to_string()).await?;
            }
        }
    }

    Ok(processed)
}

fn validate_fetch_policy(source: &SourceSite, url: &str, depth: i32) -> AppResult<()> {
    if !source.enabled {
        return Err(AppError::Validation("source is disabled".to_string()));
    }
    if let Some(max_depth) = source_max_depth(source) {
        if depth > max_depth {
            return Err(AppError::Validation(format!(
                "crawl job depth {depth} exceeds source maxDepth {max_depth}"
            )));
        }
    }
    let parsed =
        Url::parse(url).map_err(|err| AppError::Validation(format!("invalid url: {err}")))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::Validation("url host is required".to_string()))?;
    if !host.eq_ignore_ascii_case(&source.host) {
        return Err(AppError::Validation(format!(
            "url host {host} does not match source host {}",
            source.host
        )));
    }
    let path = parsed.path();
    if !source.allowed_paths.is_empty()
        && !source
            .allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed))
    {
        return Err(AppError::Validation(format!(
            "url path {path} is not allowed by source policy"
        )));
    }
    if source
        .blocked_paths
        .iter()
        .any(|blocked| path.starts_with(blocked))
    {
        return Err(AppError::Validation(format!(
            "url path {path} is blocked by source policy"
        )));
    }
    Ok(())
}

fn source_max_depth(source: &SourceSite) -> Option<i32> {
    source
        .crawl_policy
        .get("maxDepth")
        .or_else(|| source.crawl_policy.get("max_depth"))
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

pub async fn run_extractor_once(service: Arc<ResourceService>) -> AppResult<usize> {
    let artifact_ids = service
        .repository
        .list_unprocessed_fetch_artifact_ids(service.config.worker.batch_size)
        .await?;
    let mut processed = 0_usize;

    for fetch_artifact_id in artifact_ids {
        info!(%fetch_artifact_id, "extractor processing artifact");
        let request = ProcessFetchArtifactRequest {
            fetch_artifact_id,
            activate_resource: Some(true),
        };
        match service.process_fetch_artifact(request).await {
            Ok(response) => {
                processed += 1;
                info!(
                    %fetch_artifact_id,
                    resource_id = %response.resource_id,
                    version_id = %response.version_id,
                    chunk_count = response.chunk_count,
                    "artifact extracted"
                );
            }
            Err(err) => {
                error!(%fetch_artifact_id, error = %err, "artifact extraction failed");
            }
        }
    }

    Ok(processed)
}

async fn fail_job(
    service: &ResourceService,
    job_id: uuid::Uuid,
    http_status: Option<i32>,
    error: String,
) -> AppResult<()> {
    let request = CompleteJobRequest {
        succeeded: false,
        http_status,
        error: Some(error),
    };
    service.complete_crawl_job(job_id, request).await?;
    Ok(())
}

#[derive(Debug)]
struct FetchResponse {
    status: i32,
    final_url: String,
    redirect_location: Option<String>,
    content_type: Option<String>,
    etag: Option<String>,
    body: String,
}

async fn fetch_url(
    url: &str,
    request_timeout: Duration,
    max_body_bytes: usize,
) -> AppResult<FetchResponse> {
    let mut current_url = url.to_string();
    for _ in 0..5 {
        let response = fetch_url_once(&current_url, request_timeout, max_body_bytes).await?;
        if !(300..400).contains(&response.status) {
            return Ok(response);
        }
        let Some(location) = response.redirect_location.clone() else {
            return Ok(response);
        };
        let next_url = resolve_redirect(&current_url, &location)?;
        ensure_same_host_redirect(&current_url, &next_url)?;
        current_url = next_url;
    }
    Err(AppError::Validation("too many redirects".to_string()))
}

async fn fetch_url_once(
    url: &str,
    request_timeout: Duration,
    max_body_bytes: usize,
) -> AppResult<FetchResponse> {
    let parsed =
        Url::parse(url).map_err(|err| AppError::Validation(format!("invalid url: {err}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "resource_worker fetcher supports http and https URLs".to_string(),
        ));
    }

    let client = Client::builder()
        .redirect(Policy::none())
        .timeout(request_timeout)
        .user_agent("resource_worker/0.2")
        .build()
        .map_err(|err| AppError::Internal(format!("failed to build HTTP client: {err}")))?;
    let response = client
        .get(url)
        .header(
            reqwest::header::ACCEPT,
            "text/html,text/markdown,text/plain,application/json",
        )
        .send()
        .await
        .map_err(|err| AppError::Internal(format!("fetch failed: {err}")))?;

    let status = response.status().as_u16() as i32;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let redirect_location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let final_url = redirect_location
        .as_deref()
        .and_then(|location| resolve_redirect(url, location).ok())
        .unwrap_or_else(|| response.url().to_string());

    if !(300..400).contains(&status) {
        validate_fetch_content_type(content_type.as_deref())?;
        if let Some(content_length) = response.content_length() {
            if content_length > max_body_bytes as u64 {
                return Err(AppError::Validation(
                    "response body exceeded max size".to_string(),
                ));
            }
        }
    }
    let body_bytes = response
        .bytes()
        .await
        .map_err(|err| AppError::Internal(format!("failed reading response body: {err}")))?;
    if body_bytes.len() > max_body_bytes {
        return Err(AppError::Validation(
            "response body exceeded max size".to_string(),
        ));
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok(FetchResponse {
        status,
        final_url,
        redirect_location,
        content_type,
        etag,
        body,
    })
}

fn resolve_redirect(current_url: &str, location: &str) -> AppResult<String> {
    let base = Url::parse(current_url)
        .map_err(|err| AppError::Validation(format!("invalid url: {err}")))?;
    let mut next = base
        .join(location)
        .map_err(|err| AppError::Validation(format!("invalid redirect location: {err}")))?;
    next.set_fragment(None);
    if !matches!(next.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "redirect target must use http or https".to_string(),
        ));
    }
    Ok(next.to_string())
}

fn ensure_same_host_redirect(current_url: &str, next_url: &str) -> AppResult<()> {
    let current = Url::parse(current_url)
        .map_err(|err| AppError::Validation(format!("invalid url: {err}")))?;
    let next =
        Url::parse(next_url).map_err(|err| AppError::Validation(format!("invalid url: {err}")))?;
    if current.host_str() == next.host_str() {
        Ok(())
    } else {
        Err(AppError::Validation(
            "cross-host redirects are blocked by worker policy".to_string(),
        ))
    }
}

#[cfg(test)]
fn path_and_query(url: &Url) -> String {
    let mut path = url.path().to_string();
    if path.is_empty() {
        path.push('/');
    }
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }
    path
}

#[cfg(test)]
fn parse_http_response(
    url: &str,
    response: &[u8],
    max_body_bytes: usize,
) -> AppResult<FetchResponse> {
    let split_at = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| AppError::Internal("invalid HTTP response".to_string()))?;
    let (head, body) = response.split_at(split_at + 4);
    let head = str::from_utf8(head)
        .map_err(|_| AppError::Internal("HTTP response head is not utf-8".to_string()))?;
    if body.len() > max_body_bytes {
        return Err(AppError::Validation(
            "response body exceeded max size".to_string(),
        ));
    }
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<i32>().ok())
        .ok_or_else(|| AppError::Internal("HTTP status was missing".to_string()))?;
    let content_type = header_value(head, "content-type");
    if !(300..400).contains(&status) {
        validate_fetch_content_type(content_type.as_deref())?;
    }
    let body = String::from_utf8_lossy(body).to_string();
    let redirect_location = header_value(head, "location");

    Ok(FetchResponse {
        status,
        final_url: redirect_location
            .as_deref()
            .and_then(|location| resolve_redirect(url, location).ok())
            .unwrap_or_else(|| url.to_string()),
        redirect_location,
        content_type,
        etag: header_value(head, "etag"),
        body,
    })
}

#[cfg(test)]
fn header_value(head: &str, name: &str) -> Option<String> {
    head.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_string())
    })
}

fn validate_fetch_content_type(content_type: Option<&str>) -> AppResult<()> {
    let content_type = content_type
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .unwrap_or("text/plain")
        .to_ascii_lowercase();
    match content_type.as_str() {
        "text/html" | "text/markdown" | "text/plain" | "application/json" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "unsupported content type from fetch: {content_type}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn test_source() -> SourceSite {
        SourceSite {
            id: Uuid::new_v4(),
            name: "Docs".to_string(),
            kind: "official_docs".to_string(),
            base_url: "http://example.test".to_string(),
            host: "example.test".to_string(),
            trust_tier: 1,
            language_hint: "en".to_string(),
            enabled: true,
            is_official: true,
            crawl_policy: json!({"maxDepth": 2}),
            allowed_paths: vec!["/docs".to_string()],
            blocked_paths: vec!["/docs/private".to_string()],
        }
    }

    #[test]
    fn parses_basic_http_response() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nETag: abc\r\n\r\n<html><title>Ok</title></html>";
        let response = parse_http_response("http://example.test/doc", raw, 1024).unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(
            response.content_type.as_deref(),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(response.etag.as_deref(), Some("abc"));
        assert!(response.body.contains("<title>Ok</title>"));
    }

    #[test]
    fn rejects_unsupported_content_type() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\n\r\npng";
        let err = parse_http_response("http://example.test/image", raw, 1024).unwrap_err();
        assert_eq!(err.code(), "VALIDATION_ERROR");
    }

    #[test]
    fn builds_path_with_query() {
        let url = Url::parse("http://example.test/docs?page=1").unwrap();
        assert_eq!(path_and_query(&url), "/docs?page=1");
    }

    #[test]
    fn fetch_policy_allows_and_blocks_paths() {
        let source = test_source();

        assert!(validate_fetch_policy(&source, "http://example.test/docs/page", 1).is_ok());
        assert!(validate_fetch_policy(&source, "http://example.test/blog/page", 1).is_err());
        assert!(validate_fetch_policy(&source, "http://example.test/docs/private/a", 1).is_err());
        assert!(validate_fetch_policy(&source, "http://other.test/docs/page", 1).is_err());
        assert!(validate_fetch_policy(&source, "http://example.test/docs/deep", 3).is_err());
    }

    #[test]
    fn parses_redirect_location_without_content_type() {
        let raw = b"HTTP/1.1 301 Moved Permanently\r\nLocation: /docs/new\r\n\r\n";
        let response = parse_http_response("http://example.test/docs/old", raw, 1024).unwrap();

        assert_eq!(response.status, 301);
        assert_eq!(response.final_url, "http://example.test/docs/new");
        assert_eq!(response.redirect_location.as_deref(), Some("/docs/new"));
    }

    #[test]
    fn cross_host_redirect_is_blocked() {
        let err =
            ensure_same_host_redirect("http://example.test/docs/old", "http://other.test/docs/new")
                .unwrap_err();

        assert_eq!(err.code(), "VALIDATION_ERROR");
    }
}
