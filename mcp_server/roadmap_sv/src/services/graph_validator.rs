#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use crate::domain::{
    CoverageStatus, NodeStatus, RoadmapGraph, RoadmapNode, ValidationIssue, ValidationResult,
};

pub fn validate_roadmap_graph(graph: &RoadmapGraph) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    validate_non_empty(graph, &mut errors);
    validate_phase_ids(graph, &mut errors);
    validate_node_ids(graph, &mut errors);
    validate_node_phase_membership(graph, &mut errors);
    validate_phase_node_membership(graph, &mut warnings);
    validate_edges(graph, &mut errors);
    validate_no_cycles(graph, &mut errors);
    validate_node_resources(graph, &mut errors, &mut warnings);
    validate_estimates(graph, &mut errors);
    validate_coverage_summary(graph, &mut warnings);

    ValidationResult {
        valid: errors.is_empty(),
        quality_score: Some(quality_score(&errors, &warnings)),
        errors,
        warnings,
    }
}

fn validate_non_empty(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    if graph.phases.is_empty() {
        errors.push(issue(
            "ROADMAP_PHASES_EMPTY",
            "Roadmap must contain at least one phase.",
            Some("phases"),
        ));
    }
    if graph.nodes.is_empty() {
        errors.push(issue(
            "ROADMAP_NODES_EMPTY",
            "Roadmap must contain at least one node.",
            Some("nodes"),
        ));
    }
}

fn validate_phase_ids(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    let mut ids = BTreeSet::new();
    for phase in &graph.phases {
        if phase.phase_id.trim().is_empty() {
            errors.push(issue(
                "PHASE_ID_REQUIRED",
                "Each phase must have a phaseId.",
                Some("phases.phaseId"),
            ));
        }
        if !ids.insert(phase.phase_id.clone()) {
            errors.push(issue_with_details(
                "DUPLICATE_PHASE_ID",
                "Phase ids must be unique.",
                Some("phases.phaseId"),
                json!({"phaseId": phase.phase_id}),
            ));
        }
    }
}

fn validate_node_ids(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    let mut ids = BTreeSet::new();
    for node in &graph.nodes {
        if node.node_id.trim().is_empty() {
            errors.push(issue(
                "NODE_ID_REQUIRED",
                "Each node must have a nodeId.",
                Some("nodes.nodeId"),
            ));
        }
        if !ids.insert(node.node_id.clone()) {
            errors.push(issue_with_details(
                "DUPLICATE_NODE_ID",
                "Node ids must be unique.",
                Some("nodes.nodeId"),
                json!({"nodeId": node.node_id}),
            ));
        }
    }
}

fn validate_node_phase_membership(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    let phase_ids = graph
        .phases
        .iter()
        .map(|phase| phase.phase_id.as_str())
        .collect::<BTreeSet<_>>();
    for node in &graph.nodes {
        if !phase_ids.contains(node.phase_id.as_str()) {
            errors.push(issue_with_details(
                "NODE_PHASE_MISSING",
                "Every node must belong to an existing phase.",
                Some("nodes.phaseId"),
                json!({"nodeId": node.node_id, "phaseId": node.phase_id}),
            ));
        }
    }
}

fn validate_phase_node_membership(graph: &RoadmapGraph, warnings: &mut Vec<ValidationIssue>) {
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect::<BTreeSet<_>>();
    for phase in &graph.phases {
        for node_id in &phase.node_ids {
            if !node_ids.contains(node_id.as_str()) {
                warnings.push(issue_with_details(
                    "PHASE_REFERENCES_MISSING_NODE",
                    "Phase nodeIds should reference existing nodes.",
                    Some("phases.nodeIds"),
                    json!({"phaseId": phase.phase_id, "nodeId": node_id}),
                ));
            }
        }
    }
}

fn validate_edges(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect::<BTreeSet<_>>();
    for edge in &graph.edges {
        if !node_ids.contains(edge.from_node_id.as_str()) {
            errors.push(issue_with_details(
                "EDGE_FROM_NODE_MISSING",
                "Edge fromNodeId must reference an existing node.",
                Some("edges.fromNodeId"),
                json!({"fromNodeId": edge.from_node_id}),
            ));
        }
        if !node_ids.contains(edge.to_node_id.as_str()) {
            errors.push(issue_with_details(
                "EDGE_TO_NODE_MISSING",
                "Edge toNodeId must reference an existing node.",
                Some("edges.toNodeId"),
                json!({"toNodeId": edge.to_node_id}),
            ));
        }
        if edge.from_node_id == edge.to_node_id {
            errors.push(issue_with_details(
                "SELF_EDGE",
                "A roadmap edge cannot point to the same node.",
                Some("edges"),
                json!({"nodeId": edge.from_node_id}),
            ));
        }
    }
}

