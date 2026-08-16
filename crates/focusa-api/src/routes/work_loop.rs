//! Continuous work loop control/status routes.

use crate::routes::bounded::{record_json_response_size, resource_mode_status};
use crate::routes::permissions::{forbid, permission_context};
use crate::scope::ScopeContext;
use crate::server::{AppState, WriterLease};
use axum::extract::{FromRequestParts, Query, State};
use axum::http::{HeaderMap, StatusCode, request::Parts};
use axum::response::{IntoResponse, Response};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use focusa_core::scoped_state::WorkstreamKey;
use focusa_core::tool_result::{ToolResultV1, ToolStatus};
use focusa_core::types::{
    Action, BlockerClass, EventLogEntry, FocusaEvent, FocusaState, ProjectRunId, SignalOrigin,
    SpecLinkedTaskPacket, TaskClass, WorkLoopOutcomeStatus, WorkLoopPolicy,
    WorkLoopPolicyOverrides, WorkLoopPreset, WorkLoopStatus,
};
use focusa_core::work_item::{
    BdAdapter, EvidenceCitation, NoneAdapter, ProviderAdapter, WorkItemProvider, WorkItemQuery,
    WorkItemReadiness, WorkItemRef, evaluate_readiness,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{Duration, sleep, timeout};
use uuid::Uuid;

const WRITER_HEADER: &str = "x-focusa-writer-id";
const FENCING_HEADER: &str = "x-focusa-fencing-token";
const WRITER_LEASE_TTL_MS: i64 = 30_000;
const APPROVAL_HEADER: &str = "x-focusa-approval";
const WORK_LOOP_STATUS_SCHEMA: &str = "focusa.work_loop_status.v3";
const WORK_LOOP_REPLAY_SCHEMA: &str = "focusa.work_loop_replay.v2";
const WORK_LOOP_TYPED_STATES: [&str; 8] = [
    "absent",
    "unavailable",
    "stale",
    "unsupported",
    "blocked",
    "exhausted",
    "zero",
    "healthy",
];

#[derive(Clone)]
struct WorkLoopScope(WorkstreamKey);

struct WorkLoopScopeRejection {
    status: StatusCode,
    body: Value,
}

impl IntoResponse for WorkLoopScopeRejection {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

fn work_loop_scope_matches(
    key: &WorkstreamKey,
    active_root: &str,
    active_continuity: &str,
) -> bool {
    key.root_scope.scope_kind == focusa_core::scoped_state::ScopeKind::Project
        && key
            .root_scope
            .root_path
            .to_string_lossy()
            .trim_end_matches('/')
            == active_root
        && key.continuity_id == active_continuity
}

fn canonical_workpoint_id_for_scope_and_item(
    focusa: &FocusaState,
    key: &WorkstreamKey,
    work_item_id: Option<&str>,
) -> Option<focusa_core::types::WorkpointId> {
    focusa.workpoint.records.iter().find_map(|record| {
        let scope_matches = record.canonical
            && record.status == focusa_core::types::WorkpointStatus::Active
            && record.project_root.as_deref().is_some_and(|root| {
                record
                    .continuity_id
                    .as_deref()
                    .is_some_and(|continuity| work_loop_scope_matches(key, root, continuity))
            });
        let item_matches = work_item_id
            .map(|expected| record.work_item_id.as_deref() == Some(expected))
            .unwrap_or(true);
        (scope_matches && item_matches).then_some(record.workpoint_id)
    })
}

fn canonical_workpoint_exists_for_scope(focusa: &FocusaState, key: &WorkstreamKey) -> bool {
    canonical_workpoint_id_for_scope_and_item(focusa, key, None).is_some()
}

impl FromRequestParts<Arc<AppState>> for WorkLoopScope {
    type Rejection = WorkLoopScopeRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let request_scope = ScopeContext::from_request_parts(parts, state)
            .await
            .map_err(|error| WorkLoopScopeRejection {
                status: StatusCode::BAD_REQUEST,
                body: json!({
                    "schema": "focusa.work_loop_scope_rejection.v1",
                    "status": "blocked",
                    "failure_class": "scope_mismatch",
                    "error": format!("{error:?}"),
                }),
            })?;
        let key =
            request_scope
                .require_workstream_key()
                .map_err(|error| WorkLoopScopeRejection {
                    status: StatusCode::BAD_REQUEST,
                    body: json!({
                        "schema": "focusa.work_loop_scope_rejection.v1",
                        "status": "blocked",
                        "failure_class": "scope_mismatch",
                        "error": error,
                    }),
                })?;
        let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &key).await;
        if !canonical_workpoint_exists_for_scope(&focusa, &key) {
            return Err(WorkLoopScopeRejection {
                status: StatusCode::CONFLICT,
                body: json!({
                    "schema": "focusa.work_loop_scope_rejection.v1",
                    "status": "blocked",
                    "failure_class": "scope_mismatch",
                    "error": "canonical Workpoint for request WorkstreamKey is required",
                    "requested_scope": key,
                }),
            });
        }
        if focusa
            .work_loop
            .execution_scope
            .as_ref()
            .is_some_and(|active| active != &key)
        {
            return Err(WorkLoopScopeRejection {
                status: StatusCode::CONFLICT,
                body: json!({
                    "schema": "focusa.work_loop_scope_rejection.v1",
                    "status": "blocked",
                    "failure_class": "scope_mismatch",
                    "error": "request WorkstreamKey does not match active Work Loop execution scope",
                    "requested_scope": key,
                    "active_execution_scope": focusa.work_loop.execution_scope,
                }),
            });
        }
        Ok(Self(key))
    }
}

fn git_safe_directory_arg(root_hint: &str) -> String {
    format!("safe.directory={}", root_hint)
}

fn work_loop_scope_root(focusa: &focusa_core::types::FocusaState) -> Option<PathBuf> {
    focusa
        .work_loop
        .execution_scope
        .as_ref()
        .map(|scope| scope.root_scope.root_path.clone())
}

fn request_scope_root(scope: &WorkLoopScope) -> PathBuf {
    scope.0.root_scope.root_path.clone()
}

fn pi_rpc_bin() -> String {
    std::env::var("FOCUSA_PI_BIN").unwrap_or_else(|_| "pi".to_string())
}

const PI_HEADLESS_VITAL_INFO_PROMPT_MODE: &str = "warn_only";

fn extension_ui_response(request: &Value, authorized_project_root: &Path) -> Option<Value> {
    if request.get("type").and_then(Value::as_str) != Some("extension_ui_request") {
        return None;
    }
    let method = request.get("method").and_then(Value::as_str)?;
    if !matches!(method, "select" | "confirm" | "input" | "editor") {
        return None;
    }
    let id = request.get("id")?.clone();
    if method == "select"
        && let Some(options) = request.get("options").and_then(Value::as_array)
    {
        let root = authorized_project_root.to_string_lossy();
        let safe_matches = options
            .iter()
            .filter_map(Value::as_str)
            .filter(|option| {
                option.contains(root.as_ref())
                    || option.to_ascii_lowercase().contains("skip")
                    || option.to_ascii_lowercase().contains("leave unchanged")
            })
            .collect::<Vec<_>>();
        if safe_matches.len() == 1 {
            return Some(json!({
                "type": "extension_ui_response",
                "id": id,
                "value": safe_matches[0]
            }));
        }
    }
    Some(json!({
        "type": "extension_ui_response",
        "id": id,
        "cancelled": true
    }))
}

fn pi_focusa_api_base_url(api_bind: &str) -> String {
    let child_host = api_bind
        .strip_prefix("0.0.0.0:")
        .map(|port| format!("127.0.0.1:{port}"))
        .or_else(|| {
            api_bind
                .strip_prefix("[::]:")
                .map(|port| format!("127.0.0.1:{port}"))
        })
        .unwrap_or_else(|| api_bind.to_string());
    format!("http://{child_host}/v1")
}

fn pi_rpc_node_bin_dir() -> Option<String> {
    std::env::var("FOCUSA_NODE_BIN_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[cfg(unix)]
fn configure_pi_rpc_process_group(cmd: &mut Command) {
    cmd.process_group(0);
}

#[cfg(windows)]
fn configure_pi_rpc_process_group(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    cmd.as_std_mut().creation_flags(CREATE_NEW_PROCESS_GROUP);
}

#[cfg(not(any(unix, windows)))]
fn configure_pi_rpc_process_group(_cmd: &mut Command) {}

#[cfg(unix)]
async fn terminate_pi_rpc_child(child: &mut Child, process_group_id: u32) {
    let pgid = format!("-{process_group_id}");
    let _ = Command::new("kill").args(["-TERM", &pgid]).status().await;
    if timeout(Duration::from_secs(2), child.wait()).await.is_ok() {
        return;
    }
    let _ = Command::new("kill").args(["-KILL", &pgid]).status().await;
    let _ = timeout(Duration::from_secs(2), child.wait()).await;
}

#[cfg(windows)]
async fn terminate_pi_rpc_child(child: &mut Child, process_group_id: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &process_group_id.to_string(), "/T", "/F"])
        .status()
        .await;
    let _ = timeout(Duration::from_secs(2), child.wait()).await;
}

#[cfg(not(any(unix, windows)))]
async fn terminate_pi_rpc_child(child: &mut Child, _process_group_id: u32) {
    let _ = child.kill().await;
}

#[cfg(unix)]
fn spawn_pi_rpc_parent_watchdog(child_pid: u32) {
    let parent_pid = std::process::id();
    let script = r#"
while kill -0 "$1" 2>/dev/null && kill -0 "$2" 2>/dev/null; do sleep 0.2; done
/bin/kill -TERM "-$2" 2>/dev/null || true
sleep 2
/bin/kill -KILL "-$2" 2>/dev/null || true
"#;
    let _ = std::process::Command::new("sh")
        .args([
            "-c",
            script,
            "focusa-pi-watchdog",
            &parent_pid.to_string(),
            &child_pid.to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(not(unix))]
fn spawn_pi_rpc_parent_watchdog(_child_pid: u32) {}

fn bounded_orchestration_authority_payload() -> Value {
    json!({
        "authority_plane": "bounded_orchestration",
        "canonical": false,
        "focus_state_authority": false,
        "writer_ownership_required": true,
        "operator_controls": ["pause", "resume", "stop", "preflight", "approval_header_for_sensitive_actions"],
        "promotion_boundary": "orchestration may select/execute bounded work but must checkpoint/evidence/promote before cognition authority changes",
    })
}

fn supervisor_perf_payload(state: &AppState) -> Value {
    let perf = &state.supervisor_perf;
    json!({
        "supervisor_ticks_total": perf.ticks_total.load(Ordering::Relaxed),
        "driver_start_attempts": perf.driver_start_attempts.load(Ordering::Relaxed),
        "driver_stop_attempts": perf.driver_stop_attempts.load(Ordering::Relaxed),
        "dispatch_attempts": perf.dispatch_attempts.load(Ordering::Relaxed),
        "dispatch_skipped_disallowed": perf.dispatch_skipped_disallowed.load(Ordering::Relaxed),
        "dispatch_recovery_restarts": perf.dispatch_recovery_restarts.load(Ordering::Relaxed),
        "background_throttled_ticks": perf.background_throttled_ticks.load(Ordering::Relaxed),
    })
}

#[derive(Debug, Deserialize)]
struct WorkLoopStatusQuery {
    #[serde(default)]
    summary_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct EnableWorkLoopRequest {
    pub project_run_id: Option<ProjectRunId>,
    pub preset: Option<WorkLoopPreset>,
    pub policy_overrides: Option<WorkLoopPolicyOverrides>,
    pub root_work_item_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReasonRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResumeWorkLoopRequest {
    pub reason: Option<String>,
    #[serde(default)]
    pub renew_budget: bool,
    pub policy_overrides: Option<WorkLoopPolicyOverrides>,
}

#[derive(Debug, Deserialize)]
pub struct CheckpointRequest {
    pub checkpoint_id: Option<focusa_core::types::CheckpointId>,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectNextRequest {
    pub parent_work_item_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DelegationRequest {
    pub delegate_id: String,
    pub scope: String,
    pub amendment_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PauseFlagsRequest {
    pub destructive_confirmation_required: bool,
    pub governance_decision_pending: bool,
    pub operator_override_active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionAttachRequest {
    pub adapter: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DecisionContextRequest {
    pub current_ask: Option<String>,
    pub ask_kind: Option<String>,
    pub scope_kind: Option<String>,
    pub carryover_policy: Option<String>,
    pub excluded_context_reason: Option<String>,
    pub excluded_context_labels: Option<Vec<String>>,
    pub source_turn_id: Option<String>,
    pub operator_steering_detected: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PiDriverStartRequest {
    pub cwd: Option<String>,
    pub models: Option<String>,
    pub resume_session: Option<String>,
    pub session_dir: Option<String>,
    pub session_name: Option<String>,
    pub workpoint_id: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PiDriverPromptRequest {
    pub message: String,
    pub streaming_behavior: Option<String>,
}

fn agent_execution_tool_result(summary: &str, side_effect: &str) -> Value {
    let mut result = ToolResultV1::success(ToolStatus::Accepted, summary);
    result.tool = Some("focusa_agent_execution_adapter".to_string());
    result.family = Some("work_loop".to_string());
    result.side_effects = vec![side_effect.to_string()];
    serde_json::to_value(result)
        .unwrap_or_else(|_| json!({"schema": "focusa.tool_result.v1", "ok": true}))
}

fn configure_pi_rpc_invocation(command: &mut Command, request: &PiDriverStartRequest) {
    command.args(["--mode", "rpc"]);
    if let Some(models) = request.models.as_deref() {
        command.args(["--models", models]);
    }
    if let Some(resume_session) = request.resume_session.as_deref() {
        command.args(["--session", resume_session]);
    }
    if let Some(session_dir) = request.session_dir.as_deref() {
        command.args(["--session-dir", session_dir]);
    }
    if let Some(session_name) = request.session_name.as_deref() {
        command.args(["--name", session_name]);
    }
}

#[derive(Debug, Deserialize)]
pub struct TransportEventRequest {
    pub sequence: u64,
    pub kind: String,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub summary: Option<String>,
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
}

fn conflict(
    message: impl Into<String>,
    active_writer: Option<String>,
) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "error": message.into(),
            "active_writer": active_writer,
        })),
    )
}

fn work_loop_failure(
    http_status: StatusCode,
    action: &str,
    failure_class: &str,
    why: String,
) -> (StatusCode, Json<Value>) {
    let next_tools = json!([
        "focusa_work_loop_writer_status",
        "focusa_work_loop_status",
        "focusa_tool_doctor"
    ]);
    let recovery_hint = "Check writer/status first, then retry the work-loop mutation only if dispatch health and writer ownership are clear.";
    let misuse_hint = "Likely daemon command channel, writer ownership, Pi RPC dependency, or out-of-order work-loop mutation issue.";
    let retry_safe = !matches!(
        failure_class,
        "validation_rejected"
            | "not_found"
            | "permission_denied"
            | "writer_conflict"
            | "approval_required"
    );
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "error": why,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools,
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "failure_class": failure_class,
                    "canonical": false,
                    "degraded": true,
                    "summary": format!("work-loop {action} blocked"),
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools,
                    "error": {"code": failure_class, "message": why}
                }
            }
        })),
    )
}

fn work_loop_dispatch_failed(
    action: &str,
    err: impl std::fmt::Display,
) -> (StatusCode, Json<Value>) {
    work_loop_failure(
        StatusCode::SERVICE_UNAVAILABLE,
        action,
        "daemon_unavailable",
        format!("dispatch channel unavailable for {action}: {err}"),
    )
}

fn work_loop_dispatch_timeout(action: &str) -> (StatusCode, Json<Value>) {
    work_loop_failure(
        StatusCode::ACCEPTED,
        action,
        "resource_exhausted",
        format!(
            "work-loop dispatch timed out before enqueue for {action}; command backlog may be saturated"
        ),
    )
}

async fn send_work_loop_action(
    state: &Arc<AppState>,
    action_name: &str,
    action: Action,
) -> Result<(), (StatusCode, Json<Value>)> {
    match tokio::time::timeout(Duration::from_millis(1500), state.command_tx.send(action)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(work_loop_dispatch_failed(action_name, error)),
        Err(_) => Err(work_loop_dispatch_timeout(action_name)),
    }
}

fn work_loop_pi_spawn_failed(err: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    work_loop_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        "pi_rpc_start",
        "daemon_unavailable",
        format!("failed to spawn pi rpc: {err}"),
    )
}

fn normalize_partition_segment(value: impl AsRef<str>, fallback: &str) -> String {
    let cleaned = value.as_ref().trim();
    if cleaned.is_empty() {
        fallback.to_string()
    } else {
        cleaned.replace('|', "_")
    }
}

fn writer_claim_key_for_partition(
    project_root: &str,
    continuity_id: &str,
    work_item_id: &str,
) -> String {
    if project_root.trim().is_empty() || continuity_id.trim().is_empty() {
        return "blocked:canonical_workpoint_scope_required".to_string();
    }
    if work_item_id.trim().is_empty() {
        return "blocked:active_work_item_required".to_string();
    }
    format!(
        "project:{}|workstream:{}|work_item:{}",
        normalize_partition_segment(project_root, "blocked"),
        normalize_partition_segment(continuity_id, "blocked"),
        normalize_partition_segment(work_item_id, "blocked"),
    )
}

fn writer_claim_key_from_scope(
    scope: &WorkLoopScope,
    focusa: &focusa_core::types::FocusaState,
) -> String {
    let Some(work_item_id) = focusa
        .work_loop
        .execution_work_item_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
    else {
        return "blocked:active_work_item_required".to_string();
    };
    writer_claim_key_for_partition(
        scope.0.root_scope.root_path.to_string_lossy().as_ref(),
        &scope.0.continuity_id,
        work_item_id,
    )
}

fn require_authoritative_claim_key(key: String) -> Result<String, (StatusCode, Json<Value>)> {
    if key.starts_with("blocked:") {
        Err(work_loop_failure(
            StatusCode::CONFLICT,
            "writer_claim",
            "scope_mismatch",
            key.trim_start_matches("blocked:").replace('_', " "),
        ))
    } else {
        Ok(key)
    }
}

async fn writer_claim_key(scope: &WorkLoopScope, state: &Arc<AppState>) -> String {
    let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
    writer_claim_key_from_scope(scope, &focusa)
}

fn active_writer_lease_for_key(
    claims: &std::collections::HashMap<String, WriterLease>,
    key: &str,
    now: DateTime<Utc>,
) -> Option<WriterLease> {
    claims
        .get(key)
        .filter(|lease| lease.expires_at > now)
        .cloned()
}

fn active_writer_for_key(
    claims: &std::collections::HashMap<String, WriterLease>,
    key: &str,
    now: DateTime<Utc>,
) -> Option<String> {
    active_writer_lease_for_key(claims, key, now).map(|lease| lease.writer_id)
}

fn active_writer_compat(
    claims: &std::collections::HashMap<String, WriterLease>,
    key: &str,
    now: DateTime<Utc>,
) -> Option<String> {
    active_writer_for_key(claims, key, now)
}

fn writer_id_from_headers(headers: &HeaderMap) -> Result<String, (StatusCode, Json<Value>)> {
    headers
        .get(WRITER_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| bad_request(format!("missing required header: {WRITER_HEADER}")))
}

fn fencing_token_from_headers(headers: &HeaderMap) -> Result<u64, (StatusCode, Json<Value>)> {
    headers
        .get(FENCING_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|token| *token > 0)
        .ok_or_else(|| bad_request(format!("missing or invalid header: {FENCING_HEADER}")))
}

