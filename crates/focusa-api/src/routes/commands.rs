//! Command write-model routes (docs/23 §4.2).
//!
//! POST /v1/commands/submit
//! GET  /v1/commands/status/{command_id}
//! GET  /v1/commands/log/{command_id}

use crate::routes::permissions::{forbid, permission_context};
use crate::server::{AppState, CommandExecutionStatus, CommandLogEntry, CommandRecord};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::get, routing::post};
use chrono::Utc;
use focusa_core::types::{
    Action, CacheBustCategory, CandidateId, CompletionReason, FocusStackState, FrameStatus,
    HandleKind, InstanceKind, MemorySource, SessionState, SessionStatus, Signal, SignalKind,
    SignalOrigin,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct SubmitCommandRequest {
    command: String,
    #[serde(default, alias = "args")]
    payload: Value,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PushFramePayload {
    title: String,
    goal: String,
    beads_issue_id: String,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    #[serde(default)]
    constraints: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PopFramePayload {
    completion_reason: CompletionReason,
}

#[derive(Debug, Deserialize)]
struct SetActivePayload {
    frame_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct IngestSignalPayload {
    kind: SignalKind,
    summary: String,
    #[serde(default)]
    origin: Option<SignalOrigin>,
    #[serde(default)]
    frame_context: Option<Uuid>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CandidatePayload {
    candidate_id: CandidateId,
}

#[derive(Debug, Deserialize)]
struct SurfacePayload {
    candidate_id: CandidateId,
    #[serde(default = "default_boost")]
    boost: f32,
}

fn default_boost() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
struct SuppressPayload {
    candidate_id: CandidateId,
    #[serde(default = "default_scope", alias = "duration")]
    scope: String,
}

fn default_scope() -> String {
    "session".to_string()
}

#[derive(Debug, Deserialize)]
struct UpsertSemanticPayload {
    key: String,
    value: String,
    #[serde(default)]
    source: Option<MemorySource>,
}

#[derive(Debug, Deserialize)]
struct ReinforcePayload {
    rule_id: String,
}

#[derive(Debug, Deserialize)]
struct CacheBustPayload {
    category: CacheBustCategory,
}

#[derive(Debug, Deserialize)]
struct StartSessionPayload {
    #[serde(default)]
    adapter_id: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    #[serde(default)]
    instance_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CloseSessionPayload {
    #[serde(default = "default_close_reason")]
    reason: String,
    #[serde(default)]
    instance_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CheckpointPayload {
    #[serde(default)]
    frame_id: Option<Uuid>,
    #[serde(default = "default_close_reason")]
    reason: String,
}

#[derive(Debug, Deserialize)]
struct CompactPayload {
    #[serde(default)]
    force: bool,
    #[serde(default = "default_compact_tier")]
    tier: String,
    #[serde(default)]
    turn_count: Option<u64>,
    #[serde(default)]
    surface: Option<String>,
}

fn default_close_reason() -> String {
    "command_submit".to_string()
}

#[derive(Debug, Deserialize)]
struct ConnectInstancePayload {
    kind: InstanceKind,
}

#[derive(Debug, Deserialize)]
struct DisconnectInstancePayload {
    instance_id: Uuid,
    #[serde(default = "default_disconnect_reason")]
    reason: String,
}

#[derive(Debug, Deserialize)]
struct VisualEvidencePayload {
    run_id: String,
    phase: String,
    evidence_kind: String,
    label: String,
    kind: HandleKind,
    /// Base64-encoded content.
    content_b64: Option<String>,
    /// Plain text content (alternative to base64).
    #[serde(default)]
    content: Option<String>,
}

impl VisualEvidencePayload {
    fn resolve_content(&self) -> Result<Vec<u8>, Value> {
        if let Some(ref b64) = self.content_b64 {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|_| json!({"status":"rejected","reason":"invalid_content_b64"}))
        } else if let Some(ref txt) = self.content {
            Ok(txt.as_bytes().to_vec())
        } else {
            Err(json!({"status":"rejected","reason":"missing_content"}))
        }
    }

    fn to_artifact_label(&self) -> String {
        format!(
            "visual:{}:{}:{}:{}",
            self.run_id, self.phase, self.evidence_kind, self.label
        )
    }
}

fn default_disconnect_reason() -> String {
    "command_submit".to_string()
}

fn default_compact_tier() -> String {
    "auto".to_string()
}

fn command_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "failure_class": failure_class,
                    "canonical": false,
                    "degraded": true,
                    "summary": why,
                    "retry": {"safe": true, "posture": "safe_retry", "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn command_payload_rejected(
    command: &str,
    err: impl std::fmt::Display,
) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid payload for {command}: {err}"),
        "validation_rejected",
        format!("command {command} payload did not match the command schema: {err}"),
        "Inspect the command schema, then resend /v1/commands/submit with a valid payload or args object.",
        "Likely malformed JSON, wrong command alias payload, missing field, or stale extension command contract.",
        vec![
            "focusa_tool_doctor",
            "focusa_project_identity",
            "focusa_workpoint_resume",
        ],
    )
}

fn command_unknown(command: &str) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::BAD_REQUEST,
        format!("unknown or disallowed command: {command}"),
        "validation_rejected",
        format!("command {command} is not registered on the /v1/commands/submit allowlist"),
        "Use a documented command alias or inspect route/tool contracts before retrying.",
        "Likely stale extension, wrong command name, or unsupported write-model mutation.",
        vec!["focusa_tool_doctor", "focusa_trajectory_view"],
    )
}

fn command_visual_payload_rejected(details: Value) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::BAD_REQUEST,
        "invalid visual evidence payload",
        "validation_rejected",
        format!("visual evidence payload content rejected: {details}"),
        "Provide content_b64 or content with valid encoding, then retry the visual evidence command.",
        "Likely missing visual content or invalid base64 content_b64 field.",
        vec!["focusa_tool_doctor", "focusa_active_object_resolve"],
    )
}

