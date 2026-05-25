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

type SyncResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn sync_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: &[&str],
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools = next_tools.to_vec();
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools,
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {
                        "safe": failure_class != "validation_rejected",
                        "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry" },
                        "reason": failure_class,
                    },
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools,
                    "error": {"code": failure_class, "message": error},
                }
            }
        })),
    )
}

fn sync_persistence_failed(
    operation: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("sync persistence failed during {operation}: {error}"),
        "persistence_unavailable",
        format!("Sync route could not complete persistence operation: {operation}."),
        "Check SQLite/daemon health, verify the project database is writable, then retry after recovery.",
        "Likely database lock, corrupted local persistence, wrong project root, or daemon shutdown during sync.",
        &["focusa_tool_doctor", "focusa_project_identity"],
    )
}

fn sync_peer_not_found(peer_id: &str) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::NOT_FOUND,
        format!("sync peer not found: {peer_id}"),
        "not_found",
        format!("No registered sync peer matched peer_id={peer_id}."),
        "Register the peer with POST /v1/sync/peers or choose a listed peer before retrying push/pull.",
        "Likely stale peer id, wrong project database, or sync registry not initialized for this daemon.",
        &["focusa_tool_doctor", "focusa_project_identity"],
    )
}

fn sync_upstream_failed(
    peer_id: &str,
    endpoint: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::BAD_GATEWAY,
        format!("sync upstream send failed for {peer_id}: {error}"),
        "upstream_unavailable",
        format!("Focusa could not reach sync peer {peer_id} at {endpoint}."),
        "Verify the peer endpoint, network path, daemon health, and auth requirements before retrying.",
        "Likely stale endpoint URL, offline peer daemon, network/TLS failure, or remote route mismatch.",
        &["focusa_tool_doctor", "focusa_project_identity"],
    )
}

fn sync_upstream_status_failed(
    peer_id: &str,
    endpoint: &str,
    status: StatusCode,
) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::BAD_GATEWAY,
        format!("sync upstream rejected {peer_id}: HTTP {}", status.as_u16()),
        "upstream_rejected",
        format!(
            "Sync peer {peer_id} rejected receive request at {endpoint} with HTTP {}.",
            status.as_u16()
        ),
        "Inspect the remote peer response/logs and fix auth, route, schema, or daemon health before retrying.",
        "Likely incompatible peer version, missing auth, rejected payload, or remote daemon degradation.",
        &["focusa_tool_doctor", "focusa_project_identity"],
    )
}

fn sync_upstream_body_failed(
    peer_id: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::BAD_GATEWAY,
        format!("sync upstream response parse failed for {peer_id}: {error}"),
        "upstream_invalid_response",
        format!("Sync peer {peer_id} returned a response that was not valid JSON."),
        "Inspect the remote peer response body/logs and confirm compatible /v1/sync/receive behavior.",
        "Likely proxy/html error response, incompatible peer, or route returning non-Focusa payload.",
        &["focusa_tool_doctor"],
    )
}

fn sync_validation_failed(
    payload: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    sync_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid {payload}: {error}"),
        "validation_rejected",
        format!("Sync {payload} did not match the expected schema."),
        "Correct the JSON payload shape/types before retrying unchanged.",
        "Likely wrong sync API version, missing events/peer_id, malformed timestamp, or non-Focusa payload.",
        &["focusa_tool_doctor"],
    )
}

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
) -> SyncResult {
    state
        .persistence
        .add_peer(
            &body.peer_id,
            &body.name,
            &body.endpoint,
            body.auth_token.as_deref(),
        )
        .map_err(|e| sync_persistence_failed("register_peer", e))?;
    Ok(Json(json!({"status": "registered"})))
}

