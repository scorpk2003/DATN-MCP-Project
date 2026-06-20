use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        CandidateRequest, CandidateSummary, GapSummary, Page, PageQuery, ResearchTaskRequest,
        ResearchTaskSummary,
    },
};

use super::{
    ResourceRepository,
    mappers::{normalize_url, page, row_to_candidate, row_to_gap, row_to_research_task},
};

impl ResourceRepository {
    pub async fn list_gaps(&self, query: &PageQuery) -> AppResult<Page<GapSummary>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.resource_gaps",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, topic_text, normalized_query, status::text, priority,
                        min_required_resources, context, created_at::text
                 FROM resource_service.resource_gaps
                 ORDER BY priority DESC, created_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_gap).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn get_gap(&self, id: Uuid) -> AppResult<GapSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, topic_text, normalized_query, status::text, priority,
                        min_required_resources, context, created_at::text
                 FROM resource_service.resource_gaps
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or(AppError::GapNotFound)?;
        Ok(row_to_gap(&row))
    }

    pub async fn create_research_task(
        &self,
        request: &ResearchTaskRequest,
    ) -> AppResult<ResearchTaskSummary> {
        let client = self.pool.get().await?;
        let priority = request.priority.unwrap_or(100);
        let language = request.language.clone().unwrap_or_else(|| "en".to_string());
        let metadata = json!({
            "targetResourceTypes": request.target_resource_types.clone().unwrap_or_default()
        });
        let row = client
            .query_one(
                "INSERT INTO resource_service.research_tasks(
                    gap_id, query_text, language_code, priority, requested_by, metadata
                 ) VALUES ($1, $2, $3, $4, 'resource_service_api', $5)
                 RETURNING id, gap_id, query_text, status::text, priority, metadata",
                &[
                    &request.gap_id,
                    &request.topic,
                    &language,
                    &priority,
                    &Json(&metadata),
                ],
            )
            .await?;
        Ok(row_to_research_task(&row))
    }

    pub async fn create_candidate(
        &self,
        request: &CandidateRequest,
    ) -> AppResult<CandidateSummary> {
        let client = self.pool.get().await?;
        let canonical_url = normalize_url(&request.url);
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        let row = client
            .query_one(
                "INSERT INTO resource_service.research_candidates(
                    task_id, url, canonical_url, title, snippet, metadata
                 ) VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING id, task_id, url, canonical_url, title, selected, reject_reason, metadata",
                &[
                    &request.research_task_id,
                    &request.url,
                    &canonical_url,
                    &request.title,
                    &request.snippet,
                    &Json(&metadata),
                ],
            )
            .await?;
        Ok(row_to_candidate(&row))
    }

    pub async fn list_candidates(&self, query: &PageQuery) -> AppResult<Page<CandidateSummary>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.research_candidates",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, task_id, url, canonical_url, title, selected, reject_reason, metadata
                 FROM resource_service.research_candidates
                 ORDER BY created_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_candidate).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn approve_candidate(&self, id: Uuid) -> AppResult<CandidateSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "UPDATE resource_service.research_candidates
                 SET selected = true, reject_reason = NULL
                 WHERE id = $1
                 RETURNING id, task_id, url, canonical_url, title, selected, reject_reason, metadata",
                &[&id],
            )
            .await?;
        Ok(row_to_candidate(&row))
    }

    pub async fn reject_candidate(&self, id: Uuid, reason: &str) -> AppResult<CandidateSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "UPDATE resource_service.research_candidates
                 SET selected = false, reject_reason = $2
                 WHERE id = $1
                 RETURNING id, task_id, url, canonical_url, title, selected, reject_reason, metadata",
                &[&id, &reason],
            )
            .await?;
        Ok(row_to_candidate(&row))
    }
}
