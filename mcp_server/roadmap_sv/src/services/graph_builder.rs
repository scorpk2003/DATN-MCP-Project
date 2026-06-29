#![allow(dead_code)]

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::domain::{
    BoundTopicPlan, CoverageRole, CoverageStatus, CoverageSummary, EdgeType, GoalProfile,
    RoadmapBlueprint, RoadmapEdge, RoadmapGraph, RoadmapNode, RoadmapPhase, RoadmapStatus,
};

pub fn build_roadmap_graph(
    user_id: Option<String>,
    project_id: Option<String>,
    goal: &GoalProfile,
    blueprint: &RoadmapBlueprint,
    bound_topics: &[BoundTopicPlan],
) -> RoadmapGraph {
    let topic_to_node = topic_to_node_map(bound_topics);
    let node_phase = node_phase_map(blueprint, bound_topics);
    let nodes = build_nodes(bound_topics, &node_phase, &topic_to_node);
    let phases = build_phases(blueprint, &nodes);
    let edges = build_edges(bound_topics, &topic_to_node);
    let coverage_summary = coverage_summary(bound_topics);
    let resource_summary = resource_summary(bound_topics);
    let gap_warnings = gap_warnings(bound_topics);
    let status = roadmap_status(&coverage_summary);

    RoadmapGraph {
        roadmap_id: None,
        user_id,
        project_id,
        status,
        metadata: json!({
            "blueprintId": blueprint.blueprint_id,
            "normalizedGoal": goal.normalized_goal,
            "domain": goal.domain,
            "stack": goal.stack,
            "targetRole": goal.target_role,
            "estimatedHoursRange": blueprint.estimated_hours_range,
        }),
        phases,
        nodes,
        edges,
        coverage_summary,
        resource_summary,
        gap_warnings,
        validation_result: None,
    }
}

fn build_nodes(
    bound_topics: &[BoundTopicPlan],
    node_phase: &BTreeMap<String, String>,
    topic_to_node: &BTreeMap<String, String>,
) -> Vec<RoadmapNode> {
    bound_topics
        .iter()
        .map(|bound| {
            let topic = &bound.topic_plan;
            let node_id = topic_to_node
                .get(&topic.topic_name)
                .cloned()
                .unwrap_or_else(|| topic.topic_id.clone());
            let phase_id = node_phase
                .get(&topic.topic_name)
                .cloned()
                .unwrap_or_else(|| "unassigned".to_string());

            RoadmapNode {
                node_id,
                phase_id,
                title: topic.topic_name.clone(),
                topic: topic.topic_name.clone(),
                aliases: topic.aliases.clone(),
                node_type: topic.node_type.clone(),
                level: topic.level.clone(),
                purpose: purpose_for_topic(&topic.topic_name),
                learning_outcomes: learning_outcomes(&topic.topic_name),
                prerequisites: topic
                    .prerequisite_topics
                    .iter()
                    .filter_map(|name| topic_to_node.get(name).cloned())
                    .collect(),
                estimated_hours: topic.estimated_hours_hint.unwrap_or(3),
                coverage_status: bound.coverage.coverage_status.clone(),
                resource_refs: bound.resource_refs.clone(),
                missing_resource_types: bound.missing_resource_types.clone(),
                warnings: bound.warnings.clone(),
                status: bound.status.clone(),
            }
        })
        .collect()
}

fn build_phases(blueprint: &RoadmapBlueprint, nodes: &[RoadmapNode]) -> Vec<RoadmapPhase> {
    blueprint
        .phases
        .iter()
        .enumerate()
        .map(|(index, phase)| {
            let node_ids = nodes
                .iter()
                .filter(|node| node.phase_id == phase.phase_id)
                .map(|node| node.node_id.clone())
                .collect::<Vec<_>>();
            let estimated_hours = nodes
                .iter()
                .filter(|node| node.phase_id == phase.phase_id)
                .map(|node| node.estimated_hours)
                .sum();

            RoadmapPhase {
                phase_id: phase.phase_id.clone(),
                title: phase.title.clone(),
                purpose: phase.purpose.clone(),
                order_index: index as u32 + 1,
                estimated_hours,
                node_ids,
                exit_criteria: vec![
                    "Complete ready nodes or explicitly acknowledge blocked/partial nodes."
                        .to_string(),
                    "Review missing resource warnings before lesson generation.".to_string(),
                ],
            }
        })
        .collect()
}

