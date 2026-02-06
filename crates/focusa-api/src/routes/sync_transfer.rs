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
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use focusa_core::reducer;
use focusa_core::types::{EventLogEntry, FocusaEvent, SignalOrigin};
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
    /// The actual event data (flattened)
    #[serde(flatten)]
    event: serde_json::Value,
}

pub async fn transfer_impl(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TransferBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut applied = 0;
    let mut rejected = 0;
    let start = std::time::Instant::now();

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

        // Extract thread_id from event for tracking
        let thread_id = match &event {
            FocusaEvent::ThreadOwnershipTransferred { thread_id, .. } => Some(*thread_id),
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
            machine_id: Some(remote.machine_id.clone()),
            instance_id: None,
            session_id: None,
            thread_id,
            is_observation: false, // CRITICAL: Must mutate state!
        };

        // Run through reducer to apply ownership change
        let focusa_state = state.focusa.read().await;
        match reducer::reduce_with_meta(
            focusa_state.clone(),
            entry.event.clone(),
            Some(&remote.machine_id),
            thread_id,
            false, // Not an observation
        ) {
            Ok(result) => {
                // Update state with ownership change
                drop(focusa_state);
                {
                    let mut focusa_state = state.focusa.write().await;
                    *focusa_state = result.new_state;
                }

                // Persist the event
                if let Err(e) = state.persistence.append_event(&entry) {
                    tracing::warn!("Failed to persist transfer event: {}", e);
                    rejected += 1;
                    continue;
                }

                // Save state snapshot
                let current_state = state.focusa.read().await;
                if let Err(e) = state.persistence.save_state(&current_state) {
                    tracing::warn!("Failed to save state after transfer: {}", e);
                }
                drop(current_state);

                // Broadcast to SSE subscribers
                if let Ok(json) = serde_json::to_string(&entry) {
                    let _ = state.events_tx.send(json);
                }

                applied += 1;
            }
            Err(e) => {
                tracing::warn!(event_id = %event_id, error = %e, "Transfer event rejected by reducer");
                rejected += 1;
                continue;
            }
        }
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
