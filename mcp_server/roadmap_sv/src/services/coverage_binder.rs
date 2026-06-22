#![allow(dead_code)]

use serde_json::Value;

use crate::{
    clients::resource_client::{ResourceClientError, ResourceContractClient},
    domain::{
        BoundTopicPlan, CoverageCheckResult, CoverageRole, CoverageStatus, CurrentLevel,
        NodeStatus, ResourceBinding, TopicPlan,
    },
};

pub async fn bind_topic_resources(
    client: &ResourceContractClient,
    topics: &[TopicPlan],
    goal: Option<&str>,
) -> Vec<BoundTopicPlan> {
    let mut bound_topics = Vec::with_capacity(topics.len());

    for topic in topics {
        bound_topics.push(bind_single_topic(client, topic, goal).await);
    }

    bound_topics
}

pub async fn bind_single_topic(
    client: &ResourceContractClient,
    topic: &TopicPlan,
    goal: Option<&str>,
) -> BoundTopicPlan {
    let level = level_as_str(&topic.level);
    let coverage_value = match client
        .get_topic_coverage(
            &topic.topic_name,
            Some(level),
            topic.required_resource_types.clone(),
        )
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return failed_binding(topic, error);
        }
    };
    let coverage = parse_coverage_result(&coverage_value, &topic.required_resource_types);

    match coverage.coverage_status {
        CoverageStatus::Good | CoverageStatus::Partial => {
            bind_recommendations(client, topic, goal, coverage).await
        }
        CoverageStatus::Poor => bind_gap(client, topic, coverage).await,
    }
}

async fn bind_recommendations(
    client: &ResourceContractClient,
    topic: &TopicPlan,
    goal: Option<&str>,
    coverage: CoverageCheckResult,
) -> BoundTopicPlan {
    let level = level_as_str(&topic.level);
    let recommendation_value = client
        .recommend_resources_for_topic(
            &topic.topic_name,
            Some(level),
            goal,
            topic.required_resource_types.clone(),
            3,
        )
        .await;

    let mut warnings = Vec::new();
    let mut resource_refs = match recommendation_value {
        Ok(value) => parse_resource_bindings(&value),
        Err(error) => {
            warnings.push(format!("Resource recommendation failed: {error}."));
            Vec::new()
        }
    };

    if resource_refs.is_empty() && matches!(coverage.coverage_status, CoverageStatus::Good) {
        warnings.push(
            "Coverage is good but no recommended ResourceRef was returned; node must remain partial."
                .to_string(),
        );
    }

    let missing_resource_types = coverage.missing_types.clone();
    if !missing_resource_types.is_empty() {
        warnings.push(format!(
            "Missing resource types: {}.",
            missing_resource_types.join(", ")
        ));
    }

    let status =
        if !resource_refs.is_empty() && matches!(coverage.coverage_status, CoverageStatus::Good) {
            NodeStatus::Ready
        } else {
            NodeStatus::Partial
        };

    normalize_coverage_roles(&mut resource_refs);

    BoundTopicPlan {
        topic_plan: topic.clone(),
        coverage,
        resource_refs,
        missing_resource_types,
        warnings,
        status,
        gap_reported: false,
        research_requested: false,
    }
}

async fn bind_gap(
    client: &ResourceContractClient,
    topic: &TopicPlan,
    coverage: CoverageCheckResult,
) -> BoundTopicPlan {
    let level = level_as_str(&topic.level);
    let missing_types = if coverage.missing_types.is_empty() {
        topic.required_resource_types.clone()
    } else {
        coverage.missing_types.clone()
    };
    let reason = format!(
        "Roadmap topic '{}' has poor Resource coverage.",
        topic.topic_name
    );

    let gap_reported = client
        .report_resource_gap(
            &topic.topic_name,
            Some(level),
            missing_types.clone(),
            &reason,
        )
        .await
        .is_ok();
    let research_requested = client
        .request_research_for_topic(&topic.topic_name, missing_types.clone(), 4)
        .await
        .is_ok();

    BoundTopicPlan {
        topic_plan: topic.clone(),
        coverage,
        resource_refs: vec![],
        missing_resource_types: missing_types,
        warnings: vec![
            "Coverage is poor; node should be blocked or placeholder until Resource backfill completes."
                .to_string(),
        ],
        status: NodeStatus::Blocked,
        gap_reported,
        research_requested,
    }
}

fn failed_binding(topic: &TopicPlan, error: ResourceClientError) -> BoundTopicPlan {
    BoundTopicPlan {
        topic_plan: topic.clone(),
        coverage: CoverageCheckResult {
            coverage_status: CoverageStatus::Partial,
            available_types: vec![],
            missing_types: topic.required_resource_types.clone(),
            confidence: None,
            candidate_resource_count: None,
            gap_id: None,
            raw: ResourceContractClient::normalized_error(&error),
        },
        resource_refs: vec![],
        missing_resource_types: topic.required_resource_types.clone(),
        warnings: vec![format!(
            "Resource coverage check failed; topic treated as partial: {error}."
        )],
        status: NodeStatus::Partial,
        gap_reported: false,
        research_requested: false,
    }
}

