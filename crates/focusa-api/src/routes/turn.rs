//! Turn lifecycle routes — Mode A adapter integration.
//!
//! POST /v1/turn/start — Begin a new turn
//! POST /v1/turn/append — Append streaming chunk (optional)
//! POST /v1/turn/complete — End turn with assistant output
//! POST /v1/prompt/assemble — Get Focusa-enhanced prompt
//!
//! Source: docs/G1-detail-04-proxy-adapter.md

use crate::routes::ontology;
use crate::routes::work_loop::{
    maybe_dispatch_continuous_turn_prompt, parse_work_loop_outcome_receipt,
};
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use chrono::Utc;
use focusa_core::expression::budget::{available_tokens, estimate_tokens};
use focusa_core::memory::procedural;
use focusa_core::reducer;
use focusa_core::types::*;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

const RECENT_COMPLETED_TURN_CAP: usize = 2048;
async fn recent_turn_completed(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
    turn_id: &str,
) -> bool {
    state
        .recent_completed_turns_by_scope
        .read()
        .await
        .get(scope)
        .is_some_and(|turns| turns.iter().any(|value| value == turn_id))
}

async fn remember_completed_turn(
    state: &AppState,
    scope: focusa_core::scoped_state::WorkstreamKey,
    turn_id: &str,
) {
    let mut scoped_turns = state.recent_completed_turns_by_scope.write().await;
    let turns = scoped_turns.entry(scope).or_insert_with(VecDeque::new);
    if turns.iter().any(|value| value == turn_id) {
        return;
    }
    turns.push_back(turn_id.to_string());
    while turns.len() > RECENT_COMPLETED_TURN_CAP {
        turns.pop_front();
    }
}

async fn materialize_turn_event(
    state: &AppState,
    event: FocusaEvent,
    correlation_id: &'static str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let _guard = state.write_serial_lock.lock().await;
    let current = { state.focusa.read().await.clone() };
    let result = reducer::reduce_with_meta(current, event, None, None, false).map_err(|error| {
        tracing::warn!(error = %error, correlation_id, "turn event rejected by reducer");
        (
            StatusCode::OK,
            Json(json!({
                "status": "rejected",
                "failure_class": "reducer_rejected",
                "reason": error.to_string(),
            })),
        )
    })?;

    let new_state = result.new_state;
    let temporal = focusa_core::temporal_clock::capture_operator_temporal_action_envelope();
    for emitted in result.emitted_events {
        let mut entry = EventLogEntry::with_temporal(
            emitted,
            SignalOrigin::Adapter,
            Some(correlation_id.to_string()),
            temporal.clone(),
        );
        entry.session_id = new_state.session.as_ref().map(|session| session.session_id);
        if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
            tracing::error!(error = %error, correlation_id, "failed to persist turn event");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "rejected",
                    "failure_class": "persistence_failed",
                    "reason": error.to_string(),
                })),
            ));
        } else if let Ok(serialized) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(serialized);
        }
    }

    *state.focusa.write().await = new_state;
    state.mark_external_mutation();
    Ok(())
}

/// POST /v1/turn/start
///
/// Adapter calls this when user input is received.
/// API materializes TurnStarted synchronously so Pi hot paths do not depend on
/// daemon command-channel latency during LowMem or recovery windows.
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

    let event = FocusaEvent::TurnStarted {
        turn_id: req.turn_id.clone(),
        harness_name: req.harness_name.clone(),
        adapter_id: req.adapter_id.clone(),
        raw_user_input: None, // Will be set when prompt_assemble is called
    };
    materialize_turn_event(&state, event, "api:turn:start").await?;

    Ok(Json(json!({
        "status": "accepted",
        "turn_id": req.turn_id
    })))
}

