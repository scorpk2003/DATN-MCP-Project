use serde_json::Value;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{
    AppResult,
    models::{CoverageStatus, SearchRequest, SearchResult},
};

use super::{ResourceRepository, mappers::row_to_search_result};

pub(crate) struct SearchEmbedding {
    pub model_id: Uuid,
    pub vector_literal: String,
}

impl ResourceRepository {
    pub async fn search_chunks(&self, request: &SearchRequest) -> AppResult<Vec<SearchResult>> {
        self.search_chunks_with_embedding(request, None).await
    }

    pub(crate) async fn search_chunks_with_embedding(
        &self,
        request: &SearchRequest,
        embedding: Option<SearchEmbedding>,
    ) -> AppResult<Vec<SearchResult>> {
        let client = self.pool.get().await?;
        let limit = request.limit.unwrap_or(10).clamp(1, 50) as i32;
        let filters = request.filters.clone();
        let min_quality = filters
            .as_ref()
            .and_then(|f| f.min_quality_score)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let language = filters.as_ref().and_then(|f| f.language.clone());
        let candidate_limit = (limit * 10).max(50);
        let query_vector = embedding.as_ref().map(|value| value.vector_literal.clone());
        let model_id = embedding.as_ref().map(|value| value.model_id);

        let rows = client
            .query(
                "SELECT
                    hs.chunk_id, hs.resource_id, c.version_id, hs.canonical_url, hs.title,
                    hs.heading_path, hs.content, hs.score, hs.vector_score, hs.text_score,
                    hs.quality_score::double precision,
                    hs.freshness_score::double precision,
                    COALESCE(c.metadata->>'content_kind', 'mixed') AS content_kind
                 FROM resource_service.hybrid_search_chunks(
                    $1, CAST($2::text AS resource_service.vector), $3, NULL, NULL, $4, CAST($5::float8 AS numeric), false, $6, $7
                 ) hs
                 JOIN resource_service.resource_chunks c ON c.id = hs.chunk_id",
                &[
                    &request.query,
                    &query_vector,
                    &model_id,
                    &language,
                    &min_quality,
                    &candidate_limit,
                    &limit,
                ],
            )
            .await?;

        Ok(rows.iter().map(row_to_search_result).collect())
    }

    pub async fn create_gap_if_low_results(
        &self,
        requester: &str,
        query: &str,
        result_count: i32,
        min_required: i32,
        context: Value,
    ) -> AppResult<Option<Uuid>> {
        let client = self.pool.get().await?;
        let normalized = normalize_query(query);
        let gap_id: Option<Uuid> = client
            .query_one(
                "SELECT resource_service.create_gap_if_low_results($1, $2, $3, $4, $5, $6)",
                &[
                    &requester,
                    &query,
                    &normalized,
                    &result_count,
                    &min_required,
                    &Json(&context),
                ],
            )
            .await?
            .get(0);
        Ok(gap_id)
    }
}

pub fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn coverage_for_results(
    results_len: usize,
    best_score: f64,
    gap_id: Option<Uuid>,
    missing_types: &[String],
) -> CoverageStatus {
    let missing_types = missing_types.to_vec();
    let status = if results_len >= 5 && best_score >= 0.80 && missing_types.is_empty() {
        "good"
    } else if results_len >= 2 && best_score >= 0.65 {
        "partial"
    } else {
        "poor"
    };
    CoverageStatus {
        status: status.to_string(),
        low_confidence: status == "poor",
        missing_types,
        result_count: results_len as i64,
        best_score,
        gap_id,
    }
}
