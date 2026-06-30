use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::info;

use crate::{AgentKernel, AuthContext, ExecutionState, ExecutionStatus, McpRegistry, ServerConfig};

pub struct AppState {
    pub clients: Vec<ServerConfig>,
    pub registry: McpRegistry,
    pub sessions: Mutex<HashMap<String, WaitingSession>>,
}

#[derive(Clone)]
pub struct WaitingSession {
    pub state: ExecutionState,
    pub expires_at: Instant,
}
#[derive(Deserialize)]
pub struct AgentRequest {
    pub goal: String,
    pub session_id: String,
    pub user_id: Option<String>,
    pub auth_context: Option<AuthContext>,
    pub context: Option<Value>,
}

#[derive(Deserialize)]
pub struct AgentResumeRequest {
    pub session_id: String,
    pub user_id: Option<String>,
    pub auth_context: Option<AuthContext>,
    pub approval: ApprovalDecision,
}

#[derive(Deserialize, Serialize)]
pub struct ApprovalDecision {
    pub step_id: String,
    pub decision: String,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct AgentResponse {
    pub ok: bool,
    pub session_id: Option<String>,
    pub status: String,
    pub output: Option<Value>,
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy)]
enum ApprovalAction {
    Approve,
    Reject,
    Revise,
}

impl ApprovalAction {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "approve" => Some(Self::Approve),
            "reject" => Some(Self::Reject),
            "revise" => Some(Self::Revise),
            _ => None,
        }
    }
}

impl WaitingSession {
    fn new(state: ExecutionState) -> Self {
        let ttl = Duration::from_secs(env_u64("AGENT_WAITING_SESSION_TTL_SECONDS", 1800));
        Self {
            state,
            expires_at: Instant::now() + ttl,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

pub async fn agent_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AgentRequest>,
) -> impl IntoResponse {
    info!(session_id = %payload.session_id, user_id = ?payload.user_id, "agent run requested");
    let server_configs = state.clients.clone();
    let mut kernel = match AgentKernel::new(
        server_configs,
        payload.session_id.clone(),
        payload.user_id,
        payload.auth_context,
        payload.context,
    )
    .await
    {
        Ok(k) => k,
        Err(e) => {
            return (
                StatusCode::FAILED_DEPENDENCY,
                Json(AgentResponse {
                    ok: false,
                    session_id: Some(payload.session_id),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(ApiError {
                        code: "MCP_CONNECTION_ERROR".to_string(),
                        message: format!("Generate kernel failed: {e}"),
                        recoverable: true,
                    }),
                }),
            );
        }
    };
    let val = match kernel.run(payload.goal).await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AgentResponse {
                    ok: false,
                    session_id: Some(kernel.state.session_id.clone()),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(ApiError {
                        code: "AGENT_RUN_ERROR".to_string(),
                        message: format!("Running agent failed: {e}"),
                        recoverable: true,
                    }),
                }),
            );
        }
    };
    if matches!(kernel.state.status, crate::ExecutionStatus::Waiting(_)) {
        state.sessions.lock().await.insert(
            kernel.state.session_id.clone(),
            WaitingSession::new(kernel.state.clone()),
        );
    }
    (
        StatusCode::ACCEPTED,
        Json(AgentResponse {
            ok: true,
            session_id: Some(kernel.state.session_id.clone()),
            status: val
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("completed")
                .to_string(),
            output: Some(val),
            error: None,
        }),
    )
}

pub async fn agent_resume_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AgentResumeRequest>,
) -> impl IntoResponse {
    let Some(action) = ApprovalAction::parse(&payload.approval.decision) else {
        return agent_error(
            StatusCode::BAD_REQUEST,
            Some(payload.session_id),
            "INVALID_APPROVAL_DECISION",
            "approval.decision must be approve, reject, or revise",
            true,
        );
    };

    let stored_state = {
        let mut sessions = state.sessions.lock().await;
        match sessions.get(&payload.session_id) {
            Some(waiting) if waiting.expired() => {
                sessions.remove(&payload.session_id);
                None
            }
            Some(waiting) => Some(waiting.state.clone()),
            None => None,
        }
    };
    let Some(mut execution_state) = stored_state else {
        return agent_error(
            StatusCode::NOT_FOUND,
            Some(payload.session_id),
            "SESSION_NOT_FOUND",
            "No waiting session found for session_id",
            false,
        );
    };

    if let Err(message) = validate_resume_request(&execution_state, &payload) {
        return agent_error(
            StatusCode::BAD_REQUEST,
            Some(payload.session_id),
            "INVALID_RESUME_REQUEST",
            &message,
            true,
        );
    }

    state.sessions.lock().await.remove(&payload.session_id);

    execution_state.context.write_obs(
        &payload.approval.step_id,
        &json!({
            "approval": payload.approval,
            "trusted_runtime": true
        }),
    );

    match action {
        ApprovalAction::Approve => {
            execution_state.context.user_confirmed = true;
            execution_state.current_step += 1;
        }
        ApprovalAction::Reject => {
            execution_state.status = ExecutionStatus::Failed("User rejected approval gate".into());
            return (
                StatusCode::OK,
                Json(AgentResponse {
                    ok: true,
                    session_id: Some(execution_state.session_id),
                    status: "rejected".to_string(),
                    output: Some(json!({
                        "ok": true,
                        "status": "rejected",
                        "comment": execution_state.context.last_obs()
                    })),
                    error: None,
                }),
            );
        }
        ApprovalAction::Revise => {
            execution_state.status =
                ExecutionStatus::RePlanning("User requested revision".to_string());
        }
    }

    let mut kernel = match AgentKernel::from_state(state.clients.clone(), execution_state).await {
        Ok(kernel) => kernel,
        Err(error) => {
            return (
                StatusCode::FAILED_DEPENDENCY,
                Json(AgentResponse {
                    ok: false,
                    session_id: Some(payload.session_id),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(ApiError {
                        code: "MCP_CONNECTION_ERROR".to_string(),
                        message: error.to_string(),
                        recoverable: true,
                    }),
                }),
            );
        }
    };

    let value = match match action {
        ApprovalAction::Approve => kernel.continue_existing().await,
        ApprovalAction::Revise => {
            kernel
                .replan_existing("User requested revision".to_string())
                .await
        }
        ApprovalAction::Reject => unreachable!("reject returns before kernel resume"),
    } {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AgentResponse {
                    ok: false,
                    session_id: Some(kernel.state.session_id.clone()),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(ApiError {
                        code: "AGENT_RESUME_ERROR".to_string(),
                        message: error.to_string(),
                        recoverable: true,
                    }),
                }),
            );
        }
    };

    if matches!(kernel.state.status, crate::ExecutionStatus::Waiting(_)) {
        state.sessions.lock().await.insert(
            kernel.state.session_id.clone(),
            WaitingSession::new(kernel.state.clone()),
        );
    }

    (
        StatusCode::ACCEPTED,
        Json(AgentResponse {
            ok: true,
            session_id: Some(kernel.state.session_id.clone()),
            status: value
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("completed")
                .to_string(),
            output: Some(value),
            error: None,
        }),
    )
}