fn assemble_baseline_raw(
    focusa: &FocusaState,
    req: &PromptAssembleRequest,
    config: &FocusaConfig,
) -> focusa_core::expression::engine::AssembledPrompt {
    let raw_snapshot = json!({
        "focus_stack": focusa.focus_stack,
        "threads": focusa.threads,
        "semantic_memory": focusa.memory.semantic,
        "procedural_memory": focusa.memory.procedural,
        "focus_gate": focusa.focus_gate.candidates,
        "reference_handles": focusa.reference_index.handles,
        "operator_input": req.raw_user_input,
    });

    let mut content = format!(
        "RAW HARNESS BASELINE\nSTATE SNAPSHOT:\n{}\n\nUSER INPUT:\n{}\n\nDIRECTIVE: Respond with the next best step using the raw snapshot above.",
        serde_json::to_string_pretty(&raw_snapshot).unwrap_or_default(),
        req.raw_user_input,
    );

    let budget = available_tokens(config.max_prompt_tokens, config.reserve_for_response);
    let mut warnings = Vec::new();
    let mut degraded = false;
    let mut token_estimate = estimate_tokens(&content);
    if token_estimate > budget {
        let marker = "\n[BASELINE RAW TRUNCATED — fit to token budget]";
        let marker_tokens = estimate_tokens(marker);
        let content_budget = budget.saturating_sub(marker_tokens);
        let max_chars = (content_budget * 4) as usize;
        let boundary = if max_chars >= content.len() {
            content.len()
        } else {
            let mut idx = max_chars;
            while idx > 0 && !content.is_char_boundary(idx) {
                idx -= 1;
            }
            idx
        };
        content = format!("{}{}", &content[..boundary], marker);
        token_estimate = estimate_tokens(&content);
        degraded = true;
        warnings.push("Baseline truncation: raw snapshot cut to fit budget".to_string());
    }

    focusa_core::expression::engine::AssembledPrompt {
        content,
        token_estimate,
        handles_used: vec![],
        degraded,
        warnings,
    }
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
    let active_frame_id = focusa.focus_stack.active_id.map(|id| id.to_string());
    let ontology_slice_summary =
        ontology::active_mission_slice_summary(&focusa, active_frame_id.as_deref());
    let ontology_slice_payload = serde_json::json!({
        "slice_type": "active_mission",
        "summary_present": ontology_slice_summary.is_some(),
    });

    // Assemble prompt with full context.
    // Respect per-request budget override strictly: requested budget applies to
    // prompt content itself, so reserve_for_response is zeroed for this call.
    let mut effective_config = state.config.clone();
    if let Some(budget) = req.budget {
        effective_config.max_prompt_tokens = budget;
        effective_config.reserve_for_response = 0;
    }

    let input = focusa_core::expression::engine::AssemblyInput {
        focus_state: &focus_state,
        frame_title,
        ascc: ascc_ref,
        parent_frames: &parents,
        rules: &rules_owned,
        handles: &handles_owned,
        user_input: &req.raw_user_input,
        directive: ontology_slice_summary.as_deref(),
        constitution_principles: &principles,
        safety_rules: &safety,
        config: &effective_config,
        rehydrate_handles: None,
        thesis: focusa
            .threads
            .iter()
            .find(|t| t.status == focusa_core::types::ThreadStatus::Active)
            .map(|t| &t.thesis),
    };
    let assembly = match req.strategy.as_deref() {
        Some("baseline_raw") => assemble_baseline_raw(&focusa, &req, &effective_config),
        _ => focusa_core::expression::engine::assemble_from(input),
    };

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

    // Update runtime-only active turn correlation without blocking prompt assembly hot path.
    let mut warnings = assembly.warnings;
    let mut degraded = assembly.degraded;
    drop(focusa);
    if let Err(error) = state.command_tx.try_send(Action::UpdateActiveTurnRuntime {
        turn_id: req.turn_id,
        raw_user_input: Some(req.raw_user_input.clone()),
        assembled_prompt: Some(assembly.content.clone()),
        append_prompt: None,
    }) {
        degraded = true;
        warnings.push(format!("runtime_turn_update_skipped: {error}"));
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
        "strategy": req.strategy.clone().unwrap_or_else(|| "focusa".to_string()),
        "warnings": warnings,
        "degraded": degraded,
        "ontology_slice": ontology_slice_payload,
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

    // Append to runtime-only active turn correlation through the daemon action path.
    state
        .command_tx
        .send(Action::UpdateActiveTurnRuntime {
            turn_id: req.turn_id,
            raw_user_input: None,
            assembled_prompt: None,
            append_prompt: Some(req.chunk),
        })
        .await
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "runtime_turn_update_failed",
                    "failure_class": "daemon_unavailable",
                    "error": error.to_string(),
                    "retry_posture": "safe_retry"
                })),
            )
        })?;

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
    scope_context: ScopeContext,
    Json(req): Json<TurnComplete>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        turn_id = %req.turn_id,
        output_len = req.assistant_output.len(),
        artifacts = req.artifacts.len(),
        errors = req.errors.len(),
        "Turn completed"
    );

    let turn_scope = scope_context.require_workstream_key().map_err(|reason| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "rejected",
                "failure_class": "scope_required",
                "reason": reason,
            })),
        )
    })?;

    // Hot idempotency guard: repeated completion for the same turn_id must not
    // wait on SQLite or the daemon write lock during resource pressure.
    if recent_turn_completed(&state, &turn_scope, &req.turn_id).await {
        tracing::debug!(turn_id = %req.turn_id, "Duplicate turn_complete ignored");
        return Ok(Json(json!({
            "status": "accepted",
            "turn_id": req.turn_id,
            "duplicate": true
        })));
    }

    // Get harness_name/raw_user_input plus current continuous-work task if available.
    let (harness_name, raw_user_input, current_task, work_loop_enabled) = {
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
        (
            hn,
            rui,
            focusa.work_loop.current_task.clone(),
            focusa.work_loop.enabled,
        )
    };

    // Materialize synchronously for idempotency and persistence even when the
    // daemon command channel is saturated. The reducer handles CLT recording,
    // error signals, telemetry, and active_turn clearing.
    let event = FocusaEvent::TurnCompleted {
        turn_id: req.turn_id.clone(),
        harness_name,
        raw_user_input: raw_user_input.or(req.raw_user_input),
        assistant_output: Some(req.assistant_output.clone()),
        artifacts_used: req.artifacts.clone(),
        errors: req.errors.clone(),
        // §35.5: Support both canonical + extension token formats
        prompt_tokens: req
            .prompt_tokens
            .or(req.tokens.as_ref().and_then(|t| t.input_tokens)),
        completion_tokens: req
            .completion_tokens
            .or(req.tokens.as_ref().and_then(|t| t.output_tokens)),
    };

    remember_completed_turn(&state, turn_scope, &req.turn_id).await;
    let state_for_event = Arc::clone(&state);
    tokio::spawn(async move {
        if let Err((status, body)) =
            materialize_turn_event(&state_for_event, event, "api:turn:complete").await
        {
            tracing::error!(
                ?status,
                ?body,
                "failed to materialize turn_complete asynchronously"
            );
        }
    });

    if work_loop_enabled && let Some(task) = current_task {
        let assistant_output = req.assistant_output.trim();
        let assistant_excerpt = assistant_output.chars().take(220).collect::<String>();
        let summary = if assistant_output.is_empty() {
            "continuous turn completed with empty assistant output".to_string()
        } else {
            format!(
                "continuous turn completed for {}: {assistant_excerpt}",
                task.work_item_id
            )
        };
        let receipt = parse_work_loop_outcome_receipt(assistant_output);
        let receipt_matches = receipt
            .as_ref()
            .is_some_and(|receipt| receipt.work_item_id == task.work_item_id);
        let outcome_status = receipt
            .as_ref()
            .filter(|_| receipt_matches)
            .map(|receipt| receipt.status)
            .unwrap_or(WorkLoopOutcomeStatus::Continue);
        let evidence_citations = receipt
            .as_ref()
            .filter(|_| receipt_matches)
            .map(|receipt| receipt.evidence_citations.clone())
            .unwrap_or_default();
        let spec_conformant = req.errors.is_empty()
            && receipt
                .as_ref()
                .filter(|_| receipt_matches)
                .is_some_and(|receipt| receipt.spec_conformant);
        let verification_satisfied =
            outcome_status == WorkLoopOutcomeStatus::Completed && !evidence_citations.is_empty();
        let summary = receipt
            .as_ref()
            .filter(|_| receipt_matches)
            .and_then(|receipt| receipt.summary.clone())
            .unwrap_or(summary);
        let observe_action = Action::ObserveContinuousTurnOutcome {
            task_run_id: None,
            work_item_id: Some(task.work_item_id.clone()),
            summary,
            continue_reason: Some(format!(
                "turn outcome observed; typed_receipt={receipt_matches}; evidence: {assistant_excerpt}"
            )),
            verification_satisfied,
            spec_conformant,
            outcome_status,
            evidence_citations,
        };
        match tokio::time::timeout(
            std::time::Duration::from_millis(250),
            state.command_tx.send(observe_action),
        )
        .await
        {
            Ok(Ok(())) => {
                let _ = maybe_dispatch_continuous_turn_prompt(
                    &state,
                    "continuous turn outcome evaluated and ready work remains",
                )
                .await;
            }
            Ok(Err(e)) => tracing::error!("Failed to observe continuous turn outcome: {}", e),
            Err(_) => tracing::warn!(
                "Timed out enqueueing continuous turn outcome; accepted turn completion without blocking hot path"
            ),
        }
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
    use crate::scoped_store::ScopedCrdtLedger;
    use crate::server::{AppState, build_router};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use focusa_core::prediction::PredictionValue;
    use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
    use focusa_core::types::{
        Action, EventLogEntry, FocusaConfig, FocusaEvent, FocusaState, SignalOrigin,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
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
                    let entry = EventLogEntry::captured(event, SignalOrigin::Daemon, None);
                    let _ = p.append_event(&entry);
                }
            }
        });

        let state = Arc::new(AppState {
            focusa,
            command_tx: tx,
            events_tx,
            event_broadcaster: crate::routes::sse::EventBroadcaster::new(),
            config: cfg.clone(),
            license_guard: {
                let mut entitlement = focusa_license::authority::EntitlementSnapshot::unactivated(
                    "focusa",
                    "test-node",
                );
                entitlement.state = focusa_license::authority::EntitlementState::Active;
                entitlement.lease_id = Some("test-lease".to_string());
                entitlement.sequence = Some(1);
                entitlement.lease_digest = Some("sha256:test-lease-digest".to_string());
                entitlement.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
                focusa_license::LicenseGuard::from_entitlement(entitlement)
            },
            persistence: persistence.clone(),
            persistence_actor: None,
            write_serial_lock: Arc::new(Mutex::new(())),
            command_store: Arc::new(RwLock::new(HashMap::new())),
            token_store: Arc::new(RwLock::new(focusa_core::permissions::TokenStore::new())),
            writer_claims: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            next_writer_fencing_token: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            focus_stack_by_scope: Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            prediction_store: Arc::new(ScopedCrdtLedger::<PredictionValue>::new(
                &cfg.data_dir,
                "predictions-test",
                "test-actor",
            )),
            prediction_authority_store: Arc::new(ScopedCrdtLedger::<
                focusa_core::prediction_authority::ScopedAuthorityEvent,
            >::new(
                &cfg.data_dir,
                "prediction-authority-test",
                "test-actor",
            )),
            recent_completed_turns_by_scope: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            snapshots_by_scope: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metacog_by_scope: Arc::new(std::sync::Mutex::new(HashMap::new())),
            started_at: Instant::now(),
            pi_rpc_session: Arc::new(Mutex::new(None)),
            supervisor_perf: Arc::new(crate::server::SupervisorPerfCounters::default()),
            external_mutation_epoch: Arc::new(std::sync::atomic::AtomicU64::new(0)),
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
            .header("x-scope-project-root", "/tmp/focusa-spec104-turn-a")
            .header("x-scope-continuity-id", "cont-a")
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
            .header("x-scope-project-root", "/tmp/focusa-spec104-turn-a")
            .header("x-scope-continuity-id", "cont-a")
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
