#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub url: String,
    pub resource_service_base_url: String,
    pub resource_service_mcp_token: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let host = dotenv::var("SERVER_RESOURCE_MCP_HOST").unwrap_or("127.0.0.1".to_string());
        let port = dotenv::var("SERVER_RESOURCE_MCP_PORT").unwrap_or("3300".to_string());
        let resource_service_base_url =
            dotenv::var("RESOURCE_SERVICE_BASE_URL").unwrap_or("http://127.0.0.1:3200".to_string());
        let resource_service_mcp_token = dotenv::var("RESOURCE_SERVICE_MCP_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let url = format!("http://{host}:{port}");

        Self {
            host,
            port,
            url,
            resource_service_base_url,
            resource_service_mcp_token,
        }
    }
}
