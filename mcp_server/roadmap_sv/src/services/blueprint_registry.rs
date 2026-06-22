#![allow(dead_code)]

use crate::domain::{
    BlueprintPhaseTemplate, BlueprintSelection, BlueprintTopicGroup, CurrentLevel,
    EstimatedHoursRange, GoalCategory, GoalProfile, PrerequisiteRule, RoadmapBlueprint, TargetRole,
};

pub fn supported_blueprint_ids() -> Vec<&'static str> {
    vec![
        "frontend_web_beginner",
        "backend_node_postgres_beginner",
        "backend_python_postgres_beginner",
        "fullstack_react_node_postgres",
        "devops_docker_kubernetes_beginner",
        "rust_foundation",
        "python_foundation",
        "web_foundation",
        "postgres_database_foundation",
        "custom_topic_linear",
    ]
}

pub fn all_blueprints() -> Vec<RoadmapBlueprint> {
    vec![
        frontend_web_beginner(),
        backend_node_postgres_beginner(),
        backend_python_postgres_beginner(),
        fullstack_react_node_postgres(),
        devops_docker_kubernetes_beginner(),
        rust_foundation(),
        python_foundation(),
        web_foundation(),
        postgres_database_foundation(),
        custom_topic_linear(),
    ]
}

pub fn select_blueprint(goal: &GoalProfile) -> BlueprintSelection {
    let blueprint_id = match goal.category {
        GoalCategory::Fullstack if has_stack(goal, "React") && has_stack(goal, "Node.js") => {
            "fullstack_react_node_postgres"
        }
        GoalCategory::Backend if has_stack(goal, "Node.js") && has_stack(goal, "PostgreSQL") => {
            "backend_node_postgres_beginner"
        }
        GoalCategory::Backend if has_stack(goal, "Python") && has_stack(goal, "PostgreSQL") => {
            "backend_python_postgres_beginner"
        }
        GoalCategory::Frontend if has_stack(goal, "React") => "frontend_web_beginner",
        GoalCategory::Devops if has_stack(goal, "Docker") || has_stack(goal, "Kubernetes") => {
            "devops_docker_kubernetes_beginner"
        }
        GoalCategory::ProgrammingLanguage if has_stack(goal, "Rust") => "rust_foundation",
        GoalCategory::ProgrammingLanguage if has_stack(goal, "Python") => "python_foundation",
        GoalCategory::Database if has_stack(goal, "PostgreSQL") || has_stack(goal, "SQL") => {
            "postgres_database_foundation"
        }
        GoalCategory::WebFoundation => "web_foundation",
        _ => "custom_topic_linear",
    };

    let blueprint = all_blueprints()
        .into_iter()
        .find(|blueprint| blueprint.blueprint_id == blueprint_id)
        .unwrap_or_else(custom_topic_linear);

    let mut warnings = Vec::new();
    if blueprint.blueprint_id == "custom_topic_linear" {
        warnings.push(
            "No specific blueprint matched; using custom_topic_linear and requiring Resource coverage gating."
                .to_string(),
        );
    }

    BlueprintSelection {
        selection_reason: format!(
            "Selected {} for normalized domain {} and stack [{}].",
            blueprint.blueprint_id,
            goal.domain,
            goal.stack.join(", ")
        ),
        blueprint,
        warnings,
    }
}

fn frontend_web_beginner() -> RoadmapBlueprint {
    blueprint(
        "frontend_web_beginner",
        GoalCategory::Frontend,
        Some(TargetRole::Frontend),
        phases(&[
            (
                "foundation",
                "Web Foundations",
                "Understand the browser platform.",
                &["web"],
            ),
            (
                "core",
                "Frontend Core",
                "Build interactive UI foundations.",
                &["frontend_core"],
            ),
            (
                "practice",
                "Practice",
                "Apply frontend concepts in small builds.",
                &["frontend_practice"],
            ),
        ]),
        groups(&[
            (
                "web",
                "Web Basics",
                &["HTML semantics", "CSS layout", "JavaScript basics"],
                &["official_reference", "primary_learning"],
            ),
            (
                "frontend_core",
                "Frontend Core",
                &["DOM events", "React components", "React state"],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "frontend_practice",
                "Frontend Practice",
                &["Build responsive page", "Build interactive component"],
                &["practice", "project"],
            ),
        ]),
        rules(&[
            ("HTML semantics", "CSS layout"),
            ("JavaScript basics", "React components"),
        ]),
        (30, 60),
    )
}

