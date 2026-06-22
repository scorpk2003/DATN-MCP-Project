#![allow(dead_code)]

use std::{fmt, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct ResourceContractClient {
    endpoint: HttpEndpoint,
    token: Option<String>,
    timeout: Duration,
}

#[derive(Debug, Clone)]
struct HttpEndpoint {
    host: String,
    port: u16,
}

#[derive(Debug)]
pub enum ResourceClientError {
    InvalidBaseUrl,
    Unavailable,
    Timeout,
    InvalidResponse,
    ForbiddenOperation(String),
    Status(u16),
    Api { code: String, message: String },
}

impl ResourceContractClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, ResourceClientError> {
        Ok(Self {
            endpoint: parse_base_url(&base_url.into())?,
            token: None,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token.and_then(|value| {
            let value = value.trim().to_string();
            (!value.is_empty()).then_some(value)
        });
        self
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn get_integration_contract(&self) -> Result<Value, ResourceClientError> {
        Ok(json!({
            "client": "roadmap_resource_contract_client",
            "safeOperations": safe_operations(),
            "forbiddenOperations": [
                "crawl_any_url",
                "approve_research_candidate",
                "reject_research_candidate",
                "insert_resource",
                "delete_resource",
                "update_embedding",
                "boost_resource",
                "deboost_resource"
            ],
            "transport": "restricted_resource_service_http"
        }))
    }

    pub async fn get_topic_coverage(
        &self,
        topic: &str,
        level: Option<&str>,
        required_types: Vec<String>,
    ) -> Result<Value, ResourceClientError> {
        self.post_safe(
            "/coverage/topic",
            json!({
                "topic": topic,
                "level": level,
                "requiredTypes": required_types,
            }),
        )
        .await
    }

    pub async fn recommend_resources_for_topic(
        &self,
        topic: &str,
        level: Option<&str>,
        goal: Option<&str>,
        required_types: Vec<String>,
        max_resources: u32,
    ) -> Result<Value, ResourceClientError> {
        self.post_safe(
            "/recommend/resources",
            json!({
                "topic": topic,
                "level": level,
                "goal": goal,
                "requiredTypes": required_types,
                "maxResources": max_resources.min(15),
                "includeChunks": false,
            }),
        )
        .await
    }

    pub async fn search_resources(
        &self,
        query: &str,
        limit: u32,
        filters: Value,
    ) -> Result<Value, ResourceClientError> {
        self.post_safe(
            "/search/resources",
            json!({
                "query": query,
                "limit": limit.min(20),
                "maxChunksPerResource": 0,
                "includeCoverage": true,
                "createGapOnLowConfidence": false,
                "filters": filters,
            }),
        )
        .await
    }

    pub async fn report_resource_gap(
        &self,
        topic: &str,
        level: Option<&str>,
        missing_types: Vec<String>,
        reason: &str,
    ) -> Result<Value, ResourceClientError> {
        self.post_safe(
            "/gaps",
            json!({
                "topic": topic,
                "level": level,
                "missingTypes": missing_types,
                "reason": reason,
            }),
        )
        .await
    }

    pub async fn request_research_for_topic(
        &self,
        topic: &str,
        target_resource_types: Vec<String>,
        priority: i32,
    ) -> Result<Value, ResourceClientError> {
        self.post_safe(
            "/research/tasks",
            json!({
                "topic": topic,
                "language": null,
                "priority": priority.clamp(1, 5),
                "targetResourceTypes": target_resource_types,
            }),
        )
        .await
    }

    async fn post_safe(&self, path: &str, body: Value) -> Result<Value, ResourceClientError> {
        if !is_safe_path(path) {
            return Err(ResourceClientError::ForbiddenOperation(path.to_string()));
        }

        self.request("POST", path, Some(body)).await
    }

    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ResourceClientError> {
        let body = body.map(|value| value.to_string()).unwrap_or_default();
        let request = build_request(
            method,
            path,
            &self.endpoint.host,
            self.token.as_deref(),
            &body,
        );

        let response = timeout(self.timeout, async {
            let mut stream = TcpStream::connect((&self.endpoint.host[..], self.endpoint.port))
                .await
                .map_err(|_| ResourceClientError::Unavailable)?;
            stream
                .write_all(request.as_bytes())
                .await
                .map_err(|_| ResourceClientError::Unavailable)?;

            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .await
                .map_err(|_| ResourceClientError::Unavailable)?;
            Ok::<Vec<u8>, ResourceClientError>(response)
        })
        .await
        .map_err(|_| ResourceClientError::Timeout)??;

        parse_response(&response)
    }

    pub fn normalized_error(error: &ResourceClientError) -> Value {
        error.normalized()
    }
}

pub fn safe_operations() -> Vec<&'static str> {
    vec![
        "get_integration_contract",
        "get_topic_coverage",
        "recommend_resources_for_topic",
        "search_resources",
        "report_resource_gap",
        "request_research_for_topic",
    ]
}

fn is_safe_path(path: &str) -> bool {
    matches!(
        path,
        "/coverage/topic"
            | "/recommend/resources"
            | "/search/resources"
            | "/gaps"
            | "/research/tasks"
    )
}

fn build_request(method: &str, path: &str, host: &str, token: Option<&str>, body: &str) -> String {
    let auth_header = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\nContent-Type: application/json\r\n{auth_header}Connection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

impl ResourceClientError {
    pub fn normalized(&self) -> Value {
        let (code, message, retryable) = match self {
            ResourceClientError::InvalidBaseUrl => (
                "RESOURCE_CLIENT_CONFIG_ERROR",
                "Resource Service base URL is invalid.",
                false,
            ),
            ResourceClientError::Unavailable => (
                "RESOURCE_MCP_UNAVAILABLE",
                "Resource Service is temporarily unavailable.",
                true,
            ),
            ResourceClientError::Timeout => (
                "RESOURCE_MCP_TIMEOUT",
                "Resource Service call timed out.",
                true,
            ),
            ResourceClientError::InvalidResponse => (
                "RESOURCE_MCP_INVALID_RESPONSE",
                "Resource Service returned an invalid response.",
                true,
            ),
            ResourceClientError::ForbiddenOperation(_) => (
                "RESOURCE_FORBIDDEN_OPERATION",
                "Roadmap MCP attempted to call a forbidden Resource operation.",
                false,
            ),
            ResourceClientError::Status(_) => (
                "RESOURCE_MCP_ERROR",
                "Resource Service returned an unsuccessful status.",
                true,
            ),
            ResourceClientError::Api { code, message } => {
                return json!({
                    "success": false,
                    "error": {
                        "code": code,
                        "message": message,
                        "details": null,
                        "retryable": false
                    }
                });
            }
        };

        json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
                "details": null,
                "retryable": retryable
            }
        })
    }
}

