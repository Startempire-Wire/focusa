//! Thread routes — docs/38
//!
//! GET  /v1/threads — list threads
//! POST /v1/threads — create a new thread
//! GET  /v1/threads/:id — get thread details
//! POST /v1/threads/:id/transfer — transfer thread ownership

use crate::server::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, FocusaEvent};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

type AppResult<T = Json<Value>> = Result<T, (StatusCode, Json<Value>)>;

/// GET /v1/threads — list threads in state.
async fn list_threads(State(state): State<Arc<AppState>>) -> Json<Value> {
    let focus_state = state.focusa.read().await;
    let threads: Vec<Value> = focus_state
        .threads
        .iter()
        .map(|t| {
            json!({
                "id": t.id.to_string(),
                "name": t.name,
                "status": format!("{:?}", t.status),
                "owner_machine_id": t.owner_machine_id,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            })
        })
        .collect();

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
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate required fields
    if body.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "name cannot be empty"})),
        ));
    }
    if body.primary_intent.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "primary_intent cannot be empty"})),
        ));
    }

    let thread_id = Uuid::now_v7();
    let event = FocusaEvent::ThreadCreated {
        thread_id,
        name: body.name.clone(),
        primary_intent: body.primary_intent.clone(),
        owner_machine_id: body.owner_machine_id.clone(),
    };

    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to dispatch thread creation"})),
            )
        })?;

    // Wait briefly for daemon reducer + shared-state sync.
    for _ in 0..20 {
        {
            let focusa_state = state.focusa.read().await;
            if let Some(thread) = focusa_state.threads.iter().find(|t| t.id == thread_id) {
                return Ok((
                    StatusCode::CREATED,
                    Json(json!({
                        "thread": {
                            "id": thread.id.to_string(),
                            "name": thread.name,
                            "status": format!("{:?}", thread.status),
                            "owner_machine_id": thread.owner_machine_id,
                            "created_at": thread.created_at,
                        }
                    })),
                ));
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "accepted",
            "thread_id": thread_id.to_string(),
            "warning": "thread creation dispatched but not yet visible"
        })),
    ))
}

/// GET /v1/threads/:id — get thread details.
async fn get_thread(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let focus_state = state.focusa.read().await;
    let thread = focus_state
        .threads
        .iter()
        .find(|t| t.id.to_string() == id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Thread not found"})),
            )
        })?;

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
    // Validate to_machine_id is not empty
    if body.to_machine_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "to_machine_id cannot be empty"})),
        ));
    }

    // Parse thread_id
    let thread_id = match id.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid thread ID"})),
            ));
        }
    };

    // Get current state
    let focusa_state = state.focusa.read().await;
    let thread = focusa_state
        .threads
        .iter()
        .find(|t| t.id == thread_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Thread not found"})),
            )
        })?;

    let previous_owner = thread.owner_machine_id.clone();

    let event = FocusaEvent::ThreadOwnershipTransferred {
        thread_id,
        from_machine_id: previous_owner.clone(),
        to_machine_id: body.to_machine_id.clone(),
        reason: body.reason.clone().unwrap_or_default(),
    };
    drop(focusa_state);

    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to dispatch ownership transfer"})),
            )
        })?;

    for _ in 0..20 {
        {
            let focusa_state = state.focusa.read().await;
            if let Some(thread) = focusa_state.threads.iter().find(|t| t.id == thread_id)
                && thread.owner_machine_id.as_deref() == Some(body.to_machine_id.as_str())
            {
                let reason = body.reason.clone().unwrap_or_default();
                return Ok(Json(json!({
                    "thread_id": id,
                    "previous_owner": previous_owner,
                    "new_owner": body.to_machine_id,
                    "reason": reason,
                })));
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    Ok(Json(json!({
        "status": "accepted",
        "thread_id": id,
        "previous_owner": previous_owner,
        "new_owner": body.to_machine_id,
        "reason": body.reason.clone().unwrap_or_default(),
        "warning": "ownership transfer dispatched but not yet visible"
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/threads", get(list_threads).post(create_thread))
        .route("/v1/threads/{id}", get(get_thread))
        .route("/v1/threads/{id}/transfer", post(transfer_ownership))
}
