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
    Action, CandidateId, CompletionReason, InstanceKind, MemorySource, Signal, SignalKind,
    SignalOrigin,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
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
struct StartSessionPayload {
    #[serde(default)]
    adapter_id: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
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

fn default_disconnect_reason() -> String {
    "command_submit".to_string()
}

fn default_compact_tier() -> String {
    "auto".to_string()
}

fn decode<T: for<'de> Deserialize<'de>>(
    payload: Value,
    command: &str,
) -> Result<T, (StatusCode, Json<Value>)> {
    serde_json::from_value(payload).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Invalid payload for {}: {}", command, e),
            })),
        )
    })
}

fn map_command_to_action(
    command: &str,
    payload: Value,
) -> Result<Action, (StatusCode, Json<Value>)> {
    match command {
        "focus.push_frame" => {
            let p: PushFramePayload = decode(payload, command)?;
            Ok(Action::PushFrame {
                title: p.title,
                goal: p.goal,
                beads_issue_id: p.beads_issue_id,
                constraints: p.constraints,
                tags: p.tags,
            })
        }
        "focus.pop_frame" => {
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
        "session.start" => {
            let p: StartSessionPayload = decode(payload, command)?;
            Ok(Action::StartSession {
                adapter_id: p.adapter_id,
                workspace_id: p.workspace_id,
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
        "ascc.checkpoint" => {
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
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Unknown or disallowed command: {}", command)})),
        )),
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

    match state.command_tx.send(action).await {
        Ok(_) => {
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
        Err(e) => {
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
            store.insert(command_id, record.clone());

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "command dispatch failed",
                    "details": record.error,
                })),
            ))
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
    let record = store.get(&command_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "command_id not found"})),
        )
    })?;

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
    let record = store.get(&command_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "command_id not found"})),
        )
    })?;

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
