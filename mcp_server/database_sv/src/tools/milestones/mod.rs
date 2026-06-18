use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateMilestoneParam, IdParam, UpdateMilestoneParam},
};

#[derive(Debug, Clone)]
pub struct MilestoneTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl MilestoneTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_milestone(&self, param: CreateMilestoneParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn
            .query_one(
                "INSERT INTO milestones (phase_id, milestone_order, title, description)
             VALUES ($1, $2, $3, $4)
             RETURNING id, phase_id, milestone_order, title, description",
                &[
                    &param.phase_id,
                    &param.milestone_order,
                    &param.title,
                    &param.description,
                ],
            )
            .await?;
        Ok(common::milestone(row))
    }

    pub async fn get_milestone(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_opt(
            "SELECT id, phase_id, milestone_order, title, description FROM milestones WHERE id = $1",
            &[&param.id],
        ).await?;
        Ok(row.map(common::milestone).unwrap_or(Value::Null))
    }

    pub async fn update_milestone(&self, param: UpdateMilestoneParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tUpdate milestone query start: id={}", param.id);
        let row = conn
            .query_opt(
                "UPDATE milestones SET
                milestone_order = COALESCE($2, milestone_order),
                title = COALESCE($3, title),
                description = COALESCE($4, description)
             WHERE id = $1
             RETURNING id, phase_id, milestone_order, title, description",
                &[
                    &param.id,
                    &param.milestone_order,
                    &param.title,
                    &param.description,
                ],
            )
            .await?;
        info!("\tUpdate milestone query completed");
        Ok(row.map(common::milestone).unwrap_or(Value::Null))
    }

    pub async fn delete_milestone(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let count = conn
            .execute("DELETE FROM milestones WHERE id = $1", &[&param.id])
            .await?;
        Ok(common::deleted(count))
    }
}
