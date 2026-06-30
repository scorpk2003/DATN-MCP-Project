use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateUserParam, GetUserByIdParam, UpsertUserParam},
};

#[derive(Debug, Clone)]
pub struct UserTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl UserTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_user(&self, param: CreateUserParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_user_table(&conn).await?;
        info!(
            "\tCreate user query start: firebase_uid={}",
            param.firebase_id
        );
        let row = conn
            .query_one(
                "INSERT INTO users (firebase_uid, display_name, email)
             VALUES ($1, $2, $3)
             RETURNING id, firebase_uid, display_name, email, created_at::text, updated_at::text",
                &[&param.firebase_id, &param.display_name, &param.email],
            )
            .await?;
        info!("\tCreate user query completed");
        Ok(common::user(row))
    }

    pub async fn upsert_user(&self, param: UpsertUserParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_user_table(&conn).await?;
        info!(
            "\tUpsert user query start: firebase_uid={}",
            param.firebase_id
        );
        let row = conn
            .query_one(
                "INSERT INTO users (firebase_uid, display_name, email)
             VALUES ($1, $2, $3)
             ON CONFLICT (firebase_uid)
             DO UPDATE SET
                display_name = COALESCE(EXCLUDED.display_name, users.display_name),
                email = COALESCE(EXCLUDED.email, users.email),
                updated_at = NOW()
             RETURNING id, firebase_uid, display_name, email, created_at::text, updated_at::text",
                &[&param.firebase_id, &param.display_name, &param.email],
            )
            .await?;
        info!("\tUpsert user query completed");
        Ok(common::user(row))
    }

    pub async fn get_user_by_id(&self, param: GetUserByIdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_user_table(&conn).await?;
        info!("\tGet user by firebase_uid query start");
        let row = conn
            .query_opt(
                "SELECT id, firebase_uid, display_name, email, created_at::text, updated_at::text
             FROM users WHERE firebase_uid = $1",
                &[&param.firebase_id],
            )
            .await?;
        info!("\tGet user by firebase_uid query completed");
        Ok(row.map(common::user).unwrap_or(Value::Null))
    }
}

async fn ensure_user_table(conn: &deadpool_postgres::Object) -> Result<()> {
    conn.batch_execute(
        "CREATE EXTENSION IF NOT EXISTS pgcrypto;
         CREATE TABLE IF NOT EXISTS users (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            firebase_uid text NOT NULL UNIQUE,
            display_name text,
            email text,
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );",
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constructor_keeps_provider_reference() {
        let provider = Arc::new(Mutex::new(SchemaProvider::default()));
        let tool = UserTool::new(provider.clone());
        assert!(Arc::ptr_eq(&provider, &tool.provider));
    }

    #[test]
    fn upsert_user_param_accepts_firebase_uid_without_uuid_parsing() {
        let param = UpsertUserParam {
            firebase_id: "lPSFlYu0VmaYhpy42JqekNC7pNa2".to_string(),
            display_name: None,
            email: None,
        };

        assert_eq!(param.firebase_id, "lPSFlYu0VmaYhpy42JqekNC7pNa2");
    }
}
