use serde_json::{Map, Value, json};

use crate::{AppResult, models::HealthResponse};

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn health_check(&self) -> AppResult<HealthResponse> {
        let client = self.pool.get().await?;
        let database_ok: bool = client.query_one("SELECT 1 = 1", &[]).await?.get(0);
        let vector_ok = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')",
                &[],
            )
            .await
            .map(|row| row.get::<_, bool>(0))
            .unwrap_or(false);
        let table_ok: bool = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'resource_service'
                      AND table_name = 'resources'
                )",
                &[],
            )
            .await?
            .get(0);

        let status = self.pool.status();
        let mut checks = Map::new();
        checks.insert("postgres".to_string(), Value::Bool(database_ok));
        checks.insert("vectorExtension".to_string(), Value::Bool(vector_ok));
        checks.insert("requiredTables".to_string(), Value::Bool(table_ok));
        checks.insert(
            "pool".to_string(),
            json!({
                "maxSize": status.max_size,
                "currentSize": status.size,
                "idleSize": status.available,
            }),
        );

        Ok(HealthResponse {
            status: if database_ok && table_ok {
                "ready".to_string()
            } else {
                "degraded".to_string()
            },
            checks,
        })
    }
}
