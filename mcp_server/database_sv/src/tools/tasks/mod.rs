use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateTaskParam, IdParam, UpdateTaskParam},
};

#[derive(Debug, Clone)]
pub struct TaskTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl TaskTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_task(&self, param: CreateTaskParam) -> Result<Value> {
        let status = param.status.unwrap_or_else(|| "pending".to_string());
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_one(
            "INSERT INTO tasks (milestone_id, task_order, title, description, estimated_hours, difficulty, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, milestone_id, task_order, title, description, estimated_hours, difficulty, status",
            &[&param.milestone_id, &param.task_order, &param.title, &param.description, &param.estimated_hours, &param.difficulty, &status],
        ).await?;
        Ok(common::task(row))
    }

    pub async fn get_task(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_opt(
            "SELECT id, milestone_id, task_order, title, description, estimated_hours, difficulty, status FROM tasks WHERE id = $1",
            &[&param.id],
        ).await?;
        Ok(row.map(common::task).unwrap_or(Value::Null))
    }

    pub async fn update_task(&self, param: UpdateTaskParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tUpdate task query start: id={}", param.id);
        let row = conn.query_opt(
            "UPDATE tasks SET
                task_order = COALESCE($2, task_order),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                estimated_hours = COALESCE($5, estimated_hours),
                difficulty = COALESCE($6, difficulty),
                status = COALESCE($7, status)
             WHERE id = $1
             RETURNING id, milestone_id, task_order, title, description, estimated_hours, difficulty, status",
            &[&param.id, &param.task_order, &param.title, &param.description, &param.estimated_hours, &param.difficulty, &param.status],
        ).await?;
        info!("\tUpdate task query completed");
        Ok(row.map(common::task).unwrap_or(Value::Null))
    }

    pub async fn delete_task(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let count = conn
            .execute("DELETE FROM tasks WHERE id = $1", &[&param.id])
            .await?;
        Ok(common::deleted(count))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_task_default_status_is_pending() {
        let param = CreateTaskParam {
            milestone_id: uuid::Uuid::new_v4(),
            task_order: 1,
            title: "Read docs".to_string(),
            description: None,
            estimated_hours: None,
            difficulty: None,
            status: None,
        };
        assert_eq!(
            param.status.unwrap_or_else(|| "pending".to_string()),
            "pending"
        );
    }
}
