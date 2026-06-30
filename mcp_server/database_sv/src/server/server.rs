use std::{env, sync::Arc};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
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
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    VALID_TABLES,
    provider::SchemaProvider,
    schemas::*,
    server::ServerConfig,
    tools::{
        AnalyticsTool, LessonTool, MilestoneTool, PhaseTool, ProgressTool, ProjectTool,
        ResourceTool, RoadmapTool, SearchTool, TaskTool, UserTool,
    },
};

#[derive(Debug, Deserialize)]
struct LatestRoadmapQuery {
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserDataQuery {
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateNoteRequest {
    #[serde(rename = "userId")]
    user_id: String,
    content: String,
    #[serde(rename = "taskId")]
    task_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct LatestRoadmapResponse {
    roadmap: Option<Value>,
}

#[derive(Debug, Serialize)]
struct NotesResponse {
    notes: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct ReviewResponse {
    #[serde(rename = "reviewItems")]
    review_items: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct SeedRoadmapRequest {
    #[serde(rename = "userId")]
    user_id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct SeedLearningDataRequest {
    #[serde(rename = "userId")]
    user_id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct ResetTestDataRequest {
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DbServer {
    pub config: ServerConfig,
    pub provider: Arc<Mutex<SchemaProvider>>,
    pub user_tool: UserTool,
    pub project_tool: ProjectTool,
    pub roadmap_tool: RoadmapTool,
    pub phase_tool: PhaseTool,
    pub milestone_tool: MilestoneTool,
    pub task_tool: TaskTool,
    pub progress_tool: ProgressTool,
    pub resource_tool: ResourceTool,
    pub search_tool: SearchTool,
    pub analytics_tool: AnalyticsTool,
    pub lesson_tool: LessonTool,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DbServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let provider = Arc::new(Mutex::new(SchemaProvider::default()));

        Self {
            config,
            user_tool: UserTool::new(provider.clone()),
            project_tool: ProjectTool::new(provider.clone()),
            roadmap_tool: RoadmapTool::new(provider.clone()),
            phase_tool: PhaseTool::new(provider.clone()),
            milestone_tool: MilestoneTool::new(provider.clone()),
            task_tool: TaskTool::new(provider.clone()),
            progress_tool: ProgressTool::new(provider.clone()),
            resource_tool: ResourceTool::new(provider.clone()),
            search_tool: SearchTool::new(provider.clone()),
            analytics_tool: AnalyticsTool::new(provider.clone()),
            lesson_tool: LessonTool::new(provider.clone()),
            provider,
            tool_router: Self::tool_router(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Database Server at: {addr}");

        let config = StreamableHttpServerConfig::default()
            .with_allowed_hosts(allowed_mcp_hosts())
            .with_cancellation_token(CancellationToken::new());
        let mcp_server = self.clone();
        let rest_server = self.clone();
        let service = StreamableHttpService::new(
            move || Ok(mcp_server.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new()
            .route("/roadmaps/latest", get(latest_roadmap_handler))
            .route("/notes", get(notes_handler).post(create_note_handler))
            .route("/review", get(review_handler))
            .route("/test/roadmaps/seed", post(seed_roadmap_handler))
            .route("/test/learning-data/seed", post(seed_learning_data_handler))
            .route("/test/reset", post(reset_test_data_handler))
            .with_state(rest_server)
            .nest_service("/mcp", service);

        let listener = TcpListener::bind(&addr).await?;

        info!("\tDatabase MCP Server Endpoint: http://{}/mcp", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn json_success(value: Value) -> CallToolResult {
        match serde_json::to_string(&value) {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!(
                "Serialize tool result failed: {e}"
            ))]),
        }
    }

    #[tool(description = "Create a user by firebase id, display name, and email")]
    pub async fn create_user(
        &self,
        Parameters(param): Parameters<CreateUserParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE USER]");
        match self.user_tool.create_user(param).await {
            Ok(value) => {
                info!("\tTOOL: [CREATE USER] CALL COMPLETED");
                Ok(Self::json_success(value))
            }
            Err(e) => {
                error!("\tCreate user failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create user failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create or update a user by Firebase uid")]
    pub async fn upsert_user(
        &self,
        Parameters(param): Parameters<UpsertUserParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPSERT USER]");
        match self.user_tool.upsert_user(param).await {
            Ok(value) => {
                info!("\tTOOL: [UPSERT USER] CALL COMPLETED");
                Ok(Self::json_success(value))
            }
            Err(e) => {
                error!("\tUpsert user failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Upsert user failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get user by Firebase uid")]
    pub async fn get_user_by_id(
        &self,
        Parameters(param): Parameters<GetUserByIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET USER BY ID]");
        match self.user_tool.get_user_by_id(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet user by id failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get user by id failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create project for a user")]
    pub async fn create_project(
        &self,
        Parameters(param): Parameters<CreateProjectParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE PROJECT]");
        match self.project_tool.create_project(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate project failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create project failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get project by UUID")]
    pub async fn get_project(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET PROJECT]");
        match self.project_tool.get_project(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet project failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get project failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Update project fields by UUID")]
    pub async fn update_project(
        &self,
        Parameters(param): Parameters<UpdateProjectParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE PROJECT]");
        match self.project_tool.update_project(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tUpdate project failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Update project failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete project by UUID")]
    pub async fn delete_project(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE PROJECT]");
        match self.project_tool.delete_project(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete project failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete project failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "List projects owned by a user")]
    pub async fn list_projects(
        &self,
        Parameters(param): Parameters<ListProjectsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LIST PROJECTS]");
        match self.project_tool.list_projects(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tList projects failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "List projects failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create roadmap for a project")]
    pub async fn create_roadmap(
        &self,
        Parameters(param): Parameters<CreateRoadmapParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE ROADMAP]");
        match self.roadmap_tool.create_roadmap(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate roadmap failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create roadmap failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get roadmap by UUID")]
    pub async fn get_roadmap(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET ROADMAP]");
        match self.roadmap_tool.get_roadmap(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet roadmap failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get roadmap failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete roadmap by UUID")]
    pub async fn delete_roadmap(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE ROADMAP]");
        match self.roadmap_tool.delete_roadmap(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete roadmap failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete roadmap failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "List roadmaps for a project")]
    pub async fn list_project_roadmap(
        &self,
        Parameters(param): Parameters<ListProjectRoadmapParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LIST PROJECT ROADMAP]");
        match self.roadmap_tool.list_project_roadmap(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tList project roadmap failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "List project roadmap failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create roadmap phase")]
    pub async fn create_phase(
        &self,
        Parameters(param): Parameters<CreatePhaseParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE PHASE]");
        match self.phase_tool.create_phase(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate phase failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create phase failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get roadmap phase by UUID")]
    pub async fn get_phase(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET PHASE]");
        match self.phase_tool.get_phase(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet phase failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get phase failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Update roadmap phase fields by UUID")]
    pub async fn update_phase(
        &self,
        Parameters(param): Parameters<UpdatePhaseParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE PHASE]");
        match self.phase_tool.update_phase(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tUpdate phase failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Update phase failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete roadmap phase by UUID")]
    pub async fn delete_phase(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE PHASE]");
        match self.phase_tool.delete_phase(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete phase failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete phase failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create milestone")]
    pub async fn create_milestone(
        &self,
        Parameters(param): Parameters<CreateMilestoneParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE MILESTONE]");
        match self.milestone_tool.create_milestone(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate milestone failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create milestone failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get milestone by UUID")]
    pub async fn get_milestone(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET MILESTONE]");
        match self.milestone_tool.get_milestone(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet milestone failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get milestone failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Update milestone fields by UUID")]
    pub async fn update_milestone(
        &self,
        Parameters(param): Parameters<UpdateMilestoneParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE MILESTONE]");
        match self.milestone_tool.update_milestone(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tUpdate milestone failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Update milestone failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete milestone by UUID")]
    pub async fn delete_milestone(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE MILESTONE]");
        match self.milestone_tool.delete_milestone(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete milestone failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete milestone failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create task")]
    pub async fn create_task(
        &self,
        Parameters(param): Parameters<CreateTaskParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE TASK]");
        match self.task_tool.create_task(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate task failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create task failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get task by UUID")]
    pub async fn get_task(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET TASK]");
        match self.task_tool.get_task(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet task failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get task failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Update task fields by UUID")]
    pub async fn update_task(
        &self,
        Parameters(param): Parameters<UpdateTaskParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE TASK]");
        match self.task_tool.update_task(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tUpdate task failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Update task failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete task by UUID")]
    pub async fn delete_task(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE TASK]");
        match self.task_tool.delete_task(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete task failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete task failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create or update task progress for a user")]
    pub async fn update_task_progress(
        &self,
        Parameters(param): Parameters<UpdateTaskProgressParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [UPDATE TASK PROGRESS]");
        match self.progress_tool.update_task_progress(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tUpdate task progress failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Update task progress failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get task progress for a user and task")]
    pub async fn get_task_progress(
        &self,
        Parameters(param): Parameters<GetTaskProgressParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET TASK PROGRESS]");
        match self.progress_tool.get_task_progress(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet task progress failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get task progress failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get aggregate project progress for a user")]
    pub async fn get_project_progress(
        &self,
        Parameters(param): Parameters<GetProjectProgressParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET PROJECT PROGRESS]");
        match self.progress_tool.get_project_progress(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet project progress failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get project progress failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create learning resource for a task")]
    pub async fn create_resource(
        &self,
        Parameters(param): Parameters<CreateResourceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE RESOURCE]");
        match self.resource_tool.create_resource(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate resource failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create resource failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Delete learning resource by UUID")]
    pub async fn delete_resource(
        &self,
        Parameters(param): Parameters<IdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DELETE RESOURCE]");
        match self.resource_tool.delete_resource(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tDelete resource failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Delete resource failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "List learning resources for a task")]
    pub async fn list_resources(
        &self,
        Parameters(param): Parameters<ListResourcesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LIST RESOURCES]");
        match self.resource_tool.list_resources(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tList resources failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "List resources failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Search projects by keyword for LLM retrieval")]
    pub async fn search_projects(
        &self,
        Parameters(param): Parameters<SearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [SEARCH PROJECTS]");
        match self.search_tool.search_projects(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tSearch projects failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Search projects failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Search tasks by keyword for LLM retrieval")]
    pub async fn search_tasks(
        &self,
        Parameters(param): Parameters<SearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [SEARCH TASKS]");
        match self.search_tool.search_tasks(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tSearch tasks failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Search tasks failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Search notes by keyword for LLM retrieval")]
    pub async fn search_notes(
        &self,
        Parameters(param): Parameters<SearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [SEARCH NOTES]");
        match self.search_tool.search_notes(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tSearch notes failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Search notes failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get user learning statistics for Roadmap MCP Server and LLM")]
    pub async fn get_user_statistics(
        &self,
        Parameters(param): Parameters<UserIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET USER STATISTICS]");
        match self.analytics_tool.get_user_statistics(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet user statistics failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get user statistics failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Get user learning history for Roadmap MCP Server and LLM")]
    pub async fn get_learning_history(
        &self,
        Parameters(param): Parameters<UserIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET LEARNING HISTORY]");
        match self.analytics_tool.get_learning_history(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tGet learning history failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get learning history failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create or upsert a persisted lesson from Lesson MCP finalizer output")]
    pub async fn create_lesson(
        &self,
        Parameters(param): Parameters<CreateLessonParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE LESSON]");
        match self.lesson_tool.create_lesson(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate lesson failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create lesson failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create a lesson content block")]
    pub async fn create_lesson_block(
        &self,
        Parameters(param): Parameters<CreateLessonBlockParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE LESSON BLOCK]");
        match self.lesson_tool.create_lesson_block(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate lesson block failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create lesson block failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Link a resource reference to a lesson")]
    pub async fn link_lesson_resource(
        &self,
        Parameters(param): Parameters<LinkLessonResourceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [LINK LESSON RESOURCE]");
        match self.lesson_tool.link_lesson_resource(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tLink lesson resource failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Link lesson resource failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create a lesson exercise")]
    pub async fn create_lesson_exercise(
        &self,
        Parameters(param): Parameters<CreateLessonExerciseParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE LESSON EXERCISE]");
        match self.lesson_tool.create_lesson_exercise(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate lesson exercise failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create lesson exercise failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Create a lesson quiz")]
    pub async fn create_lesson_quiz(
        &self,
        Parameters(param): Parameters<CreateLessonQuizParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [CREATE LESSON QUIZ]");
        match self.lesson_tool.create_lesson_quiz(param).await {
            Ok(value) => Ok(Self::json_success(value)),
            Err(e) => {
                error!("\tCreate lesson quiz failed: {e}");
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Create lesson quiz failed: {e}"
                ))]))
            }
        }
    }

    #[tool(description = "Infomation of multi schemas in database")]
    pub async fn get_multi_schema(
        &self,
        Parameters(params): Parameters<GetMultiTableSchema>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        info!("\tCALL TOOL: [GET MULTI SCHEMA]");
        let invalid_tb: Option<String> = Some(
            params
                .table_names
                .iter()
                .filter(|tb| !VALID_TABLES.contains(&tb.as_str()))
                .cloned()
                .collect(),
        );

        if let Some(ivl) = invalid_tb {
            warn!("\tInvalid Table: {ivl}");
        }

        let mut provider_locking = self.provider.lock().await;
        let result = provider_locking
            .get_multi_table_schema(params.table_names)
            .await;
        info!("\tTOOL [GET MULTI SCHEMA] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Infomation of schema in Database")]
    pub async fn get_schema(
        &self,
        Parameters(param): Parameters<GetTableSchema>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET SCHEMA]");
        if !VALID_TABLES.contains(&param.table_name.as_str()) {
            return Ok(CallToolResult::error(vec![Content::text("Invalid schema")]));
        };

        let mut provider_locking = self.provider.lock().await;
        let schema = match provider_locking.get_table_schema(&param.table_name).await {
            Ok(db) => {
                info!("\tGet Table Information Success!!!!");
                Value::from(db)
            }
            Err(e) => {
                error!("\tGet Table Information Failed!!!");
                error!("\tErr: {e}");
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Get table information failed: {e}"
                ))]));
            }
        };
        info!("\tTOOL: [GET SCHEMA] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&schema).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Health check database")]
    pub async fn get_health_check(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [HEALTH CHECK]");
        let mut provider_locking = self.provider.lock().await;
        let health = match provider_locking.health_check().await {
            Ok(result) => {
                info!("\tHealth check success!!!");
                Value::Object(result)
            }
            Err(e) => {
                error!("\thealth check failed: {e}");
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Health check failed: {e}"
                ))]));
            }
        };
        info!("\tTOOL: [HEALTH CHECK] CALL COMPLETED");

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&health).unwrap_or_default(),
        )]))
    }
}

impl Drop for DbServer {
    fn drop(&mut self) {
        info!("\tShutting down Database Server!!!!");
    }
}

fn allowed_mcp_hosts() -> Vec<String> {
    std::env::var("MCP_ALLOWED_HOSTS")
        .unwrap_or_else(|_| {
            "localhost,127.0.0.1,::1,database-mcp,roadmap-mcp,resource-mcp,lesson-mcp".to_string()
        })
        .split(',')
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .map(ToString::to_string)
        .collect()
}

async fn latest_roadmap_handler(
    State(server): State<DbServer>,
    Query(query): Query<LatestRoadmapQuery>,
) -> Result<Json<LatestRoadmapResponse>, (StatusCode, Json<Value>)> {
    match load_latest_roadmap(&server, query.user_id).await {
        Ok(roadmap) => Ok(Json(LatestRoadmapResponse { roadmap })),
        Err(err) => {
            error!("Load latest roadmap failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "LATEST_ROADMAP_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn notes_handler(
    State(server): State<DbServer>,
    Query(query): Query<UserDataQuery>,
) -> Result<Json<NotesResponse>, (StatusCode, Json<Value>)> {
    match load_notes(&server, query.user_id).await {
        Ok(notes) => Ok(Json(NotesResponse { notes })),
        Err(err) => {
            error!("Load notes failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "NOTES_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn create_note_handler(
    State(server): State<DbServer>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match create_note(&server, payload).await {
        Ok(note) => Ok(Json(json!({ "note": note }))),
        Err(err) => {
            error!("Create note failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "CREATE_NOTE_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn review_handler(
    State(server): State<DbServer>,
    Query(query): Query<UserDataQuery>,
) -> Result<Json<ReviewResponse>, (StatusCode, Json<Value>)> {
    match load_review_items(&server, query.user_id).await {
        Ok(review_items) => Ok(Json(ReviewResponse { review_items })),
        Err(err) => {
            error!("Load review failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "REVIEW_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn seed_roadmap_handler(
    State(server): State<DbServer>,
    Json(payload): Json<SeedRoadmapRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if env::var("E2E_ENABLE_TEST_ROUTES").unwrap_or_default() != "true" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "NOT_FOUND",
                    "message": "not found"
                }
            })),
        ));
    }

    match seed_test_roadmap(&server, payload).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            error!("Seed test roadmap failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "SEED_ROADMAP_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn seed_learning_data_handler(
    State(server): State<DbServer>,
    Json(payload): Json<SeedLearningDataRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if env::var("E2E_ENABLE_TEST_ROUTES").unwrap_or_default() != "true" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "NOT_FOUND",
                    "message": "not found"
                }
            })),
        ));
    }

    match seed_test_learning_data(&server, payload).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            error!("Seed test learning data failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "SEED_LEARNING_DATA_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn reset_test_data_handler(
    State(server): State<DbServer>,
    Json(payload): Json<ResetTestDataRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if env::var("E2E_ENABLE_TEST_ROUTES").unwrap_or_default() != "true" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "NOT_FOUND",
                    "message": "not found"
                }
            })),
        ));
    }

    match reset_test_data(&server, payload).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            error!("Reset test data failed: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": "RESET_TEST_DATA_FAILED",
                        "message": err.to_string()
                    }
                })),
            ))
        }
    }
}

async fn load_latest_roadmap(
    server: &DbServer,
    user_id: Option<String>,
) -> anyhow::Result<Option<Value>> {
    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;
    let user_filter = user_id.filter(|value| !value.trim().is_empty());

    if !all_tables_exist(&conn, &["roadmaps", "projects", "users"]).await? {
        warn!("Latest roadmap read skipped because required tables do not exist");
        return Ok(None);
    }

    let roadmap_row = conn
        .query_opt(
            "SELECT
                r.id,
                r.project_id,
                r.version,
                r.title,
                r.generated_by,
                r.created_at::text,
                p.title AS project_title,
                p.description AS project_description
             FROM roadmaps r
             JOIN projects p ON p.id = r.project_id
             JOIN users u ON u.id = p.user_id
             WHERE ($1::text IS NULL OR u.firebase_uid = $1 OR p.user_id::text = $1)
             ORDER BY r.created_at DESC
             LIMIT 1",
            &[&user_filter],
        )
        .await?;

    let Some(roadmap_row) = roadmap_row else {
        return Ok(None);
    };

    let roadmap_id: uuid::Uuid = roadmap_row.get("id");
    let phase_rows = conn
        .query(
            "SELECT id, roadmap_id, phase_order, title, description, estimated_days
             FROM roadmap_phases
             WHERE roadmap_id = $1
             ORDER BY phase_order ASC",
            &[&roadmap_id],
        )
        .await?;

    let mut phases = Vec::new();
    for phase_row in phase_rows {
        let phase_id: uuid::Uuid = phase_row.get("id");
        let milestone_rows = conn
            .query(
                "SELECT id, phase_id, milestone_order, title, description
                 FROM milestones
                 WHERE phase_id = $1
                 ORDER BY milestone_order ASC",
                &[&phase_id],
            )
            .await?;

        let mut milestones = Vec::new();
        for milestone_row in milestone_rows {
            let milestone_id: uuid::Uuid = milestone_row.get("id");
            let task_rows = conn
                .query(
                    "SELECT id, milestone_id, task_order, title, description, estimated_hours, difficulty, status
                     FROM tasks
                     WHERE milestone_id = $1
                     ORDER BY task_order ASC",
                    &[&milestone_id],
                )
                .await?;

            milestones.push(json!({
                "id": milestone_id.to_string(),
                "phase_id": phase_id.to_string(),
                "milestone_order": milestone_row.get::<_, i32>("milestone_order"),
                "title": milestone_row.get::<_, String>("title"),
                "description": milestone_row.get::<_, Option<String>>("description"),
                "tasks": task_rows.into_iter().map(|task_row| {
                    let task_id: uuid::Uuid = task_row.get("id");
                    json!({
                        "id": task_id.to_string(),
                        "milestone_id": milestone_id.to_string(),
                        "task_order": task_row.get::<_, i32>("task_order"),
                        "title": task_row.get::<_, String>("title"),
                        "description": task_row.get::<_, Option<String>>("description"),
                        "estimated_hours": task_row.get::<_, Option<i32>>("estimated_hours"),
                        "difficulty": task_row.get::<_, Option<String>>("difficulty"),
                        "status": task_row.get::<_, Option<String>>("status"),
                    })
                }).collect::<Vec<Value>>(),
            }));
        }

        phases.push(json!({
            "id": phase_id.to_string(),
            "roadmap_id": roadmap_id.to_string(),
            "phase_order": phase_row.get::<_, i32>("phase_order"),
            "title": phase_row.get::<_, String>("title"),
            "description": phase_row.get::<_, Option<String>>("description"),
            "estimated_days": phase_row.get::<_, Option<i32>>("estimated_days"),
            "milestones": milestones,
        }));
    }

    Ok(Some(json!({
        "id": roadmap_id.to_string(),
        "project_id": roadmap_row.get::<_, uuid::Uuid>("project_id").to_string(),
        "version": roadmap_row.get::<_, i32>("version"),
        "title": roadmap_row.get::<_, Option<String>>("title"),
        "generated_by": roadmap_row.get::<_, Option<String>>("generated_by"),
        "created_at": roadmap_row.get::<_, Option<String>>("created_at"),
        "project_title": roadmap_row.get::<_, String>("project_title"),
        "project_description": roadmap_row.get::<_, Option<String>>("project_description"),
        "phases": phases,
    })))
}

async fn load_notes(server: &DbServer, user_id: Option<String>) -> anyhow::Result<Vec<Value>> {
    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;
    let user_filter = user_id.filter(|value| !value.trim().is_empty());

    if !all_tables_exist(&conn, &["notes", "users"]).await? {
        warn!("Notes read skipped because required tables do not exist");
        return Ok(Vec::new());
    }

    let has_task_context = all_tables_exist(
        &conn,
        &[
            "tasks",
            "milestones",
            "roadmap_phases",
            "roadmaps",
            "projects",
        ],
    )
    .await?;

    let rows = if has_task_context {
        conn.query(
            "SELECT
                n.id,
                n.user_id,
                n.task_id,
                n.content,
                n.created_at::text,
                t.title AS task_title,
                p.title AS project_title
             FROM notes n
             JOIN users u ON u.id = n.user_id
             LEFT JOIN tasks t ON t.id = n.task_id
             LEFT JOIN milestones m ON m.id = t.milestone_id
             LEFT JOIN roadmap_phases ph ON ph.id = m.phase_id
             LEFT JOIN roadmaps r ON r.id = ph.roadmap_id
             LEFT JOIN projects p ON p.id = r.project_id
             WHERE ($1::text IS NULL OR u.firebase_uid = $1 OR n.user_id::text = $1)
             ORDER BY n.created_at DESC
             LIMIT 100",
            &[&user_filter],
        )
        .await?
    } else {
        conn.query(
            "SELECT
                n.id,
                n.user_id,
                n.task_id,
                n.content,
                n.created_at::text,
                NULL::text AS task_title,
                NULL::text AS project_title
             FROM notes n
             JOIN users u ON u.id = n.user_id
             WHERE ($1::text IS NULL OR u.firebase_uid = $1 OR n.user_id::text = $1)
             ORDER BY n.created_at DESC
             LIMIT 100",
            &[&user_filter],
        )
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|row| {
            let note_id: uuid::Uuid = row.get("id");
            let owner_id: uuid::Uuid = row.get("user_id");
            let task_id = row.get::<_, Option<uuid::Uuid>>("task_id");
            json!({
                "id": note_id.to_string(),
                "user_id": owner_id.to_string(),
                "task_id": task_id.map(|value| value.to_string()),
                "content": row.get::<_, String>("content"),
                "created_at": row.get::<_, Option<String>>("created_at"),
                "task_title": row.get::<_, Option<String>>("task_title"),
                "project_title": row.get::<_, Option<String>>("project_title"),
            })
        })
        .collect())
}

async fn create_note(server: &DbServer, payload: CreateNoteRequest) -> anyhow::Result<Value> {
    let user_id = payload.user_id.trim().to_string();
    let content = payload.content.trim().to_string();
    if user_id.is_empty() {
        anyhow::bail!("userId is required");
    }
    if content.is_empty() {
        anyhow::bail!("content is required");
    }

    let task_id = match payload
        .task_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(uuid::Uuid::parse_str(value)?),
        None => None,
    };

    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;
    ensure_note_tables(&conn).await?;

    let row = conn
        .query_one(
            "WITH u AS (
                INSERT INTO users(firebase_uid, display_name, email)
                VALUES ($1, NULL, NULL)
                ON CONFLICT (firebase_uid)
                DO UPDATE SET updated_at = now()
                RETURNING id
             )
             INSERT INTO notes(user_id, task_id, content)
             SELECT id, $2, $3 FROM u
             RETURNING id, user_id, task_id, content, created_at::text",
            &[&user_id, &task_id, &content],
        )
        .await?;

    let note_id: uuid::Uuid = row.get("id");
    let owner_id: uuid::Uuid = row.get("user_id");
    let task_id = row.get::<_, Option<uuid::Uuid>>("task_id");
    Ok(json!({
        "id": note_id.to_string(),
        "user_id": owner_id.to_string(),
        "task_id": task_id.map(|value| value.to_string()),
        "content": row.get::<_, String>("content"),
        "created_at": row.get::<_, Option<String>>("created_at"),
        "task_title": Value::Null,
        "project_title": Value::Null,
    }))
}

async fn load_review_items(
    server: &DbServer,
    user_id: Option<String>,
) -> anyhow::Result<Vec<Value>> {
    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;
    let user_filter = user_id.filter(|value| !value.trim().is_empty());

    if !all_tables_exist(
        &conn,
        &[
            "tasks",
            "milestones",
            "roadmap_phases",
            "roadmaps",
            "projects",
            "users",
            "task_progress",
        ],
    )
    .await?
    {
        warn!("Review read skipped because required tables do not exist");
        return Ok(Vec::new());
    }

    let rows = conn
        .query(
            "SELECT
                t.id,
                t.title,
                t.description,
                t.difficulty,
                p.title AS project_title,
                COALESCE(tp.status, t.status, 'pending') AS review_status,
                COALESCE(tp.progress_percent, 0) AS progress_percent,
                tp.started_at::text,
                tp.completed_at::text
             FROM projects p
             JOIN users u ON u.id = p.user_id
             JOIN roadmaps r ON r.project_id = p.id
             JOIN roadmap_phases ph ON ph.roadmap_id = r.id
             JOIN milestones m ON m.phase_id = ph.id
             JOIN tasks t ON t.milestone_id = m.id
             LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.user_id = u.id
             WHERE ($1::text IS NULL OR u.firebase_uid = $1 OR u.id::text = $1)
               AND COALESCE(tp.status, t.status, 'pending') <> 'completed'
             ORDER BY
                CASE COALESCE(tp.status, t.status, 'pending')
                    WHEN 'needs_review' THEN 0
                    WHEN 'in_progress' THEN 1
                    ELSE 2
                END,
                tp.started_at ASC NULLS FIRST,
                t.task_order ASC
             LIMIT 50",
            &[&user_filter],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let task_id: uuid::Uuid = row.get("id");
            json!({
                "id": task_id.to_string(),
                "task_id": task_id.to_string(),
                "title": row.get::<_, String>("title"),
                "description": row.get::<_, Option<String>>("description"),
                "difficulty": row.get::<_, Option<String>>("difficulty"),
                "project_title": row.get::<_, String>("project_title"),
                "status": row.get::<_, String>("review_status"),
                "progress_percent": row.get::<_, i32>("progress_percent"),
                "started_at": row.get::<_, Option<String>>("started_at"),
                "completed_at": row.get::<_, Option<String>>("completed_at"),
            })
        })
        .collect())
}

async fn all_tables_exist(
    conn: &deadpool_postgres::Object,
    table_names: &[&str],
) -> anyhow::Result<bool> {
    for table_name in table_names {
        let exists = conn
            .query_one(
                "SELECT to_regclass($1) IS NOT NULL AS exists",
                &[table_name],
            )
            .await?
            .get::<_, bool>("exists");
        if !exists {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn table_exists(conn: &deadpool_postgres::Object, table_name: &str) -> anyhow::Result<bool> {
    Ok(conn
        .query_one(
            "SELECT to_regclass($1) IS NOT NULL AS exists",
            &[&table_name],
        )
        .await?
        .get::<_, bool>("exists"))
}

async fn ensure_note_tables(conn: &deadpool_postgres::Object) -> anyhow::Result<()> {
    conn.batch_execute(
        "CREATE EXTENSION IF NOT EXISTS pgcrypto;
         CREATE TABLE IF NOT EXISTS users (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            firebase_uid text NOT NULL UNIQUE,
            display_name text,
            email text,
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS notes (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id uuid NOT NULL REFERENCES users(id),
            task_id uuid,
            content text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
         );",
    )
    .await?;
    Ok(())
}

async fn reset_test_data(
    server: &DbServer,
    payload: ResetTestDataRequest,
) -> anyhow::Result<Value> {
    let user_id = payload
        .user_id
        .unwrap_or_else(|| "dev-learner".to_string())
        .trim()
        .to_string();
    if user_id.is_empty() {
        anyhow::bail!("userId is required");
    }

    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;

    if !table_exists(&conn, "users").await? {
        return Ok(json!({
            "userId": user_id,
            "deletedRows": 0
        }));
    }

    let mut deleted_rows = 0_u64;

    if table_exists(&conn, "notes").await? {
        deleted_rows += conn
            .execute(
                "DELETE FROM notes n
                 USING users u
                 WHERE n.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;
    }

    if table_exists(&conn, "task_progress").await? {
        deleted_rows += conn
            .execute(
                "DELETE FROM task_progress tp
                 USING users u
                 WHERE tp.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;
    }

    let has_roadmap_graph = all_tables_exist(
        &conn,
        &[
            "projects",
            "roadmaps",
            "roadmap_phases",
            "milestones",
            "tasks",
        ],
    )
    .await?;

    if has_roadmap_graph {
        deleted_rows += conn
            .execute(
                "DELETE FROM tasks t
                 USING milestones m, roadmap_phases ph, roadmaps r, projects p, users u
                 WHERE t.milestone_id = m.id
                   AND m.phase_id = ph.id
                   AND ph.roadmap_id = r.id
                   AND r.project_id = p.id
                   AND p.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;

        deleted_rows += conn
            .execute(
                "DELETE FROM milestones m
                 USING roadmap_phases ph, roadmaps r, projects p, users u
                 WHERE m.phase_id = ph.id
                   AND ph.roadmap_id = r.id
                   AND r.project_id = p.id
                   AND p.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;

        deleted_rows += conn
            .execute(
                "DELETE FROM roadmap_phases ph
                 USING roadmaps r, projects p, users u
                 WHERE ph.roadmap_id = r.id
                   AND r.project_id = p.id
                   AND p.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;
    }

    if all_tables_exist(&conn, &["roadmaps", "projects"]).await? {
        deleted_rows += conn
            .execute(
                "DELETE FROM roadmaps r
                 USING projects p, users u
                 WHERE r.project_id = p.id
                   AND p.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;
    }

    if table_exists(&conn, "projects").await? {
        deleted_rows += conn
            .execute(
                "DELETE FROM projects p
                 USING users u
                 WHERE p.user_id = u.id
                   AND (u.firebase_uid = $1 OR u.id::text = $1)",
                &[&user_id],
            )
            .await?;
    }

    deleted_rows += conn
        .execute(
            "DELETE FROM users
             WHERE firebase_uid = $1 OR id::text = $1",
            &[&user_id],
        )
        .await?;

    Ok(json!({
        "userId": user_id,
        "deletedRows": deleted_rows
    }))
}

async fn seed_test_roadmap(
    server: &DbServer,
    payload: SeedRoadmapRequest,
) -> anyhow::Result<Value> {
    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;

    conn.batch_execute(
        "CREATE EXTENSION IF NOT EXISTS pgcrypto;
         CREATE TABLE IF NOT EXISTS users (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            firebase_uid text NOT NULL UNIQUE,
            display_name text,
            email text,
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS projects (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id uuid NOT NULL REFERENCES users(id),
            title text NOT NULL,
            description text,
            status text NOT NULL DEFAULT 'draft',
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS roadmaps (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            project_id uuid NOT NULL REFERENCES projects(id),
            version int NOT NULL DEFAULT 1,
            title text,
            generated_by text,
            created_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS roadmap_phases (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            roadmap_id uuid NOT NULL REFERENCES roadmaps(id),
            phase_order int NOT NULL,
            title text NOT NULL,
            description text,
            estimated_days int
         );
         CREATE TABLE IF NOT EXISTS milestones (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            phase_id uuid NOT NULL REFERENCES roadmap_phases(id),
            milestone_order int NOT NULL,
            title text NOT NULL,
            description text
         );
         CREATE TABLE IF NOT EXISTS tasks (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            milestone_id uuid NOT NULL REFERENCES milestones(id),
            task_order int NOT NULL,
            title text NOT NULL,
            description text,
            estimated_hours int,
            difficulty text,
            status text
         );",
    )
    .await?;

    let project_title = format!("{} project", payload.title);
    let row = conn
        .query_one(
            "WITH u AS (
                INSERT INTO users(firebase_uid, display_name, email)
                VALUES ($1, 'Dev Learner', 'dev-learner@local.test')
                ON CONFLICT (firebase_uid)
                DO UPDATE SET display_name = EXCLUDED.display_name, email = EXCLUDED.email, updated_at = now()
                RETURNING id
             ), p AS (
                INSERT INTO projects(user_id, title, description, status)
                SELECT id, $2, 'DB-backed roadmap test', 'active' FROM u
                RETURNING id
             ), r AS (
                INSERT INTO roadmaps(project_id, version, title, generated_by, created_at)
                SELECT id, 1, $3, 'database-mcp', now() + interval '1 hour' FROM p
                RETURNING id
             ), ph AS (
                INSERT INTO roadmap_phases(roadmap_id, phase_order, title, description, estimated_days)
                SELECT id, 1, 'Database phase', 'Loaded from database', 5 FROM r
                RETURNING id
             ), m AS (
                INSERT INTO milestones(phase_id, milestone_order, title, description)
                SELECT id, 1, 'Database milestone', 'Milestone from database' FROM ph
                RETURNING id
             )
             INSERT INTO tasks(milestone_id, task_order, title, description, estimated_hours, difficulty, status)
             SELECT id, 1, 'Database task appears on roadmap', 'Task loaded by /roadmap', 2, 'easy', 'pending' FROM m
             RETURNING id",
            &[&payload.user_id, &project_title, &payload.title],
        )
        .await?;
    let task_id: uuid::Uuid = row.get("id");

    Ok(json!({
        "ok": true,
        "taskId": task_id.to_string(),
    }))
}

async fn seed_test_learning_data(
    server: &DbServer,
    payload: SeedLearningDataRequest,
) -> anyhow::Result<Value> {
    let mut provider = server.provider.lock().await;
    let conn = provider.get_connections().await?;

    conn.batch_execute(
        "CREATE EXTENSION IF NOT EXISTS pgcrypto;
         CREATE TABLE IF NOT EXISTS users (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            firebase_uid text NOT NULL UNIQUE,
            display_name text,
            email text,
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS projects (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id uuid NOT NULL REFERENCES users(id),
            title text NOT NULL,
            description text,
            status text NOT NULL DEFAULT 'draft',
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS roadmaps (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            project_id uuid NOT NULL REFERENCES projects(id),
            version int NOT NULL DEFAULT 1,
            title text,
            generated_by text,
            created_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS roadmap_phases (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            roadmap_id uuid NOT NULL REFERENCES roadmaps(id),
            phase_order int NOT NULL,
            title text NOT NULL,
            description text,
            estimated_days int
         );
         CREATE TABLE IF NOT EXISTS milestones (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            phase_id uuid NOT NULL REFERENCES roadmap_phases(id),
            milestone_order int NOT NULL,
            title text NOT NULL,
            description text
         );
         CREATE TABLE IF NOT EXISTS tasks (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            milestone_id uuid NOT NULL REFERENCES milestones(id),
            task_order int NOT NULL,
            title text NOT NULL,
            description text,
            estimated_hours int,
            difficulty text,
            status text
         );
         CREATE TABLE IF NOT EXISTS notes (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id uuid NOT NULL REFERENCES users(id),
            task_id uuid REFERENCES tasks(id),
            content text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
         );
         CREATE TABLE IF NOT EXISTS task_progress (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id uuid NOT NULL REFERENCES users(id),
            task_id uuid NOT NULL REFERENCES tasks(id),
            status text NOT NULL,
            progress_percent int,
            started_at timestamptz,
            completed_at timestamptz,
            UNIQUE(user_id, task_id)
         );",
    )
    .await?;

    let project_title = format!("{} project", payload.title);
    let task_title = format!("{} review task", payload.title);
    let note_content = format!(
        "{} note\nThis note was persisted in Database MCP and loaded by the Notes page.",
        payload.title
    );

    let row = conn
        .query_one(
            "WITH u AS (
                INSERT INTO users(firebase_uid, display_name, email)
                VALUES ($1, 'Dev Learner', 'dev-learner@local.test')
                ON CONFLICT (firebase_uid)
                DO UPDATE SET display_name = EXCLUDED.display_name, email = EXCLUDED.email, updated_at = now()
                RETURNING id
             ), p AS (
                INSERT INTO projects(user_id, title, description, status)
                SELECT id, $2, 'DB-backed learning data test', 'active' FROM u
                RETURNING id, user_id
             ), r AS (
                INSERT INTO roadmaps(project_id, version, title, generated_by)
                SELECT id, 1, $3, 'database-mcp' FROM p
                RETURNING id
             ), ph AS (
                INSERT INTO roadmap_phases(roadmap_id, phase_order, title, description, estimated_days)
                SELECT id, 1, 'Learning data phase', 'Loaded from database', 3 FROM r
                RETURNING id
             ), m AS (
                INSERT INTO milestones(phase_id, milestone_order, title, description)
                SELECT id, 1, 'Learning data milestone', 'Milestone from database' FROM ph
                RETURNING id
             ), t AS (
                INSERT INTO tasks(milestone_id, task_order, title, description, estimated_hours, difficulty, status)
                SELECT id, 1, $4, 'Task loaded by /review', 2, 'medium', 'needs_review' FROM m
                RETURNING id
             ), progress AS (
                INSERT INTO task_progress(user_id, task_id, status, progress_percent, started_at)
                SELECT p.user_id, t.id, 'needs_review', 35, now() FROM p, t
                ON CONFLICT (user_id, task_id)
                DO UPDATE SET status = EXCLUDED.status, progress_percent = EXCLUDED.progress_percent, started_at = EXCLUDED.started_at
                RETURNING id
             )
             INSERT INTO notes(user_id, task_id, content)
             SELECT p.user_id, t.id, $5 FROM p, t
             RETURNING id",
            &[
                &payload.user_id,
                &project_title,
                &payload.title,
                &task_title,
                &note_content,
            ],
        )
        .await?;
    let note_id: uuid::Uuid = row.get("id");

    Ok(json!({
        "ok": true,
        "noteId": note_id.to_string(),
    }))
}

#[tool_handler]
impl ServerHandler for DbServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let ins = format!(
            "Access Data Store in Database with valid schemas: {}",
            VALID_TABLES.join(" ")
        );
        let sv = Implementation::new("Database Server".to_string(), "1.0".to_string())
            .with_website_url(self.config.url.clone())
            .with_description("Access and Store data of roadmap, lesson, user, progress, messages, AI conservations".to_string())
            .with_title("Data Access Server".to_string());

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();

        let info = ServerInfo::new(capabilities)
            .with_instructions(ins)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(sv);
        info
    }
}
