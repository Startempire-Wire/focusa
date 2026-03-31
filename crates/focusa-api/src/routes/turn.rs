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
use focusa_core::memory::procedural;
use focusa_core::types::*;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /v1/turn/start
///
/// Adapter calls this when user input is received.
/// Daemon emits TurnStarted event for observability.
async fn turn_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TurnStart>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if turn already started (prevent recursion from magic shims).
    {
        let focusa = state.focusa.read().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == req.turn_id
        {
            tracing::debug!(turn_id = %req.turn_id, "Turn already started, skipping duplicate");
            return Ok(Json(json!({
                "status": "accepted",
                "turn_id": req.turn_id,
                "duplicate": true
            })));
        }
    }

    tracing::info!(
        turn_id = %req.turn_id,
        harness = %req.harness_name,
        adapter_id = %req.adapter_id,
        "Turn started"
    );

    // Emit TurnStarted event via command channel for persistence.
    let event = FocusaEvent::TurnStarted {
        turn_id: req.turn_id.clone(),
        harness_name: req.harness_name.clone(),
        adapter_id: req.adapter_id.clone(),
        raw_user_input: None, // Will be set when prompt_assemble is called
    };

    if let Err(e) = state.command_tx.send(Action::EmitEvent { event }).await {
        tracing::error!("Failed to emit TurnStarted event: {}", e);
    }

    // Also store in active_turn for correlation (for prompt_assemble to access).
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
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
    let project_id = focusa
        .focus_stack
        .active_id
        .and_then(|fid| focusa.focus_stack.frames.iter().find(|f| f.id == fid))
        .and_then(|frame| {
            frame
                .tags
                .iter()
                .find(|t| t.starts_with("project:"))
                .map(|t| t.trim_start_matches("project:").to_string())
        });

    let rules = procedural::select_for_prompt(
        &focusa.memory,
        focusa.focus_stack.active_id,
        project_id.as_deref(),
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

    // Build ASCC sections from FocusState (G1-07 §Prompt Serialization).
    let ascc = focusa_core::types::AsccSections::from(&focus_state);
    let ascc_ref = if ascc.is_empty() { None } else { Some(&ascc) };

    // Build parent context from stack (G1-detail-05, G1-detail-11 §Slot 4).
    let parents = focusa_core::expression::engine::build_parent_contexts(&focusa.focus_stack);

    // Get active frame title.
    let frame_title = focusa
        .focus_stack
        .active_id
        .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
        .map(|f| f.title.as_str())
        .unwrap_or(&focus_state.intent);

    // Extract constitution principles (docs/16 §2, §5).
    let (principles, safety) =
        focusa_core::expression::engine::extract_constitution(&focusa.constitution);

    // Assemble prompt with full context.
    let input = focusa_core::expression::engine::AssemblyInput {
        focus_state: &focus_state,
        frame_title,
        ascc: ascc_ref,
        parent_frames: &parents,
        rules: &rules_owned,
        handles: &handles_owned,
        user_input: &req.raw_user_input,
        directive: None,
        constitution_principles: &principles,
        safety_rules: &safety,
        config: &state.config,
        rehydrate_handles: None,
    };
    let assembly = focusa_core::expression::engine::assemble_from(input);

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
            && turn.turn_id == req.turn_id
        {
            turn.raw_user_input = Some(req.raw_user_input.clone());
            turn.assembled_prompt = Some(assembly.content.clone());
        }
    }

    // Return as messages array (chat format) or plain string based on format hint.
    let output = if req.format.as_deref() == Some("string") {
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

    Ok(Json(json!({
        // Canonical spec keys
        "assembled": output.clone(),
        "stats": context_stats.clone(),
        "handles_used": handles_owned,
        // Backward-compatible runtime keys
        "assembled_prompt": output,
        "context_stats": context_stats,
    })))
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
            && turn.turn_id == req.turn_id
        {
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
/// Daemon emits TurnCompleted event for observability.
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

    // Idempotency guard: repeated completion for the same turn_id must not double-apply.
    match state.persistence.turn_completed_exists(&req.turn_id) {
        Ok(true) => {
            tracing::debug!(turn_id = %req.turn_id, "Duplicate turn_complete ignored");
            return Ok(Json(json!({
                "status": "accepted",
                "turn_id": req.turn_id,
                "duplicate": true
            })));
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!(turn_id = %req.turn_id, error = %e, "Failed idempotency check; proceeding cautiously");
        }
    }

    // Get harness_name and raw_user_input from active turn if available.
    let (harness_name, raw_user_input) = {
        let focusa = state.focusa.read().await;
        let hn = focusa
            .active_turn
            .as_ref()
            .map(|t| t.harness_name.clone())
            .unwrap_or_default();
        let rui = focusa
            .active_turn
            .as_ref()
            .and_then(|t| t.raw_user_input.clone());
        (hn, rui)
    };

    // Emit TurnCompleted event via command channel for persistence.
    // The reducer will handle CLT recording, error signals, and active_turn clearing.
    let event = FocusaEvent::TurnCompleted {
        turn_id: req.turn_id.clone(),
        harness_name,
        raw_user_input: raw_user_input.or(req.raw_user_input),
        assistant_output: Some(req.assistant_output.clone()),
        artifacts_used: req.artifacts.clone(),
        errors: req.errors.clone(),
        prompt_tokens: None,
        completion_tokens: None,
    };

    if let Err(e) = state.command_tx.send(Action::EmitEvent { event }).await {
        tracing::error!("Failed to emit TurnCompleted event: {}", e);
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

#[cfg(test)]
mod tests {
    use crate::server::{AppState, build_router};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
    use focusa_core::types::{
        Action, EventLogEntry, FocusaConfig, FocusaEvent, FocusaState, SignalOrigin,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::{RwLock, broadcast, mpsc};
    use tower::ServiceExt;
    use uuid::Uuid;

    fn temp_config() -> FocusaConfig {
        let mut cfg = FocusaConfig::default();
        let dir = std::env::temp_dir().join(format!("focusa-api-test-{}", Uuid::now_v7()));
        cfg.data_dir = dir.to_string_lossy().to_string();
        cfg
    }

    async fn setup_app() -> (axum::Router, SqlitePersistence) {
        let cfg = temp_config();
        let persistence = SqlitePersistence::new(&cfg).expect("persistence");

        let (tx, mut rx) = mpsc::channel::<Action>(64);
        let (events_tx, _) = broadcast::channel::<String>(16);
        let focusa = Arc::new(RwLock::new(FocusaState::default()));

        let p = persistence.clone();
        tokio::spawn(async move {
            while let Some(action) = rx.recv().await {
                if let Action::EmitEvent { event } = action {
                    let entry = EventLogEntry {
                        id: Uuid::now_v7(),
                        timestamp: Utc::now(),
                        event,
                        correlation_id: None,
                        origin: SignalOrigin::Daemon,
                        machine_id: None,
                        instance_id: None,
                        session_id: None,
                        thread_id: None,
                        is_observation: false,
                    };
                    let _ = p.append_event(&entry);
                }
            }
        });

        let state = Arc::new(AppState {
            focusa,
            command_tx: tx,
            events_tx,
            event_broadcaster: crate::routes::sse::EventBroadcaster::new(),
            config: cfg,
            persistence: persistence.clone(),
            command_store: Arc::new(RwLock::new(HashMap::new())),
            token_store: Arc::new(RwLock::new(focusa_core::permissions::TokenStore::new())),
            started_at: Instant::now(),
        });

        (build_router(state), persistence)
    }

    #[tokio::test]
    async fn turn_complete_is_idempotent_by_turn_id() {
        let (app, persistence) = setup_app().await;
        let turn_id = format!("turn-{}", Uuid::now_v7());

        let start_req = Request::builder()
            .method("POST")
            .uri("/v1/turn/start")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "turn_id": turn_id,
                    "adapter_id": "spec-test",
                    "harness_name": "spec-test",
                    "timestamp": Utc::now(),
                })
                .to_string(),
            ))
            .expect("request");

        let start_resp = app
            .clone()
            .oneshot(start_req)
            .await
            .expect("start response");
        assert_eq!(start_resp.status(), StatusCode::OK);

        let complete_body = serde_json::json!({
            "turn_id": turn_id,
            "assistant_output": "done",
            "artifacts": [],
            "errors": [],
        })
        .to_string();

        let req1 = Request::builder()
            .method("POST")
            .uri("/v1/turn/complete")
            .header("content-type", "application/json")
            .body(Body::from(complete_body.clone()))
            .expect("request1");
        let resp1 = app.clone().oneshot(req1).await.expect("resp1");
        assert_eq!(resp1.status(), StatusCode::OK);

        // Allow async action consumer to persist first completion event.
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let req2 = Request::builder()
            .method("POST")
            .uri("/v1/turn/complete")
            .header("content-type", "application/json")
            .body(Body::from(complete_body))
            .expect("request2");
        let resp2 = app.clone().oneshot(req2).await.expect("resp2");
        assert_eq!(resp2.status(), StatusCode::OK);

        let body2 = axum::body::to_bytes(resp2.into_body(), usize::MAX)
            .await
            .expect("body2 bytes");
        let json2: serde_json::Value = serde_json::from_slice(&body2).expect("json2");
        assert_eq!(json2.get("duplicate").and_then(|v| v.as_bool()), Some(true));

        // Verify persistence-level dedupe signal.
        let exists = persistence
            .turn_completed_exists(&turn_id)
            .expect("turn_completed_exists");
        assert!(exists);

        let recent = persistence
            .events_since(None, None, 100)
            .expect("events_since");
        let completed_count = recent
            .iter()
            .filter(|e| matches!(e.event, FocusaEvent::TurnCompleted { .. }))
            .filter(|e| {
                if let FocusaEvent::TurnCompleted {
                    turn_id: ref tid, ..
                } = e.event
                {
                    tid == &turn_id
                } else {
                    false
                }
            })
            .count();
        assert_eq!(completed_count, 1);
    }
}
