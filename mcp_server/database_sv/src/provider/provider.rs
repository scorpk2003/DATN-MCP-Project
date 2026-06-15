use std::{collections::HashMap, time::Duration};

use deadpool::{Runtime::Tokio1, managed::QueueMode::{self, Fifo}};
use deadpool_postgres::{Config, Object, Pool, PoolConfig, PoolError, Timeouts, tokio_postgres::NoTls};
use serde_json::Value;
use tracing::info;

use crate::server::DatabaseConfig;


#[derive(Debug, Clone)]
pub struct SchemaProvider {
    pub connection_pool: Option<Pool>,
    pub pg_config: HashMap<String, Value>,
    pub db: DatabaseConfig,
}

impl Default for SchemaProvider {
    fn default() -> Self {
        let connection_pool = None;
        let db = DatabaseConfig::default();
        let pg_config = DatabaseConfig::async_params(&db);

        Self { connection_pool, pg_config, db }
    }
}

impl SchemaProvider {
    pub async fn create_pool(&mut self) {
        if self.connection_pool.is_none() {
            let mut config = Config::new();

            config.host = Some(self.pg_config.get("host").unwrap().as_str().unwrap().to_string());
            config.port = Some(self.pg_config.get("port").unwrap().as_u64().unwrap() as u16);
            config.dbname = Some(self.pg_config.get("db").unwrap().as_str().unwrap().to_string());
            config.user = Some(self.pg_config.get("user").unwrap().as_str().unwrap().to_string());
            config.password = Some(self.pg_config.get("password").unwrap().as_str().unwrap().to_string());
            config.connect_timeout = Some(Duration::from_secs(self.pg_config.get("command_timeout").unwrap().as_u64().unwrap()));

            config.pool = Some(PoolConfig {
                max_size: self.db.max_conn as usize,
                timeouts: Timeouts {
                    wait: Some(Duration::from_secs(30)),
                    create: Some(Duration::from_secs(30)),
                    recycle: Some(Duration::from_secs(30)),
                },
                queue_mode: QueueMode::Fifo,
            });

            let option = self.pg_config.get("server_settings").unwrap().as_object().unwrap();
            let mut opt = String::new();
            for (key, val) in option {
                let value = match val {
                    serde_json::Value::String(s) => s.clone(),
                    _ => val.to_string(),
                };
                opt.push_str(&format!("-c {}={} ", key, value));
            }

            config.options = Some(opt.trim().to_string());
            println!("{:?}", config);
            self.connection_pool = Some(config.create_pool(Some(Tokio1), NoTls).unwrap());

            info!("\tDatabase Connection pool created successfully!!!");
        }
    }

    pub async fn close_pool(&mut self) {
        if self.connection_pool.is_some() {
            self.connection_pool = None;
        }
        info!("\tDatabase connection pool closed successfully!!!");
    }

    pub async fn get_connections(&mut self) -> Result<Object, PoolError> {
        if self.connection_pool.is_none() {
            self.create_pool().await;
        }

        let conn = self.connection_pool.clone().unwrap().status();
        info!("\tConnection Status:\n\t\t{:?}", conn);
        self.connection_pool.clone().unwrap().get().await
    }
}

mod test {
    #[tokio::test]
    async fn test_get_connections() {
        use super::*;
        dotenv::from_path("../../.env").ok();
        let mut provider = SchemaProvider::default();
        match provider.get_connections().await {
            Ok(p) => {
                println!("\tGet Connection success:\n\t{:?}", p);
            },
            Err(e) => {
                println!("\tGet Connection failed:\n\t{:?}", e);
            }
        }
    }
}