fn backend_node_postgres_beginner() -> RoadmapBlueprint {
    blueprint(
        "backend_node_postgres_beginner",
        GoalCategory::Backend,
        Some(TargetRole::Backend),
        phases(&[
            (
                "foundation",
                "Web and JavaScript Foundation",
                "Prepare backend prerequisites.",
                &["web_js"],
            ),
            (
                "backend",
                "Backend Fundamentals",
                "Build HTTP APIs with Node.js.",
                &["node_backend"],
            ),
            (
                "database",
                "Database Foundation",
                "Use PostgreSQL safely.",
                &["postgres"],
            ),
            (
                "integration",
                "Application Integration",
                "Connect API and database.",
                &["node_postgres"],
            ),
            (
                "project",
                "Practice Project",
                "Build and package a backend project.",
                &["backend_project"],
            ),
        ]),
        groups(&[
            (
                "web_js",
                "Web and JavaScript",
                &[
                    "HTTP basics",
                    "JavaScript async/await",
                    "Node.js runtime basics",
                ],
                &["official_reference", "primary_learning"],
            ),
            (
                "node_backend",
                "Node Backend",
                &[
                    "Node.js HTTP server",
                    "REST API design",
                    "Error handling",
                    "Environment variables and configuration",
                ],
                &["primary_learning", "practice"],
            ),
            (
                "postgres",
                "PostgreSQL",
                &[
                    "PostgreSQL SELECT",
                    "SQL joins",
                    "PostgreSQL constraints",
                    "PostgreSQL indexes",
                ],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "node_postgres",
                "Node PostgreSQL Integration",
                &[
                    "Connect Node.js to PostgreSQL",
                    "CRUD API",
                    "Authentication basics",
                ],
                &["primary_learning", "practice"],
            ),
            (
                "backend_project",
                "Backend Project",
                &[
                    "Build REST API project",
                    "Add persistence",
                    "Containerize app",
                ],
                &["project", "practice"],
            ),
        ]),
        rules(&[
            ("HTTP basics", "REST API design"),
            ("PostgreSQL SELECT", "SQL joins"),
            ("Connect Node.js to PostgreSQL", "CRUD API"),
        ]),
        (60, 120),
    )
}

fn backend_python_postgres_beginner() -> RoadmapBlueprint {
    blueprint(
        "backend_python_postgres_beginner",
        GoalCategory::Backend,
        Some(TargetRole::Backend),
        phases(&[
            (
                "foundation",
                "Python and Web Foundation",
                "Prepare backend prerequisites.",
                &["python_web"],
            ),
            (
                "backend",
                "Python Backend Fundamentals",
                "Build HTTP APIs with Python.",
                &["python_backend"],
            ),
            (
                "database",
                "Database Foundation",
                "Use PostgreSQL safely.",
                &["postgres"],
            ),
            (
                "project",
                "Practice Project",
                "Build a Python backend project.",
                &["python_project"],
            ),
        ]),
        groups(&[
            (
                "python_web",
                "Python and Web",
                &[
                    "HTTP basics",
                    "Python functions",
                    "Python package management",
                ],
                &["official_reference", "primary_learning"],
            ),
            (
                "python_backend",
                "Python Backend",
                &[
                    "Python web framework basics",
                    "REST API design",
                    "Request validation",
                    "Error handling",
                ],
                &["primary_learning", "practice"],
            ),
            (
                "postgres",
                "PostgreSQL",
                &[
                    "PostgreSQL SELECT",
                    "SQL joins",
                    "PostgreSQL constraints",
                    "PostgreSQL indexes",
                ],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "python_project",
                "Python Backend Project",
                &[
                    "Connect Python to PostgreSQL",
                    "CRUD API",
                    "Build backend project",
                ],
                &["project", "practice"],
            ),
        ]),
        rules(&[
            ("Python functions", "Python web framework basics"),
            ("PostgreSQL SELECT", "Connect Python to PostgreSQL"),
        ]),
        (60, 110),
    )
}

