use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiMeta {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiEnvelope<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
    pub meta: ApiMeta,
}

impl<T> ApiEnvelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: ApiMeta::new(),
        }
    }
}

impl ApiMeta {
    pub fn new() -> Self {
        Self {
            request_id: format!("req_{}", Uuid::new_v4().simple()),
            timestamp: utc_timestamp(),
        }
    }
}

pub fn utc_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{millis}")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationMeta {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<String>,
}

impl PageQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}
