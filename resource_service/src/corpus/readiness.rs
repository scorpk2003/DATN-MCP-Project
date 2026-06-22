use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tokio_postgres::types::ToSql;

use crate::{
    AppResult, ResourceService,
    corpus::{EvaluationTopic, EvaluationTopicManifest, classify_triage},
    repository::normalize_query,
};

#[derive(Debug, Serialize)]
pub struct CorpusReadinessReport {
    pub summary: CorpusReadinessSummary,
    pub topics: Vec<TopicReadiness>,
}

#[derive(Debug, Serialize)]
pub struct CorpusReadinessSummary {
    #[serde(rename = "topicsTotal")]
    pub topics_total: usize,
    #[serde(rename = "topicsReady")]
    pub topics_ready: usize,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TopicReadiness {
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "topicName")]
    pub topic_name: String,
    #[serde(rename = "expectedDomains")]
    pub expected_domains: Vec<String>,
    #[serde(rename = "requiredResourceTypes")]
    pub required_resource_types: Vec<String>,
    #[serde(rename = "foundResources")]
    pub found_resources: i64,
    #[serde(rename = "officialResources")]
    pub official_resources: i64,
    #[serde(rename = "resourcesByType")]
    pub resources_by_type: HashMap<String, i64>,
    #[serde(rename = "versionsCount")]
    pub versions_count: i64,
    #[serde(rename = "chunksCount")]
    pub chunks_count: i64,
    #[serde(rename = "embeddedChunksCount")]
    pub embedded_chunks_count: i64,
    #[serde(rename = "embeddedRatio")]
    pub embedded_ratio: f64,
    #[serde(rename = "enrichedResourcesCount")]
    pub enriched_resources_count: i64,
    #[serde(rename = "aliasesMatched")]
    pub aliases_matched: Vec<String>,
    #[serde(rename = "readyForEval")]
    pub ready_for_eval: bool,
    #[serde(rename = "missingReasons")]
    pub missing_reasons: Vec<String>,
    pub triage: Option<crate::corpus::FailureTriage>,
}

pub async fn build_readiness_report(
    service: &ResourceService,
    manifest: &EvaluationTopicManifest,
) -> AppResult<CorpusReadinessReport> {
    let mut topics = Vec::new();
    for topic in &manifest.topics {
        topics.push(topic_readiness(service, topic).await?);
    }
    let topics_ready = topics.iter().filter(|topic| topic.ready_for_eval).count();
    Ok(CorpusReadinessReport {
        summary: CorpusReadinessSummary {
            topics_total: topics.len(),
            topics_ready,
            status: if topics_ready == topics.len() {
                "pass".to_string()
            } else {
                "fail_corpus_not_ready".to_string()
            },
        },
        topics,
    })
}

