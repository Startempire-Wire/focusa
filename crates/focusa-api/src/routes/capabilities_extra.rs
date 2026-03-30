//! Additional capabilities domains (docs/23).
//!
//! Implements cache/metrics/intuition/contribute/export/autonomy/gate/constitution extras.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::{Json, Router, routing::get, routing::post};
use chrono::Utc;
use focusa_core::types::{
    CacheBustCategory, CacheClass, ContributionStatus, Signal, SignalKind, SignalOrigin,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

fn token_enabled(state: &AppState) -> bool {
    state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok()
}

fn require_scope(
    headers: &HeaderMap,
    state: &AppState,
    scope: &str,
) -> Result<(), (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(headers, token_enabled(state));
    if permissions.allows(scope) {
        Ok(())
    } else {
        Err(forbid(scope))
    }
}

// ─── Autonomy ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AutonomyQuery {
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AutonomyExplainQuery {
    #[serde(default)]
    event_id: Option<usize>,
}

async fn autonomy_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AutonomyQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "autonomy:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "agent_id": q.agent_id,
        "level": s.autonomy.level,
        "ari_score": s.autonomy.ari_score,
        "dimensions": s.autonomy.dimensions,
        "sample_count": s.autonomy.sample_count,
        "history_count": s.autonomy.history.len(),
    })))
}

async fn autonomy_ledger(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AutonomyQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "autonomy:read")?;
    let s = state.focusa.read().await;
    let mut history = s.autonomy.history.clone();
    if let Some(limit) = q.limit {
        history.truncate(limit);
    }
    Ok(Json(json!({
        "agent_id": q.agent_id,
        "history": history,
    })))
}

async fn autonomy_explain(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AutonomyExplainQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "autonomy:read")?;
    let s = state.focusa.read().await;
    let a = &s.autonomy;
    let event = q
        .event_id
        .and_then(|idx| a.history.get(idx).cloned())
        .or_else(|| a.history.last().cloned());

    // Confidence band per docs/12 §Confidence & Sample Size.
    let confidence_band = if a.sample_count < 5 {
        "low"
    } else if a.sample_count < 30 {
        "medium"
    } else {
        "high"
    };

    // Weighted contributions per docs/12 §Scoring Categories.
    let outcome = (a.dimensions.correctness + a.dimensions.trust) / 2.0;
    let efficiency = a.dimensions.efficiency;
    let discipline = a.dimensions.grounding;
    let safety = (a.dimensions.stability + a.dimensions.recovery) / 2.0;

    let recommendation = focusa_core::autonomy::should_recommend_promotion(a);

    // Per docs/12 §Explainability Requirement:
    // "Every ARI value MUST be explainable by: listing contributing events,
    //  showing penalties applied, showing normalization factors.
    //  No opaque aggregation is allowed."
    Ok(Json(json!({
        "ari_score": a.ari_score,
        "sample_count": a.sample_count,
        "confidence_band": confidence_band,
        "level": a.level,
        "recommendation": recommendation,
        "dimension_scores": {
            "correctness": a.dimensions.correctness,
            "stability": a.dimensions.stability,
            "efficiency": a.dimensions.efficiency,
            "trust": a.dimensions.trust,
            "grounding": a.dimensions.grounding,
            "recovery": a.dimensions.recovery,
        },
        "weighted_contributions": {
            "outcome": { "weight": 0.50, "score": outcome, "sources": ["correctness", "trust"] },
            "efficiency": { "weight": 0.20, "score": efficiency, "sources": ["efficiency"] },
            "discipline": { "weight": 0.15, "score": discipline, "sources": ["grounding"] },
            "safety": { "weight": 0.15, "score": safety, "sources": ["stability", "recovery"] },
        },
        "scoring_formula": "ARI = clamp((outcome*0.50 + efficiency*0.20 + discipline*0.15 + safety*0.15) * 100, 0, 100)",
        "promotion_ladder": [
            { "from": "AL0", "to": "AL1", "requires_ari": 40.0, "requires_samples": 10 },
            { "from": "AL1", "to": "AL2", "requires_ari": 55.0, "requires_samples": 25 },
            { "from": "AL2", "to": "AL3", "requires_ari": 70.0, "requires_samples": 50 },
            { "from": "AL3", "to": "AL4", "requires_ari": 80.0, "requires_samples": 100 },
            { "from": "AL4", "to": "AL5", "requires_ari": 90.0, "requires_samples": 200 },
        ],
        "selected_event": event,
        "recent_events": &a.history[a.history.len().saturating_sub(10)..],
    })))
}

