#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub url: String,
    pub resource_mcp_url: String,
    pub resource_service_url: String,
    pub resource_service_token: Option<String>,
    pub internal_token_configured: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let host = dotenv::var("ROADMAP_MCP_HOST")
            .or_else(|_| dotenv::var("SERVER_ROADMAP_HOST"))
            .unwrap_or("127.0.0.1".to_string());
        let port = dotenv::var("ROADMAP_MCP_PORT")
            .or_else(|_| dotenv::var("SERVER_ROADMAP_PORT"))
            .unwrap_or("3100".to_string());
        let resource_mcp_url =
            dotenv::var("RESOURCE_MCP_URL").unwrap_or("http://127.0.0.1:3300/mcp".to_string());
        let resource_service_url = dotenv::var("RESOURCE_SERVICE_URL")
            .or_else(|_| dotenv::var("RESOURCE_SERVICE_BASE_URL"))
            .unwrap_or("http://127.0.0.1:3200".to_string());
        let resource_service_token = dotenv::var("RESOURCE_SERVICE_INTERNAL_TOKEN")
            .or_else(|_| dotenv::var("RESOURCE_SERVICE_MCP_TOKEN"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let internal_token_configured = dotenv::var("ROADMAP_MCP_INTERNAL_TOKEN")
            .ok()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let url = format!("http://{host}:{port}");

        Self {
            host,
            port,
            url,
            resource_mcp_url,
            resource_service_url,
            resource_service_token,
            internal_token_configured,
        }
    }
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn test_default() {
        use crate::server::config::ServerConfig;
        dotenv::from_path("../../.env").ok();
        let config = ServerConfig::default();
        println!("{:?}", config);
    }
}
