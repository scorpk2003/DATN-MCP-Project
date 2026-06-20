use crate::{
    AppResult,
    models::{RecommendRequest, RecommendedResource},
};

use super::{ResourceRepository, mappers::row_to_recommended_resource};

impl ResourceRepository {
    pub async fn recommend(
        &self,
        request: &RecommendRequest,
    ) -> AppResult<Vec<RecommendedResource>> {
        let client = self.pool.get().await?;
        let limit = request.max_resources.unwrap_or(8).clamp(1, 20);
        let rows = client
            .query(
                "SELECT r.id AS resource_id, r.canonical_url, r.title, r.difficulty::text,
                        r.quality_score::double precision, r.authority_score::double precision,
                        r.is_official,
                        COALESCE(
                          r.metadata->'enrichment'->'resourceRoles'->>0,
                          CASE WHEN r.is_official THEN 'official_reference' ELSE 'primary_learning' END
                        ) AS role,
                        array_remove(array_agg(c.id ORDER BY c.chunk_index), NULL) AS chunk_ids
                 FROM resource_service.resources r
                 LEFT JOIN resource_service.resource_chunks c ON c.resource_id = r.id
                 LEFT JOIN resource_service.resource_topics rt ON rt.resource_id = r.id
                 LEFT JOIN resource_service.topics t ON t.id = rt.topic_id
                 WHERE r.status = 'active'
                   AND (
                     r.search_vector @@ websearch_to_tsquery('english', $1)
                     OR r.title ILIKE '%' || $1 || '%'
                     OR r.summary ILIKE '%' || $1 || '%'
                     OR t.slug::text ILIKE '%' || $1 || '%'
                     OR t.name ILIKE '%' || $1 || '%'
                     OR r.metadata->'enrichment'->>'summary' ILIKE '%' || $1 || '%'
                   )
                   AND ($2 IS NULL OR r.difficulty::text IN ($2, 'mixed', 'unknown'))
                 GROUP BY r.id
                 ORDER BY
                   r.is_official DESC,
                   r.quality_score DESC,
                   r.authority_score DESC
                 LIMIT $3",
                &[&request.topic, &request.level, &limit],
            )
            .await?;
        Ok(rows.iter().map(row_to_recommended_resource).collect())
    }
}
