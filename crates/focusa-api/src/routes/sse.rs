//! Server-Sent Events (SSE) endpoint for real-time TUI updates.
//!
//! Per 27-tui-spec §19: Event-driven updates via SSE.
//! Replaces polling with push-based updates.

use crate::server::AppState;
use axum::routing::get;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Sse, sse::Event},
};
use focusa_core::runtime::persistence_sqlite::{DurableEventRecord, SqlitePersistence};
use focusa_core::tool_result::{FailureClass, ToolResultV1, ToolStatus};
use serde::Deserialize;
use serde_json::{Value, json};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub type EventSender = broadcast::Sender<String>;
#[allow(dead_code)]
pub type EventReceiver = broadcast::Receiver<String>;

/// SSE event broadcaster.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    sender: EventSender,
}

#[allow(dead_code)]
impl EventBroadcaster {
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(100);
        Self { sender }
    }

    pub fn broadcast(&self, event: String) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Deserialize)]
struct StreamQuery {
    cursor: Option<String>,
}

fn resolve_durable_cursor(
    persistence: &SqlitePersistence,
    query_cursor: Option<&str>,
    last_event_id: Option<&str>,
) -> Result<u64, String> {
    let Some(raw) = query_cursor
        .or(last_event_id)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(0);
    };
    if let Ok(sequence) = raw.parse::<u64>() {
        return Ok(sequence);
    }
    persistence
        .durable_event_sequence(raw)
        .map_err(|error| format!("durable cursor lookup failed: {error}"))?
        .ok_or_else(|| format!("Last-Event-ID is not present in durable history: {raw}"))
}

fn event_scope_value(payload: &Value, key: &str) -> Value {
    payload
        .get("scope")
        .and_then(|scope| scope.get(key))
        .or_else(|| payload.get(key))
        .cloned()
        .unwrap_or(Value::Null)
}

fn durable_event_envelope(record: &DurableEventRecord) -> Value {
    let event_type = record
        .payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let schema_version = record
        .payload
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or("1.0.0");
    let source_state_revision = record
        .payload
        .get("source_state_revision")
        .cloned()
        .unwrap_or_else(|| json!(record.sequence));
    json!({
        "schema": "focusa.stream_event.v1",
        "event_id": record.event_id.as_str(),
        "sequence": record.sequence,
        "cursor": record.sequence.to_string(),
        "timestamp": record.timestamp.as_str(),
        "event_type": event_type,
        "schema_version": schema_version,
        "scope": {
            "project_root": event_scope_value(&record.payload, "project_root"),
            "continuity_id": event_scope_value(&record.payload, "continuity_id"),
            "attachment_id": event_scope_value(&record.payload, "attachment_id"),
            "work_surface_id": event_scope_value(&record.payload, "work_surface_id"),
        },
        "source_state_revision": source_state_revision,
        "payload_ref": record.payload.get("payload_ref").cloned().unwrap_or(Value::Null),
        "invalidate": record.payload.get("invalidate").cloned().unwrap_or_else(|| json!([])),
        "correlation_id": record.correlation_id.as_deref(),
        "causation_id": record.payload.get("causation_id").cloned().unwrap_or(Value::Null),
        "payload": &record.payload,
    })
}

/// Replay durable SQLite history, then tail the broadcast channel without gaps or duplicates.
async fn sse_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<
    Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<Value>),
> {
    // Subscribe before resolving/replaying so events committed during replay remain observable.
    let mut receiver = state.events_tx.subscribe();
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok());
    let mut cursor =
        resolve_durable_cursor(&state.persistence, params.cursor.as_deref(), last_event_id)
            .map_err(|message| {
                let result = ToolResultV1::failure(
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    message,
                )
                .with_recovery(
                    "Use a numeric cursor or a known durable event UUID",
                    "Do not retry an unknown Last-Event-ID unchanged",
                    ["focusa_tool_doctor"],
                );
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::to_value(result).unwrap_or_else(
                        |_| json!({"schema": "focusa.tool_result.v1", "ok": false}),
                    )),
                )
            })?;
    let persistence = state.persistence.clone();

    let stream = async_stream::stream! {
        loop {
            match persistence.durable_events_after(cursor, 256) {
                Ok(events) if !events.is_empty() => {
                    for record in events {
                        if record.sequence <= cursor {
                            continue;
                        }
                        cursor = record.sequence;
                        let data = durable_event_envelope(&record).to_string();
                        yield Ok(Event::default()
                            .id(record.sequence.to_string())
                            .event("focusa_event")
                            .data(data));
                    }
                    continue;
                }
                Err(error) => {
                    let result = ToolResultV1::failure(
                        ToolStatus::Degraded,
                        FailureClass::ReadModelLag,
                        format!("durable event replay failed: {error}"),
                    )
                    .with_recovery(
                        "Verify SQLite health, then reconnect from the last confirmed cursor",
                        "Do not skip ahead or invent a cursor",
                        ["focusa_tool_doctor"],
                    );
                    let data = serde_json::to_string(&result).unwrap_or_else(|_| {
                        "{\"schema\":\"focusa.tool_result.v1\",\"ok\":false}".to_string()
                    });
                    yield Ok(Event::default().event("focusa_stream_error").data(data));
                    break;
                }
                Ok(_) => {}
            }

            match tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await {
                Ok(Ok(_)) | Ok(Err(broadcast::error::RecvError::Lagged(_))) | Err(_) => continue,
                Ok(Err(broadcast::error::RecvError::Closed)) => break,
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

/// Health check endpoint for SSE.
async fn sse_health() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "status": "ready",
        "surface": "events_sse",
        "stream_route": "/v1/events/stream",
        "replay_source": "sqlite_event_hash_chain",
        "live_tail": "tokio_broadcast",
        "cursor": "stable_1_based_sequence_or_last_event_id",
        "message": "Durable replay plus live-tail SSE endpoint ready"
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/events/stream", get(sse_handler))
        .route("/v1/events/health", get(sse_health))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        broadcaster.broadcast("test event".to_string());

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "test event");
    }

    #[test]
    fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        broadcaster.broadcast("broadcast".to_string());

        assert_eq!(rx1.try_recv().unwrap(), "broadcast");
        assert_eq!(rx2.try_recv().unwrap(), "broadcast");
    }

    #[test]
    fn durable_envelope_carries_stable_sequence_and_cursor() {
        let record = DurableEventRecord {
            sequence: 42,
            event_id: "event-42".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            correlation_id: Some("correlation-42".to_string()),
            payload: json!({"type": "test", "project_root": "/tmp/project"}),
        };
        let envelope = durable_event_envelope(&record);
        assert_eq!(envelope["schema"], "focusa.stream_event.v1");
        assert_eq!(envelope["event_id"], "event-42");
        assert_eq!(envelope["sequence"], 42);
        assert_eq!(envelope["cursor"], "42");
        assert_eq!(envelope["event_type"], "test");
        assert_eq!(envelope["scope"]["project_root"], "/tmp/project");
        assert_eq!(envelope["correlation_id"], "correlation-42");
    }
}
