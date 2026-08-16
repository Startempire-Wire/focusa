//! RemoteWorkspaceBinding API surface (#89 slice 3).
//!
//! Controller-owned bindings: create/upsert, list, revoke. Persistence and
//! invariants live in `focusa-core::remote_workspace`; this route is a thin
//! typed boundary with the same invariant guarantees surfaced to callers.

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;
use focusa_core::remote_workspace::{ensure_schema, list_bindings, upsert_binding, BindingStatus, RemoteWorkspaceBinding};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/remote-workspaces/bindings", post(create_binding).get(list))
        .route("/v1/remote-workspaces/bindings/revoke", post(revoke_binding))
}

#[derive(Deserialize)]
pub struct ListParams {
    pub status: Option<String>,
}

fn db_path(state: &Arc<AppState>) -> std::path::PathBuf {
    crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir)
}

async fn create_binding(
    State(state): State<Arc<AppState>>,
    Json(binding): Json<RemoteWorkspaceBinding>,
) -> Json<Value> {
    let path = db_path(&state);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        ensure_schema(&conn)?;
        let (created, stored) = upsert_binding(&conn, &binding)?;
        Ok(json!({
            "status": if created { "created" } else { "updated" },
            "binding": stored,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Json<Value> {
    let path = db_path(&state);
    let status = params
        .status
        .as_deref()
        .and_then(|value| serde_json::from_str(&format!("\"{value}\"")).ok());
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        ensure_schema(&conn)?;
        let bindings = list_bindings(&conn, status)?;
        Ok(json!({"status": "listed", "bindings": bindings}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

async fn revoke_binding(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<Value> {
    let binding_id = match body.get("binding_id").and_then(Value::as_str) {
        Some(id) => id.to_string(),
        None => return Json(json!({"status": "rejected", "error": "binding_id is required"})),
    };
    let reason = body
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("operator")
        .to_string();
    let path = db_path(&state);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        ensure_schema(&conn)?;
        let mut bindings = list_bindings(&conn, None)?;
        let binding = match bindings.iter_mut().find(|entry| entry.binding_id == binding_id) {
            Some(binding) => binding,
            None => return Ok(json!({"status": "not_found", "binding_id": binding_id})),
        };
        let now = chrono::Utc::now().to_rfc3339();
        binding.revoke(&reason, &now);
        upsert_binding(&conn, binding)?;
        Ok(json!({
            "status": "revoked",
            "binding_id": binding_id,
            "revocation": format!("{now}|{reason}"),
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

// Bindings are revocation-typed: nothing is ever deleted.
#[allow(dead_code)]
fn _assert_revocation_only() -> BindingStatus {
    BindingStatus::Revoked
}
