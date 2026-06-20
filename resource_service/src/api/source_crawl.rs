use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    AppResult, ResourceService,
    models::{
        ApiEnvelope, ClaimJobsRequest, CompleteJobRequest, CrawlJobRequest, CrawlSeedRequest,
        FetchArtifactRequest, PageQuery, ProcessFetchArtifactRequest, ScheduleCrawlRequest,
        SourcePatchRequest, SourceRequest,
    },
};

pub async fn create_source(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<SourceRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::SourceSite>>> {
    Ok(Json(ApiEnvelope::ok(service.create_source(payload).await?)))
}

pub async fn list_sources(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::SourceSite>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_sources(query).await?)))
}

pub async fn get_source(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::SourceSite>>> {
    Ok(Json(ApiEnvelope::ok(service.get_source(id).await?)))
}

pub async fn patch_source(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SourcePatchRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::SourceSite>>> {
    Ok(Json(ApiEnvelope::ok(
        service.patch_source(id, payload).await?,
    )))
}

pub async fn create_crawl_seed(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<CrawlSeedRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlSeed>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_crawl_seed(payload).await?,
    )))
}

pub async fn list_crawl_seeds(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::CrawlSeed>>>> {
    Ok(Json(ApiEnvelope::ok(
        service.list_crawl_seeds(query).await?,
    )))
}

pub async fn schedule_crawl(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ScheduleCrawlRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::ScheduleCrawlResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.schedule_crawl(payload).await?,
    )))
}

pub async fn create_crawl_job(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<CrawlJobRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlJob>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_crawl_job(payload).await?,
    )))
}

pub async fn get_crawl_job(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlJob>>> {
    Ok(Json(ApiEnvelope::ok(service.get_crawl_job(id).await?)))
}

pub async fn claim_crawl_jobs(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ClaimJobsRequest>,
) -> AppResult<Json<ApiEnvelope<Vec<crate::models::CrawlJob>>>> {
    Ok(Json(ApiEnvelope::ok(
        service.claim_crawl_jobs(payload).await?,
    )))
}

pub async fn complete_crawl_job(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteJobRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlJob>>> {
    Ok(Json(ApiEnvelope::ok(
        service.complete_crawl_job(id, payload).await?,
    )))
}

pub async fn create_fetch_artifact(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<FetchArtifactRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::FetchArtifact>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_fetch_artifact(payload).await?,
    )))
}

pub async fn process_fetch_artifact(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ProcessFetchArtifactRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::ProcessFetchArtifactResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.process_fetch_artifact(payload).await?,
    )))
}
