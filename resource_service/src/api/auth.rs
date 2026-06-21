use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, header},
    middleware::Next,
    response::Response,
};

use crate::{AppError, AppResult, ResourceService};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequiredRole {
    Admin,
    Worker,
}

pub async fn internal_auth(
    State(service): State<Arc<ResourceService>>,
    request: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let Some(required_role) = required_role(request.uri().path()) else {
        return Ok(next.run(request).await);
    };

    let configured_token = match required_role {
        RequiredRole::Admin => service.config.auth.admin_token.as_deref(),
        RequiredRole::Worker => service.config.auth.worker_token.as_deref(),
    };

    let Some(configured_token) = configured_token else {
        return Ok(next.run(request).await);
    };

    let Some(provided_token) = extract_token(&request) else {
        return Err(AppError::Unauthorized);
    };

    if constant_time_eq(provided_token.as_bytes(), configured_token.as_bytes()) {
        Ok(next.run(request).await)
    } else {
        Err(AppError::Forbidden)
    }
}

fn required_role(path: &str) -> Option<RequiredRole> {
    if path == "/admin" || path.starts_with("/admin/") {
        return Some(RequiredRole::Admin);
    }
    if path == "/worker" || path.starts_with("/worker/") {
        return Some(RequiredRole::Worker);
    }
    if path == "/admin/migrate" {
        return Some(RequiredRole::Admin);
    }
    if path.starts_with("/research/candidates/")
        && (path.ends_with("/approve") || path.ends_with("/reject"))
    {
        return Some(RequiredRole::Admin);
    }
    None
}

fn extract_token(request: &Request<Body>) -> Option<String> {
    if let Some(value) = request.headers().get(header::AUTHORIZATION) {
        let value = value.to_str().ok()?.trim();
        if let Some(token) = value.strip_prefix("Bearer ") {
            return non_empty(token);
        }
    }

    request
        .headers()
        .get("x-resource-service-token")
        .and_then(|value| value.to_str().ok())
        .and_then(non_empty)
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use axum::http::{Request, header};

    use super::*;

    #[test]
    fn classifies_sensitive_paths() {
        assert_eq!(required_role("/admin/dashboard"), Some(RequiredRole::Admin));
        assert_eq!(
            required_role("/research/candidates/abc/approve"),
            Some(RequiredRole::Admin)
        );
        assert_eq!(
            required_role("/worker/crawl/jobs/claim"),
            Some(RequiredRole::Worker)
        );
        assert_eq!(required_role("/search/resources"), None);
    }

    #[test]
    fn extracts_bearer_or_internal_token_header() {
        let request = Request::builder()
            .header(header::AUTHORIZATION, "Bearer admin-secret")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_token(&request).as_deref(), Some("admin-secret"));

        let request = Request::builder()
            .header("x-resource-service-token", " worker-secret ")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_token(&request).as_deref(), Some("worker-secret"));
    }

    #[test]
    fn compares_tokens_without_early_success_on_prefix() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"same-prefix"));
        assert!(!constant_time_eq(b"same", b"xxxx"));
    }
}
