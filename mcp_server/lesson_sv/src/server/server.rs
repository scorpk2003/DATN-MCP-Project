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
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    contract,
    domain::{
        LessonAnalyzeNodeParam, LessonCompleteSessionParam, LessonCreateDraftParam,
        LessonFinalizeParam, LessonGenerateRemediationParam, LessonGradeAnswerParam,
        LessonValidateDraftParam,
    },
    error::{error_envelope, success_envelope},
    server::config::ServerConfig,
    services::{
        access_policy, finalizer, grading, lesson_generator, lesson_validator, node_analyzer,
        observability::LessonTelemetry, progress_policy, remediation, request_guard,
        resource_packer,
    },
};

#[derive(Debug, Clone)]
pub struct LessonServer {
    pub config: ServerConfig,
    telemetry: Arc<LessonTelemetry>,
    #[allow(dead_code)]
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl LessonServer {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            telemetry: Arc::new(LessonTelemetry::default()),
            tool_router: Self::tool_router(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Lesson MCP Server at: {addr}");

        let config =
            StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new().nest_service("/mcp", service);
        let listener = TcpListener::bind(&addr).await?;

        info!("\tLesson MCP Server Endpoint: http://{}/mcp", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn json_result(value: Value) -> CallToolResult {
        match serde_json::to_string(&value) {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!(
                "Serialize lesson result failed: {e}"
            ))]),
        }
    }

    fn begin_tool(&self, tool_name: &'static str, request_id: Option<&str>) {
        self.telemetry.record_tool_call(tool_name, request_id);
    }

    fn json_success(&self, tool_name: &'static str, value: Value) -> CallToolResult {
        self.telemetry.record_tool_success(tool_name);
        Self::json_result(value)
    }

    fn json_tool_error(
        &self,
        tool_name: &'static str,
        request_id: &Option<String>,
        error: crate::error::LessonToolError,
    ) -> CallToolResult {
        self.telemetry
            .record_tool_error(tool_name, error.code.as_str(), error.retryable);
        Self::json_result(tool_error(request_id, error))
    }

