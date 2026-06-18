use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{provider::SchemaProvider, schemas::SearchParam};

#[derive(Debug, Clone)]
pub struct SearchTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl SearchTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn search_projects(&self, param: SearchParam) -> Result<Value> {
        let keyword = format!("%{}%", param.keyword);
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tSearch projects query start");
        let rows = conn
            .query(
                "SELECT id, user_id, title, description, status, created_at::text, updated_at::text
             FROM projects WHERE title ILIKE $1 OR description ILIKE $1
             ORDER BY updated_at DESC LIMIT 20",
                &[&keyword],
            )
            .await?;
        info!("\tSearch projects query completed: rows={}", rows.len());
        Ok(Value::Array(
            rows.into_iter().map(common::project).collect(),
        ))
    }

    pub async fn search_tasks(&self, param: SearchParam) -> Result<Value> {
        let keyword = format!("%{}%", param.keyword);
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tSearch tasks query start");
        let rows = conn.query(
            "SELECT id, milestone_id, task_order, title, description, estimated_hours, difficulty, status
             FROM tasks WHERE title ILIKE $1 OR description ILIKE $1
             ORDER BY task_order ASC LIMIT 20",
            &[&keyword],
        ).await?;
        info!("\tSearch tasks query completed: rows={}", rows.len());
        Ok(Value::Array(rows.into_iter().map(common::task).collect()))
    }

    pub async fn search_notes(&self, param: SearchParam) -> Result<Value> {
        let keyword = format!("%{}%", param.keyword);
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tSearch notes query start");
        let rows = conn
            .query(
                "SELECT id, user_id, task_id, content, created_at::text
             FROM notes WHERE content ILIKE $1 ORDER BY created_at DESC LIMIT 20",
                &[&keyword],
            )
            .await?;
        info!("\tSearch notes query completed: rows={}", rows.len());
        Ok(Value::Array(rows.into_iter().map(common::note).collect()))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn search_keyword_is_wrapped_for_ilike() {
        let keyword = "rust";
        assert_eq!(format!("%{}%", keyword), "%rust%");
    }
}
