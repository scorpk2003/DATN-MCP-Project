use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    chunker::Chunk,
    models::{
        CreateResourceRequest, CreateResourceResponse, CreateResourceVersionRequest,
        IngestResourceResponse, ManualIngestRequest, Page, PageQuery, ResourceChunk,
        ResourceDetail, ResourceSummary, ResourceVersionSummary, UpdateResourceRequest,
    },
};

use super::{
    ResourceRepository,
    mappers::{
        extract_domain, page, row_to_chunk, row_to_resource_summary, row_to_version_summary,
        sha256_bytes,
    },
};

impl ResourceRepository {
    pub async fn create_resource(
        &self,
        request: &CreateResourceRequest,
    ) -> AppResult<CreateResourceResponse> {
        let client = self.pool.get().await?;
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        let kind = request
            .resource_type
            .clone()
            .unwrap_or_else(|| "article".to_string());
        let format = request
            .resource_format
            .clone()
            .unwrap_or_else(|| "html".to_string());
        let language = request.language.clone().unwrap_or_else(|| "en".to_string());
        let primary_domain = extract_domain(&request.canonical_url);
        let is_official = false;

        let resource_id: Uuid = client
            .query_one(
                "SELECT resource_service.upsert_resource(
                    $1, $2, $3, $4, $5,
                    CASE $6
                        WHEN 'docs' THEN 'docs'::resource_service.resource_kind
                        WHEN 'specification' THEN 'specification'::resource_service.resource_kind
                        WHEN 'repo' THEN 'repo'::resource_service.resource_kind
                        WHEN 'paper' THEN 'paper'::resource_service.resource_kind
                        WHEN 'course' THEN 'course'::resource_service.resource_kind
                        WHEN 'tutorial' THEN 'tutorial'::resource_service.resource_kind
                        WHEN 'article' THEN 'article'::resource_service.resource_kind
                        WHEN 'qna' THEN 'qna'::resource_service.resource_kind
                        WHEN 'video' THEN 'video'::resource_service.resource_kind
                        WHEN 'book' THEN 'book'::resource_service.resource_kind
                        WHEN 'exercise' THEN 'exercise'::resource_service.resource_kind
                        WHEN 'dataset' THEN 'dataset'::resource_service.resource_kind
                        ELSE 'other'::resource_service.resource_kind
                    END,
                    CASE $7
                        WHEN 'html' THEN 'html'::resource_service.content_format
                        WHEN 'markdown' THEN 'markdown'::resource_service.content_format
                        WHEN 'pdf' THEN 'pdf'::resource_service.content_format
                        WHEN 'video' THEN 'video'::resource_service.content_format
                        WHEN 'code' THEN 'code'::resource_service.content_format
                        WHEN 'notebook' THEN 'notebook'::resource_service.content_format
                        WHEN 'plain_text' THEN 'plain_text'::resource_service.content_format
                        WHEN 'dataset' THEN 'dataset'::resource_service.content_format
                        ELSE 'other'::resource_service.content_format
                    END,
                    $8, $9, $10, $11
                )",
                &[
                    &request.source_site_id,
                    &request.canonical_url,
                    &request.title,
                    &request.summary,
                    &request.description,
                    &kind,
                    &format,
                    &language,
                    &primary_domain,
                    &is_official,
                    &Json(&metadata),
                ],
            )
            .await?
            .get(0);

