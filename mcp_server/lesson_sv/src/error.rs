use serde_json::{Value, json};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LessonErrorCode {
    InvalidInput,
    InsufficientResources,
    PermissionDenied,
    ResourceNotFound,
    DatabaseError,
    DependencyUnavailable,
    EvaluationFailed,
    GenerationFailed,
}

impl LessonErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::InsufficientResources => "INSUFFICIENT_RESOURCES",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ResourceNotFound => "RESOURCE_NOT_FOUND",
            Self::DatabaseError => "DATABASE_ERROR",
            Self::DependencyUnavailable => "DEPENDENCY_UNAVAILABLE",
            Self::EvaluationFailed => "EVALUATION_FAILED",
            Self::GenerationFailed => "GENERATION_FAILED",
        }
    }

    pub fn suggested_action(self) -> &'static str {
        match self {
            Self::InvalidInput => "Fix input and retry.",
            Self::InsufficientResources => {
                "Call Resource MCP to fetch or crawl more resources before retrying."
            }
            Self::PermissionDenied => "Stop execution and verify user context.",
            Self::ResourceNotFound => "Refresh state from Database MCP or Resource MCP.",
            Self::DatabaseError => "Retry if safe, otherwise rollback transaction.",
            Self::DependencyUnavailable => "Retry later or route to degraded mode.",
            Self::EvaluationFailed => "Review assessment context and retry evaluation.",
            Self::GenerationFailed => "Review lesson requirement and evidence, then retry.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LessonToolError {
    pub code: LessonErrorCode,
    pub message: String,
    pub details: Value,
    pub retryable: bool,
}

impl LessonToolError {
    pub fn new(code: LessonErrorCode, message: impl Into<String>, details: Value) -> Self {
        let retryable = matches!(code, LessonErrorCode::DependencyUnavailable);
        Self {
            code,
            message: message.into(),
            details,
            retryable,
        }
    }

    #[allow(dead_code)]
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

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
        "ok": true,
        "success": true,
        "data": data,
        "request_id": meta.request_id,
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

pub fn tool_error_envelope(error: LessonToolError, request_id: Option<&str>) -> Value {
    let meta = match request_id {
        Some(request_id) if !request_id.trim().is_empty() => ResponseMeta {
            request_id: request_id.trim().to_string(),
        },
        _ => ResponseMeta::new(),
    };

    json!({
        "ok": false,
        "success": false,
        "error": {
            "code": error.code.as_str(),
            "message": error.message,
            "details": error.details,
            "retryable": error.retryable,
            "suggested_action": error.code.suggested_action(),
        },
        "request_id": meta.request_id,
        "meta": meta.to_json(),
    })
}

fn current_timestamp() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_error_envelope_has_machine_readable_code() {
        let envelope = tool_error_envelope(
            LessonToolError::new(
                LessonErrorCode::InvalidInput,
                "Invalid request.",
                json!({"field": "userId"}),
            ),
            Some("req-1"),
        );

        assert_eq!(envelope["ok"], false);
        assert_eq!(envelope["error"]["code"], "INVALID_INPUT");
        assert_eq!(envelope["request_id"], "req-1");
        assert!(envelope["error"]["suggested_action"].is_string());
    }
}
