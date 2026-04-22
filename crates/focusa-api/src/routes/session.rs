//! Session routes.
//!
//! GET  /v1/status        — daemon/session status (summary)
//! GET  /v1/state/dump    — full cognitive state (debug)
//! POST /v1/session/start — start a new session
//! POST /v1/session/resume — restore a previous session
//! POST /v1/session/close — close current session

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, SessionStatus};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::sync::Arc;

async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let (
        session,
        stack_depth,
        active_frame_id,
        version,
        active_frame_summary,
        prompt_stats,
        worker_status,
        telemetry,
    ) = {
        let focusa = state.focusa.read().await;

        let session_is_active = focusa
            .session
            .as_ref()
            .map(|s| s.status == SessionStatus::Active)
            .unwrap_or(false);

        let active_frame = if session_is_active {
            focusa
                .focus_stack
                .active_id
                .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
        } else {
            None
        };

        let active_frame_summary = active_frame.map(|f| {
            json!({
                "id": f.id,
                "title": f.title,
                "goal": f.goal,
                "status": f.status,
                "updated_at": f.updated_at,
            })
        });

        let assembled_chars = focusa
            .active_turn
            .as_ref()
            .and_then(|t| t.assembled_prompt.as_ref())
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        let prompt_stats = json!({
            "last_assembled_chars": assembled_chars,
            "last_assembled_estimated_tokens": assembled_chars / 4,
            "active_turn_id": focusa.active_turn.as_ref().map(|t| t.turn_id.clone()),
        });

        let worker_status = json!({
            "queue_size_config": state.config.worker_queue_size,
            "job_timeout_ms": state.config.worker_job_timeout_ms,
            "enabled": true,
        });

        let telemetry = json!({
            "total_events": focusa.telemetry.total_events,
            "total_prompt_tokens": focusa.telemetry.total_prompt_tokens,
            "total_completion_tokens": focusa.telemetry.total_completion_tokens,
        });

        (
            focusa.session.clone(),
            focusa.focus_stack.frames.len(),
            if session_is_active {
                focusa.focus_stack.active_id
            } else {
                None
            },
            focusa.version,
            active_frame_summary,
            prompt_stats,
            worker_status,
            telemetry,
        )
    };

    let last_event_ts = state.persistence.latest_event_timestamp().ok().flatten();
    let persisted_event_count = state.persistence.event_count().ok();

    let daemon_pids = focusa_daemon_pids();
    let current_pid = std::process::id();
    let duplicate_daemon_count = daemon_pids.iter().filter(|&&p| p != current_pid).count() as u64;
    let supervisor_perf = &state.supervisor_perf;
    let memory_budget_mb = std::env::var("FOCUSA_MEMORY_BUDGET_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2200);
    let rss_kb = current_rss_kb();
    let host_mem_available_kb = host_mem_available_kb();
    let degraded = rss_kb
        .map(|kb| kb > memory_budget_mb.saturating_mul(1024))
        .unwrap_or(false);

    Json(json!({
        "session": session,
        "session_allows_focus_mutation": session.as_ref().map(|s| s.status == SessionStatus::Active).unwrap_or(false),
        "stack_depth": stack_depth,
        "active_frame_id": active_frame_id,
        "active_frame": active_frame_summary,
        "worker_status": worker_status,
        "last_event_ts": last_event_ts,
        "prompt_stats": prompt_stats,
        "telemetry": telemetry,
        "persisted_event_count": persisted_event_count,
        "version": version,
        "runtime_process": {
            "current_pid": current_pid,
            "daemon_pids": daemon_pids,
            "daemon_count": daemon_pids.len(),
            "duplicate_daemon_count": duplicate_daemon_count,
            "single_daemon_ok": duplicate_daemon_count == 0,
        },
        "runtime_memory": {
            "rss_kb": rss_kb,
            "memory_budget_mb": memory_budget_mb,
            "host_mem_available_kb": host_mem_available_kb,
            "degraded": degraded,
        },
        "runtime_perf": {
            "supervisor_ticks_total": supervisor_perf.ticks_total.load(std::sync::atomic::Ordering::Relaxed),
            "driver_start_attempts": supervisor_perf.driver_start_attempts.load(std::sync::atomic::Ordering::Relaxed),
            "driver_stop_attempts": supervisor_perf.driver_stop_attempts.load(std::sync::atomic::Ordering::Relaxed),
            "dispatch_attempts": supervisor_perf.dispatch_attempts.load(std::sync::atomic::Ordering::Relaxed),
            "dispatch_skipped_disallowed": supervisor_perf.dispatch_skipped_disallowed.load(std::sync::atomic::Ordering::Relaxed),
            "dispatch_recovery_restarts": supervisor_perf.dispatch_recovery_restarts.load(std::sync::atomic::Ordering::Relaxed),
        }
    }))
}

fn current_rss_kb() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            return Some(kb);
        }
    }
    None
}

fn host_mem_available_kb() -> Option<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            return Some(kb);
        }
    }
    None
}

fn focusa_daemon_pids() -> Vec<u32> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return out;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let comm_path = format!("/proc/{pid}/comm");
        let Ok(comm) = fs::read_to_string(comm_path) else {
            continue;
        };
        if comm.trim() == "focusa-daemon" {
            out.push(pid);
        }
    }

    out.sort_unstable();
    out
}

/// Full cognitive state dump (debug).
async fn state_dump(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(serde_json::to_value(&*focusa).unwrap_or(json!({"error": "serialization failed"})))
}

#[derive(Deserialize)]
struct StartSessionBody {
    adapter_id: Option<String>,
    workspace_id: Option<String>,
    instance_id: Option<String>,
}

async fn start_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::StartSession {
            adapter_id: body.adapter_id,
            workspace_id: body.workspace_id,
            instance_id: body
                .instance_id
                .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ResumeSessionBody {
    session_id: String,
    #[serde(default)]
    instance_id: Option<String>,
}

/// POST /v1/session/resume — restore a previous session by ID.
/// §36.4: Session resume on Pi /resume.
async fn resume_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResumeSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session_id =
        uuid::Uuid::parse_str(&body.session_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .command_tx
        .send(Action::ResumeSession { session_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        json!({"status": "accepted", "session_id": body.session_id}),
    ))
}

#[derive(Deserialize)]
struct CloseSessionBody {
    #[serde(default = "default_reason")]
    reason: String,
    instance_id: Option<String>,
}

fn default_reason() -> String {
    "user_requested".into()
}

async fn close_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CloseSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::CloseSession {
            reason: body.reason,
            instance_id: body
                .instance_id
                .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/state/dump", get(state_dump))
        .route("/v1/session/start", post(start_session))
        .route("/v1/session/resume", post(resume_session))
        .route("/v1/session/close", post(close_session))
}
