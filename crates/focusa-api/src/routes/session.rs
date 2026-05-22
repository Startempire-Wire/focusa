//! Session routes.
//!
//! GET  /v1/status        — daemon/session status (summary)
//! GET  /v1/state/dump    — full cognitive state (debug)
//! POST /v1/session/start — start a new session
//! POST /v1/session/resume — restore a previous session
//! POST /v1/session/close — close current session

use crate::routes::bounded::resource_mode_status;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    reducer,
    types::{Action, EventLogEntry, FocusaEvent, SessionStatus, SignalOrigin},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
struct StatusQuery {
    #[serde(default)]
    summary_only: bool,
    #[serde(default)]
    deep: bool,
}

async fn status(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> Json<Value> {
    Json(status_payload(&state, query.deep && !query.summary_only).await)
}

async fn status_deep(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(status_payload(&state, true).await)
}

async fn status_payload(state: &Arc<AppState>, include_deep: bool) -> Value {
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

    let current_pid = std::process::id();
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

    let mut payload = json!({
        "status": "ok",
        "route_tier": if include_deep { "cold" } else { "hot" },
        "summary_only": !include_deep,
        "resource_mode": resource_mode_status(),
        "deep_status_route": "/v1/status/deep",
        "cold_omitted": if include_deep { Vec::<&str>::new() } else { vec!["last_event_ts", "persisted_event_count", "runtime_process.daemon_pids"] },
        "session": session,
        "session_allows_focus_mutation": session.as_ref().map(|s| s.status == SessionStatus::Active).unwrap_or(false),
        "stack_depth": stack_depth,
        "active_frame_id": active_frame_id,
        "active_frame": active_frame_summary,
        "worker_status": worker_status,
        "prompt_stats": prompt_stats,
        "telemetry": telemetry,
        "version": version,
        "runtime_process": {
            "current_pid": current_pid,
            "daemon_pids": Value::Null,
            "daemon_count": Value::Null,
            "duplicate_daemon_count": Value::Null,
            "single_daemon_ok": Value::Null,
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
            "background_throttled_ticks": supervisor_perf.background_throttled_ticks.load(std::sync::atomic::Ordering::Relaxed),
        }
    });

    if include_deep {
        let persistence = state.persistence.clone();
        let deep_diagnostics = tokio::task::spawn_blocking(move || {
            let last_event_ts = persistence.latest_event_timestamp().ok().flatten();
            let persisted_event_count = persistence.event_count().ok();
            let daemon_pids = focusa_daemon_pids();
            (last_event_ts, persisted_event_count, daemon_pids)
        })
        .await
        .ok();

        if let Some((last_event_ts, persisted_event_count, daemon_pids)) = deep_diagnostics {
            let duplicate_daemon_count =
                daemon_pids.iter().filter(|&&p| p != current_pid).count() as u64;
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("last_event_ts".into(), json!(last_event_ts));
                obj.insert("persisted_event_count".into(), json!(persisted_event_count));
                obj.insert(
                    "runtime_process".into(),
                    json!({
                        "current_pid": current_pid,
                        "daemon_pids": daemon_pids,
                        "daemon_count": daemon_pids.len(),
                        "duplicate_daemon_count": duplicate_daemon_count,
                        "single_daemon_ok": duplicate_daemon_count == 0,
                    }),
                );
            }
        } else if let Some(obj) = payload.as_object_mut() {
            obj.insert("deep_status_degraded".into(), json!(true));
            obj.insert("failure_class".into(), json!("cold_path_timeout"));
            obj.insert("last_event_ts".into(), Value::Null);
            obj.insert("persisted_event_count".into(), Value::Null);
        }
    }

    payload
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
    project_root: Option<String>,
    continuity_id: Option<String>,
    instance_id: Option<String>,
}

fn session_matches_request(
    session: &focusa_core::types::SessionState,
    adapter_id: &Option<String>,
    workspace_id: &Option<String>,
    project_root: &Option<String>,
    continuity_id: &Option<String>,
) -> bool {
    session.status == SessionStatus::Active
        && adapter_id
            .as_ref()
            .is_none_or(|expected| session.adapter_id.as_ref() == Some(expected))
        && workspace_id
            .as_ref()
            .is_none_or(|expected| session.workspace_id.as_ref() == Some(expected))
        && project_root
            .as_ref()
            .is_none_or(|expected| session.project_root.as_ref() == Some(expected))
        && continuity_id
            .as_ref()
            .is_none_or(|expected| session.continuity_id.as_ref() == Some(expected))
}

fn active_session_compatible_with_request(
    session: &focusa_core::types::SessionState,
    adapter_id: &Option<String>,
    workspace_id: &Option<String>,
    project_root: &Option<String>,
) -> bool {
    let legacy_pi_recovery_request = project_root.is_none() && adapter_id.as_deref() == Some("pi");
    session.status == SessionStatus::Active
        && project_root
            .as_ref()
            .is_none_or(|expected| session.project_root.as_ref() == Some(expected))
        && (legacy_pi_recovery_request
            || workspace_id
                .as_ref()
                .is_none_or(|expected| session.workspace_id.as_ref() == Some(expected)))
        && (project_root.is_some()
            || adapter_id.as_ref().is_none()
            || session.adapter_id.as_ref() == adapter_id.as_ref()
            || (adapter_id.as_deref().is_some_and(|id| id.starts_with("pi"))
                && session
                    .adapter_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("pi"))))
}

