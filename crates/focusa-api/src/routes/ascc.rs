//! ASCC (Autonomous Session Context Continuity) routes.
//!
//! GET  /v1/ascc/frame/:frame_id   — get ASCC data for a frame
//! POST /v1/ascc/update-delta      — update focus state delta

use crate::server::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, FocusStateDelta};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

type AppResult<T = Json<serde_json::Value>> = Result<T, (StatusCode, Json<serde_json::Value>)>;

fn ascc_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": true, "posture": "safe_retry", "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn ascc_frame_not_found(frame_id: Uuid, active: bool) -> (StatusCode, Json<serde_json::Value>) {
    let label = if active {
        "Active frame not found"
    } else {
        "Frame not found"
    };
    ascc_failure(
        StatusCode::NOT_FOUND,
        label,
        "frame_unavailable",
        format!("ASCC frame {frame_id} is not visible in the focus stack"),
        "Refresh /v1/focus/stack or create/resume a canonical Focus frame before reading ASCC.",
        "Likely stale frame_id, read-model lag, wrong project/session scope, or active_id points to a missing frame.",
        vec![
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_tool_doctor",
        ],
    )
}

fn ascc_no_active_frame() -> (StatusCode, Json<serde_json::Value>) {
    ascc_failure(
        StatusCode::NOT_FOUND,
        "No active frame",
        "frame_unavailable",
        "No active Focus frame is available for ASCC state.",
        "Create or resume a canonical Focus/Workpoint frame before reading /v1/ascc/state.",
        "Likely fresh daemon state, unsafe scope suppression, or missing session initialization.",
        vec![
            "focusa_workpoint_checkpoint",
            "focusa_workpoint_resume",
            "focusa_tool_doctor",
        ],
    )
}

fn ascc_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    ascc_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to dispatch ASCC delta: {error}"),
        "daemon_unavailable",
        "ASCC delta could not be dispatched to the daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec![
            "focusa_tool_doctor",
            "focusa_work_loop_status",
            "focusa_workpoint_resume",
        ],
    )
}

/// GET /v1/ascc/frame/:frame_id — get ASCC data for a frame.
///
/// Returns checkpoints and focus state for the specified frame.
async fn get_frame_ascc(
    State(state): State<Arc<AppState>>,
    Path(frame_id): Path<Uuid>,
) -> AppResult {
    let focusa = state.focusa.read().await;

    // Find the frame.
    let frame = focusa
        .focus_stack
        .frames
        .iter()
        .find(|f| f.id == frame_id)
        .ok_or_else(|| ascc_frame_not_found(frame_id, false))?;

    Ok(Json(json!({
        "frame_id": frame_id,
        "title": frame.title,
        "goal": frame.goal,
        "focus_state": frame.focus_state,
        "ascc_checkpoint_id": frame.ascc_checkpoint_id,
        "stats": frame.stats,
        "status": frame.status,
    })))
}

/// POST /v1/ascc/update-delta — update focus state delta.
///
/// Per spec: adapters provide transcript summaries to ASCC.
#[derive(Deserialize)]
struct UpdateDeltaBody {
    #[serde(default)]
    frame_id: Option<Uuid>,
    delta: FocusStateDelta,
}

async fn update_delta(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateDeltaBody>,
) -> AppResult {
    // Get frame ID from body or use active frame.
    let frame_id = match body.frame_id {
        Some(fid) => fid,
        None => {
            let focusa = state.focusa.read().await;
            focusa
                .focus_stack
                .active_id
                .ok_or_else(ascc_no_active_frame)?
        }
    };

    state
        .command_tx
        .send(Action::UpdateCheckpointDelta {
            frame_id,
            turn_id: Uuid::now_v7().to_string(),
            delta: body.delta,
        })
        .await
        .map_err(ascc_dispatch_failed)?;

    Ok(Json(json!({"status": "accepted"})))
}

/// GET /v1/ascc/state — get ASCC state for active frame.
///
/// Per docs/G1-07-ascc.md: Global state endpoint MUST exist.
async fn get_ascc_state(State(state): State<Arc<AppState>>) -> AppResult {
    let focusa = state.focusa.read().await;

    let frame_id = focusa
        .focus_stack
        .active_id
        .ok_or_else(ascc_no_active_frame)?;

    let frame = focusa
        .focus_stack
        .frames
        .iter()
        .find(|f| f.id == frame_id)
        .ok_or_else(|| ascc_frame_not_found(frame_id, true))?;

    // Build ASCC sections from focus state.
    let ascc = focusa_core::types::AsccSections::from(&frame.focus_state);

    Ok(Json(json!({
        "frame_id": frame_id,
        "active": true,
        "title": frame.title,
        "goal": frame.goal,
        "ascc": ascc,
        "focus_state": frame.focus_state,
        "ascc_checkpoint_id": frame.ascc_checkpoint_id,
        "updated_at": frame.updated_at,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ascc/state", get(get_ascc_state))
        .route("/v1/ascc/frame/{frame_id}", get(get_frame_ascc))
        .route("/v1/ascc/update-delta", post(update_delta))
}
