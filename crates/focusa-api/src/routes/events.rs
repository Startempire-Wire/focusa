//! Event routes.
//!
//! GET /v1/events/recent?limit=200    — read recent events from JSONL log
//! GET /v1/events/stream              — SSE event stream (Server-Sent Events)
//! GET /v1/events/:event_id           — get a specific event by ID

use crate::server::AppState;
use axum::extract::{Path as AxumPath, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{Json, Router, routing::get};
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
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
                if let Ok(v) = serde_json::from_str::<Value>(&l) {
                    entries.push(v);
                }
            }
            _ => {}
        }
    }

    let total = entries.len();
    let start = entries.len().saturating_sub(params.limit);
    let recent = &entries[start..];

    Json(json!({
        "events": recent,
        "total": total,
        "returned": recent.len(),
    }))
}

/// SSE event stream — real-time event push.
///
/// Polls event log every 500ms for new lines. Sends keepalive every 15s.
async fn stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let data_dir = expand_home(&state.config.data_dir);
    let log_path = data_dir.join("events/log.jsonl");

    let initial_len = std::fs::metadata(&log_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let stream = async_stream::stream! {
        let mut offset = initial_len;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let (new_events, bytes_read) = read_new_events(&log_path, offset);
            offset += bytes_read;

            for event_json in new_events {
                yield Ok(Event::default()
                    .event("focusa_event")
                    .data(event_json));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}

/// Read new JSONL lines from file starting at byte offset.
/// Returns (events, bytes_consumed) so the caller advances offset precisely.
fn read_new_events(path: &Path, offset: u64) -> (Vec<String>, u64) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (vec![], 0),
    };

    use std::io::{Read, Seek, SeekFrom};
    let mut file = file;
    if file.seek(SeekFrom::Start(offset)).is_err() {
        return (vec![], 0);
    }

    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return (vec![], 0);
    }

    let bytes_read = buf.len() as u64;
    let events = buf.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    (events, bytes_read)
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

/// GET /v1/events/:event_id — get a specific event by ID.
async fn get_event(
    State(state): State<Arc<AppState>>,
    AxumPath(event_id): AxumPath<String>,
) -> Json<Value> {
    let data_dir = expand_home(&state.config.data_dir);
    let log_path = data_dir.join("events/log.jsonl");

    if !log_path.exists() {
        return Json(json!({ "error": "Event log not found" }));
    }

    let file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(e) => {
            return Json(json!({ "error": format!("Cannot read log: {}", e) }));
        }
    };

    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(l) = line
            && !l.trim().is_empty()
        {
            if let Ok(v) = serde_json::from_str::<Value>(&l) {
                // Check if this event matches the ID.
                if v.get("id").and_then(|id| id.as_str()) == Some(&event_id) {
                    return Json(json!({ "event": v }));
                }
            }
        }
    }

    Json(json!({ "error": "Event not found" }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/events/recent", get(recent))
        .route("/v1/events/stream", get(stream))
        .route("/v1/events/{event_id}", get(get_event))
}
