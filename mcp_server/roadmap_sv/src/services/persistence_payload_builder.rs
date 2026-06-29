#![allow(dead_code)]

use serde_json::{Value, json};
use uuid::Uuid;

use crate::domain::{
    DatabaseMcpToolCall, DatabaseReadyRoadmapPayload, OrchestratorPersistencePlan,
    PersistenceOwner, ResourceBinding, RoadmapGraph, RoadmapNode, RoadmapPhase,
    required_database_capabilities,
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
    let roadmap_title = graph
        .metadata
        .get("normalizedGoal")
        .and_then(Value::as_str)
        .unwrap_or("Generated learning roadmap")
        .to_string();

    let existing_project_id = graph
        .project_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .filter(|value| Uuid::parse_str(value).is_ok());
    let project_id = match existing_project_id {
        Some(project_id) => Value::String(project_id.to_string()),
        None => {
            let (user_id, user_dependency) = resolve_project_owner(graph, &mut calls);
            calls.push(DatabaseMcpToolCall {
                tool_name: "create_project".to_string(),
                arguments: json!({
                    "user_id": user_id,
                    "title": roadmap_title,
                    "description": "Project created from Roadmap MCP persistence plan.",
                    "status": "draft",
                }),
                depends_on: user_dependency,
                result_alias: Some("project".to_string()),
            });
            Value::String("${project.id}".to_string())
        }
    };

    calls.push(DatabaseMcpToolCall {
        tool_name: "create_roadmap".to_string(),
        arguments: json!({
            "project_id": project_id,
            "version": 1,
            "title": roadmap_title,
            "generated_by": "roadmap_mcp",
        }),
        depends_on: if existing_project_id.is_some() {
            vec![]
        } else {
            vec!["project".to_string()]
        },
        result_alias: Some("roadmap".to_string()),
    });

    for phase in &graph.phases {
        calls.push(phase_call(phase));
    }
    for (index, node) in graph.nodes.iter().enumerate() {
        calls.push(milestone_call(node, index));
        calls.push(task_call(node));
        for resource_ref in &node.resource_refs {
            calls.push(resource_call(node, resource_ref));
        }
    }

    calls
}

fn resolve_project_owner(
    graph: &RoadmapGraph,
    calls: &mut Vec<DatabaseMcpToolCall>,
) -> (Value, Vec<String>) {
    let Some(user_id) = graph
        .user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return (Value::Null, vec![]);
    };

    if Uuid::parse_str(user_id).is_ok() {
        return (Value::String(user_id.to_string()), vec![]);
    }

    calls.push(DatabaseMcpToolCall {
        tool_name: "upsert_user".to_string(),
        arguments: json!({
            "firebase_id": user_id,
            "display_name": Value::Null,
            "email": Value::Null,
        }),
        depends_on: vec![],
        result_alias: Some("user".to_string()),
    });

    (Value::String(user_id.to_string()), vec!["user".to_string()])
}

fn phase_call(phase: &RoadmapPhase) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_phase".to_string(),
        arguments: json!({
            "roadmap_id": "${roadmap.id}",
            "phase_order": phase.order_index,
            "title": phase.title,
            "description": phase.purpose,
            "estimated_days": estimated_days_from_hours(phase.estimated_hours),
        }),
        depends_on: vec!["roadmap".to_string()],
        result_alias: Some(format!("phase:{}", phase.phase_id)),
    }
}

fn milestone_call(node: &RoadmapNode, index: usize) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_milestone".to_string(),
        arguments: json!({
            "phase_id": format!("${{phase:{}.id}}", node.phase_id),
            "milestone_order": index + 1,
            "title": node.title,
            "description": node.purpose,
        }),
        depends_on: vec!["roadmap".to_string(), format!("phase:{}", node.phase_id)],
        result_alias: Some(format!("milestone:{}", node.node_id)),
    }
}