// ─── Gate ────────────────────────────────────────────────────────────────

async fn gate_policy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "gate:read")?;
    Ok(Json(json!({
        "surface_threshold": state.config.gate_surface_threshold,
        "decay_factor": state.config.gate_decay_factor,
        "max_candidates": state.config.gate_max_candidates,
    })))
}

async fn gate_scores(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "gate:read")?;
    let s = state.focusa.read().await;
    let scores: Vec<_> = s
        .focus_gate
        .candidates
        .iter()
        .map(|c| json!({"candidate_id": c.id, "pressure": c.pressure, "state": c.state}))
        .collect();

    Ok(Json(json!({"scores": scores})))
}

async fn gate_explain(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "gate:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "candidate_count": s.focus_gate.candidates.len(),
        "signal_count": s.focus_gate.signals.len(),
        "note": "gate explain summarizes current candidate/signal counts",
    })))
}

// ─── Intuition ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct IntuitionSubmit {
    kind: String,
    #[serde(default)]
    payload: Value,
    #[serde(default)]
    confidence: Option<f64>,
}

async fn intuition_signals(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "intuition:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "signals": s.focus_gate.signals,
        "total": s.focus_gate.signals.len(),
    })))
}

async fn intuition_patterns(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "intuition:read")?;
    let s = state.focusa.read().await;
    let patterns: Vec<_> = s
        .focus_gate
        .signals
        .iter()
        .filter(|s| s.kind == SignalKind::RepeatedPattern)
        .cloned()
        .collect();
    Ok(Json(json!({"patterns": patterns, "total": patterns.len()})))
}

async fn intuition_submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<IntuitionSubmit>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "intuition:submit")?;
    let kind = if body.kind == "pattern" {
        SignalKind::RepeatedPattern
    } else {
        SignalKind::Warning
    };

    let signal = Signal {
        id: Uuid::now_v7(),
        ts: Utc::now(),
        origin: SignalOrigin::Worker,
        kind,
        frame_context: None,
        summary: format!(
            "advisory intuition submit (confidence={})",
            body.confidence.unwrap_or(0.5)
        ),
        payload_ref: None,
        tags: vec!["advisory".into()],
    };

    state
        .command_tx
        .send(focusa_core::types::Action::IngestSignal { signal })
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "dispatch failed"})),
            )
        })?;

    Ok(Json(json!({"status": "accepted", "payload": body.payload})))
}

// ─── Metrics ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MetricsQuery {
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    window: Option<String>,
}

async fn metrics_uxp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<MetricsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    Ok(Json(
        json!({"agent_id": q.agent_id, "window": q.window, "uxp": s.uxp}),
    ))
}

async fn metrics_ufi(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<MetricsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    Ok(Json(
        json!({"agent_id": q.agent_id, "window": q.window, "ufi": s.ufi}),
    ))
}

async fn metrics_perf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "total_events": s.telemetry.total_events,
        "total_prompt_tokens": s.telemetry.total_prompt_tokens,
        "total_completion_tokens": s.telemetry.total_completion_tokens,
    })))
}

async fn metrics_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    let active = s.session.as_ref().map(|s| s.session_id.to_string());
    if active.as_deref() != Some(&session_id) {
        return Ok(Json(json!({"error": "session_id not active"})));
    }

    Ok(Json(json!({
        "session_id": session_id,
        "telemetry": s.telemetry,
    })))
}

// ─── Cache ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CacheEventsQuery {
    #[serde(default)]
    limit: Option<usize>,
}

async fn cache_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "cache:read")?;
    Ok(Json(
        json!({"entries": 0, "note": "cache store not yet persisted"}),
    ))
}

async fn cache_policy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "cache:read")?;
    let classes = vec![
        CacheClass::C0,
        CacheClass::C1,
        CacheClass::C2,
        CacheClass::C3,
        CacheClass::C4,
    ];
    let bust = vec![
        CacheBustCategory::FreshEvidence,
        CacheBustCategory::AuthorityChange,
        CacheBustCategory::Compaction,
        CacheBustCategory::Staleness,
        CacheBustCategory::SalienceCollapse,
        CacheBustCategory::ProviderMismatch,
    ];
    Ok(Json(json!({"classes": classes, "bust_categories": bust})))
}

async fn cache_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<CacheEventsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "cache:read")?;
    Ok(Json(json!({"events": [], "limit": q.limit.unwrap_or(0)})))
}

