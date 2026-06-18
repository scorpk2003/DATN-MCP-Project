use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreatePhaseParam, IdParam, UpdatePhaseParam},
};

#[derive(Debug, Clone)]
pub struct PhaseTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl PhaseTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_phase(&self, param: CreatePhaseParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_one(
            "INSERT INTO roadmap_phases (roadmap_id, phase_order, title, description, estimated_days)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, roadmap_id, phase_order, title, description, estimated_days",
            &[&param.roadmap_id, &param.phase_order, &param.title, &param.description, &param.estimated_days],
        ).await?;
        Ok(common::phase(row))
    }

    pub async fn get_phase(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_opt(
            "SELECT id, roadmap_id, phase_order, title, description, estimated_days FROM roadmap_phases WHERE id = $1",
            &[&param.id],
        ).await?;
        Ok(row.map(common::phase).unwrap_or(Value::Null))
    }

    pub async fn update_phase(&self, param: UpdatePhaseParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tUpdate phase query start: id={}", param.id);
        let row = conn
            .query_opt(
                "UPDATE roadmap_phases SET
                phase_order = COALESCE($2, phase_order),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                estimated_days = COALESCE($5, estimated_days)
             WHERE id = $1
             RETURNING id, roadmap_id, phase_order, title, description, estimated_days",
                &[
                    &param.id,
                    &param.phase_order,
                    &param.title,
                    &param.description,
                    &param.estimated_days,
                ],
            )
            .await?;
        info!("\tUpdate phase query completed");
        Ok(row.map(common::phase).unwrap_or(Value::Null))
    }

    pub async fn delete_phase(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let count = conn
            .execute("DELETE FROM roadmap_phases WHERE id = $1", &[&param.id])
            .await?;
        Ok(common::deleted(count))
    }
}
