//! Silent Session completion wait/backfill route (issue #311).
//!
//! - `GET /v1/silent-sessions/wait` — long-poll until the session settles
//!   (or the caller's timeout), returning the durable completion event.
//! - `GET /v1/silent-sessions/completions` — backfill cursor for missed
//!   events (`since_seq`).
//! - `POST /v1/silent-sessions/sweep-completions` — force one detection
//!   sweep; the daemon also sweeps on a timer so pushes need no client.

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;
use focusa_core::silent_session_completion_events::{
    ensure_schema, is_terminal_lifecycle, latest_completion, recent_completions,
    record_completion_event, SilentSessionCompletionEvent,
};

#[derive(Deserialize)]
pub struct WaitParams {
    pub session_id: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub since_seq: i64,
}

fn default_timeout_ms() -> u64 {
    30_000
}

#[derive(Deserialize)]
pub struct CompletionsParams {
    #[serde(default)]
    pub since_seq: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/wait", get(wait))
        .route("/v1/silent-sessions/completions", get(completions))
        .route("/v1/silent-sessions/sweep-completions", post(sweep_endpoint))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn completion_payload(event: &SilentSessionCompletionEvent) -> Value {
    json!({
        "schema": event.schema,
        "type": "silent_session_completed",
        "seq": event.seq,
        "session_id": event.session_id,
        "run_id": event.run_id,
        "generation": event.generation,
        "status": event.status,
        "summary": event.summary,
        "evidence_refs": event.evidence_refs,
        "created_at": event.created_at,
    })
}

fn record_and_broadcast_db(
    db_path: &std::path::Path,
    sender: &tokio::sync::broadcast::Sender<String>,
    event: SilentSessionCompletionEvent,
) -> Option<SilentSessionCompletionEvent> {
    let conn = rusqlite::Connection::open(db_path).ok()?;
    let recorded = record_completion_event(&conn, &event).ok()?;
    if !recorded {
        return None;
    }
    let latest = latest_completion(&conn, &event.session_id).ok().flatten()?;
    let payload = completion_payload(&latest);
    if let Ok(serialized) = serde_json::to_string(&payload) {
        let _ = sender.send(serialized);
    }
    Some(latest)
}

fn record_and_broadcast(
    state: &Arc<AppState>,
    session_id: &str,
    run_id: Option<String>,
    generation: Option<i64>,
    status: &str,
    summary: String,
    evidence_refs: Vec<String>,
) -> Option<SilentSessionCompletionEvent> {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    record_and_broadcast_db(
        &db_path,
        &state.events_tx,
        SilentSessionCompletionEvent {
            schema: focusa_core::silent_session_completion_events::SILENT_SESSION_COMPLETION_EVENT_SCHEMA
                .to_string(),
            seq: 0,
            session_id: session_id.to_string(),
            run_id,
            generation,
            status: status.to_string(),
            summary,
            evidence_refs,
            created_at: now_iso(),
        },
    )
}

/// Scan runtime_silent_sessions for settled sessions that have no recorded
/// completion event yet, record them, and broadcast. Returns how many new
/// completion events were emitted.
pub(crate) fn sweep_completions(
    db_path: &std::path::Path,
    sender: &tokio::sync::broadcast::Sender<String>,
) -> anyhow::Result<usize> {
    let conn = rusqlite::Connection::open(db_path)?;
    ensure_schema(&conn)?;
    let mut statement = conn.prepare(
        "SELECT session_id, lifecycle_state, projection_json
         FROM runtime_silent_sessions",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut emitted = 0;
    for (session_id, lifecycle, projection_raw) in rows {
        if !is_terminal_lifecycle(&lifecycle) {
            continue;
        }
        let projection: Value = serde_json::from_str(&projection_raw).unwrap_or(json!({}));
        let run_id = projection
            .get("run_id")
            .or_else(|| projection.get("run"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let generation = projection
            .get("generation")
            .and_then(|v| v.as_i64());
        let summary = projection
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let evidence_refs: Vec<String> = projection
            .get("evidence_refs")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if record_and_broadcast_db(
            &std::path::PathBuf::from(db_path),
            sender,
            SilentSessionCompletionEvent {
                schema: focusa_core::silent_session_completion_events::SILENT_SESSION_COMPLETION_EVENT_SCHEMA
                    .to_string(),
                seq: 0,
                session_id: session_id.clone(),
                run_id,
                generation,
                status: lifecycle,
                summary,
                evidence_refs,
                created_at: now_iso(),
            },
        )
        .is_some()
        {
            emitted += 1;
        }
    }
    Ok(emitted)
}

async fn wait(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WaitParams>,
) -> Json<Value> {
    let timeout = params.timeout_ms.clamp(100, 600_000);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout);
    loop {
        let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            if let Ok(Some(event)) = latest_completion(&conn, &params.session_id) {
                if event.seq > params.since_seq {
                    return Json(json!({
                        "status": "completed_event",
                        "event": completion_payload(&event),
                    }));
                }
            }
            // The session may have settled before the sweeper ran; detect it
            // here so the caller is never stuck waiting on a settled session.
            if let Ok((lifecycle, projection_raw)) = conn.query_row(
                "SELECT lifecycle_state, projection_json FROM runtime_silent_sessions WHERE session_id = ?1",
                [&params.session_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            ) {
                if is_terminal_lifecycle(&lifecycle) {
                    let projection: Value =
                        serde_json::from_str(&projection_raw).unwrap_or(json!({}));
                    let run_id = projection
                        .get("run_id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let generation = projection.get("generation").and_then(|v| v.as_i64());
                    let summary = projection
                        .get("summary")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let evidence_refs: Vec<String> = projection
                        .get("evidence_refs")
                        .and_then(|v| v.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|v| v.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(event) = record_and_broadcast(
                        &state,
                        &params.session_id,
                        run_id,
                        generation,
                        &lifecycle,
                        summary,
                        evidence_refs,
                    ) {
                        return Json(json!({
                            "status": "terminal",
                            "event": completion_payload(&event),
                        }));
                    }
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            let current: Value = conn_lifecycle(&state, &params.session_id)
                .map(|(lifecycle, _)| json!({"lifecycle_state": lifecycle}))
                .unwrap_or(json!({"lifecycle_state": "unknown"}));
            return Json(json!({
                "status": "waiting",
                "timed_out": true,
                "session_id": params.session_id,
                "current": current,
            }));
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

fn conn_lifecycle(state: &Arc<AppState>, session_id: &str) -> Option<(String, String)> {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let conn = rusqlite::Connection::open(db_path).ok()?;
    conn.query_row(
        "SELECT lifecycle_state, projection_json FROM runtime_silent_sessions WHERE session_id = ?1",
        [session_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .ok()
}

async fn completions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CompletionsParams>,
) -> Json<Value> {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<Value>> {
        let conn = rusqlite::Connection::open(db_path)?;
        let events = recent_completions(&conn, params.since_seq, params.limit.clamp(1, 500))?;
        Ok(events.iter().map(completion_payload).collect())
    })
    .await;
    match result {
        Ok(Ok(events)) => Json(json!({"status": "ok", "events": events})),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

async fn sweep_endpoint(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let tx = state.events_tx.clone();
    let result =
        tokio::task::spawn_blocking(move || sweep_completions(&db_path, &tx)).await;
    match result {
        Ok(Ok(emitted)) => Json(json!({"status": "ok", "emitted": emitted})),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}
