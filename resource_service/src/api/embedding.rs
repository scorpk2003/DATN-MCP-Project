use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    AppResult, ResourceService,
    models::{ApiEnvelope, EmbeddingModelRequest, PendingEmbeddingChunksQuery},
};

pub async fn create_embedding_model(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<EmbeddingModelRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::EmbeddingModelSummary>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_embedding_model(payload).await?,
    )))
}

pub async fn list_embedding_models(
    State(service): State<Arc<ResourceService>>,
) -> AppResult<Json<ApiEnvelope<Vec<crate::models::EmbeddingModelSummary>>>> {
    Ok(Json(ApiEnvelope::ok(
        service.list_embedding_models().await?,
    )))
}

pub async fn list_pending_embedding_chunks(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PendingEmbeddingChunksQuery>,
) -> AppResult<Json<ApiEnvelope<Vec<crate::models::PendingEmbeddingChunk>>>> {
    Ok(Json(ApiEnvelope::ok(
        service.list_pending_embedding_chunks(query).await?,
    )))
}
