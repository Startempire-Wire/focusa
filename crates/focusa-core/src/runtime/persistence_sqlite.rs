//! Persistence — SQLite canonical store.
//!
//! Canonical persistence:
//! - append-only events table
//! - versioned state snapshots
//!
//! ECS objects remain filesystem-backed (see reference::store).

use crate::clt::retain_hot_window;
use crate::sync::{CrdtEvent, VectorClock};
use crate::types::{EventLogEntry, FocusaConfig, FocusaState, SessionId};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::debug;
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 2;

fn hot_clt_snapshot_max_nodes() -> usize {
    std::env::var("FOCUSA_HOT_CLT_MAX_NODES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(10_000)
}

fn trim_hot_clt_snapshot(state: &mut FocusaState) -> usize {
    retain_hot_window(&mut state.clt, hot_clt_snapshot_max_nodes())
}

/// SQLite-backed persistence.
///
/// NOTE: Focusa daemon is single-writer, but API reads may happen concurrently.
/// We keep a single Connection behind a Mutex for now (simple + correct).
#[derive(Clone)]
pub struct SqlitePersistence {
    pub data_dir: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
pub struct RawEventLogRow {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub payload_json: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn event_chain_hash(
    previous_hash: &str,
    event_id: &str,
    timestamp: &str,
    payload_sha256: &str,
) -> String {
    sha256_hex(format!("{previous_hash}\n{event_id}\n{timestamp}\n{payload_sha256}").as_bytes())
}

fn latest_event_hash_checkpoint(conn: &Connection) -> anyhow::Result<Option<(i64, String)>> {
    conn.query_row(
        r#"
        SELECT chain_index, event_hash
        FROM event_hash_chain
        ORDER BY chain_index DESC
        LIMIT 1
        "#,
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(Into::into)
}

impl SqlitePersistence {
    pub fn new(config: &FocusaConfig) -> anyhow::Result<Self> {
        let data_dir = shellexpand(config.data_dir.as_str());
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(data_dir.join("ecs/objects"))?;
        std::fs::create_dir_all(data_dir.join("ecs/handles"))?;

        let db_path = data_dir.join("focusa.sqlite");
        let conn = Connection::open(db_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
        conn.pragma_update(None, "journal_size_limit", 67_108_864i64)?;
        conn.pragma_update(None, "cache_size", -16_384i64)?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA optimize;");

        let this = Self {
            data_dir,
            conn: Arc::new(Mutex::new(conn)),
        };

        this.init_schema()?;
        Ok(this)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS meta (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
              event_id TEXT PRIMARY KEY,
              ts TEXT NOT NULL,
              origin TEXT NOT NULL,
              correlation_id TEXT,
              payload_json TEXT NOT NULL,

              machine_id TEXT,
              instance_id TEXT,
              session_id TEXT,
              thread_id TEXT,
              is_observation INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
            CREATE INDEX IF NOT EXISTS idx_events_machine ON events(machine_id);
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_thread ON events(thread_id);

            CREATE TABLE IF NOT EXISTS event_hash_chain (
              event_id TEXT PRIMARY KEY,
              chain_index INTEGER NOT NULL,
              previous_hash TEXT NOT NULL,
              payload_sha256 TEXT NOT NULL,
              event_hash TEXT NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY(event_id) REFERENCES events(event_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_event_hash_chain_index ON event_hash_chain(chain_index);

            CREATE TABLE IF NOT EXISTS crdt_events (
              event_id TEXT PRIMARY KEY,
              project_root_key TEXT NOT NULL,
              workstream_key TEXT NOT NULL,
              machine_id TEXT NOT NULL,
              lamport_ts INTEGER NOT NULL,
              vector_clock_json TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              imported_from_peer_id TEXT,
              imported_at TEXT NOT NULL,
              FOREIGN KEY(event_id) REFERENCES events(event_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_crdt_events_scope ON crdt_events(project_root_key, workstream_key, lamport_ts, event_id);
            CREATE INDEX IF NOT EXISTS idx_crdt_events_machine ON crdt_events(machine_id);

            CREATE TABLE IF NOT EXISTS peers (
                peer_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                auth_token TEXT,
                created_at TEXT NOT NULL,
                last_seen_at TEXT,
                status TEXT NOT NULL DEFAULT 'pending'
            );

            CREATE TABLE IF NOT EXISTS sync_cursors (
                peer_id TEXT PRIMARY KEY,
                last_event_id TEXT,
                last_event_ts TEXT,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (peer_id) REFERENCES peers(peer_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS snapshots (
              name TEXT PRIMARY KEY,
              version INTEGER NOT NULL,
              ts TEXT NOT NULL,
              state_json TEXT NOT NULL
            );
            "#,
        )?;

        let existing: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match existing {
            None => {
                conn.execute(
                    "INSERT INTO meta(key, value) VALUES ('schema_version', ?1)",
                    [SCHEMA_VERSION.to_string()],
                )?;
            }
            Some(v) => {
                let parsed: i64 = v.parse().unwrap_or(0);
                if parsed > SCHEMA_VERSION || parsed <= 0 {
                    anyhow::bail!(
                        "unsupported schema_version {} (expected <= {})",
                        parsed,
                        SCHEMA_VERSION
                    );
                }
                if parsed < SCHEMA_VERSION {
                    conn.execute(
                        "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                        [SCHEMA_VERSION.to_string()],
                    )?;
                }
            }
        }

        // Ensure machine_id exists.
        let machine_id: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'machine_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if machine_id.is_none() {
            let id = Uuid::now_v7().to_string();
            conn.execute(
                "INSERT INTO meta(key, value) VALUES ('machine_id', ?1)",
                [id.clone()],
            )?;
            debug!("created machine_id in sqlite meta: {}", id);
        } else {
            debug!("machine_id already present in sqlite meta");
        }

        Ok(())
    }

    // ─── Peer Registry ─────────────────────────────────────────────────────

    pub fn add_peer(
        &self,
        peer_id: &str,
        name: &str,
        endpoint: &str,
        auth_token: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            r#"
            INSERT INTO peers(peer_id, name, endpoint, auth_token, created_at, status)
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending')
            ON CONFLICT(peer_id) DO UPDATE SET
                name=excluded.name,
                endpoint=excluded.endpoint,
                auth_token=excluded.auth_token
            "#,
            params![peer_id, name, endpoint, auth_token, Utc::now().to_rfc3339(),],
        )?;
        Ok(())
    }

    pub fn remove_peer(&self, peer_id: &str) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute("DELETE FROM peers WHERE peer_id = ?1", params![peer_id])?;
        Ok(())
    }

    pub fn list_peers(&self) -> anyhow::Result<Vec<PeerRecord>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            r#"
            SELECT peer_id, name, endpoint, auth_token, created_at, last_seen_at, status
            FROM peers
            ORDER BY name
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PeerRecord {
                peer_id: row.get(0)?,
                name: row.get(1)?,
                endpoint: row.get(2)?,
                auth_token: row.get(3)?,
                created_at: row.get(4)?,
                last_seen_at: row.get(5)?,
                status: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn update_peer_status(&self, peer_id: &str, status: &str) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            "UPDATE peers SET status = ?2, last_seen_at = ?3 WHERE peer_id = ?1",
            params![peer_id, status, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ─── Sync Cursors ──────────────────────────────────────────────────────

    pub fn get_cursor(&self, peer_id: &str) -> anyhow::Result<Option<SyncCursor>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let row: Option<(Option<String>, Option<String>, String)> = conn
            .query_row(
                "SELECT last_event_id, last_event_ts, updated_at FROM sync_cursors WHERE peer_id = ?1",
                params![peer_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        Ok(row.map(|(id, ts, updated)| SyncCursor {
            peer_id: peer_id.to_string(),
            last_event_id: id,
            last_event_ts: ts,
            updated_at: updated,
        }))
    }

    pub fn set_cursor(
        &self,
        peer_id: &str,
        last_event_id: Option<&str>,
        last_event_ts: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            r#"
            INSERT INTO sync_cursors(peer_id, last_event_id, last_event_ts, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(peer_id) DO UPDATE SET
                last_event_id=excluded.last_event_id,
                last_event_ts=excluded.last_event_ts,
                updated_at=excluded.updated_at
            "#,
            params![
                peer_id,
                last_event_id,
                last_event_ts,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    // ─── Events for Sync ───────────────────────────────────────────────────

    pub fn event_exists(&self, event_id: &str) -> anyhow::Result<bool> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM events WHERE event_id = ?1",
            params![event_id],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    /// Idempotency helper: has a turn_completed event already been persisted for turn_id?
    pub fn turn_completed_exists(&self, turn_id: &str) -> anyhow::Result<bool> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Note: FocusaEvent uses #[serde(tag = "type")] without rename_all,
        // so variant names serialize as PascalCase (TurnCompleted, not turn_completed).
        // Preferred path: JSON extraction (SQLite JSON1).
        let json_query = conn.query_row(
            "SELECT COUNT(*) FROM events WHERE json_extract(payload_json, '$.type') = 'TurnCompleted' AND json_extract(payload_json, '$.turn_id') = ?1",
            params![turn_id],
            |r| r.get::<_, i64>(0),
        );

        match json_query {
            Ok(count) => Ok(count > 0),
            Err(_) => {
                // Fallback for environments lacking JSON1 extraction.
                let needle = format!("\"turn_id\":\"{}\"", turn_id.replace('"', "\\\""));
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM events WHERE payload_json LIKE '%\"type\":\"TurnCompleted\"%' AND payload_json LIKE ?1",
                    params![format!("%{}%", needle)],
                    |r| r.get(0),
                )?;
                Ok(count > 0)
            }
        }
    }

    pub fn events_since(
        &self,
        since_ts: Option<&str>,
        since_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<EventLogEntry>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            r#"
            SELECT event_id, ts, origin, correlation_id, payload_json,
                   machine_id, instance_id, session_id, thread_id, is_observation
            FROM events
            WHERE (?1 IS NULL OR ts > ?1 OR (ts = ?1 AND event_id > ?2))
            ORDER BY ts, event_id
            LIMIT ?3
            "#,
        )?;
        let rows = stmt.query_map(params![since_ts, since_id, limit as i64], |r| {
            let event_id: String = r.get(0)?;
            let payload: String = r.get(4)?;

            let mut entry: EventLogEntry = match serde_json::from_str(&payload) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!(
                        event_id = %event_id,
                        payload_len = payload.len(),
                        error = %e,
                        "Corrupted event payload in database"
                    );
                    return Err(rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ));
                }
            };

            // Override stored columns with authoritative DB values
            entry.id = event_id.parse().map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "event_id".into(),
                    rusqlite::types::Type::Text,
                )
            })?;
            entry.timestamp = DateTime::parse_from_rfc3339(&r.get::<_, String>(1)?)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(1, "ts".into(), rusqlite::types::Type::Text)
                })?
                .with_timezone(&Utc);
            entry.machine_id = r.get(5)?;
            entry.instance_id = r.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok());
            entry.session_id = r.get::<_, Option<String>>(7)?.and_then(|s| s.parse().ok());
            entry.thread_id = r.get::<_, Option<String>>(8)?.and_then(|s| s.parse().ok());
            entry.is_observation = r.get::<_, i32>(9)? != 0;
            Ok(entry)
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn events_since_raw(
        &self,
        since_ts: Option<&str>,
        since_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawEventLogRow>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            r#"
            SELECT event_id, ts, session_id, payload_json
            FROM events
            WHERE (?1 IS NULL OR ts > ?1 OR (ts = ?1 AND event_id > ?2))
            ORDER BY ts, event_id
            LIMIT ?3
            "#,
        )?;
        let rows = stmt.query_map(params![since_ts, since_id, limit as i64], |r| {
            let event_id: String = r.get(0)?;
            let timestamp = DateTime::parse_from_rfc3339(&r.get::<_, String>(1)?)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(1, "ts".into(), rusqlite::types::Type::Text)
                })?
                .with_timezone(&Utc);
            let session_id = r
                .get::<_, Option<String>>(2)?
                .and_then(|sid| sid.parse().ok());
            let payload_json: String = r.get(3)?;
            Ok(RawEventLogRow {
                event_id,
                timestamp,
                session_id,
                payload_json,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    fn scope_keys_for_event(entry: &EventLogEntry) -> (String, String) {
        let project_root_key = entry
            .correlation_id
            .as_deref()
            .and_then(|value| {
                value
                    .split('|')
                    .find_map(|part| part.strip_prefix("project_root="))
            })
            .unwrap_or("unscoped_project_root")
            .to_string();
        let workstream_key = entry
            .correlation_id
            .as_deref()
            .and_then(|value| {
                value
                    .split('|')
                    .find_map(|part| part.strip_prefix("continuity_id="))
            })
            .unwrap_or("default_workstream")
            .to_string();
        (project_root_key, workstream_key)
    }

    pub fn append_crdt_event(
        &self,
        event: &CrdtEvent,
        imported_from_peer_id: Option<&str>,
    ) -> anyhow::Result<bool> {
        let entry = &event.entry;
        let event_id = entry.id.to_string();
        let payload_json = serde_json::to_string(entry)?;
        let vector_clock_json = serde_json::to_string(&event.vector_clock)?;
        let machine_id = entry.machine_id.clone().unwrap_or_else(|| {
            event
                .vector_clock
                .clocks
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        });
        let (project_root_key, workstream_key) = Self::scope_keys_for_event(entry);
        self.append_event(entry)?;
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let changed = conn.execute(
            r#"
            INSERT OR IGNORE INTO crdt_events(
              event_id, project_root_key, workstream_key, machine_id, lamport_ts,
              vector_clock_json, payload_json, imported_from_peer_id, imported_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                event_id.as_str(),
                project_root_key,
                workstream_key,
                machine_id,
                event.lamport_ts as i64,
                vector_clock_json,
                payload_json,
                imported_from_peer_id,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(changed > 0)
    }

    pub fn crdt_events_for_scope(
        &self,
        project_root_key: &str,
        workstream_key: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<CrdtEvent>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            r#"
            SELECT payload_json, vector_clock_json, lamport_ts
            FROM crdt_events
            WHERE project_root_key = ?1 AND workstream_key = ?2
            ORDER BY lamport_ts, event_id
            LIMIT ?3
            "#,
        )?;
        let rows = stmt.query_map(
            params![project_root_key, workstream_key, limit as i64],
            |row| {
                let payload: String = row.get(0)?;
                let clock_json: String = row.get(1)?;
                let entry: EventLogEntry = serde_json::from_str(&payload).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })?;
                let vector_clock: VectorClock =
                    serde_json::from_str(&clock_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                let lamport_ts: i64 = row.get(2)?;
                Ok(CrdtEvent {
                    entry,
                    vector_clock,
                    lamport_ts: lamport_ts.max(0) as u64,
                })
            },
        )?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn import_crdt_events_same_root(
        &self,
        peer_id: &str,
        project_root_key: &str,
        workstream_key: &str,
        events: &[CrdtEvent],
    ) -> anyhow::Result<usize> {
        self.add_peer(peer_id, peer_id, "same-root://local", None)?;
        self.update_peer_status(peer_id, "active")?;
        let mut imported = 0usize;
        for event in events {
            let (event_project, event_workstream) = Self::scope_keys_for_event(&event.entry);
            if event_project != project_root_key || event_workstream != workstream_key {
                continue;
            }
            if self.append_crdt_event(event, Some(peer_id))? {
                imported += 1;
            }
        }
        if let Some(last) = events.last() {
            self.set_cursor(
                peer_id,
                Some(&last.entry.id.to_string()),
                Some(&last.entry.timestamp.to_rfc3339()),
            )?;
        }
        Ok(imported)
    }

    pub fn save_state(&self, state: &FocusaState) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let ts = Utc::now();
        let mut snapshot_state = state.clone();
        let trimmed = trim_hot_clt_snapshot(&mut snapshot_state);
        if trimmed > 0 {
            debug!(
                trimmed,
                remaining = snapshot_state.clt.nodes.len(),
                "trimmed hot CLT snapshot before SQLite save"
            );
        }
        let state_json = serde_json::to_string(&snapshot_state)?;
        conn.execute(
            r#"
            INSERT INTO snapshots(name, version, ts, state_json)
            VALUES('focusa', ?1, ?2, ?3)
            ON CONFLICT(name) DO UPDATE SET
              version=excluded.version,
              ts=excluded.ts,
              state_json=excluded.state_json
            "#,
            params![state.version as i64, ts.to_rfc3339(), state_json],
        )?;
        Ok(())
    }

    pub fn load_state(&self) -> anyhow::Result<Option<FocusaState>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let row: Option<String> = conn
            .query_row(
                "SELECT state_json FROM snapshots WHERE name='focusa'",
                [],
                |r| r.get(0),
            )
            .optional()?;

        match row {
            None => Ok(None),
            Some(json) => match serde_json::from_str::<FocusaState>(&json) {
                Ok(mut s) => {
                    let trimmed = trim_hot_clt_snapshot(&mut s);
                    if trimmed > 0 {
                        debug!(
                            trimmed,
                            remaining = s.clt.nodes.len(),
                            "trimmed hot CLT snapshot after SQLite load"
                        );
                    }
                    Ok(Some(s))
                }
                Err(_) => {
                    // Backward compatibility: older snapshots won't have newer fields.
                    // Fall back to a fresh state rather than failing daemon startup.
                    Ok(None)
                }
            },
        }
    }

    pub fn machine_id(&self) -> anyhow::Result<String> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let id: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'machine_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        id.ok_or_else(|| anyhow::anyhow!("machine_id missing from meta"))
    }

    /// Latest persisted event timestamp (RFC3339), if any.
    pub fn latest_event_timestamp(&self) -> anyhow::Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let ts: Option<String> = conn
            .query_row("SELECT MAX(ts) FROM events", [], |row| row.get(0))
            .optional()?
            .flatten();
        Ok(ts)
    }

    /// Get the N most recent events as JSON values.
    pub fn recent_events(&self, limit: usize) -> anyhow::Result<Vec<serde_json::Value>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt =
            conn.prepare("SELECT payload_json FROM events ORDER BY ts DESC, rowid DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit as i64], |row| {
            let raw: String = row.get(0)?;
            Ok(raw)
        })?;
        let mut events = Vec::new();
        for row in rows.flatten() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&row) {
                events.push(val);
            }
        }
        Ok(events)
    }

    /// Current count of persisted events.
    pub fn event_count(&self) -> anyhow::Result<u64> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    pub fn append_event(&self, entry: &EventLogEntry) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(entry)?;
        let event_id = entry.id.to_string();
        let timestamp = entry.timestamp.to_rfc3339();
        let payload_sha256 = sha256_hex(payload_json.as_bytes());

        // Avoid re-locking the same mutex (machine_id() also locks conn).
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let machine_id = entry
            .machine_id
            .clone()
            .or_else(|| {
                conn.query_row(
                    "SELECT value FROM meta WHERE key = 'machine_id'",
                    [],
                    |row| row.get(0),
                )
                .optional()
                .ok()
                .flatten()
            })
            .unwrap_or_else(|| "unknown".to_string());

        conn.execute(
            r#"
            INSERT OR IGNORE INTO events(
              event_id, ts, origin, correlation_id, payload_json,
              machine_id, instance_id, session_id, thread_id, is_observation
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                event_id.as_str(),
                timestamp.as_str(),
                format!("{:?}", entry.origin),
                entry.correlation_id.clone(),
                payload_json.as_str(),
                machine_id,
                entry.instance_id.map(|v| v.to_string()),
                entry.session_id.map(|v| v.to_string()),
                entry.thread_id.map(|v| v.to_string()),
                entry.is_observation as i32,
            ],
        )?;

        let (chain_index, previous_hash) = latest_event_hash_checkpoint(&conn)?
            .map(|(index, hash)| (index + 1, hash))
            .unwrap_or_else(|| (0, "GENESIS".to_string()));
        let event_hash = event_chain_hash(&previous_hash, &event_id, &timestamp, &payload_sha256);
        conn.execute(
            r#"
            INSERT OR IGNORE INTO event_hash_chain(
              event_id, chain_index, previous_hash, payload_sha256, event_hash, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                event_id.as_str(),
                chain_index,
                previous_hash.as_str(),
                payload_sha256.as_str(),
                event_hash.as_str(),
                timestamp.as_str(),
            ],
        )?;

        Ok(())
    }
    /// Ensure confidence calibration table exists.
    pub fn ensure_calibration_table(&self) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS confidence_calibration (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prediction_type TEXT NOT NULL,
                predicted_confidence REAL NOT NULL,
                context TEXT,
                outcome TEXT,
                outcome_correct INTEGER,
                created_at TEXT NOT NULL,
                resolved_at TEXT
            )"#,
            [],
        )?;
        Ok(())
    }

    /// Log a confidence prediction for later calibration.
    pub fn log_confidence(
        &self,
        prediction_type: &str,
        confidence: f64,
        context: &str,
    ) -> anyhow::Result<i64> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            "INSERT INTO confidence_calibration (prediction_type, predicted_confidence, context, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![prediction_type, confidence, context, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Record the outcome for a prediction.
    pub fn resolve_confidence(&self, id: i64, outcome: &str, correct: bool) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            "UPDATE confidence_calibration SET outcome=?1, outcome_correct=?2, resolved_at=?3 WHERE id=?4",
            rusqlite::params![outcome, if correct { 1i64 } else { 0i64 }, chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Get calibration stats: for each confidence bucket, what % were correct?
    pub fn calibration_stats(&self) -> anyhow::Result<Vec<(String, f64, f64, u64)>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            r#"SELECT 
                CASE 
                    WHEN predicted_confidence < 0.3 THEN 'low'
                    WHEN predicted_confidence < 0.7 THEN 'medium'
                    ELSE 'high'
                END as bucket,
                AVG(predicted_confidence) as avg_predicted,
                AVG(CAST(outcome_correct AS REAL)) as actual_rate,
                COUNT(*) as total
            FROM confidence_calibration 
            WHERE outcome_correct IS NOT NULL
            GROUP BY bucket"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, i64>(3)? as u64,
            ))
        })?;
        let mut stats = Vec::new();
        for row in rows.flatten() {
            stats.push(row);
        }
        Ok(stats)
    }
}

