use schemars::schema_for;
use serde_json::{Value, json};

use crate::{
    domain::{
        BlueprintSelection, BoundTopicPlan, CoverageCheckResult, CoverageSummary,
        DatabaseReadyRoadmapPayload, GoalProfile, LearnerContext, ResourceBinding,
        RoadmapBlueprint, RoadmapEdge, RoadmapError, RoadmapGenerationRequest, RoadmapGraph,
        RoadmapNode, RoadmapPhase, RoadmapRequestValidationOutput, TopicPlan, ValidationResult,
        required_database_capabilities,
    },
    server::server::{CandidateTopicInput, EstimateRoadmapScopeParam, PlanRoadmapFromTopicsParam},
    services::blueprint_registry::supported_blueprint_ids,
};

pub const ROADMAP_CONTRACT_VERSION: &str = "0.1.0";

pub fn roadmap_contract() -> Value {
    json!({
        "version": ROADMAP_CONTRACT_VERSION,
        "status": "ready_for_orchestrator_integration",
        "supported_goal_types": [
            "frontend",
            "backend",
            "fullstack",
            "devops",
            "database",
            "programming_language",
            "web_foundation",
            "system_design",
            "general_software",
            "custom_topic"
        ],
        "supported_levels": [
            "beginner",
            "intermediate",
            "advanced",
            "unknown"
        ],
        "supported_node_types": [
            "foundation",
            "concept",
            "skill",
            "practice",
            "project",
            "checkpoint",
            "review"
        ],
        "supported_blueprints": supported_blueprint_ids(),
        "blueprint_policy": {
            "selection": "deterministic from GoalProfile category, targetRole, stack, and level",
            "fallback": "custom_topic_linear",
            "fallback_requires_coverage_gating": true,
            "llm_invented_blueprints_allowed": false
        },
        "topic_decomposition_policy": {
            "input": "RoadmapBlueprint + optional LearnerContext",
            "output": "TopicPlan[]",
            "rules": [
                "Expand blueprint topic groups in phase order.",
                "Preserve prerequisite topics for graph coherence.",
                "Attach required resource types from the source topic group.",
                "Mark known or completed leaf topics optional for intermediate and advanced learners.",
                "Do not remove beginner foundation topics during decomposition."
            ]
        },
        "coverage_policy": {
            "good": "Create a ready node and attach recommended ResourceRef values.",
            "partial": "Create a partial node, attach available ResourceRef values, and expose missing resource warnings.",
            "poor": "Create a blocked or placeholder node, report the gap, and mark the roadmap incomplete or needing resource backfill."
        },
        "resource_binding_policy": {
            "stores_raw_content": false,
            "include_chunks_by_default": false,
            "selection_order": [
                "prefer official resources",
                "prefer trustTier 1",
                "match resource kind to node purpose",
                "avoid outdated or needs_review resources unless warning is attached"
            ]
        },
        "coverage_binding_policy": {
            "good": "Bind recommended ResourceRef values and mark node ready when at least one resource is returned.",
            "partial": "Bind available ResourceRef values, preserve missing resource type warnings, and mark node partial.",
            "poor": "Do not attach unapproved candidates; report gap/request research through Resource Platform and mark node blocked.",
            "github_discovery": "GitHub discovery is handled by Resource Platform research flow, not directly by Roadmap MCP."
        },
        "graph_builder_policy": {
            "input": "GoalProfile + RoadmapBlueprint + BoundTopicPlan[]",
            "output": "RoadmapGraph",
            "rules": [
                "Create phases from blueprint phase templates.",
                "Create one roadmap node per bound topic.",
                "Build prerequisite edges from TopicPlan prerequisites.",
                "Set roadmap status to needs_resource_backfill when any topic has poor coverage.",
                "Set roadmap status to incomplete when coverage is partial or required ResourceRef values are missing."
            ]
        },
        "graph_validation_policy": {
            "checks": [
                "roadmap has at least one phase and one node",
                "phase ids and node ids are unique",
                "every node belongs to an existing phase",
                "edges reference existing nodes",
                "prerequisite graph has no cycles",
                "good coverage nodes have ResourceRef values",
                "partial or poor coverage nodes preserve warnings or missing resource types",
                "estimated hours are positive",
                "coverage summary counts are consistent"
            ]
        },
        "database_persistence_policy": {
            "roadmap_mcp_calls_database_mcp": false,
            "persistence_owner": "orchestrator_agent",
            "roadmap_mcp_output": "database_ready_roadmap_payload",
            "schema_version": "roadmap_draft_v1",
            "create_roadmap_behavior": "Roadmap MCP validates and returns a database-ready payload; it does not execute Database MCP calls.",
            "required_database_capabilities": required_database_capabilities(),
            "rules": [
                "Roadmap MCP must not execute Database MCP tools.",
                "Roadmap MCP must not store roadmap data directly.",
                "Roadmap MCP returns an orchestratorPersistencePlan with Database MCP tool names and arguments.",
                "Orchestrator Agent validates and executes Database MCP persistence in a separate step.",
                "The persistence payload remains notPersisted=true until Orchestrator completes Database MCP execution."
            ]
        },
        "tools": [
            "generate_roadmap_from_goal",
            "plan_roadmap_from_topics",
            "validate_roadmap_draft",
            "estimate_roadmap_scope",
            "get_roadmap_blueprints",
            "get_roadmap_integration_contract",
            "get_roadmap_contract",
            "validate_roadmap_request",
            "generate_roadmap_preview",
            "create_roadmap",
            "validate_roadmap",
            "get_roadmap_detail",
            "update_roadmap",
            "refresh_roadmap_resources",
            "get_health_check",
            "get_readiness_check"
        ],
        "planned_tools": [],
        "roadmap_output_schema": {
            "roadmapId": "string optional",
            "status": "draft | active | incomplete | needs_resource_backfill",
            "phases": "RoadmapPhase[]",
            "nodes": "RoadmapNode[]",
            "edges": "RoadmapEdge[]",
            "coverageSummary": "CoverageSummary",
            "resourceSummary": "object",
            "gapWarnings": "string[]",
            "validationResult": "ValidationResult",
            "assumptions": "string[]",
            "confidence": "number",
            "databaseReadyPayload": "DatabaseReadyRoadmapPayload"
        },
        "error_schema": {
            "success": false,
            "error": {
                "code": "string",
                "message": "string",
                "details": "object | null",
                "retryable": "boolean"
            },
            "meta": {
                "requestId": "string",
                "timestamp": "string"
            }
        },
        "integration_rules": [
            "Roadmap MCP must not crawl websites.",
            "Roadmap MCP must not write directly to Resource DB.",
            "Roadmap MCP must not write directly to application DB when Database MCP is the persistence layer.",
            "Roadmap MCP must not generate full lesson content.",
            "Roadmap MCP must preserve partial and poor coverage warnings."
        ],
        "schemas": contract_schemas()
    })
}

