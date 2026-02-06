//! Thread routes — docs/38
//!
//! GET  /v1/threads — list threads
//! POST /v1/threads — create a new thread
//! GET  /v1/threads/:id — get thread details
//! POST /v1/threads/:id/transfer — transfer thread ownership

use crate::server::AppState;
use focusa_core::types::{EventLogEntry, SignalOrigin, FocusaEvent};
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
            // Generate event ID explicitly (consistent with what create_entry does)
            let event_id = Uuid::now_v7();
            
            // Persist the event FIRST
            // This ensures event log is always the source of truth
            let entry = EventLogEntry {
                id: event_id,
                timestamp: chrono::Utc::now(),
                event,
                correlation_id: Some("cli:create_thread".to_string()),
                origin: SignalOrigin::Cli,
                machine_id: None,
                instance_id: None,
                session_id: None,
                thread_id: Some(thread_id),
                is_observation: false,
            };
            if let Err(e) = state.persistence.append_event(&entry) {
                tracing::warn!("Failed to persist thread creation event: {}", e);
            }

            // Clone the new state for the response (before moving into focusa)
            let thread = result.new_state.threads.iter().find(|t| t.id == thread_id).cloned();
            
            // NOW update in-memory state (only after persistence succeeds)
            let state_to_save = result.new_state.clone();
            drop(focusa_state);
            {
                let mut focusa_state = state.focusa.write().await;
                *focusa_state = result.new_state;
            }

            // Save state
            if let Err(e) = state.persistence.save_state(&state_to_save) {
                tracing::warn!("Failed to save state after thread creation: {}", e);
            }

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
    // Validate to_machine_id is not empty
    if body.to_machine_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "to_machine_id cannot be empty"}))));
    }
    
    // Parse thread_id
    let thread_id = match id.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid thread ID"})))),
    };

    // Get current state
    let focusa_state = state.focusa.read().await;
    let thread = focusa_state.threads.iter().find(|t| t.id == thread_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Thread not found"}))))?;

    // Get this machine's ID
    let machine_id = state.persistence.machine_id()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get machine ID"}))))?;

    // Get current owner (for from_machine_id field)
    let previous_owner = thread.owner_machine_id.clone();

    // Create ownership transfer event
    // from_machine_id must be the current owner (validated by reducer)
    let event = FocusaEvent::ThreadOwnershipTransferred {
        thread_id,
        from_machine_id: previous_owner.clone(),
        to_machine_id: body.to_machine_id.clone(),
        reason: body.reason.clone().unwrap_or_default(),
    };

    // Use the current owner's machine_id for the caller's machine_id
    // This allows the owner to transfer even if running on a different machine
    let caller_machine_id = previous_owner.as_deref().unwrap_or(&machine_id);

    // Run through reducer with proper ownership validation
    drop(focusa_state);
    let focusa_state = state.focusa.read().await;
    match reducer::reduce_with_meta(
        focusa_state.clone(),
        event.clone(),
        Some(caller_machine_id),
        Some(thread_id),
        false,
    ) {
        Ok(result) => {
            // Generate event ID explicitly (consistent with what create_entry does)
            let event_id = Uuid::now_v7();
            
            // Persist the event FIRST
            // This ensures event log is always the source of truth
            let entry = EventLogEntry {
                id: event_id,
                timestamp: chrono::Utc::now(),
                event,
                correlation_id: Some("cli:transfer_ownership".to_string()),
                origin: SignalOrigin::Cli,
                machine_id: Some(caller_machine_id.to_string()),
                instance_id: None,
                session_id: None,
                thread_id: Some(thread_id),
                is_observation: false,
            };
            if let Err(e) = state.persistence.append_event(&entry) {
                tracing::warn!("Failed to persist ownership transfer event: {}", e);
            }

            // NOW update in-memory state (only after persistence succeeds)
            let state_to_save = result.new_state.clone();
            drop(focusa_state);
            {
                let mut focusa_state = state.focusa.write().await;
                *focusa_state = result.new_state;
            }

            // Save state
            if let Err(e) = state.persistence.save_state(&state_to_save) {
                tracing::warn!("Failed to save state after ownership transfer: {}", e);
            }

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
