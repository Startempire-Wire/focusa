//! Attachment routes (Instance/Session ↔ Thread binding).
//!
//! POST /v1/attachments/attach
//! POST /v1/attachments/detach
//! GET  /v1/attachments/list

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, AttachmentRole};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct AttachBody {
    instance_id: String,
    session_id: String,
    thread_id: String,
    #[serde(default = "default_role")]
    role: AttachmentRole,
}

fn default_role() -> AttachmentRole {
    AttachmentRole::Active
}

async fn attach(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AttachBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance_id = Uuid::parse_str(&body.instance_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let session_id = Uuid::parse_str(&body.session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let thread_id = Uuid::parse_str(&body.thread_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .command_tx
        .send(Action::ThreadAttach {
            instance_id,
            session_id,
            thread_id,
            role: body.role,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct DetachBody {
    instance_id: String,
    session_id: String,
    thread_id: String,
    #[serde(default = "default_reason")]
    reason: String,
}

fn default_reason() -> String {
    "client_requested".to_string()
}

async fn detach(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DetachBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance_id = Uuid::parse_str(&body.instance_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let session_id = Uuid::parse_str(&body.session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let thread_id = Uuid::parse_str(&body.thread_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .command_tx
        .send(Action::ThreadDetach {
            instance_id,
            session_id,
            thread_id,
            reason: body.reason,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

async fn list(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({"attachments": focusa.attachments}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/attachments/attach", post(attach))
        .route("/v1/attachments/detach", post(detach))
        .route("/v1/attachments/list", get(list))
}
