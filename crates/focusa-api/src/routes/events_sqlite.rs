//! Event routes (SQLite canonical).
//!
//! GET /v1/events/recent?limit=200
//! GET /v1/events/:event_id
//!
//! NOTE: SSE streaming should be implemented via in-process broadcast channel,
//! not file tailing. This module only covers read APIs for now.

use crate::server::AppState;
use axum::extract::{Path as AxumPath, Query, State};
use axum::{Json, Router, routing::get};
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize)]
struct RecentParams {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    20
}

async fn recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentParams>,
) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(
                json!({"events": [], "total": 0, "error": format!("db open failed: {e}") }),
            );
        }
    };

    let mut stmt = match conn.prepare("SELECT payload_json FROM events ORDER BY ts DESC LIMIT ?1") {
        Ok(s) => s,
        Err(e) => {
            return Json(
                json!({"events": [], "total": 0, "error": format!("db query failed: {e}") }),
            );
        }
    };

    let rows = stmt
        .query_map([params.limit as i64], |row| row.get::<_, String>(0))
        .and_then(|iter| iter.collect::<Result<Vec<_>, _>>());

    let payloads = match rows {
        Ok(v) => v,
        Err(e) => {
            return Json(
                json!({"events": [], "total": 0, "error": format!("db read failed: {e}") }),
            );
        }
    };

    let mut events: Vec<Value> = Vec::new();
    for p in payloads.into_iter().rev() {
        if let Ok(v) = serde_json::from_str::<Value>(&p) {
            events.push(v);
        }
    }

    let total: i64 = conn
        .query_row("SELECT COUNT(1) FROM events", [], |r| r.get(0))
        .unwrap_or(0);

    Json(json!({
        "events": events,
        "total": total,
        "returned": events.len(),
    }))
}

async fn get_event(
    State(state): State<Arc<AppState>>,
    AxumPath(event_id): AxumPath<String>,
) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(json!({"error": format!("db open failed: {e}") }));
        }
    };

    let payload: Option<String> = conn
        .query_row(
            "SELECT payload_json FROM events WHERE event_id = ?1",
            [event_id.clone()],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);

    match payload {
        None => Json(json!({"error": "Event not found"})),
        Some(p) => {
            let v = serde_json::from_str::<Value>(&p).unwrap_or(json!({"raw": p}));
            Json(json!({"event": v}))
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/events/recent", get(recent))
        .route("/v1/events/{event_id}", get(get_event))
}

fn focusa_db_path(data_dir: &str) -> PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest).join("focusa.sqlite");
    }
    PathBuf::from(data_dir).join("focusa.sqlite")
}
