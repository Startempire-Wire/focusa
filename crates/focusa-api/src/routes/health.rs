//! GET /v1/health

use crate::routes::bounded::resource_mode_status;
use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::Ordering;

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
    let perf = &state.supervisor_perf;
    let driver_start_attempts = perf.driver_start_attempts.load(Ordering::Relaxed);
    let driver_stop_attempts = perf.driver_stop_attempts.load(Ordering::Relaxed);
    let dispatch_recovery_restarts = perf.dispatch_recovery_restarts.load(Ordering::Relaxed);
    let current_task_present = s.work_loop.current_task.is_some();
    let idle_without_task =
        s.work_loop.status == focusa_core::types::WorkLoopStatus::Idle && !current_task_present;
    let churn_risk = idle_without_task && (driver_start_attempts > 0 || driver_stop_attempts > 0);
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
        "work_loop": {
            "enabled": s.work_loop.enabled,
            "status": s.work_loop.status,
            "current_task_present": current_task_present,
            "supervisor_perf": {
                "supervisor_ticks_total": perf.ticks_total.load(Ordering::Relaxed),
                "driver_start_attempts": driver_start_attempts,
                "driver_stop_attempts": driver_stop_attempts,
                "dispatch_attempts": perf.dispatch_attempts.load(Ordering::Relaxed),
                "dispatch_skipped_disallowed": perf.dispatch_skipped_disallowed.load(Ordering::Relaxed),
                "dispatch_recovery_restarts": dispatch_recovery_restarts,
                "background_throttled_ticks": perf.background_throttled_ticks.load(Ordering::Relaxed),
            },
            "churn_diagnostic": {
                "status": if churn_risk { "warn" } else { "ok" },
                "risk": churn_risk,
                "reason": if churn_risk { "pi-rpc supervisor driver counters changed while work-loop is idle with no current task" } else { "no idle/no-task driver churn detected" },
                "recommended_action": if churn_risk { "inspect /v1/work-loop/status?summary_only=true, stop stale driver if present, and verify idle start gate" } else { "continue normally" },
            }
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
        "next_action": if churn_risk { "inspect work-loop supervisor churn before broad work" } else if token_records == 0 || cache_records == 0 { "run a Pi/provider turn, then re-run focusa doctor" } else { "continue normally; use focusa telemetry token-budget and focusa cache doctor for detail" },
        "commands": ["focusa resource status", "focusa telemetry token-budget", "focusa cache doctor", "focusa work-loop status", "focusa workpoint current"],
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/doctor", get(doctor))
}
