#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::domain::{CurrentLevel, LearnerContext, RoadmapBlueprint, RoadmapNodeType, TopicPlan};

pub fn decompose_topics(
    blueprint: &RoadmapBlueprint,
    learner_context: Option<&LearnerContext>,
) -> Vec<TopicPlan> {
    let prerequisite_map = prerequisite_map(blueprint);
    let known_topics = known_topic_set(learner_context);
    let completed_topics = completed_topic_set(learner_context);

    blueprint
        .topic_groups
        .iter()
        .flat_map(|group| {
            group.topics.iter().map(|topic| {
                let node_type = infer_node_type(topic, &group.required_resource_types);
                let known_or_completed = contains_normalized(&known_topics, topic)
                    || contains_normalized(&completed_topics, topic);
                let optional = should_mark_optional(blueprint, topic, known_or_completed);

                TopicPlan {
                    topic_id: topic_id(topic),
                    topic_name: topic.clone(),
                    aliases: aliases(topic),
                    level: blueprint.level.clone(),
                    required_resource_types: group.required_resource_types.clone(),
                    node_type: node_type.clone(),
                    estimated_hours_hint: Some(estimated_hours(&node_type, optional)),
                    prerequisite_topics: prerequisite_map.get(topic).cloned().unwrap_or_default(),
                    optional,
                }
            })
        })
        .collect()
}

fn prerequisite_map(blueprint: &RoadmapBlueprint) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for rule in &blueprint.prerequisite_rules {
        map.entry(rule.to.clone())
            .or_default()
            .push(rule.from.clone());
    }
    map
}

fn known_topic_set(learner_context: Option<&LearnerContext>) -> BTreeSet<String> {
    learner_context
        .map(|context| {
            context
                .known_skills
                .iter()
                .chain(context.current_level_by_topic.keys())
                .map(|value| normalize_key(value))
                .collect()
        })
        .unwrap_or_default()
}

fn completed_topic_set(learner_context: Option<&LearnerContext>) -> BTreeSet<String> {
    learner_context
        .map(|context| {
            context
                .completed_lessons
                .iter()
                .chain(context.completed_roadmaps.iter())
                .map(|value| normalize_key(value))
                .collect()
        })
        .unwrap_or_default()
}

fn should_mark_optional(
    blueprint: &RoadmapBlueprint,
    topic: &str,
    known_or_completed: bool,
) -> bool {
    if !known_or_completed {
        return false;
    }

    match blueprint.level {
        CurrentLevel::Beginner | CurrentLevel::Unknown => false,
        CurrentLevel::Intermediate | CurrentLevel::Advanced => !blueprint
            .prerequisite_rules
            .iter()
            .any(|rule| rule.from.eq_ignore_ascii_case(topic)),
    }
}

fn infer_node_type(topic: &str, required_types: &[String]) -> RoadmapNodeType {
    let lower = topic.to_ascii_lowercase();
    if required_types
        .iter()
        .any(|resource_type| resource_type.eq_ignore_ascii_case("project"))
        || lower.contains("project")
        || lower.contains("build ")
    {
        RoadmapNodeType::Project
    } else if required_types
        .iter()
        .any(|resource_type| resource_type.eq_ignore_ascii_case("practice"))
        || lower.contains("practice")
        || lower.contains("exercise")
    {
        RoadmapNodeType::Practice
    } else if lower.contains("basics")
        || lower.contains("foundation")
        || lower.contains("overview")
        || lower.contains("syntax")
    {
        RoadmapNodeType::Foundation
    } else if lower.contains("api")
        || lower.contains("connect")
        || lower.contains("docker")
        || lower.contains("deployment")
    {
        RoadmapNodeType::Skill
    } else {
        RoadmapNodeType::Concept
    }
}