fn contract_schemas() -> Value {
    json!({
        "RoadmapGenerationRequest": schema_for!(RoadmapGenerationRequest),
        "CandidateTopicInput": schema_for!(CandidateTopicInput),
        "PlanRoadmapFromTopicsParam": schema_for!(PlanRoadmapFromTopicsParam),
        "EstimateRoadmapScopeParam": schema_for!(EstimateRoadmapScopeParam),
        "RoadmapRequestValidationOutput": schema_for!(RoadmapRequestValidationOutput),
        "LearnerContext": schema_for!(LearnerContext),
        "GoalProfile": schema_for!(GoalProfile),
        "RoadmapBlueprint": schema_for!(RoadmapBlueprint),
        "BlueprintSelection": schema_for!(BlueprintSelection),
        "TopicPlan": schema_for!(TopicPlan),
        "CoverageCheckResult": schema_for!(CoverageCheckResult),
        "BoundTopicPlan": schema_for!(BoundTopicPlan),
        "RoadmapGraph": schema_for!(RoadmapGraph),
        "RoadmapPhase": schema_for!(RoadmapPhase),
        "RoadmapNode": schema_for!(RoadmapNode),
        "RoadmapEdge": schema_for!(RoadmapEdge),
        "ResourceBinding": schema_for!(ResourceBinding),
        "DatabaseReadyRoadmapPayload": schema_for!(DatabaseReadyRoadmapPayload),
        "CoverageSummary": schema_for!(CoverageSummary),
        "ValidationResult": schema_for!(ValidationResult),
        "RoadmapError": schema_for!(RoadmapError)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_exposes_phase_two_schemas() {
        let contract = roadmap_contract();

        assert_eq!(contract["version"], ROADMAP_CONTRACT_VERSION);
        assert!(
            contract["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tool| tool == "get_roadmap_integration_contract")
        );
        assert!(contract["schemas"]["RoadmapGenerationRequest"].is_object());
        assert!(contract["schemas"]["PlanRoadmapFromTopicsParam"].is_object());
        assert!(contract["schemas"]["RoadmapGraph"].is_object());
        assert!(contract["schemas"]["ResourceBinding"].is_object());
        assert_eq!(
            contract["database_persistence_policy"]["schema_version"],
            "roadmap_draft_v1"
        );
    }
}
