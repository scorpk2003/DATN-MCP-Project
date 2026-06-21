use tokio_postgres::types::Json;

use crate::{
    AppError, AppResult,
    models::{FetchArtifact, FetchArtifactRequest},
};

use super::{
    ResourceRepository,
    mappers::{row_to_fetch_artifact, sha256_bytes},
};

pub(crate) struct FetchArtifactContent {
    pub id: uuid::Uuid,
    pub source_id: Option<uuid::Uuid>,
    pub url: String,
    pub final_url: Option<String>,
    pub content_type: Option<String>,
    pub raw_body: String,
    pub metadata: serde_json::Value,
}

impl ResourceRepository {
    pub async fn create_fetch_artifact(
        &self,
        request: &FetchArtifactRequest,
    ) -> AppResult<FetchArtifact> {
        let client = self.pool.get().await?;
        let metadata = request
            .metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let body_sha256 = request.raw_body.as_deref().map(sha256_bytes);
        let row = client
            .query_one(
                "INSERT INTO resource_service.fetch_artifacts(
                    job_id, source_id, url, final_url, http_status, content_type,
                    content_length, etag, raw_object_key, raw_body, body_sha256, metadata
                 ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10, $11, $12
                 )
                 ON CONFLICT (job_id) DO UPDATE
                 SET source_id = EXCLUDED.source_id,
                     url = EXCLUDED.url,
                     final_url = EXCLUDED.final_url,
                     http_status = EXCLUDED.http_status,
                     content_type = EXCLUDED.content_type,
                     content_length = EXCLUDED.content_length,
                     etag = EXCLUDED.etag,
                     raw_object_key = EXCLUDED.raw_object_key,
                     raw_body = EXCLUDED.raw_body,
                     body_sha256 = EXCLUDED.body_sha256,
                     metadata = resource_service.fetch_artifacts.metadata || EXCLUDED.metadata,
                     fetched_at = now()
                 RETURNING id, job_id, url, final_url, http_status, content_type,
                           content_length, body_sha256",
                &[
                    &request.crawl_job_id,
                    &request.source_site_id,
                    &request.url,
                    &request.final_url,
                    &request.http_status,
                    &request.content_type,
                    &request.content_length,
                    &request.etag,
                    &request.raw_object_key,
                    &request.raw_body,
                    &body_sha256,
                    &Json(&metadata),
                ],
            )
            .await?;
        Ok(row_to_fetch_artifact(&row))
    }

    pub(crate) async fn get_fetch_artifact_content(
        &self,
        id: uuid::Uuid,
    ) -> AppResult<FetchArtifactContent> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, source_id, url, final_url, content_type, raw_body, metadata
                 FROM resource_service.fetch_artifacts
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or_else(|| AppError::Validation("fetchArtifactId was not found".to_string()))?;
        let raw_body: Option<String> = row.get("raw_body");
        let raw_body = raw_body.ok_or_else(|| {
            AppError::Validation(
                "fetch artifact has no rawBody; object storage fetch is not implemented yet"
                    .to_string(),
            )
        })?;

        Ok(FetchArtifactContent {
            id: row.get("id"),
            source_id: row.get("source_id"),
            url: row.get("url"),
            final_url: row.get("final_url"),
            content_type: row.get("content_type"),
            raw_body,
            metadata: row.get("metadata"),
        })
    }

    pub(crate) async fn list_unprocessed_fetch_artifact_ids(
        &self,
        limit: i64,
    ) -> AppResult<Vec<uuid::Uuid>> {
        let client = self.pool.get().await?;
        let limit = limit.clamp(1, 100);
        let rows = client
            .query(
                "SELECT fa.id
                 FROM resource_service.fetch_artifacts fa
                 LEFT JOIN resource_service.resource_versions rv
                   ON rv.fetch_artifact_id = fa.id
                 WHERE rv.id IS NULL
                   AND fa.http_status BETWEEN 200 AND 399
                   AND fa.raw_body IS NOT NULL
                   AND fa.metadata->'extraction'->>'resourceVersionId' IS NULL
                 ORDER BY fa.fetched_at ASC
                 LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(|row| row.get("id")).collect())
    }

    pub(crate) async fn mark_fetch_artifact_extracted(
        &self,
        id: uuid::Uuid,
        resource_id: uuid::Uuid,
        version_id: uuid::Uuid,
        chunk_count: i32,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;
        let marker = serde_json::json!({
            "extraction": {
                "resourceId": resource_id,
                "resourceVersionId": version_id,
                "chunkCount": chunk_count,
                "processedAt": crate::models::utc_timestamp()
            }
        });
        client
            .execute(
                "UPDATE resource_service.fetch_artifacts
                 SET metadata = metadata || $2
                 WHERE id = $1",
                &[&id, &Json(&marker)],
            )
            .await?;
        Ok(())
    }
}
