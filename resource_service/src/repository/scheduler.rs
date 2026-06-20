use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppResult,
    models::{ScheduleCrawlRequest, ScheduleCrawlResponse},
};

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn schedule_crawl(
        &self,
        request: &ScheduleCrawlRequest,
    ) -> AppResult<ScheduleCrawlResponse> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let crawl_run_id = create_crawl_run(&tx, request).await?;
        let seed_rows = load_schedulable_seeds(&tx, request).await?;
        let mut created_job_ids = Vec::with_capacity(seed_rows.len());

        for seed in seed_rows {
            let job_id = enqueue_seed_job(&tx, crawl_run_id, &seed).await?;
            mark_seed_enqueued(&tx, seed.id).await?;
            created_job_ids.push(job_id);
        }

        finish_crawl_run(&tx, crawl_run_id, created_job_ids.len()).await?;
        tx.commit().await?;

        Ok(ScheduleCrawlResponse {
            crawl_run_id,
            created_job_count: created_job_ids.len(),
            created_job_ids,
        })
    }
}

struct SchedulableSeed {
    id: Uuid,
    source_id: Option<Uuid>,
    seed_url: String,
    priority: i32,
    metadata: serde_json::Value,
}

async fn create_crawl_run(
    tx: &tokio_postgres::Transaction<'_>,
    request: &ScheduleCrawlRequest,
) -> AppResult<Uuid> {
    Ok(tx
        .query_one(
            "INSERT INTO resource_service.crawl_runs(trigger_kind, requested_by, status, started_at)
             VALUES ('scheduler', $1, 'running', now())
             RETURNING id",
            &[&request.requested_by],
        )
        .await?
        .get("id"))
}

async fn load_schedulable_seeds(
    tx: &tokio_postgres::Transaction<'_>,
    request: &ScheduleCrawlRequest,
) -> AppResult<Vec<SchedulableSeed>> {
    let limit = request.limit.unwrap_or(50).clamp(1, 500);
    let rows = tx
        .query(
            "SELECT id, source_id, seed_value, priority, metadata
             FROM resource_service.crawl_seeds
             WHERE enabled = true
               AND ($1 IS NULL OR source_id = $1)
             ORDER BY priority DESC, created_at ASC
             LIMIT $2",
            &[&request.source_site_id, &limit],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| SchedulableSeed {
            id: row.get("id"),
            source_id: row.get("source_id"),
            seed_url: row.get("seed_value"),
            priority: row.get("priority"),
            metadata: row.get("metadata"),
        })
        .collect())
}

async fn enqueue_seed_job(
    tx: &tokio_postgres::Transaction<'_>,
    crawl_run_id: Uuid,
    seed: &SchedulableSeed,
) -> AppResult<Uuid> {
    Ok(tx
        .query_one(
            "SELECT resource_service.enqueue_crawl_job($1, $2, $3, $4, $5, 0, 'fetch', now(), NULL, $6)",
            &[
                &seed.seed_url,
                &seed.source_id,
                &Some(seed.id),
                &Some(crawl_run_id),
                &seed.priority,
                &Json(&seed.metadata),
            ],
        )
        .await?
        .get(0))
}

async fn mark_seed_enqueued(tx: &tokio_postgres::Transaction<'_>, seed_id: Uuid) -> AppResult<()> {
    tx.execute(
        "UPDATE resource_service.crawl_seeds
         SET last_enqueued_at = now()
         WHERE id = $1",
        &[&seed_id],
    )
    .await?;
    Ok(())
}

async fn finish_crawl_run(
    tx: &tokio_postgres::Transaction<'_>,
    crawl_run_id: Uuid,
    total_jobs: usize,
) -> AppResult<()> {
    let total_jobs = total_jobs as i32;
    tx.execute(
        "UPDATE resource_service.crawl_runs
         SET stats = jsonb_build_object('total_jobs', $2),
             status = 'succeeded',
             finished_at = now()
         WHERE id = $1",
        &[&crawl_run_id, &total_jobs],
    )
    .await?;
    Ok(())
}