fn agent_error(
    status_code: StatusCode,
    session_id: Option<String>,
    code: &str,
    message: &str,
    recoverable: bool,
) -> (StatusCode, Json<AgentResponse>) {
    (
        status_code,
        Json(AgentResponse {
            ok: false,
            session_id,
            status: "failed".to_string(),
            output: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                recoverable,
            }),
        }),
    )
}

fn validate_resume_request(
    execution_state: &ExecutionState,
    payload: &AgentResumeRequest,
) -> Result<(), String> {
    if !matches!(execution_state.status, ExecutionStatus::Waiting(_)) {
        return Err("Session is not waiting for approval".to_string());
    }

    let expected_step = execution_state
        .plan
        .get(execution_state.current_step)
        .map(|step| step.id.as_str())
        .ok_or_else(|| "Waiting session has no current step".to_string())?;
    if expected_step != payload.approval.step_id {
        return Err(format!(
            "Approval step_id mismatch: expected {expected_step}, got {}",
            payload.approval.step_id
        ));
    }

    if let Some(expected_user_id) = &execution_state.context.user_id {
        let actual_user_id = payload
            .user_id
            .as_ref()
            .or_else(|| payload.auth_context.as_ref().map(|auth| &auth.user_id));
        if actual_user_id != Some(expected_user_id) {
            return Err("Resume request does not match session owner".to_string());
        }
    }

    Ok(())
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

pub async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "status": "healthy",
        "service": "orchestrator"
    }))
}

pub async fn ready_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let registry = McpRegistry::build(&state.clients).await;
    let body = registry.ready_response();
    let status = if registry.ready() {
        StatusCode::OK
    } else {
        StatusCode::FAILED_DEPENDENCY
    };
    (status, Json(body))
}

pub async fn mcp_tools_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let registry = McpRegistry::build(&state.clients).await;
    Json(registry.tools_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentContext, PlanStep, StepActions};

    fn waiting_state() -> ExecutionState {
        ExecutionState {
            session_id: "session-1".to_string(),
            status: ExecutionStatus::Waiting("confirm".to_string()),
            current_step: 0,
            plan: vec![PlanStep {
                id: "step 1".to_string(),
                action: StepActions::HumanApproval,
                step_goal: Some("confirm roadmap".to_string()),
                dependencies: Vec::new(),
                final_output: None,
            }],
            context: AgentContext {
                session_id: "session-1".to_string(),
                user_id: Some("user-1".to_string()),
                ..AgentContext::default()
            },
            resolver: Vec::new(),
        }
    }

    #[test]
    fn resume_validation_rejects_wrong_step() {
        let payload = AgentResumeRequest {
            session_id: "session-1".to_string(),
            user_id: Some("user-1".to_string()),
            auth_context: None,
            approval: ApprovalDecision {
                step_id: "step 2".to_string(),
                decision: "approve".to_string(),
                comment: None,
            },
        };

        let error = validate_resume_request(&waiting_state(), &payload).unwrap_err();
        assert!(error.contains("step_id mismatch"));
    }

    #[test]
    fn resume_validation_rejects_wrong_owner() {
        let payload = AgentResumeRequest {
            session_id: "session-1".to_string(),
            user_id: Some("user-2".to_string()),
            auth_context: None,
            approval: ApprovalDecision {
                step_id: "step 1".to_string(),
                decision: "approve".to_string(),
                comment: None,
            },
        };

        let error = validate_resume_request(&waiting_state(), &payload).unwrap_err();
        assert!(error.contains("session owner"));
    }
}
