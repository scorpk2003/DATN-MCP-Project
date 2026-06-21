use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    AppResult, ResourceService,
    models::{AdminResourceActionRequest, ApiEnvelope, PageQuery},
};

pub async fn dashboard_summary(
    State(service): State<Arc<ResourceService>>,
) -> AppResult<Json<ApiEnvelope<crate::models::AdminDashboardSummary>>> {
    Ok(Json(ApiEnvelope::ok(
        service.admin_dashboard_summary().await?,
    )))
}

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

pub async fn mark_resource_outdated(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdminResourceActionRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::AdminResourceActionResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.mark_resource_outdated(id, payload).await?,
    )))
}

pub async fn mark_resource_needs_review(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdminResourceActionRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::AdminResourceActionResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.mark_resource_needs_review(id, payload).await?,
    )))
}

pub async fn boost_resource_quality(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdminResourceActionRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::AdminResourceActionResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.boost_resource_quality(id, payload).await?,
    )))
}

pub async fn deboost_resource_quality(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdminResourceActionRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::AdminResourceActionResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.deboost_resource_quality(id, payload).await?,
    )))
}
