use serde_json::json;
use std::collections::HashMap;

use crate::{
    AppResult,
    models::{
        QueryInfo, RecommendRequest, RecommendResponse, SearchRequest, SearchResponse,
        SearchResult, TopicCoverageRequest, TopicCoverageResponse,
    },
    repository::{coverage_for_results, normalize_query},
};

use super::{ResourceService, validation};

impl ResourceService {
    pub async fn search_chunks(&self, request: SearchRequest) -> AppResult<SearchResponse> {
        validation::validate_search_request(&request)?;
        let normalized = normalize_search_query(&request.query);
        let technical_tokens = technical_tokens(&request.query);
        let requested_limit = request.limit.unwrap_or(10).clamp(1, 50) as usize;
        let mut candidate_request = request.clone();
        candidate_request.query = normalized.clone();
        candidate_request.limit = Some((requested_limit as i64 * 5).clamp(20, 50));
        let mut results = self.repository.search_chunks(&candidate_request).await?;
        apply_exact_token_boost(&mut results, &technical_tokens);
        let results = diversify_results(
            results,
            request.max_chunks_per_resource.unwrap_or(2).clamp(1, 10),
            requested_limit,
        );
        let best_score = results
            .iter()
            .map(|item| item.scores.final_score)
            .fold(0.0, f64::max);
        let min_results = self.config.search_low_confidence_min_results;
        let needs_gap = results.len() < min_results || best_score < 0.65;
        let gap_id = if needs_gap && request.create_gap_on_low_confidence.unwrap_or(true) {
            self.repository
                .create_gap_if_low_results(
                    "resource_service_api",
                    &request.query,
                    results.len() as i32,
                    min_results as i32,
                    json!({
                        "bestScore": best_score,
                        "source": "search_chunks",
                        "technicalTokens": technical_tokens,
                    }),
                )
                .await?
        } else {
            None
        };
        let coverage = coverage_for_results(results.len(), best_score, gap_id, &[]);
        Ok(SearchResponse {
            items: results,
            coverage,
            query_info: QueryInfo {
                normalized_query: normalized,
                strategy: if technical_tokens.is_empty() {
                    "hybrid".to_string()
                } else {
                    "hybrid_exact_boost".to_string()
                },
            },
        })
    }

    pub async fn search_resources(&self, request: SearchRequest) -> AppResult<SearchResponse> {
        self.search_chunks(request).await
    }

    pub async fn recommend(&self, request: RecommendRequest) -> AppResult<RecommendResponse> {
        if request.topic.trim().is_empty() {
            return Err(crate::AppError::Validation("topic is required".to_string()));
        }
        let required_types = request.required_types.clone().unwrap_or_else(|| {
            vec![
                "official_reference".to_string(),
                "primary_learning".to_string(),
            ]
        });
        let mut resources = self.repository.recommend(&request).await?;
        resources = select_recommended_resources(
            resources,
            &required_types,
            request.max_resources.unwrap_or(8).clamp(1, 20) as usize,
        );
        if !request.include_chunks.unwrap_or(false) {
            for resource in &mut resources {
                resource.chunk_ids.clear();
            }
        }
        let best_score = resources.iter().map(|item| item.score).fold(0.0, f64::max);
        let missing_types: Vec<String> = required_types
            .iter()
            .filter(|required| !resources.iter().any(|resource| &resource.role == *required))
            .cloned()
            .collect();
        let gap_id = if resources.len() < 2 || best_score < 0.65 || !missing_types.is_empty() {
            self.repository
                .create_gap_if_low_results(
                    "resource_service_api",
                    &request.topic,
                    resources.len() as i32,
                    5,
                    json!({
                        "source": "recommend_resources",
                        "missingTypes": missing_types,
                        "level": request.level,
                        "goal": request.goal,
                    }),
                )
                .await?
        } else {
            None
        };
        let coverage = coverage_for_results(resources.len(), best_score, gap_id, &missing_types);
        Ok(RecommendResponse {
            topic: request.topic.clone(),
            normalized_topic: normalize_query(&request.topic),
            level: request.level.clone(),
            resources,
            explanation: if coverage.low_confidence {
                "Not enough reliable resources for this topic yet.".to_string()
            } else if coverage.status == "partial" {
                "Found relevant resources, but coverage is missing at least one requested role."
                    .to_string()
            } else {
                "Found enough relevant resources for this topic.".to_string()
            },
            coverage,
        })
    }

