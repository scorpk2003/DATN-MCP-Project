use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseReadyRoadmapPayload {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "notPersisted")]
    pub not_persisted: bool,
    #[serde(rename = "persistenceOwner")]
    pub persistence_owner: PersistenceOwner,
    #[serde(rename = "requiredDatabaseCapabilities")]
    pub required_database_capabilities: Vec<String>,
    #[serde(rename = "orchestratorPersistencePlan")]
    pub orchestrator_persistence_plan: OrchestratorPersistencePlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceOwner {
    OrchestratorAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestratorPersistencePlan {
    #[serde(rename = "databaseMcpCalls")]
    pub database_mcp_calls: Vec<DatabaseMcpToolCall>,
    #[serde(rename = "transactionalPreference")]
    pub transactional_preference: String,
    #[serde(rename = "failurePolicy")]
    pub failure_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseMcpToolCall {
    #[serde(rename = "toolName")]
    pub tool_name: String,
    pub arguments: Value,
    #[serde(rename = "dependsOn")]
    pub depends_on: Vec<String>,
    #[serde(rename = "resultAlias")]
    pub result_alias: Option<String>,
}

pub fn required_database_capabilities() -> Vec<&'static str> {
    vec![
        "upsert_user",
        "get_user_by_id",
        "list_projects",
        "list_project_roadmap",
        "create_project",
        "create_roadmap",
        "create_phase",
        "create_milestone",
        "create_task",
        "create_resource",
    ]
}
