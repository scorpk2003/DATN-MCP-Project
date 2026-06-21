use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{Page, PageQuery, SourcePatchRequest, SourceRequest, SourceSite},
};

use super::{
    ResourceRepository,
    mappers::{extract_domain, page, row_to_source},
};

impl ResourceRepository {
    pub async fn create_source(&self, request: &SourceRequest) -> AppResult<SourceSite> {
        let client = self.pool.get().await?;
        let host = extract_domain(&request.base_url)
            .ok_or_else(|| AppError::Validation("baseUrl must be a valid URL".to_string()))?;
        let kind = request.kind.clone().unwrap_or_else(|| "other".to_string());
        let trust_tier = request.trust_tier.unwrap_or(3).clamp(1, 5);
        let language = request
            .language_hint
            .clone()
            .unwrap_or_else(|| "en".to_string());
        let enabled = request.enabled.unwrap_or(true);
        let is_official = request.is_official.unwrap_or(false);
        let crawl_policy = request.crawl_policy.clone().unwrap_or_else(
            || json!({"respect_robots": true, "max_depth": 3, "rate_limit_per_minute": 30}),
        );
        let allowed_paths = request.allowed_paths.clone().unwrap_or_default();
        let blocked_paths = request.blocked_paths.clone().unwrap_or_default();
        let tags = request.tags.clone().unwrap_or_default();

        let row = client
            .query_one(
                "INSERT INTO resource_service.source_sites(
                    name, kind, base_url, host, trust_tier, language_hint, enabled,
                    is_official, crawl_policy, allowed_paths, blocked_paths, tags, notes
                 ) VALUES (
                    $1,
                    CASE $2
                        WHEN 'official_docs' THEN 'official_docs'::resource_service.source_kind
                        WHEN 'specification' THEN 'specification'::resource_service.source_kind
                        WHEN 'repo' THEN 'repo'::resource_service.source_kind
                        WHEN 'paper' THEN 'paper'::resource_service.source_kind
                        WHEN 'course' THEN 'course'::resource_service.source_kind
                        WHEN 'tutorial' THEN 'tutorial'::resource_service.source_kind
                        WHEN 'article' THEN 'article'::resource_service.source_kind
                        WHEN 'blog' THEN 'blog'::resource_service.source_kind
                        WHEN 'qna' THEN 'qna'::resource_service.source_kind
                        WHEN 'video' THEN 'video'::resource_service.source_kind
                        WHEN 'book' THEN 'book'::resource_service.source_kind
                        WHEN 'dataset' THEN 'dataset'::resource_service.source_kind
                        ELSE 'other'::resource_service.source_kind
                    END,
                    $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
                 )
                 RETURNING id, name, kind::text, base_url, host, trust_tier, language_hint,
                           enabled, is_official, crawl_policy, allowed_paths, blocked_paths",
                &[
                    &request.name,
                    &kind,
                    &request.base_url,
                    &host,
                    &trust_tier,
                    &language,
                    &enabled,
                    &is_official,
                    &Json(&crawl_policy),
                    &allowed_paths,
                    &blocked_paths,
                    &tags,
                    &request.notes,
                ],
            )
            .await?;
        Ok(row_to_source(&row))
    }

    pub async fn list_sources(&self, query: &PageQuery) -> AppResult<Page<SourceSite>> {
        let client = self.pool.get().await?;
        let limit = query.limit();
        let offset = query.offset();
        let total: i64 = client
            .query_one(
                "SELECT count(*)::bigint FROM resource_service.source_sites",
                &[],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT id, name, kind::text, base_url, host, trust_tier, language_hint,
                        enabled, is_official, crawl_policy, allowed_paths, blocked_paths
                 FROM resource_service.source_sites
                 ORDER BY updated_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;
        Ok(page(
            rows.iter().map(row_to_source).collect(),
            limit,
            offset,
            total,
        ))
    }

    pub async fn get_source(&self, id: Uuid) -> AppResult<SourceSite> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, name, kind::text, base_url, host, trust_tier, language_hint,
                        enabled, is_official, crawl_policy, allowed_paths, blocked_paths
                 FROM resource_service.source_sites
                 WHERE id = $1",
                &[&id],
            )
            .await?
            .ok_or(AppError::SourceNotFound)?;
        Ok(row_to_source(&row))
    }

    pub(crate) async fn get_source_policy(&self, id: Uuid) -> AppResult<SourceSite> {
        self.get_source(id).await
    }

    pub async fn patch_source(
        &self,
        id: Uuid,
        request: &SourcePatchRequest,
    ) -> AppResult<SourceSite> {
        let client = self.pool.get().await?;
        let crawl_policy = request.crawl_policy.clone().unwrap_or_else(|| json!({}));
        let has_policy = request.crawl_policy.is_some();
        let allowed_paths = request.allowed_paths.clone().unwrap_or_default();
        let has_allowed = request.allowed_paths.is_some();
        let blocked_paths = request.blocked_paths.clone().unwrap_or_default();
        let has_blocked = request.blocked_paths.is_some();
        let affected = client
            .execute(
                "UPDATE resource_service.source_sites
                 SET name = COALESCE($2, name),
                     enabled = COALESCE($3, enabled),
                     crawl_policy = CASE WHEN $4 THEN crawl_policy || $5 ELSE crawl_policy END,
                     allowed_paths = CASE WHEN $6 THEN $7 ELSE allowed_paths END,
                     blocked_paths = CASE WHEN $8 THEN $9 ELSE blocked_paths END,
                     notes = COALESCE($10, notes)
                 WHERE id = $1",
                &[
                    &id,
                    &request.name,
                    &request.enabled,
                    &has_policy,
                    &Json(&crawl_policy),
                    &has_allowed,
                    &allowed_paths,
                    &has_blocked,
                    &blocked_paths,
                    &request.notes,
                ],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::SourceNotFound);
        }
        self.get_source(id).await
    }
}
