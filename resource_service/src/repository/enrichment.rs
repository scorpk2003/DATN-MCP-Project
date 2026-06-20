use serde_json::json;
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{AppError, AppResult};

use super::ResourceRepository;

pub(crate) struct EnrichmentInput {
    pub resource_id: Uuid,
    pub version_id: Uuid,
    pub title: String,
    pub url: String,
    pub source_kind: Option<String>,
    pub is_official: bool,
    pub chunks: Vec<EnrichmentChunkInput>,
}

pub(crate) struct EnrichmentChunkInput {
    pub chunk_id: Uuid,
    pub heading_path: Vec<String>,
    pub content: String,
    pub content_kind: String,
}

pub(crate) struct EnrichmentWrite {
    pub summary: String,
    pub difficulty: String,
    pub topic_matches: Vec<EnrichmentWriteMatch>,
    pub concept_matches: Vec<EnrichmentWriteMatch>,
    pub resource_roles: Vec<String>,
    pub prerequisites: Vec<String>,
    pub learning_outcomes: Vec<String>,
    pub confidence: f64,
}

pub(crate) struct EnrichmentWriteMatch {
    pub slug: String,
    pub name: String,
    pub evidence_chunk_ids: Vec<Uuid>,
}

impl ResourceRepository {
    pub(crate) async fn load_enrichment_input(
        &self,
        resource_id: Uuid,
        version_id: Option<Uuid>,
    ) -> AppResult<EnrichmentInput> {
        let client = self.pool.get().await?;
        let resource = client
            .query_opt(
                "SELECT r.id, r.title, r.canonical_url, r.is_official, s.kind::text AS source_kind
                 FROM resource_service.resources r
                 LEFT JOIN resource_service.source_sites s ON s.id = r.source_id
                 WHERE r.id = $1",
                &[&resource_id],
            )
            .await?
            .ok_or(AppError::ResourceNotFound)?;
        let version_id = match version_id {
            Some(version_id) => version_id,
            None => client
                .query_one(
                    "SELECT id
                         FROM resource_service.resource_versions
                         WHERE resource_id = $1
                         ORDER BY version_no DESC
                         LIMIT 1",
                    &[&resource_id],
                )
                .await?
                .get("id"),
        };
        let rows = client
            .query(
                "SELECT id, heading_path, content, COALESCE(metadata->>'content_kind', 'mixed') AS content_kind
                 FROM resource_service.resource_chunks
                 WHERE resource_id = $1 AND version_id = $2
                 ORDER BY chunk_index ASC",
                &[&resource_id, &version_id],
            )
            .await?;

        Ok(EnrichmentInput {
            resource_id,
            version_id,
            title: resource.get("title"),
            url: resource.get("canonical_url"),
            source_kind: resource.get("source_kind"),
            is_official: resource.get("is_official"),
            chunks: rows
                .iter()
                .map(|row| EnrichmentChunkInput {
                    chunk_id: row.get("id"),
                    heading_path: row.get("heading_path"),
                    content: row.get("content"),
                    content_kind: row.get("content_kind"),
                })
                .collect(),
        })
    }

    pub(crate) async fn write_enrichment(
        &self,
        resource_id: Uuid,
        enrichment: &EnrichmentWrite,
    ) -> AppResult<()> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        for topic in &enrichment.topic_matches {
            let topic_id = upsert_topic(&tx, &topic.slug, &topic.name).await?;
            tx.execute(
                "INSERT INTO resource_service.resource_topics(resource_id, topic_id, assigned_by)
                 VALUES ($1, $2, 'rule_based_enrichment_v1')
                 ON CONFLICT (resource_id, topic_id) DO UPDATE
                 SET assigned_by = EXCLUDED.assigned_by",
                &[&resource_id, &topic_id],
            )
            .await?;
        }
        for concept in &enrichment.concept_matches {
            let concept_id = upsert_concept(&tx, &concept.slug, &concept.name).await?;
            for chunk_id in &concept.evidence_chunk_ids {
                tx.execute(
                    "INSERT INTO resource_service.chunk_concepts(chunk_id, concept_id, assigned_by)
                     VALUES ($1, $2, 'rule_based_enrichment_v1')
                     ON CONFLICT (chunk_id, concept_id) DO UPDATE
                     SET assigned_by = EXCLUDED.assigned_by",
                    &[chunk_id, &concept_id],
                )
                .await?;
            }
        }

        let metadata = json!({
            "enrichment": {
                "version": "rule_based_v1",
                "summary": enrichment.summary,
                "resourceRoles": enrichment.resource_roles,
                "prerequisites": enrichment.prerequisites,
                "learningOutcomes": enrichment.learning_outcomes,
                "confidence": enrichment.confidence
            }
        });
        tx.execute(
            "UPDATE resource_service.resources
             SET difficulty = COALESCE(
                    CASE $2
                        WHEN 'beginner' THEN 'beginner'::resource_service.difficulty_level
                        WHEN 'intermediate' THEN 'intermediate'::resource_service.difficulty_level
                        WHEN 'advanced' THEN 'advanced'::resource_service.difficulty_level
                        WHEN 'expert' THEN 'expert'::resource_service.difficulty_level
                        ELSE 'unknown'::resource_service.difficulty_level
                    END,
                    difficulty
                 ),
                 metadata = metadata || $3
             WHERE id = $1",
            &[&resource_id, &enrichment.difficulty, &Json(&metadata)],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

async fn upsert_topic(
    tx: &tokio_postgres::Transaction<'_>,
    slug: &str,
    name: &str,
) -> AppResult<i64> {
    Ok(tx
        .query_one(
            "INSERT INTO resource_service.topics(slug, name, domain, metadata)
             VALUES ($1, $2, 'software_engineering', $3)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
            &[
                &slug,
                &name,
                &Json(&json!({"source": "rule_based_enrichment_v1"})),
            ],
        )
        .await?
        .get("id"))
}

async fn upsert_concept(
    tx: &tokio_postgres::Transaction<'_>,
    slug: &str,
    name: &str,
) -> AppResult<i64> {
    Ok(tx
        .query_one(
            "INSERT INTO resource_service.concepts(slug, name, metadata)
             VALUES ($1, $2, $3)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
            &[
                &slug,
                &name,
                &Json(&json!({"source": "rule_based_enrichment_v1"})),
            ],
        )
        .await?
        .get("id"))
}
