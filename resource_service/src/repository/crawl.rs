use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        ClaimJobsRequest, CompleteJobRequest, CrawlJob, CrawlJobRequest, CrawlSeed,
        CrawlSeedRequest, Page, PageQuery,
    },
};

use super::{
    ResourceRepository,
    mappers::{page, row_to_crawl_job, row_to_seed},
};

impl ResourceRepository {
    pub async fn create_crawl_seed(&self, request: &CrawlSeedRequest) -> AppResult<CrawlSeed> {
        let client = self.pool.get().await?;
        let kind = request
            .seed_type
            .clone()
            .unwrap_or_else(|| "url".to_string());
        let priority = request.priority.unwrap_or(100);
        let enabled = request.enabled.unwrap_or(true);
        let metadata = request.metadata.clone().unwrap_or_else(|| {
            json!({
                "maxDepth": request.max_depth.unwrap_or(0)
            })
        });
        let row = client
            .query_one(
                "INSERT INTO resource_service.crawl_seeds(
                    source_id, kind, seed_value, priority, enabled, metadata
                 ) VALUES (
                    $1,
                    CASE $2
                        WHEN 'url' THEN 'url'::resource_service.seed_kind
                        WHEN 'sitemap' THEN 'sitemap'::resource_service.seed_kind
                        WHEN 'rss' THEN 'rss'::resource_service.seed_kind
                        WHEN 'search_query' THEN 'search_query'::resource_service.seed_kind
                        WHEN 'manual_topic' THEN 'manual_topic'::resource_service.seed_kind
                        ELSE 'url'::resource_service.seed_kind
                    END,
                    $3, $4, $5, $6
                 )
                 RETURNING id, source_id, kind::text, seed_value, priority, enabled, metadata",
                &[
                    &request.source_site_id,
                    &kind,
                    &request.seed_url,
                    &priority,
                    &enabled,
                    &Json(&metadata),
                ],
            )
            .await?;
        Ok(row_to_seed(&row))
    }

    pub async fn list_crawl_seeds(&self, query: &PageQuery) -> AppResult<Page<CrawlSeed>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.crawl_seeds",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, source_id, kind::text, seed_value, priority, enabled, metadata
                 FROM resource_service.crawl_seeds
                 ORDER BY priority DESC, created_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_seed).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn create_crawl_job(&self, request: &CrawlJobRequest) -> AppResult<CrawlJob> {
        let client = self.pool.get().await?;
        let priority = request.priority.unwrap_or(100);
        let depth = request.depth.unwrap_or(0);
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        let job_id: Uuid = client
            .query_one(
                "SELECT resource_service.enqueue_crawl_job($1, $2, $3, $4, $5, $6, 'fetch', now(), NULL, $7)",
                &[
                    &request.url,
                    &request.source_site_id,
                    &request.seed_id,
                    &request.run_id,
                    &priority,
                    &depth,
                    &Json(&metadata),
                ],
            )
            .await?
            .get(0);
        self.get_crawl_job(job_id).await
    }

    pub async fn get_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, run_id, source_id, url, canonical_url, status::text,
                        priority, depth, attempts, max_attempts, last_error
                 FROM resource_service.crawl_jobs
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or(AppError::CrawlJobNotFound)?;
        Ok(row_to_crawl_job(&row))
    }

    pub async fn list_crawl_jobs(&self, query: &PageQuery) -> AppResult<Page<CrawlJob>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.crawl_jobs",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, run_id, source_id, url, canonical_url, status::text,
                        priority, depth, attempts, max_attempts, last_error
                 FROM resource_service.crawl_jobs
                 ORDER BY priority DESC, created_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_crawl_job).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn retry_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        let client = self.pool.get().await?;
        let affected = client
            .execute(
                "UPDATE resource_service.crawl_jobs
                 SET status = 'pending',
                     locked_by = NULL,
                     locked_at = NULL,
                     last_error = NULL,
                     not_before = now()
                 WHERE id = $1",
                &[&id],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::CrawlJobNotFound);
        }
        insert_outbox_event(
            &client,
            "crawl_job.retry",
            "crawl_job",
            Some(id),
            json!({"crawlJobId": id}),
        )
        .await?;
        self.get_crawl_job(id).await
    }

    pub async fn cancel_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        let client = self.pool.get().await?;
        let affected = client
            .execute(
                "UPDATE resource_service.crawl_jobs
                 SET status = 'cancelled',
                     locked_by = NULL,
                     locked_at = NULL,
                     last_error = 'cancelled by admin'
                 WHERE id = $1",
                &[&id],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::CrawlJobNotFound);
        }
        insert_outbox_event(
            &client,
            "crawl_job.cancelled",
            "crawl_job",
            Some(id),
            json!({"crawlJobId": id, "reason": "cancelled by admin"}),
        )
        .await?;
        self.get_crawl_job(id).await
    }

    pub async fn claim_crawl_jobs(&self, request: &ClaimJobsRequest) -> AppResult<Vec<CrawlJob>> {
        let client = self.pool.get().await?;
        let limit = request.limit.unwrap_or(10).clamp(1, 50) as i32;
        let rows = client
            .query(
                "SELECT id, run_id, source_id, url, canonical_url, status::text,
                        priority, depth, attempts, max_attempts, last_error
                 FROM resource_service.claim_crawl_jobs($1, $2)",
                &[&request.worker_id, &limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_crawl_job).collect())
    }

    pub async fn complete_crawl_job(
        &self,
        id: Uuid,
        request: &CompleteJobRequest,
    ) -> AppResult<CrawlJob> {
        let client = self.pool.get().await?;
        client
            .query_one(
                "SELECT resource_service.complete_crawl_job($1, $2, $3, $4)",
                &[
                    &id,
                    &request.succeeded,
                    &request.http_status,
                    &request.error,
                ],
            )
            .await?;
        self.get_crawl_job(id).await
    }
}

async fn insert_outbox_event(
    client: &tokio_postgres::Client,
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: Option<Uuid>,
    payload: serde_json::Value,
) -> AppResult<()> {
    client
        .execute(
            "INSERT INTO resource_service.outbox_events(event_type, aggregate_type, aggregate_id, payload)
             VALUES ($1, $2, $3, $4)",
            &[&event_type, &aggregate_type, &aggregate_id, &Json(&payload)],
        )
        .await?;
    Ok(())
}
