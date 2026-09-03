//! Sync receive endpoint — import remote events as observations.
//!
//! POST /v1/sync/receive
//!
//! Policy #2 enforcement: All imported remote events are tagged as observations.
//! Observations are recorded in the event log but do not mutate canonical state.

use crate::server::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use focusa_core::{
    temporal_clock::TemporalActionEnvelope,
    types::{EventLogEntry, FocusaEvent, SignalOrigin},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

type SyncReceiveResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn receive_failure(
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

fn receive_persistence_failed(
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    receive_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("sync receive persistence failed: {error}"),
        "persistence_unavailable",
        "Sync receive could not check or persist remote observation state.",
        "Check SQLite/daemon health and retry after local persistence recovers.",
        "Likely database lock, wrong project database, or daemon shutdown during sync receive.",
    )
}

fn receive_timestamp_rejected(value: &str) -> (StatusCode, Json<serde_json::Value>) {
    receive_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid remote event timestamp: {value}"),
        "validation_rejected",
        "Sync receive event timestamp must be RFC3339.",
        "Correct the remote event timestamp before retrying unchanged.",
        "Likely incompatible peer version or malformed remote event payload.",
    )
}

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
    #[serde(default)]
    temporal: Option<TemporalActionEnvelope>,
    /// The event payload (FocusaEvent as JSON)
    event: serde_json::Value,
}

pub async fn receive_impl(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReceiveBody>,
) -> SyncReceiveResult {
    let mut imported = 0;
    let mut skipped = 0;
    let mut parse_failures = 0;
    let start = std::time::Instant::now();

    // Track the last successfully imported event for cursor update.
    // Only advance cursor to events we've actually persisted.
    let mut last_imported_id: Option<String> = None;
    let mut last_imported_ts: Option<String> = None;

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
            .map_err(receive_persistence_failed)?;

        if exists {
            skipped += 1;
            continue;
        }

        // Parse the event payload with detailed error logging
        let event: FocusaEvent = match serde_json::from_value(remote.event.clone()) {
            Ok(e) => e,
            Err(e) => {
                // Unknown or malformed event type
                tracing::warn!(
                    event_id = %event_id,
                    error = %e,
                    payload_len = remote.event.to_string().len(),
                    "Failed to parse remote event"
                );
                parse_failures += 1;
                skipped += 1;
                continue;
            }
        };

        // Create entry as OBSERVATION (Policy #2)
        let entry = EventLogEntry {
            id: event_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&remote.timestamp)
                .map_err(|_| receive_timestamp_rejected(&remote.timestamp))?
                .with_timezone(&chrono::Utc),
            temporal: remote.temporal.clone().unwrap_or_else(|| {
                TemporalActionEnvelope::unavailable(
                    "legacy_remote_event_missing_temporal_action_envelope",
                )
            }),
            event,
            correlation_id: Some(format!("sync:from:{}", body.peer_id)),
            origin: SignalOrigin::Sync,
            machine_id: Some(remote.machine_id.clone()),
            instance_id: remote.instance_id.as_ref().and_then(|s| s.parse().ok()),
            session_id: remote.session_id.as_ref().and_then(|s| s.parse().ok()),
            project_root: None,
            continuity_id: None,
            thread_id: remote.thread_id.as_ref().and_then(|s| s.parse().ok()),
            is_observation: true, // POLICY #2: Always true for imports
        };

        // Persist observation
        if let Err(e) = state.append_events_checkpoint(vec![entry.clone()]).await {
            tracing::warn!("Failed to persist observation: {}", e);
            skipped += 1;
            continue;
        }

        // Track last successfully imported event
        last_imported_id = Some(remote.event_id.clone());
        last_imported_ts = Some(remote.timestamp.clone());

        // Broadcast to SSE subscribers
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(json);
        }

        imported += 1;
    }

    // Update peer cursor to the last successfully imported event.
    // This ensures we don't skip events if some fail to import.
    if let (Some(last_id), Some(last_ts)) = (last_imported_id, last_imported_ts) {
        let _ = state
            .persistence
            .set_cursor(&body.peer_id, Some(&last_id), Some(&last_ts));
    }

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(
        peer_id = %body.peer_id,
        imported,
        skipped,
        parse_failures,
        elapsed_ms,
        "Sync receive completed"
    );

    Ok(Json(json!({
        "imported": imported,
        "skipped": skipped,
        "parse_failures": parse_failures,
        "peer_id": body.peer_id,
        "elapsed_ms": elapsed_ms,
    })))
}
