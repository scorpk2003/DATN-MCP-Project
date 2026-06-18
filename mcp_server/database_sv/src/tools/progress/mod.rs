use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{GetProjectProgressParam, GetTaskProgressParam, UpdateTaskProgressParam},
};

#[derive(Debug, Clone)]
pub struct ProgressTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl ProgressTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn update_task_progress(&self, param: UpdateTaskProgressParam) -> Result<Value> {
        let percent = param.progress_percent.unwrap_or(0).clamp(0, 100);
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!(
            "\tUpsert task progress query start: user_id={}, task_id={}",
            param.user_id, param.task_id
        );
        let row = conn.query_one(
            "INSERT INTO task_progress (user_id, task_id, status, progress_percent, started_at, completed_at)
             VALUES ($1, $2, $3, $4, NOW(), CASE WHEN $3 = 'completed' THEN NOW() ELSE NULL END)
             ON CONFLICT (user_id, task_id) DO UPDATE SET
                status = EXCLUDED.status,
                progress_percent = EXCLUDED.progress_percent,
                started_at = COALESCE(task_progress.started_at, NOW()),
                completed_at = CASE WHEN EXCLUDED.status = 'completed' THEN NOW() ELSE NULL END
             RETURNING id, user_id, task_id, status, progress_percent, started_at::text, completed_at::text",
            &[&param.user_id, &param.task_id, &param.status, &percent],
        ).await?;
        info!("\tUpsert task progress query completed");
        Ok(common::progress(row))
    }

    pub async fn get_task_progress(&self, param: GetTaskProgressParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn.query_opt(
            "SELECT id, user_id, task_id, status, progress_percent, started_at::text, completed_at::text
             FROM task_progress WHERE user_id = $1 AND task_id = $2",
            &[&param.user_id, &param.task_id],
        ).await?;
        Ok(row.map(common::progress).unwrap_or(Value::Null))
    }

    pub async fn get_project_progress(&self, param: GetProjectProgressParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!(
            "\tProject progress aggregate query start: user_id={}, project_id={}",
            param.user_id, param.project_id
        );
        let row = conn
            .query_one(
                "SELECT
                COUNT(t.id)::int AS total_tasks,
                COUNT(tp.id)::int AS tracked_tasks,
                COUNT(CASE WHEN tp.status = 'completed' THEN 1 END)::int AS completed_tasks,
                COALESCE(ROUND(AVG(tp.progress_percent))::int, 0) AS average_progress
             FROM projects p
             JOIN roadmaps r ON r.project_id = p.id
             JOIN roadmap_phases ph ON ph.roadmap_id = r.id
             JOIN milestones m ON m.phase_id = ph.id
             JOIN tasks t ON t.milestone_id = m.id
             LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.user_id = $1
             WHERE p.id = $2",
                &[&param.user_id, &param.project_id],
            )
            .await?;
        info!("\tProject progress aggregate query completed");
        Ok(json!({
            "user_id": param.user_id.to_string(),
            "project_id": param.project_id.to_string(),
            "total_tasks": row.get::<_, i32>("total_tasks"),
            "tracked_tasks": row.get::<_, i32>("tracked_tasks"),
            "completed_tasks": row.get::<_, i32>("completed_tasks"),
            "average_progress": row.get::<_, i32>("average_progress"),
        }))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn progress_percent_is_clamped_before_binding() {
        assert_eq!(Some(150).unwrap_or(0).clamp(0, 100), 100);
        assert_eq!(Some(-1).unwrap_or(0).clamp(0, 100), 0);
    }
}
