use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateRoadmapParam, IdParam, ListProjectRoadmapParam},
};

#[derive(Debug, Clone)]
pub struct RoadmapTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl RoadmapTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_roadmap(&self, param: CreateRoadmapParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!(
            "\tCreate roadmap query start: project_id={}, version={}",
            param.project_id, param.version
        );
        let row = conn
            .query_one(
                "INSERT INTO roadmaps (project_id, version, title, generated_by)
             VALUES ($1, $2, $3, $4)
             RETURNING id, project_id, version, title, generated_by, created_at::text",
                &[
                    &param.project_id,
                    &param.version,
                    &param.title,
                    &param.generated_by,
                ],
            )
            .await?;
        info!("\tCreate roadmap query completed");
        Ok(common::roadmap(row))
    }

    pub async fn get_roadmap(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_opt(
            "SELECT id, project_id, version, title, generated_by, created_at::text FROM roadmaps WHERE id = $1",
            &[&param.id],
        ).await?;
        Ok(row.map(common::roadmap).unwrap_or(Value::Null))
    }

    pub async fn delete_roadmap(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tDelete roadmap query start: id={}", param.id);
        let count = conn
            .execute("DELETE FROM roadmaps WHERE id = $1", &[&param.id])
            .await?;
        info!("\tDelete roadmap query completed: affected_rows={count}");
        Ok(common::deleted(count))
    }

    pub async fn list_project_roadmap(&self, param: ListProjectRoadmapParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let rows = conn
            .query(
                "SELECT id, project_id, version, title, generated_by, created_at::text
             FROM roadmaps WHERE project_id = $1 ORDER BY version DESC",
                &[&param.project_id],
            )
            .await?;
        Ok(Value::Array(
            rows.into_iter().map(common::roadmap).collect(),
        ))
    }
}
