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
use focusa_core::types::{Action, EventLogEntry, FocusaEvent, SignalOrigin};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

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
    /// The event payload (FocusaEvent as JSON)
    event: serde_json::Value,
}

pub async fn transfer_impl(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TransferBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .with_timezone(&chrono::Utc),
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
            thread_id: Some(thread_id),
            is_observation: false, // CRITICAL: Must mutate state!
        };

        if let Err(e) = state.persistence.append_event(&entry) {
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