fn writer_lease_expiry(now: DateTime<Utc>) -> DateTime<Utc> {
    now + chrono::Duration::milliseconds(WRITER_LEASE_TTL_MS)
}

fn acquire_writer_for_key(
    claims: &mut std::collections::HashMap<String, WriterLease>,
    key: String,
    writer_id: String,
    fencing_token: u64,
    now: DateTime<Utc>,
) -> Result<WriterLease, (StatusCode, Json<Value>)> {
    if let Some(existing) = claims.get_mut(&key) {
        if existing.expires_at > now && existing.writer_id != writer_id {
            return Err(conflict(
                "continuous work loop partition already leased by another writer",
                Some(existing.writer_id.clone()),
            ));
        }
        if existing.expires_at > now {
            existing.renewed_at = now;
            existing.expires_at = writer_lease_expiry(now);
            return Ok(existing.clone());
        }
    }

    let lease = WriterLease {
        writer_id,
        fencing_token,
        acquired_at: now,
        renewed_at: now,
        expires_at: writer_lease_expiry(now),
    };
    claims.insert(key, lease.clone());
    Ok(lease)
}

fn validate_and_renew_writer_for_key(
    claims: &mut std::collections::HashMap<String, WriterLease>,
    key: &str,
    writer_id: &str,
    fencing_token: u64,
    now: DateTime<Utc>,
) -> Result<WriterLease, (StatusCode, Json<Value>)> {
    let Some(lease) = claims.get_mut(key) else {
        return Err(conflict(
            "continuous work loop partition has no active writer lease",
            None,
        ));
    };
    if lease.expires_at <= now {
        return Err(conflict(
            "continuous work loop writer lease expired; reacquire through enable",
            Some(lease.writer_id.clone()),
        ));
    }
    if lease.writer_id != writer_id || lease.fencing_token != fencing_token {
        return Err(conflict(
            "continuous work loop mutation rejected by fencing token",
            Some(lease.writer_id.clone()),
        ));
    }
    lease.renewed_at = now;
    lease.expires_at = writer_lease_expiry(now);
    Ok(lease.clone())
}

fn require_approval(headers: &HeaderMap, reason: &str) -> Result<(), (StatusCode, Json<Value>)> {
    let approved = headers
        .get(APPROVAL_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .map(|v| matches!(v, "true" | "1" | "approved"))
        .unwrap_or(false);
    if approved {
        Ok(())
    } else {
        Err((
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({
                "error": "explicit approval required",
                "reason": reason,
                "required_header": APPROVAL_HEADER,
            })),
        ))
    }
}

async fn ensure_writer_claim_for_work_item(
    scope: &WorkLoopScope,
    state: &Arc<AppState>,
    headers: &HeaderMap,
    work_item_id: &str,
) -> Result<WriterLease, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    if work_item_id.trim().is_empty() {
        return Err(bad_request("work item id is required for writer claim"));
    }
    let key = require_authoritative_claim_key(writer_claim_key_for_partition(
        scope.0.root_scope.root_path.to_string_lossy().as_ref(),
        &scope.0.continuity_id,
        work_item_id,
    ))?;
    let token = state
        .next_writer_fencing_token
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    let mut claims = state.writer_claims.write().await;
    acquire_writer_for_key(&mut claims, key, writer_id, token, Utc::now())
}

async fn ensure_writer_claim(
    scope: &WorkLoopScope,
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<WriterLease, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    let fencing_token = fencing_token_from_headers(headers)?;
    let key = require_authoritative_claim_key(writer_claim_key(scope, state).await)?;
    let mut claims = state.writer_claims.write().await;
    validate_and_renew_writer_for_key(&mut claims, &key, &writer_id, fencing_token, Utc::now())
}

async fn release_writer_claim(
    scope: &WorkLoopScope,
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<WriterLease>, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    let fencing_token = fencing_token_from_headers(headers)?;
    let key = require_authoritative_claim_key(writer_claim_key(scope, state).await)?;
    let mut claims = state.writer_claims.write().await;
    validate_and_renew_writer_for_key(&mut claims, &key, &writer_id, fencing_token, Utc::now())?;
    Ok(claims.remove(&key))
}

async fn ensure_claimed_writer_matches_for_context(
    scope: &WorkLoopScope,
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<WriterLease>, (StatusCode, Json<Value>)> {
    let key = require_authoritative_claim_key(writer_claim_key(scope, state).await)?;
    let has_claim = {
        let claims = state.writer_claims.read().await;
        claims.contains_key(&key)
    };
    if !has_claim {
        return Ok(None);
    }
    ensure_writer_claim(scope, state, headers).await.map(Some)
}

async fn worktree_status_snapshot(project_root: &Path) -> Value {
    let project_root_hint = project_root.to_string_lossy().to_string();
    let safe_dir = git_safe_directory_arg(&project_root_hint);
    let top = match Command::new("git")
        .args(["-c", safe_dir.as_str(), "rev-parse", "--show-toplevel"])
        .current_dir(project_root)
        .output()
        .await
    {
        Ok(top) if top.status.success() => top,
        Ok(top) => {
            return json!({
                "git_available": true,
                "in_worktree": false,
                "clean": false,
                "repo_root_hint": project_root_hint,
                "error": String::from_utf8_lossy(&top.stderr).trim().to_string(),
            });
        }
        Err(e) => {
            return json!({
                "git_available": false,
                "in_worktree": false,
                "clean": false,
                "error": e.to_string(),
            });
        }
    };

    let repo_root = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let status = match Command::new("git")
        .args(["-c", safe_dir.as_str(), "status", "--porcelain"])
        .current_dir(&repo_root)
        .output()
        .await
    {
        Ok(status) if status.status.success() => status,
        Ok(_) => {
            return json!({
                "git_available": true,
                "in_worktree": true,
                "clean": false,
                "repo_root": repo_root,
                "error": "git status unsuccessful",
            });
        }
        Err(e) => {
            return json!({
                "git_available": true,
                "in_worktree": true,
                "clean": false,
                "repo_root": repo_root,
                "error": e.to_string(),
            });
        }
    };

    let dirty = String::from_utf8_lossy(&status.stdout)
        .lines()
        .take(10)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let diff_stat = Command::new("git")
        .args(["-c", safe_dir.as_str(), "diff", "--stat"])
        .current_dir(&repo_root)
        .output()
        .await
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    json!({
        "git_available": true,
        "in_worktree": true,
        "clean": dirty.is_empty(),
        "repo_root": repo_root,
        "sample_changes": dirty,
        "diff_stat": diff_stat,
        "forbidden_without_explicit_approval": ["git reset --hard", "git clean", "git restore"],
    })
}

async fn provider_neutral_readiness(
    state: &Arc<AppState>,
    project_root: &Path,
    parent_work_item_id: Option<&str>,
) -> Result<(WorkItemProvider, WorkItemReadiness), String> {
    let configured_provider = {
        let focusa = state.focusa.read().await;
        focusa.work_loop.policy.work_item_provider
    };
    let provider =
        if configured_provider == WorkItemProvider::None && project_root.join(".beads").exists() {
            WorkItemProvider::Bd
        } else {
            configured_provider
        };
    let adapter: Arc<dyn ProviderAdapter> = match provider {
        WorkItemProvider::Bd => Arc::new(BdAdapter::new()),
        WorkItemProvider::None => Arc::new(NoneAdapter::new()),
        unsupported => {
            return Err(format!(
                "work item provider {unsupported} has no registered traversal adapter"
            ));
        }
    };
    if !adapter.detect().await {
        return Err(format!("work item provider {provider} is not operational"));
    }
    let query = WorkItemQuery {
        project_root: project_root.to_path_buf(),
        parent: parent_work_item_id.map(|provider_item_id| WorkItemRef {
            provider,
            provider_item_id: provider_item_id.to_string(),
            project_root: project_root.to_path_buf(),
            external_url: None,
        }),
        limit: 1_000,
    };
    let items = adapter
        .list(&query)
        .await
        .map_err(|error| error.to_string())?;
    let deferred: std::collections::HashSet<String> = {
        let focusa = state.focusa.read().await;
        focusa
            .work_loop
            .deferred_items
            .iter()
            .map(|item| item.work_item_id.clone())
            .collect()
    };
    let mut readiness = evaluate_readiness(&items, &query);
    readiness
        .ready
        .retain(|item| !deferred.contains(&item.provider_item_id));
    Ok((provider, readiness))
}

async fn alternate_ready_work_snapshot(
    state: &Arc<AppState>,
    current_task: Option<&focusa_core::types::SpecLinkedTaskPacket>,
    project_root: &Path,
) -> Value {
    let Some(task) = current_task else {
        return json!({ "exists": false, "reason": "no_current_task" });
    };
    let root_work_item_id = {
        let focusa = state.focusa.read().await;
        focusa.work_loop.execution_work_item_id.clone()
    };
    let Some(root_work_item_id) = root_work_item_id else {
        return json!({ "exists": false, "reason": "execution_root_unbound" });
    };
    match provider_neutral_readiness(state, project_root, Some(&root_work_item_id)).await {
        Ok((provider, readiness)) => {
            let alternate = readiness
                .ready
                .iter()
                .find(|item| item.provider_item_id != task.work_item_id);
            json!({
                "exists": alternate.is_some(),
                "provider": provider,
                "candidate_work_item_id": alternate.map(|item| item.provider_item_id.as_str()),
                "blocked_count": readiness.blocked.len(),
            })
        }
        Err(error) => json!({
            "exists": false,
            "reason": "provider_query_failed",
            "error": error,
        }),
    }
}

fn build_blocker_package(
    wl: &focusa_core::types::WorkLoopState,
    alternate_ready_work: Value,
) -> Option<Value> {
    let blocker_class = wl.last_blocker_class?;
    let current_task = wl.current_task.as_ref();
    let linked_spec_requirement =
        current_task.and_then(|task| task.linked_spec_refs.first().cloned());
    let mut recovery_attempts = vec!["self-recovery on same task".to_string()];
    if wl.consecutive_failures_for_task_class > 0 {
        recovery_attempts.push(format!(
            "repeated recovery attempts: {}",
            wl.consecutive_failures_for_task_class
        ));
    }
    let mut fallback_attempts = Vec::new();
    if let Some(worker) = wl.active_worker.as_ref() {
        fallback_attempts.push(format!("worker route: {}", worker.worker_id));
        if !worker.fallback_available {
            fallback_attempts.push("fallback worker already selected".to_string());
        }
    }

    let retries_remaining = wl
        .policy
        .max_consecutive_failures
        .saturating_sub(wl.consecutive_failures_for_task_class);
    let self_recovery_allowed = retries_remaining > 0
        && !wl.pause_flags.operator_override_active
        && !wl.pause_flags.destructive_confirmation_required
        && !wl.pause_flags.governance_decision_pending
        && !matches!(
            blocker_class,
            focusa_core::types::BlockerClass::Governance
                | focusa_core::types::BlockerClass::Permission
        );

    let (exact_operator_decision_needed, recommended_next_action) = if self_recovery_allowed {
        (
            "no immediate operator decision required unless retry budget is exhausted".to_string(),
            format!(
                "retry self-recovery on the blocked task (remaining retry budget: {retries_remaining})"
            ),
        )
    } else if wl.pause_flags.operator_override_active {
        (
            "confirm override intent and choose whether to resume, pause longer, or stop"
                .to_string(),
            "honor operator override before any further autonomous step".to_string(),
        )
    } else if wl.pause_flags.destructive_confirmation_required {
        (
            "approve or reject the destructive/high-risk action".to_string(),
            "provide explicit approval or redirect to a safer path".to_string(),
        )
    } else if wl.pause_flags.governance_decision_pending
        || blocker_class == focusa_core::types::BlockerClass::Governance
    {
        (
            "resolve the governance-sensitive decision or amend policy/spec".to_string(),
            "choose the governing outcome, then resume with updated authority".to_string(),
        )
    } else if alternate_ready_work.get("exists").and_then(Value::as_bool) == Some(true) {
        (
            "decide whether to defer the blocked task and switch to alternate ready work"
                .to_string(),
            "defer the blocked task and continue with the alternate ready item".to_string(),
        )
    } else {
        (
            "review blocker package because no valid ready work remains".to_string(),
            "escalate to the operator because retries and alternate ready work are exhausted"
                .to_string(),
        )
    };

    Some(json!({
        "blocker_class": blocker_class,
        "affected_work_item_id": current_task.map(|task| task.work_item_id.clone()),
        "linked_spec_requirement": linked_spec_requirement,
        "recovery_attempts_made": recovery_attempts,
        "fallback_attempts_made": fallback_attempts,
        "alternate_ready_work": alternate_ready_work,
        "exact_operator_decision_needed": exact_operator_decision_needed,
        "recommended_next_action": recommended_next_action,
    }))
}

fn continuation_boundary_reason(wl: &focusa_core::types::WorkLoopState) -> Option<&'static str> {
    if wl.decision_context.operator_steering_detected {
        return Some("operator steering detected");
    }
    if wl.pause_flags.governance_decision_pending {
        return Some("governance decision pending");
    }
    None
}

fn transport_health_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    json!({
        "status": if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
            "degraded"
        } else {
            "healthy"
        },
        "last_reason": wl.last_blocker_reason,
    })
}

fn execution_environment_for_status(
    transport_session_state: Option<&str>,
    worktree: &Value,
) -> Value {
    let git_available = worktree
        .get("git_available")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_worktree = worktree
        .get("in_worktree")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let worktree_clean = worktree
        .get("clean")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let transport_attached = matches!(
        transport_session_state,
        Some(
            "attached"
                | "running"
                | "turn_active"
                | "streaming"
                | "turn_completed"
                | "agent_completed"
        )
    );

    let affordance_status = if git_available && in_worktree && transport_attached {
        "available"
    } else {
        "blocked"
    };
    let affordance_reason = if affordance_status == "available" {
        Some("Git worktree and transport session are available for non-destructive code-edit execution".to_string())
    } else {
        Some(format!(
            "Missing execution prerequisites: git_available={git_available}, in_worktree={in_worktree}, transport_attached={transport_attached}"
        ))
    };

    json!({
        "context_kind": if in_worktree { "local_dev" } else { "constrained_runtime" },
        "facts": [
            {
                "id": "fact_git_available",
                "fact_key": "git_available",
                "fact_value": git_available,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_in_worktree",
                "fact_key": "in_worktree",
                "fact_value": in_worktree,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_worktree_clean",
                "fact_key": "worktree_clean",
                "fact_value": worktree_clean,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_transport_attached",
                "fact_key": "transport_session_attached",
                "fact_value": transport_attached,
                "source": "work_loop.transport_session_state"
            }
        ],
        "affordances": [
            {
                "id": "affordance_safe_local_code_edit",
                "affordance_kind": "safe_local_edit_available",
                "status": affordance_status,
                "recommended": affordance_status == "available" && worktree_clean,
                "reason": affordance_reason,
                "required_fact_ids": [
                    "fact_git_available",
                    "fact_in_worktree",
                    "fact_transport_attached"
                ]
            }
        ]
    })
}

fn extract_assistant_text(message: &Value) -> Option<String> {
    if let Some(text) = message.as_str() {
        return Some(text.to_string());
    }
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    message
        .get("content")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| {
                    part.get("text")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                        .or_else(|| {
                            part.get("content")
                                .and_then(Value::as_str)
                                .map(|s| s.to_string())
                        })
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .filter(|s| !s.is_empty())
}

const WORK_LOOP_OUTCOME_PREFIX: &str = "FOCUSA_WORK_LOOP_OUTCOME ";
const WORK_LOOP_OUTCOME_SCHEMA: &str = "focusa.work_loop_outcome.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkLoopOutcomeReceipt {
    schema: String,
    pub(crate) work_item_id: String,
    pub(crate) status: WorkLoopOutcomeStatus,
    #[serde(default)]
    pub(crate) summary: Option<String>,
    #[serde(default)]
    pub(crate) spec_conformant: bool,
    #[serde(default)]
    pub(crate) evidence_citations: Vec<EvidenceCitation>,
}

pub(crate) fn parse_work_loop_outcome_receipt(output: &str) -> Option<WorkLoopOutcomeReceipt> {
    let payload = output
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix(WORK_LOOP_OUTCOME_PREFIX))?;
    let receipt: WorkLoopOutcomeReceipt = serde_json::from_str(payload).ok()?;
    (receipt.schema == WORK_LOOP_OUTCOME_SCHEMA).then_some(receipt)
}

