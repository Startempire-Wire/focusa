//! Session routes.
//!
//! GET  /v1/status        — daemon/session status (summary)
//! GET  /v1/state/dump    — full cognitive state (debug)
//! POST /v1/session/start — start a new session
//! POST /v1/session/close — close current session

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::Action;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "session": focusa.session,
        "stack_depth": focusa.focus_stack.frames.len(),
        "active_frame_id": focusa.focus_stack.active_id,
        "version": focusa.version,
    }))
}

/// Full cognitive state dump (debug).
async fn state_dump(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(serde_json::to_value(&*focusa).unwrap_or(json!({"error": "serialization failed"})))
}

#[derive(Deserialize)]
struct StartSessionBody {
    adapter_id: Option<String>,
    workspace_id: Option<String>,
    instance_id: Option<String>,
}

async fn start_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::StartSession {
            adapter_id: body.adapter_id,
            workspace_id: body.workspace_id,
            instance_id: body.instance_id.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct CloseSessionBody {
    #[serde(default = "default_reason")]
    reason: String,
    instance_id: Option<String>,
}

fn default_reason() -> String {
    "user_requested".into()
}

async fn close_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CloseSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::CloseSession {
            reason: body.reason,
            instance_id: body.instance_id.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/state/dump", get(state_dump))
        .route("/v1/session/start", post(start_session))
        .route("/v1/session/close", post(close_session))
}
