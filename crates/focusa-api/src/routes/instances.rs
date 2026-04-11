//! Instance routes.
//!
//! POST /v1/instances/connect
//! POST /v1/instances/disconnect
//! GET  /v1/instances/list (MVP: stub)

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, InstanceKind};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Extension format: { instance_id, surface, session_id, cwd }
/// Canonical format: { kind }
#[derive(Debug, Clone, Deserialize)]
struct ConnectBody {
    instance_id: Option<String>,
    surface: Option<String>,
    session_id: Option<String>,
    cwd: Option<String>,
    #[serde(default)]
    kind: Option<InstanceKind>,
}

impl ConnectBody {
    fn resolved_kind(&self) -> InstanceKind {
        self.kind.unwrap_or(InstanceKind::Background)
    }
}
/// POST /v1/instances/connect
async fn connect(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConnectBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::InstanceConnect {
            kind: body.resolved_kind(),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut resp = json!({"status": "accepted"});
    if let Some(ref iid) = body.instance_id {
        resp["instance_id"] = json!(iid);
    }
    Ok(Json(resp))
}

/// POST /v1/instances/disconnect — instance_id is optional (UUID or string)
#[derive(Debug, Clone, Deserialize)]
struct DisconnectBody {
    instance_id: Option<String>,
    #[serde(default = "default_reason")]
    reason: String,
}

fn default_reason() -> String {
    "client_requested".to_string()
}

/// POST /v1/instances/disconnect
async fn disconnect(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DisconnectBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Accept UUID or string instance_id
    let instance_id = if let Some(ref iid) = body.instance_id {
        if let Ok(uuid) = Uuid::parse_str(iid) {
            uuid
        } else {
            // Non-UUID string — generate a stable UUID from the string
            let mut hash: u128 = 0;
            for byte in iid.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u128);
            }
            Uuid::from_u128(hash)
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    state
        .command_tx
        .send(Action::InstanceDisconnect {
            instance_id,
            reason: body.reason,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

/// GET /v1/instances/list
async fn list(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({"instances": focusa.instances}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/instances/connect", post(connect))
        .route("/v1/instances/disconnect", post(disconnect))
        .route("/v1/instances/list", get(list))
}