async fn remove_peer(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> SyncResult {
    state
        .persistence
        .remove_peer(&peer_id)
        .map_err(|e| sync_persistence_failed("remove_peer", e))?;
    Ok(Json(json!({"status": "removed"})))
}

async fn peer_status(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> SyncResult {
    let cursor: Option<SyncCursor> = state
        .persistence
        .get_cursor(&peer_id)
        .map_err(|e| sync_persistence_failed("get_cursor", e))?;

    let backlog: usize = state
        .persistence
        .events_since(
            cursor.as_ref().and_then(|c| c.last_event_ts.as_deref()),
            None,
            1000,
        )
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
) -> SyncResult {
    // MVP: return local events that the peer should pull (server-side perspective).
    // Real implementation would call remote peer endpoint.
    let cursor = state
        .persistence
        .get_cursor(&peer_id)
        .map_err(|e| sync_persistence_failed("get_cursor", e))?;

    let events = state
        .persistence
        .events_since(
            cursor.as_ref().and_then(|c| c.last_event_ts.as_deref()),
            cursor.as_ref().and_then(|c| c.last_event_id.as_deref()),
            100,
        )
        .map_err(|e| sync_persistence_failed("events_since", e))?;

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
) -> SyncResult {
    // Get peer info to find endpoint
    let peers = state
        .persistence
        .list_peers()
        .map_err(|e| sync_persistence_failed("list_peers", e))?;
    let peer = peers
        .into_iter()
        .find(|p| p.peer_id == peer_id)
        .ok_or_else(|| sync_peer_not_found(&peer_id))?;

    // Get local events since cursor
    let cursor = state
        .persistence
        .get_cursor(&peer_id)
        .map_err(|e| sync_persistence_failed("get_cursor", e))?;
    let events = state
        .persistence
        .events_since(
            cursor.as_ref().and_then(|c| c.last_event_ts.as_deref()),
            cursor.as_ref().and_then(|c| c.last_event_id.as_deref()),
            100, // Batch size limit
        )
        .map_err(|e| sync_persistence_failed("events_since", e))?;

    // Prepare payload for remote
    let payload = json!({
        "peer_id": "self", // Will be replaced by peer_id in request
        "events": events.iter().map(|e| json!({
            "event_id": e.id.to_string(),
            "timestamp": e.timestamp.to_rfc3339(),
            "machine_id": e.machine_id.clone().unwrap_or_else(|| "unknown".to_string()),
            "instance_id": e.instance_id.map(|v| v.to_string()),
            "session_id": e.session_id.map(|v| v.to_string()),
            "thread_id": e.thread_id.map(|v| v.to_string()),
            "event": e.event,
        })).collect::<Vec<_>>(),
    });

    // POST to remote peer's receive endpoint
    let client = reqwest::Client::new();
    let receive_url = format!("{}/v1/sync/receive", peer.endpoint);

    let response = client
        .post(&receive_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to push to peer {}: {}", peer_id, e);
            sync_upstream_failed(&peer_id, &receive_url, e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        tracing::error!("Peer {} returned status {}", peer_id, status);
        return Err(sync_upstream_status_failed(&peer_id, &receive_url, status));
    }

    // Update cursor to last event sent (not just now())
    let last_event = events.last();
    let last_event_id = last_event.map(|e| e.id.to_string());
    let last_event_ts = last_event.map(|e| e.timestamp.to_rfc3339());
    state
        .persistence
        .set_cursor(&peer_id, last_event_id.as_deref(), last_event_ts.as_deref())
        .map_err(|e| sync_persistence_failed("set_cursor", e))?;

    let result = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| sync_upstream_body_failed(&peer_id, e))?;

    Ok(Json(json!({
        "peer_id": peer_id,
        "status": "pushed",
        "events_sent": events.len(),
        "remote_response": result,
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
        .route("/v1/sync/transfer", post(transfer))
}

async fn receive(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> SyncResult {
    // Parse body into ReceiveBody
    let body: crate::routes::sync_receive::ReceiveBody =
        serde_json::from_value(body).map_err(|e| sync_validation_failed("receive payload", e))?;
    // Delegate to sync_receive module
    crate::routes::sync_receive::receive_impl(State(state), Json(body)).await
}

async fn transfer(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> SyncResult {
    // Parse body into TransferBody
    let body: crate::routes::sync_transfer::TransferBody =
        serde_json::from_value(body).map_err(|e| sync_validation_failed("transfer payload", e))?;
    // Delegate to sync_transfer module
    crate::routes::sync_transfer::transfer_impl(State(state), Json(body)).await
}
