use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    AppResult, ResourceService,
    models::{
        ApiEnvelope, CreateResourceRequest, CreateResourceVersionRequest, ManualIngestRequest,
        PageQuery, ResourceChunksQuery, UpdateResourceRequest,
    },
};

pub async fn create_resource(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<CreateResourceRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CreateResourceResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_resource(payload).await?,
    )))
}

pub async fn ingest_manual(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ManualIngestRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::IngestResourceResponse>>> {
    Ok(Json(ApiEnvelope::ok(service.ingest_manual(payload).await?)))
}

pub async fn list_resources(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::ResourceSummary>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_resources(query).await?)))
}

pub async fn get_resource_detail(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::ResourceDetail>>> {
    Ok(Json(ApiEnvelope::ok(
        service.get_resource_detail(id).await?,
    )))
}

pub async fn update_resource(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResourceRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::ResourceDetail>>> {
    Ok(Json(ApiEnvelope::ok(
        service.update_resource(id, payload).await?,
    )))
}

pub async fn list_versions(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<Vec<crate::models::ResourceVersionSummary>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_versions(id).await?)))
}

pub async fn create_resource_version(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateResourceVersionRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::IngestResourceResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_resource_version(id, payload).await?,
    )))
}

pub async fn get_resource_chunks(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ResourceChunksQuery>,
) -> AppResult<Json<ApiEnvelope<Vec<crate::models::ResourceChunk>>>> {
    Ok(Json(ApiEnvelope::ok(
        service
            .get_resource_chunks(id, query.version_id, query.max_chunks)
            .await?,
    )))
}
