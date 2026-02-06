//! Thread routes — docs/38
//!
//! GET  /v1/threads — list threads
//! POST /v1/threads — create a new thread
//! GET  /v1/threads/:id — get thread details
//! POST /v1/threads/:id/transfer — transfer thread ownership

use crate::server::AppState;
use focusa_core::runtime::events::create_entry;
use focusa_core::types::{SignalOrigin, FocusaEvent};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::reducer;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

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
    // Create thread through reducer for proper event logging and state management.
    let thread_id = Uuid::now_v7();
    let event = FocusaEvent::ThreadCreated {
        thread_id,
        name: body.name.clone(),
        primary_intent: body.primary_intent.clone(),
        owner_machine_id: body.owner_machine_id.clone(),
    };

    let focusa_state = state.focusa.read().await;
    match reducer::reduce_with_meta(
        focusa_state.clone(),
        event.clone(),
        None, // No machine_id for local creation
        None, // No thread_id (creating new thread)
        false,
    ) {
        Ok(result) => {
            // Update state
            drop(focusa_state);
            {
                let mut focusa_state = state.focusa.write().await;
                *focusa_state = result.new_state;
            }

            // Persist the event
            let entry = create_entry(event, SignalOrigin::Cli, None);
            if let Err(e) = state.persistence.append_event(&entry) {
                tracing::warn!("Failed to persist thread creation event: {}", e);
            }

            // Save state
            let current_state = state.focusa.read().await;
            if let Err(e) = state.persistence.save_state(&current_state) {
                tracing::warn!("Failed to save state after thread creation: {}", e);
            }

            // Get the created thread from state
            drop(current_state);
            let focusa_state = state.focusa.read().await;
            let thread = focusa_state.threads.iter().find(|t| t.id == thread_id);

            match thread {
                Some(thread) => (StatusCode::CREATED, Json(json!({
                    "thread": {
                        "id": thread.id.to_string(),
                        "name": thread.name,
                        "status": format!("{:?}", thread.status),
                        "owner_machine_id": thread.owner_machine_id,
                        "created_at": thread.created_at,
                    }
                }))),
                None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "Thread creation failed"
                }))),
            }
        }
        Err(e) => {
            tracing::warn!("Thread creation rejected by reducer: {}", e);
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": e.to_string()
            })))
        }
    }
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
    // Parse thread_id
    let thread_id = match id.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid thread ID"})))),
    };

    // Get current state and machine_id
    let focusa_state = state.focusa.read().await;
    let thread = focusa_state.threads.iter().find(|t| t.id == thread_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Thread not found"}))))?;

    // Get this machine's ID for ownership verification
    let machine_id = state.persistence.machine_id()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get machine ID"}))))?;

    // Verify ownership (current owner must match this machine, or owner must be None)
    if let Some(owner) = &thread.owner_machine_id {
        if owner != &machine_id {
            return Err((StatusCode::FORBIDDEN, Json(json!({
                "error": "Only the thread owner can transfer ownership",
                "current_owner": owner
            }))));
        }
    }
    // If thread has no owner, anyone can transfer (first claim)

    let previous_owner = thread.owner_machine_id.clone();

    // Create ownership transfer event
    let event = FocusaEvent::ThreadOwnershipTransferred {
        thread_id,
        from_machine_id: previous_owner.clone(),
        to_machine_id: body.to_machine_id.clone(),
        reason: body.reason.clone().unwrap_or_default(),
    };

    // Run through reducer with proper ownership validation
    drop(focusa_state);
    let focusa_state = state.focusa.read().await;
    match reducer::reduce_with_meta(
        focusa_state.clone(),
        event.clone(),
        Some(&machine_id),
        Some(thread_id),
        false,
    ) {
        Ok(result) => {
            // Update state
            drop(focusa_state);
            {
                let mut focusa_state = state.focusa.write().await;
                *focusa_state = result.new_state;
            }

            // Persist the event
            let entry = create_entry(event, SignalOrigin::Cli, None);
            if let Err(e) = state.persistence.append_event(&entry) {
                tracing::warn!("Failed to persist ownership transfer event: {}", e);
            }

            // Save state
            let current_state = state.focusa.read().await;
            if let Err(e) = state.persistence.save_state(&current_state) {
                tracing::warn!("Failed to save state after ownership transfer: {}", e);
            }

            drop(current_state);
            let reason = body.reason.clone().unwrap_or_default();
            Ok(Json(json!({
                "thread_id": id,
                "previous_owner": previous_owner,
                "new_owner": body.to_machine_id,
                "reason": reason,
            })))
        }
        Err(e) => {
            tracing::warn!("Ownership transfer rejected by reducer: {}", e);
            Err((StatusCode::FORBIDDEN, Json(json!({
                "error": e.to_string()
            }))))
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/threads", get(list_threads).post(create_thread))
        .route("/v1/threads/{id}", get(get_thread))
        .route("/v1/threads/{id}/transfer", post(transfer_ownership))
}
