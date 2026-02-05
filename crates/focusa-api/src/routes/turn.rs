//! Turn lifecycle routes — Mode A adapter integration.
//!
//! POST /v1/turn/start — Begin a new turn
//! POST /v1/turn/append — Append streaming chunk (optional)
//! POST /v1/turn/complete — End turn with assistant output
//! POST /v1/prompt/assemble — Get Focusa-enhanced prompt
//!
//! Source: docs/G1-detail-04-proxy-adapter.md

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use focusa_core::expression::engine::assemble;
use focusa_core::memory::procedural;
use focusa_core::types::*;
use serde_json::{json, Value};
use std::sync::Arc;

/// POST /v1/turn/start
///
/// Adapter calls this when user input is received.
/// Daemon tracks the active turn.
async fn turn_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TurnStart>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        turn_id = %req.turn_id,
        harness = %req.harness_name,
        "Turn started"
    );

    // Store active turn in state (for correlation).
    {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn = Some(ActiveTurn {
            turn_id: req.turn_id.clone(),
            adapter_id: req.adapter_id,
            harness_name: req.harness_name,
            started_at: req.timestamp,
            raw_user_input: None,
            assembled_prompt: None,
        });
    }

    Ok(Json(json!({
        "status": "accepted",
        "turn_id": req.turn_id
    })))
}

/// POST /v1/prompt/assemble
///
/// Adapter calls this to get the Focusa-enhanced prompt.
/// Returns assembled prompt with Focus State, rules, handles injected.
async fn prompt_assemble(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PromptAssembleRequest>,
) -> Result<Json<PromptAssembleResponse>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;

    // Get active frame's focus state.
    let focus_state = focusa
        .focus_stack
        .active_id
        .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
        .map(|f| &f.focus_state)
        .cloned()
        .unwrap_or_default();

    // Select procedural rules.
    let rules = procedural::select_for_prompt(
        &focusa.memory,
        focusa.focus_stack.active_id,
        5,
    );
    let rules_owned: Vec<RuleRecord> = rules.into_iter().cloned().collect();

    // Collect artifact handles.
    let session_id = focusa.session.as_ref().map(|s| s.session_id);
    let handles_owned: Vec<HandleRef> = focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned)
        .cloned()
        .collect();

    // Assemble prompt.
    let assembly = assemble(
        &focus_state,
        None,
        &rules_owned,
        &handles_owned,
        &req.raw_user_input,
        &state.config,
    );

    // Estimate token counts (rough: 4 chars per token).
    let estimate_tokens = |s: &str| (s.len() / 4) as u32;
    let user_tokens = estimate_tokens(&req.raw_user_input);

    let context_stats = ContextStats {
        estimated_tokens: assembly.token_estimate,
        focus_state_tokens: 0, // Not tracked individually in MVP
        rules_tokens: 0,
        handles_tokens: (assembly.handles_used.len() * 50) as u32, // Estimate
        user_input_tokens: user_tokens,
    };

    // Update active turn with the assembled prompt.
    drop(focusa);
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn
            && turn.turn_id == req.turn_id {
                turn.raw_user_input = Some(req.raw_user_input.clone());
                turn.assembled_prompt = Some(assembly.content.clone());
            }
    }

    // Return as messages array (chat format) or plain string based on harness_context hint.
    let output = if req.harness_context.as_deref() == Some("plain") {
        AssembledPromptOutput::Plain(assembly.content)
    } else {
        // Default: chat messages format.
        AssembledPromptOutput::Messages(vec![
            ChatMessage {
                role: "system".into(),
                content: assembly.content,
            },
            ChatMessage {
                role: "user".into(),
                content: req.raw_user_input,
            },
        ])
    };

    Ok(Json(PromptAssembleResponse {
        assembled_prompt: output,
        handles_used: handles_owned,
        context_stats,
    }))
}

/// POST /v1/turn/append — streaming chunk (optional).
///
/// For adapters that support streaming, append chunks during turn.
async fn turn_append(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TurnAppend>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::trace!(turn_id = %req.turn_id, chunk_len = req.chunk.len(), "Turn chunk appended");

    // Append to active turn's assembled_prompt (accumulating response).
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn
            && turn.turn_id == req.turn_id {
                let existing = turn.assembled_prompt.take().unwrap_or_default();
                turn.assembled_prompt = Some(format!("{}{}", existing, req.chunk));
            }
    }

    Ok(Json(json!({"status": "accepted"})))
}

/// Streaming append request.
#[derive(Debug, Clone, serde::Deserialize)]
struct TurnAppend {
    turn_id: String,
    chunk: String,
}

/// POST /v1/turn/complete
///
/// Adapter calls this when the turn ends.
/// Daemon records the assistant output, emits events.
async fn turn_complete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TurnComplete>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        turn_id = %req.turn_id,
        output_len = req.assistant_output.len(),
        artifacts = req.artifacts.len(),
        errors = req.errors.len(),
        "Turn completed"
    );

    // Get active turn and clear it.
    let active_turn = {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn.take()
    };

    // Validate turn_id matches.
    if let Some(turn) = &active_turn
        && turn.turn_id != req.turn_id {
            tracing::warn!(
                expected = %turn.turn_id,
                got = %req.turn_id,
                "Turn ID mismatch"
            );
        }

    // Update frame stats if we have an active frame.
    let frame_id = {
        let focusa = state.focusa.read().await;
        focusa.focus_stack.active_id
    };

    if let Some(fid) = frame_id {
        // Emit turn completion event.
        let delta = FocusStateDelta {
            current_state: Some(format!(
                "Turn {} completed: {} chars output",
                req.turn_id,
                req.assistant_output.len()
            )),
            ..Default::default()
        };

        let _ = state.command_tx.send(Action::UpdateCheckpointDelta {
            frame_id: fid,
            turn_id: req.turn_id.clone(),
            delta,
        }).await;
    }

    // Handle errors.
    for err in &req.errors {
        tracing::error!(turn_id = %req.turn_id, error = %err, "Turn error");
    }

    Ok(Json(json!({
        "status": "accepted",
        "turn_id": req.turn_id
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/turn/start", post(turn_start))
        .route("/v1/turn/append", post(turn_append))
        .route("/v1/turn/complete", post(turn_complete))
        .route("/v1/prompt/assemble", post(prompt_assemble))
}
