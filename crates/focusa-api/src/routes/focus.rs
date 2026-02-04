//! Focus stack routes.
//!
//! GET  /v1/focus/stack   — read current stack
//! POST /v1/focus/push    — push a new frame
//! POST /v1/focus/pop     — pop (complete) active frame
//! POST /v1/focus/set-active — switch active frame

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::{Action, CompletionReason};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn get_stack(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "stack": focusa.focus_stack,
        "active_frame_id": focusa.focus_stack.active_id,
    }))
}

#[derive(Deserialize)]
struct PushFrameBody {
    title: String,
    goal: String,
    beads_issue_id: String,
    #[serde(default)]
    constraints: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

async fn push_frame(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PushFrameBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::PushFrame {
            title: body.title,
            goal: body.goal,
            beads_issue_id: body.beads_issue_id,
            constraints: body.constraints,
            tags: body.tags,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct PopFrameBody {
    #[serde(default = "default_completion_reason")]
    completion_reason: CompletionReason,
}

fn default_completion_reason() -> CompletionReason {
    CompletionReason::GoalAchieved
}

async fn pop_frame(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PopFrameBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::PopFrame {
            completion_reason: body.completion_reason,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct SetActiveBody {
    frame_id: uuid::Uuid,
}

async fn set_active(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetActiveBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::SetActiveFrame {
            frame_id: body.frame_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/stack", get(get_stack))
        .route("/v1/focus/push", post(push_frame))
        .route("/v1/focus/pop", post(pop_frame))
        .route("/v1/focus/set-active", post(set_active))
}
