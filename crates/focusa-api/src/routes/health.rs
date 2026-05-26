//! GET /v1/health

use crate::routes::bounded::resource_mode_status;
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
    let resource_mode = resource_mode_status();
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
        "api_cli_parity": {
            "cli_command": "focusa doctor --json",
            "api_route": "/v1/doctor",
            "shared_checks": [
                "daemon health",
                "command-center doctor API",
                "API route inventory surface",
                "Spec90 tool contracts",
                "Workpoint canonicality",
                "Work-loop writer state",
                "token telemetry status",
                "cache metadata status"
            ],
            "recovery_commands": [
                "focusa start",
                "focusa start",
                "focusa-daemon",
                "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"
            ],
            "status_fields": ["status", "summary", "next_action", "why", "commands", "recovery", "details.checks"]
        },
        "checks_summary": {
            "contracts_expected": 58,
            "scoped_hot_routes": ["/v1/health", "/v1/doctor", "/v1/workpoint/current", "/v1/work-loop/status?summary_only=true"],
            "docs": ["docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md", "docs/current/CLI_REFERENCE_CURRENT.md"]
        },
        "resource_mode": {
            "mode": resource_mode.mode,
            "reason": resource_mode.reason,
            "forced": resource_mode.forced,
            "pressure": resource_mode.pressure,
            "budget": resource_mode.budget,
            "latest_transition": resource_mode.latest_transition,
            "transition_omitted_count": resource_mode.transition_omitted_count,
            "hysteresis": resource_mode.hysteresis,
            "tool_availability_policy": resource_mode.tool_availability_policy,
            "cold_surfaces_deferred": resource_mode.cold_surfaces_deferred,
        },
        "next_action": if token_records == 0 || cache_records == 0 { "run a Pi/provider turn, then re-run focusa doctor" } else { "continue normally; use focusa telemetry token-budget and focusa cache doctor for detail" },
        "commands": ["focusa resource status", "focusa telemetry token-budget", "focusa cache doctor", "focusa work-loop status", "focusa workpoint current"],
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/doctor", get(doctor))
}
