use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        ApproveCandidateResponse, CandidateRequest, CandidateSummary, GapSummary, Page, PageQuery,
        ResearchTaskRequest, ResearchTaskSummary,
    },
};

use super::{
    ResourceRepository,
    mappers::{normalize_url, page, row_to_candidate, row_to_gap, row_to_research_task},
};

impl ResourceRepository {
    pub async fn create_gap(
        &self,
        requester: &str,
        topic: &str,
        min_required: i32,
        context: serde_json::Value,
    ) -> AppResult<Option<Uuid>> {
        let normalized = crate::repository::normalize_query(topic);
        let client = self.pool.get().await?;
        let gap_id: Option<Uuid> = client
            .query_one(
                "SELECT resource_service.create_gap_if_low_results($1, $2, $3, 0, $4, $5)",
                &[
                    &requester,
                    &topic,
                    &normalized,
                    &min_required,
                    &Json(&context),
                ],
            )
            .await?
            .get(0);
        Ok(gap_id)
    }

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

    pub async fn update_gap_status(&self, id: Uuid, status: &str) -> AppResult<GapSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "UPDATE resource_service.resource_gaps
                 SET status = CASE $2
                    WHEN 'pending' THEN 'pending'::resource_service.task_status
                    WHEN 'running' THEN 'running'::resource_service.task_status
                    WHEN 'succeeded' THEN 'succeeded'::resource_service.task_status
                    WHEN 'failed' THEN 'failed'::resource_service.task_status
                    WHEN 'cancelled' THEN 'cancelled'::resource_service.task_status
                    ELSE status
                 END,
                 fulfilled_at = CASE WHEN $2 = 'succeeded' THEN now() ELSE fulfilled_at END
                 WHERE id = $1
                 RETURNING id, topic_text, normalized_query, status::text, priority,
                           min_required_resources, context, created_at::text",
                &[&id, &status],
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

    pub async fn list_research_tasks(
        &self,
        query: &PageQuery,
    ) -> AppResult<Page<ResearchTaskSummary>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.research_tasks",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, gap_id, query_text, status::text, priority, metadata
                 FROM resource_service.research_tasks
                 ORDER BY priority DESC, created_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_research_task).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn get_research_task(&self, id: Uuid) -> AppResult<ResearchTaskSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, gap_id, query_text, status::text, priority, metadata
                 FROM resource_service.research_tasks
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or_else(|| AppError::Validation("research task not found".to_string()))?;
        Ok(row_to_research_task(&row))
    }

    pub async fn create_research_task_for_gap(
        &self,
        gap_id: Uuid,
        topic: &str,
        target_resource_types: &[String],
    ) -> AppResult<ResearchTaskSummary> {
        let request = ResearchTaskRequest {
            topic: topic.to_string(),
            gap_id: Some(gap_id),
            language: Some("en".to_string()),
            priority: Some(100),
            target_resource_types: Some(target_resource_types.to_vec()),
        };
        self.create_research_task(&request).await
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

    pub async fn approve_candidate_with_crawl(
        &self,
        id: Uuid,
    ) -> AppResult<ApproveCandidateResponse> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let row = tx
            .query_one(
                "UPDATE resource_service.research_candidates
                 SET selected = true, reject_reason = NULL
                 WHERE id = $1
                 RETURNING id, task_id, url, canonical_url, title, selected, reject_reason, metadata",
                &[&id],
            )
            .await?;
        let candidate = row_to_candidate(&row);
        let source_id = ensure_source_for_candidate(&tx, &candidate).await?;
        let seed_id = create_seed_for_candidate(&tx, source_id, &candidate).await?;
        let job_id = enqueue_candidate_job(&tx, source_id, seed_id, &candidate).await?;
        mark_gap_researching(&tx, candidate.task_id).await?;
        tx.commit().await?;

        Ok(ApproveCandidateResponse {
            candidate,
            created_crawl_seed_id: Some(seed_id),
            created_crawl_job_id: Some(job_id),
        })
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

    pub async fn get_candidate(&self, id: Uuid) -> AppResult<CandidateSummary> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, task_id, url, canonical_url, title, selected, reject_reason, metadata
                 FROM resource_service.research_candidates
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or_else(|| AppError::Validation("research candidate not found".to_string()))?;
        Ok(row_to_candidate(&row))
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

async fn ensure_source_for_candidate(
    tx: &tokio_postgres::Transaction<'_>,
    candidate: &CandidateSummary,
) -> AppResult<Option<Uuid>> {
    let host = url::Url::parse(&candidate.canonical_url)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string));
    let Some(host) = host else {
        return Ok(None);
    };
    let base_url = format!("https://{host}");
    let existing = tx
        .query_opt(
            "SELECT id FROM resource_service.source_sites WHERE host = $1 LIMIT 1",
            &[&host],
        )
        .await?;
    if let Some(row) = existing {
        return Ok(Some(row.get("id")));
    }
    let row = tx
        .query_one(
            "INSERT INTO resource_service.source_sites(
                name, kind, base_url, host, enabled, crawl_policy, notes
             ) VALUES (
                $1,
                'other'::resource_service.source_kind,
                $2,
                $3,
                true,
                $4,
                'created from approved research candidate'
             )
             RETURNING id",
            &[
                &host,
                &base_url,
                &host,
                &Json(
                    &json!({"respect_robots": true, "max_depth": 1, "rate_limit_per_minute": 20}),
                ),
            ],
        )
        .await?;
    Ok(Some(row.get("id")))
}

async fn create_seed_for_candidate(
    tx: &tokio_postgres::Transaction<'_>,
    source_id: Option<Uuid>,
    candidate: &CandidateSummary,
) -> AppResult<Uuid> {
    Ok(tx
        .query_one(
            "INSERT INTO resource_service.crawl_seeds(source_id, kind, seed_value, priority, enabled, metadata)
             VALUES ($1, 'url'::resource_service.seed_kind, $2, 150, true, $3)
             ON CONFLICT (source_id, kind, seed_value) DO UPDATE
             SET enabled = true,
                 priority = GREATEST(resource_service.crawl_seeds.priority, 150),
                 metadata = resource_service.crawl_seeds.metadata || EXCLUDED.metadata
             RETURNING id",
            &[
                &source_id,
                &candidate.canonical_url,
                &Json(&json!({"researchCandidateId": candidate.id})),
            ],
        )
        .await?
        .get("id"))
}

async fn enqueue_candidate_job(
    tx: &tokio_postgres::Transaction<'_>,
    source_id: Option<Uuid>,
    seed_id: Uuid,
    candidate: &CandidateSummary,
) -> AppResult<Uuid> {
    Ok(tx
        .query_one(
            "SELECT resource_service.enqueue_crawl_job($1, $2, $3, NULL, 150, 0, 'fetch', now(), NULL, $4)",
            &[
                &candidate.canonical_url,
                &source_id,
                &Some(seed_id),
                &Json(&json!({"researchCandidateId": candidate.id, "candidateType": candidate.candidate_type})),
            ],
        )
        .await?
        .get(0))
}

async fn mark_gap_researching(
    tx: &tokio_postgres::Transaction<'_>,
    task_id: Uuid,
) -> AppResult<()> {
    tx.execute(
        "UPDATE resource_service.resource_gaps g
         SET status = 'running'
         FROM resource_service.research_tasks t
         WHERE t.id = $1 AND g.id = t.gap_id",
        &[&task_id],
    )
    .await?;
    Ok(())
}
