//! Attachment routes (Instance/Session ↔ Thread binding).
//!
//! POST /v1/attachments/attach
//! POST /v1/attachments/detach
//! GET  /v1/attachments/list

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, AttachmentRole};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

type AttachmentResult<T = Json<serde_json::Value>> =
    Result<T, (StatusCode, Json<serde_json::Value>)>;

fn attachment_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn attachment_invalid_uuid(field: &str, value: &str) -> (StatusCode, Json<serde_json::Value>) {
    attachment_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid {field}"),
        "validation_rejected",
        format!("{field} value {value} is not a valid UUID"),
        "Send valid UUID strings for instance_id, session_id, and thread_id before retrying unchanged.",
        "Likely stale client payload, wrong id field, or non-UUID route contract mismatch.",
    )
}

fn attachment_dispatch_failed(
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    attachment_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("attachment dispatch failed: {error}"),
        "daemon_unavailable",
        "attachment event could not be dispatched to daemon command channel",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
    )
}

#[derive(Deserialize)]
struct AttachBody {
    instance_id: String,
    session_id: String,
    thread_id: String,
    #[serde(default = "default_role")]
    role: AttachmentRole,
}

fn default_role() -> AttachmentRole {
    AttachmentRole::Active
}

async fn attach(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AttachBody>,
) -> AttachmentResult {
    let instance_id = Uuid::parse_str(&body.instance_id)
        .map_err(|_| attachment_invalid_uuid("instance_id", &body.instance_id))?;
    let session_id = Uuid::parse_str(&body.session_id)
        .map_err(|_| attachment_invalid_uuid("session_id", &body.session_id))?;
    let thread_id = Uuid::parse_str(&body.thread_id)
        .map_err(|_| attachment_invalid_uuid("thread_id", &body.thread_id))?;

    state
        .command_tx
        .send(Action::ThreadAttach {
            instance_id,
            session_id,
            thread_id,
            role: body.role,
        })
        .await
        .map_err(attachment_dispatch_failed)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct DetachBody {
    instance_id: String,
    session_id: String,
    thread_id: String,
    #[serde(default = "default_reason")]
    reason: String,
}

fn default_reason() -> String {
    "client_requested".to_string()
}

async fn detach(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DetachBody>,
) -> AttachmentResult {
    let instance_id = Uuid::parse_str(&body.instance_id)
        .map_err(|_| attachment_invalid_uuid("instance_id", &body.instance_id))?;
    let session_id = Uuid::parse_str(&body.session_id)
        .map_err(|_| attachment_invalid_uuid("session_id", &body.session_id))?;
    let thread_id = Uuid::parse_str(&body.thread_id)
        .map_err(|_| attachment_invalid_uuid("thread_id", &body.thread_id))?;

    state
        .command_tx
        .send(Action::ThreadDetach {
            instance_id,
            session_id,
            thread_id,
            reason: body.reason,
        })
        .await
        .map_err(attachment_dispatch_failed)?;

    Ok(Json(json!({"status": "accepted"})))
}

async fn list(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({"attachments": focusa.attachments}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/attachments/attach", post(attach))
        .route("/v1/attachments/detach", post(detach))
        .route("/v1/attachments/list", get(list))
}
