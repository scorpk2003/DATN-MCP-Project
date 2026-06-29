use serde_json::json;

use crate::{
    contract,
    domain::{
        BoundTopicPlan, CoverageCheckResult, CoverageRole, CoverageStatus, CurrentLevel,
        GoalProfile, NodeStatus, ResourceBinding, RoadmapConstraints, RoadmapGenerationRequest,
        RoadmapGraph, RoadmapStatus, SaveMode, TargetRole, TimeBudget, TopicPlan,
    },
    server::server::{
        CandidateTopicInput, PlanRoadmapFromTopicsParam, blueprint_from_topic_plan,
        topic_plan_from_candidates,
    },
    services::{
        blueprint_registry, graph_builder, graph_validator, persistence_payload_builder,
        request_validator, topic_decomposer,
    },
};

#[test]
fn phase_13_core_roadmap_scenarios_pass_quality_gate() {
    let scenarios = [
        (
            "Learn React basics",
            CurrentLevel::Beginner,
            Some(TargetRole::Frontend),
        ),
        (
            "Learn frontend web foundation",
            CurrentLevel::Beginner,
            Some(TargetRole::Frontend),
        ),
        (
            "Learn backend with Node.js and PostgreSQL",
            CurrentLevel::Beginner,
            Some(TargetRole::Backend),
        ),
        (
            "Learn backend with Python and PostgreSQL",
            CurrentLevel::Beginner,
            Some(TargetRole::Backend),
        ),
        (
            "Learn Docker basics",
            CurrentLevel::Beginner,
            Some(TargetRole::Devops),
        ),
        (
            "Learn Kubernetes basics",
            CurrentLevel::Beginner,
            Some(TargetRole::Devops),
        ),
        (
            "Learn PostgreSQL foundation",
            CurrentLevel::Beginner,
            Some(TargetRole::Backend),
        ),
        (
            "Learn Rust foundation",
            CurrentLevel::Beginner,
            Some(TargetRole::GeneralSoftware),
        ),
        (
            "Learn Python foundation",
            CurrentLevel::Beginner,
            Some(TargetRole::GeneralSoftware),
        ),
        (
            "Learn fullstack React + Node + PostgreSQL",
            CurrentLevel::Beginner,
            Some(TargetRole::Fullstack),
        ),
    ];

    for (goal, level, role) in scenarios {
        let graph = evaluated_graph(goal, level, role);
        let validation = graph_validator::validate_roadmap_graph(&graph);
        let payload = persistence_payload_builder::build_database_ready_payload(&graph);

        assert!(validation.valid, "{goal}: {:?}", validation.errors);
        assert!(!graph.phases.is_empty(), "{goal}: phases missing");
        assert!(!graph.nodes.is_empty(), "{goal}: nodes missing");
        assert_eq!(
            graph.coverage_summary.coverage_poor, 0,
            "{goal}: poor coverage"
        );
        assert_eq!(
            graph.coverage_summary.total_topics as usize,
            graph.nodes.len(),
            "{goal}: summary/node mismatch"
        );
        assert!(
            graph
                .nodes
                .iter()
                .filter(|node| matches!(node.status, NodeStatus::Ready))
                .all(|node| !node.resource_refs.is_empty()),
            "{goal}: ready node without ResourceRef"
        );
        assert!(
            payload.not_persisted,
            "{goal}: Roadmap MCP must not execute Database MCP"
        );
        assert!(
            payload
                .orchestrator_persistence_plan
                .database_mcp_calls
                .iter()
                .any(|call| call.tool_name == "create_roadmap"),
            "{goal}: persistence plan missing create_roadmap"
        );
    }
}

