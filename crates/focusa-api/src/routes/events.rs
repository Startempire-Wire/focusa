//! Event routes.
//!
//! GET /v1/events/recent?limit=200 — read recent events from JSONL log

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
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

/// Read the last N events from the JSONL event log.
///
/// Reads the entire file and returns the last `limit` entries.
/// For MVP this is acceptable — the log is append-only and bounded
/// by session lifetime. A future optimization could use seek-from-end.
async fn recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentParams>,
) -> Json<Value> {
    let data_dir = expand_home(&state.config.data_dir);
    let log_path = data_dir.join("events/log.jsonl");

    if !log_path.exists() {
        return Json(json!({ "events": [], "total": 0 }));
    }

    let file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to open event log: {}", e);
            return Json(json!({ "events": [], "error": format!("Cannot read log: {}", e) }));
        }
    };

    let reader = BufReader::new(file);
    let mut entries: Vec<Value> = Vec::new();

    for line in reader.lines() {
        match line {
            Ok(l) if !l.trim().is_empty() => {
                match serde_json::from_str::<Value>(&l) {
                    Ok(v) => entries.push(v),
                    Err(e) => {
                        tracing::warn!("Skipping malformed event log line: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    let total = entries.len();
    // Return the last `limit` entries (most recent).
    let start = entries.len().saturating_sub(params.limit);
    let recent = &entries[start..];

    Json(json!({
        "events": recent,
        "total": total,
        "returned": recent.len(),
    }))
}

/// Expand ~ to $HOME (mirrors persistence.rs logic).
fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/events/recent", get(recent))
}
