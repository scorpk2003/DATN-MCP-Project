use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::tools::common;
use crate::{
    provider::SchemaProvider,
    schemas::{CreateUserParam, GetUserByIdParam},
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

    pub async fn get_user_by_id(&self, param: GetUserByIdParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constructor_keeps_provider_reference() {
        let provider = Arc::new(Mutex::new(SchemaProvider::default()));
        let tool = UserTool::new(provider.clone());
        assert!(Arc::ptr_eq(&provider, &tool.provider));
    }
}