fn task_call(node: &RoadmapNode) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_task".to_string(),
        arguments: json!({
            "milestone_id": format!("${{milestone:{}.id}}", node.node_id),
            "task_order": 1,
            "title": node.title,
            "description": task_description(node),
            "estimated_hours": node.estimated_hours,
            "difficulty": format!("{:?}", node.level).to_ascii_lowercase(),
            "status": task_status(node),
        }),
        depends_on: vec![format!("milestone:{}", node.node_id)],
        result_alias: Some(format!("task:{}", node.node_id)),
    }
}

fn resource_call(node: &RoadmapNode, resource: &ResourceBinding) -> DatabaseMcpToolCall {
    DatabaseMcpToolCall {
        tool_name: "create_resource".to_string(),
        arguments: json!({
            "task_id": format!("${{task:{}.id}}", node.node_id),
            "resource_type": resource.kind,
            "title": resource.title,
            "url": resource.canonical_url,
            "description": resource.source_domain,
        }),
        depends_on: vec![format!("task:{}", node.node_id)],
        result_alias: Some(format!(
            "resource:{}:{}",
            node.node_id, resource.resource_id
        )),
    }
}

fn estimated_days_from_hours(hours: u32) -> i32 {
    hours.div_ceil(2) as i32
}

fn task_status(node: &RoadmapNode) -> &'static str {
    match node.status {
        crate::domain::NodeStatus::Blocked | crate::domain::NodeStatus::Placeholder => "blocked",
        _ => "pending",
    }
}

fn task_description(node: &RoadmapNode) -> String {
    let mut parts = vec![node.purpose.clone()];
    if !node.learning_outcomes.is_empty() {
        parts.push(format!("Outcomes: {}", node.learning_outcomes.join("; ")));
    }
    if !node.warnings.is_empty() {
        parts.push(format!("Warnings: {}", node.warnings.join("; ")));
    }
    parts.join("\n")
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
                .any(|call| call.tool_name == "create_task")
        );
    }

    #[test]
    fn payload_call_order_starts_with_roadmap() {
        let graph = sample_graph();
        let payload = build_database_ready_payload(&graph);
        let calls = payload.orchestrator_persistence_plan.database_mcp_calls;

        assert_eq!(calls[0].tool_name, "upsert_user");
        assert_eq!(calls[1].tool_name, "create_project");
        assert_eq!(calls[2].tool_name, "create_roadmap");
        assert!(calls[2].arguments.get("project_id").is_some());
        assert!(calls[3].depends_on.contains(&"roadmap".to_string()));
    }

    #[test]
    fn existing_project_id_skips_project_creation() {
        let mut graph = sample_graph();
        graph.project_id = Some("11111111-1111-1111-1111-111111111111".to_string());
        let payload = build_database_ready_payload(&graph);
        let calls = payload.orchestrator_persistence_plan.database_mcp_calls;

        assert_eq!(calls[0].tool_name, "create_roadmap");
        assert_eq!(
            calls[0].arguments["project_id"],
            "11111111-1111-1111-1111-111111111111"
        );
        assert!(
            !calls
                .iter()
                .any(|call| call.tool_name == "create_project" || call.tool_name == "upsert_user")
        );
    }

    #[test]
    fn non_uuid_project_id_does_not_reach_database_project_uuid_field() {
        let mut graph = sample_graph();
        graph.project_id = Some("frontend-project-id".to_string());
        let payload = build_database_ready_payload(&graph);
        let calls = payload.orchestrator_persistence_plan.database_mcp_calls;

        assert!(calls.iter().any(|call| call.tool_name == "create_project"));
        let roadmap_call = calls
            .iter()
            .find(|call| call.tool_name == "create_roadmap")
            .unwrap();
        assert_eq!(roadmap_call.arguments["project_id"], "${project.id}");
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
            None,
            &goal,
            &selection.blueprint,
            &bound,
        )
    }
}
