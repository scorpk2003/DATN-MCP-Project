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
        LessonFinalizeParam, LessonGradeAnswerParam, LessonValidateDraftParam,
    },
    error::{error_envelope, success_envelope},
    server::config::ServerConfig,
    services::{
        finalizer, grading, lesson_generator, lesson_validator, node_analyzer, progress_policy,
        resource_packer,
    },
};

#[derive(Debug, Clone)]
pub struct LessonServer {
    pub config: ServerConfig,
    #[allow(dead_code)]
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl LessonServer {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
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

    fn json_success(value: Value) -> CallToolResult {
        match serde_json::to_string(&value) {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!(
                "Serialize lesson result failed: {e}"
            ))]),
        }
    }

    #[tool(description = "Return Lesson MCP supported tools, boundaries, and integration rules.")]
    pub async fn get_lesson_contract(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET LESSON CONTRACT]");
        Ok(Self::json_success(success_envelope(
            contract::lesson_contract(),
        )))
    }

    #[tool(
        description = "Return Lesson MCP integration contract. Alias of get_lesson_contract for checklist compatibility."
    )]
    pub async fn get_lesson_integration_contract(&self) -> Result<CallToolResult, ErrorData> {
        self.get_lesson_contract().await
    }

    #[tool(description = "Return Lesson MCP process health.")]
    pub async fn lesson_health(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON HEALTH]");
        Ok(Self::json_success(success_envelope(json!({
            "service": "lesson_mcp",
            "status": "ok",
            "version": "0.1.0",
            "endpoint": format!("{}/mcp", self.config.url),
        }))))
    }

    #[tool(description = "Return Lesson MCP readiness for Orchestrator-managed v0.1 flow.")]
    pub async fn lesson_readiness(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON READINESS]");
        let ready = !self.config.resource_mcp_url.trim().is_empty()
            && !self.config.database_mcp_url.trim().is_empty();

        let data = json!({
            "service": "lesson_mcp",
            "ready": ready,
            "mode": "orchestrator_managed_v0_1",
            "checks": {
                "resourceMcpUrl": self.config.resource_mcp_url,
                "databaseMcpUrl": self.config.database_mcp_url,
                "internalTokenConfigured": self.config.internal_token_configured,
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
                "lesson_complete_session"
            ],
            "implementedServices": [
                "node_analyzer",
                "resource_packer",
                "lesson_generator",
                "lesson_validator",
                "finalizer",
                "grading",
                "progress_policy"
            ],
            "nextBuildStep": "Add MCP/client-level integration tests and fixtures.",
        });

        if ready {
            Ok(Self::json_success(success_envelope(data)))
        } else {
            Ok(Self::json_success(error_envelope(
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
        info!("\tCALL TOOL: [LESSON ANALYZE NODE]");
        if let Some(error) =
            required_context_error(&param.user_id, &param.roadmap_id, &param.roadmap_node_id)
        {
            return Ok(Self::json_success(error));
        }

        let requirement = node_analyzer::analyze_node(&param);

        Ok(Self::json_success(success_envelope(json!({
            "status": "ok",
            "implementationStatus": "rule_based_v1",
            "lessonRequirement": requirement,
        }))))
    }

    #[tool(
        description = "Create an evidence-based lesson draft from lesson requirements and Resource MCP candidates."
    )]
    pub async fn lesson_create_draft(
        &self,
        Parameters(param): Parameters<LessonCreateDraftParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON CREATE DRAFT]");
        if let Some(error) =
            required_context_error(&param.user_id, &param.roadmap_id, &param.roadmap_node_id)
        {
            return Ok(Self::json_success(error));
        }

        let packed_evidence =
            resource_packer::pack_resources(&param.lesson_requirement, &param.resources);

        if !resource_packer::has_sufficient_evidence(&packed_evidence) {
            return Ok(Self::json_success(success_envelope(json!({
                "status": "insufficient_resources",
                "lessonDraft": Value::Null,
                "issues": [{
                    "type": "insufficient_evidence",
                    "message": "Resource candidates do not provide enough high-quality chunk evidence for lesson generation.",
                    "severity": "high"
                }],
                "requiredResourceQuery": param.lesson_requirement.resource_queries,
                "packedEvidence": packed_evidence,
                "implementationStatus": "resource_packing_v1",
            }))));
        }

        let lesson_draft =
            lesson_generator::generate_draft(&param.lesson_requirement, &packed_evidence);

        Ok(Self::json_success(success_envelope(json!({
            "status": "ready",
            "lessonDraft": lesson_draft,
            "issues": [],
            "packedEvidence": packed_evidence,
            "implementationStatus": "lesson_generator_v1",
        }))))
    }

    #[tool(description = "Validate a lesson draft against quality and evidence policies.")]
    pub async fn lesson_validate_draft(
        &self,
        Parameters(param): Parameters<LessonValidateDraftParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON VALIDATE DRAFT]");
        let validation =
            lesson_validator::validate_draft(&param.lesson_draft, param.validation_policy);

        Ok(Self::json_success(success_envelope(json!({
            "status": if validation.passed { "passed" } else { "failed" },
            "qualityScore": validation.quality_score,
            "checks": validation.checks,
            "fixSuggestions": validation.fix_suggestions,
            "implementationStatus": "lesson_validator_v1",
        }))))
    }

    #[tool(description = "Finalize a lesson draft into a database-ready persistence payload.")]
    pub async fn lesson_finalize(
        &self,
        Parameters(param): Parameters<LessonFinalizeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON FINALIZE]");
        let status = param
            .save_policy
            .as_ref()
            .and_then(|policy| policy.status.clone())
            .unwrap_or_else(|| "ready".to_string());
        let lesson_payload = finalizer::build_lesson_payload(param.lesson_draft, status);

        Ok(Self::json_success(success_envelope(json!({
            "status": "ok",
            "lessonPayload": lesson_payload,
            "implementationStatus": "finalizer_v1",
        }))))
    }

    #[tool(description = "Grade a learner answer against an activity rubric.")]
    pub async fn lesson_grade_answer(
        &self,
        Parameters(param): Parameters<LessonGradeAnswerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON GRADE ANSWER]");
        if param.answer.trim().is_empty() {
            return Ok(Self::json_success(error_envelope(
                "LESSON_EMPTY_ANSWER",
                "answer is required.",
                json!({ "activity": param.activity }),
                false,
            )));
        }

        let result = grading::grade_answer(&param.answer, &param.rubric);

        Ok(Self::json_success(success_envelope(json!({
            "status": result.status,
            "score": result.score,
            "passed": result.passed,
            "feedback": result.feedback,
            "mistakes": result.mistakes,
            "improvementSuggestions": result.improvement_suggestions,
            "nextRecommendation": result.next_recommendation,
            "implementationStatus": "grading_v1",
        }))))
    }

    #[tool(description = "Complete a lesson session and produce a progress update payload.")]
    pub async fn lesson_complete_session(
        &self,
        Parameters(param): Parameters<LessonCompleteSessionParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LESSON COMPLETE SESSION]");
        let result = progress_policy::complete_session(param);

        Ok(Self::json_success(success_envelope(json!({
            "status": result.status,
            "masteryScore": result.mastery_score,
            "progressPayload": result.progress_payload,
            "nextAction": result.next_action,
            "implementationStatus": "progress_policy_v1",
        }))))
    }
}

impl Drop for LessonServer {
    fn drop(&mut self) {
        info!("\tShutting down Lesson MCP Server");
    }
}

fn required_context_error(user_id: &str, roadmap_id: &str, roadmap_node_id: &str) -> Option<Value> {
    if user_id.trim().is_empty()
        || roadmap_id.trim().is_empty()
        || roadmap_node_id.trim().is_empty()
    {
        return Some(error_envelope(
            "LESSON_INVALID_CONTEXT",
            "userId, roadmapId, and roadmapNodeId are required.",
            json!({
                "userIdEmpty": user_id.trim().is_empty(),
                "roadmapIdEmpty": roadmap_id.trim().is_empty(),
                "roadmapNodeIdEmpty": roadmap_node_id.trim().is_empty(),
            }),
            false,
        ));
    }

    None
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
