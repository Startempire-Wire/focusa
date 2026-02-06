//! Sync receive endpoint — import remote events as observations.
//!
//! POST /v1/sync/receive
//!
//! Policy #2 enforcement: All imported remote events are tagged as observations.
//! Observations are recorded in the event log but do not mutate canonical state.

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use focusa_core::types::{EventLogEntry, FocusaEvent, SignalOrigin};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ReceiveBody {
    /// Events from remote peer (already filtered by cursor)
    events: Vec<RemoteEvent>,
    /// Peer that sent these events
    peer_id: String,
}

#[derive(Deserialize)]
pub struct RemoteEvent {
    event_id: String,
    timestamp: String,
    machine_id: String,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(flatten)]
    event: serde_json::Value,
}

pub async fn receive_impl(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReceiveBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut imported = 0;
    let mut skipped = 0;

    for remote in &body.events {
        // Parse event_id for idempotency check
        let event_id = match remote.event_id.parse::<Uuid>() {
            Ok(id) => id,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        // Check if already exists (idempotency)
        let exists = state
            .persistence
            .event_exists(&event_id.to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if exists {
            skipped += 1;
            continue;
        }

        // Parse the event payload
        let event: FocusaEvent = match serde_json::from_value(remote.event.clone()) {
            Ok(e) => e,
            Err(_) => {
                // Unknown event type — store as raw observation
                skipped += 1;
                continue;
            }
        };

        // Create entry as OBSERVATION (Policy #2)
        let entry = EventLogEntry {
            id: event_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&remote.timestamp)
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .with_timezone(&chrono::Utc),
            event,
            correlation_id: Some(format!("sync:from:{}", body.peer_id)),
            origin: SignalOrigin::Sync,
            machine_id: Some(remote.machine_id.clone()),
            instance_id: remote.instance_id.as_ref().and_then(|s| s.parse().ok()),
            session_id: remote.session_id.as_ref().and_then(|s| s.parse().ok()),
            thread_id: remote.thread_id.as_ref().and_then(|s| s.parse().ok()),
            is_observation: true, // POLICY #2: Always true for imports
        };

        // Persist observation
        if let Err(e) = state.persistence.append_event(&entry) {
            tracing::warn!("Failed to persist observation: {}", e);
            skipped += 1;
            continue;
        }

        // Broadcast to SSE subscribers
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(json);
        }

        imported += 1;
    }

    // Update peer cursor
    if let Some(last) = body.events.last() {
        let _ = state
            .persistence
            .set_cursor(&body.peer_id, Some(&last.event_id), Some(&last.timestamp));
    }

    Ok(Json(json!({
        "imported": imported,
        "skipped": skipped,
        "peer_id": body.peer_id,
    })))
}


