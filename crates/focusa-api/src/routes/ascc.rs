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

/// GET /v1/ascc/frame/:frame_id — get ASCC data for a frame.
///
/// Returns checkpoints and focus state for the specified frame.
async fn get_frame_ascc(
    State(state): State<Arc<AppState>>,
    Path(frame_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let focusa = state.focusa.read().await;

    // Find the frame.
    let frame = focusa
        .focus_stack
        .frames
        .iter()
        .find(|f| f.id == frame_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Frame not found"})),
            )
        })?;

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
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get frame ID from body or use active frame.
    let frame_id = match body.frame_id {
        Some(fid) => fid,
        None => {
            let focusa = state.focusa.read().await;
            focusa
                .focus_stack
                .active_id
                .ok_or(StatusCode::BAD_REQUEST)?
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ascc/frame/{frame_id}", get(get_frame_ascc))
        .route("/v1/ascc/update-delta", post(update_delta))
}
