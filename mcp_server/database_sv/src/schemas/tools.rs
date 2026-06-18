use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IdParam {
    #[schemars(description = "Target row UUID.")]
    pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateUserParam {
    #[schemars(description = "Firebase unique user id. Maps to users.firebase_uid.")]
    pub firebase_id: String,
    #[schemars(description = "Optional human-readable user display name.")]
    pub display_name: Option<String>,
    #[schemars(description = "Optional user email address.")]
    #[schemars(email)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetUserByIdParam {
    #[schemars(description = "Firebase unique user id. Maps to users.firebase_uid.")]
    pub firebase_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateProjectParam {
    #[schemars(description = "Owner user UUID.")]
    pub user_id: Uuid,
    #[schemars(description = "Project title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: String,
    #[schemars(description = "Optional project description.")]
    pub description: Option<String>,
    #[schemars(description = "Project status. Defaults to draft when omitted.")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateProjectParam {
    #[schemars(description = "Project UUID.")]
    pub id: Uuid,
    #[schemars(description = "New project title. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "New project description. Omit to keep current value.")]
    pub description: Option<String>,
    #[schemars(description = "New project status. Omit to keep current value.")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListProjectsParam {
    #[schemars(description = "Owner user UUID.")]
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateRoadmapParam {
    #[schemars(description = "Project UUID this roadmap belongs to.")]
    pub project_id: Uuid,
    #[schemars(description = "Roadmap version number within a project.")]
    #[schemars(range(min = 1))]
    pub version: i32,
    #[schemars(description = "Optional roadmap title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "Source that generated this roadmap, such as llm, user, or system.")]
    #[schemars(length(min = 1, max = 100))]
    pub generated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListProjectRoadmapParam {
    #[schemars(description = "Project UUID used to list its roadmaps.")]
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreatePhaseParam {
    #[schemars(description = "Roadmap UUID this phase belongs to.")]
    pub roadmap_id: Uuid,
    #[schemars(description = "Display order of the phase inside the roadmap.")]
    #[schemars(range(min = 1))]
    pub phase_order: i32,
    #[schemars(description = "Phase title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: String,
    #[schemars(description = "Optional phase description.")]
    pub description: Option<String>,
    #[schemars(description = "Estimated number of days needed to complete the phase.")]
    #[schemars(range(min = 0))]
    pub estimated_days: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdatePhaseParam {
    #[schemars(description = "Phase UUID.")]
    pub id: Uuid,
    #[schemars(description = "New phase order. Omit to keep current value.")]
    #[schemars(range(min = 1))]
    pub phase_order: Option<i32>,
    #[schemars(description = "New phase title. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "New phase description. Omit to keep current value.")]
    pub description: Option<String>,
    #[schemars(description = "New estimated days. Omit to keep current value.")]
    #[schemars(range(min = 0))]
    pub estimated_days: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateMilestoneParam {
    #[schemars(description = "Phase UUID this milestone belongs to.")]
    pub phase_id: Uuid,
    #[schemars(description = "Display order of the milestone inside the phase.")]
    #[schemars(range(min = 1))]
    pub milestone_order: i32,
    #[schemars(description = "Milestone title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: String,
    #[schemars(description = "Optional milestone description.")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateMilestoneParam {
    #[schemars(description = "Milestone UUID.")]
    pub id: Uuid,
    #[schemars(description = "New milestone order. Omit to keep current value.")]
    #[schemars(range(min = 1))]
    pub milestone_order: Option<i32>,
    #[schemars(description = "New milestone title. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "New milestone description. Omit to keep current value.")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateTaskParam {
    #[schemars(description = "Milestone UUID this task belongs to.")]
    pub milestone_id: Uuid,
    #[schemars(description = "Display order of the task inside the milestone.")]
    #[schemars(range(min = 1))]
    pub task_order: i32,
    #[schemars(description = "Task title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: String,
    #[schemars(description = "Optional task description.")]
    pub description: Option<String>,
    #[schemars(description = "Estimated number of hours needed to complete the task.")]
    #[schemars(range(min = 0))]
    pub estimated_hours: Option<i32>,
    #[schemars(description = "Task difficulty label, such as easy, medium, hard, or advanced.")]
    #[schemars(length(min = 1, max = 30))]
    pub difficulty: Option<String>,
    #[schemars(description = "Task status. Defaults to pending when omitted.")]
    #[schemars(length(min = 1, max = 30))]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateTaskParam {
    #[schemars(description = "Task UUID.")]
    pub id: Uuid,
    #[schemars(description = "New task order. Omit to keep current value.")]
    #[schemars(range(min = 1))]
    pub task_order: Option<i32>,
    #[schemars(description = "New task title. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "New task description. Omit to keep current value.")]
    pub description: Option<String>,
    #[schemars(description = "New estimated hours. Omit to keep current value.")]
    #[schemars(range(min = 0))]
    pub estimated_hours: Option<i32>,
    #[schemars(description = "New difficulty label. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 30))]
    pub difficulty: Option<String>,
    #[schemars(description = "New task status. Omit to keep current value.")]
    #[schemars(length(min = 1, max = 30))]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateTaskProgressParam {
    #[schemars(description = "User UUID that owns this task progress record.")]
    pub user_id: Uuid,
    #[schemars(description = "Task UUID being updated.")]
    pub task_id: Uuid,
    #[schemars(
        description = "Progress status, such as pending, in_progress, blocked, or completed."
    )]
    #[schemars(length(min = 1, max = 50))]
    pub status: String,
    #[schemars(description = "Task completion percentage from 0 to 100.")]
    #[schemars(range(min = 0, max = 100))]
    pub progress_percent: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetTaskProgressParam {
    #[schemars(description = "User UUID that owns this task progress record.")]
    pub user_id: Uuid,
    #[schemars(description = "Task UUID to retrieve progress for.")]
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetProjectProgressParam {
    #[schemars(description = "User UUID whose project progress is being calculated.")]
    pub user_id: Uuid,
    #[schemars(description = "Project UUID to calculate aggregate progress for.")]
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateResourceParam {
    #[schemars(description = "Task UUID this learning resource belongs to.")]
    pub task_id: Uuid,
    #[schemars(
        description = "Resource type, such as article, video, course, book, or documentation."
    )]
    #[schemars(length(min = 1, max = 50))]
    pub resource_type: Option<String>,
    #[schemars(description = "Learning resource title.")]
    #[schemars(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[schemars(description = "Learning resource URL.")]
    #[schemars(url)]
    pub url: Option<String>,
    #[schemars(description = "Optional learning resource description.")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListResourcesParam {
    #[schemars(description = "Task UUID used to list its learning resources.")]
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchParam {
    #[schemars(description = "Keyword used for case-insensitive search.")]
    #[schemars(length(min = 1))]
    pub keyword: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UserIdParam {
    #[schemars(description = "User UUID.")]
    pub user_id: Uuid,
}