        Ok(CreateResourceResponse {
            resource_id,
            status: "created".to_string(),
            processing_status: "discovered".to_string(),
        })
    }

    pub async fn update_resource(
        &self,
        id: Uuid,
        request: &UpdateResourceRequest,
    ) -> AppResult<ResourceDetail> {
        let client = self.pool.get().await?;
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        let metadata_patch = request.metadata.is_some();
        let affected = client
            .execute(
                "UPDATE resource_service.resources
                 SET title = COALESCE($2, title),
                     summary = COALESCE($3, summary),
                     description = COALESCE($4, description),
                     status = COALESCE(
                        CASE $5
                            WHEN 'candidate' THEN 'candidate'::resource_service.resource_status
                            WHEN 'active' THEN 'active'::resource_service.resource_status
                            WHEN 'stale' THEN 'stale'::resource_service.resource_status
                            WHEN 'rejected' THEN 'rejected'::resource_service.resource_status
                            WHEN 'archived' THEN 'archived'::resource_service.resource_status
                            ELSE NULL
                        END,
                        status
                     ),
                     difficulty = COALESCE(
                        CASE $6
                            WHEN 'unknown' THEN 'unknown'::resource_service.difficulty_level
                            WHEN 'beginner' THEN 'beginner'::resource_service.difficulty_level
                            WHEN 'intermediate' THEN 'intermediate'::resource_service.difficulty_level
                            WHEN 'advanced' THEN 'advanced'::resource_service.difficulty_level
                            WHEN 'expert' THEN 'expert'::resource_service.difficulty_level
                            WHEN 'mixed' THEN 'mixed'::resource_service.difficulty_level
                            ELSE NULL
                        END,
                        difficulty
                     ),
                     quality_score = COALESCE($7, quality_score),
                     metadata = CASE WHEN $8 THEN metadata || $9 ELSE metadata END
                 WHERE id = $1",
                &[
                    &id,
                    &request.title,
                    &request.summary,
                    &request.description,
                    &request.status,
                    &request.difficulty,
                    &request.quality_score,
                    &metadata_patch,
                    &Json(&metadata),
                ],
            )
            .await?;

        if affected == 0 {
            return Err(AppError::ResourceNotFound);
        }
        self.get_resource_detail(id).await
    }

    pub async fn create_resource_version(
        &self,
        resource_id: Uuid,
        request: &CreateResourceVersionRequest,
        chunks: &[Chunk],
    ) -> AppResult<IngestResourceResponse> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        let text_hash = sha256_bytes(&request.content);
        let version_id: Uuid = tx
            .query_one(
                "SELECT resource_service.create_resource_version(
                    $1, $2, $3, $4, $5, $6, 'api', 'resource_service_v1', $7
                )",
                &[
                    &resource_id,
                    &request.fetch_artifact_id,
                    &request.title,
                    &request.content,
                    &text_hash,
                    &request.markdown,
                    &Json(&metadata),
                ],
            )
            .await?
            .get(0);

        let existing_chunk_count: i64 = tx
            .query_one(
                "SELECT count(*)::bigint
                 FROM resource_service.resource_chunks
                 WHERE version_id = $1",
                &[&version_id],
            )
            .await?
            .get(0);
        if existing_chunk_count > 0 {
            tx.commit().await?;
            return Ok(IngestResourceResponse {
                resource_id,
                version_id,
                chunk_count: existing_chunk_count as i32,
            });
        }

        let chunk_json = serde_json::to_value(chunks)
            .map_err(|err| AppError::Internal(format!("serialize chunks failed: {err}")))?;
        let chunk_count: i32 = tx
            .query_one(
                "SELECT resource_service.replace_chunks_from_json($1, $2, $3)",
                &[&resource_id, &version_id, &Json(&chunk_json)],
            )
            .await?
            .get(0);
        tx.commit().await?;

        Ok(IngestResourceResponse {
            resource_id,
            version_id,
            chunk_count,
        })
    }

    pub async fn activate_resource(&self, resource_id: Uuid) -> AppResult<()> {
        let client = self.pool.get().await?;
        let affected = client
            .execute(
                "UPDATE resource_service.resources
                 SET status = 'active'
                 WHERE id = $1",
                &[&resource_id],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::ResourceNotFound);
        }
        Ok(())
    }

    pub async fn ingest_manual(
        &self,
        request: &ManualIngestRequest,
        chunks: &[Chunk],
    ) -> AppResult<IngestResourceResponse> {
        let create = CreateResourceRequest {
            title: request.title.clone(),
            canonical_url: request.canonical_url.clone(),
            source_site_id: request.source_id,
            language: request.language_code.clone(),
            resource_type: request.kind.clone(),
            resource_format: request.format.clone(),
            summary: request.summary.clone(),
            description: request.description.clone(),
            metadata: request.metadata.clone(),
        };
        let created = self.create_resource(&create).await?;
        let version = CreateResourceVersionRequest {
            title: Some(request.title.clone()),
            content: request.content.clone(),
            markdown: None,
            fetch_artifact_id: None,
            metadata: request.metadata.clone(),
        };
        let result = self
            .create_resource_version(created.resource_id, &version, chunks)
            .await?;
        self.activate_resource(created.resource_id).await?;
        Ok(result)
    }

    pub async fn list_resources(&self, query: &PageQuery) -> AppResult<Page<ResourceSummary>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.resources",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT
                    id, canonical_url, title, summary, kind::text, format::text, status::text,
                    language_code, difficulty::text, quality_score::double precision, is_official,
                    created_at::text, updated_at::text
                 FROM resource_service.resources
                 ORDER BY updated_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;

        Ok(page(
            rows.iter().map(row_to_resource_summary).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn get_resource_detail(&self, id: Uuid) -> AppResult<ResourceDetail> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT
                    r.id, r.canonical_url, r.title, r.summary, r.kind::text, r.format::text,
                    r.status::text, r.language_code, r.difficulty::text,
                    r.quality_score::double precision, r.is_official, r.created_at::text,
                    r.updated_at::text, s.name AS source_name, s.kind::text AS source_type
                 FROM resource_service.resources r
                 LEFT JOIN resource_service.source_sites s ON s.id = r.source_id
                 WHERE r.id = $1",
                &[&id],
            )
            .await?
            .ok_or(AppError::ResourceNotFound)?;

        let latest_version = client
            .query_opt(
                "SELECT id, version_no, title, extracted_at::text
                 FROM resource_service.resource_versions
                 WHERE resource_id = $1
                 ORDER BY version_no DESC
                 LIMIT 1",
                &[&id],
            )
            .await?
            .map(row_to_version_summary);

        let chunk_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.resource_chunks WHERE resource_id = $1",
                &[&id],
            )
            .await?
            .get(0);

        Ok(ResourceDetail {
            resource: row_to_resource_summary(&row),
            source_name: row.get("source_name"),
            source_type: row.get("source_type"),
            latest_version,
            chunk_count,
        })
    }

    pub async fn list_versions(&self, resource_id: Uuid) -> AppResult<Vec<ResourceVersionSummary>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, version_no, title, extracted_at::text
                 FROM resource_service.resource_versions
                 WHERE resource_id = $1
                 ORDER BY version_no DESC",
                &[&resource_id],
            )
            .await?;
        Ok(rows.into_iter().map(row_to_version_summary).collect())
    }

    pub async fn get_resource_chunks(
        &self,
        resource_id: Uuid,
        version_id: Option<Uuid>,
        limit: i64,
    ) -> AppResult<Vec<ResourceChunk>> {
        let client = self.pool.get().await?;
        let rows = match version_id {
            Some(version_id) => {
                client
                    .query(
                        "SELECT id, version_id, chunk_index, heading_path, content, content_tokens, metadata
                         FROM resource_service.resource_chunks
                         WHERE resource_id = $1 AND version_id = $2
                         ORDER BY chunk_index ASC
                         LIMIT $3",
                        &[&resource_id, &version_id, &limit],
                    )
                    .await?
            }
            None => {
                client
                    .query(
                        "SELECT id, version_id, chunk_index, heading_path, content, content_tokens, metadata
                         FROM resource_service.resource_chunks
                         WHERE resource_id = $1
                         ORDER BY chunk_index ASC
                         LIMIT $2",
                        &[&resource_id, &limit],
                    )
                    .await?
            }
        };
        Ok(rows.iter().map(row_to_chunk).collect())
    }
}
