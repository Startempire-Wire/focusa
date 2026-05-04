//! Event routes.
//!
//! NOTE: SQLite is canonical; JSONL tailing is deprecated.
//! This module is retained temporarily until SSE is re-implemented
//! using an in-process broadcast channel.

use crate::server::AppState;
use axum::extract::{Path as AxumPath, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{Json, Router, routing::get};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Deserialize)]
struct RecentParams {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    cursor: Option<usize>,
    #[serde(default)]
    event_type: Option<String>,
}

fn default_limit() -> usize {
    20
}

fn event_type_matches(event: &Value, event_type: Option<&str>) -> bool {
    event_type
        .map(|wanted| event.get("event_type").and_then(|v| v.as_str()) == Some(wanted))
        .unwrap_or(true)
}

fn bounded_recent_events_from_reader<R: BufRead>(
    reader: R,
    limit: usize,
    cursor: Option<usize>,
    event_type: Option<&str>,
) -> (Vec<Value>, usize, Option<usize>) {
    let limit = limit.clamp(1, 1000);
    let mut total_matching = 0usize;
    let mut page = Vec::with_capacity(limit);
    let mut tail = VecDeque::with_capacity(limit);
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if !event_type_matches(&event, event_type) {
            continue;
        }
        let index = total_matching;
        total_matching += 1;
        if let Some(cursor) = cursor {
            if index >= cursor && page.len() < limit {
                page.push(event);
            }
        } else {
            if tail.len() == limit {
                tail.pop_front();
            }
            tail.push_back(event);
        }
    }
    if let Some(cursor) = cursor {
        let next_cursor = (cursor + page.len() < total_matching).then_some(cursor + page.len());
        (page, total_matching, next_cursor)
    } else {
        (tail.into_iter().collect(), total_matching, None)
    }
}

/// Read a bounded tail page from the JSONL event log without materializing the full log.
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
    let requested_limit = params.limit.clamp(1, 1000);
    let cursor = params.cursor;
    let (events, total, next_cursor) = bounded_recent_events_from_reader(
        reader,
        requested_limit,
        cursor,
        params.event_type.as_deref(),
    );

    Json(json!({
        "events": events,
        "total": total,
        "returned": events.len(),
        "limit": requested_limit,
        "cursor": cursor,
        "next_cursor": next_cursor,
        "truncated": next_cursor.is_some() || cursor.unwrap_or(0) > 0 || total > events.len(),
        "bounds": {
            "total": total,
            "returned": events.len(),
            "limit": requested_limit,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "truncated": next_cursor.is_some() || cursor.unwrap_or(0) > 0 || total > events.len(),
            "filter_event_type": params.event_type,
        }
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
            && let Ok(v) = serde_json::from_str::<Value>(&l) {
                // Check if this event matches the ID.
                if v.get("id").and_then(|id| id.as_str()) == Some(&event_id) {
                    return Json(json!({ "event": v }));
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

#[cfg(test)]
mod tests {
    use super::bounded_recent_events_from_reader;
    use std::io::Cursor;

    #[test]
    fn recent_events_tail_is_bounded_without_full_materialization() {
        let jsonl = (0..5)
            .map(|i| format!(r#"{{"event_type":"turn","n":{i}}}"#))
            .collect::<Vec<_>>()
            .join("\n");
        let (events, total, next_cursor) =
            bounded_recent_events_from_reader(Cursor::new(jsonl), 2, None, None);
        assert_eq!(total, 5);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["n"].as_i64(), Some(3));
        assert_eq!(events[1]["n"].as_i64(), Some(4));
        assert_eq!(next_cursor, None);
    }

    #[test]
    fn recent_events_cursor_pages_with_filter() {
        let jsonl = [
            r#"{"event_type":"a","n":0}"#,
            r#"{"event_type":"b","n":1}"#,
            r#"{"event_type":"a","n":2}"#,
            r#"{"event_type":"a","n":3}"#,
        ]
        .join("\n");
        let (events, total, next_cursor) =
            bounded_recent_events_from_reader(Cursor::new(jsonl), 1, Some(1), Some("a"));
        assert_eq!(total, 3);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["n"].as_i64(), Some(2));
        assert_eq!(next_cursor, Some(2));
    }
}
