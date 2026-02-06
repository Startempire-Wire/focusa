//! Instance / daemon info routes.
//!
//! GET /v1/info

use crate::server::AppState;
use axum::{routing::get, Json, Router};
use axum::extract::State;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

async fn info(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": format!("db open failed: {e}") })),
    };

    let machine_id: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'machine_id'",
            [],
            |r| r.get(0),
        )
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
    if let Some(rest) = data_dir.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest).join("focusa.sqlite");
        }
    }
    PathBuf::from(data_dir).join("focusa.sqlite")
}
