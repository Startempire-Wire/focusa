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
        .route("/v1/workstreams/migrate", post(migrate_projects_to_workstreams))
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}

// Bindings are revocation-typed: nothing is ever deleted.
#[allow(dead_code)]
fn _assert_revocation_only() -> BindingStatus {
    BindingStatus::Revoked
}


/// Spec 164 slice 5 (#125): migrate existing project profiles into
/// workstream roots (preview + apply). Profiles live under
/// `~/.config/focusa/projects/*.json`; each one with a safe project_root
/// and continuity becomes a WorkstreamRoot row. Preview reports what
/// apply will create; apply performs the upserts idempotently.
#[derive(Deserialize)]
pub struct MigrateBody {
    #[serde(default)]
    pub preview: bool,
    #[serde(default)]
    pub apply: bool,
}

fn project_profiles_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".config")
        .join("focusa")
        .join("projects")
}

fn read_json_value(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

async fn migrate_projects_to_workstreams(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MigrateBody>,
) -> Json<Value> {
    use focusa_core::workstream_root::{load_workstream, upsert_workstream, WorkstreamRoot};

    let path = db_path(&state);
    let profiles: Vec<serde_json::Value> = match std::fs::read_dir(project_profiles_dir()) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.path().extension().is_some())
            .filter_map(|entry| read_json_value(&entry.path()))
            .collect(),
        Err(_) => vec![],
    };

    let candidates: Vec<Value> = profiles
        .iter()
        .filter_map(|profile| {
            let project_root = profile.get("project_root")?.as_str()?.to_string();
            let continuity_id = profile
                .get("continuity_id")
                .and_then(|value| value.as_str())
                .unwrap_or("default")
                .to_string();
            if project_root.is_empty()
                || !focusa_core::scope_safety::classify_project_root(&project_root).is_safe()
            {
                return None;
            }
            let workstream_id =
                focusa_core::workstream_root::workstream_scope_key(&project_root, &continuity_id);
            Some(json!({
                "workstream_id": workstream_id,
                "project_root": project_root,
                "continuity_id": continuity_id,
            }))
        })
        .collect();

    if body.preview {
        return Json(json!({
            "status": "preview",
            "candidates": candidates,
            "count": candidates.len(),
        }));
    }

    if !body.apply {
        return Json(json!({
            "status": "specify_preview_or_apply",
            "candidates": candidates,
            "count": candidates.len(),
        }));
    }

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(&path)?;
        focusa_core::workstream_root::ensure_schema(&conn)?;
        let mut created = Vec::new();
        let mut already = Vec::new();
        for candidate in candidates {
            let project_root = candidate["project_root"].as_str().unwrap_or_default().to_string();
            let continuity_id = candidate["continuity_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let workstream_id =
                focusa_core::workstream_root::workstream_scope_key(&project_root, &continuity_id);
            if load_workstream(&conn, &workstream_id)?.is_some() {
                already.push(workstream_id);
                continue;
            }
            let now = chrono::Utc::now().to_rfc3339();
            let root = WorkstreamRoot {
                schema: focusa_core::workstream_root::WORKSTREAM_SCHEMA.to_string(),
                workstream_id: workstream_id.clone(),
                root_scope: focusa_core::workstream_root::RootScope {
                    scope_kind: "project".to_string(),
                    remote_binding_id: None,
                    canonical_root: project_root,
                    working_subpath: None,
                },
                continuity: focusa_core::workstream_root::Continuity {
                    continuity_id,
                    principal: None,
                },
                runtime: focusa_core::workstream_root::partition_paths(
                    std::path::Path::new("."),
                    &workstream_id,
                ),
                state: focusa_core::workstream_root::WorkstreamState::Active,
                created_at: now.clone(),
                updated_at: now,
            };
            upsert_workstream(&conn, &root)?;
            created.push(workstream_id);
        }
        Ok(json!({
            "status": "applied",
            "created": created,
            "already_exists": already,
        }))
    })
    .await;

    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}