fn render_continuous_turn_prompt(
    task: &SpecLinkedTaskPacket,
    mission: Option<String>,
    focus: Option<String>,
    last_checkpoint: Option<String>,
) -> String {
    let acceptance = if task.acceptance_criteria.is_empty() {
        "- satisfy the authoritative spec and verification requirements".to_string()
    } else {
        task.acceptance_criteria
            .iter()
            .map(|item| format!("- {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let scope = if task.allowed_scope.is_empty() {
        "(inherit current mission scope)".to_string()
    } else {
        task.allowed_scope.join(", ")
    };
    let refs = if task.linked_spec_refs.is_empty() {
        "(none)".to_string()
    } else {
        task.linked_spec_refs.join(", ")
    };
    format!(
        "Continuous work mode.\nWork item: {id} — {title}\nMission: {mission}\nFocus: {focus}\nAllowed scope: {scope}\nLinked specs: {refs}\nAcceptance criteria:\n{acceptance}\nLast checkpoint: {checkpoint}\nDelivery boundary: work locally; source/test commits are allowed, but never push, deploy, merge, or release. Prohibited delivery actions are not acceptance requirements and must not be reported as blockers.\nExecute the next concrete step only within scope. Never claim completion from prose alone. End with one typed line: FOCUSA_WORK_LOOP_OUTCOME {{\"schema\":\"focusa.work_loop_outcome.v1\",\"work_item_id\":\"{id}\",\"status\":\"continue|completed|blocked\",\"summary\":\"bounded result\",\"spec_conformant\":true|false,\"evidence_citations\":[{{\"kind\":\"test\",\"ref\":\"stable/path/or/proof\",\"required\":true}}]}}. Use completed only when acceptance evidence is stable and verifiable.",
        id = task.work_item_id,
        title = task.title,
        mission = mission.unwrap_or_else(|| "(none)".to_string()),
        focus = focus.unwrap_or_else(|| "(none)".to_string()),
        checkpoint = last_checkpoint.unwrap_or_else(|| "(none)".to_string()),
    )
}

async fn dispatch_pi_prompt(
    state: &Arc<AppState>,
    message: String,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg =
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": message})
            .to_string()
            + "\n";
    session
        .stdin
        .write_all(msg.as_bytes())
        .await
        .map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    Ok(())
}

async fn maybe_auto_advance_from_blocked(
    state: &Arc<AppState>,
    reason: &str,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let (enabled, status, current_task, boundary_reason, scope_root) = {
        let focusa = state.focusa.read().await;
        (
            focusa.work_loop.enabled,
            focusa.work_loop.status,
            focusa.work_loop.current_task.clone(),
            continuation_boundary_reason(&focusa.work_loop),
            work_loop_scope_root(&focusa),
        )
    };

    let blocked = status == WorkLoopStatus::Blocked;
    if !enabled || !blocked || boundary_reason.is_some() {
        return Ok(false);
    }
    let Some(scope_root) = scope_root else {
        return Ok(false);
    };

    let Some(task) = current_task else {
        if maybe_select_rooted_ready_work_item(state, &scope_root).await? {
            let _ = state
                .command_tx
                .send(Action::CheckpointContinuousLoop {
                    checkpoint_id: Uuid::now_v7(),
                    summary: format!(
                        "auto-advanced from blocked state without bound task ({})",
                        reason.chars().take(120).collect::<String>()
                    ),
                })
                .await;
            sleep(Duration::from_millis(120)).await;
            return Ok(true);
        }
        return Ok(false);
    };

    if blocked {
        state
            .command_tx
            .send(Action::DeferContinuousWorkItem {
                work_item_id: task.work_item_id.clone(),
                reason: reason.chars().take(220).collect(),
            })
            .await
            .map_err(|error| work_loop_dispatch_failed("work_loop_defer", error))?;
    }

    let parent_work_item_id = {
        let focusa = state.focusa.read().await;
        focusa
            .work_loop
            .execution_work_item_id
            .clone()
            .ok_or_else(|| {
                bad_request("cannot select alternate work without root WorkItem binding")
            })?
    };

    state
        .command_tx
        .send(Action::SelectNextContinuousSubtask {
            parent_work_item_id,
        })
        .await
        .map_err(|e| work_loop_dispatch_failed("work_loop_dispatch", e))?;

    let _ = state
        .command_tx
        .send(Action::CheckpointContinuousLoop {
            checkpoint_id: Uuid::now_v7(),
            summary: format!(
                "auto-advanced from blocked task {} ({})",
                task.work_item_id,
                reason.chars().take(120).collect::<String>()
            ),
        })
        .await;

    sleep(Duration::from_millis(120)).await;
    Ok(true)
}

async fn maybe_select_rooted_ready_work_item(
    state: &Arc<AppState>,
    scope_root: &Path,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let (boundary_reason, root_work_item_id) = {
        let focusa = state.focusa.read().await;
        (
            continuation_boundary_reason(&focusa.work_loop),
            focusa.work_loop.execution_work_item_id.clone(),
        )
    };
    if boundary_reason.is_some() {
        return Ok(false);
    }
    let Some(root_work_item_id) = root_work_item_id else {
        return Ok(false);
    };
    let (provider, readiness) =
        provider_neutral_readiness(state, scope_root, Some(&root_work_item_id))
            .await
            .map_err(|error| {
                work_loop_failure(
                    StatusCode::BAD_GATEWAY,
                    "work_loop_select_next",
                    "provider_query_failed",
                    error,
                )
            })?;
    let Some(selected) = readiness.ready.into_iter().next() else {
        return Ok(false);
    };

    let task_run_id = {
        let focusa = state.focusa.read().await;
        focusa.work_loop.run.task_run_id
    };
    state
        .command_tx
        .send(Action::SetContinuousWorkItem {
            task_run_id,
            packet: SpecLinkedTaskPacket {
                work_item_id: selected.provider_item_id,
                title: selected.title,
                task_class: focusa_core::types::TaskClass::Unknown,
                linked_spec_refs: selected.spec_refs,
                acceptance_criteria: selected.acceptance_criteria,
                required_verification_tier: Some("task-class".to_string()),
                allowed_scope: vec![],
                dependencies: selected
                    .dependencies
                    .into_iter()
                    .map(|dependency| dependency.provider_item_id)
                    .collect(),
                tranche_id: None,
                blocker_class: None,
                checkpoint_summary: Some(format!("selected by provider-neutral {provider} graph")),
            },
        })
        .await
        .map_err(|error| work_loop_dispatch_failed("work_loop_dispatch", error))?;

    sleep(Duration::from_millis(120)).await;
    Ok(true)
}

pub async fn maybe_dispatch_continuous_turn_prompt(
    state: &Arc<AppState>,
    reason: &str,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let _ = maybe_auto_advance_from_blocked(state, reason).await?;

    let (
        enabled,
        status,
        task_run_id,
        current_task,
        mission,
        focus,
        last_checkpoint_id,
        last_turn_requested_at,
        status_heartbeat_ms,
        transport_session_state,
        transport_partition_matches,
        boundary_reason,
        scope_root,
    ) = {
        let focusa = state.focusa.read().await;
        let active_frame = focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid));
        (
            focusa.work_loop.enabled,
            focusa.work_loop.status,
            focusa.work_loop.run.task_run_id,
            focusa.work_loop.current_task.clone(),
            active_frame.map(|f| f.focus_state.intent.clone()),
            active_frame.map(|f| f.focus_state.current_state.clone()),
            focusa
                .work_loop
                .run
                .last_checkpoint_id
                .map(|v| v.to_string()),
            focusa.work_loop.last_turn_requested_at,
            focusa.work_loop.policy.status_heartbeat_ms,
            focusa.work_loop.transport_session_state.clone(),
            focusa.work_loop.transport_scope == focusa.work_loop.execution_scope
                && focusa.work_loop.transport_work_item_id
                    == focusa.work_loop.execution_work_item_id
                && focusa.work_loop.transport_workpoint_id
                    == focusa.work_loop.execution_workpoint_id
                && focusa.work_loop.transport_session_id.is_some(),
            continuation_boundary_reason(&focusa.work_loop),
            work_loop_scope_root(&focusa),
        )
    };

    if !enabled
        || !matches!(
            status,
            WorkLoopStatus::SelectingReadyWork
                | WorkLoopStatus::Idle
                | WorkLoopStatus::AwaitingHarnessTurn
                | WorkLoopStatus::AdvancingTask
                | WorkLoopStatus::EvaluatingOutcome
        )
    {
        return Ok(false);
    }
    if boundary_reason.is_some() {
        return Ok(false);
    }
    let Some(scope_root) = scope_root else {
        return Ok(false);
    };

    if current_task.is_none() {
        if maybe_select_rooted_ready_work_item(state, &scope_root).await? {
            let refreshed_task = {
                let focusa = state.focusa.read().await;
                focusa.work_loop.current_task.clone()
            };
            if let Some(task) = refreshed_task {
                if !transport_partition_matches {
                    return Ok(false);
                }
                state
                    .command_tx
                    .send(Action::RequestNextContinuousTurn {
                        task_run_id,
                        work_item_id: Some(task.work_item_id.clone()),
                        reason: "re-selected work after unassigned turn state".to_string(),
                    })
                    .await
                    .map_err(|e| work_loop_dispatch_failed("work_loop_dispatch", e))?;

                let prompt =
                    render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
                dispatch_pi_prompt(state, prompt).await?;
                return Ok(true);
            }
        }

        if status != WorkLoopStatus::Blocked {
            let _ = state
                .command_tx
                .send(Action::EmitEvent {
                    event: FocusaEvent::ContinuousTurnBlocked {
                        blocker_class: BlockerClass::SpecGap,
                        reason: "no ready work available for autonomous continuation".to_string(),
                        work_item_id: None,
                    },
                })
                .await;
        }

        return Ok(false);
    }

    if !transport_partition_matches {
        return Ok(false);
    }

    if let Some(last_turn_at) = last_turn_requested_at {
        let since_last_turn_ms = (Utc::now() - last_turn_at).num_milliseconds().max(0) as u64;
        if status == WorkLoopStatus::AwaitingHarnessTurn {
            let reprompt_stale_ms = status_heartbeat_ms.saturating_mul(3).max(1_500);
            if since_last_turn_ms < reprompt_stale_ms {
                return Ok(false);
            }
        } else if since_last_turn_ms < status_heartbeat_ms {
            return Ok(false);
        }
    }
    let Some(task) = current_task else {
        return Ok(false);
    };

    let task_requires_local_edit_affordance = matches!(
        task.task_class,
        TaskClass::Code | TaskClass::Refactor | TaskClass::Integration | TaskClass::Architecture
    );
    if task_requires_local_edit_affordance {
        let worktree = worktree_status_snapshot(&scope_root).await;
        let execution_environment =
            execution_environment_for_status(transport_session_state.as_deref(), &worktree);
        let safe_local_edit_affordance = execution_environment
            .get("affordances")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find(|item| {
                    item.get("id").and_then(Value::as_str)
                        == Some("affordance_safe_local_code_edit")
                })
            });
        let affordance_status = safe_local_edit_affordance
            .and_then(|item| item.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("blocked");
        if affordance_status != "available" {
            let affordance_reason = safe_local_edit_affordance
                .and_then(|item| item.get("reason"))
                .and_then(Value::as_str)
                .unwrap_or("safe_local_edit_available is blocked in current execution environment")
                .to_string();
            state
                .command_tx
                .send(Action::PauseContinuousWork {
                    reason: format!(
                        "execution affordance blocked before dispatch: safe_local_edit_available; {affordance_reason}"
                    ),
                })
                .await
                .map_err(|e| work_loop_dispatch_failed("work_loop_dispatch", e))?;
            return Ok(false);
        }
    }

    state
        .command_tx
        .send(Action::RequestNextContinuousTurn {
            task_run_id,
            work_item_id: Some(task.work_item_id.clone()),
            reason: reason.to_string(),
        })
        .await
        .map_err(|e| work_loop_dispatch_failed("work_loop_dispatch", e))?;

    let prompt = render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
    dispatch_pi_prompt(state, prompt).await?;
    Ok(true)
}

fn budget_remaining_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    let policy = &wl.policy;
    let elapsed_ms = wl
        .budget_epoch_started_at
        .map(|ts| (Utc::now() - ts).num_milliseconds().max(0) as u64)
        .unwrap_or(0);
    json!({
        "epoch_id": wl.budget_epoch_id,
        "epoch_started_at": wl.budget_epoch_started_at,
        "renewal_count": wl.budget_renewal_count,
        "state": if wl.budget_exhaustion.is_some() { "exhausted" } else { "active" },
        "exhaustion": wl.budget_exhaustion,
        "renewal": {
            "authorized_action": "POST /v1/work-loop/resume with {\"renew_budget\":true}",
            "requires_explicit_approval": true,
        },
        "max_turns": policy.max_turns,
        "max_wall_clock_ms": policy.max_wall_clock_ms,
        "max_retries": policy.max_retries,
        "max_consecutive_failures": policy.max_consecutive_failures,
        "max_consecutive_low_productivity_turns": policy.max_consecutive_low_productivity_turns,
        "max_same_subproblem_retries": policy.max_same_subproblem_retries,
        "status_heartbeat_ms": policy.status_heartbeat_ms,
        "turn_count": wl.turn_count,
        "elapsed_wall_clock_ms": elapsed_ms,
        "cooldown_ms": policy.cooldown_ms,
        "last_turn_requested_at": wl.last_turn_requested_at,
        "remaining_turn_budget": policy.max_turns.map(|max| max.saturating_sub(wl.turn_count)),
        "remaining_wall_clock_ms": policy.max_wall_clock_ms.map(|max| max.saturating_sub(elapsed_ms)),
        "remaining_failure_budget": policy
            .max_consecutive_failures
            .saturating_sub(wl.consecutive_failures_for_task_class),
        "remaining_low_productivity_budget": policy
            .max_consecutive_low_productivity_turns
            .saturating_sub(wl.consecutive_low_productivity_turns),
        "remaining_same_subproblem_budget": policy
            .max_same_subproblem_retries
            .saturating_sub(wl.consecutive_same_work_item_retries),
    })
}

fn next_work_risk_class_for_status(wl: &focusa_core::types::WorkLoopState) -> &'static str {
    let Some(task) = wl.current_task.as_ref() else {
        return "none";
    };
    let title = task.title.to_ascii_lowercase();
    if task
        .allowed_scope
        .iter()
        .any(|scope| scope.to_ascii_lowercase().contains("governance"))
        || matches!(
            wl.last_blocker_class,
            Some(
                focusa_core::types::BlockerClass::Governance
                    | focusa_core::types::BlockerClass::Permission
            )
        )
        || [
            "delete",
            "drop",
            "remove",
            "rename",
            "migrate",
            "rewrite",
            "destructive",
            "governance",
        ]
        .iter()
        .any(|needle| title.contains(needle))
    {
        "high"
    } else if matches!(
        task.task_class,
        focusa_core::types::TaskClass::Architecture | focusa_core::types::TaskClass::Integration
    ) {
        "medium"
    } else {
        "low"
    }
}

fn scoped_workpoint_summary_for_status(
    s: &focusa_core::types::FocusaState,
    key: &WorkstreamKey,
) -> Value {
    let active =
        s.workpoint
            .records
            .iter()
            .filter(|record| {
                record.canonical
                    && record.status == focusa_core::types::WorkpointStatus::Active
                    && record.project_root.as_deref().is_some_and(|root| {
                        record.continuity_id.as_deref().is_some_and(|continuity| {
                            work_loop_scope_matches(key, root, continuity)
                        })
                    })
            })
            .max_by_key(|record| record.updated_at);

    json!({
        "active_workpoint_id": active.map(|record| record.workpoint_id),
        "records_count": s.workpoint.records.len(),
        "recent_drift_count": s.workpoint.drift_events.len(),
        "degraded_fallback_count": s.workpoint.degraded_fallbacks.len(),
        "active": active.map(|record| json!({
            "workpoint_id": record.workpoint_id,
            "work_item_id": record.work_item_id,
            "session_id": record.session_id,
            "frame_id": record.frame_id,
            "status": record.status,
            "checkpoint_reason": record.checkpoint_reason,
            "confidence": record.confidence,
            "canonical": record.canonical,
            "mission": record.mission,
            "action_intent": record.action_intent.as_ref().map(|intent| json!({
                "action_type": intent.action_type,
                "target_ref": intent.target_ref,
                "verification_hooks": intent.verification_hooks,
                "status": intent.status,
            })),
            "verification_count": record.verification_records.len(),
            "blocker_count": record.blockers.len(),
            "next_slice": record.next_slice,
            "source_turn_id": record.source_turn_id,
            "updated_at": record.updated_at,
        })),
    })
}

fn work_loop_execution_partition_payload(
    wl: &focusa_core::types::WorkLoopState,
    active_lease: Option<&WriterLease>,
    writer_claim_key: &str,
) -> Value {
    let parts: std::collections::HashMap<_, _> = writer_claim_key
        .split('|')
        .filter_map(|part| part.split_once(':'))
        .collect();
    let effective_provider = if wl.policy.work_item_provider == WorkItemProvider::None
        && wl
            .execution_scope
            .as_ref()
            .is_some_and(|scope| scope.root_scope.root_path.join(".beads").exists())
    {
        WorkItemProvider::Bd
    } else {
        wl.policy.work_item_provider
    };
    json!({
        "schema": "focusa.work_loop_execution_partition.v2",
        "project_root_key": parts.get("project").copied(),
        "workstream_key": parts.get("workstream").copied(),
        "work_item_key": parts.get("work_item").copied(),
        "work_item_provider": effective_provider,
        "workpoint_id": wl.execution_workpoint_id,
        "current_task_work_item_id": wl.current_task.as_ref().map(|task| task.work_item_id.as_str()),
        "deferred_work_item_ids": wl.deferred_items.iter().map(|item| item.work_item_id.as_str()).collect::<Vec<_>>(),
        "transport_session_id": wl.transport_session_id,
        "transport_work_item_id": wl.transport_work_item_id,
        "transport_workpoint_id": wl.transport_workpoint_id,
        "writer_key": active_lease.map(|lease| lease.writer_id.as_str()),
        "fencing_token": active_lease.map(|lease| lease.fencing_token),
        "lease_acquired_at": active_lease.map(|lease| lease.acquired_at),
        "lease_renewed_at": active_lease.map(|lease| lease.renewed_at),
        "lease_expires_at": active_lease.map(|lease| lease.expires_at),
        "lease_freshness": active_lease.map(|lease| if lease.expires_at > Utc::now() { "current" } else { "expired" }).unwrap_or("unclaimed"),
        "writer_claim_key": writer_claim_key,
        "legacy_active_writer_global": false,
        "partition_status": if writer_claim_key.starts_with("blocked:") {
            writer_claim_key.trim_start_matches("blocked:")
        } else { "work_item_pinned" },
        "migration_note": "writer claims are scoped by ProjectRootKey + WorkstreamKey + WorkItemKey"
    })
}

fn resume_payload_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
    key: &WorkstreamKey,
) -> Value {
    json!({
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
        "restored_context_summary": wl.restored_context_summary,
        "last_blocker_reason": wl.last_blocker_reason,
        "last_completed_turn_summary": wl.last_observed_summary,
        "active_workpoint": scoped_workpoint_summary_for_status(s, key),
        "continuation_eligibility": wl.enabled && !wl.pause_flags.operator_override_active,
        "current_transport_health": if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
            "degraded"
        } else {
            "healthy"
        },
        "exact_recovery_action": if wl.budget_exhaustion.is_some() {
            "approved resume with renew_budget=true"
        } else if !wl.deferred_items.is_empty() {
            "select next non-deferred ready WorkItem under execution root"
        } else if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
            "attach a transport matching the execution partition"
        } else {
            "continue current selected WorkItem"
        },
        "current_ask_and_scope_posture": json!({
            "current_ask": wl.decision_context.current_ask,
            "ask_kind": wl.decision_context.ask_kind,
            "scope_kind": wl.decision_context.scope_kind,
            "carryover_policy": wl.decision_context.carryover_policy,
            "excluded_context_reason": wl.decision_context.excluded_context_reason,
            "excluded_context_labels": wl.decision_context.excluded_context_labels,
            "work_item": wl.current_task.as_ref().map(|task| json!({
                "work_item_id": task.work_item_id,
                "allowed_scope": task.allowed_scope,
                "linked_spec_refs": task.linked_spec_refs,
            })),
        }),
    })
}

fn commitment_lifecycle_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    let active_commitment = wl.current_task.as_ref().map(|task| {
        json!({
            "commitment_id": format!("commitment:{}", task.work_item_id),
            "work_item_id": task.work_item_id,
            "commitment_kind": "continuous_work_item",
            "status": if matches!(wl.status, focusa_core::types::WorkLoopStatus::Blocked | focusa_core::types::WorkLoopStatus::Paused) {
                "at_risk"
            } else {
                "active"
            }
        })
    });

    let decay_pressure =
        wl.consecutive_low_productivity_turns + wl.consecutive_same_work_item_retries;
    let persistence_posture = if wl.current_task.is_none() {
        "none"
    } else if decay_pressure == 0 && wl.consecutive_failures_for_task_class == 0 {
        "stable"
    } else {
        "stressed"
    };

    let release_state = if wl.current_task.is_none() && wl.last_completed_task_id.is_some() {
        "released_on_completion"
    } else if wl.current_task.is_none() && wl.last_blocker_reason.is_some() {
        "released_on_blocker"
    } else if wl.current_task.is_none() {
        "released_or_unbound"
    } else {
        "bound"
    };

    json!({
        "active_commitment": active_commitment,
        "creation_semantics": {
            "trigger": "SetContinuousWorkItem",
            "evidence_fields": ["current_task.work_item_id", "run.task_run_id"],
            "created_when": wl.current_task.as_ref().map(|task| format!("commitment:{}", task.work_item_id)),
        },
        "persistence_semantics": {
            "posture": persistence_posture,
            "policy": "commitment remains bound across turns unless completion, blocker escalation, or explicit pause/stop release occurs",
            "inhibits_drift_via": ["current_task pinning", "same-work-item retry tracking", "pause_flags"],
        },
        "decay_semantics": {
            "decay_pressure": decay_pressure,
            "failure_pressure": wl.consecutive_failures_for_task_class,
            "decay_triggers": {
                "low_productivity_turns": wl.consecutive_low_productivity_turns,
                "same_subproblem_retries": wl.consecutive_same_work_item_retries,
                "task_class_failures": wl.consecutive_failures_for_task_class,
            },
            "decay_posture": if decay_pressure > 0 || wl.consecutive_failures_for_task_class > 0 {
                "decaying"
            } else {
                "healthy"
            },
        },
        "release_semantics": {
            "state": release_state,
            "release_conditions": [
                "verification-backed completion transition",
                "explicit pause/stop or operator override",
                "blocker escalation when continuation is no longer productive"
            ],
            "last_completed_task_id": wl.last_completed_task_id,
            "last_blocker_reason": wl.last_blocker_reason,
        }
    })
}