#[test]
fn phase_14_niche_candidate_topic_cases_pass_adaptive_planning_gate() {
    let cases = [
        (
            "PostgreSQL logical decoding with Debezium",
            vec![
                topic(
                    "PostgreSQL WAL basics",
                    "Required before logical decoding",
                    vec![],
                ),
                topic(
                    "Logical replication",
                    "Replication foundation",
                    vec!["PostgreSQL WAL basics"],
                ),
                topic(
                    "Logical decoding",
                    "Main target",
                    vec!["Logical replication"],
                ),
                topic(
                    "Debezium PostgreSQL connector",
                    "Tooling target",
                    vec!["Logical decoding"],
                ),
            ],
        ),
        (
            "Rust async Pin/Unpin",
            vec![
                topic("Rust ownership refresh", "Prerequisite", vec![]),
                topic(
                    "Future trait",
                    "Async foundation",
                    vec!["Rust ownership refresh"],
                ),
                topic("Pin and Unpin", "Main target", vec!["Future trait"]),
            ],
        ),
        (
            "eBPF observability for backend services",
            vec![
                topic("Linux tracing basics", "Prerequisite", vec![]),
                topic(
                    "eBPF programs",
                    "Core concept",
                    vec!["Linux tracing basics"],
                ),
                topic(
                    "Backend service telemetry",
                    "Application target",
                    vec!["eBPF programs"],
                ),
            ],
        ),
        (
            "Kubernetes admission webhooks",
            vec![
                topic("Kubernetes API objects", "Prerequisite", vec![]),
                topic(
                    "Admission controllers",
                    "Core concept",
                    vec!["Kubernetes API objects"],
                ),
                topic(
                    "Validating webhooks",
                    "Main target",
                    vec!["Admission controllers"],
                ),
            ],
        ),
        (
            "Redis stream consumer groups",
            vec![
                topic("Redis Streams", "Core concept", vec![]),
                topic("Consumer groups", "Main target", vec!["Redis Streams"]),
                topic(
                    "Pending entries recovery",
                    "Operational concern",
                    vec!["Consumer groups"],
                ),
            ],
        ),
        (
            "Kafka exactly-once semantics",
            vec![
                topic("Kafka producer guarantees", "Prerequisite", vec![]),
                topic(
                    "Transactions",
                    "Core mechanism",
                    vec!["Kafka producer guarantees"],
                ),
                topic(
                    "Exactly-once processing",
                    "Main target",
                    vec!["Transactions"],
                ),
            ],
        ),
        (
            "LSM tree storage engine",
            vec![
                topic("Write-ahead logging", "Prerequisite", vec![]),
                topic(
                    "Memtables and SSTables",
                    "Core structure",
                    vec!["Write-ahead logging"],
                ),
                topic(
                    "Compaction strategies",
                    "Main target",
                    vec!["Memtables and SSTables"],
                ),
            ],
        ),
        (
            "OAuth2 PKCE for SPA",
            vec![
                topic("OAuth2 authorization code flow", "Prerequisite", vec![]),
                topic(
                    "PKCE verifier and challenge",
                    "Main target",
                    vec!["OAuth2 authorization code flow"],
                ),
                topic(
                    "SPA token handling",
                    "Application target",
                    vec!["PKCE verifier and challenge"],
                ),
            ],
        ),
        (
            "OpenTelemetry distributed tracing",
            vec![
                topic("Trace/span model", "Core concept", vec![]),
                topic(
                    "Context propagation",
                    "Main target",
                    vec!["Trace/span model"],
                ),
                topic(
                    "Collector pipeline",
                    "Operational target",
                    vec!["Context propagation"],
                ),
            ],
        ),
        (
            "NGINX reverse proxy and TLS termination",
            vec![
                topic("Reverse proxy basics", "Prerequisite", vec![]),
                topic("TLS certificates", "Security foundation", vec![]),
                topic(
                    "NGINX TLS termination",
                    "Main target",
                    vec!["Reverse proxy basics", "TLS certificates"],
                ),
            ],
        ),
    ];

    for (goal, candidate_topics) in cases {
        let param = PlanRoadmapFromTopicsParam {
            user_id: Some("user_eval".to_string()),
            goal: goal.to_string(),
            current_level: Some(CurrentLevel::Intermediate),
            candidate_topics,
            constraints: None,
            resource_context: Some(json!({"prefetched": false})),
        };
        let (topic_plan, assumptions) = topic_plan_from_candidates(&param);
        let blueprint = blueprint_from_topic_plan(&param, &topic_plan);
        let profile = GoalProfile {
            category: crate::domain::GoalCategory::CustomTopic,
            domain: "candidate_topics".to_string(),
            stack: vec![],
            target_role: None,
            level: CurrentLevel::Intermediate,
            desired_outcome: Some(goal.to_string()),
            normalized_goal: goal.to_string(),
            warnings: vec![],
        };
        let bound = topic_plan.iter().map(good_bound_topic).collect::<Vec<_>>();
        let graph = graph_builder::build_roadmap_graph(
            Some("user_eval".to_string()),
            None,
            &profile,
            &blueprint,
            &bound,
        );
        let validation = graph_validator::validate_roadmap_graph(&graph);
        let payload = persistence_payload_builder::build_database_ready_payload(&graph);

        assert_eq!(
            blueprint.blueprint_id, "candidate_topics_adaptive",
            "{goal}"
        );
        assert!(
            assumptions.iter().any(|item| item.contains("primary")),
            "{goal}"
        );
        assert!(validation.valid, "{goal}: {:?}", validation.errors);
        assert_eq!(graph.nodes.len(), topic_plan.len(), "{goal}");
        assert!(
            graph.phases.iter().all(|phase| phase.node_ids.len() <= 5),
            "{goal}: phase too large"
        );
        assert_eq!(payload.schema_version, "roadmap_draft_v1", "{goal}");
        assert!(payload.not_persisted, "{goal}");
        assert!(
            graph.nodes.iter().all(|node| param
                .candidate_topics
                .iter()
                .any(|topic| topic.name == node.topic)),
            "{goal}: candidate topic was discarded or replaced"
        );
    }
}

