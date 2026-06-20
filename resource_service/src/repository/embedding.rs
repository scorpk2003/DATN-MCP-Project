use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppResult,
    models::{
        EmbeddingModelRequest, EmbeddingModelSummary, PendingEmbeddingChunk,
        PendingEmbeddingChunksQuery,
    },
};

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn create_embedding_model(
        &self,
        request: &EmbeddingModelRequest,
    ) -> AppResult<EmbeddingModelSummary> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        if request.is_default.unwrap_or(false) {
            tx.execute(
                "UPDATE resource_service.embedding_models
                 SET is_default = false
                 WHERE is_default = true",
                &[],
            )
            .await?;
        }
        let metric = request
            .metric
            .clone()
            .unwrap_or_else(|| "cosine".to_string());
        let metadata = serde_json::json!({"createdBy": "resource_service_api"});
        let row = tx
            .query_one(
                "INSERT INTO resource_service.embedding_models(
                    provider, name, dimensions, metric, is_default, metadata
                 ) VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (provider, name) DO UPDATE
                 SET dimensions = EXCLUDED.dimensions,
                     metric = EXCLUDED.metric,
                     is_default = EXCLUDED.is_default,
                     metadata = resource_service.embedding_models.metadata || EXCLUDED.metadata
                 RETURNING id, provider, name, dimensions, metric, is_default",
                &[
                    &request.provider,
                    &request.name,
                    &request.dimensions,
                    &metric,
                    &request.is_default.unwrap_or(false),
                    &Json(&metadata),
                ],
            )
            .await?;
        tx.commit().await?;
        Ok(row_to_embedding_model(&row))
    }

    pub async fn list_embedding_models(&self) -> AppResult<Vec<EmbeddingModelSummary>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, provider, name, dimensions, metric, is_default
                 FROM resource_service.embedding_models
                 ORDER BY is_default DESC, provider ASC, name ASC",
                &[],
            )
            .await?;
        Ok(rows.iter().map(row_to_embedding_model).collect())
    }

    pub async fn list_pending_embedding_chunks(
        &self,
        query: &PendingEmbeddingChunksQuery,
    ) -> AppResult<Vec<PendingEmbeddingChunk>> {
        let client = self.pool.get().await?;
        let model_id = match query.embedding_model_id {
            Some(model_id) => model_id,
            None => client
                .query_one(
                    "SELECT id
                         FROM resource_service.embedding_models
                         WHERE is_default = true
                         LIMIT 1",
                    &[],
                )
                .await?
                .get("id"),
        };
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let rows = client
            .query(
                "SELECT
                    c.id AS chunk_id,
                    c.resource_id,
                    c.version_id,
                    c.heading_path,
                    c.content,
                    c.content_tokens,
                    r.title
                 FROM resource_service.resource_chunks c
                 JOIN resource_service.resources r ON r.id = c.resource_id
                 LEFT JOIN resource_service.resource_chunk_embeddings e
                   ON e.chunk_id = c.id AND e.model_id = $1
                 WHERE r.status = 'active'
                   AND e.chunk_id IS NULL
                 ORDER BY c.created_at ASC
                 LIMIT $2",
                &[&model_id, &limit],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|row| row_to_pending_chunk(row, model_id))
            .collect())
    }
}

fn row_to_embedding_model(row: &tokio_postgres::Row) -> EmbeddingModelSummary {
    EmbeddingModelSummary {
        id: row.get("id"),
        provider: row.get("provider"),
        name: row.get("name"),
        dimensions: row.get("dimensions"),
        metric: row.get("metric"),
        is_default: row.get("is_default"),
    }
}

fn row_to_pending_chunk(row: &tokio_postgres::Row, model_id: Uuid) -> PendingEmbeddingChunk {
    let title: String = row.get("title");
    let heading_path: Vec<String> = row.get("heading_path");
    let content: String = row.get("content");
    let section = heading_path.join(" > ");
    let input_text = format!("Title: {title}\nSection: {section}\nContent:\n{content}");
    PendingEmbeddingChunk {
        chunk_id: row.get("chunk_id"),
        resource_id: row.get("resource_id"),
        version_id: row.get("version_id"),
        embedding_model_id: model_id,
        title,
        heading_path,
        input_text,
        token_count: row.get("content_tokens"),
    }
}
