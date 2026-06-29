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
                "Direct application database persistence; Orchestrator owns executing Database MCP call plans.",
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
                "name": "remediation",
                "status": "rule_based_v1",
                "responsibility": "Generate grounded remedial blocks, hint ladder, retry activity, and next action for weak or failed answers."
            },
            {
                "name": "progress_policy",
                "status": "policy_v1",
                "responsibility": "Compute mastery score, completion status, weak topics, progress payload, and next action for completed lesson sessions."
            },
            {
                "name": "request_guard",
                "status": "guard_v1",
                "responsibility": "Reject missing identifiers, oversized answers/resources, invalid draft sizes, and scores outside the 0..1 range."
            },
            {
                "name": "access_policy",
                "status": "permission_boundary_v1",
                "responsibility": "Reject user-scoped tool calls that lack verified auth context, required scope, or matching user ownership context."
            },
            {
                "name": "observability",
                "status": "telemetry_v1",
                "responsibility": "Track in-process tool call, success, error, per-tool, and error-code counters; expose snapshots through health and readiness."
            }
        ],
        "errorTaxonomy": [
            "INVALID_INPUT",
            "INSUFFICIENT_RESOURCES",
            "PERMISSION_DENIED",
            "RESOURCE_NOT_FOUND",
            "DATABASE_ERROR",
            "DEPENDENCY_UNAVAILABLE",
            "EVALUATION_FAILED",
            "GENERATION_FAILED"
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
            "lesson_generate_remediation",
            "lesson_complete_session"
        ],
        "plannedTools": [
            "lesson_start_session",
            "lesson_get_next_activity",
            "lesson_submit_answer"
        ],
        "resourceMcpDependencies": [
            "search_resources",
            "get_resource_detail",
            "get_resource_chunks",
            "recommend_resources_for_topic"
        ],
        "databaseMcpDependencyMode": "Orchestrator executes database calls produced by Lesson MCP payloads.",
        "databaseContract": {
            "status": "verified",
            "mappingDocument": "mcp_server/lesson_sv/docs/database_mcp_contract_mapping.md",
            "requiredLessonTools": [
                "create_lesson",
                "create_lesson_block",
                "link_lesson_resource",
                "create_lesson_exercise",
                "create_lesson_quiz"
            ],
            "currentDatabaseMcpHasLessonTools": true
        },
        "observability": {
            "status": "ready",
            "structuredLogs": true,
            "inProcessCounters": true,
            "exposedBy": [
                "lesson_health",
                "lesson_readiness"
            ],
            "counters": [
                "totalToolCalls",
                "totalToolSuccesses",
                "totalToolErrors",
                "toolCalls",
                "toolErrors",
                "errorCodes"
            ]
        },
        "hardeningStatus": {
            "phase": "v0.2",
            "completedPhases": [
                "error_taxonomy_request_guards",
                "permission_boundary",
                "database_contract_verification",
                "remediation_flow",
                "mcp_client_integration_tests",
                "observability_readiness_gate"
            ],
            "remainingPhases": []
        },
        "guardrails": [
            "Do not hardcode topic-specific lesson templates.",
            "Do not generate lesson content without resource evidence.",
            "Do not persist directly while Database MCP is the persistence boundary.",
            "Reject user-scoped requests without verified auth context.",
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
            "observability",
        ] {
            assert!(
                services
                    .iter()
                    .any(|service| service["name"] == expected_service),
                "missing service {expected_service}"
            );
        }
    }

    #[test]
    fn contract_declares_completed_hardening_and_observability() {
        let contract = lesson_contract();

        assert_eq!(contract["observability"]["status"], "ready");
        assert_eq!(contract["observability"]["inProcessCounters"], true);
        assert_eq!(
            contract["hardeningStatus"]["remainingPhases"]
                .as_array()
                .expect("remainingPhases should be an array")
                .len(),
            0
        );
    }
}
