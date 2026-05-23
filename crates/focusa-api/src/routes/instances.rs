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

type InstanceResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn instance_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_project_identity"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": failure_class != "validation_rejected", "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry_after_recovery" }, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_project_identity"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn instance_dispatch_failed(
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    instance_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("instance dispatch failed: {error}"),
        "daemon_unavailable",
        "Instance action could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
    )
}

fn instance_missing_id() -> (StatusCode, Json<serde_json::Value>) {
    instance_failure(
        StatusCode::BAD_REQUEST,
        "missing instance_id",
        "validation_rejected",
        "Instance disconnect requires instance_id.",
        "Send instance_id before retrying disconnect unchanged.",
        "Likely stale extension payload or wrong route contract for disconnect.",
    )
}

/// Extension format: { instance_id, surface, session_id, cwd }
/// Canonical format: { kind }
#[allow(dead_code)]
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
) -> InstanceResult {
    state
        .command_tx
        .send(Action::InstanceConnect {
            kind: body.resolved_kind(),
        })
        .await
        .map_err(instance_dispatch_failed)?;

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
) -> InstanceResult {
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
        return Err(instance_missing_id());
    };

    state
        .command_tx
        .send(Action::InstanceDisconnect {
            instance_id,
            reason: body.reason,
        })
        .await
        .map_err(instance_dispatch_failed)?;

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