#[derive(Debug, Clone)]
pub struct PeerRecord {
    pub peer_id: String,
    pub name: String,
    pub endpoint: String,
    pub auth_token: Option<String>,
    pub created_at: String,
    pub last_seen_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct SyncCursor {
    pub peer_id: String,
    pub last_event_id: Option<String>,
    pub last_event_ts: Option<String>,
    pub updated_at: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HLT LEDGER — Spec98/99: scope-bounded, no singleton, CRDT-grade events
// ═══════════════════════════════════════════════════════════════════════════════

use crate::types::HltLedgerEntry;

/// Compute the HLT ledger directory for a given project_root.
/// Returns: {hlt_ledger_dir}/{project_root_hash}/
fn hlt_ledger_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    // Hash project_root to avoid path issues and ensure scope isolation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    data_dir.join(format!("hlt-ledger/{}", hash))
}

impl SqlitePersistence {
    /// Append an HLT ledger entry to the scope-bounded JSONL file.
    /// Per Spec98/99: no singleton, scope-bounded by project_root.
    pub fn append_hlt_ledger_entry(&self, entry: &HltLedgerEntry) -> anyhow::Result<()> {
        let ledger_dir = hlt_ledger_dir_for_project(&self.data_dir, &entry.project_root);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("hlt.jsonl");
        let line = serde_json::to_string(entry)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        writeln!(file, "{}", line)?;
        debug!("Appended HLT ledger entry to {:?}", ledger_file);
        Ok(())
    }

