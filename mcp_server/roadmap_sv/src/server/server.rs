use anyhow::Result;
use axum::Router;
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    clients::resource_client::{ResourceClientError, ResourceContractClient},
    contract,
    domain::{
        BlueprintPhaseTemplate, BlueprintTopicGroup, BoundTopicPlan, CoverageRole, CoverageStatus,
        CoverageSummary, CurrentLevel, DatabaseMcpToolCall, EstimatedHoursRange, GoalCategory,
        GoalProfile, PrerequisiteRule, RoadmapBlueprint, RoadmapGenerationRequest, RoadmapGraph,
        RoadmapNode, RoadmapNodeType, RoadmapRequestValidationOutput, RoadmapStatus, TimeBudget,
        TopicPlan, ValidationResult,
    },
    error::{error_envelope, success_envelope},
    server::config::ServerConfig,
    services::{
        blueprint_registry, coverage_binder, graph_builder, graph_validator,
        persistence_payload_builder, request_validator, topic_decomposer,
    },
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetRoadmapDetailParam {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "roadmapId")]
    pub roadmap_id: String,
    #[serde(rename = "includeNodes")]
    pub include_nodes: Option<bool>,
    #[serde(rename = "includeResourceRefs")]
    pub include_resource_refs: Option<bool>,
    #[serde(rename = "includeProgress")]
    pub include_progress: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateRoadmapParam {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "roadmapId")]
    pub roadmap_id: String,
    pub title: Option<String>,
    pub status: Option<RoadmapStatus>,
    #[serde(rename = "timeBudget")]
    pub time_budget: Option<TimeBudget>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RefreshRoadmapResourcesParam {
    #[serde(rename = "roadmapGraph")]
    pub roadmap_graph: RoadmapGraph,
    #[serde(rename = "goal")]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CandidateTopicInput {
    pub name: String,
    pub reason: Option<String>,
    pub prerequisites: Option<Vec<String>>,
    #[serde(rename = "requiredResourceTypes")]
    pub required_resource_types: Option<Vec<String>>,
    #[serde(rename = "estimatedHours")]
    pub estimated_hours: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PlanRoadmapFromTopicsParam {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub goal: String,
    #[serde(rename = "currentLevel")]
    pub current_level: Option<CurrentLevel>,
    #[serde(rename = "candidateTopics")]
    pub candidate_topics: Vec<CandidateTopicInput>,
    pub constraints: Option<serde_json::Value>,
    #[serde(rename = "resourceContext")]
    pub resource_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EstimateRoadmapScopeParam {
    pub goal: String,
    #[serde(rename = "candidateTopicCount")]
    pub candidate_topic_count: Option<u32>,
    #[serde(rename = "timeBudget")]
    pub time_budget: Option<TimeBudget>,
}

#[derive(Debug, Clone)]
pub struct RoadmapServer {
    pub config: ServerConfig,
    pub resource_client: Result<ResourceContractClient, Arc<ResourceClientError>>,
    #[allow(dead_code)]
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RoadmapServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let resource_client = ResourceContractClient::new(config.resource_service_url.clone())
            .map(|client| client.with_token(config.resource_service_token.clone()))
            .map_err(Arc::new);
        let tool_router = Self::tool_router();
        Self {
            config,
            resource_client,
            tool_router,
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Roadmap Server at: {addr}");

        let config =
            StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new().nest_service("/mcp", service);

        let listener = TcpListener::bind(&addr).await?;

        info!("\tRoadmap MCP Server Endpoint: http://{}/mcp", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn json_success(value: Value) -> CallToolResult {
        match serde_json::to_string(&value) {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!(
                "Serialize roadmap result failed: {e}"
            ))]),
        }
    }

    #[tool(
        description = "Return Roadmap MCP supported inputs, output schema, coverage behavior, and integration rules."
    )]
    pub async fn get_roadmap_contract(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET ROADMAP CONTRACT]");
        Ok(Self::json_success(success_envelope(
            contract::roadmap_contract(),
        )))
    }

    #[tool(
        description = "Return Roadmap MCP integration contract. Alias of get_roadmap_contract for checklist compatibility."
    )]
    pub async fn get_roadmap_integration_contract(&self) -> Result<CallToolResult, ErrorData> {
        self.get_roadmap_contract().await
    }

    #[tool(description = "Return supported roadmap blueprint ids and selection policy.")]
    pub async fn get_roadmap_blueprints(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET ROADMAP BLUEPRINTS]");
        Ok(Self::json_success(success_envelope(json!({
            "blueprintIds": blueprint_registry::supported_blueprint_ids(),
            "selectionPolicy": "deterministic from normalized goal, target role, stack, and level",
            "fallbackBlueprintId": "custom_topic_linear",
        }))))
    }

    #[tool(description = "Validate and normalize a roadmap generation request.")]
    pub async fn validate_roadmap_request(
        &self,
        Parameters(param): Parameters<RoadmapGenerationRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [VALIDATE ROADMAP REQUEST]");
        let result = request_validator::validate_roadmap_request(param);
        Ok(Self::json_success(success_envelope(json!(result))))
    }

    #[tool(description = "Generate a non-persisted, coverage-aware roadmap preview.")]
    pub async fn generate_roadmap_preview(
        &self,
        Parameters(param): Parameters<RoadmapGenerationRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GENERATE ROADMAP PREVIEW]");

        match self.build_generation_result(param, false).await {
            Ok(data) => Ok(Self::json_success(success_envelope(data))),
            Err(error) => Ok(Self::json_success(error)),
        }
    }

    #[tool(
        description = "Generate a database-ready roadmap draft from a learning goal. Alias-compatible with the evaluation checklist."
    )]
    pub async fn generate_roadmap_from_goal(
        &self,
        Parameters(param): Parameters<RoadmapGenerationRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GENERATE ROADMAP FROM GOAL]");

        match self.build_generation_result(param, false).await {
            Ok(data) => Ok(Self::json_success(success_envelope(data))),
            Err(error) => Ok(Self::json_success(error)),
        }
    }

    #[tool(description = "Plan a roadmap from Orchestrator-provided candidate topics.")]
    pub async fn plan_roadmap_from_topics(
        &self,
        Parameters(param): Parameters<PlanRoadmapFromTopicsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [PLAN ROADMAP FROM TOPICS]");

        match self.build_topic_plan_result(param).await {
            Ok(data) => Ok(Self::json_success(success_envelope(data))),
            Err(error) => Ok(Self::json_success(error)),
        }
    }

    #[tool(description = "Estimate roadmap scope before generation.")]
    pub async fn estimate_roadmap_scope(
        &self,
        Parameters(param): Parameters<EstimateRoadmapScopeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [ESTIMATE ROADMAP SCOPE]");
        if param.goal.trim().is_empty() {
            return Ok(Self::json_success(error_envelope(
                "INVALID_SCOPE_REQUEST",
                "goal is required.",
                json!({"field": "goal"}),
                false,
            )));
        }

        let topic_count = param.candidate_topic_count.unwrap_or(8).clamp(1, 50);
        let estimated_hours = topic_count * 4;
        let hours_per_week = param
            .time_budget
            .as_ref()
            .and_then(|budget| budget.hours_per_week)
            .unwrap_or(8)
            .max(1);
        let estimated_weeks = estimated_hours.div_ceil(hours_per_week);

        Ok(Self::json_success(success_envelope(json!({
            "goal": param.goal,
            "estimatedNodes": topic_count,
            "estimatedHours": estimated_hours,
            "estimatedWeeks": estimated_weeks,
            "confidence": 0.72,
            "assumptions": [
                "Scope estimate uses candidateTopicCount when supplied; otherwise it assumes a medium roadmap.",
                "Resource coverage can increase warnings but does not increase estimated scope."
            ],
        }))))
    }

    #[tool(
        description = "Create a validated roadmap plan and database-ready persistence payload for Orchestrator execution."
    )]
    pub async fn create_roadmap(
        &self,
        Parameters(param): Parameters<RoadmapGenerationRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE ROADMAP]");

        match self.build_generation_result(param, true).await {
            Ok(data) => Ok(Self::json_success(success_envelope(data))),
            Err(error) => Ok(Self::json_success(error)),
        }
    }

    #[tool(description = "Validate a generated roadmap graph without persisting it.")]
    pub async fn validate_roadmap(
        &self,
        Parameters(param): Parameters<RoadmapGraph>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [VALIDATE ROADMAP]");
        let validation = graph_validator::validate_roadmap_graph(&param);

        Ok(Self::json_success(success_envelope(json!({
            "valid": validation.valid,
            "validationResult": validation,
        }))))
    }

    #[tool(
        description = "Validate a roadmap draft. Alias of validate_roadmap for checklist compatibility."
    )]
    pub async fn validate_roadmap_draft(
        &self,
        Parameters(param): Parameters<RoadmapGraph>,
    ) -> Result<CallToolResult, ErrorData> {
        self.validate_roadmap(Parameters(param)).await
    }

    #[tool(
        description = "Return a Database MCP call descriptor for Orchestrator to fetch roadmap detail."
    )]
    pub async fn get_roadmap_detail(
        &self,
        Parameters(param): Parameters<GetRoadmapDetailParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET ROADMAP DETAIL]");

        if param.user_id.trim().is_empty() || param.roadmap_id.trim().is_empty() {
            return Ok(Self::json_success(error_envelope(
                "INVALID_ROADMAP_DETAIL_REQUEST",
                "userId and roadmapId are required.",
                json!({
                    "userIdEmpty": param.user_id.trim().is_empty(),
                    "roadmapIdEmpty": param.roadmap_id.trim().is_empty(),
                }),
                false,
            )));
        }

        let call = DatabaseMcpToolCall {
            tool_name: "get_roadmap_detail".to_string(),
            arguments: json!({
                "userId": param.user_id,
                "roadmapId": param.roadmap_id,
                "includeNodes": param.include_nodes.unwrap_or(true),
                "includeResourceRefs": param.include_resource_refs.unwrap_or(true),
                "includeProgress": param.include_progress.unwrap_or(false),
            }),
            depends_on: vec![],
            result_alias: Some("roadmap_detail".to_string()),
        };

        Ok(Self::json_success(success_envelope(json!({
            "notFetched": true,
            "dataOwner": "database_mcp",
            "executionOwner": "orchestrator_agent",
            "databaseMcpCall": call,
            "nextSuggestedAction": "execute_database_mcp_call",
        }))))
    }

    #[tool(
        description = "Return a Database MCP call descriptor for Orchestrator to update roadmap metadata."
    )]
    pub async fn update_roadmap(
        &self,
        Parameters(param): Parameters<UpdateRoadmapParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE ROADMAP]");

        if param.user_id.trim().is_empty() || param.roadmap_id.trim().is_empty() {
            return Ok(Self::json_success(error_envelope(
                "INVALID_ROADMAP_UPDATE_REQUEST",
                "userId and roadmapId are required.",
                json!({
                    "userIdEmpty": param.user_id.trim().is_empty(),
                    "roadmapIdEmpty": param.roadmap_id.trim().is_empty(),
                }),
                false,
            )));
        }
        if param.title.is_none() && param.status.is_none() && param.time_budget.is_none() {
            return Ok(Self::json_success(error_envelope(
                "INVALID_ROADMAP_UPDATE_REQUEST",
                "At least one supported update field is required.",
                json!({"supportedFields": ["title", "status", "timeBudget"]}),
                false,
            )));
        }

        let call = DatabaseMcpToolCall {
            tool_name: "update_roadmap".to_string(),
            arguments: json!({
                "userId": param.user_id,
                "roadmapId": param.roadmap_id,
                "title": param.title,
                "status": param.status,
                "timeBudget": param.time_budget,
            }),
            depends_on: vec![],
            result_alias: Some("updated_roadmap".to_string()),
        };

        Ok(Self::json_success(success_envelope(json!({
            "notUpdated": true,
            "dataOwner": "database_mcp",
            "executionOwner": "orchestrator_agent",
            "databaseMcpCall": call,
            "nextSuggestedAction": "execute_database_mcp_call",
        }))))
    }

    #[tool(description = "Refresh Resource coverage and bindings for an existing roadmap graph.")]
    pub async fn refresh_roadmap_resources(
        &self,
        Parameters(param): Parameters<RefreshRoadmapResourcesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [REFRESH ROADMAP RESOURCES]");

        let resource_client = match &self.resource_client {
            Ok(client) => client,
            Err(error) => return Ok(Self::json_success(resource_error_envelope(error))),
        };
        let old_coverage_summary = param.roadmap_graph.coverage_summary.clone();
        let topic_plans = param
            .roadmap_graph
            .nodes
            .iter()
            .map(topic_plan_from_node)
            .collect::<Vec<_>>();
        let bound_topics = coverage_binder::bind_topic_resources(
            resource_client,
            &topic_plans,
            param.goal.as_deref(),
        )
        .await;
        let mut refreshed_graph = param.roadmap_graph.clone();
        let mut changed_nodes = Vec::new();

        for bound in &bound_topics {
            if let Some(node) = refreshed_graph
                .nodes
                .iter_mut()
                .find(|node| node.node_id == bound.topic_plan.topic_id)
            {
                let changed = coverage_key(&node.coverage_status)
                    != coverage_key(&bound.coverage.coverage_status)
                    || node.resource_refs.len() != bound.resource_refs.len()
                    || node.missing_resource_types != bound.missing_resource_types;
                node.coverage_status = bound.coverage.coverage_status.clone();
                node.resource_refs = bound.resource_refs.clone();
                node.missing_resource_types = bound.missing_resource_types.clone();
                node.warnings = bound.warnings.clone();
                node.status = bound.status.clone();
                if changed {
                    changed_nodes.push(node.node_id.clone());
                }
            }
        }

        refreshed_graph.coverage_summary = refreshed_coverage_summary(&bound_topics);
        refreshed_graph.resource_summary = refreshed_resource_summary(&refreshed_graph.nodes);
        refreshed_graph.gap_warnings = refreshed_gap_warnings(&bound_topics);
        refreshed_graph.status = refreshed_status(&refreshed_graph.coverage_summary);
        let validation = graph_validator::validate_roadmap_graph(&refreshed_graph);
        refreshed_graph.validation_result = Some(validation.clone());
        let database_ready_payload =
            persistence_payload_builder::build_database_ready_payload(&refreshed_graph);

        Ok(Self::json_success(success_envelope(json!({
            "notPersisted": true,
            "changedNodes": changed_nodes,
            "oldCoverageSummary": old_coverage_summary,
            "newCoverageSummary": refreshed_graph.coverage_summary,
            "updatedResourceRefs": refreshed_graph.nodes.iter().map(|node| json!({
                "nodeId": node.node_id,
                "resourceRefs": node.resource_refs,
            })).collect::<Vec<_>>(),
            "roadmapPreview": refreshed_graph,
            "databaseReadyPayload": database_ready_payload,
            "validationResult": validation,
            "nextSuggestedAction": "execute_database_persistence_plan",
        }))))
    }

    async fn build_generation_result(
        &self,
        param: RoadmapGenerationRequest,
        require_user_id: bool,
    ) -> Result<Value, Value> {
        let validation = request_validator::validate_roadmap_request(param);
        if !validation.valid {
            return Ok(json!({
                "valid": false,
                "notPersisted": true,
                "validation": validation,
            }));
        }

        let Some(normalized_request) = validation.normalized_request.clone() else {
            return Err(error_envelope(
                "INVALID_ROADMAP_REQUEST",
                "Roadmap request validation did not produce a normalized request.",
                json!({"validation": validation}),
                false,
            ));
        };
        let Some(goal_profile) = validation.goal_profile.clone() else {
            return Err(error_envelope(
                "GOAL_NORMALIZATION_FAILED",
                "Roadmap request validation did not produce a goal profile.",
                json!({"validation": validation}),
                false,
            ));
        };
        if require_user_id
            && normalized_request
                .user_id
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(error_envelope(
                "INVALID_ROADMAP_REQUEST",
                "create_roadmap requires userId so Orchestrator can persist ownership through Database MCP.",
                json!({"field": "userId"}),
                false,
            ));
        };
        let resource_client = match &self.resource_client {
            Ok(client) => client,
            Err(error) => return Err(resource_error_envelope(error)),
        };

        let blueprint_selection = blueprint_registry::select_blueprint(&goal_profile);
        let topic_plan = topic_decomposer::decompose_topics(&blueprint_selection.blueprint, None);
        let bound_topics = coverage_binder::bind_topic_resources(
            resource_client,
            &topic_plan,
            Some(&goal_profile.normalized_goal),
        )
        .await;
        let mut roadmap_graph = graph_builder::build_roadmap_graph(
            normalized_request.user_id.clone(),
            &goal_profile,
            &blueprint_selection.blueprint,
            &bound_topics,
        );
        let graph_validation = graph_validator::validate_roadmap_graph(&roadmap_graph);
        roadmap_graph.validation_result = Some(graph_validation.clone());
        let database_ready_payload =
            persistence_payload_builder::build_database_ready_payload(&roadmap_graph);
        let coverage_summary = roadmap_graph.coverage_summary.clone();
        let resource_summary = roadmap_graph.resource_summary.clone();
        let gap_warnings = roadmap_graph.gap_warnings.clone();
        let warnings = collect_preview_warnings(
            &validation,
            &blueprint_selection.warnings,
            &gap_warnings,
            &graph_validation,
        );

        let next_suggested_action = if graph_validation.valid {
            "execute_database_persistence_plan"
        } else {
            "review_validation_errors"
        };

        Ok(json!({
            "valid": graph_validation.valid,
            "notPersisted": true,
            "mode": if require_user_id { "create_roadmap" } else { "preview" },
            "normalizedRequest": normalized_request,
            "goalProfile": goal_profile,
            "blueprintSelection": blueprint_selection,
            "topicPlan": topic_plan,
            "boundTopics": bound_topics,
            "roadmapPreview": roadmap_graph,
            "roadmapDraft": roadmap_graph,
            "phases": roadmap_graph.phases,
            "nodes": roadmap_graph.nodes,
            "edges": roadmap_graph.edges,
            "resourceRefs": roadmap_graph.nodes.iter().flat_map(|node| node.resource_refs.clone()).collect::<Vec<_>>(),
            "databaseReadyPayload": database_ready_payload,
            "persistencePayload": database_ready_payload,
            "coverageSummary": coverage_summary,
            "resourceSummary": resource_summary,
            "assumptions": assumptions_for_generation(&blueprint_selection.warnings, &gap_warnings),
            "confidence": confidence_for_validation(&graph_validation, &coverage_summary),
            "warnings": warnings,
            "nextSuggestedAction": next_suggested_action,
        }))
    }

    async fn build_topic_plan_result(
        &self,
        param: PlanRoadmapFromTopicsParam,
    ) -> Result<Value, Value> {
        if param.goal.trim().is_empty() {
            return Err(error_envelope(
                "INVALID_TOPIC_PLAN_REQUEST",
                "goal is required.",
                json!({"field": "goal"}),
                false,
            ));
        }
        if param.candidate_topics.is_empty() {
            return Err(error_envelope(
                "INVALID_TOPIC_PLAN_REQUEST",
                "candidateTopics must contain at least one topic.",
                json!({"field": "candidateTopics"}),
                false,
            ));
        }

        let resource_client = match &self.resource_client {
            Ok(client) => client,
            Err(error) => return Err(resource_error_envelope(error)),
        };
        let (topic_plan, assumptions) = topic_plan_from_candidates(&param);
        let blueprint = blueprint_from_topic_plan(&param, &topic_plan);
        let goal_profile = GoalProfile {
            category: GoalCategory::CustomTopic,
            domain: "candidate_topics".to_string(),
            stack: topic_plan
                .iter()
                .take(8)
                .map(|topic| topic.topic_name.clone())
                .collect(),
            target_role: None,
            level: param.current_level.clone().unwrap_or(CurrentLevel::Unknown),
            desired_outcome: Some(param.goal.clone()),
            normalized_goal: param.goal.trim().to_string(),
            warnings: vec![],
        };
        let bound_topics =
            coverage_binder::bind_topic_resources(resource_client, &topic_plan, Some(&param.goal))
                .await;
        let mut roadmap_graph = graph_builder::build_roadmap_graph(
            param.user_id.clone(),
            &goal_profile,
            &blueprint,
            &bound_topics,
        );
        let validation = graph_validator::validate_roadmap_graph(&roadmap_graph);
        roadmap_graph.validation_result = Some(validation.clone());
        let database_ready_payload =
            persistence_payload_builder::build_database_ready_payload(&roadmap_graph);
        let coverage_summary = roadmap_graph.coverage_summary.clone();
        let warnings = collect_preview_warnings(
            &RoadmapRequestValidationOutput {
                valid: true,
                normalized_request: None,
                goal_profile: Some(goal_profile.clone()),
                validation_errors: vec![],
                warnings: vec![],
            },
            &[],
            &roadmap_graph.gap_warnings,
            &validation,
        );

        Ok(json!({
            "valid": validation.valid,
            "notPersisted": true,
            "mode": "plan_roadmap_from_topics",
            "goalProfile": goal_profile,
            "topicPlan": topic_plan,
            "boundTopics": bound_topics,
            "roadmapPreview": roadmap_graph,
            "roadmapDraft": roadmap_graph,
            "phases": roadmap_graph.phases,
            "nodes": roadmap_graph.nodes,
            "edges": roadmap_graph.edges,
            "resourceRefs": roadmap_graph.nodes.iter().flat_map(|node| node.resource_refs.clone()).collect::<Vec<_>>(),
            "databaseReadyPayload": database_ready_payload,
            "persistencePayload": database_ready_payload,
            "coverageSummary": coverage_summary,
            "resourceSummary": roadmap_graph.resource_summary,
            "warnings": warnings,
            "assumptions": assumptions,
            "confidence": confidence_for_validation(&validation, &coverage_summary),
            "validationResult": validation,
            "nextSuggestedAction": if validation.valid { "execute_database_persistence_plan" } else { "review_validation_errors" },
        }))
    }

    #[tool(description = "Return Roadmap MCP process health.")]
    pub async fn get_health_check(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [HEALTH CHECK]");
        Ok(Self::json_success(success_envelope(json!({
            "service": "roadmap_mcp",
            "status": "ok",
            "version": contract::ROADMAP_CONTRACT_VERSION,
        }))))
    }

    #[tool(description = "Return Roadmap MCP readiness based on scaffold config presence.")]
    pub async fn get_readiness_check(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [READINESS CHECK]");

        let resource_configured = !self.config.resource_mcp_url.trim().is_empty();
        let resource_service_configured = !self.config.resource_service_url.trim().is_empty();
        let resource_client_ready = self.resource_client.is_ok();
        let ready = resource_configured && resource_service_configured && resource_client_ready;

        let data = json!({
            "service": "roadmap_mcp",
            "ready": ready,
            "checks": {
                "resourceMcpUrlConfigured": resource_configured,
                "resourceServiceUrlConfigured": resource_service_configured,
                "resourceClientReady": resource_client_ready,
                "internalTokenConfigured": self.config.internal_token_configured,
            },
            "downstreams": {
                "resourceMcpUrl": self.config.resource_mcp_url,
                "resourceServiceUrl": self.config.resource_service_url,
            }
        });

        if ready {
            Ok(Self::json_success(success_envelope(data)))
        } else {
            Ok(Self::json_success(error_envelope(
                "ROADMAP_MCP_NOT_READY",
                "Roadmap MCP required downstream configuration is incomplete.",
                data,
                true,
            )))
        }
    }
}

