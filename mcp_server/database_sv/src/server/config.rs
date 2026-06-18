use std::{collections::HashMap, env};

use serde_json::{Value};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: usize,
    pub db: String,
    pub user: String,
    pub pass: String,
    pub max_conn: i32,
    pub min_conn: i32,
    pub timeout: i32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let host = env::var("DB_HOST")
            .unwrap_or("localhost01".to_string());
        let port = env::var("DB_PORT")
            .unwrap_or("5433".to_string()).parse::<usize>().unwrap();
        let db = env::var("DB_NAME")
            .unwrap_or("DATABASE_SERVER".to_string());
        let user = env::var("DB_USER")
            .unwrap_or("admin".to_string());
        let pass = env::var("DB_PASS")
            .unwrap_or("admin".to_string());
        let max_conn = env::var("POSTGRES_MAX_CONNECTIONS")
            .unwrap_or("10".to_string()).parse::<i32>().unwrap();
        let min_conn = env::var("POSTGRES_MIN_CONNECTIONS")
            .unwrap_or("1".to_string()).parse::<i32>().unwrap();
        let timeout = env::var("CMD_TIMEOUT")
            .unwrap_or("30".to_string()).parse::<i32>().unwrap();
        Self { host, port, db, user, pass, max_conn, min_conn, timeout }
    }
}

impl DatabaseConfig {
    pub fn async_params(&self) -> HashMap<String, Value> {
        let mut params = HashMap::new();
        

        params.insert("host".to_string(), Value::String(self.host.clone()));
        params.insert("port".to_string(), Value::Number(self.port.into()));
        params.insert("db".to_string(), Value::String(self.db.clone()));
        params.insert("user".to_string(), Value::String(self.user.clone()));
        params.insert("password".to_string(), Value::String(self.pass.clone()));
        params.insert("command_timeout".to_string(), Value::Number(self.timeout.into()));

        // LOCAL
        // let db_timeout = self.timeout*1000;
        // let server_settings = json!({
        //     "application_name": "self-learn",
        //     "jit": "off",
        //     "work_mem": "4MB",
        //     "statement_timeout": &db_timeout.to_string(),
        // });
        // params.insert("server_settings".to_string(), server_settings);

        params
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: usize,
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let host = env::var("SERVER_DATABASE_HOST").unwrap_or("0.0.0.1".to_string());
        let port = env::var("SERVER_DATABASE_PORT").unwrap_or("3000".to_string()).parse::<usize>().unwrap();
        let url = format!("http://{}:{}", host, port);
        Self { host, port, url }
    }
}

mod test {
    
    #[tokio::test]
    async fn test_db() {
        use crate::server::DatabaseConfig;
        dotenv::from_path("../../.env").ok();
        let db_cfg = DatabaseConfig::default();
        println!("Database Config:\n\t{:?}", db_cfg);
    }

    #[tokio::test]
    async fn test_server() {
        use crate::ServerConfig;
        dotenv::from_path("../../.env").ok();
        let sv_cfg = ServerConfig::default();
        println!("Server Config:\n\t{:?}", sv_cfg);
    }

    #[tokio::test]
    async fn test_async_params() {
        use crate::DatabaseConfig;
        dotenv::from_path("../../.env").ok();
        let db = DatabaseConfig::default();
        let params = db.async_params();
        println!("Async Params:\n\t{:?}", params);
    }
}