pub fn parse_coverage_result(value: &Value, required_types: &[String]) -> CoverageCheckResult {
    let status = value
        .get("coverageStatus")
        .or_else(|| value.get("status"))
        .and_then(Value::as_str)
        .map(parse_coverage_status)
        .unwrap_or(CoverageStatus::Partial);
    let available_types = string_array(
        value
            .get("availableTypes")
            .or_else(|| value.get("available_types")),
    );
    let mut missing_types = string_array(
        value
            .get("missingTypes")
            .or_else(|| value.get("missing_types")),
    );
    if missing_types.is_empty() {
        missing_types = required_types
            .iter()
            .filter(|required| {
                !available_types
                    .iter()
                    .any(|available| available.eq_ignore_ascii_case(required))
            })
            .cloned()
            .collect();
    }

    CoverageCheckResult {
        coverage_status: status,
        available_types,
        missing_types,
        confidence: value.get("confidence").and_then(Value::as_f64),
        candidate_resource_count: value
            .get("candidateResourceCount")
            .or_else(|| value.get("resultCount"))
            .and_then(Value::as_u64)
            .map(|count| count as u32),
        gap_id: value
            .get("gapId")
            .and_then(Value::as_str)
            .map(str::to_string),
        raw: value.clone(),
    }
}

pub fn parse_resource_bindings(value: &Value) -> Vec<ResourceBinding> {
    let resources = value
        .get("resources")
        .or_else(|| value.get("items"))
        .or_else(|| value.get("recommendations"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    resources
        .iter()
        .filter_map(parse_resource_binding)
        .collect()
}

fn parse_resource_binding(value: &Value) -> Option<ResourceBinding> {
    let resource_id = string_field(value, &["resourceId", "id"])?;
    let title = string_field(value, &["title"]).unwrap_or_else(|| "Untitled resource".to_string());
    let canonical_url = string_field(value, &["canonicalUrl", "url"]).unwrap_or_default();
    let kind = string_field(value, &["kind", "resourceType", "type"])
        .unwrap_or_else(|| "primary_learning".to_string());

    Some(ResourceBinding {
        resource_id,
        title,
        canonical_url,
        source_domain: string_field(value, &["sourceDomain", "sourceName"]),
        kind,
        format: string_field(value, &["format"]),
        language_code: string_field(value, &["languageCode", "language"]),
        is_official: value
            .get("isOfficial")
            .or_else(|| value.get("official"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        quality_score: value
            .get("qualityScore")
            .or_else(|| value.get("score"))
            .and_then(Value::as_f64),
        trust_tier: value
            .get("trustTier")
            .and_then(Value::as_u64)
            .map(|tier| tier as u8),
        coverage_role: role_from_resource(value),
        selected_chunks: None,
    })
}

fn normalize_coverage_roles(resources: &mut [ResourceBinding]) {
    if !resources
        .iter()
        .any(|resource| matches!(resource.coverage_role, CoverageRole::Primary))
    {
        if let Some(first) = resources.first_mut() {
            first.coverage_role = CoverageRole::Primary;
        }
    }
}

fn role_from_resource(value: &Value) -> CoverageRole {
    match string_field(value, &["coverageRole", "role"])
        .unwrap_or_default()
        .as_str()
    {
        "reference" | "official_reference" => CoverageRole::Reference,
        "practice" => CoverageRole::Practice,
        "optional" => CoverageRole::Optional,
        _ => CoverageRole::Primary,
    }
}

fn parse_coverage_status(value: &str) -> CoverageStatus {
    match value {
        "good" => CoverageStatus::Good,
        "poor" => CoverageStatus::Poor,
        _ => CoverageStatus::Partial,
    }
}

fn level_as_str(level: &CurrentLevel) -> &'static str {
    match level {
        CurrentLevel::Beginner => "beginner",
        CurrentLevel::Intermediate => "intermediate",
        CurrentLevel::Advanced => "advanced",
        CurrentLevel::Unknown => "unknown",
    }
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_coverage_and_derives_missing_types() {
        let coverage = parse_coverage_result(
            &json!({
                "coverageStatus": "good",
                "availableTypes": ["official_reference"],
                "confidence": 0.91,
                "candidateResourceCount": 2
            }),
            &["official_reference".to_string(), "practice".to_string()],
        );

        assert!(matches!(coverage.coverage_status, CoverageStatus::Good));
        assert_eq!(coverage.missing_types, vec!["practice"]);
        assert_eq!(coverage.candidate_resource_count, Some(2));
    }

    #[test]
    fn parses_resource_recommendations_into_refs() {
        let refs = parse_resource_bindings(&json!({
            "resources": [{
                "resourceId": "res_1",
                "title": "Official docs",
                "canonicalUrl": "https://example.com",
                "kind": "official_reference",
                "isOfficial": true,
                "qualityScore": 0.9,
                "trustTier": 1,
                "coverageRole": "reference"
            }]
        }));

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].resource_id, "res_1");
        assert!(matches!(refs[0].coverage_role, CoverageRole::Reference));
    }
}
