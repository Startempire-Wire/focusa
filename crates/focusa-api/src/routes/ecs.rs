//! ECS routes.
//!
//! POST /v1/ecs/store               — store an artifact
//! GET  /v1/ecs/resolve/:handle_id  — resolve a handle

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::{Action, HandleKind};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
struct StoreBody {
    kind: HandleKind,
    label: String,
    /// Base64-encoded content.
    content_b64: String,
}

async fn store_artifact(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use base64::Engine;
    let content = base64::engine::general_purpose::STANDARD
        .decode(&body.content_b64)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .command_tx
        .send(Action::StoreArtifact {
            kind: body.kind,
            label: body.label,
            content,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

async fn resolve_handle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(handle_id): axum::extract::Path<uuid::Uuid>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    match focusa.reference_index.handles.iter().find(|h| h.id == handle_id) {
        Some(handle) => Json(json!({"handle": handle})),
        None => Json(json!({"error": "handle not found"})),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ecs/store", post(store_artifact))
        .route("/v1/ecs/resolve/{handle_id}", get(resolve_handle))
}