fn command_action_rejected(details: Value) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::BAD_REQUEST,
        "command action validation rejected",
        "validation_rejected",
        format!("command cannot be applied in current Focusa state: {details}"),
        "Inspect current session/focus stack state, then retry only after prerequisites are true.",
        "Likely no active session, inactive frame, invalid frame transition, or out-of-order command sequence.",
        vec![
            "focusa_tool_doctor",
            "focusa_project_identity",
            "focusa_workpoint_resume",
        ],
    )
}

fn command_dispatch_failed(command_id: &str, details: Option<String>) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "command dispatch unavailable",
        "daemon_unavailable",
        format!("command {command_id} could not be dispatched because the daemon action channel is unavailable"),
        "Check daemon health and the command record before retrying; use the idempotency key/command_id to avoid duplicate side effects.",
        "Likely daemon command channel closed, runtime shutdown, or dispatch owner unavailable before the command was enqueued.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status", "focusa_resource_mode", "focusa_workpoint_resume"],
    ).tap_details(details)
}

fn command_dispatch_timeout(command_id: &str) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::ACCEPTED,
        "command dispatch pending",
        "resource_exhausted",
        format!("command {command_id} could not be dispatched within bounded command-channel wait"),
        "Check command status/log after backlog drains; retry only if the command record remains pending/failed.",
        "Likely daemon command channel saturated or reducer backlog under resource pressure.",
        vec!["focusa_tool_doctor", "focusa_resource_mode", "focusa_work_loop_status"],
    )
}

trait CommandFailureDetailsExt {
    fn tap_details(self, details: Option<String>) -> Self;
}

impl CommandFailureDetailsExt for (StatusCode, Json<Value>) {
    fn tap_details(mut self, details: Option<String>) -> Self {
        if let Some(details) = details {
            if let Some(obj) = self.1.0.as_object_mut() {
                obj.insert("dispatch_error".to_string(), json!(details));
            }
        }
        self
    }
}

fn command_not_found(command_id: &str) -> (StatusCode, Json<Value>) {
    command_failure(
        StatusCode::NOT_FOUND,
        "command_id not found",
        "not_found",
        format!("command_id {command_id} is not present in the in-memory command store"),
        "Verify the command_id from submit response, then inspect command status/log only for known IDs.",
        "Likely stale command_id, daemon restart cleared volatile command store, or wrong server instance.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status"],
    )
}

fn decode<T: for<'de> Deserialize<'de>>(
    payload: Value,
    command: &str,
) -> Result<T, (StatusCode, Json<Value>)> {
    serde_json::from_value(payload).map_err(|e| command_payload_rejected(command, e))
}

