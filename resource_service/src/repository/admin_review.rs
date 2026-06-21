use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{AdminDashboardSummary, AdminResourceActionRequest, AdminResourceActionResponse},
};

use super::ResourceRepository;

impl ResourceRepository {
    pub async fn admin_dashboard_summary(&self) -> AppResult<AdminDashboardSummary> {
        let client = self.pool.get().await?;
        let failed_jobs = count_one(
            &client,
            "SELECT count(*)::bigint FROM resource_service.crawl_jobs WHERE status = 'failed'",
        )
        .await?;
        let open_gaps = count_one(
            &client,
            "SELECT count(*)::bigint FROM resource_service.resource_gaps WHERE status IN ('pending', 'running')",
        )
        .await?;
        let pending_candidates = count_one(
            &client,
            "SELECT count(*)::bigint FROM resource_service.research_candidates WHERE selected = false AND reject_reason IS NULL",
        )
        .await?;
        let resources_need_review = count_one(
            &client,
            "SELECT count(DISTINCT resource_id)::bigint FROM resource_service.resource_issues WHERE status = 'open'",
        )
        .await?;
        let outdated_resources = count_one(
            &client,
            "SELECT count(*)::bigint FROM resource_service.resources WHERE status = 'stale'",
        )
        .await?;
        let last_crawl_run_status = client
            .query_opt(
                "SELECT status::text FROM resource_service.crawl_runs ORDER BY created_at DESC LIMIT 1",
                &[],
            )
            .await?
            .map(|row| row.get("status"));

        Ok(AdminDashboardSummary {
            failed_jobs,
            open_gaps,
            pending_candidates,
            resources_need_review,
            outdated_resources,
            last_crawl_run_status,
        })
    }

    pub async fn mark_resource_outdated(
        &self,
        resource_id: Uuid,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.update_resource_status_with_issue(resource_id, "stale", "outdated", request)
            .await
    }

    pub async fn mark_resource_needs_review(
        &self,
        resource_id: Uuid,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.update_resource_status_with_issue(resource_id, "candidate", "needs_review", request)
            .await
    }

    pub async fn boost_resource_quality(
        &self,
        resource_id: Uuid,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.adjust_resource_quality(resource_id, true, request)
            .await
    }

    pub async fn deboost_resource_quality(
        &self,
        resource_id: Uuid,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.adjust_resource_quality(resource_id, false, request)
            .await
    }

    async fn update_resource_status_with_issue(
        &self,
        resource_id: Uuid,
        status: &str,
        issue_type: &str,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(
                "UPDATE resource_service.resources
                 SET status = CASE $2
                    WHEN 'candidate' THEN 'candidate'::resource_service.resource_status
                    WHEN 'stale' THEN 'stale'::resource_service.resource_status
                    ELSE status
                 END
                 WHERE id = $1
                 RETURNING id, status::text, quality_score::double precision",
                &[&resource_id, &status],
            )
            .await?
            .ok_or(AppError::ResourceNotFound)?;
        insert_resource_issue(&tx, resource_id, issue_type, request).await?;
        insert_quality_event(&tx, resource_id, issue_type, request, json!({})).await?;
        tx.commit().await?;
        Ok(AdminResourceActionResponse {
            resource_id: row.get("id"),
            action: issue_type.to_string(),
            status: row.get("status"),
            quality_score: row.get("quality_score"),
        })
    }

    async fn adjust_resource_quality(
        &self,
        resource_id: Uuid,
        boost: bool,
        request: &AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        let sql = if boost {
            "UPDATE resource_service.resources
             SET quality_score = LEAST(1.0000, quality_score + 0.1000)
             WHERE id = $1
             RETURNING id, status::text, quality_score::double precision"
        } else {
            "UPDATE resource_service.resources
             SET quality_score = GREATEST(0.0000, quality_score - 0.1000)
             WHERE id = $1
             RETURNING id, status::text, quality_score::double precision"
        };
        let row = tx
            .query_opt(sql, &[&resource_id])
            .await?
            .ok_or(AppError::ResourceNotFound)?;
        let action = if boost {
            "quality_boost"
        } else {
            "quality_deboost"
        };
        insert_quality_event(
            &tx,
            resource_id,
            action,
            request,
            json!({"adjustment": if boost { 0.1 } else { -0.1 }}),
        )
        .await?;
        tx.commit().await?;
        Ok(AdminResourceActionResponse {
            resource_id: row.get("id"),
            action: action.to_string(),
            status: row.get("status"),
            quality_score: row.get("quality_score"),
        })
    }
}

async fn count_one(client: &tokio_postgres::Client, sql: &str) -> AppResult<i64> {
    Ok(client.query_one(sql, &[]).await?.get(0))
}

async fn insert_resource_issue(
    tx: &tokio_postgres::Transaction<'_>,
    resource_id: Uuid,
    issue_type: &str,
    request: &AdminResourceActionRequest,
) -> AppResult<()> {
    let metadata = json!({
        "actorId": request.actor_id,
        "reason": request.reason,
    });
    tx.execute(
        "INSERT INTO resource_service.resource_issues(resource_id, issue_type, description, metadata)
         VALUES ($1, $2, $3, $4)",
        &[&resource_id, &issue_type, &request.reason, &Json(&metadata)],
    )
    .await?;
    Ok(())
}

async fn insert_quality_event(
    tx: &tokio_postgres::Transaction<'_>,
    resource_id: Uuid,
    action: &str,
    request: &AdminResourceActionRequest,
    scores: serde_json::Value,
) -> AppResult<()> {
    let notes = request.reason.clone();
    let evaluator_name = request
        .actor_id
        .clone()
        .unwrap_or_else(|| "admin".to_string());
    tx.execute(
        "INSERT INTO resource_service.resource_quality_events(
            resource_id, evaluator_type, evaluator_name, scores, notes
         ) VALUES ($1, 'admin', $2, $3, $4)",
        &[
            &resource_id,
            &evaluator_name,
            &Json(&json!({"action": action, "scores": scores})),
            &notes,
        ],
    )
    .await?;
    Ok(())
}
