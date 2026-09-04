//! Session routes.
//!
//! GET  /v1/status        — daemon/session status (summary)
//! GET  /v1/state/dump    — full cognitive state (debug)
//! GET  /v1/session/discover — discover Pi sessions from filesystem (Spec98)
//! POST /v1/session/start — start a new session
//! POST /v1/session/resume — restore a previous session
//! POST /v1/session/close — close current session
//! POST /v1/session/bind — bind daemon trajectory to discovered session

use crate::routes::bounded::resource_mode_status;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::{NaiveDateTime, Utc};
use focusa_core::{
    reducer,
    types::{Action, EventLogEntry, FocusaEvent, SessionStatus, SignalOrigin},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Discovered Pi session from filesystem
#[derive(Debug, serde::Serialize)]
struct DiscoveredSession {
    agent: String,
    session_id: String,
    continuity_id: Option<String>,
    project_root: Option<String>,
    current_ask: Option<String>,
    last_activity: Option<String>,
    session_path: String,
}

type SessionResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn session_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": failure_class != "validation_rejected", "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry" }, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn session_invalid_uuid(value: &str) -> (StatusCode, Json<serde_json::Value>) {
    session_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid session_id: {value}"),
        "validation_rejected",
        "Session resume requires session_id to be a UUID.",
        "Send a valid session_id UUID before retrying unchanged.",
        "Likely stale Pi resume payload or non-UUID session handle.",
    )
}