fn ensure_active_session(session: Option<&SessionState>) -> Result<(), Value> {
    match session {
        Some(session) if session.status == SessionStatus::Active => Ok(()),
        Some(session) => Err(json!({
            "status": "rejected",
            "reason": "session_inactive",
            "session_status": session.status,
        })),
        None => Err(json!({
            "status": "rejected",
            "reason": "no_active_session",
        })),
    }
}

fn validate_can_pop(stack: &FocusStackState) -> Result<(), Value> {
    let active_id = match stack.active_id {
        Some(id) => id,
        None => return Err(json!({"status": "no_active_frame"})),
    };

    let active = stack
        .frames
        .iter()
        .find(|f| f.id == active_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "active_frame_missing"}))?;

    let parent_id = active
        .parent_id
        .ok_or_else(|| json!({"status": "rejected", "reason": "cannot_complete_root_frame"}))?;

    let parent = stack
        .frames
        .iter()
        .find(|f| f.id == parent_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "parent_frame_missing"}))?;

    if parent.status != FrameStatus::Paused {
        return Err(json!({
            "status": "rejected",
            "reason": "parent_not_paused",
            "parent_status": parent.status,
        }));
    }

    Ok(())
}

fn validate_set_active(stack: &FocusStackState, frame_id: Uuid) -> Result<(), Value> {
    let active_id = match stack.active_id {
        Some(id) => id,
        None => return Err(json!({"status": "no_active_frame"})),
    };

    if active_id == frame_id {
        return Ok(());
    }

    if !stack.stack_path_cache.contains(&frame_id) {
        return Err(json!({
            "status": "rejected",
            "reason": "target_not_in_current_stack_path",
        }));
    }

    let target = stack
        .frames
        .iter()
        .find(|f| f.id == frame_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "frame_not_found"}))?;

    if target.status != FrameStatus::Paused {
        return Err(json!({
            "status": "rejected",
            "reason": "target_not_paused",
            "frame_status": target.status,
        }));
    }

    Ok(())
}

fn validate_action(
    action: &Action,
    session: Option<&SessionState>,
    stack: &FocusStackState,
) -> Result<(), Value> {
    match action {
        Action::PushFrame { beads_issue_id, .. } => {
            ensure_active_session(session)?;
            if beads_issue_id.trim().is_empty() {
                return Err(json!({
                    "status": "rejected",
                    "reason": "missing_beads_issue_id",
                }));
            }
            Ok(())
        }
        Action::PopFrame { .. } => {
            ensure_active_session(session)?;
            validate_can_pop(stack)
        }
        Action::SetActiveFrame { frame_id } => {
            ensure_active_session(session)?;
            validate_set_active(stack, *frame_id)
        }
        Action::StartSession { .. } => {
            if let Some(session) = session
                && session.status == SessionStatus::Active
            {
                return Err(json!({
                    "status": "rejected",
                    "reason": "session_already_active",
                    "session_id": session.session_id,
                }));
            }
            Ok(())
        }
        Action::CloseSession { .. } => ensure_active_session(session),
        _ => Ok(()),
    }
}