// ─── Contribution ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ContributeQuery {
    #[serde(default)]
    status: Option<ContributionStatus>,
}

async fn contribute_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "contribute:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "enabled": s.contribution.enabled,
        "total_contributed": s.contribution.total_contributed,
        "queue_size": s.contribution.queue.len(),
    })))
}

async fn contribute_policy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "contribute:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({"policy": s.contribution.policy})))
}

async fn contribute_queue(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ContributeQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "contribute:read")?;
    let s = state.focusa.read().await;
    let items: Vec<_> = s
        .contribution
        .queue
        .iter()
        .filter(|item| q.status.map(|s| s == item.status).unwrap_or(true))
        .cloned()
        .collect();

    Ok(Json(json!({"items": items, "total": items.len()})))
}

// ─── Export ──────────────────────────────────────────────────────────────

async fn export_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "export:read")?;
    Ok(Json(json!({"exports": []})))
}

async fn export_manifest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(export_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "export:read")?;
    Ok(Json(
        json!({"error": "export_id not found", "export_id": export_id}),
    ))
}

// ─── Constitution extras ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ConstitutionDiffQuery {
    #[serde(default)]
    agent_id: Option<String>,
    from: String,
    to: String,
}

async fn constitution_diff(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ConstitutionDiffQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "constitution:read")?;
    let s = state.focusa.read().await;
    let from = s.constitution.versions.iter().find(|c| c.version == q.from);
    let to = s.constitution.versions.iter().find(|c| c.version == q.to);

    Ok(Json(json!({
        "agent_id": q.agent_id,
        "from": q.from,
        "to": q.to,
        "changes": [
            {"field": "principles", "from": from.map(|c| c.principles.len()), "to": to.map(|c| c.principles.len())},
            {"field": "safety_rules", "from": from.map(|c| c.safety_rules.len()), "to": to.map(|c| c.safety_rules.len())},
            {"field": "expression_rules", "from": from.map(|c| c.expression_rules.len()), "to": to.map(|c| c.expression_rules.len())}
        ]
    })))
}

#[derive(Debug, Deserialize)]
struct ConstitutionDraftsQuery {
    #[serde(default)]
    agent_id: Option<String>,
}

async fn constitution_drafts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ConstitutionDraftsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "constitution:read")?;
    Ok(Json(json!({"agent_id": q.agent_id, "drafts": []})))
}

// ─── Missing skill-mapped endpoints (docs/34) ─────────────────────────────

/// GET /v1/state/explain — docs/34 §5.1 focusa.explain_last_decision
async fn state_explain(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "state:read")?;
    let s = state.focusa.read().await;
    let active = s.focus_stack.active_id
        .and_then(|aid| s.focus_stack.frames.iter().find(|f| f.id == aid));
    Ok(Json(json!({
        "active_frame": active.map(|f| json!({
            "title": f.title,
            "goal": f.goal,
            "focus_state": f.focus_state,
        })),
        "recent_decisions": active.map(|f| &f.focus_state.decisions).unwrap_or(&vec![]),
        "recent_failures": active.map(|f| &f.focus_state.failures).unwrap_or(&vec![]),
        "gate_candidates": s.focus_gate.candidates.iter()
            .filter(|c| c.state == focusa_core::types::CandidateState::Surfaced)
            .map(|c| json!({"id": c.id, "label": c.label, "pressure": c.pressure}))
            .collect::<Vec<_>>(),
    })))
}

/// GET /v1/references/salient — docs/34 §3.5 focusa.get_salient_references
async fn references_salient(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;
    let session_id = s.session.as_ref().map(|sess| sess.session_id);
    let salient: Vec<&focusa_core::types::HandleRef> = s.reference_index.handles.iter()
        .filter(|h| h.pinned || h.session_id == session_id)
        .collect();
    Ok(Json(json!({ "references": salient })))
}

/// GET /v1/references/trace — docs/34 §5.2 focusa.trace_reference_usage
#[derive(Debug, Deserialize)]
struct TraceQuery {
    ref_id: String,
}

async fn references_trace(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<TraceQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;
    // Find CLT nodes that reference this artifact.
    let ref_id = &q.ref_id;
    let used_in: Vec<&str> = s.clt.nodes.iter()
        .filter(|n| {
            if let focusa_core::types::CltPayload::Interaction { content_ref, .. } = &n.payload {
                content_ref.as_deref() == Some(ref_id)
            } else {
                false
            }
        })
        .map(|n| n.node_id.as_str())
        .collect();
    Ok(Json(json!({ "ref_id": ref_id, "used_in": used_in })))
}

