//! Session routes.
//!
//! GET  /v1/status        — daemon/session status (summary)
//! GET  /v1/state/dump    — full cognitive state (debug)
//! POST /v1/session/start — start a new session
//! POST /v1/session/close — close current session

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::Action;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let (
        session,
        stack_depth,
        active_frame_id,
        version,
        active_frame_summary,
        prompt_stats,
        worker_status,
        telemetry,
    ) = {
        let focusa = state.focusa.read().await;

        let active_frame = focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid));

        let active_frame_summary = active_frame.map(|f| {
            json!({
                "id": f.id,
                "title": f.title,
                "goal": f.goal,
                "status": f.status,
                "updated_at": f.updated_at,
            })
        });

        let assembled_chars = focusa
            .active_turn
            .as_ref()
            .and_then(|t| t.assembled_prompt.as_ref())
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        let prompt_stats = json!({
            "last_assembled_chars": assembled_chars,
            "last_assembled_estimated_tokens": assembled_chars / 4,
            "active_turn_id": focusa.active_turn.as_ref().map(|t| t.turn_id.clone()),
        });

        let worker_status = json!({
            "queue_size_config": state.config.worker_queue_size,
            "job_timeout_ms": state.config.worker_job_timeout_ms,
            "enabled": true,
        });

        let telemetry = json!({
            "total_events": focusa.telemetry.total_events,
            "total_prompt_tokens": focusa.telemetry.total_prompt_tokens,
            "total_completion_tokens": focusa.telemetry.total_completion_tokens,
        });

        (
            focusa.session.clone(),
            focusa.focus_stack.frames.len(),
            focusa.focus_stack.active_id,
            focusa.version,
            active_frame_summary,
            prompt_stats,
            worker_status,
            telemetry,
        )
    };

    let last_event_ts = state.persistence.latest_event_timestamp().ok().flatten();
    let persisted_event_count = state.persistence.event_count().ok();

    Json(json!({
        "session": session,
        "stack_depth": stack_depth,
        "active_frame_id": active_frame_id,
        "active_frame": active_frame_summary,
        "worker_status": worker_status,
        "last_event_ts": last_event_ts,
        "prompt_stats": prompt_stats,
        "telemetry": telemetry,
        "persisted_event_count": persisted_event_count,
        "version": version,
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
            instance_id: body
                .instance_id
                .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
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
            instance_id: body
                .instance_id
                .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
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
