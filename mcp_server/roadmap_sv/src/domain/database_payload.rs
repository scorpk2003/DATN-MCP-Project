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
        "get_user_profile",
        "get_user_learning_context",
        "list_user_roadmaps",
        "create_roadmap",
        "create_roadmap_phase",
        "create_roadmap_node",
        "create_roadmap_edge",
        "attach_resource_ref_to_node",
        "update_roadmap_status",
        "get_roadmap_detail",
        "create_audit_event",
    ]
}