/// GET /v1/telemetry/process — docs/34 §4.2 focusa.get_cognitive_metrics
async fn telemetry_process(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    let stack_depth = s.focus_stack.stack_path_cache.len();
    let completed = s.focus_stack.frames.iter()
        .filter(|f| f.status == focusa_core::types::FrameStatus::Completed).count();
    let abandoned = s.focus_stack.frames.iter()
        .filter(|f| f.completion_reason == Some(focusa_core::types::CompletionReason::Abandoned)).count();
    let total_frames = s.focus_stack.frames.len().max(1);
    Ok(Json(json!({
        "avg_focus_depth": stack_depth,
        "abandonment_rate": abandoned as f64 / total_frames as f64,
        "gate_acceptance": s.focus_gate.candidates.iter()
            .filter(|c| c.state == focusa_core::types::CandidateState::Surfaced).count(),
        "total_frames": total_frames,
        "completed_frames": completed,
    })))
}

/// GET /v1/telemetry/ux — docs/34 §4.3 focusa.get_ux_signals
async fn telemetry_ux(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "metrics:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "uxp": {
            "autonomy_tolerance": s.uxp.autonomy_tolerance.value,
            "verbosity_preference": s.uxp.verbosity_preference.value,
            "interruption_sensitivity": s.uxp.interruption_sensitivity.value,
        },
        "ufi": {
            "aggregate": s.ufi.aggregate,
            "signal_count": s.ufi.signals.len(),
        },
    })))
}

/// POST /v1/constitution/propose — docs/34 §6.3 focusa.propose_constitution_update
#[derive(Debug, Deserialize)]
struct ConstitutionProposeBody {
    draft: String,
    justification: String,
}

async fn constitution_propose(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ConstitutionProposeBody>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "constitution:propose")?;
    // Submit as a proposal via PRE (docs/41).
    let _ = state.command_tx.send(focusa_core::types::Action::SubmitProposal {
        kind: focusa_core::types::ProposalKind::ConstitutionRevision,
        source: "agent".into(),
        payload: serde_json::json!({
            "draft": body.draft,
            "justification": body.justification,
        }),
        deadline_ms: 3600_000, // 1 hour.
    }).await.map_err(|_| (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "failed to submit proposal"})),
    ))?;
    Ok(Json(json!({ "status": "proposed" })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Autonomy
        .route("/v1/autonomy/status", get(autonomy_status))
        .route("/v1/autonomy/ledger", get(autonomy_ledger))
        .route("/v1/autonomy/explain", get(autonomy_explain))
        // Gate
        .route("/v1/gate/policy", get(gate_policy))
        .route("/v1/gate/scores", get(gate_scores))
        .route("/v1/gate/explain", get(gate_explain))
        // Intuition
        .route("/v1/intuition/signals", get(intuition_signals))
        .route("/v1/intuition/patterns", get(intuition_patterns))
        .route("/v1/intuition/submit", post(intuition_submit))
        // Metrics
        .route("/v1/metrics/uxp", get(metrics_uxp))
        .route("/v1/metrics/ufi", get(metrics_ufi))
        .route("/v1/metrics/perf", get(metrics_perf))
        .route("/v1/metrics/session/{session_id}", get(metrics_session))
        // Cache
        .route("/v1/cache/status", get(cache_status))
        .route("/v1/cache/policy", get(cache_policy))
        .route("/v1/cache/events", get(cache_events))
        // Contribution
        .route("/v1/contribute/status", get(contribute_status))
        .route("/v1/contribute/policy", get(contribute_policy))
        .route("/v1/contribute/queue", get(contribute_queue))
        // Export
        .route("/v1/export/history", get(export_history))
        .route("/v1/export/manifest/{export_id}", get(export_manifest))
        // Constitution extras
        .route("/v1/constitution/diff", get(constitution_diff))
        .route("/v1/constitution/drafts", get(constitution_drafts))
        .route("/v1/constitution/propose", post(constitution_propose))
        // Skill-mapped endpoints (docs/34)
        .route("/v1/state/explain", get(state_explain))
        .route("/v1/references/salient", get(references_salient))
        .route("/v1/references/trace", get(references_trace))
        .route("/v1/telemetry/process", get(telemetry_process))
        .route("/v1/telemetry/ux", get(telemetry_ux))
}
