use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::{error, warn};

use crate::models::{ApiEnvelope, ApiErrorBody, ApiMeta};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication is required")]
    Unauthorized,
    #[error("permission denied")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("resource not found")]
    ResourceNotFound,
    #[error("source not found")]
    SourceNotFound,
    #[error("crawl job not found")]
    CrawlJobNotFound,
    #[error("gap not found")]
    GapNotFound,
    #[error("database pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    #[error("configuration error: {0}")]
    PoolBuild(#[from] deadpool_postgres::CreatePoolError),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::Validation(_) | AppError::BadRequest(_) => "VALIDATION_ERROR",
            AppError::ResourceNotFound => "RESOURCE_NOT_FOUND",
            AppError::SourceNotFound => "SOURCE_NOT_FOUND",
            AppError::CrawlJobNotFound => "CRAWL_JOB_NOT_FOUND",
            AppError::GapNotFound => "RESOURCE_GAP_NOT_FOUND",
            AppError::Pool(_)
            | AppError::Database(_)
            | AppError::PoolBuild(_)
            | AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ResourceNotFound
            | AppError::SourceNotFound
            | AppError::CrawlJobNotFound
            | AppError::GapNotFound => StatusCode::NOT_FOUND,
            AppError::Pool(_)
            | AppError::Database(_)
            | AppError::PoolBuild(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            error!("request failed with server error: {}", self);
        } else {
            warn!("request failed with client error: {}", self);
        }

        let body: ApiEnvelope<serde_json::Value> = ApiEnvelope {
            success: false,
            data: None,
            error: Some(ApiErrorBody {
                code: self.code().to_string(),
                message: self.to_string(),
                details: json!({}),
            }),
            meta: ApiMeta::new(),
        };

        (status, Json(body)).into_response()
    }
}
