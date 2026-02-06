//! Thread routes — docs/38
//!
//! GET  /v1/threads — list threads
//! POST /v1/threads — create a new thread
//! GET  /v1/threads/:id — get thread details
//! POST /v1/threads/:id/transfer — transfer thread ownership

use crate::server::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::threads;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

type AppResult<T = Json<Value>> = Result<T, (StatusCode, Json<Value>)>;

/// GET /v1/threads — list threads in state.
async fn list_threads(State(state): State<Arc<AppState>>) -> Json<Value> {
    let focus_state = state.focusa.read().await;
    let threads: Vec<Value> = focus_state.threads.iter().map(|t| json!({
        "id": t.id.to_string(),
        "name": t.name,
        "status": format!("{:?}", t.status),
        "owner_machine_id": t.owner_machine_id,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })).collect();

    Json(json!({ "threads": threads }))
}

/// POST /v1/threads — create a new thread.
#[derive(Deserialize)]
struct CreateThreadBody {
    name: String,
    primary_intent: String,
    #[serde(default)]
    owner_machine_id: Option<String>,
}

async fn create_thread(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateThreadBody>,
) -> impl IntoResponse {
    let thread = threads::create_thread(
        &body.name,
        &body.primary_intent,
        body.owner_machine_id.as_deref(),
    );

    // Add to state
    {
        let mut focus_state = state.focusa.write().await;
        focus_state.threads.push(thread.clone());
    }

    (StatusCode::CREATED, Json(json!({
        "thread": {
            "id": thread.id.to_string(),
            "name": thread.name,
            "status": format!("{:?}", thread.status),
            "owner_machine_id": thread.owner_machine_id,
            "created_at": thread.created_at,
        }
    })))
}

/// GET /v1/threads/:id — get thread details.
async fn get_thread(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let focus_state = state.focusa.read().await;
    let thread = focus_state.threads.iter().find(|t| t.id.to_string() == id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Thread not found"}))))?;

    Ok(Json(json!({
        "thread": {
            "id": thread.id.to_string(),
            "name": thread.name,
            "status": format!("{:?}", thread.status),
            "owner_machine_id": thread.owner_machine_id,
            "created_at": thread.created_at,
            "updated_at": thread.updated_at,
            "thesis": thread.thesis,
        }
    })))
}

/// POST /v1/threads/:id/transfer — transfer thread ownership.
#[derive(Deserialize)]
struct TransferBody {
    to_machine_id: String,
    #[serde(default)]
    reason: Option<String>,
}

async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<TransferBody>,
) -> AppResult<Json<Value>> {
    let mut focus_state = state.focusa.write().await;
    let thread = focus_state.threads.iter_mut().find(|t| t.id.to_string() == id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Thread not found"}))))?;

    let previous_owner = thread.owner_machine_id.clone();
    thread.owner_machine_id = Some(body.to_machine_id.clone());
    thread.updated_at = chrono::Utc::now();

    Ok(Json(json!({
        "thread_id": id,
        "previous_owner": previous_owner,
        "new_owner": body.to_machine_id,
        "reason": body.reason,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/threads", get(list_threads).post(create_thread))
        .route("/v1/threads/{id}", get(get_thread))
        .route("/v1/threads/{id}/transfer", post(transfer_ownership))
}