fn map_command_to_action(
    command: &str,
    payload: Value,
) -> Result<Action, (StatusCode, Json<Value>)> {
    match command {
        "focus.push_frame" | "visual.start_run" | "start_visual_run" => {
            let p: PushFramePayload = decode(payload, command)?;
            Ok(Action::PushFrame {
                title: p.title,
                goal: p.goal,
                beads_issue_id: p.beads_issue_id,
                project_root: p.project_root,
                continuity_id: p.continuity_id,
                constraints: p.constraints,
                tags: p.tags,
            })
        }
        "focus.pop_frame" | "visual.close_run" | "close_visual_run" => {
            let p: PopFramePayload = decode(payload, command)?;
            Ok(Action::PopFrame {
                completion_reason: p.completion_reason,
            })
        }
        "focus.set_active" => {
            let p: SetActivePayload = decode(payload, command)?;
            Ok(Action::SetActiveFrame {
                frame_id: p.frame_id,
            })
        }
        "gate.ingest_signal" => {
            let p: IngestSignalPayload = decode(payload, command)?;
            Ok(Action::IngestSignal {
                signal: Signal {
                    id: Uuid::now_v7(),
                    ts: Utc::now(),
                    origin: p.origin.unwrap_or(SignalOrigin::Cli),
                    kind: p.kind,
                    frame_context: p.frame_context,
                    summary: p.summary,
                    payload_ref: None,
                    tags: p.tags,
                },
            })
        }
        "gate.surface_candidate" => {
            let p: SurfacePayload = decode(payload, command)?;
            Ok(Action::SurfaceCandidate {
                candidate_id: p.candidate_id,
                boost: p.boost,
            })
        }
        "gate.pin" | "gate.pin_candidate" => {
            let p: CandidatePayload = decode(payload, command)?;
            Ok(Action::PinCandidate {
                candidate_id: p.candidate_id,
            })
        }
        "gate.suppress" | "gate.suppress_candidate" => {
            let p: SuppressPayload = decode(payload, command)?;
            Ok(Action::SuppressCandidate {
                candidate_id: p.candidate_id,
                scope: p.scope,
            })
        }
        "memory.semantic.upsert" => {
            let p: UpsertSemanticPayload = decode(payload, command)?;
            Ok(Action::UpsertSemantic {
                key: p.key,
                value: p.value,
                source: p.source.unwrap_or(MemorySource::User),
            })
        }
        "memory.procedural.reinforce" => {
            let p: ReinforcePayload = decode(payload, command)?;
            Ok(Action::ReinforceRule { rule_id: p.rule_id })
        }
        "memory.decay_tick" => Ok(Action::DecayTick),
        "cache.bust" => {
            let p: CacheBustPayload = decode(payload, command)?;
            Ok(Action::CacheBust {
                category: p.category,
            })
        }
        "session.start" => {
            let p: StartSessionPayload = decode(payload, command)?;
            Ok(Action::StartSession {
                adapter_id: p.adapter_id,
                workspace_id: p.workspace_id,
                project_root: p.project_root,
                continuity_id: p.continuity_id,
                instance_id: p.instance_id,
            })
        }
        "session.close" => {
            let p: CloseSessionPayload = decode(payload, command)?;
            Ok(Action::CloseSession {
                reason: p.reason,
                instance_id: p.instance_id,
            })
        }
        "ascc.checkpoint" | "visual.start_iteration" | "start_iteration" => {
            let p: CheckpointPayload = decode(payload, command)?;
            Ok(Action::CheckpointFrame {
                frame_id: p.frame_id,
                reason: p.reason,
            })
        }
        "compact" | "micro-compact" => {
            let mut p: CompactPayload = decode(payload, command)?;
            if command == "micro-compact" && p.tier == default_compact_tier() {
                p.tier = "micro".to_string();
            }
            Ok(Action::CompactContext {
                force: p.force,
                tier: p.tier,
                turn_count: p.turn_count,
                surface: p.surface,
            })
        }
        "visual.register_reference_artifacts"
        | "register_reference_artifacts"
        | "visual.create_blueprint"
        | "create_blueprint"
        | "visual.record_build_output"
        | "record_build_output"
        | "visual.record_comparison"
        | "record_comparison"
        | "visual.record_critique"
        | "record_critique"
        | "visual.synthesize_fixes"
        | "synthesize_fixes"
        | "visual.apply_fix_set"
        | "apply_fix_set" => {
            let p: VisualEvidencePayload = decode(payload, command)?;
            let content = p
                .resolve_content()
                .map_err(|e| command_visual_payload_rejected(e))?;
            Ok(Action::StoreArtifact {
                kind: p.kind,
                label: p.to_artifact_label(),
                content,
            })
        }
        "instances.connect" => {
            let p: ConnectInstancePayload = decode(payload, command)?;
            Ok(Action::InstanceConnect { kind: p.kind })
        }
        "instances.disconnect" => {
            let p: DisconnectInstancePayload = decode(payload, command)?;
            Ok(Action::InstanceDisconnect {
                instance_id: p.instance_id,
                reason: p.reason,
            })
        }
        _ => Err(command_unknown(command)),
    }
}

