//! Instance / daemon info routes.
//!
//! GET /v1/info

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use rusqlite::{Connection, OptionalExtension};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;

fn info_failure(error: impl Into<String>, why: impl Into<String>) -> Value {
    let error = error.into();
    let why = why.into();
    json!({
        "status": "blocked", "canonical": false, "degraded": true,
        "error": error, "failure_class": "persistence_failed", "why": why,
        "recovery_hint": "Check daemon data_dir and SQLite permissions before relying on /v1/info.",
        "misuse_hint": "Likely wrong data_dir, SQLite unavailable, or file permission/resource issue.",
        "next_tools": ["focusa_tool_doctor", "focusa_resource_mode"],
        "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": "persistence_failed", "summary": why, "retry": {"safe": true, "posture": "safe_retry", "reason": "persistence_failed"}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_resource_mode"], "error": {"code": "persistence_failed", "message": error}}}
    })
}

async fn info(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(info_failure(
                format!("db open failed: {e}"),
                format!("Info SQLite open failed: {e}"),
            ));
        }
    };

    let machine_id: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'machine_id'", [], |r| {
            r.get(0)
        })
        .optional()
        .unwrap_or(None);

    let schema_version: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);

    Json(json!({
        "ok": true,
        "machine_id": machine_id,
        "schema_version": schema_version,
        "api_bind": state.config.api_bind,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/info", get(info))
}

fn focusa_db_path(data_dir: &str) -> PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest).join("focusa.sqlite");
    }
    PathBuf::from(data_dir).join("focusa.sqlite")
}
