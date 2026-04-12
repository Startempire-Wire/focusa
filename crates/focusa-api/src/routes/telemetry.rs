//! Telemetry routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/telemetry/tokens — token usage metrics.
async fn tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "total_events": s.telemetry.total_events,
        "total_prompt_tokens": s.telemetry.total_prompt_tokens,
        "total_completion_tokens": s.telemetry.total_completion_tokens,
        "tokens_per_task": s.telemetry.tokens_per_task,
    }))
}

/// GET /v1/telemetry/cost — cost estimate.
async fn cost(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let estimated = focusa_core::telemetry::estimate_cost(&s.telemetry, 0.003, 0.015);
    Json(json!({
        "estimated_cost_usd": estimated,
        "prompt_tokens": s.telemetry.total_prompt_tokens,
        "completion_tokens": s.telemetry.total_completion_tokens,
    }))
}

/// POST /v1/telemetry/tool-usage — record batch of tool calls for autonomy.
#[derive(Debug, Deserialize)]
struct ToolUsageBody {
    turn_id: Option<String>,
    tools: Vec<String>,
}

/// GET /v1/telemetry/tools — get tool usage summary.
async fn tool_usage(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let summary: std::collections::HashMap<String, u32> = s
        .telemetry
        .tool_calls
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, name| {
            *acc.entry(name.clone()).or_insert(0) += 1;
            acc
        });
    Json(json!({
        "total_calls": s.telemetry.tool_calls.len(),
        "tool_summary": summary,
    }))
}

/// POST /v1/telemetry/tool-usage — receive tool call batch from extension.
async fn record_tool_usage(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ToolUsageBody>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    // Feed tool names to telemetry for autonomy analysis.
    let mut focusa = state.focusa.write().await;
    for tool in &body.tools {
        focusa.telemetry.tool_calls.push(tool.clone());
    }
    Ok(Json(json!({"status": "accepted", "recorded": body.tools.len()})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/telemetry/tokens", get(tokens))
        .route("/v1/telemetry/cost", get(cost))
        .route("/v1/telemetry/tools", get(tool_usage))
        .route("/v1/telemetry/tool-usage", post(record_tool_usage))
        // SPEC 56: Trace dimension endpoints
        .route("/v1/telemetry/trace", post(record_trace_event))
        .route("/v1/telemetry/trace", get(get_trace_events))
        .route("/v1/telemetry/trace/stats", get(get_trace_stats))
}
// ═══════════════════════════════════════════════════════════════════════════════
// SPEC 56: Trace Dimensions
// ═══════════════════════════════════════════════════════════════════════════════

/// POST /v1/telemetry/trace — Record a trace dimension event (SPEC 56)
async fn record_trace_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    use focusa_core::types::TelemetryEventType;
    use uuid::Uuid;
    use chrono::Utc;

    let event_type_str = body.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("ModelTokens");
    let _event_type = match event_type_str {
        "working_set_used" => TelemetryEventType::WorkingSetUsed,
        "constraints_consulted" => TelemetryEventType::ConstraintsConsulted,
        "decisions_consulted" => TelemetryEventType::DecisionsConsulted,
        "action_intents_proposed" => TelemetryEventType::ActionIntentsProposed,
        "verification_result" => TelemetryEventType::VerificationResult,
        "ontology_delta_applied" => TelemetryEventType::OntologyDeltaApplied,
        "operator_subject" => TelemetryEventType::OperatorSubject,
        "steering_detected" => TelemetryEventType::SteeringDetected,
        "subject_hijack_prevented" => TelemetryEventType::SubjectHijackPrevented,
        "subject_hijack_occurred" => TelemetryEventType::SubjectHijackOccurred,
        "prior_mission_reused" => TelemetryEventType::PriorMissionReused,
        "focus_slice_size" => TelemetryEventType::FocusSliceSize,
        "focus_slice_relevance_score" => TelemetryEventType::FocusSliceRelevanceScore,
        _ => TelemetryEventType::ModelTokens,
    };

    // Store in focusa telemetry state (in-memory)
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": event_type_str,
        "timestamp": Utc::now().to_rfc3339(),
        "payload": body,
    }));

    Json(serde_json::json!({
        "status": "recorded",
        "event_type": event_type_str,
    }))
}

/// GET /v1/telemetry/trace — Get trace events (SPEC 56)
async fn get_trace_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let events = &focusa.telemetry.trace_events;

    let limit = params.get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
        .min(1000);

    let filtered: Vec<_> = events.iter().rev().take(limit).cloned().collect();
    let count = filtered.len();

    Json(serde_json::json!({
        "events": filtered,
        "count": count,
        "limit": limit,
    }))
}

/// GET /v1/telemetry/trace/stats — Get trace stats (SPEC 56)
async fn get_trace_stats(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let events = &focusa.telemetry.trace_events;

    let mut by_type: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for e in events {
        if let Some(t) = e.get("event_type").and_then(|v| v.as_str()) {
            *by_type.entry(t.to_string()).or_insert(0) += 1;
        }
    }

    Json(serde_json::json!({
        "total_events": events.len(),
        "by_event_type": by_type,
    }))
}