#[test]
fn phase_14_candidate_topic_planning_deduplicates_without_fabricating_topics() {
    let param = PlanRoadmapFromTopicsParam {
        user_id: None,
        goal: "Learn duplicate topic handling".to_string(),
        current_level: Some(CurrentLevel::Intermediate),
        candidate_topics: vec![
            topic("Redis Streams", "Main target", vec![]),
            topic("redis streams", "Duplicate spelling", vec![]),
            topic("Consumer groups", "Dependent topic", vec!["Redis Streams"]),
        ],
        constraints: None,
        resource_context: None,
    };

    let (topic_plan, assumptions) = topic_plan_from_candidates(&param);

    assert_eq!(topic_plan.len(), 2);
    assert!(
        assumptions
            .iter()
            .any(|assumption| assumption.contains("Duplicate candidate topic skipped"))
    );
    assert!(
        topic_plan
            .iter()
            .any(|topic| topic.topic_name == "Redis Streams")
    );
    assert!(
        topic_plan
            .iter()
            .any(|topic| topic.topic_name == "Consumer groups")
    );
}

#[test]
fn phase_13_poor_coverage_produces_non_misleading_roadmap() {
    let profile = goal_profile("Learn backend with Node.js and PostgreSQL");
    let selection = blueprint_registry::select_blueprint(&profile);
    let topics = topic_decomposer::decompose_topics(&selection.blueprint, None);
    let bound_topics = topics
        .iter()
        .enumerate()
        .map(|(index, topic)| {
            if index == 0 {
                poor_bound_topic(topic)
            } else {
                good_bound_topic(topic)
            }
        })
        .collect::<Vec<_>>();
    let graph = graph_builder::build_roadmap_graph(
        Some("user_eval".to_string()),
        None,
        &profile,
        &selection.blueprint,
        &bound_topics,
    );
    let validation = graph_validator::validate_roadmap_graph(&graph);

    assert!(validation.valid, "{:?}", validation.errors);
    assert!(matches!(graph.status, RoadmapStatus::NeedsResourceBackfill));
    assert_eq!(graph.coverage_summary.coverage_poor, 1);
    assert!(!graph.gap_warnings.is_empty());
    assert!(
        graph
            .nodes
            .iter()
            .filter(|node| matches!(node.coverage_status, CoverageStatus::Poor))
            .all(|node| !node.warnings.is_empty() && !node.missing_resource_types.is_empty())
    );
}