fn estimated_hours(node_type: &RoadmapNodeType, optional: bool) -> u32 {
    let base = match node_type {
        RoadmapNodeType::Foundation => 2,
        RoadmapNodeType::Concept => 3,
        RoadmapNodeType::Skill => 5,
        RoadmapNodeType::Practice => 6,
        RoadmapNodeType::Project => 12,
        RoadmapNodeType::Checkpoint => 1,
        RoadmapNodeType::Review => 1,
    };

    if optional { base.min(2) } else { base }
}

fn aliases(topic: &str) -> Vec<String> {
    let lower = topic.to_ascii_lowercase();
    let compact = lower.replace([' ', '-', '_', '/'], "");
    let slug = topic_id(topic);
    let mut aliases = vec![lower, slug];
    if compact != aliases[0] && compact != aliases[1] {
        aliases.push(compact);
    }
    aliases
}

fn topic_id(topic: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for ch in topic.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }

    output.trim_matches('-').to_string()
}

fn contains_normalized(values: &BTreeSet<String>, topic: &str) -> bool {
    values.contains(&normalize_key(topic))
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CurrentLevel, GoalCategory, GoalProfile, LearnerContext, TargetRole},
        services::blueprint_registry::select_blueprint,
    };

    #[test]
    fn decomposes_backend_blueprint_into_topic_plans() {
        let selection = select_blueprint(&GoalProfile {
            category: GoalCategory::Backend,
            domain: "backend".to_string(),
            stack: vec!["Node.js".to_string(), "PostgreSQL".to_string()],
            target_role: Some(TargetRole::Backend),
            level: CurrentLevel::Beginner,
            desired_outcome: None,
            normalized_goal: "learn backend with node and postgres".to_string(),
            warnings: vec![],
        });

        let topics = decompose_topics(&selection.blueprint, None);

        assert!(topics.len() >= 10);
        assert!(
            topics
                .iter()
                .any(|topic| topic.topic_name == "REST API design")
        );
        let joins = topics
            .iter()
            .find(|topic| topic.topic_name == "SQL joins")
            .unwrap();
        assert_eq!(joins.prerequisite_topics, vec!["PostgreSQL SELECT"]);
    }

    #[test]
    fn beginner_keeps_known_prerequisite_required() {
        let selection = select_blueprint(&GoalProfile {
            category: GoalCategory::Backend,
            domain: "backend".to_string(),
            stack: vec!["Node.js".to_string(), "PostgreSQL".to_string()],
            target_role: Some(TargetRole::Backend),
            level: CurrentLevel::Beginner,
            desired_outcome: None,
            normalized_goal: "learn backend".to_string(),
            warnings: vec![],
        });
        let context = LearnerContext {
            known_skills: vec!["HTTP basics".to_string()],
            ..LearnerContext::default()
        };

        let topics = decompose_topics(&selection.blueprint, Some(&context));
        let http = topics
            .iter()
            .find(|topic| topic.topic_name == "HTTP basics")
            .unwrap();

        assert!(!http.optional);
    }

    #[test]
    fn intermediate_marks_known_leaf_topic_optional() {
        let mut selection = select_blueprint(&GoalProfile {
            category: GoalCategory::Backend,
            domain: "backend".to_string(),
            stack: vec!["Node.js".to_string(), "PostgreSQL".to_string()],
            target_role: Some(TargetRole::Backend),
            level: CurrentLevel::Intermediate,
            desired_outcome: None,
            normalized_goal: "learn backend".to_string(),
            warnings: vec![],
        });
        selection.blueprint.level = CurrentLevel::Intermediate;
        let context = LearnerContext {
            known_skills: vec!["Authentication basics".to_string()],
            ..LearnerContext::default()
        };

        let topics = decompose_topics(&selection.blueprint, Some(&context));
        let auth = topics
            .iter()
            .find(|topic| topic.topic_name == "Authentication basics")
            .unwrap();

        assert!(auth.optional);
        assert_eq!(auth.estimated_hours_hint, Some(2));
    }
}