async fn start_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartSessionBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let adapter_id = body.adapter_id.clone();
    let workspace_id = body.workspace_id.clone();
    let project_root = body.project_root.clone();
    let continuity_id = body.continuity_id.clone();
    let instance_id = body
        .instance_id
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    if let Some(session) = state.focusa.read().await.session.clone()
        && active_session_compatible_with_request(
            &session,
            &adapter_id,
            &workspace_id,
            &project_root,
        )
    {
        return Ok(Json(json!({
            "status": "accepted",
            "session_id": session.session_id,
            "adapter_id": session.adapter_id,
            "workspace_id": session.workspace_id,
            "project_root": session.project_root,
            "continuity_id": session.continuity_id,
            "requested_continuity_id": continuity_id,
            "continuity_mismatch": session.continuity_id != continuity_id,
            "materialized_by": "existing_active_project_session",
        })));
    }

    let _guard = state.write_serial_lock.lock().await;

    let current = { state.focusa.read().await.clone() };
    if let Some(session) = &current.session
        && session_matches_request(
            session,
            &adapter_id,
            &workspace_id,
            &project_root,
            &continuity_id,
        )
    {
        return Ok(Json(json!({
            "status": "accepted",
            "session_id": session.session_id,
            "adapter_id": session.adapter_id,
            "workspace_id": session.workspace_id,
            "project_root": session.project_root,
            "continuity_id": session.continuity_id,
            "materialized_by": "existing_active_session",
        })));
    }

    let event = FocusaEvent::SessionStarted {
        session_id: Uuid::now_v7(),
        adapter_id: adapter_id.clone(),
        workspace_id: workspace_id.clone(),
        project_root: project_root.clone(),
        continuity_id: continuity_id.clone(),
    };

    let result = match reducer::reduce_with_meta(current, event, None, None, false) {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(error = %error, "session start rejected by reducer");
            return Ok(Json(json!({
                "status": "rejected",
                "failure_class": "reducer_rejected",
                "reason": error.to_string(),
                "requested": {
                    "adapter_id": adapter_id,
                    "workspace_id": workspace_id,
                    "project_root": project_root,
                    "continuity_id": continuity_id,
                },
                "current_session": state.focusa.read().await.session.clone(),
            })));
        }
    };

    let new_state = result.new_state;
    for emitted in result.emitted_events {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event: emitted,
            correlation_id: Some("api:session:start".to_string()),
            origin: SignalOrigin::Adapter,
            machine_id: None,
            instance_id,
            session_id: new_state.session.as_ref().map(|session| session.session_id),
            thread_id: None,
            is_observation: false,
        };
        if let Err(error) = state.persistence.append_event(&entry) {
            tracing::warn!(error = %error, "failed to persist session start event");
        } else if let Ok(serialized) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(serialized);
        }
    }

    let session = new_state.session.clone();
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();
    // Keep /session/start hot for Pi daemon-recovery paths. The daemon adopts
    // this external mutation on its next action and writes the full snapshot;
    // the event log entry above preserves the session-start fact meanwhile.

    if let Some(session) = session {
        return Ok(Json(json!({
            "status": "accepted",
            "session_id": session.session_id,
            "adapter_id": session.adapter_id,
            "workspace_id": session.workspace_id,
            "project_root": session.project_root,
            "continuity_id": session.continuity_id,
            "materialized_by": "api_reducer_sync",
        })));
    }

    Ok(Json(json!({
        "status": "rejected",
        "failure_class": "session_unavailable",
        "message": "session start reducer completed without materializing a session",
    })))
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
    let reason = body.reason;
    let instance_id = body
        .instance_id
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let _guard = state.write_serial_lock.lock().await;
    let current = { state.focusa.read().await.clone() };
    if current
        .session
        .as_ref()
        .is_none_or(|session| session.status == SessionStatus::Closed)
    {
        return Ok(Json(json!({
            "status": "accepted",
            "materialized_by": "already_closed",
        })));
    }

    let event = FocusaEvent::SessionClosed {
        reason: reason.clone(),
    };
    let result = match reducer::reduce_with_meta(current, event, None, None, false) {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(error = %error, "session close rejected by reducer");
            return Ok(Json(json!({
                "status": "rejected",
                "failure_class": "reducer_rejected",
                "reason": error.to_string(),
            })));
        }
    };

    let new_state = result.new_state;
    for emitted in result.emitted_events {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event: emitted,
            correlation_id: Some("api:session:close".to_string()),
            origin: SignalOrigin::Adapter,
            machine_id: None,
            instance_id,
            session_id: new_state.session.as_ref().map(|session| session.session_id),
            thread_id: None,
            is_observation: false,
        };
        if let Err(error) = state.persistence.append_event(&entry) {
            tracing::warn!(error = %error, "failed to persist session close event");
        } else if let Ok(serialized) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(serialized);
        }
    }

    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(json!({
        "status": "accepted",
        "reason": reason,
        "materialized_by": "api_reducer_sync",
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/status/deep", get(status_deep))
        .route("/v1/state/dump", get(state_dump))
        .route("/v1/session/start", post(start_session))
        .route("/v1/session/resume", post(resume_session))
        .route("/v1/session/close", post(close_session))
}
