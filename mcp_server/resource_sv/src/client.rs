use std::fmt;

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Clone)]
pub struct ResourceApiClient {
    endpoint: HttpEndpoint,
    token: Option<String>,
}

#[derive(Debug, Clone)]
struct HttpEndpoint {
    host: String,
    port: u16,
}

#[derive(Debug)]
pub enum ResourceApiError {
    InvalidBaseUrl,
    Unavailable,
    InvalidResponse,
    Status(u16),
    Api { code: String, message: String },
}

impl ResourceApiClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, ResourceApiError> {
        Ok(Self {
            endpoint: parse_base_url(&base_url.into())?,
            token: None,
        })
    }

    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token.and_then(|value| {
            let value = value.trim().to_string();
            (!value.is_empty()).then_some(value)
        });
        self
    }

    pub async fn get(&self, path: &str) -> Result<Value, ResourceApiError> {
        self.request("GET", path, None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value, ResourceApiError> {
        self.request("POST", path, Some(body)).await
    }

    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ResourceApiError> {
        let body = body.map(|value| value.to_string()).unwrap_or_default();
        let request = build_request(
            method,
            path,
            &self.endpoint.host,
            self.token.as_deref(),
            &body,
        );

        let mut stream = TcpStream::connect((&self.endpoint.host[..], self.endpoint.port))
            .await
            .map_err(|_| ResourceApiError::Unavailable)?;
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|_| ResourceApiError::Unavailable)?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|_| ResourceApiError::Unavailable)?;

        parse_response(&response)
    }
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

impl ResourceApiError {
    pub fn normalized(&self) -> Value {
        let (code, message) = match self {
            ResourceApiError::InvalidBaseUrl => (
                "RESOURCE_API_CONFIG_ERROR",
                "Resource Service base URL is invalid.",
            ),
            ResourceApiError::Unavailable => (
                "RESOURCE_API_UNAVAILABLE",
                "Resource Service is temporarily unavailable.",
            ),
            ResourceApiError::InvalidResponse => (
                "RESOURCE_API_INVALID_RESPONSE",
                "Resource Service returned an invalid response.",
            ),
            ResourceApiError::Status(_) => (
                "RESOURCE_API_ERROR",
                "Resource Service returned an unsuccessful status.",
            ),
            ResourceApiError::Api { code, message } => {
                return json!({"ok": false, "error": {"code": code, "message": message}});
            }
        };

        json!({"ok": false, "error": {"code": code, "message": message}})
    }
}

impl fmt::Display for ResourceApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceApiError::InvalidBaseUrl => write!(f, "invalid resource api base url"),
            ResourceApiError::Unavailable => write!(f, "resource api unavailable"),
            ResourceApiError::InvalidResponse => write!(f, "invalid resource api response"),
            ResourceApiError::Status(status) => write!(f, "resource api status {status}"),
            ResourceApiError::Api { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

fn parse_base_url(value: &str) -> Result<HttpEndpoint, ResourceApiError> {
    let rest = value
        .trim()
        .strip_prefix("http://")
        .ok_or(ResourceApiError::InvalidBaseUrl)?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| ResourceApiError::InvalidBaseUrl)?;
            (host.to_string(), port)
        }
        None => (authority.to_string(), 80),
    };

    if host.is_empty() {
        return Err(ResourceApiError::InvalidBaseUrl);
    }

    Ok(HttpEndpoint { host, port })
}

fn parse_response(response: &[u8]) -> Result<Value, ResourceApiError> {
    let split_at = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(ResourceApiError::InvalidResponse)?;
    let (head, body) = response.split_at(split_at + 4);
    let head = std::str::from_utf8(head).map_err(|_| ResourceApiError::InvalidResponse)?;
    let status = parse_status(head)?;
    if !(200..300).contains(&status) {
        return Err(ResourceApiError::Status(status));
    }

    let value: Value =
        serde_json::from_slice(body).map_err(|_| ResourceApiError::InvalidResponse)?;
    unwrap_envelope(value)
}

fn parse_status(head: &str) -> Result<u16, ResourceApiError> {
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or(ResourceApiError::InvalidResponse)?;
    Ok(status)
}

fn unwrap_envelope(value: Value) -> Result<Value, ResourceApiError> {
    match value.get("success").and_then(Value::as_bool) {
        Some(true) => Ok(value.get("data").cloned().unwrap_or(Value::Null)),
        Some(false) => {
            let error = value.get("error").cloned().unwrap_or(Value::Null);
            let code = error
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("RESOURCE_API_ERROR")
                .to_string();
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Resource Service rejected the request.")
                .to_string();
            Err(ResourceApiError::Api { code, message })
        }
        None => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_http_base_url() {
        let endpoint = parse_base_url("http://127.0.0.1:3200").unwrap();
        assert_eq!(endpoint.host, "127.0.0.1");
        assert_eq!(endpoint.port, 3200);
    }

    #[test]
    fn builds_authorized_request_without_logging_token_elsewhere() {
        let request = build_request(
            "POST",
            "/search/resources",
            "127.0.0.1",
            Some("mcp-secret"),
            "{}",
        );

        assert!(request.contains("Authorization: Bearer mcp-secret\r\n"));
        assert!(request.contains("Content-Length: 2\r\n"));
    }

    #[test]
    fn unwraps_mock_resource_api_envelope() {
        let body = json!({"success": true, "data": {"status": "good"}}).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let value = parse_response(response.as_bytes()).unwrap();

        assert_eq!(value["status"], "good");
    }
}
