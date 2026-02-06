//! Sync routes (bidirectional push/pull + peer registry).
//!
//! GET  /v1/sync/peers          — list configured peers
//! POST /v1/sync/peers          — register/update a peer
//! DELETE /v1/sync/peers/:id    — remove a peer
//! POST /v1/sync/pull/:peer_id  — pull events since cursor from peer
//! POST /v1/sync/push/:peer_id  — push local events since cursor to peer
//! GET  /v1/sync/status/:peer_id — get sync cursor + backlog estimate

use crate::server::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use focusa_core::runtime::persistence_sqlite::SyncCursor;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
struct RegisterPeerBody {
    peer_id: String,
    name: String,
    endpoint: String,
    #[serde(default)]
    auth_token: Option<String>,
}

async fn list_peers(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let peers = state
        .persistence
        .list_peers()
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            json!({
                "peer_id": p.peer_id,
                "name": p.name,
                "endpoint": p.endpoint,
                "created_at": p.created_at,
                "last_seen_at": p.last_seen_at,
                "status": p.status,
            })
        })
        .collect::<Vec<_>>();
    Json(json!({"peers": peers}))
}

async fn register_peer(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterPeerBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .persistence
        .add_peer(
            &body.peer_id,
            &body.name,
            &body.endpoint,
            body.auth_token.as_deref(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"status": "registered"})))
}

async fn remove_peer(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .persistence
        .remove_peer(&peer_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"status": "removed"})))
}

async fn peer_status(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cursor: Option<SyncCursor> = state
        .persistence
        .get_cursor(&peer_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let backlog: usize = state
        .persistence
        .events_since(cursor.as_ref().and_then(|c| c.last_event_ts.as_deref()), None, 1000)
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(Json(json!({
        "peer_id": peer_id,
        "cursor": cursor.map(|c| json!({
            "last_event_id": c.last_event_id,
            "last_event_ts": c.last_event_ts,
            "updated_at": c.updated_at,
        })),
        "backlog_estimate": backlog,
    })))
}

async fn pull_from_peer(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // MVP: return local events that the peer should pull (server-side perspective).
    // Real implementation would call remote peer endpoint.
    let cursor = state
        .persistence
        .get_cursor(&peer_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let events = state
        .persistence
        .events_since(
            cursor.as_ref().and_then(|c| c.last_event_ts.as_deref()),
            cursor.as_ref().and_then(|c| c.last_event_id.as_deref()),
            100,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event_json: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            json!({
                "event_id": e.id.to_string(),
                "timestamp": e.timestamp,
                "machine_id": e.machine_id,
                "instance_id": e.instance_id,
                "session_id": e.session_id,
                "thread_id": e.thread_id,
                "origin": format!("{:?}", e.origin),
                "is_observation": e.is_observation,
                "event": e.event,
            })
        })
        .collect();

    Ok(Json(json!({
        "peer_id": peer_id,
        "events": event_json,
        "count": events.len(),
    })))
}

async fn push_to_peer(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // MVP stub: in real implementation, this would POST to remote peer's /v1/sync/receive.
    // For now, update cursor to "now" to acknowledge sync attempt.
    let now = chrono::Utc::now().to_rfc3339();
    state
        .persistence
        .set_cursor(&peer_id, None, Some(&now))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "peer_id": peer_id,
        "status": "acknowledged",
        "note": "MVP: push is a stub; implement remote POST to peer endpoint for full sync",
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/sync/peers", get(list_peers).post(register_peer))
        .route("/v1/sync/peers/{peer_id}", delete(remove_peer))
        .route("/v1/sync/status/{peer_id}", get(peer_status))
        .route("/v1/sync/pull/{peer_id}", post(pull_from_peer))
        .route("/v1/sync/push/{peer_id}", post(push_to_peer))
        .route("/v1/sync/receive", post(receive))
}

async fn receive(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Parse body into ReceiveBody
    let body: crate::routes::sync_receive::ReceiveBody = serde_json::from_value(body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    // Delegate to sync_receive module
    crate::routes::sync_receive::receive_impl(State(state), Json(body)).await
}