fn safe_rate(numerator: u64, denominator: u64) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

fn secondary_loop_quality_metrics_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
) -> Value {
    let verification_result_events = s.telemetry.verification_result_events;
    let decision_consult_events = s.telemetry.decision_consult_events;
    let scope_contamination_events = s.telemetry.scope_contamination_events;
    let subject_hijack_prevented_events = s.telemetry.subject_hijack_prevented_events;
    let subject_hijack_occurred_events = s.telemetry.subject_hijack_occurred_events;

    json!({
        "verification_result_events": verification_result_events,
        "decision_consult_events": decision_consult_events,
        "scope_contamination_events": scope_contamination_events,
        "subject_hijack_prevented_events": subject_hijack_prevented_events,
        "subject_hijack_occurred_events": subject_hijack_occurred_events,
        "useful_events": s.telemetry.secondary_loop_useful_events,
        "low_quality_events": s.telemetry.secondary_loop_low_quality_events,
        "archived_events": s.telemetry.secondary_loop_archived_events,
        "decision_consult_rate": safe_rate(decision_consult_events, verification_result_events),
        "scope_contamination_rate": safe_rate(scope_contamination_events, verification_result_events),
        "subject_hijack_rate": safe_rate(subject_hijack_occurred_events, verification_result_events),
        "verification_coverage_rate": safe_rate(verification_result_events, wl.turn_count as u64),
        "verification_coverage_denominator": wl.turn_count,
    })
}

fn metacognitive_outcome_contracts() -> Value {
    json!([
        {
            "contract_id": "self_monitoring_signal",
            "category": "self_regulation",
            "machine_check_fields": ["quality_trace_events", "objective_counts", "continuation_decision_counts"]
        },
        {
            "contract_id": "strategy_selection_signal",
            "category": "cognitive_strategy",
            "machine_check_fields": ["objective_counts", "dominant_objective", "continuation_decision_counts"]
        },
        {
            "contract_id": "progress_regulation_signal",
            "category": "progress_control",
            "machine_check_fields": ["continuation_decision_counts", "non_closure_objective_events", "non_closure_objective_rate"]
        },
        {
            "contract_id": "transfer_to_new_context_signal",
            "category": "transfer_learning",
            "machine_check_fields": ["objective_counts", "non_closure_objective_events"]
        },
        {
            "contract_id": "motivation_ownership_signal",
            "category": "motivation",
            "machine_check_fields": ["objective_counts", "continuation_decision_counts"]
        },
        {
            "contract_id": "social_emotional_perspective_signal",
            "category": "social_emotional",
            "machine_check_fields": ["objective_counts", "non_closure_objective_events"]
        },
        {
            "contract_id": "teaching_regulation_signal",
            "category": "instructor_regulation",
            "machine_check_fields": ["objective_counts", "non_closure_objective_rate"]
        }
    ])
}

fn secondary_loop_objective_profile_for_status(s: &focusa_core::types::FocusaState) -> Value {
    let mut objective_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut continuation_decision_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut quality_trace_events = 0_u64;
    let mut non_closure_objective_events = 0_u64;

    for payload in s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("verification_result")
        })
        .filter_map(|event| event.get("payload"))
        .filter(|payload| {
            payload.get("verification_kind").and_then(Value::as_str)
                == Some("secondary_loop_quality")
        })
    {
        quality_trace_events += 1;

        let objective = payload
            .get("loop_objective")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|objective| !objective.is_empty())
            .unwrap_or("continuous_turn_outcome_quality")
            .to_string();
        if objective != "continuous_turn_outcome_quality" {
            non_closure_objective_events += 1;
        }
        *objective_counts.entry(objective).or_insert(0) += 1;

        let continuation_decision = payload
            .get("continuation_decision")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|decision| !decision.is_empty())
            .unwrap_or("unknown")
            .to_string();
        *continuation_decision_counts
            .entry(continuation_decision)
            .or_insert(0) += 1;
    }

    let dominant_objective = objective_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(objective, _)| objective.clone());

    json!({
        "quality_trace_events": quality_trace_events,
        "objective_counts": objective_counts,
        "continuation_decision_counts": continuation_decision_counts,
        "non_closure_objective_events": non_closure_objective_events,
        "non_closure_objective_rate": safe_rate(non_closure_objective_events, quality_trace_events),
        "dominant_objective": dominant_objective,
        "metacognitive_outcome_contracts": metacognitive_outcome_contracts(),
    })
}

fn secondary_loop_eval_bundle_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
) -> Value {
    let promoted = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "promoted")
        .count() as u64;
    let rejected = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "rejected")
        .count() as u64;

    let retained_as_projection = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "retained_as_projection")
        .count() as u64;
    let deferred_for_review = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "deferred_for_review")
        .count() as u64;
    let archived_failed_attempt = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "archived_failed_attempt")
        .count() as u64;
    let archived = s.telemetry.secondary_loop_archived_events + archived_failed_attempt;

    let recent_entries: Vec<focusa_core::types::SecondaryLoopLedgerEntry> = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .rev()
        .take(20)
        .cloned()
        .collect();
    let trace_handles: Vec<String> = recent_entries
        .iter()
        .map(|entry| format!("trace://{}", entry.trace_id))
        .collect();
    let ledger_refs: Vec<String> = recent_entries
        .iter()
        .map(|entry| entry.proposal_id.clone())
        .collect();

    json!({
        "task_id": wl
            .current_task
            .as_ref()
            .map(|task| task.work_item_id.clone())
            .or_else(|| wl.last_completed_task_id.clone()),
        "scenario_id": wl
            .run
            .task_run_id
            .map(|id| id.to_string())
            .or_else(|| wl.decision_context.source_turn_id.clone()),
        "model_runtime_configuration": {
            "rfm_level": format!("{:?}", s.rfm.level),
            "autonomy_level": format!("{:?}", wl.current_autonomy_level.unwrap_or(s.autonomy.level)),
            "work_loop_status": format!("{:?}", wl.status),
            "transport_session_state": wl.transport_session_state,
            "policy": {
                "max_turns": wl.policy.max_turns,
                "max_wall_clock_ms": wl.policy.max_wall_clock_ms,
                "max_consecutive_low_productivity_turns": wl.policy.max_consecutive_low_productivity_turns,
                "max_same_subproblem_retries": wl.policy.max_same_subproblem_retries,
            }
        },
        "secondary_loop_kind_invoked": "continuous_turn_outcome_quality",
        "secondary_loop_objective_profile": secondary_loop_objective_profile_for_status(s),
        "trace_handles": trace_handles,
        "promotion_rejection_archival_result": {
            "promoted": promoted,
            "retained_as_projection": retained_as_projection,
            "deferred_for_review": deferred_for_review,
            "rejected": rejected,
            "archived_failed_attempt": archived_failed_attempt,
            "archived": archived,
        },
        "latency_token_cost_impact": {
            "total_prompt_tokens": s.telemetry.total_prompt_tokens,
            "total_completion_tokens": s.telemetry.total_completion_tokens,
            "verification_result_events": s.telemetry.verification_result_events,
            "useful_events": s.telemetry.secondary_loop_useful_events,
            "low_quality_events": s.telemetry.secondary_loop_low_quality_events,
        },
        "final_task_outcome": {
            "last_completed_task_id": wl.last_completed_task_id,
            "last_blocker_class": wl.last_blocker_class,
            "last_blocker_reason": wl.last_blocker_reason,
            "last_observed_summary": wl.last_observed_summary,
        },
        "ledger_refs": ledger_refs,
    })
}

fn secondary_loop_acceptance_hooks_for_status(s: &focusa_core::types::FocusaState) -> Value {
    let quality_payloads: Vec<&Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("verification_result")
        })
        .filter_map(|event| event.get("payload"))
        .filter(|payload| {
            payload.get("verification_kind").and_then(Value::as_str)
                == Some("secondary_loop_quality")
        })
        .collect();

    let useful_quality_traces = quality_payloads
        .iter()
        .filter(|payload| payload.get("loop_quality").and_then(Value::as_str) == Some("useful"))
        .count() as u64;
    let low_quality_traces = quality_payloads
        .iter()
        .filter(|payload| {
            payload.get("loop_quality").and_then(Value::as_str) == Some("low_quality")
        })
        .count() as u64;
    let suppressed_irrelevant_suggestions = quality_payloads
        .iter()
        .filter(|payload| {
            payload.get("continuation_decision").and_then(Value::as_str) == Some("suppress")
        })
        .count() as u64;

    let rejected_or_deferred = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| {
            matches!(
                entry.promotion_status.as_str(),
                "rejected" | "deferred_for_review"
            )
        })
        .count() as u64;
    let archived_attempts = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "archived_failed_attempt")
        .count() as u64
        + s.telemetry.secondary_loop_archived_events;

    let mut comparative_outcomes_by_task: std::collections::BTreeMap<String, (bool, bool)> =
        std::collections::BTreeMap::new();
    for entry in &s.telemetry.secondary_loop_ledger {
        let Some(correlation_id) = entry.correlation_id.as_deref() else {
            continue;
        };
        let slot = comparative_outcomes_by_task
            .entry(correlation_id.to_string())
            .or_insert((false, false));
        if entry.promotion_status == "promoted" {
            slot.0 = true;
        } else if matches!(
            entry.promotion_status.as_str(),
            "rejected" | "deferred_for_review" | "archived_failed_attempt"
        ) {
            slot.1 = true;
        }
    }
    let comparative_improvement_pairs = comparative_outcomes_by_task
        .values()
        .filter(|(has_promoted, has_baseline_failure)| *has_promoted && *has_baseline_failure)
        .count() as u64;

    json!({
        "bounded_improvement_over_no_secondary_baseline": comparative_improvement_pairs > 0
            || (useful_quality_traces > low_quality_traces && useful_quality_traces > 0),
        "irrelevant_secondary_suggestion_suppressed": suppressed_irrelevant_suggestions > 0
            || s.telemetry.subject_hijack_occurred_events > 0,
        "verification_rejection_observed": rejected_or_deferred > 0,
        "decay_or_archival_observed": archived_attempts > 0,
        "evidence_counts": {
            "quality_trace_events": quality_payloads.len(),
            "useful_quality_traces": useful_quality_traces,
            "low_quality_traces": low_quality_traces,
            "suppressed_irrelevant_suggestions": suppressed_irrelevant_suggestions,
            "rejected_or_deferred_outcomes": rejected_or_deferred,
            "archived_outcomes": archived_attempts,
            "comparative_improvement_pairs": comparative_improvement_pairs,
        }
    })
}

fn secondary_loop_closure_replay_evidence_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &focusa_core::replay::SecondaryLoopComparativeReplaySummary,
) -> Value {
    let mut correlation_candidates = Vec::new();

    if let Some(task_run_id) = wl.run.task_run_id {
        correlation_candidates.push(task_run_id.to_string());
    }
    if let Some(current_task) = wl.current_task.as_ref()
        && !correlation_candidates
            .iter()
            .any(|candidate| candidate == &current_task.work_item_id)
    {
        correlation_candidates.push(current_task.work_item_id.clone());
    }
    if let Some(last_completed_task_id) = wl.last_completed_task_id.as_ref()
        && !correlation_candidates
            .iter()
            .any(|candidate| candidate == last_completed_task_id)
    {
        correlation_candidates.push(last_completed_task_id.clone());
    }

    let matched_pair = correlation_candidates.iter().find_map(|candidate| {
        replay_summary
            .task_pairs
            .iter()
            .find(|pair| pair.correlation_id == *candidate)
    });

    json!({
        "correlation_candidates": correlation_candidates,
        "replay_events_scanned": replay_summary.replay_events_scanned,
        "secondary_loop_outcome_events": replay_summary.secondary_loop_outcome_events,
        "comparative_improvement_pairs": replay_summary.comparative_improvement_pairs,
        "current_task_pair_observed": matched_pair
            .map(|pair| pair.comparative_improvement_observed)
            .unwrap_or(false),
        "current_task_pair_id": matched_pair.map(|pair| pair.correlation_id.as_str()),
        "current_task_pair_promoted_outcomes": matched_pair.map(|pair| pair.promoted_outcomes),
        "current_task_pair_non_promoted_outcomes": matched_pair
            .map(|pair| pair.non_promoted_outcomes),
    })
}

fn secondary_loop_replay_surface_payloads_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> (Value, Value) {
    match replay_summary {
        Ok(summary) => {
            let closure_evidence = secondary_loop_closure_replay_evidence_for_status(wl, summary);
            let state = if summary.replay_events_scanned == 0 {
                "zero"
            } else {
                "healthy"
            };
            (
                json!({
                    "schema": WORK_LOOP_REPLAY_SCHEMA,
                    "state": state,
                    "supported_states": WORK_LOOP_TYPED_STATES,
                    "status": "ok",
                    "summary": summary
                }),
                json!({
                    "schema": WORK_LOOP_REPLAY_SCHEMA,
                    "state": state,
                    "supported_states": WORK_LOOP_TYPED_STATES,
                    "status": "ok",
                    "evidence": closure_evidence
                }),
            )
        }
        Err(error) => (
            json!({
                "schema": WORK_LOOP_REPLAY_SCHEMA,
                "state": "unavailable",
                "supported_states": WORK_LOOP_TYPED_STATES,
                "status": "error",
                "error": error
            }),
            json!({
                "schema": WORK_LOOP_REPLAY_SCHEMA,
                "state": "unavailable",
                "supported_states": WORK_LOOP_TYPED_STATES,
                "status": "error",
                "error": error
            }),
        ),
    }
}

