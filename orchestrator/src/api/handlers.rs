use axum::{Json, extract::State};
use serde_json::Value;

use crate::McpClient;

pub struct AppState {
    pub clients: Vec<McpClient>,
}
pub struct AgentRequest {
    pub goal: String,
}
pub struct AgentResponse {
    pub success: bool,
    pub output: Value,
    pub message: String,
}
pub async fn agent_handler(
    State(state): State<AppState>,
    Json(payload): Json<AgentRequest>
)
{}