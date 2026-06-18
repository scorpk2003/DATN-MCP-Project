use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::{provider::SchemaProvider, schemas::UserIdParam};

#[derive(Debug, Clone)]
pub struct AnalyticsTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl AnalyticsTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn get_user_statistics(&self, param: UserIdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tUser statistics query start: user_id={}", param.user_id);
        let row = conn.query_one(
            "SELECT
                (SELECT COUNT(*)::int FROM projects WHERE user_id = $1) AS project_count,
                (SELECT COUNT(*)::int FROM notes WHERE user_id = $1) AS note_count,
                (SELECT COUNT(*)::int FROM task_progress WHERE user_id = $1) AS tracked_task_count,
                (SELECT COUNT(*)::int FROM task_progress WHERE user_id = $1 AND status = 'completed') AS completed_task_count,
                (SELECT COALESCE(ROUND(AVG(progress_percent))::int, 0) FROM task_progress WHERE user_id = $1) AS average_progress",
            &[&param.user_id],
        ).await?;
        info!("\tUser statistics query completed");
        Ok(json!({
            "user_id": param.user_id.to_string(),
            "project_count": row.get::<_, i32>("project_count"),
            "note_count": row.get::<_, i32>("note_count"),
            "tracked_task_count": row.get::<_, i32>("tracked_task_count"),
            "completed_task_count": row.get::<_, i32>("completed_task_count"),
            "average_progress": row.get::<_, i32>("average_progress"),
        }))
    }

    pub async fn get_learning_history(&self, param: UserIdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tLearning history query start: user_id={}", param.user_id);
        let rows = conn
            .query(
                "SELECT
                tp.id,
                tp.user_id,
                tp.task_id,
                tp.status,
                tp.progress_percent,
                tp.started_at::text,
                tp.completed_at::text,
                t.title AS task_title,
                p.id AS project_id,
                p.title AS project_title
             FROM task_progress tp
             JOIN tasks t ON t.id = tp.task_id
             JOIN milestones m ON m.id = t.milestone_id
             JOIN roadmap_phases ph ON ph.id = m.phase_id
             JOIN roadmaps r ON r.id = ph.roadmap_id
             JOIN projects p ON p.id = r.project_id
             WHERE tp.user_id = $1
             ORDER BY COALESCE(tp.completed_at, tp.started_at) DESC NULLS LAST",
                &[&param.user_id],
            )
            .await?;
        info!("\tLearning history query completed: rows={}", rows.len());
        Ok(Value::Array(
            rows.into_iter()
                .map(|row| {
                    json!({
                        "progress": {
                            "id": row.get::<_, Uuid>("id").to_string(),
                            "user_id": row.get::<_, Uuid>("user_id").to_string(),
                            "task_id": row.get::<_, Uuid>("task_id").to_string(),
                            "status": row.get::<_, String>("status"),
                            "progress_percent": row.get::<_, Option<i32>>("progress_percent"),
                            "started_at": row.get::<_, Option<String>>("started_at"),
                            "completed_at": row.get::<_, Option<String>>("completed_at"),
                        },
                        "task_title": row.get::<_, String>("task_title"),
                        "project_id": row.get::<_, Uuid>("project_id").to_string(),
                        "project_title": row.get::<_, String>("project_title"),
                    })
                })
                .collect(),
        ))
    }
}