fn secondary_loop_replay_consumer_payload_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> Value {
    let (secondary_loop_replay_comparative, secondary_loop_closure_replay_evidence) =
        secondary_loop_replay_surface_payloads_for_status(wl, replay_summary);

    match replay_summary {
        Ok(summary) => json!({
            "schema": WORK_LOOP_REPLAY_SCHEMA,
            "state": if summary.replay_events_scanned == 0 { "zero" } else { "healthy" },
            "supported_states": WORK_LOOP_TYPED_STATES,
            "status": "ok",
            "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
            "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        }),
        Err(error) => json!({
            "schema": WORK_LOOP_REPLAY_SCHEMA,
            "state": "unavailable",
            "supported_states": WORK_LOOP_TYPED_STATES,
            "status": "error",
            "error": error,
            "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
            "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        }),
    }
}

fn secondary_loop_continuity_gate_for_status(
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
    replay_consumer_payload: &Value,
) -> Value {
    let current_task_pair_observed = replay_consumer_payload
        .get("secondary_loop_closure_replay_evidence")
        .and_then(|value| value.get("evidence"))
        .and_then(|value| value.get("current_task_pair_observed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let current_task_pair_id = replay_consumer_payload
        .get("secondary_loop_closure_replay_evidence")
        .and_then(|value| value.get("evidence"))
        .and_then(|value| value.get("current_task_pair_id"))
        .and_then(Value::as_str);

    match replay_summary {
        Ok(_) => json!({
            "state": "open",
            "fail_closed": false,
            "reason": "replay_consumer_ok",
            "replay_status": "ok",
            "current_task_pair_observed": current_task_pair_observed,
            "current_task_pair_id": current_task_pair_id,
            "requires_replay_consumer_ok": true,
        }),
        Err(error) => json!({
            "state": "fail-closed",
            "fail_closed": true,
            "reason": "replay_consumer_error",
            "error": error,
            "replay_status": "error",
            "current_task_pair_observed": false,
            "current_task_pair_id": Value::Null,
            "requires_replay_consumer_ok": true,
        }),
    }
}

fn secondary_loop_closure_bundle_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> Value {
    let secondary_loop_quality_metrics = secondary_loop_quality_metrics_for_status(s, wl);
    let secondary_loop_eval_bundle = secondary_loop_eval_bundle_for_status(s, wl);
    let secondary_loop_acceptance_hooks = secondary_loop_acceptance_hooks_for_status(s);
    let replay_consumer_payload =
        secondary_loop_replay_consumer_payload_for_status(wl, replay_summary);
    let secondary_loop_continuity_gate =
        secondary_loop_continuity_gate_for_status(replay_summary, &replay_consumer_payload);

    let project_status = if wl.enabled {
        if wl.current_task.is_none()
            && wl.last_completed_task_id.is_some()
            && wl.status == focusa_core::types::WorkLoopStatus::AdvancingTask
        {
            "completing"
        } else {
            "active"
        }
    } else {
        "idle"
    };

    let tranche_status = match (&wl.run.tranche_run_id, &wl.current_task, &wl.status) {
        (Some(_), Some(_), _) => "active",
        (Some(_), None, focusa_core::types::WorkLoopStatus::AdvancingTask) => "completed",
        (Some(_), None, _) => "advancing",
        _ => "none",
    };

    json!({
        "status": "ok",
        "doc": "78",
        "work_loop": {
            "enabled": wl.enabled,
            "status": wl.status,
            "project_status": project_status,
            "tranche_status": tranche_status,
            "current_task": wl.current_task,
            "last_completed_task_id": wl.last_completed_task_id,
            "last_continue_reason": wl.last_continue_reason,
            "last_blocker_reason": wl.last_blocker_reason,
        },
        "secondary_loop_quality_metrics": secondary_loop_quality_metrics,
        "secondary_loop_eval_bundle": secondary_loop_eval_bundle,
        "secondary_loop_acceptance_hooks": secondary_loop_acceptance_hooks,
        "secondary_loop_replay_consumer": replay_consumer_payload,
        "secondary_loop_continuity_gate": secondary_loop_continuity_gate,
        "evidence_contract": {
            "watchdog_consumer": "scripts/work_loop_watchdog.sh",
            "replay_consumer_route": "/v1/work-loop/replay/closure-evidence",
            "continuity_gate_policy": "fail-closed when replay consumer is unavailable",
        },
    })
}

#[cfg(test)]
fn compatible_typed_surface_state<'a>(
    schema: &str,
    expected_schema: &str,
    state: &'a str,
) -> &'a str {
    if schema == expected_schema && WORK_LOOP_TYPED_STATES.contains(&state) {
        state
    } else {
        "unsupported"
    }
}

fn work_loop_status_surface_state(
    wl: &focusa_core::types::WorkLoopState,
    boundary_reason: Option<&str>,
    active_lease: Option<&WriterLease>,
) -> &'static str {
    if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
        "unavailable"
    } else if wl.budget_exhaustion.is_some() {
        "exhausted"
    } else if boundary_reason.is_some()
        || wl.pause_flags.operator_override_active
        || wl.pause_flags.destructive_confirmation_required
        || wl.pause_flags.governance_decision_pending
    {
        "blocked"
    } else if wl.enabled && active_lease.is_none() {
        "absent"
    } else if wl.enabled {
        "healthy"
    } else if wl.current_task.is_some() || active_lease.is_some() {
        "stale"
    } else {
        "zero"
    }
}

async fn health(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    // Never hold the daemon projection lock while acquiring another lock.
    let s = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let wl = &s.work_loop;
    let claim_key = writer_claim_key_from_scope(&scope, &s);
    let active_lease = {
        let claims = state.writer_claims.read().await;
        active_writer_lease_for_key(&claims, &claim_key, Utc::now())
    };
    let active_writer = active_lease.as_ref().map(|lease| lease.writer_id.clone());
    let boundary_reason = continuation_boundary_reason(wl);
    let dispatch_ready = wl.enabled
        && boundary_reason.is_none()
        && !wl.pause_flags.operator_override_active
        && !wl.pause_flags.destructive_confirmation_required
        && !wl.pause_flags.governance_decision_pending
        && wl.status != focusa_core::types::WorkLoopStatus::TransportDegraded;
    let payload = json!({
        "schema": WORK_LOOP_STATUS_SCHEMA,
        "state": work_loop_status_surface_state(wl, boundary_reason, active_lease.as_ref()),
        "supported_states": WORK_LOOP_TYPED_STATES,
        "status": "ok",
        "route_tier": "hot",
        "authority": bounded_orchestration_authority_payload(),
        "summary_only": true,
        "enabled": wl.enabled,
        "work_loop_status": wl.status,
        "project_status": if wl.enabled { "active" } else { "idle" },
        "current_task_id": wl.current_task.as_ref().map(|task| task.work_item_id.clone()),
        "last_completed_task_id": wl.last_completed_task_id,
        "active_writer": active_writer,
        "execution_partition": work_loop_execution_partition_payload(wl, active_lease.as_ref(), &claim_key),
        "dispatch_readiness": {
            "ready": dispatch_ready,
            "boundary_reason": boundary_reason,
            "transport_status": if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded { "degraded" } else { "healthy" },
            "pause_flags": wl.pause_flags,
            "next_step": if dispatch_ready { "dispatch may proceed via work-loop control or heartbeat" } else { "inspect writer/status/deep before dispatching or retrying" },
        },
        "deep_status_route": "/v1/work-loop/status/deep",
        "cold_omitted": [
            "policy", "run", "blocker_package", "secondary_loop_eval_artifacts",
            "secondary_loop_replay_consumer", "worktree", "governance"
        ],
        "next_tools": ["focusa_work_loop_status", "focusa_work_loop_writer_status"],
    });
    record_json_response_size("/v1/work-loop/health", &payload);
    Ok(Json(payload))
}

async fn status(
    scope: WorkLoopScope,
    Query(query): Query<WorkLoopStatusQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    // Status performs provider, worktree, and transport awaits below. Clone
    // first so daemon projection writes cannot be starved behind this reader.
    let s = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let wl = &s.work_loop;
    let claim_key = writer_claim_key_from_scope(&scope, &s);
    let active_lease = {
        let claims = state.writer_claims.read().await;
        active_writer_lease_for_key(&claims, &claim_key, Utc::now())
    };
    let active_writer = active_lease.as_ref().map(|lease| lease.writer_id.clone());
    let boundary_reason = continuation_boundary_reason(wl);
    if query.summary_only {
        let transport_health = transport_health_for_status(wl);
        let budget_remaining = budget_remaining_for_status(wl);
        let resume_payload = resume_payload_for_status(&s, wl, &scope.0);
        let payload = json!({
            "schema": WORK_LOOP_STATUS_SCHEMA,
            "state": work_loop_status_surface_state(wl, boundary_reason, active_lease.as_ref()),
            "supported_states": WORK_LOOP_TYPED_STATES,
            "route_tier": "hot",
            "summary_only": true,
            "authority": bounded_orchestration_authority_payload(),
            "deep_status_route": "/v1/work-loop/status/deep",
            "cold_omitted": [
                "policy", "run", "blocker_package", "secondary_loop_eval_artifacts",
                "secondary_loop_replay_consumer", "worktree", "governance"
            ],
            "enabled": wl.enabled,
            "status": wl.status,
            "project_status": if wl.enabled { "active" } else { "idle" },
            "current_task": wl.current_task,
            "last_completed_task_id": wl.last_completed_task_id,
            "decision_context": wl.decision_context,
            "active_writer": active_writer,
            "execution_partition": work_loop_execution_partition_payload(wl, active_lease.as_ref(), &claim_key),
            "transport_health": transport_health,
            "budget_remaining": budget_remaining,
            "supervisor_perf": supervisor_perf_payload(&state),
            "resume_payload": resume_payload,
            "active_workpoint": scoped_workpoint_summary_for_status(&s, &scope.0),
            "bounds": {
                "summary_only": true,
                "truncated": true,
                "omitted_categories": [
                    "policy", "run", "blocker_package", "secondary_loop_eval_artifacts",
                    "secondary_loop_replay_consumer", "worktree", "governance"
                ],
                "rehydrate": {"route":"/v1/work-loop/status", "summary_only":"false"}
            }
        });
        record_json_response_size("/v1/work-loop/status", &payload);
        return Ok(Json(payload));
    }
    let driver_snapshot = {
        let guard = state.pi_rpc_session.lock().await;
        guard.as_ref().map(|session| {
            json!({
                "adapter": "pi-rpc",
                "session_id": session.session_id,
                "cwd": session.cwd,
                "uptime_ms": session.started_at.elapsed().as_millis(),
            })
        })
    };
    let scope_root = request_scope_root(&scope);
    let worktree = worktree_status_snapshot(&scope_root).await;
    let alternate_ready_work =
        alternate_ready_work_snapshot(&state, wl.current_task.as_ref(), &scope_root).await;
    let blocker_package = build_blocker_package(wl, alternate_ready_work.clone());
    let transport_health = transport_health_for_status(wl);
    let execution_environment =
        execution_environment_for_status(wl.transport_session_state.as_deref(), &worktree);
    let budget_remaining = budget_remaining_for_status(wl);
    let resume_payload = resume_payload_for_status(&s, wl, &scope.0);
    let commitment_lifecycle = commitment_lifecycle_for_status(wl);
    let secondary_loop_quality_metrics = secondary_loop_quality_metrics_for_status(&s, wl);
    let secondary_loop_eval_bundle = secondary_loop_eval_bundle_for_status(&s, wl);
    let secondary_loop_acceptance_hooks = secondary_loop_acceptance_hooks_for_status(&s);
    let replay_config = focusa_core::replay::ReplayConfig {
        from: None,
        until: None,
        session_id: s.session.as_ref().map(|session| session.session_id),
        frame_id: None,
    };
    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &replay_config,
        )
        .map_err(|error| error.to_string());
    let workpoint_replay_summary =
        focusa_core::replay::workpoint_summary_from_replay(&state.persistence, &replay_config)
            .map_err(|error| error.to_string());
    let secondary_loop_replay_consumer =
        secondary_loop_replay_consumer_payload_for_status(wl, &secondary_loop_replay_summary);
    let secondary_loop_continuity_gate = secondary_loop_continuity_gate_for_status(
        &secondary_loop_replay_summary,
        &secondary_loop_replay_consumer,
    );
    let secondary_loop_replay_comparative = secondary_loop_replay_consumer
        .get("secondary_loop_replay_comparative")
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "status": "error",
                "error": "missing secondary_loop_replay_comparative payload",
            })
        });
    let secondary_loop_closure_replay_evidence = secondary_loop_replay_consumer
        .get("secondary_loop_closure_replay_evidence")
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "schema": WORK_LOOP_REPLAY_SCHEMA,
                "state": "unavailable",
                "supported_states": WORK_LOOP_TYPED_STATES,
                "status": "error",
                "error": "missing secondary_loop_closure_replay_evidence payload",
            })
        });
    let pending_proposals = focusa_core::pre::pending_count(&s.pre);
    let next_work_risk_class = next_work_risk_class_for_status(wl);
    let payload = json!({
        "schema": WORK_LOOP_STATUS_SCHEMA,
        "state": work_loop_status_surface_state(wl, boundary_reason, active_lease.as_ref()),
        "supported_states": WORK_LOOP_TYPED_STATES,
        "route_tier": "cold",
        "summary_only": false,
        "authority": bounded_orchestration_authority_payload(),
        "cold_omitted": [],
        "enabled": wl.enabled,
        "status": wl.status,
        "project_status": if wl.enabled {
            if wl.current_task.is_none() && wl.last_completed_task_id.is_some() && wl.status == focusa_core::types::WorkLoopStatus::AdvancingTask {
                "completing"
            } else {
                "active"
            }
        } else {
            "idle"
        },
        "tranche_status": match (&wl.run.tranche_run_id, &wl.current_task, &wl.status) {
            (Some(_), Some(_), _) => "active",
            (Some(_), None, focusa_core::types::WorkLoopStatus::AdvancingTask) => "completed",
            (Some(_), None, _) => "advancing",
            _ => "none",
        },
        "authorship_mode": wl.authorship_mode,
        "policy": wl.policy,
        "run": wl.run,
        "identity_summary": {
            "project_run_id": wl.run.project_run_id,
            "tranche_run_id": wl.run.tranche_run_id,
            "task_run_id": wl.run.task_run_id,
            "worker_session_id": wl.run.worker_session_id,
            "last_checkpoint_id": wl.run.last_checkpoint_id,
        },
        "current_task": wl.current_task,
        "last_completed_task_id": wl.last_completed_task_id,
        "execution_partition": work_loop_execution_partition_payload(wl, active_lease.as_ref(), &claim_key),
        "last_recorded_bd_transition_id": wl.last_recorded_bd_transition_id,
        "last_blocker_class": wl.last_blocker_class,
        "last_blocker_reason": wl.last_blocker_reason,
        "last_continue_reason": wl.last_continue_reason,
        "last_observed_summary": wl.last_observed_summary,
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "consecutive_failures_for_task_class": wl.consecutive_failures_for_task_class,
        "pause_flags": wl.pause_flags,
        "decision_context": wl.decision_context,
        "continuation_inputs": {
            "active_mission": { "intent": s.focus_stack.frames.iter().find(|f| Some(f.id) == s.focus_stack.active_id).map(|f| f.focus_state.intent.clone()), "frame_id": s.focus_stack.active_id },
            "current_ask": wl.decision_context.current_ask,
            "ask_kind": wl.decision_context.ask_kind,
            "scope_kind": wl.decision_context.scope_kind,
            "carryover_policy": wl.decision_context.carryover_policy,
            "excluded_context_reason": wl.decision_context.excluded_context_reason,
            "excluded_context_labels": wl.decision_context.excluded_context_labels,
            "operator_steering_detected": wl.decision_context.operator_steering_detected,
            "pending_proposals_requiring_resolution": wl.pending_proposals_requiring_resolution.max(pending_proposals),
            "autonomy_level": wl.current_autonomy_level.unwrap_or(s.autonomy.level),
            "autonomy_scope": s.autonomy.granted_scope,
            "verification_required": wl.current_task.as_ref().map(|task| task.required_verification_tier.clone()),
            "next_work_risk_class": wl.next_work_risk_class.clone().unwrap_or_else(|| next_work_risk_class.to_string()),
            "budget_caps": {
                "max_turns": wl.policy.max_turns,
                "max_wall_clock_ms": wl.policy.max_wall_clock_ms,
                "max_retries": wl.policy.max_retries,
                "max_consecutive_failures": wl.policy.max_consecutive_failures,
                "max_same_subproblem_retries": wl.policy.max_same_subproblem_retries,
            },
            "operator_overrides": wl.pause_flags,
            "recent_checkpoint_state": {
                "last_checkpoint_id": wl.run.last_checkpoint_id,
                "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
                "restored_context_summary": wl.restored_context_summary,
            },
            "active_workpoint": scoped_workpoint_summary_for_status(&s, &scope.0)
        },
        "delegated_authorship": wl.delegated_authorship,
        "transport": {
            "adapter": wl.transport_adapter,
            "session_state": wl.transport_session_state,
            "last_event_kind": wl.last_transport_event_kind,
            "last_event_summary": wl.last_transport_event_summary,
            "last_event_sequence": wl.last_transport_event_sequence,
            "abort_reason": wl.transport_abort_reason,
            "daemon_supervised_session": driver_snapshot,
        },
        "active_worker": wl.active_worker,
        "blocker_package": blocker_package,
        "active_writer": active_writer,
        "transport_health": transport_health,
        "execution_environment": execution_environment,
        "budget_remaining": budget_remaining,
        "supervisor_perf": supervisor_perf_payload(&state),
        "secondary_loop_quality_metrics": secondary_loop_quality_metrics,
        "secondary_loop_eval_artifacts": {
            "ledger_size": s.telemetry.secondary_loop_ledger.len(),
            "recent_entries": s
                .telemetry
                .secondary_loop_ledger
                .iter()
                .rev()
                .take(20)
                .cloned()
                .collect::<Vec<_>>(),
        },
        "secondary_loop_eval_bundle": secondary_loop_eval_bundle,
        "secondary_loop_acceptance_hooks": secondary_loop_acceptance_hooks,
        "secondary_loop_replay_consumer": secondary_loop_replay_consumer,
        "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
        "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        "secondary_loop_continuity_gate": secondary_loop_continuity_gate,
        "workpoint_replay_summary": workpoint_replay_summary,
        "resume_payload": resume_payload,
        "commitment_lifecycle": commitment_lifecycle,
        "governance": {
            "writer_header_required": WRITER_HEADER,
            "approval_header_required_for_enable": APPROVAL_HEADER,
            "explicit_enable_approval_required": true,
            "policy_owner": "daemon",
            "api_role": "dispatch_and_observability_only",
            "extension_role": "bridge_only_not_cognitive_authority",
            "llm_authority": "executor_only_unless_explicitly_delegated",
            "operator_override_supersedes_loop": true,
        },
        "worktree": worktree,
    });
    record_json_response_size("/v1/work-loop/status", &payload);
    Ok(Json(payload))
}

async fn status_deep(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let mut payload = {
        let s = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
        let wl = &s.work_loop;
        let claim_key = writer_claim_key_from_scope(&scope, &s);
        let active_lease = {
            let claims = state.writer_claims.read().await;
            active_writer_lease_for_key(&claims, &claim_key, Utc::now())
        };
        let active_writer = active_lease.as_ref().map(|lease| lease.writer_id.clone());
        json!({
            "route_tier": "cold",
            "summary_only": false,
            "cold_omitted": [],
            "enabled": wl.enabled,
            "status": wl.status,
            "project_status": if wl.enabled { "active" } else { "idle" },
            "authorship_mode": wl.authorship_mode,
            "policy": wl.policy,
            "run": wl.run,
            "current_task": wl.current_task,
            "last_completed_task_id": wl.last_completed_task_id,
            "last_blocker_class": wl.last_blocker_class,
            "last_blocker_reason": wl.last_blocker_reason,
            "last_continue_reason": wl.last_continue_reason,
            "decision_context": wl.decision_context,
            "active_writer": active_writer,
            "execution_partition": work_loop_execution_partition_payload(wl, active_lease.as_ref(), &claim_key),
            "active_workpoint": scoped_workpoint_summary_for_status(&s, &scope.0),
            "supervisor_perf": supervisor_perf_payload(&state),
            "deep_status_route": "/v1/work-loop/status/deep",
            "resource_mode": resource_mode_status(),
            "bounds": {
                "summary_only": false,
                "cold_diagnostics": true,
                "hot_safe": true,
                "note": "Deep route copies Focusa state before awaiting cold diagnostics so hot health/status readers are not held hostage."
            }
        })
    };

    let scope_root = request_scope_root(&scope);
    let worktree = worktree_status_snapshot(&scope_root).await;
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("worktree".into(), worktree);
    }
    record_json_response_size("/v1/work-loop/status/deep", &payload);
    Ok(Json(payload))
}

async fn closure_replay_evidence(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let s = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
    let wl = &s.work_loop;

    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &focusa_core::replay::ReplayConfig {
                from: None,
                until: None,
                session_id: s.session.as_ref().map(|session| session.session_id),
                frame_id: None,
            },
        )
        .map_err(|error| error.to_string());

    let replay_consumer_payload =
        secondary_loop_replay_consumer_payload_for_status(wl, &secondary_loop_replay_summary);
    let secondary_loop_continuity_gate = secondary_loop_continuity_gate_for_status(
        &secondary_loop_replay_summary,
        &replay_consumer_payload,
    );

    let mut payload = replay_consumer_payload;
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "secondary_loop_continuity_gate".to_string(),
            secondary_loop_continuity_gate,
        );
    }

    Ok(Json(payload))
}

async fn closure_replay_bundle(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let s = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
    let wl = &s.work_loop;

    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &focusa_core::replay::ReplayConfig {
                from: None,
                until: None,
                session_id: s.session.as_ref().map(|session| session.session_id),
                frame_id: None,
            },
        )
        .map_err(|error| error.to_string());

    Ok(Json(secondary_loop_closure_bundle_for_status(
        &s,
        wl,
        &secondary_loop_replay_summary,
    )))
}

