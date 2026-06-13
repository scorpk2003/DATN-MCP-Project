use std::env;

#[derive(Clone)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: usize,
    pub description: String,
    pub url: String,
}

impl ServerConfig {
    pub fn new(server: &str) -> Self {
        let name = env::var(format!("SERVER_{}_NAME", server.to_uppercase())).unwrap_or_else(|_| server.to_string());
        let host = env::var(format!("SERVER_{}_HOST", server.to_uppercase())).unwrap_or_else(|_| "0.0.0.1".to_string());
        let port = env::var(format!("SERVER_{}_PORT", server.to_uppercase())).unwrap_or_else(|_| "3001".to_string()).parse::<usize>().unwrap_or(3001);
        let description = env::var(format!("SERVER_{}_DESCRIPTION", server.to_uppercase())).unwrap_or_else(|_| "no description".to_string());
        let url = format!("http://{}:{}", host, port);
        Self { name, host, port, description, url }
    }
}