    /// Read HLT ledger entries for a project, scoped by continuity_id if provided.
    /// Returns entries in chronological order (oldest first), most recent last.
    pub fn read_hlt_ledger_entries(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<HltLedgerEntry>> {
        let ledger_dir = hlt_ledger_dir_for_project(&self.data_dir, project_root);
        let ledger_file = ledger_dir.join("hlt.jsonl");
        if !ledger_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&ledger_file)?;
        let entries: Vec<HltLedgerEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .filter(|entry: &HltLedgerEntry| match continuity_id {
                Some(cid) => entry.continuity_id.as_deref() == Some(cid),
                None => true,
            })
            .collect();
        // Return most recent `limit` entries
        let start = entries.len().saturating_sub(limit);
        Ok(entries[start..].to_vec())
    }

    /// Get the latest HLT entry for a project (most recent HLT value).
    pub fn latest_hlt_for_project(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
    ) -> anyhow::Result<Option<HltLedgerEntry>> {
        let entries = self.read_hlt_ledger_entries(project_root, continuity_id, 1)?;
        Ok(entries.into_iter().last())
    }

    /// Get the HLT ledger file path for a project (for API exposure).
    pub fn hlt_ledger_path_for_project(&self, project_root: &str) -> PathBuf {
        hlt_ledger_dir_for_project(&self.data_dir, project_root).join("hlt.jsonl")
    }
}

fn shellexpand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

#[allow(dead_code)]
fn _parse_ts(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[allow(dead_code)]
fn _exists(path: &Path) -> bool {
    path.exists()
}