/// POST /v1/commands/submit
async fn submit_command(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SubmitCommandRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token_enabled =
        state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok();
    let permissions = permission_context(&headers, token_enabled);
    if !permissions.allows("commands:submit") {
        return Err(forbid("commands:submit"));
    }

    let command_id = Uuid::now_v7().to_string();
    let now = Utc::now();

    let action = map_command_to_action(&req.command, req.payload)?;

    {
        let focusa = state.focusa.read().await;
        if let Err(resp) = validate_action(&action, focusa.session.as_ref(), &focusa.focus_stack) {
            return Err(command_action_rejected(resp));
        }
    }

    let mut record = CommandRecord {
        command_id: command_id.clone(),
        command: req.command.clone(),
        status: CommandExecutionStatus::Accepted,
        submitted_at: now,
        dispatched_at: None,
        completed_at: None,
        error: None,
        logs: vec![CommandLogEntry {
            ts: now,
            level: "info".to_string(),
            message: format!(
                "Accepted command{}",
                req.reason
                    .as_deref()
                    .map(|r| format!(" (reason: {})", r))
                    .unwrap_or_default()
            ),
        }],
    };

    {
        let mut store = state.command_store.write().await;
        store.insert(command_id.clone(), record.clone());
    }

    match tokio::time::timeout(Duration::from_millis(1500), state.command_tx.send(action)).await {
        Ok(Ok(_)) => {
            let dispatched_at = Utc::now();
            record.status = CommandExecutionStatus::Dispatched;
            record.dispatched_at = Some(dispatched_at);
            record.logs.push(CommandLogEntry {
                ts: dispatched_at,
                level: "info".to_string(),
                message: "Command dispatched to daemon action channel".to_string(),
            });

            let mut store = state.command_store.write().await;
            store.insert(command_id.clone(), record.clone());

            Ok(Json(json!({
                "accepted": true,
                "command_id": command_id,
                "status": record.status,
                "submitted_at": record.submitted_at,
                "dispatched_at": record.dispatched_at,
                "idempotency_key": req.idempotency_key,
            })))
        }
        Ok(Err(e)) => {
            let failed_at = Utc::now();
            record.status = CommandExecutionStatus::Failed;
            record.completed_at = Some(failed_at);
            record.error = Some(e.to_string());
            record.logs.push(CommandLogEntry {
                ts: failed_at,
                level: "error".to_string(),
                message: format!("Command dispatch failed: {}", e),
            });

            let mut store = state.command_store.write().await;
            store.insert(command_id.clone(), record.clone());

            Err(command_dispatch_failed(&command_id, record.error.clone()))
        }
        Err(_) => {
            let pending_at = Utc::now();
            record.status = CommandExecutionStatus::Accepted;
            record.completed_at = None;
            record.error = Some("command dispatch timed out before enqueue".to_string());
            record.logs.push(CommandLogEntry {
                ts: pending_at,
                level: "warn".to_string(),
                message: "Command dispatch timed out before enqueue; command remains accepted/pending for recovery inspection".to_string(),
            });

            let mut store = state.command_store.write().await;
            store.insert(command_id.clone(), record.clone());

            Err(command_dispatch_timeout(&command_id))
        }
    }
}

/// GET /v1/commands/status/{command_id}
async fn command_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(command_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token_enabled =
        state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok();
    let permissions = permission_context(&headers, token_enabled);
    if !permissions.allows("commands:submit") {
        return Err(forbid("commands:submit"));
    }

    let store = state.command_store.read().await;
    let record = store
        .get(&command_id)
        .ok_or_else(|| command_not_found(&command_id))?;

    Ok(Json(json!({
        "command_id": record.command_id,
        "command": record.command,
        "status": record.status,
        "submitted_at": record.submitted_at,
        "dispatched_at": record.dispatched_at,
        "completed_at": record.completed_at,
        "error": record.error,
    })))
}

/// GET /v1/commands/log/{command_id}
async fn command_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(command_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token_enabled =
        state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok();
    let permissions = permission_context(&headers, token_enabled);
    if !permissions.allows("commands:submit") {
        return Err(forbid("commands:submit"));
    }

    let store = state.command_store.read().await;
    let record = store
        .get(&command_id)
        .ok_or_else(|| command_not_found(&command_id))?;

    Ok(Json(json!({
        "command_id": record.command_id,
        "command": record.command,
        "logs": record.logs,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/commands/submit", post(submit_command))
        .route("/v1/commands/status/{command_id}", get(command_status))
        .route("/v1/commands/log/{command_id}", get(command_log))
}
