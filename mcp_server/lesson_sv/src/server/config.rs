#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub url: String,
    pub resource_mcp_url: String,
    pub database_mcp_url: String,
    pub internal_token_configured: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let host = dotenv::var("LESSON_MCP_HOST")
            .or_else(|_| dotenv::var("SERVER_LESSON_HOST"))
            .unwrap_or("127.0.0.1".to_string());
        let port = dotenv::var("LESSON_MCP_PORT")
            .or_else(|_| dotenv::var("SERVER_LESSON_PORT"))
            .unwrap_or("3400".to_string());
        let resource_mcp_url =
            dotenv::var("RESOURCE_MCP_URL").unwrap_or("http://127.0.0.1:3300/mcp".to_string());
        let database_mcp_url =
            dotenv::var("DATABASE_MCP_URL").unwrap_or("http://127.0.0.1:3000/mcp".to_string());
        let internal_token_configured = dotenv::var("LESSON_MCP_INTERNAL_TOKEN")
            .ok()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let url = format!("http://{host}:{port}");

        Self {
            host,
            port,
            url,
            resource_mcp_url,
            database_mcp_url,
            internal_token_configured,
        }
    }
}