fn session_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    session_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("session dispatch failed: {error}"),
        "daemon_unavailable",
        "Session resume action could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
    )
}

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

        let active_turn_id = focusa.active_turn.as_ref().map(|t| t.turn_id.clone());
        let assembled_chars = focusa
            .active_turn
            .as_ref()
            .and_then(|t| t.assembled_prompt.as_ref())
            .map(|s| s.len() as u64)
            .unwrap_or(0);
        let raw_input_chars = focusa
            .active_turn
            .as_ref()
            .and_then(|t| t.raw_user_input.as_ref())
            .map(|s| s.len() as u64)
            .unwrap_or(0);
        let active_turn_diagnostics_handle = active_turn_id
            .as_ref()
            .map(|turn_id| format!("runtime-diagnostics:active-turn:{turn_id}"));
        let active_turn_diagnostics_handle_status = if active_turn_diagnostics_handle.is_some() {
            "bounded_runtime_metadata_only"
        } else {
            "none"
        };

        let prompt_stats = json!({
            "last_assembled_chars": assembled_chars,
            "last_assembled_estimated_tokens": assembled_chars / 4,
            "active_turn_id": active_turn_id,
            "active_turn_diagnostics_handle": active_turn_diagnostics_handle,
            "active_turn_diagnostics_handle_status": active_turn_diagnostics_handle_status,
            "active_turn_diagnostics_fields": ["turn_id", "raw_input_chars", "assembled_chars", "estimated_tokens"],
            "raw_input_chars": raw_input_chars,
            "raw_user_input": Value::Null,
            "assembled_prompt": Value::Null,
            "authority_class": "runtime_correlation",
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
    let resource_mode = resource_mode_status();
    let rss_kb = resource_mode.rss_kb;
    let host_mem_available_kb = resource_mode.host_mem_available_kb;
    let rss_soft_mb = resource_mode.budget.rss_soft_mb;
    let rss_hard_mb = resource_mode.budget.rss_hard_mb;
    let degraded = resource_mode.mode != "normal";

    let mut payload = json!({
        "status": "ok",
        "route_tier": if include_deep { "cold" } else { "hot" },
        "summary_only": !include_deep,
        "resource_mode": resource_mode,
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
        "app_version": env!("CARGO_PKG_VERSION"),
        "runtime_process": {
            "current_pid": current_pid,
            "daemon_pids": Value::Null,
            "daemon_count": Value::Null,
            "duplicate_daemon_count": Value::Null,
            "single_daemon_ok": Value::Null,
        },
        "runtime_memory": {
            "rss_kb": rss_kb,
            "memory_budget_mb": rss_hard_mb,
            "rss_soft_mb": rss_soft_mb,
            "rss_hard_mb": rss_hard_mb,
            "budget_authority": "resource_mode",
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
    let temporal = focusa_core::temporal_clock::capture_operator_temporal_action_envelope();
    for emitted in result.emitted_events {
        let mut entry = EventLogEntry::with_temporal(
            emitted,
            SignalOrigin::Adapter,
            Some("api:session:start".to_string()),
            temporal.clone(),
        );
        entry.instance_id = instance_id;
        entry.session_id = new_state.session.as_ref().map(|session| session.session_id);
        if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
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
) -> SessionResult {
    let session_id = uuid::Uuid::parse_str(&body.session_id)
        .map_err(|_| session_invalid_uuid(&body.session_id))?;

    state
        .command_tx
        .send(Action::ResumeSession { session_id })
        .await
        .map_err(session_dispatch_failed)?;

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
    let temporal = focusa_core::temporal_clock::capture_operator_temporal_action_envelope();
    for emitted in result.emitted_events {
        let mut entry = EventLogEntry::with_temporal(
            emitted,
            SignalOrigin::Adapter,
            Some("api:session:close".to_string()),
            temporal.clone(),
        );
        entry.instance_id = instance_id;
        entry.session_id = new_state.session.as_ref().map(|session| session.session_id);
        if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
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

/// Discover sessions from filesystem (Pi, Claude Code, Codex, Letta, etc.)
/// Per Spec98: Multi-agent session discovery with project binding
async fn discover_sessions(Query(params): Query<DiscoverSessionsQuery>) -> Json<Value> {
    let agent_type = params.agent.as_deref().unwrap_or("all");
    let project_root = params.project_root.as_deref();
    let active_only = params.active_only.unwrap_or(false);

    let mut sessions = Vec::new();

    // Discover sessions for each agent type
    let agent_types: &[&str] = if agent_type == "all" {
        &["pi", "claude", "codex", "letta", "opencode", "claude-code"]
    } else {
        &[agent_type]
    };

    for &agent in agent_types {
        match agent {
            "pi" => {
                if let Some(pi_sessions) =
                    discover_agent_sessions("pi", "$HOME/.pi/agent/sessions").await
                {
                    sessions.extend(pi_sessions);
                }
            }
            "claude" | "claude-code" => {
                if let Some(s) = discover_agent_sessions("claude", "$HOME/.claude/sessions").await {
                    sessions.extend(s);
                }
                // Also check backups for recent sessions
                if let Some(s) = discover_claude_backups().await {
                    sessions.extend(s);
                }
            }
            "codex" => {
                if let Some(s) = discover_agent_sessions("codex", "$HOME/.codex/sessions").await {
                    sessions.extend(s);
                }
            }
            "letta" => {
                if let Some(s) = discover_agent_sessions("letta", "$HOME/.letta/sessions").await {
                    sessions.extend(s);
                }
            }
            "opencode" => {
                if let Some(s) =
                    discover_agent_sessions("opencode", "$HOME/.opencode/sessions").await
                {
                    sessions.extend(s);
                }
            }
            _ => {}
        }
    }

    // Filter by project_root if specified
    if let Some(root) = project_root {
        sessions.retain(|s| s.project_root.as_ref() == Some(&root.to_string()));
    }

    // Filter for active sessions if requested (last hour)
    if active_only {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        sessions.retain(|s| {
            s.last_activity
                .as_ref()
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc) > cutoff)
                .unwrap_or(false)
        });
    }

    // Sort by last_activity descending
    sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

    Json(json!({
        "sessions": sessions,
        "agent_types": agent_types,
        "count": sessions.len(),
        "active_only": active_only,
    }))
}

/// Generic agent session discovery - expands $HOME in path
async fn discover_agent_sessions(
    agent_type: &str,
    path_template: &str,
) -> Option<Vec<DiscoveredSession>> {
    let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let path_str = path_template.replace("$HOME", &home);
    let sessions_dir = PathBuf::from(&path_str);

    tracing::debug!("Discovering {} sessions at: {:?}", agent_type, sessions_dir);
    if !sessions_dir.exists() {
        return Some(Vec::new());
    }

    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Subdirectory: scan for session files
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.extension().is_some_and(|e| e == "jsonl")
                            && let Some(session) =
                                parse_generic_session(&sub_path, agent_type).await
                        {
                            sessions.push(session);
                        }
                    }
                }
            } else if path.extension().is_some_and(|e| e == "jsonl")
                && let Some(session) = parse_generic_session(&path, agent_type).await
            {
                sessions.push(session);
            }
        }
    }

    tracing::debug!("Discovered {} {} sessions", sessions.len(), agent_type);
    Some(sessions)
}