fn resource_error_envelope(error: &ResourceClientError) -> Value {
    let normalized = error.normalized();
    let details = normalized.get("error").cloned().unwrap_or(Value::Null);
    let code = details
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("RESOURCE_MCP_UNAVAILABLE")
        .to_string();
    let message = details
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Resource integration is unavailable.")
        .to_string();
    let retryable = details
        .get("retryable")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    error_envelope(&code, &message, details, retryable)
}

fn collect_preview_warnings(
    validation: &RoadmapRequestValidationOutput,
    blueprint_warnings: &[String],
    gap_warnings: &[String],
    graph_validation: &ValidationResult,
) -> Vec<String> {
    validation
        .warnings
        .iter()
        .map(|warning| warning.message.clone())
        .chain(blueprint_warnings.iter().cloned())
        .chain(gap_warnings.iter().cloned())
        .chain(
            graph_validation
                .warnings
                .iter()
                .map(|warning| warning.message.clone()),
        )
        .collect()
}

pub(crate) fn topic_plan_from_candidates(
    param: &PlanRoadmapFromTopicsParam,
) -> (Vec<TopicPlan>, Vec<String>) {
    let mut seen = std::collections::BTreeSet::new();
    let mut assumptions = vec![
        "Candidate topics are treated as the primary planning input.".to_string(),
        "Roadmap MCP may reorder only through explicit prerequisite edges.".to_string(),
    ];
    if param.resource_context.is_some() {
        assumptions.push(
            "resourceContext was supplied; current implementation still verifies coverage through the Resource contract client.".to_string(),
        );
    }

    let level = param.current_level.clone().unwrap_or(CurrentLevel::Unknown);
    let topics = param
        .candidate_topics
        .iter()
        .filter_map(|candidate| {
            let name = candidate.name.trim();
            if name.is_empty() {
                return None;
            }
            let key = normalize_candidate_key(name);
            if !seen.insert(key) {
                assumptions.push(format!("Duplicate candidate topic skipped: {name}."));
                return None;
            }

            let required_types = candidate
                .required_resource_types
                .clone()
                .filter(|types| !types.is_empty())
                .unwrap_or_else(|| vec!["primary_learning".to_string()]);
            let node_type = if required_types
                .iter()
                .any(|kind| kind.eq_ignore_ascii_case("project"))
            {
                RoadmapNodeType::Project
            } else if required_types
                .iter()
                .any(|kind| kind.eq_ignore_ascii_case("practice"))
            {
                RoadmapNodeType::Practice
            } else {
                RoadmapNodeType::Concept
            };

            Some(TopicPlan {
                topic_id: slug_id(name),
                topic_name: name.to_string(),
                aliases: vec![name.to_ascii_lowercase(), slug_id(name)],
                level: level.clone(),
                required_resource_types: required_types,
                node_type,
                estimated_hours_hint: Some(candidate.estimated_hours.unwrap_or(4).clamp(1, 40)),
                prerequisite_topics: candidate.prerequisites.clone().unwrap_or_default(),
                optional: false,
            })
        })
        .take(50)
        .collect::<Vec<_>>();

    if param.candidate_topics.len() > 50 {
        assumptions.push("Candidate topic list was clamped to 50 topics.".to_string());
    }

    (topics, assumptions)
}

