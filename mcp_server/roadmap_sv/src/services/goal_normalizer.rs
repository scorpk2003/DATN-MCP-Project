use crate::domain::{
    CurrentLevel, GoalCategory, GoalProfile, RoadmapConstraints, RoadmapGenerationRequest,
    TargetRole,
};

pub fn normalize_goal(request: &RoadmapGenerationRequest) -> GoalProfile {
    let goal = compact_whitespace(&request.learning_goal);
    let lower_goal = goal.to_lowercase();
    let stack = extract_stack(&lower_goal, request.constraints.as_ref());
    let inferred_role = request
        .target_role
        .clone()
        .or_else(|| infer_target_role(&lower_goal, &stack));
    let category = infer_category(&lower_goal, &stack, inferred_role.as_ref());
    let domain = domain_for_category(&category).to_string();
    let level = request
        .current_level
        .clone()
        .unwrap_or(CurrentLevel::Unknown);
    let desired_outcome = infer_desired_outcome(&lower_goal, inferred_role.as_ref());
    let mut warnings = Vec::new();

    if matches!(category, GoalCategory::CustomTopic) {
        warnings.push("Goal category is custom_topic; blueprint selection will use fallback unless a specific blueprint matches.".to_string());
    }
    if stack.is_empty() {
        warnings.push("No target stack detected from goal or constraints.".to_string());
    }

    GoalProfile {
        category,
        domain,
        stack,
        target_role: inferred_role,
        level,
        desired_outcome,
        normalized_goal: goal,
        warnings,
    }
}

pub fn compact_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_stack(goal: &str, constraints: Option<&RoadmapConstraints>) -> Vec<String> {
    let candidates = [
        ("react", "React"),
        ("vue", "Vue"),
        ("angular", "Angular"),
        ("node.js", "Node.js"),
        ("nodejs", "Node.js"),
        ("node", "Node.js"),
        ("postgresql", "PostgreSQL"),
        ("postgres", "PostgreSQL"),
        ("python", "Python"),
        ("rust", "Rust"),
        ("docker", "Docker"),
        ("kubernetes", "Kubernetes"),
        ("javascript", "JavaScript"),
        ("typescript", "TypeScript"),
        ("sql", "SQL"),
    ];
    let mut stack = Vec::new();

    for (needle, label) in candidates {
        if goal.contains(needle) && !stack.iter().any(|item| item == label) {
            stack.push(label.to_string());
        }
    }

    if let Some(target_stack) =
        constraints.and_then(|constraints| constraints.target_stack.as_ref())
    {
        for item in target_stack {
            let item = compact_whitespace(item);
            if !item.is_empty()
                && !stack
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(&item))
            {
                stack.push(item);
            }
        }
    }

    stack
}

fn infer_target_role(goal: &str, stack: &[String]) -> Option<TargetRole> {
    if goal.contains("fullstack") || goal.contains("full stack") {
        Some(TargetRole::Fullstack)
    } else if goal.contains("frontend") || goal.contains("front-end") || stack_has(stack, "React") {
        Some(TargetRole::Frontend)
    } else if goal.contains("backend") || goal.contains("back-end") || stack_has(stack, "Node.js") {
        Some(TargetRole::Backend)
    } else if goal.contains("devops")
        || stack_has(stack, "Docker")
        || stack_has(stack, "Kubernetes")
    {
        Some(TargetRole::Devops)
    } else if goal.contains("data") {
        Some(TargetRole::Data)
    } else if goal.contains("ai") || goal.contains("machine learning") || goal.contains("ml") {
        Some(TargetRole::AiMl)
    } else {
        None
    }
}

fn infer_category(goal: &str, stack: &[String], role: Option<&TargetRole>) -> GoalCategory {
    match role {
        Some(TargetRole::Frontend) => GoalCategory::Frontend,
        Some(TargetRole::Backend) => GoalCategory::Backend,
        Some(TargetRole::Fullstack) => GoalCategory::Fullstack,
        Some(TargetRole::Devops) => GoalCategory::Devops,
        Some(TargetRole::GeneralSoftware) => GoalCategory::GeneralSoftware,
        _ if goal.contains("system design") => GoalCategory::SystemDesign,
        _ if goal.contains("database")
            || stack_has(stack, "PostgreSQL")
            || stack_has(stack, "SQL") =>
        {
            GoalCategory::Database
        }
        _ if stack_has(stack, "Rust")
            || stack_has(stack, "Python")
            || stack_has(stack, "JavaScript") =>
        {
            GoalCategory::ProgrammingLanguage
        }
        _ if goal.contains("web foundation") || goal.contains("html") || goal.contains("css") => {
            GoalCategory::WebFoundation
        }
        _ => GoalCategory::CustomTopic,
    }
}

fn domain_for_category(category: &GoalCategory) -> &'static str {
    match category {
        GoalCategory::Frontend => "frontend",
        GoalCategory::Backend => "backend",
        GoalCategory::Fullstack => "fullstack",
        GoalCategory::Devops => "devops",
        GoalCategory::Database => "database",
        GoalCategory::ProgrammingLanguage => "programming_language",
        GoalCategory::WebFoundation => "web_foundation",
        GoalCategory::SystemDesign => "system_design",
        GoalCategory::GeneralSoftware => "general_software",
        GoalCategory::CustomTopic => "custom_topic",
    }
}

fn infer_desired_outcome(goal: &str, role: Option<&TargetRole>) -> Option<String> {
    if goal.contains("build") {
        Some("build a working software artifact".to_string())
    } else {
        role.map(|role| match role {
            TargetRole::Frontend => "build frontend user interfaces".to_string(),
            TargetRole::Backend => "build backend services".to_string(),
            TargetRole::Fullstack => "build fullstack applications".to_string(),
            TargetRole::Devops => "operate and deploy software systems".to_string(),
            TargetRole::Data => "work with data workflows".to_string(),
            TargetRole::AiMl => "build AI or machine learning workflows".to_string(),
            TargetRole::GeneralSoftware | TargetRole::Custom => {
                "improve general software engineering capability".to_string()
            }
        })
    }
}

fn stack_has(stack: &[String], needle: &str) -> bool {
    stack.iter().any(|item| item.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_backend_node_postgres_goal() {
        let request = RoadmapGenerationRequest {
            user_id: None,
            project_id: None,
            learning_goal: " I want to learn backend with Node.js and PostgreSQL ".to_string(),
            current_level: Some(CurrentLevel::Beginner),
            target_role: None,
            preferred_language: None,
            time_budget: None,
            constraints: None,
            save_mode: None,
        };

        let profile = normalize_goal(&request);

        assert_eq!(profile.category, GoalCategory::Backend);
        assert_eq!(profile.target_role, Some(TargetRole::Backend));
        assert!(profile.stack.contains(&"Node.js".to_string()));
        assert!(profile.stack.contains(&"PostgreSQL".to_string()));
    }
}