fn fullstack_react_node_postgres() -> RoadmapBlueprint {
    blueprint(
        "fullstack_react_node_postgres",
        GoalCategory::Fullstack,
        Some(TargetRole::Fullstack),
        phases(&[
            (
                "frontend",
                "React Frontend",
                "Build frontend foundations.",
                &["react"],
            ),
            (
                "backend",
                "Node Backend",
                "Build backend foundations.",
                &["node_backend"],
            ),
            (
                "database",
                "PostgreSQL",
                "Use relational persistence.",
                &["postgres"],
            ),
            (
                "integration",
                "Fullstack Integration",
                "Connect frontend, backend, and database.",
                &["fullstack_project"],
            ),
        ]),
        groups(&[
            (
                "react",
                "React",
                &[
                    "React components",
                    "React state",
                    "React effects",
                    "Client-side forms",
                ],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "node_backend",
                "Node Backend",
                &[
                    "Node.js HTTP server",
                    "REST API design",
                    "API error handling",
                ],
                &["primary_learning", "practice"],
            ),
            (
                "postgres",
                "PostgreSQL",
                &["PostgreSQL SELECT", "SQL joins", "Database constraints"],
                &["official_reference", "primary_learning"],
            ),
            (
                "fullstack_project",
                "Fullstack Project",
                &[
                    "Connect React to API",
                    "CRUD fullstack flow",
                    "Fullstack deployment basics",
                ],
                &["project", "practice"],
            ),
        ]),
        rules(&[
            ("React state", "Connect React to API"),
            ("REST API design", "CRUD fullstack flow"),
        ]),
        (90, 160),
    )
}

fn devops_docker_kubernetes_beginner() -> RoadmapBlueprint {
    blueprint(
        "devops_docker_kubernetes_beginner",
        GoalCategory::Devops,
        Some(TargetRole::Devops),
        phases(&[
            (
                "containers",
                "Container Foundations",
                "Package applications with containers.",
                &["docker"],
            ),
            (
                "orchestration",
                "Kubernetes Foundations",
                "Run workloads on Kubernetes.",
                &["kubernetes"],
            ),
            (
                "delivery",
                "Delivery Practice",
                "Practice deployment workflows.",
                &["delivery"],
            ),
        ]),
        groups(&[
            (
                "docker",
                "Docker",
                &["Docker image", "Dockerfile basics", "Docker Compose"],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "kubernetes",
                "Kubernetes",
                &[
                    "Kubernetes pod",
                    "Kubernetes deployment",
                    "Kubernetes service",
                ],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "delivery",
                "Delivery",
                &[
                    "Basic CI/CD concept",
                    "Containerize app",
                    "Deploy sample service",
                ],
                &["practice", "project"],
            ),
        ]),
        rules(&[
            ("Docker image", "Docker Compose"),
            ("Docker Compose", "Kubernetes pod"),
        ]),
        (50, 100),
    )
}

fn rust_foundation() -> RoadmapBlueprint {
    foundation_blueprint(
        "rust_foundation",
        GoalCategory::ProgrammingLanguage,
        "Rust",
        &[
            "Rust ownership",
            "Rust borrowing",
            "Rust structs and enums",
            "Rust error handling",
        ],
    )
}

