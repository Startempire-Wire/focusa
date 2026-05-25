//! Thread routes — docs/38
//!
//! GET  /v1/threads — list threads
//! POST /v1/threads — create a new thread
//! GET  /v1/threads/:id — get thread details
//! POST /v1/threads/:id/fork — fork a thread
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

fn thread_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe { "safe_retry" } else { "do_not_retry_unchanged" };
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
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn thread_validation_rejected(field: &str, why: impl Into<String>) -> (StatusCode, Json<Value>) {
    let why = why.into();
    thread_failure(
        StatusCode::BAD_REQUEST,
        why.clone(),
        "validation_rejected",
        why,
        "Correct the thread request payload before retrying unchanged.",
        "Likely empty required field or malformed thread id in the route path.",
        vec!["focusa_tool_doctor", "focusa_trajectory_view"],
    )
    .with_field(field)
}

fn thread_not_found(thread_id: &str) -> (StatusCode, Json<Value>) {
    thread_failure(
        StatusCode::NOT_FOUND,
        "Thread not found",
        "not_found",
        format!("thread_id {thread_id} is not present in Focusa thread state"),
        "Verify the thread id from create/list before get/fork/transfer.",
        "Likely stale thread id, wrong daemon instance, or thread not materialized yet.",
        vec!["focusa_tool_doctor", "focusa_traverse"],
    )
}

fn thread_dispatch_failed(
    action: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<Value>) {
    thread_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to dispatch {action}: {error}"),
        "daemon_unavailable",
        format!("thread {action} event could not be dispatched to daemon command channel"),
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status"],
    )
}

trait ThreadFailureFieldExt {
    fn with_field(self, field: &str) -> Self;
}

impl ThreadFailureFieldExt for (StatusCode, Json<Value>) {
    fn with_field(mut self, field: &str) -> Self {
        if let Some(obj) = self.1.0.as_object_mut() {
            obj.insert("field".to_string(), json!(field));
        }
        self
    }
}

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
        return Err(thread_validation_rejected("name", "name cannot be empty"));
    }
    if body.primary_intent.trim().is_empty() {
        return Err(thread_validation_rejected(
            "primary_intent",
            "primary_intent cannot be empty",
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
        .map_err(|error| thread_dispatch_failed("thread creation", error))?;

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
        .ok_or_else(|| thread_not_found(&id))?;

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

/// POST /v1/threads/:id/fork — fork a thread.
#[derive(Deserialize)]
struct ForkBody {
    name: String,
    #[serde(default)]
    owner_machine_id: Option<String>,
}

async fn fork_thread(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<ForkBody>,
) -> AppResult<Json<Value>> {
    if body.name.trim().is_empty() {
        return Err(thread_validation_rejected("name", "name cannot be empty"));
    }

    let thread_id = id
        .parse::<uuid::Uuid>()
        .map_err(|_| thread_validation_rejected("thread_id", "Invalid thread ID"))?;

    let source = {
        let focusa = state.focusa.read().await;
        focusa
            .threads
            .iter()
            .find(|t| t.id == thread_id)
            .cloned()
            .ok_or_else(|| thread_not_found(&id))?
    };

    let forked_id = Uuid::now_v7();
    let event = FocusaEvent::ThreadForked {
        source_thread_id: source.id,
        thread_id: forked_id,
        name: body.name.clone(),
        owner_machine_id: body.owner_machine_id.clone(),
    };

    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(|error| thread_dispatch_failed("thread fork", error))?;

    let mut forked = None;
    for _ in 0..80 {
        {
            let focusa = state.focusa.read().await;
            forked = focusa.threads.iter().find(|t| t.id == forked_id).cloned();
        }
        if forked.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    let forked = forked.ok_or_else(|| {
        (
            StatusCode::ACCEPTED,
            Json(json!({"status": "accepted", "thread_id": forked_id.to_string(), "warning": "thread fork dispatched but not yet visible"})),
        )
    })?;

    Ok(Json(json!({
        "thread": {
            "id": forked.id.to_string(),
            "name": forked.name,
            "status": format!("{:?}", forked.status),
            "owner_machine_id": forked.owner_machine_id,
            "created_at": forked.created_at,
            "updated_at": forked.updated_at,
            "thesis": forked.thesis,
            "clt_head": forked.clt_head,
            "forked_from": source.id.to_string(),
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
        return Err(thread_validation_rejected(
            "to_machine_id",
            "to_machine_id cannot be empty",
        ));
    }

    // Parse thread_id
    let thread_id = match id.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => return Err(thread_validation_rejected("thread_id", "Invalid thread ID")),
    };

    // Get current state
    let focusa_state = state.focusa.read().await;
    let thread = focusa_state
        .threads
        .iter()
        .find(|t| t.id == thread_id)
        .ok_or_else(|| thread_not_found(&id))?;

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
        .map_err(|error| thread_dispatch_failed("ownership transfer", error))?;

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
        .route("/v1/threads/{id}/fork", post(fork_thread))
        .route("/v1/threads/{id}/transfer", post(transfer_ownership))
}
