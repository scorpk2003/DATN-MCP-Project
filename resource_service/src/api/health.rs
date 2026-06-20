use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{AppResult, ResourceService, models::ApiEnvelope};

pub async fn health() -> AppResult<Json<ApiEnvelope<serde_json::Value>>> {
    Ok(Json(ApiEnvelope::ok(serde_json::json!({"status": "ok"}))))
}

pub async fn ready(
    State(service): State<Arc<ResourceService>>,
) -> AppResult<Json<ApiEnvelope<crate::models::HealthResponse>>> {
    Ok(Json(ApiEnvelope::ok(service.health_check().await?)))
}

pub async fn metrics() -> AppResult<Json<ApiEnvelope<serde_json::Value>>> {
    Ok(Json(ApiEnvelope::ok(serde_json::json!({
        "format": "json",
        "status": "metrics_placeholder"
    }))))
}

pub async fn migrate(
    State(service): State<Arc<ResourceService>>,
) -> AppResult<Json<ApiEnvelope<&'static str>>> {
    service.migrate().await?;
    Ok(Json(ApiEnvelope::ok("migration_completed")))
}
