//! Sync transfer endpoint — import ownership transfer events that mutate state.
//!
//! POST /v1/sync/transfer
//!
//! Unlike /v1/sync/receive (observations only), this endpoint handles events
//! that MUST mutate canonical state:
//! - ThreadOwnershipTransferred: Updates thread ownership on receiving peer
//!
//! Policy #2 exception: Ownership transfers are explicit actions, not remote
//! changes being observed. They must propagate to all peers to maintain
//! consistent thread ownership.

use crate::server::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use focusa_core::{
    temporal_clock::TemporalActionEnvelope,
    types::{Action, EventLogEntry, FocusaEvent, SignalOrigin},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

type SyncTransferResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn transfer_failure(
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
            "next_tools": ["focusa_tool_doctor", "focusa_project_identity"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": failure_class != "validation_rejected", "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry" }, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_project_identity"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn transfer_persistence_failed(
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    transfer_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("sync transfer persistence failed: {error}"),
        "persistence_unavailable",
        "Sync transfer could not check or persist ownership-transfer state.",
        "Check SQLite/daemon health and retry after local persistence recovers.",
        "Likely database lock, wrong project database, or daemon shutdown during sync transfer.",
    )
}

fn transfer_timestamp_rejected(value: &str) -> (StatusCode, Json<serde_json::Value>) {
    transfer_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid transfer event timestamp: {value}"),
        "validation_rejected",
        "Sync transfer event timestamp must be RFC3339.",
        "Correct the transfer event timestamp before retrying unchanged.",
        "Likely incompatible peer version or malformed ownership-transfer payload.",
    )
}

#[derive(Deserialize)]
pub struct TransferBody {
    /// Events that must mutate state (not observations)
    events: Vec<TransferEvent>,
    /// Peer that sent these events
    peer_id: String,
}

#[derive(Deserialize)]
struct TransferEvent {
    event_id: String,
    timestamp: String,
    machine_id: String,
    #[serde(default)]
    temporal: Option<TemporalActionEnvelope>,
    /// The event payload (FocusaEvent as JSON)
    event: serde_json::Value,
}

pub async fn transfer_impl(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TransferBody>,
) -> SyncTransferResult {
    let mut applied = 0;
    let mut rejected = 0;
    let start = std::time::Instant::now();

    // Track the last successfully applied event for cursor update.
    // Only advance cursor to events we've actually persisted.
    let mut last_applied_id: Option<String> = None;
    let mut last_applied_ts: Option<String> = None;

    for remote in &body.events {
        // Parse event_id for idempotency check
        let event_id = match remote.event_id.parse::<Uuid>() {
            Ok(id) => id,
            Err(_) => {
                rejected += 1;
                continue;
            }
        };

        // Check if already exists (idempotency)
        let exists = state
            .persistence
            .event_exists(&event_id.to_string())
            .map_err(transfer_persistence_failed)?;

        if exists {
            rejected += 1;
            continue;
        }

        // Parse the event - only accept ownership transfer events
        let event: FocusaEvent = match serde_json::from_value(remote.event.clone()) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(event_id = %event_id, error = %e, "Failed to parse transfer event");
                rejected += 1;
                continue;
            }
        };

        // Extract thread_id and validate to_machine_id from event
        let (thread_id, from_machine_id, to_machine_id) = match &event {
            FocusaEvent::ThreadOwnershipTransferred {
                thread_id,
                from_machine_id,
                to_machine_id,
                ..
            } => {
                if to_machine_id.is_empty() {
                    tracing::warn!(event_id = %event_id, "Rejected transfer with empty to_machine_id");
                    rejected += 1;
                    continue;
                }
                (*thread_id, from_machine_id.clone(), to_machine_id.clone())
            }
            _ => {
                tracing::warn!(event_id = %event_id, "Rejected non-ownership event via /v1/sync/transfer");
                rejected += 1;
                continue;
            }
        };

        // Create entry as NON-observation (must mutate state)
        let entry = EventLogEntry {
            id: event_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&remote.timestamp)
                .map_err(|_| transfer_timestamp_rejected(&remote.timestamp))?
                .with_timezone(&chrono::Utc),
            temporal: remote.temporal.clone().unwrap_or_else(|| {
                TemporalActionEnvelope::unavailable(
                    "legacy_remote_transfer_missing_temporal_action_envelope",
                )
            }),
            event,
            correlation_id: Some(format!("sync:transfer:from:{}", body.peer_id)),
            origin: SignalOrigin::Sync,
            // Use from_machine_id for the log (who's transferring ownership)
            machine_id: from_machine_id
                .as_ref()
                .cloned()
                .or(Some(remote.machine_id.clone())),
            instance_id: None,
            session_id: None,
            project_root: None,
            continuity_id: None,
            thread_id: Some(thread_id),
            is_observation: false, // CRITICAL: Must mutate state!
        };

        if let Err(e) = state.append_events_checkpoint(vec![entry.clone()]).await {
            tracing::warn!("Failed to persist transfer event: {}", e);
            rejected += 1;
            continue;
        }

        if let Err(e) = state
            .command_tx
            .send(Action::EmitEvent {
                event: entry.event.clone(),
            })
            .await
        {
            tracing::warn!(event_id = %event_id, error = %e, "Failed to dispatch transfer event");
            rejected += 1;
            continue;
        }

        let mut visible = false;
        for _ in 0..80 {
            {
                let focusa_state = state.focusa.read().await;
                if let Some(thread) = focusa_state.threads.iter().find(|t| t.id == thread_id)
                    && thread.owner_machine_id.as_deref() == Some(to_machine_id.as_str())
                {
                    visible = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        if !visible {
            tracing::warn!(event_id = %event_id, "Transfer event dispatched but not yet visible");
        }

        last_applied_id = Some(remote.event_id.clone());
        last_applied_ts = Some(remote.timestamp.clone());
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(json);
        }
        applied += 1;
    }

    // Update peer cursor to the last successfully applied event.
    // This ensures we don't skip events if some fail to import.
    if let (Some(last_id), Some(last_ts)) = (last_applied_id, last_applied_ts) {
        let _ = state
            .persistence
            .set_cursor(&body.peer_id, Some(&last_id), Some(&last_ts));
    }

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(
        peer_id = %body.peer_id,
        applied,
        rejected,
        elapsed_ms,
        "Sync transfer completed"
    );

    Ok(Json(json!({
        "applied": applied,
        "rejected": rejected,
        "peer_id": body.peer_id,
        "elapsed_ms": elapsed_ms,
    })))
}