fn python_foundation() -> RoadmapBlueprint {
    foundation_blueprint(
        "python_foundation",
        GoalCategory::ProgrammingLanguage,
        "Python",
        &[
            "Python syntax",
            "Python functions",
            "Python collections",
            "Python modules",
            "Python error handling",
        ],
    )
}

fn web_foundation() -> RoadmapBlueprint {
    blueprint(
        "web_foundation",
        GoalCategory::WebFoundation,
        None,
        phases(&[
            (
                "foundation",
                "Web Platform",
                "Understand core web technologies.",
                &["web"],
            ),
            (
                "practice",
                "Web Practice",
                "Build small browser projects.",
                &["practice"],
            ),
        ]),
        groups(&[
            (
                "web",
                "Web Platform",
                &[
                    "HTML semantics",
                    "CSS layout",
                    "JavaScript basics",
                    "HTTP basics",
                ],
                &["official_reference", "primary_learning"],
            ),
            (
                "practice",
                "Web Practice",
                &["Build static page", "Add browser interaction"],
                &["practice", "project"],
            ),
        ]),
        rules(&[
            ("HTML semantics", "CSS layout"),
            ("JavaScript basics", "Add browser interaction"),
        ]),
        (25, 50),
    )
}

fn postgres_database_foundation() -> RoadmapBlueprint {
    blueprint(
        "postgres_database_foundation",
        GoalCategory::Database,
        None,
        phases(&[
            ("querying", "Querying", "Read data with SQL.", &["querying"]),
            (
                "modeling",
                "Modeling",
                "Design reliable relational data.",
                &["modeling"],
            ),
            (
                "performance",
                "Performance Basics",
                "Understand indexes and query plans.",
                &["performance"],
            ),
        ]),
        groups(&[
            (
                "querying",
                "SQL Querying",
                &["PostgreSQL SELECT", "SQL joins", "Aggregations"],
                &["official_reference", "primary_learning", "practice"],
            ),
            (
                "modeling",
                "Relational Modeling",
                &[
                    "PostgreSQL constraints",
                    "Normalization basics",
                    "Transactions",
                ],
                &["official_reference", "primary_learning"],
            ),
            (
                "performance",
                "Performance",
                &[
                    "PostgreSQL indexes",
                    "EXPLAIN basics",
                    "Query optimization basics",
                ],
                &["official_reference", "practice"],
            ),
        ]),
        rules(&[
            ("PostgreSQL SELECT", "SQL joins"),
            ("PostgreSQL constraints", "PostgreSQL indexes"),
        ]),
        (35, 80),
    )
}

fn custom_topic_linear() -> RoadmapBlueprint {
    blueprint(
        "custom_topic_linear",
        GoalCategory::CustomTopic,
        None,
        phases(&[
            (
                "orientation",
                "Orientation",
                "Establish vocabulary and prerequisite concepts.",
                &["orientation"],
            ),
            (
                "core",
                "Core Topic",
                "Study the core concepts with Resource coverage gating.",
                &["core"],
            ),
            (
                "application",
                "Application",
                "Apply the topic only when practice resources exist.",
                &["application"],
            ),
        ]),
        groups(&[
            (
                "orientation",
                "Orientation",
                &["Topic overview", "Prerequisites", "Terminology"],
                &["primary_learning", "official_reference"],
            ),
            (
                "core",
                "Core Concepts",
                &["Core concept 1", "Core concept 2", "Common pitfalls"],
                &["primary_learning", "reference"],
            ),
            (
                "application",
                "Application",
                &["Guided practice", "Small project or case study"],
                &["practice", "project"],
            ),
        ]),
        rules(&[
            ("Topic overview", "Core concept 1"),
            ("Core concept 2", "Guided practice"),
        ]),
        (20, 60),
    )
}

