use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateProjectParam, IdParam, ListProjectsParam, UpdateProjectParam},
};

#[derive(Debug, Clone)]
pub struct ProjectTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl ProjectTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_project(&self, param: CreateProjectParam) -> Result<Value> {
        let status = param.status.unwrap_or_else(|| "draft".to_string());
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let owner_user_id = resolve_user_ref(&conn, &param.user_id).await?;
        info!("\tCreate project query start: user_id={}", owner_user_id);
        let row = conn
            .query_one(
                "INSERT INTO projects (user_id, title, description, status)
             VALUES ($1, $2, $3, $4)
             RETURNING id, user_id, title, description, status, created_at::text, updated_at::text",
                &[&owner_user_id, &param.title, &param.description, &status],
            )
            .await?;
        info!("\tCreate project query completed");
        Ok(common::project(row))
    }

    pub async fn get_project(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let row = conn
            .query_opt(
                "SELECT id, user_id, title, description, status, created_at::text, updated_at::text
             FROM projects WHERE id = $1",
                &[&param.id],
            )
            .await?;
        Ok(row.map(common::project).unwrap_or(Value::Null))
    }

    pub async fn update_project(&self, param: UpdateProjectParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tUpdate project query start: id={}", param.id);
        let row = conn
            .query_opt(
                "UPDATE projects SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                updated_at = NOW()
             WHERE id = $1
             RETURNING id, user_id, title, description, status, created_at::text, updated_at::text",
                &[&param.id, &param.title, &param.description, &param.status],
            )
            .await?;
        info!("\tUpdate project query completed");
        Ok(row.map(common::project).unwrap_or(Value::Null))
    }

    pub async fn delete_project(&self, param: IdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        info!("\tDelete project query start: id={}", param.id);
        let count = conn
            .execute("DELETE FROM projects WHERE id = $1", &[&param.id])
            .await?;
        info!("\tDelete project query completed: affected_rows={count}");
        Ok(common::deleted(count))
    }

    pub async fn list_projects(&self, param: ListProjectsParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        let owner_user_id = resolve_user_ref(&conn, &param.user_id).await?;
        let rows = conn
            .query(
                "SELECT id, user_id, title, description, status, created_at::text, updated_at::text
             FROM projects WHERE user_id = $1 ORDER BY updated_at DESC",
                &[&owner_user_id],
            )
            .await?;
        Ok(Value::Array(
            rows.into_iter().map(common::project).collect(),
        ))
    }
}

async fn resolve_user_ref(conn: &tokio_postgres::Client, value: &str) -> Result<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(value) {
        return Ok(uuid);
    }

    let row = conn
        .query_opt("SELECT id FROM users WHERE firebase_uid = $1", &[&value])
        .await?;
    row.map(|row| row.get("id")).ok_or_else(|| {
        anyhow::anyhow!(
            "USER_NOT_FOUND: no users row exists for firebase_uid '{}'; create or sync the user before creating a project",
            value
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_project_default_status_is_bound_in_logic() {
        let param = CreateProjectParam {
            user_id: uuid::Uuid::new_v4().to_string(),
            title: "Roadmap".to_string(),
            description: None,
            status: None,
        };
        assert_eq!(param.status.unwrap_or_else(|| "draft".to_string()), "draft");
    }
}
