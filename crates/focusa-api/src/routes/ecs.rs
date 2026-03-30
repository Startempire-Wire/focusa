//! ECS routes.
//!
//! GET  /v1/ecs/handles              — list all handles
//! POST /v1/ecs/store                — store an artifact
//! GET  /v1/ecs/resolve/:handle_id   — resolve a handle (metadata)
//! GET  /v1/ecs/content/:handle_id   — get artifact content
//! POST /v1/ecs/rehydrate/:handle_id — rehydrate with token limit

use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
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

/// GET /v1/ecs/handles — list all handles.
async fn list_handles(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "handles": focusa.reference_index.handles,
        "count": focusa.reference_index.handles.len(),
    }))
}

async fn resolve_handle(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    match focusa
        .reference_index
        .handles
        .iter()
        .find(|h| h.id == handle_id)
    {
        Some(handle) => Json(json!({"handle": handle})),
        None => Json(json!({"error": "handle not found"})),
    }
}

/// GET /v1/ecs/content/:handle_id — get artifact content.
async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use base64::Engine;
    let focusa = state.focusa.read().await;

    // Find handle.
    let handle = focusa
        .reference_index
        .handles
        .iter()
        .find(|h| h.id == handle_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Compute blob path from sha256.
    let ecs_root = expand_data_path(&state.config.data_dir).join("ecs/objects");
    let blob_path = ecs_root.join(&handle.sha256);

    // Get content from store.
    let content = std::fs::read(&blob_path).map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "handle_id": handle_id,
        "content_b64": base64::engine::general_purpose::STANDARD.encode(&content),
        "size": content.len(),
    })))
}

/// Expand ~ in path.
fn expand_data_path(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(rest);
    }
    std::path::PathBuf::from(path)
}

/// POST /v1/ecs/rehydrate/:handle_id — rehydrate with token limit.
#[derive(Deserialize)]
struct RehydrateQuery {
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    300
}

async fn rehydrate(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
    Query(query): Query<RehydrateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let focusa = state.focusa.read().await;

    // Find handle.
    let handle = focusa
        .reference_index
        .handles
        .iter()
        .find(|h| h.id == handle_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Compute blob path from sha256.
    let ecs_root = expand_data_path(&state.config.data_dir).join("ecs/objects");
    let blob_path = ecs_root.join(&handle.sha256);

    // Get content from store.
    let content = std::fs::read(&blob_path).map_err(|_| StatusCode::NOT_FOUND)?;

    // Convert to string if possible.
    let text = String::from_utf8_lossy(&content);

    // Estimate chars per token (rough: 4 chars = 1 token).
    let max_chars = (query.max_tokens * 4) as usize;
    let truncated = if text.len() > max_chars {
        // UTF-8 safe truncation.
        let boundary = text
            .char_indices()
            .take_while(|(i, _)| *i < max_chars)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}…", &text[..boundary])
    } else {
        text.to_string()
    };

    Ok(Json(json!({
        "handle_id": handle_id,
        "content": truncated,
        "truncated": text.len() > max_chars,
        "original_size": content.len(),
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ecs/handles", get(list_handles))
        .route("/v1/ecs/store", post(store_artifact))
        .route("/v1/ecs/resolve/{handle_id}", get(resolve_handle))
        .route("/v1/ecs/content/{handle_id}", get(get_content))
        .route("/v1/ecs/rehydrate/{handle_id}", post(rehydrate))
}