fn validate_no_cycles(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &graph.edges {
        adjacency
            .entry(edge.from_node_id.as_str())
            .or_default()
            .push(edge.to_node_id.as_str());
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in &graph.nodes {
        if has_cycle(
            node.node_id.as_str(),
            &adjacency,
            &mut visiting,
            &mut visited,
        ) {
            errors.push(issue(
                "PREREQUISITE_CYCLE",
                "Roadmap prerequisite graph must not contain cycles.",
                Some("edges"),
            ));
            return;
        }
    }
}

fn has_cycle<'a>(
    node_id: &'a str,
    adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> bool {
    if visited.contains(node_id) {
        return false;
    }
    if !visiting.insert(node_id) {
        return true;
    }
    for next in adjacency.get(node_id).cloned().unwrap_or_default() {
        if has_cycle(next, adjacency, visiting, visited) {
            return true;
        }
    }
    visiting.remove(node_id);
    visited.insert(node_id);
    false
}

fn validate_node_resources(
    graph: &RoadmapGraph,
    errors: &mut Vec<ValidationIssue>,
    warnings: &mut Vec<ValidationIssue>,
) {
    for node in &graph.nodes {
        validate_required_fields(node, errors);
        match node.coverage_status {
            CoverageStatus::Good => {
                if node.resource_refs.is_empty() {
                    errors.push(issue_with_details(
                        "GOOD_COVERAGE_NODE_MISSING_RESOURCE",
                        "A good-coverage node must have at least one ResourceRef.",
                        Some("nodes.resourceRefs"),
                        json!({"nodeId": node.node_id}),
                    ));
                }
                if !matches!(node.status, NodeStatus::Ready | NodeStatus::Partial) {
                    warnings.push(issue_with_details(
                        "GOOD_COVERAGE_STATUS_INCONSISTENT",
                        "Good coverage should normally produce ready or partial nodes.",
                        Some("nodes.status"),
                        json!({"nodeId": node.node_id}),
                    ));
                }
            }
            CoverageStatus::Partial => {
                if node.warnings.is_empty() && node.missing_resource_types.is_empty() {
                    warnings.push(issue_with_details(
                        "PARTIAL_NODE_MISSING_WARNING",
                        "A partial node should include warning or missing resource type details.",
                        Some("nodes.warnings"),
                        json!({"nodeId": node.node_id}),
                    ));
                }
            }
            CoverageStatus::Poor => {
                if node.warnings.is_empty() && node.missing_resource_types.is_empty() {
                    errors.push(issue_with_details(
                        "POOR_NODE_MISSING_GAP_DETAILS",
                        "A poor-coverage node must expose warnings or missing resource types.",
                        Some("nodes"),
                        json!({"nodeId": node.node_id}),
                    ));
                }
                if !matches!(node.status, NodeStatus::Blocked | NodeStatus::Placeholder) {
                    warnings.push(issue_with_details(
                        "POOR_COVERAGE_STATUS_INCONSISTENT",
                        "Poor coverage should produce blocked or placeholder nodes.",
                        Some("nodes.status"),
                        json!({"nodeId": node.node_id}),
                    ));
                }
            }
        }
    }
}

fn validate_required_fields(node: &RoadmapNode, errors: &mut Vec<ValidationIssue>) {
    if node.title.trim().is_empty() {
        errors.push(issue_with_details(
            "NODE_TITLE_REQUIRED",
            "Each node must have a title.",
            Some("nodes.title"),
            json!({"nodeId": node.node_id}),
        ));
    }
    if node.topic.trim().is_empty() {
        errors.push(issue_with_details(
            "NODE_TOPIC_REQUIRED",
            "Each node must have a topic.",
            Some("nodes.topic"),
            json!({"nodeId": node.node_id}),
        ));
    }
}

fn validate_estimates(graph: &RoadmapGraph, errors: &mut Vec<ValidationIssue>) {
    for node in &graph.nodes {
        if node.estimated_hours == 0 {
            errors.push(issue_with_details(
                "NODE_ESTIMATE_INVALID",
                "Node estimatedHours must be positive.",
                Some("nodes.estimatedHours"),
                json!({"nodeId": node.node_id}),
            ));
        }
    }
}

fn validate_coverage_summary(graph: &RoadmapGraph, warnings: &mut Vec<ValidationIssue>) {
    let counted = graph.coverage_summary.coverage_good
        + graph.coverage_summary.coverage_partial
        + graph.coverage_summary.coverage_poor;
    if counted != graph.coverage_summary.total_topics {
        warnings.push(issue_with_details(
            "COVERAGE_SUMMARY_COUNT_MISMATCH",
            "Coverage summary counts should equal totalTopics.",
            Some("coverageSummary"),
            json!({
                "totalTopics": graph.coverage_summary.total_topics,
                "countedTopics": counted,
            }),
        ));
    }
}

fn quality_score(errors: &[ValidationIssue], warnings: &[ValidationIssue]) -> f64 {
    (1.0 - errors.len() as f64 * 0.20 - warnings.len() as f64 * 0.05).clamp(0.0, 1.0)
}

fn issue(code: &str, message: &str, field: Option<&str>) -> ValidationIssue {
    ValidationIssue {
        code: code.to_string(),
        message: message.to_string(),
        field: field.map(str::to_string),
        details: None,
    }
}

fn issue_with_details(
    code: &str,
    message: &str,
    field: Option<&str>,
    details: serde_json::Value,
) -> ValidationIssue {
    ValidationIssue {
        code: code.to_string(),
        message: message.to_string(),
        field: field.map(str::to_string),
        details: Some(details),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            BoundTopicPlan, CoverageCheckResult, CoverageRole, CurrentLevel, GoalCategory,
            GoalProfile, NodeStatus, ResourceBinding, TargetRole,
        },
        services::{
            blueprint_registry::select_blueprint, graph_builder::build_roadmap_graph,
            topic_decomposer::decompose_topics,
        },
    };

    #[test]
    fn validates_generated_ready_graph() {
        let graph = ready_graph();
        let result = validate_roadmap_graph(&graph);

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn catches_cycle() {
        let mut graph = ready_graph();
        let first = graph.nodes[0].node_id.clone();
        let second = graph.nodes[1].node_id.clone();
        graph.edges.push(crate::domain::RoadmapEdge {
            from_node_id: first.clone(),
            to_node_id: second.clone(),
            edge_type: crate::domain::EdgeType::Prerequisite,
            reason: "test".to_string(),
        });
        graph.edges.push(crate::domain::RoadmapEdge {
            from_node_id: second,
            to_node_id: first,
            edge_type: crate::domain::EdgeType::Prerequisite,
            reason: "test".to_string(),
        });

        let result = validate_roadmap_graph(&graph);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.code == "PREREQUISITE_CYCLE")
        );
    }

    #[test]
    fn catches_good_node_without_resource() {
        let mut graph = ready_graph();
        graph.nodes[0].resource_refs.clear();

        let result = validate_roadmap_graph(&graph);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.code == "GOOD_COVERAGE_NODE_MISSING_RESOURCE")
        );
    }

    fn ready_graph() -> RoadmapGraph {
        let goal = GoalProfile {
            category: GoalCategory::Backend,
            domain: "backend".to_string(),
            stack: vec!["Node.js".to_string(), "PostgreSQL".to_string()],
            target_role: Some(TargetRole::Backend),
            level: CurrentLevel::Beginner,
            desired_outcome: None,
            normalized_goal: "learn backend".to_string(),
            warnings: vec![],
        };
        let selection = select_blueprint(&goal);
        let topics = decompose_topics(&selection.blueprint, None);
        let bound = topics
            .into_iter()
            .map(|topic| BoundTopicPlan {
                topic_plan: topic,
                coverage: CoverageCheckResult {
                    coverage_status: CoverageStatus::Good,
                    available_types: vec!["primary_learning".to_string()],
                    missing_types: vec![],
                    confidence: Some(0.9),
                    candidate_resource_count: Some(1),
                    gap_id: None,
                    raw: json!({}),
                },
                resource_refs: vec![resource_ref()],
                missing_resource_types: vec![],
                warnings: vec![],
                status: NodeStatus::Ready,
                gap_reported: false,
                research_requested: false,
            })
            .collect::<Vec<_>>();

        build_roadmap_graph(None, &goal, &selection.blueprint, &bound)
    }

    fn resource_ref() -> ResourceBinding {
        ResourceBinding {
            resource_id: "res_1".to_string(),
            title: "Resource".to_string(),
            canonical_url: "https://example.com".to_string(),
            source_domain: None,
            kind: "primary_learning".to_string(),
            format: None,
            language_code: Some("en".to_string()),
            is_official: false,
            quality_score: Some(0.8),
            trust_tier: Some(2),
            coverage_role: CoverageRole::Primary,
            selected_chunks: None,
        }
    }
}
