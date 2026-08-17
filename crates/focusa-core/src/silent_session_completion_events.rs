//! Durable Silent Session completion events (issue #311).
//!
//! When a silent session run settles (completed/failed/cancelled), the API
//! layer records a typed completion event here and broadcasts it over SSE so
//! agents are notified instead of polling. The table doubles as the
//! at-least-once delivery log: missed events are recoverable by re-reading
//! `recent_silent_session_completions` with a `since_seq` backfill cursor.

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const SILENT_SESSION_COMPLETION_EVENT_SCHEMA: &str = "focusa.silent_session_completion.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionCompletionEvent {
    pub schema: String,
    pub seq: i64,
    pub session_id: String,
    pub run_id: Option<String>,
    pub generation: Option<i64>,
    pub status: String,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub created_at: String,
}

/// Lifecycle states that settle a session (serde snake_case values).
pub fn is_terminal_lifecycle(state: &str) -> bool {
    matches!(state, "completed" | "failed" | "cancelled")
}

/// Create the completion-event table if missing. Safe to call on demand.
pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS silent_session_completion_events (
           seq INTEGER PRIMARY KEY AUTOINCREMENT,
           session_id TEXT NOT NULL,
           run_id TEXT,
           generation INTEGER,
           status TEXT NOT NULL,
           summary TEXT NOT NULL DEFAULT '',
           evidence_refs TEXT NOT NULL DEFAULT '[]',
           created_at TEXT NOT NULL,
           UNIQUE(session_id, run_id, status)
         );
         CREATE INDEX IF NOT EXISTS idx_ssce_session ON silent_session_completion_events(session_id);
         CREATE INDEX IF NOT EXISTS idx_ssce_created ON silent_session_completion_events(created_at);",
    )?;
    Ok(())
}

/// Record a completion event. Returns `Ok(true)` when the event is new
/// (caller should broadcast), `Ok(false)` when it was already recorded.
pub fn record_completion_event(conn: &Connection, event: &SilentSessionCompletionEvent) -> Result<bool> {
    ensure_schema(conn)?;
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO silent_session_completion_events
           (session_id, run_id, generation, status, summary, evidence_refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            event.session_id,
            event.run_id,
            event.generation,
            event.status,
            event.summary,
            serde_json::to_string(&event.evidence_refs)?,
            event.created_at,
        ],
    )?;
    Ok(inserted > 0)
}

/// Latest recorded completion for a session.
pub fn latest_completion(conn: &Connection, session_id: &str) -> Result<Option<SilentSessionCompletionEvent>> {
    ensure_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT seq, session_id, run_id, generation, status, summary, evidence_refs, created_at
         FROM silent_session_completion_events
         WHERE session_id = ?1
         ORDER BY seq DESC LIMIT 1",
    )?;
    let mut rows = statement.query_map([session_id], row_to_event)?;
    Ok(rows.next().transpose()?)
}

/// Backfill: completion events with seq greater than `since_seq`.
pub fn recent_completions(
    conn: &Connection,
    since_seq: i64,
    limit: i64,
) -> Result<Vec<SilentSessionCompletionEvent>> {
    ensure_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT seq, session_id, run_id, generation, status, summary, evidence_refs, created_at
         FROM silent_session_completion_events
         WHERE seq > ?1
         ORDER BY seq ASC LIMIT ?2",
    )?;
    let events = statement
        .query_map(rusqlite::params![since_seq, limit], row_to_event)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(events)
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<SilentSessionCompletionEvent> {
    let evidence_raw: String = row.get(6)?;
    let evidence_refs: Vec<String> = serde_json::from_str(&evidence_raw).unwrap_or_default();
    Ok(SilentSessionCompletionEvent {
        schema: SILENT_SESSION_COMPLETION_EVENT_SCHEMA.to_string(),
        seq: row.get(0)?,
        session_id: row.get(1)?,
        run_id: row.get(2)?,
        generation: row.get(3)?,
        status: row.get(4)?,
        summary: row.get(5)?,
        evidence_refs,
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn event(session: &str, status: &str) -> SilentSessionCompletionEvent {
        SilentSessionCompletionEvent {
            schema: SILENT_SESSION_COMPLETION_EVENT_SCHEMA.to_string(),
            seq: 0,
            session_id: session.to_string(),
            run_id: Some("run-1".to_string()),
            generation: Some(1),
            status: status.to_string(),
            summary: "done".to_string(),
            evidence_refs: vec!["ev-1".to_string()],
            created_at: "2026-08-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn record_is_idempotent_and_backfill_works() {
        let conn = conn();
        assert!(record_completion_event(&conn, &event("s1", "completed")).unwrap());
        assert!(!record_completion_event(&conn, &event("s1", "completed")).unwrap());
        let latest = latest_completion(&conn, "s1").unwrap().unwrap();
        assert_eq!(latest.status, "completed");
        assert_eq!(latest.seq, 1);
        let backfill = recent_completions(&conn, 0, 10).unwrap();
        assert_eq!(backfill.len(), 1);
        let newer = recent_completions(&conn, latest.seq, 10).unwrap();
        assert!(newer.is_empty());
    }

    #[test]
    fn terminal_lifecycle_detection() {
        assert!(is_terminal_lifecycle("completed"));
        assert!(is_terminal_lifecycle("failed"));
        assert!(is_terminal_lifecycle("cancelled"));
        assert!(!is_terminal_lifecycle("running"));
        assert!(!is_terminal_lifecycle("waiting_input"));
    }
}