impl fmt::Display for ResourceClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceClientError::InvalidBaseUrl => write!(f, "invalid resource service base url"),
            ResourceClientError::Unavailable => write!(f, "resource service unavailable"),
            ResourceClientError::Timeout => write!(f, "resource service timeout"),
            ResourceClientError::InvalidResponse => write!(f, "invalid resource service response"),
            ResourceClientError::ForbiddenOperation(path) => {
                write!(f, "forbidden resource operation: {path}")
            }
            ResourceClientError::Status(status) => write!(f, "resource service status {status}"),
            ResourceClientError::Api { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

fn parse_base_url(value: &str) -> Result<HttpEndpoint, ResourceClientError> {
    let rest = value
        .trim()
        .strip_prefix("http://")
        .ok_or(ResourceClientError::InvalidBaseUrl)?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| ResourceClientError::InvalidBaseUrl)?;
            (host.to_string(), port)
        }
        None => (authority.to_string(), 80),
    };

    if host.is_empty() {
        return Err(ResourceClientError::InvalidBaseUrl);
    }

    Ok(HttpEndpoint { host, port })
}

fn parse_response(response: &[u8]) -> Result<Value, ResourceClientError> {
    let split_at = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(ResourceClientError::InvalidResponse)?;
    let (head, body) = response.split_at(split_at + 4);
    let head = std::str::from_utf8(head).map_err(|_| ResourceClientError::InvalidResponse)?;
    let status = parse_status(head)?;
    if !(200..300).contains(&status) {
        return Err(ResourceClientError::Status(status));
    }

    let value: Value =
        serde_json::from_slice(body).map_err(|_| ResourceClientError::InvalidResponse)?;
    unwrap_envelope(value)
}

fn parse_status(head: &str) -> Result<u16, ResourceClientError> {
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or(ResourceClientError::InvalidResponse)?;
    Ok(status)
}

fn unwrap_envelope(value: Value) -> Result<Value, ResourceClientError> {
    match value.get("success").and_then(Value::as_bool) {
        Some(true) => Ok(value.get("data").cloned().unwrap_or(Value::Null)),
        Some(false) => {
            let error = value.get("error").cloned().unwrap_or(Value::Null);
            let code = error
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("RESOURCE_MCP_ERROR")
                .to_string();
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Resource Service rejected the request.")
                .to_string();
            Err(ResourceClientError::Api { code, message })
        }
        None => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_resource_service_url() {
        let endpoint = parse_base_url("http://127.0.0.1:3200").unwrap();

        assert_eq!(endpoint.host, "127.0.0.1");
        assert_eq!(endpoint.port, 3200);
    }

    #[test]
    fn allows_only_resource_contract_paths() {
        assert!(is_safe_path("/coverage/topic"));
        assert!(is_safe_path("/recommend/resources"));
        assert!(is_safe_path("/search/resources"));
        assert!(is_safe_path("/gaps"));
        assert!(is_safe_path("/research/tasks"));
        assert!(!is_safe_path("/admin/resources/approve"));
        assert!(!is_safe_path("/crawl/jobs"));
        assert!(!is_safe_path("/resources"));
    }

    #[test]
    fn unwraps_resource_service_success_envelope() {
        let body = json!({"success": true, "data": {"coverageStatus": "good"}}).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        let value = parse_response(response.as_bytes()).unwrap();

        assert_eq!(value["coverageStatus"], "good");
    }

    #[test]
    fn maps_error_to_normalized_shape() {
        let value = ResourceClientError::Timeout.normalized();

        assert_eq!(value["success"], false);
        assert_eq!(value["error"]["code"], "RESOURCE_MCP_TIMEOUT");
        assert_eq!(value["error"]["retryable"], true);
    }
}