fn build_edges(
    bound_topics: &[BoundTopicPlan],
    topic_to_node: &BTreeMap<String, String>,
) -> Vec<RoadmapEdge> {
    bound_topics
        .iter()
        .flat_map(|bound| {
            let to_topic = &bound.topic_plan.topic_name;
            let to_node_id = topic_to_node.get(to_topic).cloned();
            bound
                .topic_plan
                .prerequisite_topics
                .iter()
                .filter_map(move |from_topic| {
                    let from_node_id = topic_to_node.get(from_topic)?.clone();
                    let to_node_id = to_node_id.clone()?;
                    Some(RoadmapEdge {
                        from_node_id,
                        to_node_id,
                        edge_type: EdgeType::Prerequisite,
                        reason: format!("{from_topic} should be learned before {to_topic}."),
                    })
                })
        })
        .collect()
}

fn coverage_summary(bound_topics: &[BoundTopicPlan]) -> CoverageSummary {
    let mut summary = CoverageSummary {
        total_topics: bound_topics.len() as u32,
        ready_for_lesson_generation: true,
        ..CoverageSummary::default()
    };

    for bound in bound_topics {
        match bound.coverage.coverage_status {
            CoverageStatus::Good => summary.coverage_good += 1,
            CoverageStatus::Partial => {
                summary.coverage_partial += 1;
                summary.ready_for_lesson_generation = false;
            }
            CoverageStatus::Poor => {
                summary.coverage_poor += 1;
                summary.ready_for_lesson_generation = false;
            }
        }

        if bound
            .missing_resource_types
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case("official_reference"))
        {
            summary.missing_official_reference_count += 1;
        }
        if bound
            .missing_resource_types
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case("practice"))
        {
            summary.missing_practice_count += 1;
        }
        if bound
            .missing_resource_types
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case("project"))
        {
            summary.missing_project_count += 1;
        }
        if bound.gap_reported {
            summary.gaps_created += 1;
        }
        if bound.research_requested {
            summary.research_tasks_requested += 1;
        }
        if bound.resource_refs.is_empty() {
            summary.ready_for_lesson_generation = false;
        }
    }

    summary
}

fn resource_summary(bound_topics: &[BoundTopicPlan]) -> Value {
    let resource_count = bound_topics
        .iter()
        .map(|bound| bound.resource_refs.len() as u32)
        .sum::<u32>();
    let official_count = bound_topics
        .iter()
        .flat_map(|bound| bound.resource_refs.iter())
        .filter(|resource| resource.is_official)
        .count() as u32;
    let primary_count = bound_topics
        .iter()
        .flat_map(|bound| bound.resource_refs.iter())
        .filter(|resource| matches!(resource.coverage_role, CoverageRole::Primary))
        .count() as u32;
    let practice_count = bound_topics
        .iter()
        .flat_map(|bound| bound.resource_refs.iter())
        .filter(|resource| matches!(resource.coverage_role, CoverageRole::Practice))
        .count() as u32;

    json!({
        "totalResourceRefs": resource_count,
        "officialResourceRefs": official_count,
        "primaryResourceRefs": primary_count,
        "practiceResourceRefs": practice_count,
    })
}

fn gap_warnings(bound_topics: &[BoundTopicPlan]) -> Vec<String> {
    bound_topics
        .iter()
        .filter(|bound| matches!(bound.coverage.coverage_status, CoverageStatus::Poor))
        .map(|bound| {
            format!(
                "{} needs resource backfill: missing {}.",
                bound.topic_plan.topic_name,
                if bound.missing_resource_types.is_empty() {
                    "required resource types".to_string()
                } else {
                    bound.missing_resource_types.join(", ")
                }
            )
        })
        .collect()
}

fn roadmap_status(summary: &CoverageSummary) -> RoadmapStatus {
    if summary.coverage_poor > 0 {
        RoadmapStatus::NeedsResourceBackfill
    } else if summary.coverage_partial > 0 || !summary.ready_for_lesson_generation {
        RoadmapStatus::Incomplete
    } else {
        RoadmapStatus::Draft
    }
}

fn topic_to_node_map(bound_topics: &[BoundTopicPlan]) -> BTreeMap<String, String> {
    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
    let mut map = BTreeMap::new();

    for bound in bound_topics {
        let base = bound.topic_plan.topic_id.clone();
        let count = seen.entry(base.clone()).or_default();
        *count += 1;
        let node_id = if *count == 1 {
            base
        } else {
            format!("{base}-{count}")
        };
        map.insert(bound.topic_plan.topic_name.clone(), node_id);
    }

    map
}

