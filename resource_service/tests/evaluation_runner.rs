use std::{collections::HashSet, fs, sync::Arc};

use resource_service::{
    AppConfig, ResourceService, create_pool,
    models::{RecommendRequest, SearchRequest, TopicCoverageRequest},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct EvaluationDataset {
    version: String,
    topics: Vec<EvaluationTopic>,
}

#[derive(Debug, Deserialize)]
struct EvaluationTopic {
    topic: String,
    group: String,
    level: Option<String>,
    #[serde(rename = "expectedCoverage")]
    expected_coverage: String,
    #[serde(rename = "requiredTypes")]
    required_types: Vec<String>,
    #[serde(rename = "expectedOfficialDomains")]
    expected_official_domains: Vec<String>,
    #[serde(rename = "expectedMinResources")]
    expected_min_resources: i64,
    #[serde(rename = "expectedGapCreated")]
    expected_gap_created: bool,
}

#[derive(Debug, Serialize)]
struct EvaluationSummary {
    dataset_version: String,
    total_topics: usize,
    evaluated_topics: usize,
    passed_topics: usize,
    failed_topics: usize,
    metrics: EvaluationMetrics,
    failures: Vec<EvaluationFailure>,
}

#[derive(Debug, Default, Serialize)]
struct EvaluationMetrics {
    coverage_accuracy: f64,
    gap_creation_accuracy: f64,
    official_priority_accuracy: f64,
    recommendation_diversity_accuracy: f64,
    duplicate_free_accuracy: f64,
    average_top_k_results: f64,
}

#[derive(Debug, Serialize)]
struct EvaluationFailure {
    topic: String,
    group: String,
    reasons: Vec<String>,
}

#[tokio::test]
async fn resource_eval_dataset_runner() {
    if std::env::var("RUN_RESOURCE_EVAL").ok().as_deref() != Some("1") {
        eprintln!("skipping resource eval runner; set RUN_RESOURCE_EVAL=1 to enable");
        return;
    }

    dotenv::from_path("../.env").ok();

    let service = test_service().await;
    let dataset: EvaluationDataset = serde_json::from_str(include_str!(
        "../evaluation/resource_eval_dataset_v0_2.json"
    ))
    .expect("evaluation dataset should parse");

    let mut failures = Vec::new();
    let mut coverage_matches = 0_usize;
    let mut gap_matches = 0_usize;
    let mut official_matches = 0_usize;
    let mut diversity_matches = 0_usize;
    let mut duplicate_free_matches = 0_usize;
    let mut total_results = 0_usize;

    for topic in &dataset.topics {
        let outcome = evaluate_topic(service.clone(), topic)
            .await
            .unwrap_or_else(|err| TopicOutcome {
                coverage_matches: false,
                gap_matches: false,
                official_priority_matches: false,
                recommendation_diversity_matches: false,
                duplicate_free: false,
                top_k_results: 0,
                failure_reasons: vec![format!("evaluation error: {err}")],
            });

        coverage_matches += usize::from(outcome.coverage_matches);
        gap_matches += usize::from(outcome.gap_matches);
        official_matches += usize::from(outcome.official_priority_matches);
        diversity_matches += usize::from(outcome.recommendation_diversity_matches);
        duplicate_free_matches += usize::from(outcome.duplicate_free);
        total_results += outcome.top_k_results;

        if !outcome.failure_reasons.is_empty() {
            failures.push(EvaluationFailure {
                topic: topic.topic.clone(),
                group: topic.group.clone(),
                reasons: outcome.failure_reasons,
            });
        }
    }

    let total = dataset.topics.len().max(1);
    let summary = EvaluationSummary {
        dataset_version: dataset.version,
        total_topics: dataset.topics.len(),
        evaluated_topics: dataset.topics.len(),
        passed_topics: dataset.topics.len().saturating_sub(failures.len()),
        failed_topics: failures.len(),
        metrics: EvaluationMetrics {
            coverage_accuracy: ratio(coverage_matches, total),
            gap_creation_accuracy: ratio(gap_matches, total),
            official_priority_accuracy: ratio(official_matches, total),
            recommendation_diversity_accuracy: ratio(diversity_matches, total),
            duplicate_free_accuracy: ratio(duplicate_free_matches, total),
            average_top_k_results: total_results as f64 / total as f64,
        },
        failures,
    };

    let output = serde_json::to_string_pretty(&summary).expect("summary should serialize");
    fs::write("/tmp/resource_eval_summary.json", &output)
        .expect("evaluation summary should be written");
    println!("{output}");

    let strict = std::env::var("RESOURCE_EVAL_STRICT").ok().as_deref() == Some("1");
    if strict {
        assert_eq!(
            summary.failed_topics, 0,
            "strict resource eval failed; see /tmp/resource_eval_summary.json"
        );
    }
}

#[derive(Debug)]
struct TopicOutcome {
    coverage_matches: bool,
    gap_matches: bool,
    official_priority_matches: bool,
    recommendation_diversity_matches: bool,
    duplicate_free: bool,
    top_k_results: usize,
    failure_reasons: Vec<String>,
}

async fn evaluate_topic(
    service: Arc<ResourceService>,
    topic: &EvaluationTopic,
) -> resource_service::AppResult<TopicOutcome> {
    let search = service
        .search_chunks(SearchRequest {
            query: topic.topic.clone(),
            filters: None,
            limit: Some(10),
            max_chunks_per_resource: Some(2),
            include_coverage: Some(true),
            create_gap_on_low_confidence: Some(topic.expected_gap_created),
        })
        .await?;
    let recommendation = service
        .recommend(RecommendRequest {
            topic: topic.topic.clone(),
            level: topic.level.clone(),
            goal: None,
            required_types: Some(topic.required_types.clone()),
            max_resources: Some(10),
            include_chunks: Some(true),
        })
        .await?;
    let coverage = service
        .topic_coverage(TopicCoverageRequest {
            topic: topic.topic.clone(),
            level: topic.level.clone(),
            required_types: Some(topic.required_types.clone()),
        })
        .await?;

    let coverage_matches = coverage_matches(&coverage.coverage.status, &topic.expected_coverage);
    let gap_created = search.coverage.gap_id.is_some()
        || recommendation.coverage.gap_id.is_some()
        || coverage.coverage.gap_id.is_some();
    let gap_matches = gap_created == topic.expected_gap_created
        || (!topic.expected_gap_created && !search.coverage.low_confidence);
    let official_priority_matches =
        official_priority_matches(&recommendation.resources, &topic.expected_official_domains);
    let recommendation_diversity_matches =
        recommendation_diversity_matches(&recommendation.resources, &topic.required_types);
    let duplicate_free = duplicate_free(&search.items, &recommendation.resources);

    let mut failure_reasons = Vec::new();
    if !coverage_matches {
        failure_reasons.push(format!(
            "coverage expected {}, got {}",
            topic.expected_coverage, coverage.coverage.status
        ));
    }
    if !gap_matches {
        failure_reasons.push(format!(
            "gap expected {}, got {}",
            topic.expected_gap_created, gap_created
        ));
    }
    if !official_priority_matches {
        failure_reasons.push(format!(
            "official domains not prioritized: expected any of {:?}",
            topic.expected_official_domains
        ));
    }
    if !recommendation_diversity_matches {
        failure_reasons.push(format!(
            "recommendation roles missing required diversity {:?}",
            topic.required_types
        ));
    }
    if !duplicate_free {
        failure_reasons.push("duplicate resources found in search or recommendations".to_string());
    }
    if search.items.len() < topic.expected_min_resources as usize
        && topic.expected_coverage != "poor"
    {
        failure_reasons.push(format!(
            "expected at least {} search results, got {}",
            topic.expected_min_resources,
            search.items.len()
        ));
    }

    Ok(TopicOutcome {
        coverage_matches,
        gap_matches,
        official_priority_matches,
        recommendation_diversity_matches,
        duplicate_free,
        top_k_results: search.items.len(),
        failure_reasons,
    })
}

fn coverage_matches(actual: &str, expected: &str) -> bool {
    actual == expected || (expected == "good" && actual == "partial")
}

fn official_priority_matches(
    resources: &[resource_service::models::RecommendedResource],
    expected_domains: &[String],
) -> bool {
    if expected_domains.is_empty() {
        return true;
    }
    resources.iter().take(3).any(|resource| {
        expected_domains
            .iter()
            .any(|domain| resource.url.contains(domain))
    })
}

fn recommendation_diversity_matches(
    resources: &[resource_service::models::RecommendedResource],
    required_types: &[String],
) -> bool {
    if resources.is_empty() {
        return false;
    }
    let roles = resources
        .iter()
        .map(|resource| resource.role.as_str())
        .collect::<HashSet<_>>();
    required_types
        .iter()
        .filter(|required| roles.contains(required.as_str()))
        .count()
        >= required_types.len().min(2)
}

fn duplicate_free(
    search_items: &[resource_service::models::SearchResult],
    resources: &[resource_service::models::RecommendedResource],
) -> bool {
    let mut search_seen = HashSet::new();
    let search_unique = search_items
        .iter()
        .all(|item| search_seen.insert(item.chunk_id));
    let mut resource_seen = HashSet::new();
    let recommendation_unique = resources
        .iter()
        .all(|resource| resource_seen.insert(resource.resource_id));
    search_unique && recommendation_unique
}

fn ratio(count: usize, total: usize) -> f64 {
    count as f64 / total.max(1) as f64
}

async fn test_service() -> Arc<ResourceService> {
    let config = AppConfig::from_env();
    let pool = create_pool(&config).expect("resource postgres pool should be created");
    let service = Arc::new(ResourceService::new(pool, config));
    service
        .migrate()
        .await
        .expect("schema migration should pass");
    service
}
