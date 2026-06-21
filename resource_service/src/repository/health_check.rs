use serde_json::{Map, Value, json};

use crate::{AppResult, models::HealthResponse};

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn health_check(&self) -> AppResult<HealthResponse> {
        let client = self.pool.get().await?;
        let database_ok: bool = client.query_one("SELECT 1 = 1", &[]).await?.get(0);
        let vector_ok = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')",
                &[],
            )
            .await
            .map(|row| row.get::<_, bool>(0))
            .unwrap_or(false);
        let table_ok: bool = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'resource_service'
                      AND table_name = 'resources'
                )",
                &[],
            )
            .await?
            .get(0);

        let status = self.pool.status();
        let mut checks = Map::new();
        checks.insert("postgres".to_string(), Value::Bool(database_ok));
        checks.insert("vectorExtension".to_string(), Value::Bool(vector_ok));
        checks.insert("requiredTables".to_string(), Value::Bool(table_ok));
        checks.insert(
            "pool".to_string(),
            json!({
                "maxSize": status.max_size,
                "currentSize": status.size,
                "idleSize": status.available,
            }),
        );

        Ok(HealthResponse {
            status: if database_ok && table_ok {
                "ready".to_string()
            } else {
                "degraded".to_string()
            },
            checks,
        })
    }

    pub async fn metrics_snapshot(&self) -> AppResult<serde_json::Value> {
        let client = self.pool.get().await?;
        let status = self.pool.status();
        let metrics = json!({
            "crawlJobsQueued": count_one(&client, "SELECT count(*)::bigint FROM resource_service.crawl_jobs WHERE status = 'pending'").await?,
            "crawlJobsClaimed": count_one(&client, "SELECT count(*)::bigint FROM resource_service.crawl_jobs WHERE status = 'running'").await?,
            "crawlJobsCompleted": count_one(&client, "SELECT count(*)::bigint FROM resource_service.crawl_jobs WHERE status = 'succeeded'").await?,
            "crawlJobsFailed": count_one(&client, "SELECT count(*)::bigint FROM resource_service.crawl_jobs WHERE status = 'failed'").await?,
            "fetchArtifactsCreated": count_one(&client, "SELECT count(*)::bigint FROM resource_service.fetch_artifacts").await?,
            "extractSuccessTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.fetch_artifacts WHERE metadata->'extraction'->>'resourceVersionId' IS NOT NULL").await?,
            "extractFailedTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.fetch_artifacts WHERE http_status >= 400").await?,
            "chunksCreatedTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.resource_chunks").await?,
            "embeddingPendingTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.resource_chunks c JOIN resource_service.resources r ON r.id = c.resource_id LEFT JOIN resource_service.resource_chunk_embeddings e ON e.chunk_id = c.id WHERE r.status = 'active' AND e.chunk_id IS NULL").await?,
            "embeddingSuccessTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.resource_chunk_embeddings").await?,
            "embeddingFailedTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.outbox_events WHERE event_type = 'embedding.failed'").await?,
            "searchRequestsTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.retrieval_queries").await?,
            "coverageGoodTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.outbox_events WHERE event_type = 'coverage.good'").await?,
            "coveragePartialTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.outbox_events WHERE event_type = 'coverage.partial'").await?,
            "coveragePoorTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.outbox_events WHERE event_type = 'coverage.poor'").await?,
            "gapsOpenTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.resource_gaps WHERE status IN ('pending', 'running')").await?,
            "researchCandidatesPendingTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.research_candidates WHERE selected = false AND reject_reason IS NULL").await?,
            "outboxPendingTotal": count_one(&client, "SELECT count(*)::bigint FROM resource_service.outbox_events WHERE status = 'pending'").await?,
            "pool": {
                "maxSize": status.max_size,
                "currentSize": status.size,
                "idleSize": status.available
            }
        });
        Ok(metrics)
    }

    pub async fn count_outbox_events(
        &self,
        event_type: &str,
        aggregate_id: Option<uuid::Uuid>,
    ) -> AppResult<i64> {
        let client = self.pool.get().await?;
        let count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM resource_service.outbox_events
                 WHERE event_type = $1
                   AND ($2::uuid IS NULL OR aggregate_id = $2)",
                &[&event_type, &aggregate_id],
            )
            .await?
            .get(0);
        Ok(count)
    }
}

async fn count_one(client: &tokio_postgres::Client, sql: &str) -> AppResult<i64> {
    Ok(client.query_one(sql, &[]).await?.get(0))
}
