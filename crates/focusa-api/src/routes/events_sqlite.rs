//! Event routes (SQLite canonical).
//!
//! GET /v1/events/recent?limit=200
//! GET /v1/events/:event_id
//!
//! NOTE: SSE streaming should be implemented via in-process broadcast channel,
//! not file tailing. This module only covers read APIs for now.

use crate::routes::bounded::{
    budgeted_default_limit, budgeted_hard_limit, budgeted_requested_limit,
    record_json_response_size,
};
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
    cursor: Option<String>,
    since: Option<String>,
    event_type: Option<String>,
}

fn default_limit() -> usize {
    budgeted_default_limit("FOCUSA_EVENTS_RECENT_DEFAULT_LIMIT", 20)
}

fn hard_limit() -> usize {
    budgeted_hard_limit("FOCUSA_EVENTS_RECENT_HARD_LIMIT", 500, default_limit())
}

fn events_failure(
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> Value {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    json!({
        "status": "blocked",
        "canonical": false,
        "degraded": true,
        "error": error,
        "failure_class": failure_class,
        "why": why,
        "recovery_hint": recovery_hint,
        "misuse_hint": misuse_hint,
        "next_tools": next_tools_value.clone(),
        "details": {
            "tool_result_v1": {
                "ok": false,
                "status": "blocked",
                "canonical": false,
                "degraded": true,
                "failure_class": failure_class,
                "summary": why,
                "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                "recovery_hint": recovery_hint,
                "misuse_hint": misuse_hint,
                "side_effects": [],
                "evidence_refs": [],
                "next_tools": next_tools_value,
                "error": {"code": failure_class, "message": error}
            }
        }
    })
}

fn events_db_failure(stage: &str, error: impl std::fmt::Display) -> Value {
    events_failure(
        format!("db {stage} failed: {error}"),
        "persistence_failed",
        format!("SQLite events {stage} operation failed: {error}"),
        "Check daemon persistence health and data_dir permissions before relying on event history.",
        "Likely SQLite unavailable, wrong data_dir, file permission issue, or resource pressure.",
        vec![
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_traverse",
        ],
    )
}

fn event_not_found(event_id: &str) -> Value {
    events_failure(
        "Event not found",
        "not_found",
        format!("event_id {event_id} is not present in SQLite event history"),
        "Use /v1/events/recent to discover valid event ids before requesting a specific event.",
        "Likely stale event id, wrong daemon data_dir, or pruned/unpersisted event history.",
        vec!["focusa_traverse", "focusa_tool_doctor"],
    )
}

async fn recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentParams>,
) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            let mut payload = events_db_failure("open", e);
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("events".to_string(), json!([]));
                obj.insert("total".to_string(), json!(0));
            }
            return Json(payload);
        }
    };

    let limit = budgeted_requested_limit(Some(params.limit), default_limit(), hard_limit());
    let query_limit = limit + 1;
    // Bounded window for the event_type LIKE subquery. 50k rows is well under the
    // 5s API timeout on busy hosts while still returning recent activity. The
    // outer LIMIT-N keeps the wire response small.
    let recent_window: usize = 50_000;

    // Validate event_type shape up front to avoid a payload_json LIKE full-table scan
    // when the caller passes garbage (e.g. focusa audit --event-type DefinitelyNotARealType).
    // The events.payload_json column has no index, so a non-matching LIKE scans the entire
    // events table (5+ GB on busy hosts) and trips the 5s API_TIMEOUT. Reject malformed or
    // unknown-shaped identifiers with a 400 instead of silently timing out.
    if let Some(event_type) = params.event_type.as_deref() {
        if event_type.is_empty()
            || event_type.len() > 64
            || !event_type
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            let mut payload = events_failure(
                format!("invalid event_type filter: {event_type:?}"),
                "validation_rejected",
                "event_type must match [A-Za-z0-9_]{1,64}",
                "Pass a known event type (e.g. MemoryDecayTick, TurnStarted) or omit the filter.",
                "Likely a CLI typo, a removed event class, or an injection probe.",
                vec!["focusa audit", "focusa traverse"],
            );
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("events".to_string(), json!([]));
                obj.insert("total".to_string(), json!(0));
                obj.insert("returned".to_string(), json!(0));
            }
            return Json(payload);
        }
    }

    let mut sql = "SELECT ts, payload_json FROM events".to_string();
    let mut clauses = Vec::new();
    if params.cursor.is_some() {
        clauses.push("ts < ?".to_string());
    }
    if params.since.is_some() {
        clauses.push("ts >= ?".to_string());
    }
    // event_type filter is a LIKE on the JSON blob (no index on payload_json).
    // Wrap it in a subquery bounded by the most recent rows so the LIKE scan
    // is O(recent_window) instead of O(all events). Without this bound, a
    // non-matching event_type on a 5+ GB events table trips the 5s API_TIMEOUT.
    if params.event_type.is_some() {
        sql = format!(
            "SELECT ts, payload_json FROM (SELECT ts, payload_json FROM events ORDER BY ts DESC LIMIT {recent_window})"
        );
        clauses.push("payload_json LIKE ?".to_string());
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY ts DESC LIMIT ?");

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            let mut payload = events_db_failure("query", e);
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("events".to_string(), json!([]));
                obj.insert("total".to_string(), json!(0));
            }
            return Json(payload);
        }
    };

    let mut values: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(cursor) = &params.cursor {
        values.push(cursor.clone().into());
    }
    if let Some(since) = &params.since {
        values.push(since.clone().into());
    }
    if let Some(event_type) = &params.event_type {
        values.push(format!("%\"{}\"%", event_type).into());
    }
    values.push((query_limit as i64).into());

    let rows = stmt
        .query_map(rusqlite::params_from_iter(values), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .and_then(|iter| iter.collect::<Result<Vec<_>, _>>());

    let rows = match rows {
        Ok(v) => v,
        Err(e) => {
            let mut payload = events_db_failure("read", e);
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("events".to_string(), json!([]));
                obj.insert("total".to_string(), json!(0));
            }
            return Json(payload);
        }
    };

    let has_more = rows.len() > limit;
    let mut page_rows = rows;
    if has_more {
        page_rows.truncate(limit);
    }
    let next_cursor = has_more
        .then(|| page_rows.last().map(|(ts, _)| ts.clone()))
        .flatten();
    let mut events: Vec<Value> = Vec::new();
    for (_, p) in page_rows.into_iter().rev() {
        if let Ok(v) = serde_json::from_str::<Value>(&p) {
            events.push(v);
        }
    }
    if params.event_type.is_none()
        && !events
            .iter()
            .any(|event| event.get("type").and_then(Value::as_str) == Some("MemoryDecayTick"))
        && let Ok(Some(payload)) = conn
            .query_row(
                "SELECT payload_json FROM events WHERE payload_json LIKE ?1 ORDER BY ts DESC LIMIT 1",
                ["%\"MemoryDecayTick\"%"],
                |row| row.get::<_, String>(0),
            )
            .optional()
        && let Ok(event) = serde_json::from_str::<Value>(&payload)
    {
        events.insert(0, event);
    }

    let total: i64 = conn
        .query_row("SELECT COUNT(1) FROM events", [], |r| r.get(0))
        .unwrap_or(0);

    let cursor_filter = params.cursor.clone();
    let since_filter = params.since.clone();
    let event_type_filter = params.event_type.clone();
    let next_cursor_for_bounds = next_cursor.clone();
    let next_cursor_for_rehydrate = next_cursor.clone();
    let payload = json!({
        "events": events,
        "total": total,
        "returned": events.len(),
        "limit": limit,
        "next_cursor": next_cursor,
        "truncated": has_more,
        "bounds": {
            "total": total,
            "returned": events.len(),
            "limit": limit,
            "cursor": cursor_filter,
            "next_cursor": next_cursor_for_bounds,
            "truncated": has_more,
            "rehydrate": {"route":"/v1/events/recent", "cursor": next_cursor_for_rehydrate}
        },
        "filters": {
            "cursor": params.cursor,
            "since": since_filter,
            "event_type": event_type_filter,
        },
        "tail_strategy": "sqlite_reverse_ts_bounded",
    });
    record_json_response_size("/v1/events/recent", &payload);
    Json(payload)
}

async fn get_event(
    State(state): State<Arc<AppState>>,
    AxumPath(event_id): AxumPath<String>,
) -> Json<Value> {
    let db_path = focusa_db_path(&state.config.data_dir);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(events_db_failure("open", e));
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
        None => Json(event_not_found(&event_id)),
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
