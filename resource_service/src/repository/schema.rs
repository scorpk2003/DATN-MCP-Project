use crate::AppResult;

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn run_schema_migration(&self) -> AppResult<()> {
        let client = self.pool.get().await?;
        client
            .batch_execute(include_str!("../../../resource_service_postgres.sql"))
            .await?;
        Ok(())
    }
}