    #[tool(description = "Return Lesson MCP supported tools, boundaries, and integration rules.")]
    pub async fn get_lesson_contract(&self) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "get_lesson_contract";
        self.begin_tool(TOOL, None);
        Ok(self.json_success(TOOL, success_envelope(contract::lesson_contract())))
    }

    #[tool(
        description = "Return Lesson MCP integration contract. Alias of get_lesson_contract for checklist compatibility."
    )]
    pub async fn get_lesson_integration_contract(&self) -> Result<CallToolResult, ErrorData> {
        self.get_lesson_contract().await
    }

    #[tool(description = "Return Lesson MCP process health.")]
    pub async fn lesson_health(&self) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_health";
        self.begin_tool(TOOL, None);
        self.telemetry.record_tool_success(TOOL);
        Ok(Self::json_result(success_envelope(json!({
                "service": "lesson_mcp",
                "status": "ok",
                "version": "0.1.0",
                "endpoint": format!("{}/mcp", self.config.url),
                "observability": self.telemetry.snapshot(),
        }))))
    }

    #[tool(description = "Return Lesson MCP readiness for Orchestrator-managed v0.1 flow.")]
    pub async fn lesson_readiness(&self) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_readiness";
        self.begin_tool(TOOL, None);
        let ready = !self.config.resource_mcp_url.trim().is_empty()
            && !self.config.database_mcp_url.trim().is_empty();
        if ready {
            self.telemetry.record_tool_success(TOOL);
        } else {
            self.telemetry
                .record_tool_error(TOOL, "LESSON_MCP_NOT_READY", false);
        }
        let telemetry = self.telemetry.snapshot();

        let data = json!({
            "service": "lesson_mcp",
            "ready": ready,
            "status": if ready { "ready" } else { "not_ready" },
            "mode": "orchestrator_managed_v0_1",
            "checks": {
                "resourceMcp": {
                    "configured": !self.config.resource_mcp_url.trim().is_empty(),
                    "url": self.config.resource_mcp_url,
                    "runtimeVerified": false,
                    "requiredFor": "lesson_create_draft evidence supplied by Orchestrator"
                },
                "databaseMcp": {
                    "configured": !self.config.database_mcp_url.trim().is_empty(),
                    "url": self.config.database_mcp_url,
                    "runtimeVerified": false,
                    "contractStatus": "missing_database_tools"
                },
                "internalToken": {
                    "configured": self.config.internal_token_configured,
                    "required": false
                },
                "telemetry": {
                    "configured": true,
                    "runtimeVerified": true
                }
            },
            "implementedTools": [
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
            "implementedServices": [
                "node_analyzer",
                "resource_packer",
                "lesson_generator",
                "lesson_validator",
                "finalizer",
                "grading",
                "remediation",
                "progress_policy",
                "request_guard",
                "access_policy"
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
            "requestGuards": {
                "answerTextMaxBytes": 8192,
                "codeSubmissionMaxBytes": 65536,
                "resourcePackMaxBytes": 262144,
                "singleChunkMaxBytes": 16384,
                "maxResourceChunksPerLesson": 20,
                "maxDraftBlocks": 20,
                "maxQuizItems": 20,
                "scoreRange": "0..1"
            },
            "permissionPolicy": {
                "mode": "hybrid_orchestrator_verified_context",
                "requiresVerifiedAuthContext": true,
                "sensitiveActionsRequireScope": true,
                "databaseRuntimeVerification": false
            },
            "databaseContract": {
                "status": "missing_database_tools",
                "mappingDocument": "mcp_server/lesson_sv/docs/database_mcp_contract_mapping.md",
                "requiredLessonTools": [
                    "create_lesson",
                    "create_lesson_block",
                    "link_lesson_resource",
                    "create_lesson_exercise",
                    "create_lesson_quiz"
                ],
                "currentDatabaseMcpHasLessonTools": false
            },
            "remediation": {
                "status": "ready",
                "requiresResourceRefs": true,
                "requiresFailedOrWeakGradingResult": true,
                "generatesRetryActivity": true
            },
            "observability": {
                "status": "ready",
                "structuredLogs": true,
                "inProcessCounters": true,
                "counters": telemetry
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
            "nextBuildStep": null,
        });

        if ready {
            Ok(Self::json_result(success_envelope(data)))
        } else {
            Ok(Self::json_result(error_envelope(
                "LESSON_MCP_NOT_READY",
                "Lesson MCP required downstream configuration is incomplete.",
                data,
                false,
            )))
        }
    }

    #[tool(description = "Analyze a roadmap node into lesson requirements and resource queries.")]
    pub async fn lesson_analyze_node(
        &self,
        Parameters(param): Parameters<LessonAnalyzeNodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_analyze_node";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = request_guard::require_context(
            &param.user_id,
            &param.roadmap_id,
            &param.roadmap_node_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            Some(&param.user_id),
            "roadmap:read",
            "roadmap_node",
            Some(&param.roadmap_node_id),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }

        let requirement = node_analyzer::analyze_node(&param);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": "ok",
                "implementationStatus": "rule_based_v1",
                "lessonRequirement": requirement,
            })),
        ))
    }

    #[tool(
        description = "Create an evidence-based lesson draft from lesson requirements and Resource MCP candidates."
    )]
    pub async fn lesson_create_draft(
        &self,
        Parameters(param): Parameters<LessonCreateDraftParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_create_draft";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = request_guard::require_context(
            &param.user_id,
            &param.roadmap_id,
            &param.roadmap_node_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            Some(&param.user_id),
            "lesson:write",
            "roadmap_node",
            Some(&param.roadmap_node_id),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_create_draft(&param) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }

        let packed_evidence =
            resource_packer::pack_resources(&param.lesson_requirement, &param.resources);

        if !resource_packer::has_sufficient_evidence(&packed_evidence) {
            return Ok(self.json_tool_error(
                TOOL,
                &param.request_id,
                crate::error::LessonToolError::new(
                    crate::error::LessonErrorCode::InsufficientResources,
                    "Resource candidates do not provide enough high-quality chunk evidence for lesson generation.",
                    json!({
                        "requiredResourceQuery": param.lesson_requirement.resource_queries,
                        "packedEvidence": packed_evidence,
                    }),
                ),
            ));
        }

        let lesson_draft =
            lesson_generator::generate_draft(&param.lesson_requirement, &packed_evidence);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": "ready",
                "lessonDraft": lesson_draft,
                "issues": [],
                "packedEvidence": packed_evidence,
                "implementationStatus": "lesson_generator_v1",
            })),
        ))
    }

    #[tool(description = "Validate a lesson draft against quality and evidence policies.")]
    pub async fn lesson_validate_draft(
        &self,
        Parameters(param): Parameters<LessonValidateDraftParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_validate_draft";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            None,
            "lesson:write",
            "lesson_draft",
            Some(&param.lesson_draft.topic),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_draft_size(&param.lesson_draft) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        let validation =
            lesson_validator::validate_draft(&param.lesson_draft, param.validation_policy);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": if validation.passed { "passed" } else { "failed" },
                "qualityScore": validation.quality_score,
                "checks": validation.checks,
                "fixSuggestions": validation.fix_suggestions,
                "implementationStatus": "lesson_validator_v1",
            })),
        ))
    }

    #[tool(description = "Finalize a lesson draft into a database-ready persistence payload.")]
    pub async fn lesson_finalize(
        &self,
        Parameters(param): Parameters<LessonFinalizeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_finalize";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            None,
            "lesson:write",
            "lesson_draft",
            Some(&param.lesson_draft.topic),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_draft_size(&param.lesson_draft) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        let status = param
            .save_policy
            .as_ref()
            .and_then(|policy| policy.status.clone())
            .unwrap_or_else(|| "ready".to_string());
        let lesson_payload = finalizer::build_lesson_payload(param.lesson_draft, status);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": "ok",
                "lessonPayload": lesson_payload,
                "implementationStatus": "finalizer_v1",
            })),
        ))
    }

    #[tool(description = "Grade a learner answer against an activity rubric.")]
    pub async fn lesson_grade_answer(
        &self,
        Parameters(param): Parameters<LessonGradeAnswerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_grade_answer";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = request_guard::require_lesson_session_context(
            &param.user_id,
            &param.lesson_id,
            &param.session_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            Some(&param.user_id),
            "lesson:evaluate",
            "lesson",
            Some(&param.lesson_id),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_answer(&param) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }

        let result = grading::grade_answer(&param.answer, &param.rubric);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": result.status,
                "score": result.score,
                "passed": result.passed,
                "feedback": result.feedback,
                "mistakes": result.mistakes,
                "improvementSuggestions": result.improvement_suggestions,
                "nextRecommendation": result.next_recommendation,
                "implementationStatus": "grading_v1",
            })),
        ))
    }

    #[tool(description = "Complete a lesson session and produce a progress update payload.")]
    pub async fn lesson_complete_session(
        &self,
        Parameters(param): Parameters<LessonCompleteSessionParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_complete_session";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = request_guard::require_lesson_session_context(
            &param.user_id,
            &param.lesson_id,
            &param.session_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            Some(&param.user_id),
            "lesson:progress",
            "session",
            Some(&param.session_id),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_session_scores(&param.session_summary) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        let result = progress_policy::complete_session(param);

        Ok(self.json_success(
            TOOL,
            success_envelope(json!({
                "status": result.status,
                "masteryScore": result.mastery_score,
                "progressPayload": result.progress_payload,
                "nextAction": result.next_action,
                "implementationStatus": "progress_policy_v1",
            })),
        ))
    }

    #[tool(description = "Generate grounded remediation after a weak or failed lesson answer.")]
    pub async fn lesson_generate_remediation(
        &self,
        Parameters(param): Parameters<LessonGenerateRemediationParam>,
    ) -> Result<CallToolResult, ErrorData> {
        const TOOL: &str = "lesson_generate_remediation";
        self.begin_tool(TOOL, param.request_id.as_deref());
        if let Err(error) = request_guard::require_context(
            &param.user_id,
            &param.roadmap_id,
            &param.roadmap_node_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::require_lesson_session_context(
            &param.user_id,
            &param.lesson_id,
            &param.session_id,
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = access_policy::require_verified_access(
            &param.auth_context,
            Some(&param.user_id),
            "lesson:evaluate",
            "lesson",
            Some(&param.lesson_id),
        ) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }
        if let Err(error) = request_guard::validate_remediation_request(&param) {
            return Ok(self.json_tool_error(TOOL, &param.request_id, error));
        }

        match remediation::generate_remediation(&param) {
            Ok(remediation) => Ok(self.json_success(
                TOOL,
                success_envelope(json!({
                    "status": "ok",
                    "remediation": remediation,
                    "implementationStatus": "remediation_v1",
                })),
            )),
            Err(error) => Ok(self.json_tool_error(TOOL, &param.request_id, error)),
        }
    }
}

impl Drop for LessonServer {
    fn drop(&mut self) {
        info!("\tShutting down Lesson MCP Server");
    }
}

fn tool_error(request_id: &Option<String>, error: crate::error::LessonToolError) -> Value {
    crate::error::tool_error_envelope(error, request_id.as_deref())
}

#[tool_handler]
impl ServerHandler for LessonServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let instructions = [
            "Use this server to create, validate, finalize, grade, and complete evidence-based lessons.",
            "Orchestrator owns Database MCP and Resource MCP execution in v0.1.",
            "Do not ask this server to crawl websites, store records directly, or generate unsupported lesson claims.",
        ]
        .join(" ");

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();
        let implementation =
            Implementation::new("Lesson MCP Server".to_string(), "0.1.0".to_string())
                .with_website_url(self.config.url.clone())
                .with_description(
                    "Evidence-based lesson generation MCP scaffold for Orchestrator integration."
                        .to_string(),
                )
                .with_title("Lesson MCP Server".to_string());

        ServerInfo::new(capabilities)
            .with_instructions(instructions)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(implementation)
    }
}
