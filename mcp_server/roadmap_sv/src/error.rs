use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ResponseMeta {
    pub request_id: String,
}

impl ResponseMeta {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "requestId": self.request_id,
            "timestamp": current_timestamp(),
        })
    }
}

pub fn success_envelope(data: Value) -> Value {
    let meta = ResponseMeta::new();
    json!({
        "success": true,
        "data": data,
        "meta": meta.to_json(),
    })
}

pub fn error_envelope(code: &str, message: &str, details: Value, retryable: bool) -> Value {
    let meta = ResponseMeta::new();
    json!({
        "success": false,
        "error": {
            "code": code,
            "message": message,
            "details": details,
            "retryable": retryable,
        },
        "meta": meta.to_json(),
    })
}

fn current_timestamp() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}
