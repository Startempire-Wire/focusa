//! Focus stack routes.
//!
//! GET  /v1/focus/stack   — read current stack
//! POST /v1/focus/push    — push a new frame
//! POST /v1/focus/pop     — pop (complete) active frame
//! POST /v1/focus/set-active — switch active frame
//! POST /v1/focus/update  — update focus state delta (ASCC)
//! GET  /v1/focusa/enabled — get focusa toggle state (Pi-session-local)
//! PATCH /v1/focusa/enabled — set focusa toggle state

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post, patch},
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
/// Slot-specific stricter rules for result-adjacent slots.
fn validate_slot(value: &str, max_chars: usize, slot_kind: &str) -> bool {
    if value.is_empty() || value.len() > max_chars {
        return false;
    }
    let lower = value.to_lowercase();

    // Slot-specific: reject verbose process narration in result/question slots
    if matches!(slot_kind, "recent_results" | "notes" | "open_questions") && value.len() > 180 {
        return false; // verbose entries don't belong in result/question slots
    }

    // Verbose process narration patterns — NEVER valid in any slot
    if lower.contains("root cause") || lower.contains("bypass") || lower.contains("pollut")
        || lower.contains("investigation") || lower.contains("pattern ") && lower.contains("match")
        || lower.contains("verbose") && lower.len() < 100
        || lower.contains("i was able to") || lower.contains("it appears that")
        || lower.contains("this confirms") || lower.contains("as suspected")
        || lower.contains("confirmed in the running system")
        || lower.contains("still the old version")
        || lower.contains("three bugs") || lower.contains("daemon restarted")
        || lower.contains("binary confirmed")
    {
        return false;
    }

    // Table/structured markup — investigation noise, not results
    if value.contains("| ") && value.contains(" | ") && value.contains(" : ") {
        return false;
    }

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
        || value.starts_with("Modified:") || value.starts_with("Added:") || value.starts_with("Deleted:")
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

/// Slot capacity caps per §AsccSections.
fn slot_cap(slot_kind: &str) -> usize {
    match slot_kind {
        "decisions" | "next_steps" | "recent_results" => 10,
        "open_questions" | "notes" => 20,
        "constraints" => 15,
        "failures" => 10,
        _ => 50,
    }
}

async fn update_delta(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateDeltaBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // §AsccSections: validate ALL slots at API boundary before any write.
    let delta = &body.delta;
    let frame = {
        let focusa = state.focusa.read().await;
        focusa.focus_stack.frames.iter().find(|f| Some(f.id) == focusa.focus_stack.active_id).cloned()
    };

    // Per-slot validation with kind + capacity cap check
    if let Some(ref intent) = delta.intent
        && !validate_slot(intent, 500, "intent") {
            return Ok(Json(json!({"status": "rejected", "reason": "intent: validation failed"})));
        }
    if let Some(ref cs) = delta.current_state
        && !validate_slot(cs, 300, "current_state") {
            return Ok(Json(json!({"status": "rejected", "reason": "current_state: validation failed"})));
        }
    for (kind, values, max_chars) in [
        ("decisions", &delta.decisions, 160),
        ("constraints", &delta.constraints, 200),
        ("failures", &delta.failures, 300),
        ("next_steps", &delta.next_steps, 160),
        ("recent_results", &delta.recent_results, 300),
        ("notes", &delta.notes, 200),
        ("open_questions", &delta.open_questions, 200),
    ] {
        if let Some(vals) = values {
            if let Some(ref f) = frame {
                let current_len = match kind {
                    "decisions" => f.focus_state.decisions.len(),
                    "constraints" => f.focus_state.constraints.len(),
                    "failures" => f.focus_state.failures.len(),
                    "next_steps" => f.focus_state.next_steps.len(),
                    "recent_results" => f.focus_state.recent_results.len(),
                    "notes" => f.focus_state.notes.len(),
                    "open_questions" => f.focus_state.open_questions.len(),
                    _ => 0,
                };
                if current_len >= slot_cap(kind) {
                    return Ok(Json(json!({"status": "rejected", "reason": format!("{}: at capacity ({})", kind, current_len)})));
                }
            }
            if vals.iter().any(|s| !validate_slot(s, max_chars, kind)) {
                return Ok(Json(json!({"status": "rejected", "reason": format!("{}: validation failed", kind)})));
            }
        }
    }
    // Validate artifacts
    if let Some(ref artifacts) = delta.artifacts {
        for a in artifacts {
            if a.label.is_empty() || a.label.len() > 100 {
                return Ok(Json(json!({"status": "rejected", "reason": "artifacts: label validation failed"})));
            }
        }
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

// ═══════════════════════════════════════════════════════════════════════════════
// PI-TOGGLE ENDPOINT — SPEC-33.5 disk persistence
// ═══════════════════════════════════════════════════════════════════════════════

/// Path to the Pi toggle state file.
fn pi_enabled_path(config: &focusa_core::types::FocusaConfig) -> std::path::PathBuf {
    let expanded = if config.data_dir.starts_with("~") {
        std::env::var("HOME").unwrap_or_else(|_| "~".to_string()) + &config.data_dir[1..]
    } else {
        config.data_dir.clone()
    };
    std::path::PathBuf::from(expanded).join("pi-enabled")
}

/// GET /v1/focusa/enabled — read current toggle state.
async fn get_enabled(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let path = pi_enabled_path(&state.config);
    let enabled = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().strip_prefix("enabled=").map(|v| v == "1"))
            .unwrap_or(true) // default: enabled
    } else {
        true
    };
    Json(json!({"enabled": enabled}))
}


#[derive(Deserialize)]
struct SetEnabledBody {
    enabled: bool,
}

/// PATCH /v1/focusa/enabled — set toggle state.
async fn set_enabled(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetEnabledBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = pi_enabled_path(&state.config);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    let content = format!("enabled={}", if body.enabled { "1" } else { "0" });
    std::fs::write(&path, content)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::info!(path = path.display().to_string(), enabled = body.enabled, "Pi focusa toggle updated");
    Ok(Json(json!({"status": "updated", "enabled": body.enabled})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/stack", get(get_stack))
        .route("/v1/focus/push", post(push_frame))
        .route("/v1/focus/pop", post(pop_frame))
        .route("/v1/focus/set-active", post(set_active))
        .route("/v1/focus/update", post(update_delta))
        .route("/v1/focusa/enabled", get(get_enabled))
        .route("/v1/focusa/enabled", patch(set_enabled))
}
