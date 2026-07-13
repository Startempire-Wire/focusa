//! Spec 101 §5.12.11 — Recent-turns daemon routes
//!
//! Canonical ring buffer lives here; agents (Pi, Claude Code, Aider, Cursor, ...)
//! are thin adapters that POST on turn_end and GET on injection. Cross-agent
//! consistency comes from a single source of truth.
//!
//! Routes:
//!   GET  /v1/turns/recent?n=4&continuity_id=...  → RecentTurnsResponse
//!   POST /v1/turns/recent                        → AppendTurnRequest  (idempotent)
//!   POST /v1/events/recall-trigger               → telemetry ack
//!
//! Storage: SQLite table `recent_turns` in `focusa.sqlite`,
//!   keyed by (continuity_id, turn_id), bounded by retention window.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

pub const RECENT_TURNS_SCHEMA: &str = "focusa.recent_turns.v1";
pub const DEFAULT_N: usize = 4;
pub const MAX_N: usize = 8;
const RETENTION_SECS_DEFAULT: i64 = 7 * 86_400; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppendTurnRequest {
    pub turn_id: String,
    pub continuity_id: String,
    pub mission_at_turn: String,
    pub outcome: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub tool_call_count: u32,
    #[serde(default)]
    pub emitted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentTurnSlice {
    pub turn_id: String,
    pub continuity_id: String,
    pub mission_at_turn: String,
    pub outcome: String,
    pub evidence_refs: Vec<String>,
    pub tool_call_count: u32,
    pub emitted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTurnsResponse {
    pub schema: String,
    pub count: usize,
    pub turns: Vec<RecentTurnSlice>,
    pub fetched_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub n: Option<usize>,
    #[serde(default)]
    pub continuity_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecallTriggerRequest {
    #[serde(default)]
    pub matched_category: String,
    #[serde(default)]
    pub matched_phrase: String,
    #[serde(default)]
    pub slice_size: usize,
    #[serde(default)]
    pub ring_size: usize,
    #[serde(default)]
    pub forced_re_emit: bool,
    #[serde(default)]
    pub alternative_tools_surfaced: Vec<String>,
    #[serde(default)]
    pub continuity_id: String,
    #[serde(default)]
    pub agent_kind: String,
}

fn recent_db_path(data_dir: &str) -> std::path::PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home)
            .join(rest)
            .join("focusa.sqlite");
    }
    std::path::PathBuf::from(data_dir).join("focusa.sqlite")
}

fn ensure_recent_turns_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS recent_turns (
            continuity_id  TEXT NOT NULL,
            turn_id        TEXT NOT NULL,
            mission        TEXT NOT NULL,
            outcome        TEXT NOT NULL,
            evidence_refs  TEXT NOT NULL,
            tool_calls     INTEGER NOT NULL,
            emitted_at     INTEGER NOT NULL,
            inserted_at    INTEGER NOT NULL,
            PRIMARY KEY (continuity_id, turn_id)
        );
        CREATE INDEX IF NOT EXISTS recent_turns_lookup
            ON recent_turns(continuity_id, emitted_at DESC)",
    )
}

fn unix_now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn append_recent_turn(conn: &Connection, req: &AppendTurnRequest) -> rusqlite::Result<bool> {
    let refs_json = serde_json::to_string(&req.evidence_refs).unwrap_or_else(|_| "[]".to_string());
    let now = unix_now_secs() as i64;
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO recent_turns (
            continuity_id, turn_id, mission, outcome,
            evidence_refs, tool_calls, emitted_at, inserted_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            req.continuity_id,
            req.turn_id,
            req.mission_at_turn,
            req.outcome,
            refs_json,
            req.tool_call_count as i64,
            req.emitted_at as i64,
            now,
        ],
    )?;
    // Retention prune (per-continuity, opportunistic)
    let _ = conn.execute(
        "DELETE FROM recent_turns
         WHERE continuity_id = ?1
           AND inserted_at < ?2",
        params![req.continuity_id, now - RETENTION_SECS_DEFAULT],
    );
    Ok(inserted > 0)
}

