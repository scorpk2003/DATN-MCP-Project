use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    AppResult, ResourceService,
    models::{ApiEnvelope, RecommendRequest, SearchRequest, TopicCoverageRequest},
};

pub async fn search_chunks(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<SearchRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::SearchResponse>>> {
    Ok(Json(ApiEnvelope::ok(service.search_chunks(payload).await?)))
}

pub async fn search_resources(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<SearchRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::SearchResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.search_resources(payload).await?,
    )))
}

pub async fn recommend_resources(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<RecommendRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::RecommendResponse>>> {
    Ok(Json(ApiEnvelope::ok(service.recommend(payload).await?)))
}

pub async fn topic_coverage(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<TopicCoverageRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::TopicCoverageResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.topic_coverage(payload).await?,
    )))
}
