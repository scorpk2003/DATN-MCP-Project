#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let host = dotenv::var("SERVER_ROADMAP_HOST").unwrap_or("127.0.0.1".to_string());
        let port = dotenv::var("SERVER_ROADMAP_PORT").unwrap_or("3100".to_string());
        let url = format!("http://{host}:{port}");
        Self { host, port, url }
    }
}

mod test {
    
    #[tokio::test]
    async fn test_default() {
        use crate::server::config::ServerConfig;
        dotenv::from_path("../../.env").ok();
        let config = ServerConfig::default();
        println!("{:?}", config);
    }
}