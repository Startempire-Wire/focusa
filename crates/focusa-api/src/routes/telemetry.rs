//! Telemetry routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

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
    let recorded = body.tools.len();
    let turn_id = body.turn_id.clone();
    let tools = body.tools.clone();
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    for tool in &body.tools {
        focusa.telemetry.tool_calls.push(tool.clone());
    }
    focusa.telemetry.trace_events.push(json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": "tools_invoked",
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": turn_id,
        "payload": {
            "turn_id": body.turn_id,
            "tools": body.tools,
        },
    }));
    Ok(Json(json!({"status": "accepted", "recorded": recorded, "turn_id": turn_id, "tools": tools})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/telemetry/tokens", get(tokens))
        .route("/v1/telemetry/cost", get(cost))
        .route("/v1/telemetry/tools", get(tool_usage))
        .route("/v1/telemetry/tool-usage", post(record_tool_usage))
        .route("/v1/telemetry/activity", post(record_activity_event))
        .route("/v1/telemetry/ops", post(record_operational_event))
        // Deprecated compatibility alias for legacy extension callers.
        .route("/v1/telemetry/event", post(record_operational_event))
        // SPEC 56: Trace dimension endpoints
        .route("/v1/telemetry/trace", post(record_trace_event))
        .route("/v1/telemetry/trace", get(get_trace_events))
        .route("/v1/telemetry/trace/stats", get(get_trace_stats))
}
// ═══════════════════════════════════════════════════════════════════════════════
// Operational + Activity Telemetry
// ═══════════════════════════════════════════════════════════════════════════════

/// POST /v1/telemetry/activity — record session/activity telemetry.
async fn record_activity_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event_name = body.get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("activity_event");

    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "channel": "activity",
        "event": event_name,
        "timestamp": Utc::now().to_rfc3339(),
        "payload": body,
    }));

    Json(serde_json::json!({
        "status": "recorded",
        "channel": "activity",
        "event": event_name,
    }))
}

/// POST /v1/telemetry/ops — record operational telemetry.
/// `/v1/telemetry/event` is kept as a deprecated compatibility alias.
async fn record_operational_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event_name = body.get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("operational_event");

    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "channel": "ops",
        "event": event_name,
        "timestamp": Utc::now().to_rfc3339(),
        "payload": body,
    }));

    Json(serde_json::json!({
        "status": "recorded",
        "channel": "ops",
        "event": event_name,
    }))
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

    let event_type_str = body.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("ModelTokens");
    let _event_type = match event_type_str {
        "mission_frame_context" => TelemetryEventType::MissionFrameContext,
        "working_set_used" => TelemetryEventType::WorkingSetUsed,
        "constraints_consulted" => TelemetryEventType::ConstraintsConsulted,
        "decisions_consulted" => TelemetryEventType::DecisionsConsulted,
        "action_intents_proposed" => TelemetryEventType::ActionIntentsProposed,
        "tools_invoked" => TelemetryEventType::ToolsInvoked,
        "verification_result" => TelemetryEventType::VerificationResult,
        "ontology_delta_applied" => TelemetryEventType::OntologyDeltaApplied,
        "blockers_failures_emitted" => TelemetryEventType::BlockersFailuresEmitted,
        "final_state_transition" => TelemetryEventType::FinalStateTransition,
        "operator_subject" => TelemetryEventType::OperatorSubject,
        "active_subject_after_routing" => TelemetryEventType::ActiveSubjectAfterRouting,
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
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": event_type_str,
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": body.get("turn_id").cloned().unwrap_or(serde_json::Value::Null),
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

    let event_type_filter = params.get("event_type").map(String::as_str);
    let turn_id_filter = params.get("turn_id").map(String::as_str);
    let turn_id_prefix_filter = params.get("turn_id_prefix").map(String::as_str);
    let filtered: Vec<_> = events
        .iter()
        .rev()
        .filter(|e| {
            event_type_filter
                .map(|wanted| e.get("event_type").and_then(|v| v.as_str()) == Some(wanted))
                .unwrap_or(true)
        })
        .filter(|e| {
            turn_id_filter
                .map(|wanted| {
                    let nested = e.get("payload").and_then(|p| p.get("turn_id")).and_then(|v| v.as_str());
                    let top = e.get("turn_id").and_then(|v| v.as_str());
                    nested == Some(wanted) || top == Some(wanted)
                })
                .unwrap_or(true)
        })
        .filter(|e| {
            turn_id_prefix_filter
                .map(|wanted| {
                    let nested = e.get("payload").and_then(|p| p.get("turn_id")).and_then(|v| v.as_str());
                    let top = e.get("turn_id").and_then(|v| v.as_str());
                    nested
                        .or(top)
                        .map(|turn_id| turn_id.starts_with(wanted))
                        .unwrap_or(false)
                })
                .unwrap_or(true)
        })
        .take(limit)
        .cloned()
        .collect();
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