fn node_phase_map(
    blueprint: &RoadmapBlueprint,
    bound_topics: &[BoundTopicPlan],
) -> BTreeMap<String, String> {
    let group_by_topic = blueprint
        .topic_groups
        .iter()
        .flat_map(|group| {
            group
                .topics
                .iter()
                .map(move |topic| (topic.clone(), group.group_id.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let phase_by_group = blueprint
        .phases
        .iter()
        .flat_map(|phase| {
            phase
                .topic_group_ids
                .iter()
                .map(move |group_id| (group_id.clone(), phase.phase_id.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    bound_topics
        .iter()
        .map(|bound| {
            let phase_id = group_by_topic
                .get(&bound.topic_plan.topic_name)
                .and_then(|group_id| phase_by_group.get(group_id))
                .cloned()
                .unwrap_or_else(|| "unassigned".to_string());
            (bound.topic_plan.topic_name.clone(), phase_id)
        })
        .collect()
}

fn purpose_for_topic(topic: &str) -> String {
    format!("Learn and apply {topic} in the context of the selected roadmap.")
}

fn learning_outcomes(topic: &str) -> Vec<String> {
    vec![
        format!("Explain the purpose of {topic}."),
        format!("Apply {topic} in a small learning task."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            CoverageCheckResult, CurrentLevel, GoalCategory, GoalProfile, NodeStatus,
            RoadmapNodeType, TargetRole, TopicPlan,
        },
        services::{blueprint_registry::select_blueprint, topic_decomposer::decompose_topics},
    };

    #[test]
    fn builds_graph_with_phases_nodes_and_edges() {
        let goal = GoalProfile {
            category: GoalCategory::Backend,
            domain: "backend".to_string(),
            stack: vec!["Node.js".to_string(), "PostgreSQL".to_string()],
            target_role: Some(TargetRole::Backend),
            level: CurrentLevel::Beginner,
            desired_outcome: None,
            normalized_goal: "learn backend with node and postgres".to_string(),
            warnings: vec![],
        };
        let selection = select_blueprint(&goal);
        let topics = decompose_topics(&selection.blueprint, None);
        let bound = topics
            .iter()
            .map(|topic| ready_bound_topic(topic.clone()))
            .collect::<Vec<_>>();

        let graph = build_roadmap_graph(
            Some("user_1".to_string()),
            None,
            &goal,
            &selection.blueprint,
            &bound,
        );

        assert_eq!(graph.user_id, Some("user_1".to_string()));
        assert!(!graph.phases.is_empty());
        assert_eq!(graph.nodes.len(), bound.len());
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.reason.contains("before"))
        );
        assert_eq!(graph.coverage_summary.coverage_good, bound.len() as u32);
    }

    #[test]
    fn poor_coverage_marks_graph_needs_backfill() {
        let goal = GoalProfile {
            category: GoalCategory::CustomTopic,
            domain: "custom_topic".to_string(),
            stack: vec![],
            target_role: None,
            level: CurrentLevel::Unknown,
            desired_outcome: None,
            normalized_goal: "learn niche topic".to_string(),
            warnings: vec![],
        };
        let selection = select_blueprint(&goal);
        let topic = TopicPlan {
            topic_id: "niche-topic".to_string(),
            topic_name: "Niche topic".to_string(),
            aliases: vec![],
            level: CurrentLevel::Unknown,
            required_resource_types: vec!["primary_learning".to_string()],
            node_type: RoadmapNodeType::Concept,
            estimated_hours_hint: Some(3),
            prerequisite_topics: vec![],
            optional: false,
        };
        let bound = vec![BoundTopicPlan {
            topic_plan: topic,
            coverage: CoverageCheckResult {
                coverage_status: CoverageStatus::Poor,
                available_types: vec![],
                missing_types: vec!["primary_learning".to_string()],
                confidence: Some(0.1),
                candidate_resource_count: Some(0),
                gap_id: None,
                raw: json!({}),
            },
            resource_refs: vec![],
            missing_resource_types: vec!["primary_learning".to_string()],
            warnings: vec!["missing".to_string()],
            status: NodeStatus::Blocked,
            gap_reported: true,
            research_requested: true,
        }];

        let graph = build_roadmap_graph(None, None, &goal, &selection.blueprint, &bound);

        assert!(matches!(graph.status, RoadmapStatus::NeedsResourceBackfill));
        assert_eq!(graph.coverage_summary.coverage_poor, 1);
        assert_eq!(graph.coverage_summary.gaps_created, 1);
        assert!(!graph.gap_warnings.is_empty());
    }

    fn ready_bound_topic(topic: TopicPlan) -> BoundTopicPlan {
        BoundTopicPlan {
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
            resource_refs: vec![crate::domain::ResourceBinding {
                resource_id: "res_1".to_string(),
                title: "Resource".to_string(),
                canonical_url: "https://example.com".to_string(),
                source_domain: Some("example.com".to_string()),
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
        }
    }
}
