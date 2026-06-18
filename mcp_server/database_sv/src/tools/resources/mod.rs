use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateResourceParam, IdParam, ListResourcesParam},
};

#[derive(Debug, Clone)]
pub struct ResourceTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl ResourceTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_resource(&self, param: CreateResourceParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn
            .query_one(
                "INSERT INTO learning_resources (task_id, resource_type, title, url, description)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, task_id, resource_type, title, url, description",
                &[
                    &param.task_id,
                    &param.resource_type,
                    &param.title,
                    &param.url,
                    &param.description,
                ],
            )
            .await?;
        Ok(common::resource(row))
    }

    pub async fn delete_resource(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tDelete learning resource query start: id={}", param.id);
        let count = conn
            .execute("DELETE FROM learning_resources WHERE id = $1", &[&param.id])
            .await?;
        info!("\tDelete learning resource query completed: affected_rows={count}");
        Ok(common::deleted(count))
    }

    pub async fn list_resources(&self, param: ListResourcesParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let rows = conn
            .query(
                "SELECT id, task_id, resource_type, title, url, description
             FROM learning_resources WHERE task_id = $1 ORDER BY title NULLS LAST",
                &[&param.task_id],
            )
            .await?;
        Ok(Value::Array(
            rows.into_iter().map(common::resource).collect(),
        ))
    }
}
