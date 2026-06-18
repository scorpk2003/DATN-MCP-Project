use anyhow::Result;
use serde_json::{Map, Value, json};
use tracing::{error, info};

use crate::provider::SchemaProvider;

impl SchemaProvider {
    pub async fn health_check(&mut self) -> Result<Map<String, Value>> {
        let conn = match self.get_connections().await {
            Ok(c) => {
                info!("\tGet connection for health check success!!!");
                c
            }
            Err(e) => {
                error!("\tGet connection for health check failed!!!");
                return Err(e.into());
            }
        };

        let result = conn.query_one("SELECT 1", &[]).await;

        info!("\tHEALTH CHECK");
        let result = match result {
            Ok(res) => {
                info!("\tDatabase health check OK!!!");
                info!("\tResult: {:?}", res);
                res.get::<_, i32>(0)
            }
            Err(e) => {
                error!("\tDatabase health check error: {e}");
                return Err(e.into());
            }
        };

        let (max_size, current_size, idle_size) = match &self.connection_pool {
            Some(info) => {
                let mx = info.status().max_size;
                let idle = info.status().available;
                let curr = info.status().size;
                (mx, curr, idle)
            }
            None => (0, 0, 0),
        };
        let pool_info = json!({
            "max_size": max_size,
            "current_size": current_size,
            "idle_size": idle_size,
        });

        let mut health = Map::new();
        health.insert("status".to_string(), Value::String("healthy".to_string()));
        health.insert("database_response".to_string(), Value::Bool(result == 1));
        health.insert("pool_info".to_string(), pool_info);

        Ok(health)
    }
}

mod test {

    #[tokio::test]
    async fn test_health_check() {
        use crate::provider::SchemaProvider;
        dotenv::from_path("../../.env").ok();
        let mut provider = SchemaProvider::default();
        let res = provider.health_check().await;
        println!("{:?}", res);
    }
}
