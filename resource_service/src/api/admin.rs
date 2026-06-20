use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{AppResult, ResourceService, models::ApiEnvelope, models::PageQuery};

pub async fn list_crawl_jobs(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::CrawlJob>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_crawl_jobs(query).await?)))
}

pub async fn retry_crawl_job(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlJob>>> {
    Ok(Json(ApiEnvelope::ok(service.retry_crawl_job(id).await?)))
}

pub async fn cancel_crawl_job(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::CrawlJob>>> {
    Ok(Json(ApiEnvelope::ok(service.cancel_crawl_job(id).await?)))
}