pub(crate) fn blueprint_from_topic_plan(
    param: &PlanRoadmapFromTopicsParam,
    topic_plan: &[TopicPlan],
) -> RoadmapBlueprint {
    let groups = topic_plan
        .iter()
        .map(|topic| BlueprintTopicGroup {
            group_id: format!("group_{}", topic.topic_id),
            title: topic.topic_name.clone(),
            topics: vec![topic.topic_name.clone()],
            required_resource_types: topic.required_resource_types.clone(),
        })
        .collect::<Vec<_>>();
    let phase_size = 5usize;
    let phases = groups
        .chunks(phase_size)
        .enumerate()
        .map(|(index, chunk)| BlueprintPhaseTemplate {
            phase_id: format!("phase_{}", index + 1),
            title: match index {
                0 => "Foundations and prerequisites".to_string(),
                1 => "Core concepts".to_string(),
                2 => "Applied practice".to_string(),
                _ => format!("Extension {}", index + 1),
            },
            purpose: "Study candidate topics in prerequisite-aware order.".to_string(),
            topic_group_ids: chunk.iter().map(|group| group.group_id.clone()).collect(),
        })
        .collect::<Vec<_>>();
    let topic_names = topic_plan
        .iter()
        .map(|topic| topic.topic_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let prerequisite_rules = topic_plan
        .iter()
        .flat_map(|topic| {
            topic.prerequisite_topics.iter().filter_map(|prerequisite| {
                topic_names
                    .contains(prerequisite.as_str())
                    .then(|| PrerequisiteRule {
                        from: prerequisite.clone(),
                        to: topic.topic_name.clone(),
                        reason: "Candidate topic prerequisite supplied by Orchestrator."
                            .to_string(),
                    })
            })
        })
        .collect::<Vec<_>>();

    RoadmapBlueprint {
        blueprint_id: "candidate_topics_adaptive".to_string(),
        domain: GoalCategory::CustomTopic,
        target_role: None,
        level: param.current_level.clone().unwrap_or(CurrentLevel::Unknown),
        phases,
        topic_groups: groups,
        prerequisite_rules,
        default_required_resource_types: vec!["primary_learning".to_string()],
        estimated_hours_range: EstimatedHoursRange {
            min: topic_plan
                .iter()
                .map(|topic| topic.estimated_hours_hint.unwrap_or(1))
                .sum(),
            max: topic_plan
                .iter()
                .map(|topic| topic.estimated_hours_hint.unwrap_or(1) + 2)
                .sum(),
        },
    }
}

fn assumptions_for_generation(
    blueprint_warnings: &[String],
    gap_warnings: &[String],
) -> Vec<String> {
    let mut assumptions = vec![
        "Blueprint selection is deterministic from the normalized goal.".to_string(),
        "Resource references are metadata only; lesson content is produced by Lesson MCP."
            .to_string(),
    ];
    assumptions.extend(blueprint_warnings.iter().cloned());
    if !gap_warnings.is_empty() {
        assumptions
            .push("Some topics need Resource backfill before lesson generation.".to_string());
    }
    assumptions
}

fn confidence_for_validation(validation: &ValidationResult, coverage: &CoverageSummary) -> f64 {
    let validation_score =
        validation
            .quality_score
            .unwrap_or(if validation.valid { 1.0 } else { 0.0 });
    let coverage_score = if coverage.total_topics == 0 {
        0.0
    } else {
        let weighted = coverage.coverage_good as f64
            + coverage.coverage_partial as f64 * 0.55
            + coverage.coverage_poor as f64 * 0.15;
        weighted / coverage.total_topics as f64
    };

    ((validation_score + coverage_score) / 2.0 * 100.0).round() / 100.0
}

fn normalize_candidate_key(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn slug_id(value: &str) -> String {
    let mut output = String::new();
    let mut last_dash = false;

    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            last_dash = false;
        } else if !last_dash {
            output.push('-');
            last_dash = true;
        }
    }

    output.trim_matches('-').to_string()
}

