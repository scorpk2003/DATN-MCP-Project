use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AgentKernel, ServerConfig};

pub struct AppState {
    pub clients: Vec<ServerConfig>,
}
#[derive(Deserialize)]
pub struct AgentRequest {
    pub goal: String,
    pub session_id: String,
}
#[derive(Serialize)]
pub struct AgentResponse {
    pub success: bool,
    pub output: Option<Value>,
    pub message: String,
}
pub async fn agent_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AgentRequest>
) -> impl IntoResponse
{
    let server_configs = state.clients.clone();
    let mut kernel = match AgentKernel::new(server_configs, payload.session_id).await {
        Ok(k) => {k},
        Err(e) => {return (
            StatusCode::FAILED_DEPENDENCY,
            Json(AgentResponse {
                success: false,
                output: None,
                message: format!("Generate kernel failed, error: {:?}", e),
            })
        );}
    };
    let val = match kernel.run(payload.goal).await {
        Ok(v) => {v},
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(AgentResponse { success: false, output: None, message: format!("Running agent failed, error: {:?}", e), }))
        }
    };
    (StatusCode::ACCEPTED, Json(AgentResponse { success: true, output: Some(val), message: format!("Agent Run Successfully!!!") }))
}