/// Parse a generic JSONL session file
async fn parse_generic_session(path: &PathBuf, agent_type: &str) -> Option<DiscoveredSession> {
    let content = fs::read_to_string(path).ok()?;
    if content.is_empty() {
        return None;
    }

    // Extract session_id from filename as fallback
    let fallback_session_id = path
        .file_stem()?
        .to_str()?
        .split('_')
        .next_back()?
        .to_string();

    // Find last non-empty line
    let last_line = content.lines().rfind(|l| !l.trim().is_empty())?;

    let json: Value = serde_json::from_str(last_line).ok()?;

    // Agent-specific parsing based on session format
    let (session_id, continuity_id, current_ask, project_root) = match agent_type {
        "pi" => parse_pi_session_data(&json, fallback_session_id),
        _ => parse_generic_session_data(&json, fallback_session_id),
    };

    let last_activity = extract_timestamp_from_path(path);

    Some(DiscoveredSession {
        agent: agent_type.to_string(),
        session_id,
        continuity_id,
        project_root,
        current_ask,
        last_activity,
        session_path: path.to_string_lossy().to_string(),
    })
}

/// Parse Pi session-specific data format
fn parse_pi_session_data(
    json: &Value,
    fallback_id: String,
) -> (String, Option<String>, Option<String>, Option<String>) {
    let data = match json.get("data") {
        Some(d) => d,
        None => {
            return (
                format!("pi-{}", fallback_id.chars().take(20).collect::<String>()),
                None,
                None,
                None,
            );
        }
    };
    let session_id = data
        .get("sessionId")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("pi-{}", fallback_id.chars().take(20).collect::<String>()));
    let continuity_id = data
        .get("continuityId")
        .and_then(|v| v.as_str())
        .map(String::from);
    let current_ask = data
        .get("currentAsk")
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.chars().take(200).collect());
    let project_root = data
        .get("currentAsk")
        .and_then(|v| v.get("projectRoot"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    (session_id, continuity_id, current_ask, project_root)
}

/// Parse generic session data format (Claude, Codex, etc.)
fn parse_generic_session_data(
    json: &Value,
    fallback_id: String,
) -> (String, Option<String>, Option<String>, Option<String>) {
    // Try various common session ID field names
    let session_id = json
        .get("sessionId")
        .or_else(|| json.get("session_id"))
        .or_else(|| json.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("gen-{}", fallback_id.chars().take(20).collect::<String>()));

    // Try various continuity/context field names
    let continuity_id = json
        .get("continuityId")
        .or_else(|| json.get("continuity_id"))
        .or_else(|| json.get("threadId"))
        .or_else(|| json.get("thread_id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    // Try various current task/goal field names
    let current_ask = json
        .get("currentAsk")
        .or_else(|| json.get("current_task"))
        .or_else(|| json.get("task"))
        .or_else(|| json.get("instruction"))
        .and_then(|v| {
            v.as_str()
                .or_else(|| v.get("text").and_then(|t| t.as_str()))
        })
        .map(|s| s.chars().take(200).collect());

    // Try various project root field names
    let project_root = json
        .get("projectRoot")
        .or_else(|| json.get("project_root"))
        .or_else(|| json.get("cwd"))
        .or_else(|| json.get("root"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    (session_id, continuity_id, current_ask, project_root)
}

fn parse_session_timestamp_from_filename(token: &str) -> Option<String> {
    let token = token.trim();
    let token = token
        .trim_end_matches(".json")
        .trim_end_matches(".jsonl")
        .trim_end_matches(".backup");
    let token = token.trim_end_matches('Z');

    if token.len() < 19 {
        return None;
    }

    let dt = &token[..19.min(token.len())];
    NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H-%M-%S")
        .ok()
        .map(|dt| format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")))
}

/// Extract ISO timestamp from filename
fn extract_timestamp_from_path(path: &Path) -> Option<String> {
    let token = path
        .file_name()
        .and_then(|n| n.to_str())?
        .split('_')
        .next()?;
    parse_session_timestamp_from_filename(token)
}

/// Discover Claude Code sessions from backups
async fn discover_claude_backups() -> Option<Vec<DiscoveredSession>> {
    let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let backups_dir = PathBuf::from(&home).join(".claude/backups");

    if !backups_dir.exists() {
        return Some(Vec::new());
    }

    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&backups_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|e| e.to_str().unwrap_or("").contains("backup"))
                && let Some(session) = parse_claude_backup(&path).await
            {
                sessions.push(session);
            }
        }
    }

    tracing::debug!("Discovered {} Claude backups", sessions.len());
    Some(sessions)
}

/// Parse Claude Code backup file
async fn parse_claude_backup(path: &PathBuf) -> Option<DiscoveredSession> {
    let content = fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    // Claude Code stores session in clientId or similar
    let session_id = json
        .get("clientId")
        .or_else(|| json.get("sessionId"))
        .and_then(|v| v.as_str())
        .map(|s| format!("claude-{}", s.chars().take(16).collect::<String>()))
        .unwrap_or_else(|| "claude-backup-unknown".to_string());

    // Extract project from Claude Code config
    let project_root = json
        .get("currentProject")
        .or_else(|| json.get("projectPath"))
        .or_else(|| json.get("workspaceRoot"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let last_activity = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.split("backup.").nth(1))
        .and_then(parse_session_timestamp_from_filename);

    Some(DiscoveredSession {
        agent: "claude-backup".to_string(),
        session_id,
        continuity_id: None,
        project_root,
        current_ask: None,
        last_activity,
        session_path: path.to_string_lossy().to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct DiscoverSessionsQuery {
    agent: Option<String>, // "pi", "claude", "codex", "letta", "opencode", "all"
    project_root: Option<String>, // Filter by project root
    active_only: Option<bool>, // Only return sessions from last hour
}

#[derive(Debug, Deserialize)]
struct BindSessionRequest {
    session_id: String,
    project_root: String,
    continuity_id: Option<String>,
    trajectory_scope: Option<String>, // "project" or "session"
}

/// Bind a discovered session to a trajectory scope
async fn bind_session_to_trajectory(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BindSessionRequest>,
) -> SessionResult {
    let focusa = state.focusa.read().await;
    let trajectory_scope = match body.trajectory_scope.as_deref().unwrap_or("project") {
        "session" => "session",
        "project" => "project",
        "" => "project",
        _ => {
            return Err(session_failure(
                StatusCode::BAD_REQUEST,
                "invalid trajectory_scope",
                "validation_rejected",
                "trajectory_scope must be 'project' or 'session'",
                "Submit trajectory_scope as 'project' or 'session'",
                "Use a valid trajectory_scope value",
            ));
        }
    };

    let continuity_id = body
        .continuity_id
        .unwrap_or_else(|| format!("session-{}", body.session_id));
    let continuity_id = continuity_id.trim().to_string();

    // Find or create trajectory for this session.
    let scope_key = if trajectory_scope == "session" {
        format!("{}:session:{}", body.project_root, body.session_id)
    } else {
        body.project_root.clone()
    };

    // Check if trajectory exists for this scope.
    let existing_trajectory = focusa.trajectory.records.iter().find(|r| {
        r.project_root.as_ref() == Some(&body.project_root)
            && r.continuity_id.as_ref() == Some(&continuity_id)
    });

    let result = json!({
        "session_id": body.session_id,
        "project_root": body.project_root,
        "continuity_id": continuity_id,
        "trajectory_scope": scope_key,
        "trajectory_scope_mode": trajectory_scope,
        "trajectory_exists": existing_trajectory.is_some(),
        "bound": true,
    });

    Ok(Json(result))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/status/deep", get(status_deep))
        .route("/v1/state/dump", get(state_dump))
        .route("/v1/session/discover", get(discover_sessions))
        .route("/v1/session/start", post(start_session))
        .route("/v1/session/resume", post(resume_session))
        .route("/v1/session/close", post(close_session))
        .route("/v1/session/bind", post(bind_session_to_trajectory))
}

#[cfg(test)]
mod tests {
    use super::{
        discover_agent_sessions, parse_claude_backup, parse_generic_session,
        parse_generic_session_data, parse_pi_session_data, parse_session_timestamp_from_filename,
    };
    use chrono::{Duration, Utc};
    use serde_json::json;
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_home(prefix: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("focusa-session-mux-{prefix}-{stamp}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_jsonl(path: &Path, payload: &str) {
        let mut f = File::create(path).unwrap();
        f.write_all(payload.as_bytes()).unwrap();
    }

    #[test]
    fn parse_session_timestamp_from_pi_filename() {
        let now = Utc::now() - Duration::seconds(90);
        let token = now.format("%Y-%m-%dT%H-%M-%S").to_string();
        let expected = format!("{}Z", now.format("%Y-%m-%dT%H:%M:%S"));
        let ts = parse_session_timestamp_from_filename(&format!("{token}Z"));
        assert_eq!(ts.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn parse_session_timestamp_from_short_or_invalid_filename() {
        let now = Utc::now().format("%Y-%m-%dT%H").to_string();
        assert!(parse_session_timestamp_from_filename("not-a-ts").is_none());
        assert_eq!(
            parse_session_timestamp_from_filename(&format!("{now}-02")),
            None
        );
    }

    #[test]
    fn parse_generic_session_data_fields() {
        let json = json!({
            "sessionId": "codex-abc-1",
            "continuityId": "cont-codex-1",
            "currentAsk": {"text": "continue docs work"},
            "projectRoot": "/tmp/proj-codex"
        });

        let (session_id, continuity_id, current_ask, project_root) =
            parse_generic_session_data(&json, "fallback-x".to_string());

        assert_eq!(session_id, "codex-abc-1");
        assert_eq!(continuity_id.as_deref(), Some("cont-codex-1"));
        assert_eq!(current_ask.as_deref(), Some("continue docs work"));
        assert_eq!(project_root.as_deref(), Some("/tmp/proj-codex"));
    }

    #[test]
    fn parse_generic_session_data_variants_and_fallback() {
        let json = json!({
            "thread_id": "cont-letta",
            "task": "sync endpoints",
            "root": "/tmp/proj-letta",
            "uuid": "should-not-use"
        });

        let (session_id, continuity_id, current_ask, project_root) =
            parse_generic_session_data(&json, "fallback-42".to_string());

        assert_eq!(session_id, "gen-fallback-42");
        assert_eq!(continuity_id.as_deref(), Some("cont-letta"));
        assert_eq!(current_ask.as_deref(), Some("sync endpoints"));
        assert_eq!(project_root.as_deref(), Some("/tmp/proj-letta"));
    }

    #[test]
    fn parse_pi_session_data_fallback_when_data_missing() {
        let json = json!({"session": "noop"});
        let (session_id, continuity_id, current_ask, project_root) =
            parse_pi_session_data(&json, "fallback-pi".to_string());

        assert_eq!(session_id, "pi-fallback-pi");
        assert!(continuity_id.is_none());
        assert!(current_ask.is_none());
        assert!(project_root.is_none());
    }

    #[tokio::test]
    async fn parse_generic_and_pi_file_scan_supports_multi_agent_formats() {
        let home = temp_home("file_scan");
        let pi_root = home.join(".pi/agent/sessions");
        let codex_root = home.join(".codex/sessions");
        let letta_root = home.join(".letta/sessions");
        fs::create_dir_all(&pi_root).unwrap();
        fs::create_dir_all(&codex_root).unwrap();
        fs::create_dir_all(&letta_root).unwrap();

        let pi_subdir = pi_root.join("session-1");
        fs::create_dir_all(&pi_subdir).unwrap();
        let token = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        write_jsonl(
            &pi_subdir.join(format!("{token}_pi.jsonl")),
            "{\"data\":{\"sessionId\":\"pi-sess-1\",\"continuityId\":\"cont-pi-1\",\"currentAsk\":{\"text\":\"pi task\",\"projectRoot\":\"/tmp/proj-pi\"}}}\n\n",
        );

        write_jsonl(
            &codex_root.join(format!("{token}_codex.jsonl")),
            "{\"sessionId\":\"codex-sess-1\",\"threadId\":\"cont-codex-1\",\"current_task\":\"codex task\",\"cwd\":\"/tmp/proj-codex\"}\n",
        );

        write_jsonl(
            &letta_root.join(format!("{token}_letta.jsonl")),
            "{\"id\":\"letta-sess-1\",\"continuity_id\":\"cont-letta-1\",\"instruction\":\"letta task\",\"project_root\":\"/tmp/proj-letta\"}\n",
        );

        let pi_sessions = discover_agent_sessions("pi", pi_root.to_str().unwrap())
            .await
            .unwrap();
        let codex_sessions = discover_agent_sessions("codex", codex_root.to_str().unwrap())
            .await
            .unwrap();
        let letta_sessions = discover_agent_sessions("letta", letta_root.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(pi_sessions.len(), 1);
        assert_eq!(codex_sessions.len(), 1);
        assert_eq!(letta_sessions.len(), 1);

        assert_eq!(pi_sessions[0].agent, "pi");
        assert_eq!(pi_sessions[0].session_id, "pi-sess-1");
        assert_eq!(pi_sessions[0].continuity_id.as_deref(), Some("cont-pi-1"));

        assert_eq!(codex_sessions[0].agent, "codex");
        assert_eq!(codex_sessions[0].session_id, "codex-sess-1");
        assert_eq!(
            codex_sessions[0].continuity_id.as_deref(),
            Some("cont-codex-1")
        );

        assert_eq!(letta_sessions[0].agent, "letta");
        assert_eq!(letta_sessions[0].session_id, "letta-sess-1");
        assert_eq!(
            letta_sessions[0].continuity_id.as_deref(),
            Some("cont-letta-1")
        );

        let _ = fs::remove_dir_all(&home);
    }

    #[tokio::test]
    async fn parse_generic_session_from_file_handles_fallback_and_current_ask() {
        let home = temp_home("file_parse");
        let pi_root = home.join(".pi/agent/sessions/single");
        fs::create_dir_all(&pi_root).unwrap();

        let token = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let file_path = pi_root.join(format!("{token}_single.jsonl"));
        write_jsonl(
            &file_path,
            "{\"data\":{\"sessionId\":\"pi-single-1\",\"continuityId\":\"cont-single\",\"currentAsk\":{\"text\":\"single task\",\"projectRoot\":\"/tmp/proj-single\"}}}\n",
        );

        let session = parse_generic_session(&file_path, "pi").await.unwrap();
        assert_eq!(session.session_id, "pi-single-1");
        assert_eq!(session.continuity_id.as_deref(), Some("cont-single"));
        assert_eq!(session.current_ask.as_deref(), Some("single task"));
        assert_eq!(session.project_root.as_deref(), Some("/tmp/proj-single"));
        assert!(session.last_activity.is_some());

        let _ = fs::remove_dir_all(&home);
    }

    #[tokio::test]
    async fn parse_claude_backup_file_parse() {
        let home = temp_home("claude_backup");
        let backup_stamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let expected_backup =
            parse_session_timestamp_from_filename(&format!("{backup_stamp}-001Z"));
        let file_path = home.join(format!("foo.backup.{backup_stamp}-001Z"));
        write_jsonl(
            &file_path,
            "{\"clientId\":\"cl-client\",\"currentProject\":\"/tmp/proj-claude\",\"projectPath\":\"/tmp/project\"}\n",
        );

        let parsed = parse_claude_backup(&file_path).await.unwrap();
        assert_eq!(parsed.session_id, "claude-cl-client");
        assert_eq!(parsed.project_root.as_deref(), Some("/tmp/proj-claude"));
        assert_eq!(parsed.last_activity, expected_backup);

        let _ = fs::remove_dir_all(&home);
    }
}
