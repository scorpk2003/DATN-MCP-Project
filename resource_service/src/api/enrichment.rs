use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    AppResult, ResourceService,
    models::{ApiEnvelope, EnrichResourceRequest},
};

pub async fn enrich_resource(
    State(service): State<Arc<ResourceService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<EnrichResourceRequest>,
) -> AppResult<Json<ApiEnvelope<crate::models::EnrichResourceResponse>>> {
    Ok(Json(ApiEnvelope::ok(
        service.enrich_resource(id, payload).await?,
    )))
}
