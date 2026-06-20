use anyhow::Result;
use axum::Router;
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::server::config::ServerConfig;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateLearningRoadmapParam {
    #[schemars(description = "Programming topic or career target the learner wants to study.")]
    #[schemars(length(min = 1))]
    pub topic: String,
    #[schemars(description = "Learner level: beginner, intermediate, or advanced.")]
    pub level: Option<String>,
    #[schemars(description = "How many hours the learner can study each week.")]
    #[schemars(range(min = 1, max = 80))]
    pub weekly_hours: Option<u32>,
    #[schemars(description = "Total number of weeks for the generated roadmap.")]
    #[schemars(range(min = 2, max = 52))]
    pub duration_weeks: Option<u32>,
    #[schemars(description = "Specific outcomes the learner wants to achieve.")]
    pub goals: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct RoadmapServer {
    pub config: ServerConfig,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RoadmapServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let tool_router = Self::tool_router();
        Self {
            config,
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
        description = "Generate a structured self-learning programming roadmap with phases, weekly tasks, projects, and checkpoints."
    )]
    pub async fn generate_learning_roadmap(
        &self,
        Parameters(param): Parameters<GenerateLearningRoadmapParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GENERATE LEARNING ROADMAP]");
        let level = param.level.unwrap_or_else(|| "beginner".to_string());
        let weekly_hours = param.weekly_hours.unwrap_or(8);
        let duration_weeks = param.duration_weeks.unwrap_or(8);
        let goals = param.goals.unwrap_or_default();
        let phase_count = if duration_weeks <= 4 {
            2
        } else if duration_weeks <= 10 {
            4
        } else {
            6
        };
        let weeks_per_phase = (duration_weeks as f32 / phase_count as f32).ceil() as u32;

        let phases = (0..phase_count)
            .map(|idx| {
                let start_week = idx as u32 * weeks_per_phase + 1;
                let end_week = ((idx as u32 + 1) * weeks_per_phase).min(duration_weeks);
                let focus = match idx {
                    0 => "Foundation and setup",
                    1 => "Core syntax and problem solving",
                    2 => "Applied projects and debugging",
                    3 => "Real-world patterns and architecture",
                    4 => "Testing, deployment, and collaboration",
                    _ => "Portfolio polish and interview practice",
                };
                json!({
                    "order": idx + 1,
                    "title": focus,
                    "weeks": [start_week, end_week],
                    "outcomes": [
                        format!("Understand {} concepts at {} level", param.topic, level),
                        "Complete exercises with clear notes",
                        "Build one reviewable artifact"
                    ],
                    "weekly_tasks": [
                        format!("Study {} hours of focused material", weekly_hours / 2),
                        "Practice small coding problems",
                        "Write a short learning log and blockers",
                        "Ship or improve a project slice"
                    ],
                    "checkpoint": format!("Demo phase {} result and decide whether to review or advance", idx + 1)
                })
            })
            .collect::<Vec<_>>();

        let result = json!({
            "topic": param.topic,
            "level": level,
            "duration_weeks": duration_weeks,
            "weekly_hours": weekly_hours,
            "goals": goals,
            "strategy": "Learn in short cycles: concept, practice, build, review, then adjust the next phase.",
            "phases": phases,
            "capstone_project": {
                "title": "Portfolio learning project",
                "description": "Build a small but complete application that demonstrates the main skills from the roadmap.",
                "acceptance_criteria": [
                    "Readable source code",
                    "Documented setup steps",
                    "At least one automated or manual test checklist",
                    "Short retrospective with next steps"
                ]
            }
        });

        Ok(Self::json_success(result))
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
        let ins = format!("Generating and Planning Roadmap for topic, skill");

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();

        let info = Implementation::default()
            .with_description("Roadmap for topics, skill")
            .with_title("Roadmap Server")
            .with_website_url(self.config.url.clone());

        let sv_info = ServerInfo::new(capabilities)
            .with_instructions(ins)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(info);

        sv_info
    }
}
