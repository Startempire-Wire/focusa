//! Instance routes.
//!
//! POST /v1/instances/connect
//! POST /v1/instances/disconnect
//! GET  /v1/instances/list (MVP: empty / stub)

use crate::server::AppState;
use axum::{routing::{get, post}, Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use focusa_core::types::{Action, InstanceKind};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct ConnectBody {
    kind: InstanceKind,
}

async fn connect(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConnectBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::InstanceConnect { kind: body.kind })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct DisconnectBody {
    instance_id: String,
    #[serde(default = "default_reason")]
    reason: String,
}

fn default_reason() -> String {
    "client_requested".to_string()
}

async fn disconnect(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DisconnectBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance_id = Uuid::parse_str(&body.instance_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

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

async fn list() -> Json<serde_json::Value> {
    // MVP stub: instance tracking is observability-only for now.
    Json(json!({"instances": []}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/instances/connect", post(connect))
        .route("/v1/instances/disconnect", post(disconnect))
        .route("/v1/instances/list", get(list))
}