    pub async fn topic_coverage(
        &self,
        request: TopicCoverageRequest,
    ) -> AppResult<TopicCoverageResponse> {
        let recommendation = self
            .recommend(RecommendRequest {
                topic: request.topic,
                level: request.level,
                goal: None,
                required_types: request.required_types,
                max_resources: Some(8),
                include_chunks: Some(false),
            })
            .await?;
        Ok(TopicCoverageResponse {
            topic: recommendation.topic,
            normalized_topic: recommendation.normalized_topic,
            coverage: recommendation.coverage,
        })
    }
}

fn select_recommended_resources(
    mut resources: Vec<crate::models::RecommendedResource>,
    required_types: &[String],
    max_resources: usize,
) -> Vec<crate::models::RecommendedResource> {
    resources.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut selected = Vec::new();
    let mut used = std::collections::HashSet::new();

    for role in required_types {
        if let Some(resource) = resources
            .iter()
            .find(|resource| &resource.role == role && !used.contains(&resource.resource_id))
            .cloned()
        {
            used.insert(resource.resource_id);
            selected.push(resource);
        }
    }

    for resource in resources {
        if selected.len() >= max_resources {
            break;
        }
        if used.insert(resource.resource_id) {
            selected.push(resource);
        }
    }
    selected
}

fn normalize_search_query(query: &str) -> String {
    let normalized = normalize_query(query);
    normalized
        .split_whitespace()
        .map(expand_alias)
        .collect::<Vec<_>>()
        .join(" ")
}

fn expand_alias(token: &str) -> &str {
    match token {
        "js" => "javascript",
        "ts" => "typescript",
        "postgres" => "postgresql",
        "k8s" => "kubernetes",
        "ctx" => "context",
        _ => token,
    }
}

fn technical_tokens(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|token| token.trim_matches(|ch: char| ch == ',' || ch == '.' || ch == ';'))
        .filter(|token| {
            token.chars().any(char::is_uppercase)
                || token.contains("::")
                || token.contains('_')
                || token.contains('-')
                || token.contains('.')
                || token.chars().any(|ch| ch.is_ascii_digit())
        })
        .map(ToString::to_string)
        .collect()
}

fn apply_exact_token_boost(results: &mut [SearchResult], tokens: &[String]) {
    if tokens.is_empty() {
        return;
    }
    for result in results {
        let haystack = format!(
            "{} {} {}",
            result.title,
            result.heading_path.join(" "),
            result.snippet
        )
        .to_lowercase();
        let exact_hits = tokens
            .iter()
            .filter(|token| haystack.contains(&token.to_lowercase()))
            .count();
        if exact_hits > 0 {
            let boost = (exact_hits as f64 * 0.08).min(0.24);
            result.scores.keyword = (result.scores.keyword + boost).min(1.0);
            result.scores.final_score = (result.scores.final_score + boost).min(1.0);
        }
    }
}

fn diversify_results(
    mut results: Vec<SearchResult>,
    max_chunks_per_resource: usize,
    limit: usize,
) -> Vec<SearchResult> {
    results.sort_by(|a, b| b.scores.final_score.total_cmp(&a.scores.final_score));
    let mut per_resource = HashMap::new();
    let mut selected = Vec::with_capacity(limit);
    let mut overflow = Vec::new();

    for result in results {
        let count = per_resource.entry(result.resource_id).or_insert(0usize);
        if *count < max_chunks_per_resource && selected.len() < limit {
            *count += 1;
            selected.push(result);
        } else {
            overflow.push(result);
        }
    }

    for result in overflow {
        if selected.len() >= limit {
            break;
        }
        selected.push(result);
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_aliases_are_expanded() {
        assert_eq!(
            normalize_search_query("postgres k8s"),
            "postgresql kubernetes"
        );
    }

    #[test]
    fn technical_tokens_detect_code_terms() {
        let tokens = technical_tokens("React useEffect cleanup ON CONFLICT");

        assert!(tokens.contains(&"useEffect".to_string()));
        assert!(tokens.contains(&"ON".to_string()));
        assert!(tokens.contains(&"CONFLICT".to_string()));
    }
}