async fn topic_readiness(
    service: &ResourceService,
    topic: &EvaluationTopic,
) -> AppResult<TopicReadiness> {
    let client = service.repository.pool.get().await?;
    let aliases = aliases(topic);
    let domains = &topic.expected_official_domains;
    let rows = client
        .query(
            "WITH matched AS (
                SELECT DISTINCT r.id,
                       COALESCE(r.metadata->'enrichment'->'resourceRoles'->>0,
                         CASE WHEN r.is_official THEN 'official_reference' ELSE 'primary_learning' END
                       ) AS role,
                       r.metadata,
                       r.title,
                       r.canonical_url
                FROM resource_service.resources r
                LEFT JOIN resource_service.resource_topics rt ON rt.resource_id = r.id
                LEFT JOIN resource_service.topics t ON t.id = rt.topic_id
                WHERE r.status = 'active'
                  AND r.is_official = true
                  AND EXISTS (
                    SELECT 1 FROM unnest($1::text[]) domain
                    WHERE r.canonical_url ILIKE '%' || domain || '%'
                  )
                  AND (
                    r.title ILIKE ANY($2::text[])
                    OR r.summary ILIKE ANY($2::text[])
                    OR r.canonical_url ILIKE ANY($2::text[])
                    OR t.name ILIKE ANY($2::text[])
                    OR t.slug::text ILIKE ANY($2::text[])
                    OR r.metadata::text ILIKE ANY($2::text[])
                  )
              )
              SELECT
                count(DISTINCT m.id)::bigint AS official_resources,
                count(DISTINCT rv.id)::bigint AS versions_count,
                count(DISTINCT c.id)::bigint AS chunks_count,
                count(DISTINCT e.chunk_id)::bigint AS embedded_chunks_count,
                count(DISTINCT m.id) FILTER (WHERE m.metadata ? 'enrichment')::bigint AS enriched_resources_count,
                m.role
              FROM matched m
              LEFT JOIN resource_service.resource_versions rv ON rv.resource_id = m.id
              LEFT JOIN resource_service.resource_chunks c ON c.resource_id = m.id
              LEFT JOIN resource_service.resource_chunk_embeddings e ON e.chunk_id = c.id
              GROUP BY m.role",
            &[&domains as &(dyn ToSql + Sync), &like_patterns(&aliases)],
        )
        .await?;

    let mut official_resources = 0_i64;
    let mut versions_count = 0_i64;
    let mut chunks_count = 0_i64;
    let mut embedded_chunks_count = 0_i64;
    let mut enriched_resources_count = 0_i64;
    let mut resources_by_type = HashMap::new();
    for row in rows {
        let role: Option<String> = row.get("role");
        let role = role.unwrap_or_else(|| "official_reference".to_string());
        let resources: i64 = row.get("official_resources");
        official_resources += resources;
        versions_count += row.get::<_, i64>("versions_count");
        chunks_count += row.get::<_, i64>("chunks_count");
        embedded_chunks_count += row.get::<_, i64>("embedded_chunks_count");
        enriched_resources_count += row.get::<_, i64>("enriched_resources_count");
        *resources_by_type.entry(role).or_insert(0) += resources;
    }

    let aliases_matched = aliases_matched(service, topic, &aliases).await?;
    let embedded_ratio = if chunks_count == 0 {
        0.0
    } else {
        embedded_chunks_count as f64 / chunks_count as f64
    };
    let mut missing = Vec::new();
    if official_resources < topic.expected_min_resources {
        missing.push("missing_official_resource".to_string());
    }
    if versions_count == 0 {
        missing.push("missing_versions".to_string());
    }
    if chunks_count < topic.expected_min_chunks {
        missing.push("missing_chunks".to_string());
    }
    if chunks_count > 0 && embedded_ratio < 0.95 {
        missing.push("missing_embeddings".to_string());
    }
    if enriched_resources_count == 0 {
        missing.push("enrichment_missing".to_string());
    }
    if aliases_matched.is_empty() {
        missing.push("topic_alias_missing".to_string());
    }
    if !topic.allow_partial_if_missing_types {
        for required in &topic.required_resource_types {
            if resources_by_type.get(required).copied().unwrap_or(0) == 0 {
                missing.push(format!("missing_required_type:{required}"));
            }
        }
    }
    let ready_for_eval = missing.is_empty();
    let triage = (!ready_for_eval).then(|| classify_triage(&topic.topic_id, &missing));

    Ok(TopicReadiness {
        topic_id: topic.topic_id.clone(),
        topic_name: topic.topic_name.clone(),
        expected_domains: topic.expected_official_domains.clone(),
        required_resource_types: topic.required_resource_types.clone(),
        found_resources: official_resources,
        official_resources,
        resources_by_type,
        versions_count,
        chunks_count,
        embedded_chunks_count,
        embedded_ratio,
        enriched_resources_count,
        aliases_matched,
        ready_for_eval,
        missing_reasons: missing,
        triage,
    })
}

async fn aliases_matched(
    service: &ResourceService,
    topic: &EvaluationTopic,
    aliases: &[String],
) -> AppResult<Vec<String>> {
    let client = service.repository.pool.get().await?;
    let domains = &topic.expected_official_domains;
    let patterns = like_patterns(aliases);
    let rows = client
        .query(
            "SELECT DISTINCT alias
             FROM unnest($1::text[]) alias
             WHERE EXISTS (
               SELECT 1
               FROM resource_service.resources r
               LEFT JOIN resource_service.resource_versions rv ON rv.resource_id = r.id
               LEFT JOIN resource_service.resource_chunks c ON c.resource_id = r.id
               WHERE r.status = 'active'
                 AND r.is_official = true
                 AND EXISTS (
                   SELECT 1 FROM unnest($2::text[]) domain
                   WHERE r.canonical_url ILIKE '%' || domain || '%'
                 )
                 AND (
                   r.title ILIKE '%' || alias || '%'
                   OR r.summary ILIKE '%' || alias || '%'
                   OR r.canonical_url ILIKE '%' || alias || '%'
                   OR c.heading_path::text ILIKE '%' || alias || '%'
                   OR c.content ILIKE '%' || alias || '%'
                 )
             )
             OR alias ILIKE ANY($3::text[])",
            &[&aliases, &domains as &(dyn ToSql + Sync), &patterns],
        )
        .await?;
    let mut unique = HashSet::new();
    Ok(rows
        .iter()
        .filter_map(|row| {
            let alias: String = row.get("alias");
            unique.insert(alias.clone()).then_some(alias)
        })
        .collect())
}

fn aliases(topic: &EvaluationTopic) -> Vec<String> {
    let mut values = vec![topic.topic_name.clone(), topic.topic_id.replace('_', " ")];
    values.extend(topic.aliases.clone());
    values
        .into_iter()
        .map(|value| normalize_query(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn like_patterns(values: &[String]) -> Vec<String> {
    values.iter().map(|value| format!("%{value}%")).collect()
}