async fn enable(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EnableWorkLoopRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "continuous work enable crosses a governance boundary and must be explicitly approved",
    )?;
    let preset = payload.preset.unwrap_or_default();
    let policy =
        WorkLoopPolicy::with_overrides(preset, payload.policy_overrides.unwrap_or_default());
    let parent_work_item_id = payload
        .root_work_item_id
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| bad_request("root_work_item_id is required"))?;
    let workpoint_id = {
        let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
        canonical_workpoint_id_for_scope_and_item(
            &focusa,
            &scope.0,
            Some(&parent_work_item_id),
        )
        .ok_or_else(|| {
            bad_request(
                "enable requires an active canonical Workpoint bound to the exact scope and root_work_item_id",
            )
        })?
    };
    let action = Action::EnableContinuousWork {
        project_run_id: payload.project_run_id.unwrap_or_else(Uuid::now_v7),
        policy,
        scope: scope.0.clone(),
        work_item_id: parent_work_item_id.clone(),
        workpoint_id,
    };
    let writer_lease =
        ensure_writer_claim_for_work_item(&scope, &state, &headers, &parent_work_item_id).await?;
    send_work_loop_action(&state, "work_loop_dispatch", action).await?;

    {
        send_work_loop_action(
            &state,
            "work_loop_select_next",
            Action::SelectNextContinuousSubtask {
                parent_work_item_id,
            },
        )
        .await?;
        let _ = maybe_dispatch_continuous_turn_prompt(
            &state,
            "continuous work enabled with ready work selected",
        )
        .await;
    }

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn pause(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;

    send_work_loop_action(
        &state,
        "work_loop_pause",
        Action::PauseContinuousWork {
            reason: payload.reason.unwrap_or_default(),
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn resume(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ResumeWorkLoopRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    if payload.renew_budget || payload.policy_overrides.is_some() {
        require_approval(
            &headers,
            "renewing or changing Work Loop budgets requires explicit approval",
        )?;
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let policy = if let Some(overrides) = payload.policy_overrides {
        let mut policy = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.work_loop.policy.clone();
        policy.apply_overrides(overrides);
        Some(policy)
    } else {
        None
    };

    send_work_loop_action(
        &state,
        "work_loop_resume",
        Action::ResumeContinuousWork {
            reason: payload.reason.unwrap_or_default(),
            renew_budget: payload.renew_budget,
            policy,
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn select_next(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SelectNextRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let parent_work_item_id = payload.parent_work_item_id.clone();

    send_work_loop_action(
        &state,
        "work_loop_select_next",
        Action::SelectNextContinuousSubtask {
            parent_work_item_id: payload.parent_work_item_id,
        },
    )
    .await?;
    {
        let mut focusa = state.focusa.write().await;
        if focusa.work_loop.pause_flags.governance_decision_pending {
            focusa.work_loop.last_blocker_class = Some(BlockerClass::Governance);
            focusa.work_loop.last_continue_reason = Some(
                "governance continuation boundary: paused select-next pending governing decision"
                    .to_string(),
            );
            let turn_id = focusa
                .work_loop
                .decision_context
                .source_turn_id
                .clone()
                .unwrap_or_else(|| "work_loop_select_next".to_string());
            focusa.telemetry.trace_events.push(json!({
                "event_id": Uuid::now_v7().to_string(),
                "event_type": "scope_failure_recorded",
                "timestamp": Utc::now().to_rfc3339(),
                "turn_id": turn_id,
                "payload": {
                    "event_type": "scope_failure_recorded",
                    "failure_kind": "governance_continuation_boundary",
                    "reason": "governance decision pending",
                    "path": "select_next_continuous_subtask",
                    "parent_work_item_id": parent_work_item_id,
                }
            }));
            state.mark_external_mutation();
        }
    }
    let _ = maybe_dispatch_continuous_turn_prompt(
        &state,
        "ready work selected for continuous execution",
    )
    .await;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn set_decision_context(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<DecisionContextRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_claimed_writer_matches_for_context(&scope, &state, &headers).await?;

    let event = FocusaEvent::ContinuousDecisionContextUpdated {
        current_ask: payload.current_ask,
        ask_kind: payload.ask_kind,
        scope_kind: payload.scope_kind,
        carryover_policy: payload.carryover_policy,
        excluded_context_reason: payload.excluded_context_reason,
        excluded_context_labels: payload.excluded_context_labels,
        source_turn_id: payload.source_turn_id,
        operator_steering_detected: payload.operator_steering_detected,
    };

    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_write_serial_lock"))?;
    let current = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        work_loop_failure(
            StatusCode::BAD_REQUEST,
            "work_loop_context",
            "reducer_rejected",
            error.to_string(),
        )
    })?;
    let new_state = result.new_state;
    let entry = EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id: Some("api:work_loop_context".to_string()),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    let _ = state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(json!({
        "status": "accepted",
        "writer_id": writer_lease.as_ref().map(|lease| lease.writer_id.as_str()),
        "fencing_token": writer_lease.as_ref().map(|lease| lease.fencing_token),
        "lease_expires_at": writer_lease.as_ref().map(|lease| lease.expires_at),
        "canonical": true,
        "materialized": true
    })))
}

async fn start_pi_driver(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PiDriverStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let (transport_work_item_id, transport_workpoint_id) = {
        let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
        if focusa.work_loop.execution_scope.as_ref() != Some(&scope.0) {
            return Err(bad_request(
                "Pi transport scope does not match the active Work Loop execution scope",
            ));
        }
        (
            focusa
                .work_loop
                .execution_work_item_id
                .clone()
                .ok_or_else(|| bad_request("active Work Loop root WorkItem is unbound"))?,
            focusa
                .work_loop
                .execution_workpoint_id
                .ok_or_else(|| bad_request("active Work Loop canonical Workpoint is unbound"))?,
        )
    };
    let transport_scope = scope.0.clone();

    let work_loop_root = request_scope_root(&scope);

    if payload.idempotency_key.trim().is_empty() {
        return Err(bad_request("idempotency_key must not be empty"));
    }
    let mut guard = state.pi_rpc_session.lock().await;
    if let Some(existing) = guard.as_ref() {
        if existing.idempotency_key == payload.idempotency_key {
            return Ok(Json(json!({
                "schema": "focusa.agent_execution_adapter_result.v1",
                "status": "accepted",
                "adapter": "pi-rpc",
                "session_id": existing.session_id,
                "resumable": true,
                "idempotent_replay": true,
                "authority": "focusa.spec133.work_loop",
                "tool_result": agent_execution_tool_result("Pi RPC execution already active", "none"),
            })));
        }
        return Err(conflict(
            "pi rpc driver already active",
            Some(writer_lease.writer_id.clone()),
        ));
    }

    let session_id = format!("pi-rpc-{}", Uuid::now_v7());
    let mut cmd = Command::new(pi_rpc_bin());
    let base_path = std::env::var("PATH").unwrap_or_default();
    let merged_path = if let Some(node_bin_dir) = pi_rpc_node_bin_dir() {
        if base_path.split(':').any(|segment| segment == node_bin_dir) {
            base_path
        } else if base_path.is_empty() {
            node_bin_dir
        } else {
            format!("{node_bin_dir}:{base_path}")
        }
    } else {
        base_path
    };

    cmd.env("PATH", merged_path)
        .env(
            "FOCUSA_PI_API_BASE_URL",
            pi_focusa_api_base_url(&state.config.api_bind),
        )
        .env(
            "FOCUSA_PI_VITAL_INFO_PROMPT_MODE",
            PI_HEADLESS_VITAL_INFO_PROMPT_MODE,
        )
        .args([
            "--mode",
            "rpc",
            "--no-session",
            "--no-extensions",
            "--no-skills",
            "--no-prompt-templates",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    configure_pi_rpc_process_group(&mut cmd);
    configure_pi_rpc_invocation(&mut cmd, &payload);
    if payload.cwd.as_deref().is_some_and(|cwd| {
        Path::new(cwd).components().collect::<PathBuf>()
            != work_loop_root.components().collect::<PathBuf>()
    }) {
        return Err(work_loop_failure(
            StatusCode::CONFLICT,
            "pi_driver_start",
            "scope_mismatch",
            "driver cwd must match the request WorkstreamKey project root".into(),
        ));
    }
    cmd.current_dir(&work_loop_root);

    let mut child = cmd.spawn().map_err(work_loop_pi_spawn_failed)?;
    let child_pid = child
        .id()
        .ok_or_else(|| bad_request("pi rpc process id unavailable after spawn"))?;
    spawn_pi_rpc_parent_watchdog(child_pid);
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| bad_request("pi rpc stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| bad_request("pi rpc stdout unavailable"))?;
    let stderr = child.stderr.take();

    let state_for_events = state.clone();
    let command_tx = state.command_tx.clone();
    let attach_session_id = session_id.clone();
    command_tx
        .send(Action::AttachContinuousTransportSession {
            adapter: "pi-rpc".to_string(),
            session_id: attach_session_id.clone(),
            scope: transport_scope,
            work_item_id: transport_work_item_id,
            workpoint_id: transport_workpoint_id,
        })
        .await
        .map_err(|error| work_loop_dispatch_failed("work_loop_transport_attach", error))?;

    if let Some(stderr_stream) = stderr {
        let stderr_command_tx = command_tx.clone();
        let stderr_session_id = attach_session_id.clone();
        tokio::spawn(async move {
            let mut stderr_seq: u64 = 1;
            let mut err_lines = BufReader::new(stderr_stream).lines();
            while let Ok(Some(line)) = err_lines.next_line().await {
                let _ = stderr_command_tx
                    .send(Action::IngestContinuousTransportEvent {
                        sequence: stderr_seq,
                        kind: "stderr_line".to_string(),
                        session_id: Some(stderr_session_id.clone()),
                        turn_id: None,
                        summary: Some(line),
                    })
                    .await;
                stderr_seq = stderr_seq.saturating_add(1);
            }
        });
    }

    *guard = Some(crate::server::PiRpcSession {
        child,
        process_group_id: child_pid,
        stdin,
        session_id: session_id.clone(),
        cwd: Some(work_loop_root.to_string_lossy().to_string()),
        idempotency_key: payload.idempotency_key.clone(),
        started_at: std::time::Instant::now(),
    });

    tokio::spawn(async move {
        let mut seq: u64 = 1;
        let mut last_assistant_output = String::new();
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let parsed: Value = serde_json::from_str(&line)
                .unwrap_or_else(|_| json!({"type":"raw","summary":line}));
            let kind = parsed
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            if let Some(response) = extension_ui_response(&parsed, &work_loop_root) {
                let encoded = format!("{}\n", response);
                let mut session_guard = state_for_events.pi_rpc_session.lock().await;
                if let Some(session) = session_guard.as_mut()
                    && session.session_id == attach_session_id
                {
                    let _ = session.stdin.write_all(encoded.as_bytes()).await;
                    let _ = session.stdin.flush().await;
                }
            }
            if kind == "turn_start" || kind == "agent_start" {
                last_assistant_output.clear();
            }
            if kind == "message_update"
                && let Some(delta) = parsed
                    .get("assistantMessageEvent")
                    .and_then(|v| v.get("delta"))
                    .and_then(Value::as_str)
            {
                last_assistant_output.push_str(delta);
            }
            if (kind == "turn_end" || kind == "agent_end") && last_assistant_output.is_empty() {
                if let Some(text) = parsed.get("message").and_then(extract_assistant_text) {
                    last_assistant_output = text;
                } else if let Some(text) = parsed
                    .get("messages")
                    .and_then(Value::as_array)
                    .and_then(|msgs| msgs.iter().rev().find_map(extract_assistant_text))
                {
                    last_assistant_output = text;
                }
            }
            let summary = parsed
                .get("message")
                .and_then(|m| m.get("role").and_then(Value::as_str).or_else(|| m.as_str()))
                .map(|s| s.to_string())
                .or_else(|| {
                    parsed
                        .get("assistantMessageEvent")
                        .and_then(|v| v.get("type"))
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    parsed
                        .get("method")
                        .and_then(Value::as_str)
                        .map(|method| format!("{kind}:{method}"))
                })
                .or_else(|| {
                    parsed
                        .get("command")
                        .and_then(Value::as_str)
                        .map(|s| format!("response:{s}"))
                })
                .or_else(|| Some(kind.clone()));
            let _ = command_tx
                .send(Action::IngestContinuousTransportEvent {
                    sequence: seq,
                    kind: kind.clone(),
                    session_id: Some(attach_session_id.clone()),
                    turn_id: None,
                    summary,
                })
                .await;
            if matches!(
                kind.as_str(),
                "session_compact" | "compaction_end" | "session_compact_end"
            ) {
                let _ = maybe_dispatch_continuous_turn_prompt(
                    &state_for_events,
                    "pi rpc compaction completed; dispatching automatic continuation prompt",
                )
                .await;
            }
            if kind == "agent_end" {
                let current_task = {
                    let focusa = state_for_events.focusa.read().await;
                    focusa.work_loop.current_task.clone()
                };
                if let Some(task) = current_task {
                    let assistant_output = last_assistant_output.trim();
                    let has_assistant_output = !assistant_output.is_empty();
                    if has_assistant_output {
                        let assistant_excerpt =
                            assistant_output.chars().take(220).collect::<String>();
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
                        let spec_conformant = receipt
                            .as_ref()
                            .filter(|_| receipt_matches)
                            .is_some_and(|receipt| receipt.spec_conformant);
                        let verification_satisfied = outcome_status
                            == WorkLoopOutcomeStatus::Completed
                            && !evidence_citations.is_empty();
                        let summary = receipt
                            .as_ref()
                            .filter(|_| receipt_matches)
                            .and_then(|receipt| receipt.summary.clone())
                            .unwrap_or_else(|| {
                                format!("{kind} for {}: {assistant_excerpt}", task.work_item_id)
                            });
                        let _ = command_tx
                            .send(Action::ObserveContinuousTurnOutcome {
                                task_run_id: None,
                                work_item_id: Some(task.work_item_id.clone()),
                                summary,
                                continue_reason: Some(format!(
                                    "{kind} observed from pi rpc stream: {assistant_excerpt}"
                                )),
                                verification_satisfied,
                                spec_conformant,
                                outcome_status,
                                evidence_citations,
                            })
                            .await;
                        let _ = maybe_dispatch_continuous_turn_prompt(
                            &state_for_events,
                            "pi rpc agent_end observed and ready work remains",
                        )
                        .await;
                    } else {
                        let _ = maybe_dispatch_continuous_turn_prompt(
                            &state_for_events,
                            "pi rpc turn ended without assistant output (compaction/housekeeping); auto-retrying",
                        )
                        .await;
                    }
                }
                last_assistant_output.clear();
            }
            seq += 1;
        }

        let stale_session = {
            let mut session_guard = state_for_events.pi_rpc_session.lock().await;
            if session_guard
                .as_ref()
                .is_some_and(|session| session.session_id == attach_session_id)
            {
                session_guard.take()
            } else {
                None
            }
        };
        if let Some(mut stale_session) = stale_session {
            terminate_pi_rpc_child(&mut stale_session.child, stale_session.process_group_id).await;
        }

        let _ = command_tx
            .send(Action::IngestContinuousTransportEvent {
                sequence: seq,
                kind: "stream_closed".to_string(),
                session_id: Some(attach_session_id.clone()),
                turn_id: None,
                summary: Some("pi rpc stdout stream closed".to_string()),
            })
            .await;
        let _ = command_tx
            .send(Action::MarkContinuousLoopTransportDegraded {
                reason: "pi rpc stdout stream closed; restart required".to_string(),
            })
            .await;
    });

    Ok(Json(json!({
        "schema": "focusa.agent_execution_adapter_result.v1",
        "status": "accepted",
        "adapter": "pi-rpc",
        "session_id": session_id,
        "resumable": true,
        "resumed_from": payload.resume_session,
        "workpoint_id": payload.workpoint_id,
        "cancellation": {"abort_route": "/v1/work-loop/driver/abort", "stop_route": "/v1/work-loop/driver/stop"},
        "authority": "focusa.spec133.work_loop",
        "tool_result": agent_execution_tool_result("Pi RPC execution started or resumed", "process_started"),
    })))
}

async fn prompt_pi_driver(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PiDriverPromptRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    ensure_writer_claim(&scope, &state, &headers).await?;
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    if payload.message.trim().is_empty() {
        return Err(bad_request("pi rpc prompt message must not be empty"));
    }
    if let Some(streaming_behavior) = payload.streaming_behavior.as_deref()
        && !matches!(streaming_behavior, "steer" | "followUp")
    {
        return Err(bad_request("streaming_behavior must be steer or followUp"));
    }
    let msg = if let Some(streaming_behavior) = payload.streaming_behavior.as_deref() {
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": payload.message, "streamingBehavior": streaming_behavior})
    } else {
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": payload.message})
    };
    session
        .stdin
        .write_all(msg.to_string().as_bytes())
        .await
        .map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    session
        .stdin
        .write_all(b"\n")
        .await
        .map_err(|e| bad_request(format!("failed writing newline: {e}")))?;
    Ok(Json(json!({
        "schema": "focusa.agent_execution_adapter_result.v1",
        "status": "accepted",
        "adapter": "pi-rpc",
        "session_id": session.session_id,
        "resumable": true,
        "authority": "focusa.spec133.work_loop",
        "tool_result": agent_execution_tool_result("Prompt accepted by Pi RPC", "prompt_accepted"),
    })))
}

async fn abort_pi_driver(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    ensure_writer_claim(&scope, &state, &headers).await?;
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(mut session) = guard.take() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg = json!({"type":"abort"}).to_string() + "\n";
    let _ = session.stdin.write_all(msg.as_bytes()).await;
    terminate_pi_rpc_child(&mut session.child, session.process_group_id).await;
    Ok(Json(json!({
        "schema": "focusa.agent_execution_adapter_result.v1",
        "status": "accepted",
        "adapter": "pi-rpc",
        "session_id": session.session_id,
        "resumable": true,
        "cancelled": true,
        "process_tree_terminated": true,
        "authority": "focusa.spec133.work_loop",
        "tool_result": agent_execution_tool_result("Pi RPC turn aborted", "turn_aborted"),
    })))
}

async fn stop_pi_driver(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    ensure_writer_claim(&scope, &state, &headers).await?;
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(mut session) = guard.take() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    terminate_pi_rpc_child(&mut session.child, session.process_group_id).await;
    Ok(Json(json!({
        "schema": "focusa.agent_execution_adapter_result.v1",
        "status": "stopped",
        "adapter": "pi-rpc",
        "session_id": session.session_id,
        "resumable": true,
        "cancelled": true,
        "process_tree_terminated": true,
        "authority": "focusa.spec133.work_loop",
        "tool_result": agent_execution_tool_result("Pi RPC execution stopped", "process_stopped"),
    })))
}

async fn attach_session(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SessionAttachRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let (work_item_id, workpoint_id) = {
        let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
        if focusa.work_loop.execution_scope.as_ref() != Some(&scope.0) {
            return Err(bad_request(
                "transport session scope does not match active Work Loop execution scope",
            ));
        }
        (
            focusa
                .work_loop
                .execution_work_item_id
                .clone()
                .ok_or_else(|| bad_request("active Work Loop root WorkItem is unbound"))?,
            focusa
                .work_loop
                .execution_workpoint_id
                .ok_or_else(|| bad_request("active Work Loop Workpoint is unbound"))?,
        )
    };
    let event = FocusaEvent::ContinuousTransportSessionAttached {
        adapter: payload.adapter,
        session_id: payload.session_id,
        scope: scope.0.clone(),
        work_item_id,
        workpoint_id,
    };
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_write_serial_lock"))?;
    let current = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        work_loop_failure(
            StatusCode::BAD_REQUEST,
            "work_loop_attach_session",
            "reducer_rejected",
            error.to_string(),
        )
    })?;
    let new_state = result.new_state;
    let entry = EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id: Some("api:work_loop_attach_session".to_string()),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    let _ = state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn abort_session(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let event = FocusaEvent::ContinuousTransportAbortForwarded {
        reason: payload
            .reason
            .unwrap_or_else(|| "abort requested".to_string()),
    };
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_write_serial_lock"))?;
    let current = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        work_loop_failure(
            StatusCode::BAD_REQUEST,
            "work_loop_abort_session",
            "reducer_rejected",
            error.to_string(),
        )
    })?;
    let new_state = result.new_state;
    let entry = EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id: Some("api:work_loop_abort_session".to_string()),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    let _ = state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn ingest_transport_event(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<TransportEventRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_ingest_transport_lock"))?;
    send_work_loop_action(
        &state,
        "work_loop_ingest_transport",
        Action::IngestContinuousTransportEvent {
            sequence: payload.sequence,
            kind: payload.kind,
            session_id: payload.session_id,
            turn_id: payload.turn_id,
            summary: payload.summary,
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn set_pause_flags(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PauseFlagsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let event = FocusaEvent::ContinuousPauseFlagsUpdated {
        destructive_confirmation_required: payload.destructive_confirmation_required,
        governance_decision_pending: payload.governance_decision_pending,
        operator_override_active: payload.operator_override_active,
        reason: payload.reason,
    };
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_write_serial_lock"))?;
    let current = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        work_loop_failure(
            StatusCode::BAD_REQUEST,
            "work_loop_pause_flags",
            "reducer_rejected",
            error.to_string(),
        )
    })?;
    let mut new_state = result.new_state;
    if new_state.work_loop.pause_flags.governance_decision_pending {
        new_state.work_loop.last_blocker_class = Some(BlockerClass::Governance);
        new_state.work_loop.last_continue_reason = Some(
            "governance continuation boundary: paused select-next pending governing decision"
                .to_string(),
        );
        if new_state.work_loop.last_blocker_reason.is_none() {
            new_state.work_loop.last_blocker_reason =
                Some("governance decision pending".to_string());
        }
    }
    let entry = EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id: Some("api:work_loop_pause_flags".to_string()),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    let _ = state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn delegate_authorship(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<DelegationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "delegated authorship changes authority state and requires explicit approval",
    )?;
    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    send_work_loop_action(
        &state,
        "work_loop_delegate_authorship",
        Action::SetDelegatedContinuousAuthorship {
            delegate_id: Some(payload.delegate_id),
            scope: Some(payload.scope),
            amendment_summary: payload.amendment_summary,
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn clear_delegated_authorship(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "clearing delegated authorship changes authority state and requires explicit approval",
    )?;
    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    send_work_loop_action(
        &state,
        "work_loop_clear_delegated_authorship",
        Action::SetDelegatedContinuousAuthorship {
            delegate_id: None,
            scope: None,
            amendment_summary: payload.reason,
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn transport_degraded(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    send_work_loop_action(
        &state,
        "work_loop_transport_degraded",
        Action::MarkContinuousLoopTransportDegraded {
            reason: payload
                .reason
                .unwrap_or_else(|| "transport degraded".to_string()),
        },
    )
    .await?;

    Ok(Json(
        json!({ "ok": true, "writer_id": writer_lease.writer_id, "fencing_token": writer_lease.fencing_token, "lease_expires_at": writer_lease.expires_at }),
    ))
}

async fn checkpoints(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await;
    let wl = &focusa.work_loop;
    Ok(Json(json!({
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "resume_payload": resume_payload_for_status(&focusa, wl, &scope.0),
        "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
        "restored_context_summary": wl.restored_context_summary,
        "last_continue_reason": wl.last_continue_reason,
        "last_blocker_reason": wl.last_blocker_reason,
    })))
}

async fn heartbeat(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let dispatched =
        maybe_dispatch_continuous_turn_prompt(&state, "daemon heartbeat supervisor tick").await?;

    Ok(Json(json!({
        "ok": true,
        "writer_id": writer_lease.writer_id,
        "fencing_token": writer_lease.fencing_token,
        "lease_expires_at": writer_lease.expires_at,
        "dispatched": dispatched,
    })))
}

async fn checkpoint(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_lease = ensure_writer_claim(&scope, &state, &headers).await?;
    let checkpoint_id = payload.checkpoint_id.unwrap_or_else(Uuid::now_v7);
    let summary = payload.summary;
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| work_loop_dispatch_timeout("work_loop_checkpoint_write_lock"))?;
    if state
        .persistence
        .event_exists(&checkpoint_id.to_string())
        .map_err(|error| {
            work_loop_failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "work_loop_checkpoint",
                "persistence_failed",
                format!("checkpoint idempotency lookup failed: {error}"),
            )
        })?
    {
        return Ok(Json(json!({
            "ok": true,
            "idempotent_replay": true,
            "checkpoint_id": checkpoint_id,
            "writer_id": writer_lease.writer_id,
            "fencing_token": writer_lease.fencing_token,
            "lease_expires_at": writer_lease.expires_at,
        })));
    }

    let current = { crate::workstream_store::scoped_focusa_read_workstream(state.clone(), &scope.0).await.clone() };
    let event = FocusaEvent::ContinuousLoopRecoveryCheckpointed {
        checkpoint_id,
        summary,
    };
    let machine_id = state.persistence.machine_id().ok();
    let result = focusa_core::reducer::reduce_with_meta(
        current,
        event.clone(),
        machine_id.as_deref(),
        None,
        false,
    )
    .map_err(|error| {
        work_loop_failure(
            StatusCode::BAD_REQUEST,
            "work_loop_checkpoint",
            "reducer_rejected",
            error.to_string(),
        )
    })?;
    let new_state = result.new_state;
    let entry = EventLogEntry {
        id: checkpoint_id,
        timestamp: Utc::now(),
        event,
        correlation_id: Some(format!(
            "work_loop_checkpoint|project_root={}|continuity_id={}",
            scope.0.root_scope.root_path.display(),
            scope.0.continuity_id
        )),
        origin: SignalOrigin::Cli,
        machine_id,
        instance_id: None,
        session_id: new_state.session.as_ref().map(|session| session.session_id),
        thread_id: None,
        is_observation: false,
    };
    state
        .persist_events_checkpoint(vec![entry.clone()], new_state.clone())
        .await
        .map_err(|error| {
            work_loop_failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "work_loop_checkpoint",
                "persistence_failed",
                format!("atomic checkpoint commit failed: {error}"),
            )
        })?;
    if let Ok(serialized) = serde_json::to_string(&entry) {
        let _ = state.events_tx.send(serialized);
    }
    *state.focusa.write().await = new_state;
    state.mark_external_mutation();

    Ok(Json(json!({
        "ok": true,
        "idempotent_replay": false,
        "checkpoint_id": checkpoint_id,
        "writer_id": writer_lease.writer_id,
        "fencing_token": writer_lease.fencing_token,
        "lease_expires_at": writer_lease.expires_at,
    })))
}

async fn stop(
    scope: WorkLoopScope,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let released_writer = release_writer_claim(&scope, &state, &headers).await?;
    send_work_loop_action(
        &state,
        "work_loop_stop",
        Action::StopContinuousWork {
            reason: payload.reason.unwrap_or_default(),
        },
    )
    .await?;

    Ok(Json(json!({
        "ok": true,
        "released_writer": released_writer.as_ref().map(|lease| lease.writer_id.as_str()),
        "released_fencing_token": released_writer.as_ref().map(|lease| lease.fencing_token),
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/work-loop", get(status))
        .route("/v1/work-loop/health", get(health))
        .route("/v1/work-loop/status", get(status))
        .route("/v1/work-loop/status/deep", get(status_deep))
        .route(
            "/v1/work-loop/replay/closure-evidence",
            get(closure_replay_evidence),
        )
        .route(
            "/v1/work-loop/replay/closure-bundle",
            get(closure_replay_bundle),
        )
        .route("/v1/work-loop/enable", post(enable))
        .route("/v1/work-loop/pause", post(pause))
        .route("/v1/work-loop/resume", post(resume))
        .route("/v1/work-loop/select-next", post(select_next))
        .route("/v1/work-loop/context", post(set_decision_context))
        .route("/v1/work-loop/driver/start", post(start_pi_driver))
        .route("/v1/work-loop/driver/prompt", post(prompt_pi_driver))
        .route("/v1/work-loop/driver/abort", post(abort_pi_driver))
        .route("/v1/work-loop/driver/stop", post(stop_pi_driver))
        .route("/v1/work-loop/session/attach", post(attach_session))
        .route("/v1/work-loop/session/abort", post(abort_session))
        .route("/v1/work-loop/events", post(ingest_transport_event))
        .route("/v1/work-loop/pause-flags", post(set_pause_flags))
        .route("/v1/work-loop/delegation/enable", post(delegate_authorship))
        .route(
            "/v1/work-loop/delegation/clear",
            post(clear_delegated_authorship),
        )
        .route("/v1/work-loop/degraded", post(transport_degraded))
        .route("/v1/work-loop/checkpoints", get(checkpoints))
        .route("/v1/work-loop/checkpoint", post(checkpoint))
        .route("/v1/work-loop/heartbeat", post(heartbeat))
        .route("/v1/work-loop/stop", post(stop))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::scoped_state::ScopeRef;

    #[test]
    fn extension_ui_dialogs_receive_safe_or_fail_closed_matching_responses() {
        let root = Path::new("/project");
        let skip = extension_ui_response(
            &json!({
                "type": "extension_ui_request",
                "id": "request-skip",
                "method": "select",
                "options": ["A) Define trajectory", "F) Skip — leave warning active"]
            }),
            root,
        )
        .unwrap();
        assert_eq!(skip["value"], "F) Skip — leave warning active");

        let ambiguous = extension_ui_response(
            &json!({
                "type": "extension_ui_request",
                "id": "request-ambiguous",
                "method": "select",
                "options": ["Skip once", "Skip always"]
            }),
            root,
        )
        .unwrap();
        assert_eq!(ambiguous["cancelled"], true);

        for method in ["confirm", "input", "editor"] {
            let response = extension_ui_response(
                &json!({
                    "type": "extension_ui_request",
                    "id": "request-1",
                    "method": method
                }),
                root,
            )
            .unwrap();
            assert_eq!(response["type"], "extension_ui_response");
            assert_eq!(response["id"], "request-1");
            assert_eq!(response["cancelled"], true);
        }
        for method in [
            "notify",
            "setStatus",
            "setWidget",
            "setTitle",
            "set_editor_text",
        ] {
            assert!(
                extension_ui_response(
                    &json!({
                        "type": "extension_ui_request",
                        "id": "request-2",
                        "method": method
                    }),
                    root,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn spawned_pi_uses_owning_daemon_endpoint_not_installed_default() {
        assert_eq!(PI_HEADLESS_VITAL_INFO_PROMPT_MODE, "warn_only");
        assert_eq!(
            pi_focusa_api_base_url("127.0.0.1:18787"),
            "http://127.0.0.1:18787/v1"
        );
        assert_eq!(
            pi_focusa_api_base_url("0.0.0.0:8787"),
            "http://127.0.0.1:8787/v1"
        );
        assert_eq!(
            pi_focusa_api_base_url("[::]:8788"),
            "http://127.0.0.1:8788/v1"
        );
    }

    #[test]
    fn writer_scope_rejects_host_and_cross_continuity_authority() {
        let project = ScopeRef::project(
            "project:focusa",
            "/home/wirebot/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let exact = WorkstreamKey::new(project.clone(), "cont-focusa").unwrap();
        let other_continuity = WorkstreamKey::new(project, "cont-other").unwrap();
        let host =
            ScopeRef::host("host:operator", "/root", "operator-host", "sha256:host").unwrap();
        let host_key = WorkstreamKey::new(host, "cont-focusa").unwrap();

        assert!(work_loop_scope_matches(
            &exact,
            "/home/wirebot/focusa",
            "cont-focusa"
        ));
        assert!(!work_loop_scope_matches(
            &other_continuity,
            "/home/wirebot/focusa",
            "cont-focusa"
        ));
        assert!(!work_loop_scope_matches(
            &host_key,
            "/home/wirebot/focusa",
            "cont-focusa"
        ));
    }

    #[test]
    fn canonical_workpoint_resolution_is_partitioned_and_ignores_singleton_active_id() {
        let mut state = focusa_core::types::FocusaState::default();
        let focusa_id = Uuid::now_v7();
        let other_id = Uuid::now_v7();
        state.workpoint.active_workpoint_id = Some(other_id);
        state.workpoint.records.extend([
            focusa_core::types::WorkpointRecord {
                workpoint_id: focusa_id,
                canonical: true,
                status: focusa_core::types::WorkpointStatus::Active,
                project_root: Some("/home/wirebot/focusa".to_string()),
                continuity_id: Some("cont-focusa".to_string()),
                work_item_id: Some("focusa-root".to_string()),
                ..focusa_core::types::WorkpointRecord::default()
            },
            focusa_core::types::WorkpointRecord {
                workpoint_id: other_id,
                canonical: true,
                status: focusa_core::types::WorkpointStatus::Active,
                project_root: Some("/home/wirebot/other".to_string()),
                continuity_id: Some("cont-other".to_string()),
                ..focusa_core::types::WorkpointRecord::default()
            },
        ]);

        let focusa = ScopeRef::project(
            "project:focusa",
            "/home/wirebot/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let focusa_key = WorkstreamKey::new(focusa, "cont-focusa").unwrap();
        let missing_key = WorkstreamKey::new(
            ScopeRef::project(
                "project:focusa",
                "/home/wirebot/focusa",
                "Focusa",
                "sha256:focusa",
            )
            .unwrap(),
            "cont-missing",
        )
        .unwrap();

        assert!(canonical_workpoint_exists_for_scope(&state, &focusa_key));
        assert_eq!(
            canonical_workpoint_id_for_scope_and_item(&state, &focusa_key, Some("focusa-root")),
            Some(focusa_id)
        );
        assert_eq!(
            canonical_workpoint_id_for_scope_and_item(&state, &focusa_key, Some("wrong-root")),
            None
        );
        assert!(!canonical_workpoint_exists_for_scope(&state, &missing_key));
        let summary = scoped_workpoint_summary_for_status(&state, &focusa_key);
        assert_eq!(summary["active_workpoint_id"], json!(focusa_id));
    }

    #[test]
    fn work_loop_root_comes_only_from_typed_execution_scope() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.execution_scope = Some(sample_workstream_key("cont-focusa"));
        state.work_loop.current_task = Some(focusa_core::types::SpecLinkedTaskPacket {
            allowed_scope: vec!["project_root:/home/wirebot/other".to_string()],
            ..sample_current_task("focusa-workloop-completion.2")
        });

        assert_eq!(
            work_loop_scope_root(&state),
            Some(PathBuf::from("/home/wirebot/focusa"))
        );
        state.work_loop.execution_scope = None;
        assert_eq!(work_loop_scope_root(&state), None);
    }

    #[test]
    fn writer_claim_keys_isolate_project_continuity_and_work_item() {
        let base = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-focusa",
            "focusa-workloop-completion.1",
        );
        let other_project = writer_claim_key_for_partition(
            "/home/wirebot/other",
            "cont-focusa",
            "focusa-workloop-completion.1",
        );
        let other_continuity = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-other",
            "focusa-workloop-completion.1",
        );
        let other_work_item = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-focusa",
            "focusa-workloop-completion.2",
        );

        assert_ne!(base, other_project);
        assert_ne!(base, other_continuity);
        assert_ne!(base, other_work_item);
        assert_eq!(
            base,
            "project:/home/wirebot/focusa|workstream:cont-focusa|work_item:focusa-workloop-completion.1"
        );
    }

    #[test]
    fn writer_lookup_never_falls_back_across_partitions() {
        let now = Utc::now();
        let claimed_key = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-focusa",
            "focusa-workloop-completion.1",
        );
        let unclaimed_key = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-other",
            "focusa-workloop-completion.1",
        );
        let mut claims = std::collections::HashMap::new();
        acquire_writer_for_key(
            &mut claims,
            claimed_key.clone(),
            "writer-focusa-cont-1".to_string(),
            1,
            now,
        )
        .unwrap();

        assert_eq!(
            active_writer_compat(&claims, &claimed_key, now).as_deref(),
            Some("writer-focusa-cont-1")
        );
        assert_eq!(active_writer_compat(&claims, &unclaimed_key, now), None);
    }

    #[test]
    fn writer_claim_runtime_isolates_concurrent_partitions() {
        let now = Utc::now();
        let project_continuity_one = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-one",
            "focusa-workloop-completion.1",
        );
        let project_continuity_two = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-two",
            "focusa-workloop-completion.1",
        );
        let second_work_item = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-one",
            "focusa-workloop-completion.2",
        );
        let mut claims = std::collections::HashMap::new();

        for (key, writer, token) in [
            (project_continuity_one.clone(), "writer-one", 1),
            (project_continuity_two.clone(), "writer-two", 2),
            (second_work_item.clone(), "writer-three", 3),
        ] {
            let lease =
                acquire_writer_for_key(&mut claims, key, writer.to_string(), token, now).unwrap();
            assert_eq!(lease.writer_id, writer);
            assert_eq!(lease.fencing_token, token);
        }
        assert_eq!(claims.len(), 3);
        assert_eq!(
            active_writer_for_key(&claims, &project_continuity_one, now).as_deref(),
            Some("writer-one")
        );
        assert_eq!(
            active_writer_for_key(&claims, &project_continuity_two, now).as_deref(),
            Some("writer-two")
        );
        assert_eq!(
            active_writer_for_key(&claims, &second_work_item, now).as_deref(),
            Some("writer-three")
        );

        let rejected = acquire_writer_for_key(
            &mut claims,
            project_continuity_one,
            "late-writer".to_string(),
            4,
            now,
        )
        .unwrap_err();
        assert_eq!(rejected.0, StatusCode::CONFLICT);
        assert_eq!(claims.len(), 3);
    }

    #[test]
    fn expired_lease_takeover_fences_late_writer() {
        let acquired_at = Utc::now();
        let after_expiry = acquired_at + chrono::Duration::milliseconds(WRITER_LEASE_TTL_MS + 1);
        let key = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-one",
            "focusa-workloop-completion.3",
        );
        let mut claims = std::collections::HashMap::new();
        let first = acquire_writer_for_key(
            &mut claims,
            key.clone(),
            "writer-one".to_string(),
            41,
            acquired_at,
        )
        .unwrap();
        let replacement = acquire_writer_for_key(
            &mut claims,
            key.clone(),
            "writer-two".to_string(),
            42,
            after_expiry,
        )
        .unwrap();

        assert!(replacement.fencing_token > first.fencing_token);
        let late = validate_and_renew_writer_for_key(
            &mut claims,
            &key,
            &first.writer_id,
            first.fencing_token,
            after_expiry,
        )
        .unwrap_err();
        assert_eq!(late.0, StatusCode::CONFLICT);
        assert!(
            validate_and_renew_writer_for_key(
                &mut claims,
                &key,
                &replacement.writer_id,
                replacement.fencing_token,
                after_expiry,
            )
            .is_ok()
        );
    }

    #[test]
    fn execution_partition_payload_reports_claimed_work_item_key() {
        let wl = focusa_core::types::WorkLoopState::default();
        let claim_key = writer_claim_key_for_partition(
            "/home/wirebot/focusa",
            "cont-one",
            "focusa-workloop-completion.1",
        );
        let now = Utc::now();
        let lease = WriterLease {
            writer_id: "writer-one".to_string(),
            fencing_token: 42,
            acquired_at: now,
            renewed_at: now,
            expires_at: writer_lease_expiry(now),
        };
        let payload = work_loop_execution_partition_payload(&wl, Some(&lease), &claim_key);

        assert_eq!(payload["project_root_key"], json!("/home/wirebot/focusa"));
        assert_eq!(payload["workstream_key"], json!("cont-one"));
        assert_eq!(
            payload["work_item_key"],
            json!("focusa-workloop-completion.1")
        );
        assert_eq!(payload["current_task_work_item_id"], Value::Null);
        assert_eq!(payload["writer_key"], json!("writer-one"));
        assert_eq!(payload["fencing_token"], json!(42));
        assert_eq!(payload["lease_freshness"], json!("current"));
    }

    #[test]
    fn typed_status_states_distinguish_zero_absent_stale_blocked_unavailable_and_healthy() {
        let mut wl = focusa_core::types::WorkLoopState::default();
        let now = Utc::now();
        let lease = WriterLease {
            writer_id: "writer-one".to_string(),
            fencing_token: 42,
            acquired_at: now,
            renewed_at: now,
            expires_at: writer_lease_expiry(now),
        };

        assert_eq!(work_loop_status_surface_state(&wl, None, None), "zero");
        wl.enabled = true;
        assert_eq!(work_loop_status_surface_state(&wl, None, None), "absent");
        assert_eq!(
            work_loop_status_surface_state(&wl, None, Some(&lease)),
            "healthy"
        );
        assert_eq!(
            work_loop_status_surface_state(&wl, Some("operator boundary"), Some(&lease)),
            "blocked"
        );
        wl.status = focusa_core::types::WorkLoopStatus::TransportDegraded;
        assert_eq!(
            work_loop_status_surface_state(&wl, None, Some(&lease)),
            "unavailable"
        );
        wl.enabled = false;
        wl.status = focusa_core::types::WorkLoopStatus::Idle;
        assert_eq!(
            work_loop_status_surface_state(&wl, None, Some(&lease)),
            "stale"
        );
    }

    #[test]
    fn typed_status_compatibility_fails_closed_on_unknown_schema_or_state() {
        assert_eq!(
            compatible_typed_surface_state(
                WORK_LOOP_STATUS_SCHEMA,
                WORK_LOOP_STATUS_SCHEMA,
                "healthy"
            ),
            "healthy"
        );
        for state in WORK_LOOP_TYPED_STATES {
            assert_eq!(
                compatible_typed_surface_state(
                    WORK_LOOP_STATUS_SCHEMA,
                    WORK_LOOP_STATUS_SCHEMA,
                    state
                ),
                state
            );
        }
        assert_eq!(
            compatible_typed_surface_state(
                "focusa.work_loop_status.v999",
                WORK_LOOP_STATUS_SCHEMA,
                "healthy"
            ),
            "unsupported"
        );
        assert_eq!(
            compatible_typed_surface_state(
                WORK_LOOP_STATUS_SCHEMA,
                WORK_LOOP_STATUS_SCHEMA,
                "maybe"
            ),
            "unsupported"
        );
    }

    #[test]
    fn writer_claim_key_fails_closed_without_complete_partition() {
        assert_eq!(
            writer_claim_key_for_partition("", "cont-focusa", "item-1"),
            "blocked:canonical_workpoint_scope_required"
        );
        assert_eq!(
            writer_claim_key_for_partition("/home/wirebot/focusa", "", "item-1"),
            "blocked:canonical_workpoint_scope_required"
        );
        assert_eq!(
            writer_claim_key_for_partition("/home/wirebot/focusa", "cont-focusa", ""),
            "blocked:active_work_item_required"
        );
    }

    fn sample_ledger_entry(
        proposal_id: &str,
        promotion_status: &str,
        trace_id: &str,
    ) -> focusa_core::types::SecondaryLoopLedgerEntry {
        focusa_core::types::SecondaryLoopLedgerEntry {
            proposal_id: proposal_id.to_string(),
            source_function: "Action::ObserveContinuousTurnOutcome".to_string(),
            actor_instance_id: None,
            role_profile_id: "daemon.work_loop.secondary_cognition".to_string(),
            current_ask_id: Some("implement spec78".to_string()),
            query_scope_id: Some("mission_carryover".to_string()),
            input_window_ref: Some("pi-turn-7001".to_string()),
            evidence_refs: vec![format!("trace://{}", trace_id)],
            proposed_delta: "secondary loop delta".to_string(),
            verification_status: if promotion_status == "promoted" {
                "verified".to_string()
            } else {
                "unverified".to_string()
            },
            promotion_status: promotion_status.to_string(),
            confidence: 0.8,
            impact_metrics: json!({
                "loop_quality": if promotion_status == "promoted" { "useful" } else { "low_quality" },
                "latency_ms_since_turn_request": 12,
            }),
            failure_class: if promotion_status == "promoted" {
                None
            } else {
                Some("verification".to_string())
            },
            description: "continuous outcome quality artifact".to_string(),
            trace_id: trace_id.to_string(),
            correlation_id: Some("task-run-1".to_string()),
            created_at: Utc::now(),
        }
    }

    fn sample_current_task(work_item_id: &str) -> SpecLinkedTaskPacket {
        SpecLinkedTaskPacket {
            work_item_id: work_item_id.to_string(),
            title: "doc78 bounded secondary cognition".to_string(),
            task_class: TaskClass::Code,
            linked_spec_refs: vec![
                "docs/78-bounded-secondary-cognition-and-persistent-autonomy.md#15.2".to_string(),
            ],
            acceptance_criteria: vec![
                "emit replay/eval bundle dimensions".to_string(),
                "persist proposal advancement ledger".to_string(),
            ],
            required_verification_tier: Some("code-task-verification".to_string()),
            allowed_scope: vec!["mission_carryover".to_string()],
            dependencies: vec![],
            tranche_id: None,
            blocker_class: None,
            checkpoint_summary: None,
        }
    }

    fn sample_workstream_key(continuity_id: &str) -> WorkstreamKey {
        let project = ScopeRef::project(
            "project:focusa",
            "/home/wirebot/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        WorkstreamKey::new(project, continuity_id).unwrap()
    }

    #[test]
    fn workpoint_summary_surfaces_scoped_packet_for_status() {
        let mut state = focusa_core::types::FocusaState::default();
        let workpoint_id = Uuid::now_v7();
        state.workpoint.active_workpoint_id = Some(Uuid::now_v7());
        state
            .workpoint
            .records
            .push(focusa_core::types::WorkpointRecord {
                workpoint_id,
                work_item_id: Some("focusa-a2w2.3".to_string()),
                session_id: Some("pi-session".to_string()),
                status: focusa_core::types::WorkpointStatus::Active,
                checkpoint_reason: focusa_core::types::WorkpointCheckpointReason::BeforeCompact,
                confidence: focusa_core::types::WorkpointConfidence::Verified,
                canonical: true,
                project_root: Some("/home/wirebot/focusa".to_string()),
                continuity_id: Some("cont-focusa".to_string()),
                mission: Some("Preserve continuation across compaction".to_string()),
                next_slice: Some("Project active workpoint into status payload".to_string()),
                action_intent: Some(focusa_core::types::WorkpointActionIntentRecord {
                    action_type: "resume_workpoint".to_string(),
                    target_ref: Some("focusa-a2w2.3".to_string()),
                    verification_hooks: vec!["status includes active_workpoint".to_string()],
                    status: Some("ready".to_string()),
                }),
                ..focusa_core::types::WorkpointRecord::default()
            });

        let key = sample_workstream_key("cont-focusa");
        let summary = scoped_workpoint_summary_for_status(&state, &key);
        assert_eq!(
            summary
                .get("active_workpoint_id")
                .and_then(Value::as_str)
                .map(str::to_string),
            Some(workpoint_id.to_string())
        );
        let active = summary.get("active").unwrap();
        assert_eq!(active.get("canonical").and_then(Value::as_bool), Some(true));
        assert_eq!(
            active.get("next_slice").and_then(Value::as_str),
            Some("Project active workpoint into status payload")
        );
    }

    #[test]
    fn resume_payload_includes_active_workpoint_summary() {
        let mut state = focusa_core::types::FocusaState::default();
        let workpoint_id = Uuid::now_v7();
        state.workpoint.active_workpoint_id = Some(Uuid::now_v7());
        state
            .workpoint
            .records
            .push(focusa_core::types::WorkpointRecord {
                workpoint_id,
                work_item_id: Some("focusa-a2w2.3".to_string()),
                status: focusa_core::types::WorkpointStatus::Active,
                canonical: true,
                project_root: Some("/home/wirebot/focusa".to_string()),
                continuity_id: Some("cont-focusa".to_string()),
                next_slice: Some("Resume from typed packet".to_string()),
                ..focusa_core::types::WorkpointRecord::default()
            });
        let key = sample_workstream_key("cont-focusa");
        let payload = resume_payload_for_status(&state, &state.work_loop, &key);
        assert_eq!(
            payload
                .pointer("/active_workpoint/active/next_slice")
                .and_then(Value::as_str),
            Some("Resume from typed packet")
        );
    }

    fn sample_secondary_quality_trace(
        continuation_decision: &str,
        loop_quality: &str,
        subject_hijack_occurred: bool,
    ) -> Value {
        json!({
            "event_type": "verification_result",
            "payload": {
                "verification_kind": "secondary_loop_quality",
                "loop_quality": loop_quality,
                "continuation_decision": continuation_decision,
                "subject_hijack_occurred": subject_hijack_occurred,
            }
        })
    }

    #[test]
    fn secondary_loop_quality_metrics_include_rate_surfaces() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.turn_count = 8;
        state.telemetry.verification_result_events = 4;
        state.telemetry.decision_consult_events = 2;
        state.telemetry.scope_contamination_events = 1;
        state.telemetry.subject_hijack_prevented_events = 3;
        state.telemetry.subject_hijack_occurred_events = 1;
        state.telemetry.secondary_loop_useful_events = 3;
        state.telemetry.secondary_loop_low_quality_events = 1;
        state.telemetry.secondary_loop_archived_events = 5;

        let metrics = secondary_loop_quality_metrics_for_status(&state, &state.work_loop);

        assert_eq!(
            metrics.get("decision_consult_rate").and_then(Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            metrics
                .get("scope_contamination_rate")
                .and_then(Value::as_f64),
            Some(0.25)
        );
        assert_eq!(
            metrics
                .get("verification_coverage_rate")
                .and_then(Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            metrics.get("subject_hijack_rate").and_then(Value::as_f64),
            Some(0.25)
        );
        assert_eq!(
            metrics
                .get("subject_hijack_occurred_events")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            metrics.get("archived_events").and_then(Value::as_u64),
            Some(5)
        );
    }

    #[test]
    fn secondary_loop_quality_metrics_handle_zero_denominators() {
        let state = focusa_core::types::FocusaState::default();
        let metrics = secondary_loop_quality_metrics_for_status(&state, &state.work_loop);

        assert_eq!(
            metrics.get("decision_consult_rate").and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics
                .get("scope_contamination_rate")
                .and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics
                .get("verification_coverage_rate")
                .and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics.get("subject_hijack_rate").and_then(Value::as_f64),
            None
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_surfaces_doc78_audit_dimensions() {
        let mut state = focusa_core::types::FocusaState::default();
        let scenario_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(scenario_id);
        state.work_loop.last_completed_task_id = Some("focusa-o8vn".to_string());

        state.telemetry.total_prompt_tokens = 1200;
        state.telemetry.total_completion_tokens = 420;
        state.telemetry.verification_result_events = 2;
        state.telemetry.secondary_loop_useful_events = 1;
        state.telemetry.secondary_loop_low_quality_events = 1;
        state.telemetry.secondary_loop_archived_events = 3;
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "promoted", "trace-1"),
            sample_ledger_entry("proposal-2", "rejected", "trace-2"),
        ];

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        assert_eq!(
            bundle.get("task_id").and_then(Value::as_str),
            Some("focusa-o8vn")
        );
        let scenario_id_str = scenario_id.to_string();
        assert_eq!(
            bundle.get("scenario_id").and_then(Value::as_str),
            Some(scenario_id_str.as_str())
        );
        assert_eq!(
            bundle
                .get("secondary_loop_kind_invoked")
                .and_then(Value::as_str),
            Some("continuous_turn_outcome_quality")
        );

        let trace_handles = bundle
            .get("trace_handles")
            .and_then(Value::as_array)
            .expect("trace handles");
        assert_eq!(trace_handles.len(), 2);
        assert!(
            trace_handles
                .iter()
                .any(|value| value.as_str() == Some("trace://trace-1"))
        );
        assert!(
            trace_handles
                .iter()
                .any(|value| value.as_str() == Some("trace://trace-2"))
        );

        let promoted = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("promoted"))
            .and_then(Value::as_u64);
        let rejected = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("rejected"))
            .and_then(Value::as_u64);
        let archived = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("archived"))
            .and_then(Value::as_u64);
        assert_eq!(promoted, Some(1));
        assert_eq!(rejected, Some(1));
        assert_eq!(archived, Some(3));

        assert_eq!(
            bundle
                .get("latency_token_cost_impact")
                .and_then(|v| v.get("total_prompt_tokens"))
                .and_then(Value::as_u64),
            Some(1200)
        );
        assert_eq!(
            bundle
                .get("latency_token_cost_impact")
                .and_then(|v| v.get("total_completion_tokens"))
                .and_then(Value::as_u64),
            Some(420)
        );

        let ledger_refs = bundle
            .get("ledger_refs")
            .and_then(Value::as_array)
            .expect("ledger refs");
        assert_eq!(ledger_refs.len(), 2);
        assert!(
            ledger_refs
                .iter()
                .any(|value| value.as_str() == Some("proposal-1"))
        );
        assert!(
            ledger_refs
                .iter()
                .any(|value| value.as_str() == Some("proposal-2"))
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_tracks_extended_outcome_classes() {
        let mut state = focusa_core::types::FocusaState::default();
        state.telemetry.secondary_loop_archived_events = 2;
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "deferred_for_review", "trace-1"),
            sample_ledger_entry("proposal-2", "archived_failed_attempt", "trace-2"),
        ];

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        let outcome = bundle
            .get("promotion_rejection_archival_result")
            .expect("outcome summary");
        assert_eq!(
            outcome.get("deferred_for_review").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            outcome
                .get("archived_failed_attempt")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(outcome.get("archived").and_then(Value::as_u64), Some(3));
    }

    #[test]
    fn secondary_loop_acceptance_hooks_surface_controlled_run_proofs() {
        let mut state = focusa_core::types::FocusaState::default();
        state.telemetry.subject_hijack_occurred_events = 1;
        state.telemetry.secondary_loop_archived_events = 1;
        state.telemetry.trace_events = vec![
            sample_secondary_quality_trace("continue", "useful", false),
            sample_secondary_quality_trace("continue", "useful", false),
            sample_secondary_quality_trace("suppress", "low_quality", true),
        ];
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "promoted", "trace-1"),
            sample_ledger_entry("proposal-2", "deferred_for_review", "trace-2"),
            sample_ledger_entry("proposal-3", "archived_failed_attempt", "trace-3"),
        ];

        let hooks = secondary_loop_acceptance_hooks_for_status(&state);
        let evidence_counts = hooks.get("evidence_counts").expect("evidence counts");

        assert_eq!(
            hooks
                .get("bounded_improvement_over_no_secondary_baseline")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("irrelevant_secondary_suggestion_suppressed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("verification_rejection_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("decay_or_archival_observed")
                .and_then(Value::as_bool),
            Some(true)
        );

        assert_eq!(
            evidence_counts
                .get("quality_trace_events")
                .and_then(Value::as_u64),
            Some(3)
        );
        assert_eq!(
            evidence_counts
                .get("suppressed_irrelevant_suggestions")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence_counts
                .get("rejected_or_deferred_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence_counts
                .get("archived_outcomes")
                .and_then(Value::as_u64),
            Some(2)
        );
        assert_eq!(
            evidence_counts
                .get("comparative_improvement_pairs")
                .and_then(Value::as_u64),
            Some(1)
        );
    }

    #[test]
    fn secondary_loop_acceptance_hooks_default_to_false_without_evidence() {
        let state = focusa_core::types::FocusaState::default();
        let hooks = secondary_loop_acceptance_hooks_for_status(&state);

        assert_eq!(
            hooks
                .get("bounded_improvement_over_no_secondary_baseline")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("irrelevant_secondary_suggestion_suppressed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("verification_rejection_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("decay_or_archival_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("evidence_counts")
                .and_then(|value| value.get("comparative_improvement_pairs"))
                .and_then(Value::as_u64),
            Some(0)
        );
    }

    #[test]
    fn secondary_loop_closure_replay_evidence_surfaces_current_task_pair() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));
        state.work_loop.last_completed_task_id = Some("focusa-prev".to_string());

        let summary = focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        };

        let evidence =
            secondary_loop_closure_replay_evidence_for_status(&state.work_loop, &summary);

        let task_run_id_str = task_run_id.to_string();
        assert_eq!(
            evidence.get("current_task_pair_id").and_then(Value::as_str),
            Some(task_run_id_str.as_str())
        );
        assert_eq!(
            evidence
                .get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            evidence
                .get("current_task_pair_promoted_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence
                .get("current_task_pair_non_promoted_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence
                .get("correlation_candidates")
                .and_then(Value::as_array)
                .map(|entries| entries.len()),
            Some(3)
        );
    }

    #[test]
    fn secondary_loop_closure_replay_evidence_defaults_fail_closed_without_match() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));
        state.work_loop.last_completed_task_id = Some("focusa-prev".to_string());

        let summary = focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 9,
            secondary_loop_outcome_events: 1,
            promoted_outcomes: 1,
            rejected_outcomes: 0,
            deferred_for_review_outcomes: 0,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 0,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: "unrelated".to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 0,
                comparative_improvement_observed: false,
            }],
        };

        let evidence =
            secondary_loop_closure_replay_evidence_for_status(&state.work_loop, &summary);

        assert_eq!(
            evidence
                .get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(evidence.get("current_task_pair_id"), Some(&Value::Null));
        assert_eq!(
            evidence
                .get("correlation_candidates")
                .and_then(Value::as_array)
                .map(|entries| entries.len()),
            Some(2)
        );
    }

    #[test]
    fn secondary_loop_replay_consumer_payload_surfaces_ok_state() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary = Ok(focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        });

        let payload =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);

        let task_run_id_str = task_run_id.to_string();
        assert_eq!(payload.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("ok")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("summary"))
                .and_then(|value| value.get("comparative_improvement_pairs"))
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("ok")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence"))
                .and_then(|value| value.get("current_task_pair_observed"))
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence"))
                .and_then(|value| value.get("current_task_pair_id"))
                .and_then(Value::as_str),
            Some(task_run_id_str.as_str())
        );
    }

    #[test]
    fn secondary_loop_replay_consumer_payload_surfaces_error_state_fail_closed() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let payload =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);

        assert_eq!(payload.get("status").and_then(Value::as_str), Some("error"));
        assert_eq!(
            payload.get("error").and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence")),
            None
        );
    }

    #[test]
    fn secondary_loop_continuity_gate_surfaces_open_state_when_replay_ok() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary = Ok(focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        });

        let replay_consumer =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);
        let gate = secondary_loop_continuity_gate_for_status(&replay_summary, &replay_consumer);

        assert_eq!(gate.get("state").and_then(Value::as_str), Some("open"));
        assert_eq!(
            gate.get("fail_closed").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            gate.get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn secondary_loop_continuity_gate_surfaces_fail_closed_when_replay_error() {
        let state = focusa_core::types::FocusaState::default();
        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let replay_consumer =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);
        let gate = secondary_loop_continuity_gate_for_status(&replay_summary, &replay_consumer);

        assert_eq!(
            gate.get("state").and_then(Value::as_str),
            Some("fail-closed")
        );
        assert_eq!(gate.get("fail_closed").and_then(Value::as_bool), Some(true));
        assert_eq!(
            gate.get("reason").and_then(Value::as_str),
            Some("replay_consumer_error")
        );
    }

    #[test]
    fn secondary_loop_closure_bundle_surfaces_replay_gate_contract() {
        let state = focusa_core::types::FocusaState::default();
        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let bundle =
            secondary_loop_closure_bundle_for_status(&state, &state.work_loop, &replay_summary);

        assert_eq!(bundle.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(bundle.get("doc").and_then(Value::as_str), Some("78"));
        assert_eq!(
            bundle
                .get("secondary_loop_replay_consumer")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            bundle
                .get("secondary_loop_continuity_gate")
                .and_then(|value| value.get("state"))
                .and_then(Value::as_str),
            Some("fail-closed")
        );
        assert_eq!(
            bundle
                .get("evidence_contract")
                .and_then(|value| value.get("replay_consumer_route"))
                .and_then(Value::as_str),
            Some("/v1/work-loop/replay/closure-evidence")
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_prefers_current_task_when_bound() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.last_completed_task_id = Some("focusa-old".to_string());
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        assert_eq!(
            bundle.get("task_id").and_then(Value::as_str),
            Some("focusa-live")
        );
    }

    #[test]
    fn pi_rpc_execution_invocation_is_persisted_resumable_and_governed() {
        let request = PiDriverStartRequest {
            cwd: Some("/tmp/project".to_string()),
            models: Some("anthropic/claude".to_string()),
            resume_session: Some("session-123".to_string()),
            session_dir: Some("/tmp/pi-sessions".to_string()),
            session_name: Some("governed-workpoint".to_string()),
            workpoint_id: Some("workpoint-123".to_string()),
            idempotency_key: "execution-123".to_string(),
        };
        let mut command = Command::new("pi");
        configure_pi_rpc_invocation(&mut command, &request);
        let args: Vec<_> = command
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        assert_eq!(args[0..2], ["--mode", "rpc"]);
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--session", "session-123"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--session-dir", "/tmp/pi-sessions"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--name", "governed-workpoint"])
        );
        assert!(!args.iter().any(|arg| arg == "--no-session"));
    }

    #[test]
    fn prose_without_typed_receipt_never_claims_completion() {
        assert!(parse_work_loop_outcome_receipt("implemented and all tests pass").is_none());
    }

    #[test]
    fn typed_completion_receipt_carries_stable_evidence() {
        let output = r#"work done
FOCUSA_WORK_LOOP_OUTCOME {"schema":"focusa.work_loop_outcome.v1","work_item_id":"focusa-1","status":"completed","summary":"verified","spec_conformant":true,"evidence_citations":[{"kind":"test","ref":"tests/work_loop.rs","required":true}]}"#;
        let receipt = parse_work_loop_outcome_receipt(output).unwrap();
        assert_eq!(receipt.work_item_id, "focusa-1");
        assert_eq!(receipt.status, WorkLoopOutcomeStatus::Completed);
        assert!(receipt.spec_conformant);
        assert_eq!(receipt.evidence_citations.len(), 1);
        assert_eq!(receipt.evidence_citations[0].ref_, "tests/work_loop.rs");
    }

    #[test]
    fn mismatched_or_unknown_receipt_schema_is_rejected() {
        let output = r#"FOCUSA_WORK_LOOP_OUTCOME {"schema":"focusa.work_loop_outcome.v2","work_item_id":"focusa-1","status":"completed","spec_conformant":true,"evidence_citations":[]}"#;
        assert!(parse_work_loop_outcome_receipt(output).is_none());
    }
}
