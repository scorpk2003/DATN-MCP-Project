use std::sync::Arc;

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
use serde_json::Value;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    VALID_TABLES,
    provider::SchemaProvider,
    schemas::*,
    server::ServerConfig,
    tools::{
        AnalyticsTool, MilestoneTool, PhaseTool, ProgressTool, ProjectTool, ResourceTool,
        RoadmapTool, SearchTool, TaskTool, UserTool,
    },
};

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
            provider,
            tool_router: Self::tool_router(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Database Server at: {addr}");

        let config =
            StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new().nest_service("/mcp", service);

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
        let mut provider = self.provider.blocking_lock();
        let _ = provider.close_pool();
        info!("\tDatabase Server shutdown is completed!!!");
    }
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
