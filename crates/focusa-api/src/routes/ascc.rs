//! ASCC (Autonomous Session Context Continuity) routes.
//!
//! GET  /v1/ascc/frame/:frame_id   — get ASCC data for a frame
//! POST /v1/ascc/update-delta      — update focus state delta

use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::types::{
    AsccSections, CheckpointRecord, EventLogEntry, FocusStateDelta, FocusaEvent, FocusaState,
    SignalOrigin,
};
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
    let retry_safe = !matches!(
        failure_class,
        "validation_rejected" | "not_found" | "frame_unavailable"
    );
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
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
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
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

fn persist_ascc_checkpoint(
    data_dir: &str,
    focusa: &mut FocusaState,
    frame_id: Uuid,
) -> Result<(), String> {
    let Some(frame_index) = focusa
        .focus_stack
        .frames
        .iter()
        .position(|frame| frame.id == frame_id)
    else {
        return Err(format!(
            "ASCC frame {frame_id} vanished after reducer write"
        ));
    };

    let sections = AsccSections::from(&focusa.focus_stack.frames[frame_index].focus_state);
    if sections.is_empty() {
        return Ok(());
    }

    let turn_id = focusa
        .active_turn
        .as_ref()
        .map(|turn| turn.turn_id.clone())
        .unwrap_or_else(|| format!("api-{}", focusa.version));
    focusa.focus_stack.frames[frame_index].ascc_checkpoint_id = Some(format!("ascc:{frame_id}"));
    let frame = focusa.focus_stack.frames[frame_index].clone();
    let checkpoint = match focusa_core::ascc::load_checkpoint(data_dir, frame_id)
        .map_err(|error| format!("failed to load ASCC checkpoint: {error}"))?
    {
        Some(mut checkpoint) => {
            checkpoint.update_from_frame(&frame, &turn_id);
            checkpoint
        }
        None => CheckpointRecord::from_frame(&frame, &turn_id),
    };
    focusa_core::ascc::save_checkpoint(data_dir, &checkpoint)
        .map_err(|error| format!("failed to persist ASCC checkpoint: {error}"))
}

/// GET /v1/ascc/frame/:frame_id — get ASCC data for a frame.
///
/// Returns checkpoints and focus state for the specified frame.
async fn get_frame_ascc(
    _scope: ScopeContext,
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
    _scope: ScopeContext,
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

    let event = FocusaEvent::FocusStateUpdated {
        frame_id,
        delta: body.delta,
    };
    let _guard = state.write_serial_lock.lock().await;
    let current = { state.focusa.read().await.clone() };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        ascc_failure(
            StatusCode::BAD_REQUEST,
            error.to_string(),
            "reducer_rejected",
            "ASCC delta was rejected by the canonical reducer.",
            "Verify frame_id exists and retry with a valid FocusStateDelta.",
            "Likely stale frame_id, invalid delta shape, or conflicting frame lifecycle state.",
            vec!["focusa_active_object_resolve", "focusa_tool_doctor"],
        )
    })?;
    let mut new_state = result.new_state;
    persist_ascc_checkpoint(&state.config.data_dir, &mut new_state, frame_id).map_err(|error| {
        ascc_failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            error,
            "persistence_failed",
            "ASCC delta reduced successfully but checkpoint file persistence failed.",
            "Check daemon data_dir permissions and retry once checkpoint storage is writable.",
            "Likely data_dir permissions, missing parent directory, or invalid checkpoint JSON on disk.",
            vec!["focusa_tool_doctor", "focusa_workpoint_resume"],
        )
    })?;
    let entry = EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id: Some("api:ascc_update_delta".to_string()),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    let _ = state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(json!({"status": "accepted", "canonical": true})))
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
