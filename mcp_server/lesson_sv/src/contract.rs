use serde_json::{Value, json};

pub fn lesson_contract() -> Value {
    json!({
        "service": "lesson_mcp",
        "version": "0.1.0",
        "schemaVersion": "lesson_contract_v1",
        "role": {
            "owns": [
                "Analyze a roadmap node into lesson requirements.",
                "Create evidence-based lesson drafts from Resource MCP context.",
                "Validate lesson quality before persistence.",
                "Finalize database-ready lesson payloads for Orchestrator execution.",
                "Grade learner answers against rubrics.",
                "Produce progress update payloads after lesson completion."
            ],
            "doesNotOwn": [
                "User account management.",
                "Long-term roadmap generation.",
                "Web crawling, indexing, or source ranking.",
                "Direct application database persistence in v0.1.",
                "System-level learning strategy decisions."
            ]
        },
        "orchestration": {
            "mode": "orchestrator_managed_v0_1",
            "lessonMcpCallsDatabaseMcp": false,
            "lessonMcpCallsResourceMcp": false,
            "resourceEvidenceRequired": true,
            "insufficientResourceBehavior": "Return insufficient_resources instead of generating unsupported lesson content."
        },
        "implementedServices": [
            {
                "name": "node_analyzer",
                "status": "rule_based_v1",
                "responsibility": "Normalize objectives, detect prerequisite gaps, build resource queries, recommend exercise types, and estimate lesson duration."
            },
            {
                "name": "resource_packer",
                "status": "rule_based_v1",
                "responsibility": "Filter low-quality resources, dedupe candidates, select top chunks, group sources, and calculate objective coverage."
            },
            {
                "name": "lesson_generator",
                "status": "deterministic_v1",
                "responsibility": "Generate a schema-valid lesson draft with content blocks, selected resources, a practice exercise, quiz, and rubric from packed evidence."
            },
            {
                "name": "lesson_validator",
                "status": "policy_v1",
                "responsibility": "Validate objective/resource/exercise/quiz/content/source-reference quality before persistence."
            },
            {
                "name": "finalizer",
                "status": "database_payload_v1",
                "responsibility": "Convert a validated lesson draft into ordered Database MCP call descriptors without executing persistence."
            },
            {
                "name": "grading",
                "status": "rule_based_v1",
                "responsibility": "Score learner answers against rubric criteria, classify mistakes, and recommend the next action."
            },
            {
                "name": "progress_policy",
                "status": "policy_v1",
                "responsibility": "Compute mastery score, completion status, weak topics, progress payload, and next action for completed lesson sessions."
            }
        ],
        "tools": [
            "get_lesson_contract",
            "get_lesson_integration_contract",
            "lesson_health",
            "lesson_readiness",
            "lesson_analyze_node",
            "lesson_create_draft",
            "lesson_validate_draft",
            "lesson_finalize",
            "lesson_grade_answer",
            "lesson_complete_session"
        ],
        "plannedTools": [
            "lesson_start_session",
            "lesson_get_next_activity",
            "lesson_submit_answer",
            "lesson_generate_remediation"
        ],
        "resourceMcpDependencies": [
            "search_resources",
            "get_resource_detail",
            "get_resource_chunks",
            "recommend_resources_for_topic"
        ],
        "databaseMcpDependencyMode": "Orchestrator executes database calls produced by Lesson MCP payloads.",
        "guardrails": [
            "Do not hardcode topic-specific lesson templates.",
            "Do not generate lesson content without resource evidence.",
            "Do not persist directly while Database MCP is the persistence boundary.",
            "Return structured JSON envelopes for success and error cases."
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_lists_mvp_tools_and_boundaries() {
        let contract = lesson_contract();
        let tools = contract["tools"]
            .as_array()
            .expect("tools should be an array");

        for expected_tool in [
            "lesson_analyze_node",
            "lesson_create_draft",
            "lesson_validate_draft",
            "lesson_finalize",
            "lesson_grade_answer",
            "lesson_complete_session",
        ] {
            assert!(
                tools.iter().any(|tool| tool == expected_tool),
                "missing tool {expected_tool}"
            );
        }

        assert_eq!(
            contract["orchestration"]["lessonMcpCallsDatabaseMcp"],
            false
        );
        assert_eq!(
            contract["orchestration"]["lessonMcpCallsResourceMcp"],
            false
        );
        assert_eq!(contract["orchestration"]["resourceEvidenceRequired"], true);
    }

    #[test]
    fn contract_lists_core_services() {
        let contract = lesson_contract();
        let services = contract["implementedServices"]
            .as_array()
            .expect("implementedServices should be an array");

        for expected_service in [
            "node_analyzer",
            "resource_packer",
            "lesson_generator",
            "lesson_validator",
            "finalizer",
            "grading",
            "progress_policy",
        ] {
            assert!(
                services
                    .iter()
                    .any(|service| service["name"] == expected_service),
                "missing service {expected_service}"
            );
        }
    }
}