fn topic_plan_from_node(node: &RoadmapNode) -> TopicPlan {
    TopicPlan {
        topic_id: node.node_id.clone(),
        topic_name: node.topic.clone(),
        aliases: node.aliases.clone(),
        level: node.level.clone(),
        required_resource_types: required_types_from_node(node),
        node_type: node.node_type.clone(),
        estimated_hours_hint: Some(node.estimated_hours),
        prerequisite_topics: node.prerequisites.clone(),
        optional: false,
    }
}

fn required_types_from_node(node: &RoadmapNode) -> Vec<String> {
    if !node.missing_resource_types.is_empty() {
        return node.missing_resource_types.clone();
    }

    let mut types = node
        .resource_refs
        .iter()
        .map(|resource| match resource.coverage_role {
            CoverageRole::Primary => "primary_learning",
            CoverageRole::Reference => "official_reference",
            CoverageRole::Practice => "practice",
            CoverageRole::Optional => "primary_learning",
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    if types.is_empty() {
        types.push("primary_learning".to_string());
    }
    types.sort();
    types.dedup();
    types
}

fn refreshed_coverage_summary(bound_topics: &[BoundTopicPlan]) -> CoverageSummary {
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

fn refreshed_resource_summary(nodes: &[RoadmapNode]) -> Value {
    let refs = nodes
        .iter()
        .flat_map(|node| node.resource_refs.iter())
        .collect::<Vec<_>>();
    json!({
        "totalResourceRefs": refs.len(),
        "officialResourceRefs": refs.iter().filter(|resource| resource.is_official).count(),
        "primaryResourceRefs": refs.iter().filter(|resource| matches!(resource.coverage_role, CoverageRole::Primary)).count(),
        "practiceResourceRefs": refs.iter().filter(|resource| matches!(resource.coverage_role, CoverageRole::Practice)).count(),
    })
}

fn refreshed_gap_warnings(bound_topics: &[BoundTopicPlan]) -> Vec<String> {
    bound_topics
        .iter()
        .filter(|bound| matches!(bound.coverage.coverage_status, CoverageStatus::Poor))
        .map(|bound| {
            format!(
                "{} still needs resource backfill: missing {}.",
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

fn refreshed_status(summary: &CoverageSummary) -> RoadmapStatus {
    if summary.coverage_poor > 0 {
        RoadmapStatus::NeedsResourceBackfill
    } else if summary.coverage_partial > 0 || !summary.ready_for_lesson_generation {
        RoadmapStatus::Incomplete
    } else {
        RoadmapStatus::Draft
    }
}

fn coverage_key(status: &CoverageStatus) -> u8 {
    match status {
        CoverageStatus::Good => 1,
        CoverageStatus::Partial => 2,
        CoverageStatus::Poor => 3,
    }
}

impl Drop for RoadmapServer {
    fn drop(&mut self) {
        info!("\tShutting down Roadmap Server");
    }
}

#[tool_handler]
impl ServerHandler for RoadmapServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let instructions = [
            "Use this Roadmap MCP server to validate requests, generate coverage-aware roadmap previews, and produce database-ready persistence payloads.",
            "Orchestrator Agent owns Database MCP execution; Roadmap MCP never persists directly.",
            "Do not ask this server to crawl websites, mutate Resource DB state, or generate full lesson content.",
        ]
        .join(" ");

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();

        let implementation =
            Implementation::new("Roadmap MCP Server".to_string(), "0.1.0".to_string())
                .with_description(
                    "Coverage-aware roadmap planning MCP scaffold for Orchestrator integration."
                        .to_string(),
                )
                .with_title("Roadmap MCP Server".to_string())
                .with_website_url(self.config.url.clone());

        ServerInfo::new(capabilities)
            .with_instructions(instructions)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(implementation)
    }
}
