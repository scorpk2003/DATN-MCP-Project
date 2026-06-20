use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    AppResult, ResourceService,
    models::{
        ApiEnvelope, CandidateRequest, PageQuery, RejectCandidateRequest, ReportGapRequest,
        ResearchTaskRequest,
    },
};

pub async fn list_gaps(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::GapSummary>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_gaps(query).await?)))
}

pub async fn get_gap(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::GapSummary>>> {
    Ok(Json(ApiEnvelope::ok(service.get_gap(id).await?)))
}

pub async fn report_gap(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ReportGapRequest>,
) -> AppResult<Json<ApiEnvelope<serde_json::Value>>> {
    let gap_id = service.report_gap(payload).await?;
    Ok(Json(ApiEnvelope::ok(serde_json::json!({
        "gapId": gap_id,
        "status": "pending"
    }))))
}

pub async fn create_research_task(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<ResearchTaskRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::ResearchTaskSummary>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_research_task(payload).await?,
    )))
}

pub async fn create_candidate(
    State(service): State<Arc<ResourceService>>,
    Json(payload): Json<CandidateRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CandidateSummary>>> {
    Ok(Json(ApiEnvelope::ok(
        service.create_candidate(payload).await?,
    )))
}

pub async fn list_candidates(
    State(service): State<Arc<ResourceService>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<ApiEnvelope<crate::models::Page<crate::models::CandidateSummary>>>> {
    Ok(Json(ApiEnvelope::ok(service.list_candidates(query).await?)))
}

pub async fn approve_candidate(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiEnvelope<crate::models::CandidateSummary>>> {
    Ok(Json(ApiEnvelope::ok(service.approve_candidate(id).await?)))
}

pub async fn reject_candidate(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectCandidateRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::CandidateSummary>>> {
    Ok(Json(ApiEnvelope::ok(
        service.reject_candidate(id, payload.reason).await?,
    )))
}
