#![allow(dead_code)]

use serde_json::{Value, json};

use crate::domain::{
    DatabaseMcpToolCall, DatabaseReadyRoadmapPayload, OrchestratorPersistencePlan,
    PersistenceOwner, RoadmapGraph, RoadmapNode, RoadmapPhase, required_database_capabilities,
};

pub fn build_database_ready_payload(graph: &RoadmapGraph) -> DatabaseReadyRoadmapPayload {
    DatabaseReadyRoadmapPayload {
        schema_version: "roadmap_draft_v1".to_string(),
        not_persisted: true,
        persistence_owner: PersistenceOwner::OrchestratorAgent,
        required_database_capabilities: required_database_capabilities()
            .into_iter()
            .map(str::to_string)
            .collect(),
        orchestrator_persistence_plan: OrchestratorPersistencePlan {
            database_mcp_calls: build_database_calls(graph),
            transactional_preference:
                "Execute as an all-or-nothing persistence workflow when Database MCP supports transactions."
                    .to_string(),
            failure_policy:
                "If any create_* call fails, stop execution and return the original RoadmapGraph as notPersisted."
                    .to_string(),
        },
    }
}

fn build_database_calls(graph: &RoadmapGraph) -> Vec<DatabaseMcpToolCall> {
    let mut calls = Vec::new();

    calls.push(DatabaseMcpToolCall {
        tool_name: "create_roadmap".to_string(),
        arguments: json!({
            "userId": graph.user_id,
            "status": graph.status,
            "metadata": graph.metadata,
            "coverageSummary": graph.coverage_summary,
            "resourceSummary": graph.resource_summary,
            "gapWarnings": graph.gap_warnings,
            "generatedBy": "roadmap_mcp",
        }),
        depends_on: vec![],
        result_alias: Some("roadmap".to_string()),
    });

    for phase in &graph.phases {
        calls.push(phase_call(phase));
    }
    for node in &graph.nodes {
        calls.push(node_call(node));
        for resource_ref in &node.resource_refs {
            calls.push(DatabaseMcpToolCall {
                tool_name: "attach_resource_ref_to_node".to_string(),
                arguments: json!({
                    "nodeId": format!("${{node:{}}}.nodeId", node.node_id),
                    "resourceRef": resource_ref,
                }),
                depends_on: vec![format!("node:{}", node.node_id)],
                result_alias: Some(format!(
                    "resource_ref:{}:{}",
                    node.node_id, resource_ref.resource_id
                )),
            });
        }
    }
    for edge in &graph.edges {
        calls.push(DatabaseMcpToolCall {
            tool_name: "create_roadmap_edge".to_string(),
            arguments: json!({
                "roadmapId": "${roadmap.roadmapId}",
                "fromNodeId": format!("${{node:{}}}.nodeId", edge.from_node_id),
                "toNodeId": format!("${{node:{}}}.nodeId", edge.to_node_id),
                "edgeType": edge.edge_type,
                "reason": edge.reason,
            }),
            depends_on: vec![
                "roadmap".to_string(),
                format!("node:{}", edge.from_node_id),
                format!("node:{}", edge.to_node_id),
            ],
            result_alias: Some(format!("edge:{}:{}", edge.from_node_id, edge.to_node_id)),
        });
    }

    calls.push(DatabaseMcpToolCall {
        tool_name: "create_audit_event".to_string(),
        arguments: json!({
            "roadmapId": "${roadmap.roadmapId}",
            "eventType": "roadmap_generated",
            "source": "roadmap_mcp",
            "summary": {
                "nodeCount": graph.nodes.len(),
                "phaseCount": graph.phases.len(),
                "status": graph.status,
                "coverageSummary": graph.coverage_summary,
            }
        }),
        depends_on: vec!["roadmap".to_string()],
        result_alias: Some("audit_event".to_string()),
    });

    calls
}

fn phase_call(phase: &RoadmapPhase) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_roadmap_phase".to_string(),
        arguments: json!({
            "roadmapId": "${roadmap.roadmapId}",
            "phaseId": phase.phase_id,
            "title": phase.title,
            "purpose": phase.purpose,
            "orderIndex": phase.order_index,
            "estimatedHours": phase.estimated_hours,
            "nodeIds": phase.node_ids,
            "exitCriteria": phase.exit_criteria,
        }),
        depends_on: vec!["roadmap".to_string()],
        result_alias: Some(format!("phase:{}", phase.phase_id)),
    }
}

fn node_call(node: &RoadmapNode) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_roadmap_node".to_string(),
        arguments: json!({
            "roadmapId": "${roadmap.roadmapId}",
            "phaseId": format!("${{phase:{}}}.phaseId", node.phase_id),
            "clientNodeId": node.node_id,
            "title": node.title,
            "topic": node.topic,
            "aliases": node.aliases,
            "nodeType": node.node_type,
            "level": node.level,
            "purpose": node.purpose,
            "learningOutcomes": node.learning_outcomes,
            "prerequisites": node.prerequisites,
            "estimatedHours": node.estimated_hours,
            "coverageStatus": node.coverage_status,
            "missingResourceTypes": node.missing_resource_types,
            "warnings": node.warnings,
            "status": node.status,
        }),
        depends_on: vec!["roadmap".to_string(), format!("phase:{}", node.phase_id)],
        result_alias: Some(format!("node:{}", node.node_id)),
    }
}

pub fn payload_summary(payload: &DatabaseReadyRoadmapPayload) -> Value {
    let calls = &payload.orchestrator_persistence_plan.database_mcp_calls;
    json!({
        "notPersisted": payload.not_persisted,
        "persistenceOwner": payload.persistence_owner,
        "callCount": calls.len(),
        "toolNames": calls.iter().map(|call| call.tool_name.as_str()).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            BoundTopicPlan, CoverageCheckResult, CoverageRole, CoverageStatus, CurrentLevel,
            GoalCategory, GoalProfile, NodeStatus, ResourceBinding, TargetRole,
        },
        services::{
            blueprint_registry::select_blueprint, graph_builder::build_roadmap_graph,
            topic_decomposer::decompose_topics,
        },
    };

    #[test]
    fn builds_database_ready_payload_without_persisting() {
        let graph = sample_graph();
        let payload = build_database_ready_payload(&graph);

        assert!(payload.not_persisted);
        assert!(matches!(
            payload.persistence_owner,
            PersistenceOwner::OrchestratorAgent
        ));
        assert!(
            payload
                .orchestrator_persistence_plan
                .database_mcp_calls
                .iter()
                .any(|call| call.tool_name == "create_roadmap")
        );
        assert!(
            payload
                .orchestrator_persistence_plan
                .database_mcp_calls
                .iter()
                .any(|call| call.tool_name == "create_roadmap_node")
        );
    }

    #[test]
    fn payload_call_order_starts_with_roadmap() {
        let graph = sample_graph();
        let payload = build_database_ready_payload(&graph);
        let calls = payload.orchestrator_persistence_plan.database_mcp_calls;

        assert_eq!(calls[0].tool_name, "create_roadmap");
        assert!(calls[1].depends_on.contains(&"roadmap".to_string()));
    }

    fn sample_graph() -> RoadmapGraph {
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
            .take(2)
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
                resource_refs: vec![ResourceBinding {
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
                }],
                missing_resource_types: vec![],
                warnings: vec![],
                status: NodeStatus::Ready,
                gap_reported: false,
                research_requested: false,
            })
            .collect::<Vec<_>>();

        build_roadmap_graph(
            Some("user_1".to_string()),
            &goal,
            &selection.blueprint,
            &bound,
        )
    }
}