fn foundation_blueprint(
    id: &str,
    category: GoalCategory,
    label: &str,
    topics: &[&str],
) -> RoadmapBlueprint {
    blueprint(
        id,
        category,
        None,
        phases(&[
            (
                "foundation",
                &format!("{label} Foundation"),
                "Learn language fundamentals.",
                &["foundation"],
            ),
            (
                "practice",
                &format!("{label} Practice"),
                "Practice with small programs.",
                &["practice"],
            ),
        ]),
        groups(&[
            (
                "foundation",
                &format!("{label} Core"),
                topics,
                &["official_reference", "primary_learning"],
            ),
            (
                "practice",
                &format!("{label} Practice"),
                &["Exercises", "Small project", "Debugging practice"],
                &["practice", "project"],
            ),
        ]),
        rules(&[(topics[0], topics[1]), (topics[1], "Exercises")]),
        (30, 70),
    )
}

fn blueprint(
    id: &str,
    domain: GoalCategory,
    target_role: Option<TargetRole>,
    phases: Vec<BlueprintPhaseTemplate>,
    topic_groups: Vec<BlueprintTopicGroup>,
    prerequisite_rules: Vec<PrerequisiteRule>,
    estimated_hours: (u32, u32),
) -> RoadmapBlueprint {
    RoadmapBlueprint {
        blueprint_id: id.to_string(),
        domain,
        target_role,
        level: CurrentLevel::Beginner,
        phases,
        topic_groups,
        prerequisite_rules,
        default_required_resource_types: vec![
            "official_reference".to_string(),
            "primary_learning".to_string(),
            "practice".to_string(),
        ],
        estimated_hours_range: EstimatedHoursRange {
            min: estimated_hours.0,
            max: estimated_hours.1,
        },
    }
}

fn phases(items: &[(&str, &str, &str, &[&str])]) -> Vec<BlueprintPhaseTemplate> {
    items
        .iter()
        .map(|(id, title, purpose, group_ids)| BlueprintPhaseTemplate {
            phase_id: (*id).to_string(),
            title: (*title).to_string(),
            purpose: (*purpose).to_string(),
            topic_group_ids: group_ids.iter().map(|item| (*item).to_string()).collect(),
        })
        .collect()
}

fn groups(items: &[(&str, &str, &[&str], &[&str])]) -> Vec<BlueprintTopicGroup> {
    items
        .iter()
        .map(|(id, title, topics, required_types)| BlueprintTopicGroup {
            group_id: (*id).to_string(),
            title: (*title).to_string(),
            topics: topics.iter().map(|item| (*item).to_string()).collect(),
            required_resource_types: required_types
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
        })
        .collect()
}

fn rules(items: &[(&str, &str)]) -> Vec<PrerequisiteRule> {
    items
        .iter()
        .map(|(from, to)| PrerequisiteRule {
            from: (*from).to_string(),
            to: (*to).to_string(),
            reason: format!("{from} should be learned before {to}."),
        })
        .collect()
}

fn has_stack(goal: &GoalProfile, value: &str) -> bool {
    goal.stack
        .iter()
        .any(|item| item.eq_ignore_ascii_case(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CurrentLevel, GoalProfile};

    #[test]
    fn registry_contains_required_blueprints() {
        let ids = supported_blueprint_ids();

        assert_eq!(ids.len(), 10);
        assert!(ids.contains(&"backend_node_postgres_beginner"));
        assert!(ids.contains(&"custom_topic_linear"));
    }

    #[test]
    fn selects_backend_node_postgres_blueprint() {
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

        assert_eq!(
            selection.blueprint.blueprint_id,
            "backend_node_postgres_beginner"
        );
        assert!(selection.warnings.is_empty());
    }

    #[test]
    fn falls_back_for_custom_topic() {
        let selection = select_blueprint(&GoalProfile {
            category: GoalCategory::CustomTopic,
            domain: "custom_topic".to_string(),
            stack: vec![],
            target_role: None,
            level: CurrentLevel::Unknown,
            desired_outcome: None,
            normalized_goal: "learn niche topic".to_string(),
            warnings: vec![],
        });

        assert_eq!(selection.blueprint.blueprint_id, "custom_topic_linear");
        assert!(!selection.warnings.is_empty());
    }
}