fn list_recent_turns(
    conn: &Connection,
    continuity_id: &str,
    n: usize,
) -> rusqlite::Result<Vec<RecentTurnSlice>> {
    let cap = n.clamp(1, MAX_N) as i64;
    let mut stmt = conn.prepare(
        "SELECT turn_id, continuity_id, mission, outcome, evidence_refs, tool_calls, emitted_at
         FROM recent_turns
         WHERE continuity_id = ?1
         ORDER BY emitted_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![continuity_id, cap], |row| {
            let refs_json: String = row.get(4)?;
            let refs: Vec<String> = serde_json::from_str(&refs_json).unwrap_or_default();
            Ok(RecentTurnSlice {
                turn_id: row.get(0)?,
                continuity_id: row.get(1)?,
                mission_at_turn: row.get(2)?,
                outcome: row.get(3)?,
                evidence_refs: refs,
                tool_call_count: row.get::<_, i64>(5)? as u32,
                emitted_at: row.get::<_, i64>(6)? as u64,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Bounded internal reader used by Spec 130 packet construction. Failure is
/// advisory and returns an empty slice; canonical recent-turn storage remains
/// unchanged.
pub(crate) fn read_recent_turns_bounded(
    data_dir: &str,
    continuity_id: &str,
    n: usize,
) -> Vec<RecentTurnSlice> {
    if continuity_id.trim().is_empty() {
        return vec![];
    }
    let Ok(conn) = Connection::open(recent_db_path(data_dir)) else {
        return vec![];
    };
    if ensure_recent_turns_table(&conn).is_err() {
        return vec![];
    }
    list_recent_turns(&conn, continuity_id, n).unwrap_or_default()
}

// ─── Handlers ─────────────────────────────────────────────────────────────

async fn list_recent_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let db_path = recent_db_path(&state.config.data_dir);
    let continuity = q.continuity_id.clone().unwrap_or_default();
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": "db_open_failed", "why": e.to_string()})),
    };
    if let Err(e) = ensure_recent_turns_table(&conn) {
        return Json(json!({"error": "schema_init_failed", "why": e.to_string()}));
    }
    let n = q.n.unwrap_or(DEFAULT_N);
    if continuity.is_empty() {
        return Json(json!({
            "error": "continuity_id_required",
            "schema": RECENT_TURNS_SCHEMA,
        }));
    }
    match list_recent_turns(&conn, &continuity, n) {
        Ok(rows) => Json(
            serde_json::to_value(RecentTurnsResponse {
                schema: RECENT_TURNS_SCHEMA.to_string(),
                count: rows.len(),
                turns: rows,
                fetched_at: unix_now_secs(),
            })
            .unwrap_or_else(|_| json!({"error": "encode_failed"})),
        ),
        Err(e) => Json(json!({"error": "read_failed", "why": e.to_string()})),
    }
}

async fn append_recent_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let req: AppendTurnRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": "invalid_body", "why": e.to_string()})),
    };
    if req.turn_id.is_empty() || req.continuity_id.is_empty() {
        return Json(json!({"error": "turn_id_and_continuity_id_required"}));
    }
    let db_path = recent_db_path(&state.config.data_dir);
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": "db_open_failed", "why": e.to_string()})),
    };
    if let Err(e) = ensure_recent_turns_table(&conn) {
        return Json(json!({"error": "schema_init_failed", "why": e.to_string()}));
    }
    match append_recent_turn(&conn, &req) {
        Ok(inserted) => Json(json!({
            "schema": RECENT_TURNS_SCHEMA,
            "ok": true,
            "turn_id": req.turn_id,
            "continuity_id": req.continuity_id,
            "inserted": inserted,
        })),
        Err(e) => Json(json!({"error": "write_failed", "why": e.to_string()})),
    }
}

async fn recall_trigger_handler(Json(req): Json<RecallTriggerRequest>) -> Json<Value> {
    // Telemetry-only ack; canonical ring buffer is updated by append_recent_turn.
    Json(json!({
        "schema": "focusa.recall_trigger.v1",
        "ok": true,
        "matched_category": req.matched_category,
        "matched_phrase": req.matched_phrase,
        "slice_size": req.slice_size,
        "ring_size": req.ring_size,
        "forced_re_emit": req.forced_re_emit,
        "continuity_id": req.continuity_id,
        "agent_kind": req.agent_kind,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/turns/recent",
            get(list_recent_handler).post(append_recent_handler),
        )
        .route("/v1/events/recall-trigger", post(recall_trigger_handler))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_turns_schema_constant() {
        assert_eq!(RECENT_TURNS_SCHEMA, "focusa.recent_turns.v1");
        assert!(DEFAULT_N <= MAX_N);
        assert_eq!(MAX_N, 8);
    }

    #[test]
    fn append_request_roundtrip() {
        let r = AppendTurnRequest {
            turn_id: "t1".into(),
            continuity_id: "c1".into(),
            mission_at_turn: "set OVH topology".into(),
            outcome: "committed".into(),
            evidence_refs: vec!["ev:42".into()],
            tool_call_count: 3,
            emitted_at: 1_783_361_221,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["turn_id"], "t1");
        assert_eq!(v["tool_call_count"], 3);
        let back: AppendTurnRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn list_query_n_defaulted() {
        let q: ListQuery = serde_json::from_value(json!({})).unwrap();
        assert!(q.n.is_none());
        assert!(q.continuity_id.is_none());
    }

    #[test]
    fn response_count_matches_turns_len() {
        let r = RecentTurnsResponse {
            schema: RECENT_TURNS_SCHEMA.into(),
            count: 2,
            turns: vec![
                RecentTurnSlice {
                    turn_id: "t2".into(),
                    continuity_id: "c1".into(),
                    mission_at_turn: "fix topology".into(),
                    outcome: "filed_bead".into(),
                    evidence_refs: vec![],
                    tool_call_count: 1,
                    emitted_at: 100,
                },
                RecentTurnSlice {
                    turn_id: "t1".into(),
                    continuity_id: "c1".into(),
                    mission_at_turn: "audit gaps".into(),
                    outcome: "tooled".into(),
                    evidence_refs: vec!["ev:42".into()],
                    tool_call_count: 5,
                    emitted_at: 99,
                },
            ],
            fetched_at: 200,
        };
        assert_eq!(r.count, r.turns.len());
        // newest first invariant
        assert!(r.turns[0].emitted_at >= r.turns[1].emitted_at);
    }

    #[test]
    fn retention_secs_default_is_seven_days() {
        assert_eq!(RETENTION_SECS_DEFAULT, 7 * 86_400);
    }
}
