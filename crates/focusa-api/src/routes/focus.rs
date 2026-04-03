//! Focus stack routes.
//!
//! GET  /v1/focus/stack   — read current stack
//! POST /v1/focus/push    — push a new frame
//! POST /v1/focus/pop     — pop (complete) active frame
//! POST /v1/focus/set-active — switch active frame
//! POST /v1/focus/update  — update focus state delta (ASCC)

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, CompletionReason, FocusStateDelta};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

async fn get_stack(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "stack": focusa.focus_stack,
        "active_frame_id": focusa.focus_stack.active_id,
    }))
}

#[derive(Deserialize)]
struct PushFrameBody {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    beads_issue_id: Option<String>,
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
            title: body.title.unwrap_or_default(),
            goal: body.goal.unwrap_or_default(),
            beads_issue_id: body.beads_issue_id.unwrap_or_default(),
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

/// POST /v1/focus/update — update focus state delta (ASCC).
///
/// Per spec: adapters provide transcript summaries to ASCC.
/// §AsccSections §Validation: validates ALL slots at API boundary before any write.
#[derive(Deserialize)]
struct UpdateDeltaBody {
    delta: FocusStateDelta,
}

/// Validate a single slot value. Rejects verbose output, task patterns,
/// self-reference, markdown noise — same rules as tools.ts validateSlot.
fn validate_slot(value: &str, max_chars: usize) -> bool {
    if value.is_empty() || value.len() > max_chars {
        return false;
    }
    let lower = value.to_lowercase();
    // Task patterns
    if lower.contains("implement ") || lower.contains(" add ") || lower.contains("create ")
        || lower.contains("update ") || lower.contains("remove ") || lower.contains("fix all")
        || lower.contains("next:") || lower.contains("signal:")
    {
        return false;
    }
    // Self-reference
    if lower.contains("i think") || lower.contains("i tried") || lower.contains("i'm working")
        || lower.contains("i was") || lower.contains("in this session") || lower.contains("while i was")
        || lower.contains("my fs.") || lower.contains("my fix") || lower.contains("let me")
        || lower.contains("i need to") || lower.contains("i will") || lower.contains("i'll need")
    {
        return false;
    }
    // Markdown / noise patterns
    if value.contains("**") || value.contains("\u{2705}") || value.contains("\u{274C}")
        || value.contains("- [ ]") || value.contains("---") || value.contains("```")
        || value.contains("|") || value.starts_with("2.") || value.starts_with("3.")
        || value.starts_with("- ") || lower.contains("spec-compliant") || lower.contains("matches")
        || lower.contains("exactly") || lower.contains("fixme")
    {
        return false;
    }
    // Verbose continuation
    if lower.contains("now") && lower.contains("need to") {
        return false;
    }
    if lower.contains("continue") && value.len() > 80 {
        return false;
    }
    true
}

async fn update_delta(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateDeltaBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // §AsccSections: validate ALL slots at API boundary before any write.
    let delta = &body.delta;
    if delta.intent.as_ref().is_some_and(|v| !validate_slot(v, 500)) {
        return Ok(Json(json!({"status": "rejected", "reason": "intent: validation failed"})));
    }
    if delta.current_state.as_ref().is_some_and(|v| !validate_slot(v, 300)) {
        return Ok(Json(json!({"status": "rejected", "reason": "current_state: validation failed"})));
    }
    if delta.decisions.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 160))) {
        return Ok(Json(json!({"status": "rejected", "reason": "decisions: validation failed"})));
    }
    if delta.constraints.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 200))) {
        return Ok(Json(json!({"status": "rejected", "reason": "constraints: validation failed"})));
    }
    if delta.failures.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 300))) {
        return Ok(Json(json!({"status": "rejected", "reason": "failures: validation failed"})));
    }
    if delta.open_questions.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 200))) {
        return Ok(Json(json!({"status": "rejected", "reason": "open_questions: validation failed"})));
    }
    if delta.next_steps.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 160))) {
        return Ok(Json(json!({"status": "rejected", "reason": "next_steps: validation failed"})));
    }
    if delta.recent_results.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 300))) {
        return Ok(Json(json!({"status": "rejected", "reason": "recent_results: validation failed"})));
    }
    if delta.notes.as_ref().is_some_and(|v| v.iter().any(|s| !validate_slot(s, 200))) {
        return Ok(Json(json!({"status": "rejected", "reason": "notes: validation failed"})));
    }

    // Get active frame ID.
    let frame_id = {
        let focusa = state.focusa.read().await;
        focusa.focus_stack.active_id
    };

    let Some(fid) = frame_id else {
        return Ok(Json(json!({"status": "no_active_frame"})));
    };

    state
        .command_tx
        .send(Action::UpdateCheckpointDelta {
            frame_id: fid,
            turn_id: Uuid::now_v7().to_string(),
            delta: body.delta,
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
        .route("/v1/focus/update", post(update_delta))
}
