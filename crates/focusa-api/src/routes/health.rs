//! GET /v1/health

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_ms": state.started_at.elapsed().as_millis() as u64,
    }))
}

async fn doctor(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let s = state.focusa.read().await;
    let token_records = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_token_budget")
        })
        .count();
    let cache_records = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_cache_metadata")
        })
        .count();
    let active_frame = s
        .focus_stack
        .active_id
        .and_then(|id| s.focus_stack.frames.iter().find(|frame| frame.id == id));
    Json(json!({
        "status": "ok",
        "summary": "Focusa daemon is reachable; minimal Spec92 doctor checks passed",
        "daemon": {
            "ok": true,
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_ms": state.started_at.elapsed().as_millis() as u64,
        },
        "focus": {
            "active_frame_id": active_frame.map(|frame| frame.id.to_string()),
            "active_frame_title": active_frame.map(|frame| frame.title.clone()),
            "stack_depth": s.focus_stack.frames.len(),
        },
        "telemetry": {
            "total_events": s.telemetry.total_events,
            "token_budget_records": token_records,
            "cache_metadata_records": cache_records,
            "tool_calls": s.telemetry.tool_calls.len(),
        },
        "next_action": if token_records == 0 || cache_records == 0 { "run a Pi/provider turn, then re-run focusa doctor" } else { "continue normally; use focusa telemetry token-budget and focusa cache doctor for detail" },
        "commands": ["focusa telemetry token-budget", "focusa cache doctor", "focusa work-loop status", "focusa workpoint current"],
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/doctor", get(doctor))
}