#[test]
fn phase_13_unsupported_goal_uses_custom_blueprint_without_inventing_runtime_structure() {
    let profile = goal_profile("Learn a niche internal deployment tool");
    let selection = blueprint_registry::select_blueprint(&profile);
    let topics = topic_decomposer::decompose_topics(&selection.blueprint, None);

    assert_eq!(selection.blueprint.blueprint_id, "custom_topic_linear");
    assert!(!topics.is_empty());
    assert!(
        selection
            .warnings
            .iter()
            .any(|warning| warning.contains("custom_topic_linear"))
    );
}

#[test]
fn phase_13_contract_and_persistence_boundary_are_explicit() {
    let contract = contract::roadmap_contract();
    let policy = &contract["database_persistence_policy"];

    assert_eq!(contract["version"], contract::ROADMAP_CONTRACT_VERSION);
    assert_eq!(contract["planned_tools"].as_array().unwrap().len(), 0);
    assert_eq!(policy["roadmap_mcp_calls_database_mcp"], false);
    assert_eq!(policy["persistence_owner"], "orchestrator_agent");
    assert!(
        contract["integration_rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|rule| rule == "Roadmap MCP must not write directly to application DB when Database MCP is the persistence layer.")
    );
}

#[test]
fn phase_13_failure_modes_are_caught_by_validation() {
    let invalid = request_validator::validate_roadmap_request(RoadmapGenerationRequest {
        user_id: Some("user_eval".to_string()),
        project_id: None,
        learning_goal: "Learn backend".to_string(),
        current_level: Some(CurrentLevel::Beginner),
        target_role: Some(TargetRole::Backend),
        preferred_language: Some("en".to_string()),
        time_budget: Some(TimeBudget {
            hours_per_week: Some(10),
            target_weeks: Some(10),
            max_total_hours: Some(40),
        }),
        constraints: Some(RoadmapConstraints {
            prefer_official_docs: None,
            prefer_project_based: None,
            include_practice: None,
            avoid_advanced_math: None,
            target_stack: Some(vec!["Node.js".to_string()]),
            excluded_topics: Some(vec!["node.js".to_string()]),
        }),
        save_mode: Some(SaveMode::Draft),
    });

    assert!(!invalid.valid);
    assert!(
        invalid
            .validation_errors
            .iter()
            .any(|issue| issue.code == "CONTRADICTORY_TIME_BUDGET")
    );
    assert!(
        invalid
            .validation_errors
            .iter()
            .any(|issue| issue.code == "CONTRADICTORY_CONSTRAINTS")
    );
}

#[test]
fn phase_13_cycle_injected_into_graph_is_rejected() {
    let mut graph = evaluated_graph(
        "Learn backend with Node.js and PostgreSQL",
        CurrentLevel::Beginner,
        Some(TargetRole::Backend),
    );
    let first = graph.nodes[0].node_id.clone();
    let second = graph.nodes[1].node_id.clone();

    graph.edges.push(crate::domain::RoadmapEdge {
        from_node_id: first.clone(),
        to_node_id: second.clone(),
        edge_type: crate::domain::EdgeType::Prerequisite,
        reason: "Injected test edge".to_string(),
    });
    graph.edges.push(crate::domain::RoadmapEdge {
        from_node_id: second,
        to_node_id: first,
        edge_type: crate::domain::EdgeType::Prerequisite,
        reason: "Injected cycle".to_string(),
    });

    let validation = graph_validator::validate_roadmap_graph(&graph);
    assert!(!validation.valid);
    assert!(
        validation
            .errors
            .iter()
            .any(|issue| issue.code == "PREREQUISITE_CYCLE")
    );
}

fn evaluated_graph(
    learning_goal: &str,
    current_level: CurrentLevel,
    target_role: Option<TargetRole>,
) -> RoadmapGraph {
    let validation = request_validator::validate_roadmap_request(RoadmapGenerationRequest {
        user_id: Some("user_eval".to_string()),
        project_id: None,
        learning_goal: learning_goal.to_string(),
        current_level: Some(current_level),
        target_role,
        preferred_language: Some("en".to_string()),
        time_budget: Some(TimeBudget {
            hours_per_week: Some(8),
            target_weeks: Some(12),
            max_total_hours: Some(120),
        }),
        constraints: None,
        save_mode: Some(SaveMode::Draft),
    });

    assert!(
        validation.valid,
        "{learning_goal}: {:?}",
        validation.validation_errors
    );
    let profile = validation.goal_profile.unwrap();
    let selection = blueprint_registry::select_blueprint(&profile);
    let topics = topic_decomposer::decompose_topics(&selection.blueprint, None);
    let bound_topics = topics.iter().map(good_bound_topic).collect::<Vec<_>>();

    graph_builder::build_roadmap_graph(
        Some("user_eval".to_string()),
        None,
        &profile,
        &selection.blueprint,
        &bound_topics,
    )
}

fn goal_profile(goal: &str) -> GoalProfile {
    let validation = request_validator::validate_roadmap_request(RoadmapGenerationRequest {
        user_id: Some("user_eval".to_string()),
        project_id: None,
        learning_goal: goal.to_string(),
        current_level: Some(CurrentLevel::Beginner),
        target_role: None,
        preferred_language: Some("en".to_string()),
        time_budget: None,
        constraints: None,
        save_mode: Some(SaveMode::Draft),
    });

    assert!(validation.valid);
    validation.goal_profile.unwrap()
}

fn good_bound_topic(topic: &TopicPlan) -> BoundTopicPlan {
    BoundTopicPlan {
        topic_plan: topic.clone(),
        coverage: CoverageCheckResult {
            coverage_status: CoverageStatus::Good,
            available_types: topic.required_resource_types.clone(),
            missing_types: vec![],
            confidence: Some(0.95),
            candidate_resource_count: Some(3),
            gap_id: None,
            raw: json!({"coverageStatus": "good"}),
        },
        resource_refs: vec![ResourceBinding {
            resource_id: format!("res_{}", topic.topic_id),
            title: format!("{} reference", topic.topic_name),
            canonical_url: "https://example.com/resource".to_string(),
            source_domain: Some("example.com".to_string()),
            kind: topic
                .required_resource_types
                .first()
                .cloned()
                .unwrap_or_else(|| "primary_learning".to_string()),
            format: Some("docs".to_string()),
            language_code: Some("en".to_string()),
            is_official: true,
            quality_score: Some(0.9),
            trust_tier: Some(1),
            coverage_role: CoverageRole::Primary,
            selected_chunks: None,
        }],
        missing_resource_types: vec![],
        warnings: vec![],
        status: NodeStatus::Ready,
        gap_reported: false,
        research_requested: false,
    }
}

fn topic(name: &str, reason: &str, prerequisites: Vec<&str>) -> CandidateTopicInput {
    CandidateTopicInput {
        name: name.to_string(),
        reason: Some(reason.to_string()),
        prerequisites: Some(prerequisites.into_iter().map(str::to_string).collect()),
        required_resource_types: Some(vec![
            "primary_learning".to_string(),
            "official_reference".to_string(),
        ]),
        estimated_hours: Some(4),
    }
}

fn poor_bound_topic(topic: &TopicPlan) -> BoundTopicPlan {
    BoundTopicPlan {
        topic_plan: topic.clone(),
        coverage: CoverageCheckResult {
            coverage_status: CoverageStatus::Poor,
            available_types: vec![],
            missing_types: topic.required_resource_types.clone(),
            confidence: Some(0.2),
            candidate_resource_count: Some(0),
            gap_id: Some("gap_eval".to_string()),
            raw: json!({"coverageStatus": "poor"}),
        },
        resource_refs: vec![],
        missing_resource_types: topic.required_resource_types.clone(),
        warnings: vec!["Coverage is poor; node should wait for Resource backfill.".to_string()],
        status: NodeStatus::Blocked,
        gap_reported: true,
        research_requested: true,
    }
}
