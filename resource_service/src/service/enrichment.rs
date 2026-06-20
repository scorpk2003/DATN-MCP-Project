use uuid::Uuid;

use crate::{
    AppResult,
    models::{EnrichResourceRequest, EnrichResourceResponse, EnrichmentMatch},
    repository::{
        enrichment::{EnrichmentInput, EnrichmentWrite, EnrichmentWriteMatch},
        normalize_query,
    },
};

use super::ResourceService;

impl ResourceService {
    pub async fn enrich_resource(
        &self,
        resource_id: Uuid,
        request: EnrichResourceRequest,
    ) -> AppResult<EnrichResourceResponse> {
        let input = self
            .repository
            .load_enrichment_input(resource_id, request.resource_version_id)
            .await?;
        let enrichment = classify_resource(&input);
        self.repository
            .write_enrichment(resource_id, &enrichment)
            .await?;

        Ok(EnrichResourceResponse {
            resource_id: input.resource_id,
            resource_version_id: input.version_id,
            summary: enrichment.summary,
            difficulty: enrichment.difficulty,
            topics: enrichment
                .topic_matches
                .into_iter()
                .map(write_match_to_response)
                .collect(),
            concepts: enrichment
                .concept_matches
                .into_iter()
                .map(write_match_to_response)
                .collect(),
            prerequisites: enrichment.prerequisites,
            learning_outcomes: enrichment.learning_outcomes,
            resource_roles: enrichment.resource_roles,
            confidence: enrichment.confidence,
        })
    }
}

fn classify_resource(input: &EnrichmentInput) -> EnrichmentWrite {
    let haystack = build_haystack(input);
    let topic_matches = classify_topics(&haystack, input);
    let concept_matches = classify_concepts(&haystack, input);
    let difficulty = estimate_difficulty(&haystack);
    let resource_roles = classify_roles(&haystack, input);
    let summary = build_summary(input, &topic_matches);
    let learning_outcomes = build_learning_outcomes(&topic_matches, &concept_matches);
    let prerequisites = build_prerequisites(&difficulty, &topic_matches);
    let confidence = confidence(
        topic_matches.len(),
        concept_matches.len(),
        input.chunks.len(),
    );

    EnrichmentWrite {
        summary,
        difficulty,
        topic_matches,
        concept_matches,
        resource_roles,
        prerequisites,
        learning_outcomes,
        confidence,
    }
}

fn classify_topics(text: &str, input: &EnrichmentInput) -> Vec<EnrichmentWriteMatch> {
    let rules = [
        (
            "postgresql",
            "PostgreSQL",
            &["postgresql", "postgres", "sql", "index", "transaction"][..],
        ),
        ("react", "React", &["react", "jsx", "useeffect", "hook"][..]),
        (
            "javascript",
            "JavaScript",
            &["javascript", "promise", "async", "typescript", "node.js"][..],
        ),
        (
            "kubernetes",
            "Kubernetes",
            &["kubernetes", "k8s", "pod", "deployment"][..],
        ),
        (
            "docker",
            "Docker",
            &["docker", "dockerfile", "container"][..],
        ),
        (
            "databases",
            "Databases",
            &["database", "query", "schema", "isolation"][..],
        ),
        (
            "backend-engineering",
            "Backend Engineering",
            &["api", "http", "service", "backend"][..],
        ),
        (
            "frontend-engineering",
            "Frontend Engineering",
            &["html", "css", "dom", "frontend"][..],
        ),
        (
            "security",
            "Security",
            &["security", "auth", "xss", "csrf", "cryptography"][..],
        ),
    ];
    let mut matches = Vec::new();
    for (slug, name, keywords) in rules {
        if keywords.iter().any(|keyword| text.contains(keyword)) {
            matches.push(EnrichmentWriteMatch {
                slug: slug.to_string(),
                name: name.to_string(),
                evidence_chunk_ids: evidence_chunks(input, keywords),
            });
        }
    }
    if matches.is_empty() {
        matches.push(EnrichmentWriteMatch {
            slug: "software-engineering".to_string(),
            name: "Software Engineering".to_string(),
            evidence_chunk_ids: input
                .chunks
                .iter()
                .take(3)
                .map(|chunk| chunk.chunk_id)
                .collect(),
        });
    }
    matches
}

fn classify_concepts(text: &str, input: &EnrichmentInput) -> Vec<EnrichmentWriteMatch> {
    let rules = [
        (
            "btree-index",
            "B-tree Index",
            &["b-tree", "btree", "index"][..],
        ),
        (
            "database-transaction",
            "Database Transaction",
            &["transaction", "commit", "rollback"][..],
        ),
        (
            "query-planning",
            "Query Planning",
            &["query planner", "explain", "execution plan"][..],
        ),
        (
            "react-useeffect",
            "React useEffect",
            &["useeffect", "effect cleanup", "cleanup function"][..],
        ),
        ("promise", "Promise", &["promise", "async", "await"][..]),
        (
            "api-reference",
            "API Reference",
            &["api", "reference", "method", "parameter"][..],
        ),
        (
            "container-image",
            "Container Image",
            &["image", "dockerfile", "container"][..],
        ),
    ];
    rules
        .into_iter()
        .filter(|(_, _, keywords)| keywords.iter().any(|keyword| text.contains(keyword)))
        .map(|(slug, name, keywords)| EnrichmentWriteMatch {
            slug: slug.to_string(),
            name: name.to_string(),
            evidence_chunk_ids: evidence_chunks(input, keywords),
        })
        .collect()
}

