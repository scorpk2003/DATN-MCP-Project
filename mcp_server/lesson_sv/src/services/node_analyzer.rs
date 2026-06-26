use crate::domain::{
    ExerciseType, LessonAnalyzeNodeParam, LessonLevel, LessonRequirement, ResourceQuery, SourceType,
};

pub fn analyze_node(param: &LessonAnalyzeNodeParam) -> LessonRequirement {
    let known_topics = param
        .user_context
        .as_ref()
        .and_then(|context| context.known_topics.clone())
        .unwrap_or_default();
    let weak_topics = param
        .user_context
        .as_ref()
        .and_then(|context| context.weak_topics.clone())
        .unwrap_or_default();

    let mut prerequisite_gaps = param
        .node
        .prerequisites
        .iter()
        .filter(|prerequisite| !contains_case_insensitive(&known_topics, prerequisite))
        .cloned()
        .collect::<Vec<_>>();

    for weak_topic in weak_topics {
        if contains_case_insensitive(&param.node.prerequisites, &weak_topic)
            && !contains_case_insensitive(&prerequisite_gaps, &weak_topic)
        {
            prerequisite_gaps.push(weak_topic);
        }
    }

    let objectives = normalize_objectives(
        &param.node.topic,
        &param.node.description,
        &param.node.expected_outcomes,
    );

    LessonRequirement {
        topic: param.node.topic.clone(),
        level: param.node.level.clone(),
        resource_queries: build_resource_queries(
            &param.node.topic,
            &param.node.description,
            &param.node.level,
        ),
        recommended_exercise_types: recommended_exercise_types(
            &objectives,
            &param.node.topic,
            &param.node.level,
        ),
        estimated_minutes: estimate_minutes(
            &param.node.level,
            objectives.len(),
            prerequisite_gaps.len(),
        ),
        objectives,
        prerequisite_gaps,
    }
}

fn normalize_objectives(
    topic: &str,
    description: &str,
    expected_outcomes: &[String],
) -> Vec<String> {
    let mut objectives = expected_outcomes
        .iter()
        .map(|objective| objective.trim())
        .filter(|objective| !objective.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if objectives.is_empty() {
        objectives.push(format!("Understand the core concept of {topic}."));
        if !description.trim().is_empty() {
            objectives.push(format!("Apply {topic} in the context of {description}."));
        }
    }

    objectives
}

fn build_resource_queries(
    topic: &str,
    description: &str,
    level: &LessonLevel,
) -> Vec<ResourceQuery> {
    let difficulty = level.clone();
    let mut queries = vec![ResourceQuery {
        query: format!("{topic} {description} official documentation tutorial examples"),
        resource_types: vec![SourceType::Docs, SourceType::Article, SourceType::Github],
        difficulty: difficulty.clone(),
        must_have: true,
    }];

    if matches!(level, LessonLevel::Intermediate | LessonLevel::Advanced) {
        queries.push(ResourceQuery {
            query: format!("{topic} deep dive best practices common mistakes"),
            resource_types: vec![SourceType::Docs, SourceType::Article, SourceType::Paper],
            difficulty,
            must_have: false,
        });
    }

    queries
}

fn recommended_exercise_types(
    objectives: &[String],
    topic: &str,
    level: &LessonLevel,
) -> Vec<ExerciseType> {
    let joined = objectives.join(" ").to_lowercase();
    let topic = topic.to_lowercase();
    let mut types = Vec::new();

    if contains_any(
        &joined,
        &["implement", "build", "code", "debug", "query", "write"],
    ) {
        types.push(ExerciseType::Coding);
        types.push(ExerciseType::Debugging);
    }
    if contains_any(
        &joined,
        &["explain", "compare", "understand", "describe", "identify"],
    ) {
        types.push(ExerciseType::ShortAnswer);
        types.push(ExerciseType::MultipleChoice);
    }
    if contains_any(&joined, &["design", "architect", "model"]) || topic.contains("system design") {
        types.push(ExerciseType::Design);
    }
    if matches!(level, LessonLevel::Advanced) {
        types.push(ExerciseType::MiniProject);
    }
    if types.is_empty() {
        types.push(ExerciseType::ShortAnswer);
    }

    dedupe_exercise_types(types)
}

fn estimate_minutes(
    level: &LessonLevel,
    objective_count: usize,
    prerequisite_gap_count: usize,
) -> u32 {
    let base = match level {
        LessonLevel::Beginner => 30,
        LessonLevel::Intermediate => 45,
        LessonLevel::Advanced => 60,
    };
    let objective_extra = objective_count.saturating_sub(1) as u32 * 5;
    let prerequisite_extra = prerequisite_gap_count as u32 * 10;

    (base + objective_extra + prerequisite_extra).min(90)
}

fn contains_case_insensitive(items: &[String], candidate: &str) -> bool {
    items
        .iter()
        .any(|item| item.eq_ignore_ascii_case(candidate))
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn dedupe_exercise_types(types: Vec<ExerciseType>) -> Vec<ExerciseType> {
    let mut names = Vec::new();
    let mut deduped = Vec::new();

    for exercise_type in types {
        let name = format!("{exercise_type:?}");
        if !names.iter().any(|existing| existing == &name) {
            names.push(name);
            deduped.push(exercise_type);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RoadmapNodeInput, UserContextInput};

    #[test]
    fn detects_prerequisite_gaps_and_queries() {
        let param = LessonAnalyzeNodeParam {
            request_id: None,
            auth_context: None,
            user_id: "user".to_string(),
            roadmap_id: "roadmap".to_string(),
            roadmap_node_id: "node".to_string(),
            node: RoadmapNodeInput {
                title: "SQL joins".to_string(),
                topic: "SQL joins".to_string(),
                description: "Combine rows from multiple tables".to_string(),
                level: LessonLevel::Beginner,
                prerequisites: vec!["SQL basics".to_string()],
                expected_outcomes: vec!["Write inner join queries".to_string()],
            },
            user_context: Some(UserContextInput {
                skill_level: Some("beginner".to_string()),
                known_topics: Some(vec![]),
                weak_topics: Some(vec![]),
                learning_goal: None,
            }),
        };

        let requirement = analyze_node(&param);
        assert_eq!(requirement.prerequisite_gaps, vec!["SQL basics"]);
        assert!(!requirement.resource_queries.is_empty());
        assert!(requirement.estimated_minutes >= 40);
    }
}