fn estimate_difficulty(text: &str) -> String {
    let advanced = [
        "internals",
        "optimization",
        "distributed",
        "isolation",
        "performance",
    ];
    let beginner = [
        "introduction",
        "getting started",
        "basics",
        "tutorial",
        "beginner",
    ];
    if advanced.iter().any(|keyword| text.contains(keyword)) {
        "advanced".to_string()
    } else if beginner.iter().any(|keyword| text.contains(keyword)) {
        "beginner".to_string()
    } else {
        "intermediate".to_string()
    }
}

fn classify_roles(text: &str, input: &EnrichmentInput) -> Vec<String> {
    let mut roles = Vec::new();
    if input.is_official || input.source_kind.as_deref() == Some("official_docs") {
        roles.push("official_reference".to_string());
    }
    if text.contains("exercise") || text.contains("practice") {
        roles.push("practice".to_string());
    }
    if text.contains("project") || text.contains("build ") {
        roles.push("project".to_string());
    }
    if text.contains("internals") || text.contains("deep dive") || text.contains("performance") {
        roles.push("deep_dive".to_string());
    }
    if text.contains("troubleshooting") || text.contains("common error") || text.contains("debug") {
        roles.push("troubleshooting".to_string());
    }
    if roles.is_empty() || text.contains("tutorial") || text.contains("guide") {
        roles.push("primary_learning".to_string());
    }
    roles.sort();
    roles.dedup();
    roles
}

fn build_summary(input: &EnrichmentInput, topics: &[EnrichmentWriteMatch]) -> String {
    let topic_names = topics
        .iter()
        .take(3)
        .map(|topic| topic.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} covers {}.", input.title, topic_names)
}

fn build_learning_outcomes(
    topics: &[EnrichmentWriteMatch],
    concepts: &[EnrichmentWriteMatch],
) -> Vec<String> {
    let mut outcomes = Vec::new();
    for topic in topics.iter().take(2) {
        outcomes.push(format!("Understand core ideas in {}.", topic.name));
    }
    for concept in concepts.iter().take(3) {
        outcomes.push(format!("Explain and apply {}.", concept.name));
    }
    outcomes
}

fn build_prerequisites(difficulty: &str, topics: &[EnrichmentWriteMatch]) -> Vec<String> {
    if difficulty == "beginner" {
        Vec::new()
    } else {
        topics
            .iter()
            .take(2)
            .map(|topic| format!("Basic {}", topic.name))
            .collect()
    }
}

fn evidence_chunks(input: &EnrichmentInput, keywords: &[&str]) -> Vec<Uuid> {
    let mut chunks = input
        .chunks
        .iter()
        .filter(|chunk| {
            let text = format!(
                "{} {} {}",
                chunk.heading_path.join(" "),
                chunk.content_kind,
                chunk.content
            )
            .to_ascii_lowercase();
            keywords.iter().any(|keyword| text.contains(keyword))
        })
        .take(5)
        .map(|chunk| chunk.chunk_id)
        .collect::<Vec<_>>();
    if chunks.is_empty() {
        chunks.extend(input.chunks.iter().take(2).map(|chunk| chunk.chunk_id));
    }
    chunks
}

fn build_haystack(input: &EnrichmentInput) -> String {
    normalize_query(&format!(
        "{} {} {}",
        input.title,
        input.url,
        input
            .chunks
            .iter()
            .take(20)
            .map(|chunk| format!(
                "{} {} {}",
                chunk.heading_path.join(" "),
                chunk.content_kind,
                chunk.content
            ))
            .collect::<Vec<_>>()
            .join(" ")
    ))
}

fn confidence(topic_count: usize, concept_count: usize, chunk_count: usize) -> f64 {
    let topic_score = if topic_count > 0 { 0.45 } else { 0.0 };
    let concept_score = (concept_count.min(3) as f64) * 0.12;
    let chunk_score = if chunk_count > 0 { 0.15 } else { 0.0 };
    (topic_score + concept_score + chunk_score).min(0.96)
}

fn write_match_to_response(value: EnrichmentWriteMatch) -> EnrichmentMatch {
    EnrichmentMatch {
        slug: value.slug,
        name: value.name,
        score: 1.0,
        evidence_chunk_ids: value.evidence_chunk_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::enrichment::{EnrichmentChunkInput, EnrichmentInput};

    #[test]
    fn postgres_index_resource_gets_topic_and_concept() {
        let input = EnrichmentInput {
            resource_id: Uuid::new_v4(),
            version_id: Uuid::new_v4(),
            title: "PostgreSQL B-tree Indexes".to_string(),
            url: "https://www.postgresql.org/docs/current/indexes.html".to_string(),
            source_kind: Some("official_docs".to_string()),
            is_official: true,
            chunks: vec![EnrichmentChunkInput {
                chunk_id: Uuid::new_v4(),
                heading_path: vec!["Indexes".to_string()],
                content: "B-tree indexes improve query performance.".to_string(),
                content_kind: "concept".to_string(),
            }],
        };

        let enrichment = classify_resource(&input);

        assert!(
            enrichment
                .topic_matches
                .iter()
                .any(|topic| topic.slug == "postgresql")
        );
        assert!(
            enrichment
                .concept_matches
                .iter()
                .any(|concept| concept.slug == "btree-index")
        );
        assert!(
            enrichment
                .resource_roles
                .contains(&"official_reference".to_string())
        );
    }
}
