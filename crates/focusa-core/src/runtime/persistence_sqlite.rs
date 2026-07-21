//! Persistence — SQLite canonical store.
//!
//! Canonical persistence:
//! - append-only events table
//! - versioned state snapshots
//!
//! ECS objects remain filesystem-backed (see reference::store).

use crate::clt::retain_hot_window;
use crate::sync::{CrdtEvent, VectorClock};
use crate::types::{
    CallStackDesign, CognitionOptimizerArtifact, CuratorEvalRun, DeviceRecord, EventLogEntry,
    FocusaConfig, FocusaState, SessionId,
};
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

/// Stable replay record joined to the append-only event hash-chain sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct DurableEventRecord {
    pub sequence: u64,
    pub event_id: String,
    pub timestamp: String,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
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
        crate::silent_sessions::migrate_silent_session_schema(
            &this,
            crate::silent_sessions::MigrationMode::Apply,
        )?;
        Ok(this)
    }

    /// Canonical SQLite path used by bounded derived indexes such as Context retrieval.
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("focusa.sqlite")
    }

    pub(crate) fn with_connection_mut<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let mut connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut connection)
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

            CREATE TABLE IF NOT EXISTS pairing_codes (
              code TEXT PRIMARY KEY,
              device_id TEXT NOT NULL,
              device_name TEXT,
              platform TEXT,
              scopes_json TEXT,
              daemon_base_url TEXT,
              created_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              consumed INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_pairing_codes_expires ON pairing_codes(expires_at);

            CREATE TABLE IF NOT EXISTS connect_sessions (
              connect_id TEXT PRIMARY KEY,
              device_id TEXT,
              mac_nonce TEXT,
              mac_pubkey TEXT,
              mac_callback TEXT,
              server_url TEXT NOT NULL,
              scopes_json TEXT,
              status TEXT NOT NULL DEFAULT 'waiting_for_mac',
              created_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              completed_at TEXT,
              room_claim_secret TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_connect_sessions_expires ON connect_sessions(expires_at);
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

            -- V2: device_tokens survives daemon restart. Each row is a
            -- Bearer-acceptable device token minted at pair completion.
            CREATE TABLE IF NOT EXISTS device_tokens (
                token TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                scopes_json TEXT,
                issued_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_used_at TEXT,
                issued_to TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_device_tokens_expires ON device_tokens(expires_at);
            CREATE INDEX IF NOT EXISTS idx_device_tokens_device ON device_tokens(device_id);
            -- V2 Invariant 6: at most one active (non-revoked, non-expired)
            -- token per (device_id, host) tuple. Re-pair revokes the prior
            -- active token server-side so the menubar can't end up with two
            -- live tokens for the same logical device. Tokens are revoked
            -- by deleting the row (see revoke_device_tokens_by_device +
            -- revoke_active_token_for_device_host), which keeps the
            -- UNIQUE partial index in sync.
            CREATE UNIQUE INDEX IF NOT EXISTS uniq_device_tokens_device_host_active
              ON device_tokens(device_id, issued_to)
              WHERE issued_to IS NOT NULL;

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

        // V2 P0 round 2: add room_claim_secret column to existing
        // connect_sessions tables that were created before the column
        // existed. SQLite has no IF NOT EXISTS for ALTER TABLE ADD
        // COLUMN, so probe via PRAGMA table_info first.
        let has_room_secret: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('connect_sessions') WHERE name='room_claim_secret'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if has_room_secret == 0 {
            conn.execute(
                "ALTER TABLE connect_sessions ADD COLUMN room_claim_secret TEXT",
                [],
            )?;
        }

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

    /// Read durable events strictly after a stable sequence cursor.
    pub fn durable_events_after(
        &self,
        sequence: u64,
        limit: usize,
    ) -> anyhow::Result<Vec<DurableEventRecord>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut statement = conn.prepare(
            r#"
            SELECT h.chain_index + 1, e.event_id, e.ts, e.payload_json, e.correlation_id
            FROM event_hash_chain h
            INNER JOIN events e ON e.event_id = h.event_id
            WHERE h.chain_index >= ?1
            ORDER BY h.chain_index ASC
            LIMIT ?2
            "#,
        )?;
        let rows = statement.query_map(
            params![sequence as i64, limit.clamp(1, 1_000) as i64],
            |row| {
                let payload_json: String = row.get(3)?;
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    payload_json,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (sequence, event_id, timestamp, payload_json, correlation_id) = row?;
            Ok(DurableEventRecord {
                sequence: u64::try_from(sequence)?,
                event_id,
                timestamp,
                correlation_id,
                payload: serde_json::from_str(&payload_json)?,
            })
        })
        .collect()
    }

    /// Resolve an SSE Last-Event-ID UUID to its durable sequence cursor.
    pub fn durable_event_sequence(&self, event_id: &str) -> anyhow::Result<Option<u64>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let sequence: Option<i64> = conn
            .query_row(
                "SELECT chain_index + 1 FROM event_hash_chain WHERE event_id = ?1",
                [event_id],
                |row| row.get(0),
            )
            .optional()?;
        sequence.map(u64::try_from).transpose().map_err(Into::into)
    }

    /// Latest durable sequence, or zero when the event ledger is empty.
    pub fn latest_durable_event_sequence(&self) -> anyhow::Result<u64> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let sequence: Option<i64> = conn.query_row(
            "SELECT MAX(chain_index) + 1 FROM event_hash_chain",
            [],
            |row| row.get(0),
        )?;
        sequence
            .map(u64::try_from)
            .transpose()
            .map(Option::unwrap_or_default)
            .map_err(Into::into)
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
/// V2: Full DeviceToken record rehydrated from SQLite on daemon restart.
/// Mirrors focusa_core::types::DeviceToken but persisted via the device_tokens
/// SQLite table (no JSONL audit) so reads are cheap.
pub struct PersistedDeviceToken {
    pub device_id: String,
    pub scopes: Vec<String>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub issued_to: String,
}

/// V2: SQL row shape for the device_tokens table, used by
/// list_device_tokens(). Aliased so callers don't have to spell out the
/// long tuple type.
pub type DeviceTokenRow = (
    String,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
);

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

fn call_stack_designs_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    data_dir.join(format!("call-stack-designs/{}", hash))
}

impl SqlitePersistence {
    /// Append a Call Stack Design to the scope-bounded JSONL file.
    /// Per Spec 103: no singleton, scope-bounded by `project_root`.
    pub fn append_call_stack_design(&self, design: &CallStackDesign) -> anyhow::Result<()> {
        let ledger_dir = call_stack_designs_dir_for_project(&self.data_dir, &design.project_root);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("designs.jsonl");
        let line = serde_json::to_string(design)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        writeln!(file, "{}", line)?;
        debug!("Appended Call Stack Design to {:?}", ledger_file);
        Ok(())
    }

    /// Read Call Stack Designs for a project, scoped by continuity_id and entry_name.
    pub fn read_call_stack_designs(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
        entry_name: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<CallStackDesign>> {
        let ledger_dir = call_stack_designs_dir_for_project(self.data_dir.as_path(), project_root);
        let ledger_file = ledger_dir.join("designs.jsonl");
        if !ledger_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&ledger_file)?;
        let mut entries: Vec<CallStackDesign> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .filter(|d: &CallStackDesign| match continuity_id {
                Some(cid) => d.continuity_id.as_deref() == Some(cid),
                None => true,
            })
            .filter(|d: &CallStackDesign| match entry_name {
                Some(name) => d.entry_name == name,
                None => true,
            })
            .collect();
        // Return most recent `limit` entries
        let start = entries.len().saturating_sub(limit);
        entries = entries[start..].to_vec();
        Ok(entries)
    }

    /// Get the Call Stack Designs ledger file path for a project (for API exposure).
    pub fn call_stack_designs_path_for_project(&self, project_root: &str) -> PathBuf {
        call_stack_designs_dir_for_project(self.data_dir.as_path(), project_root)
            .join("designs.jsonl")
    }
}

fn device_pairing_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    data_dir.join(format!("device-pairing/{}", hash))
}

impl SqlitePersistence {
    /// Append a DeviceRecord to the scope-bounded JSONL ledger.
    pub fn append_device_record(&self, record: &DeviceRecord) -> anyhow::Result<()> {
        let ledger_dir = device_pairing_dir_for_project(&self.data_dir, &record.host);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("devices.jsonl");
        let line = serde_json::to_string(record)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        writeln!(file, "{}", line)?;
        debug!("Appended DeviceRecord to {:?}", ledger_file);
        Ok(())
    }

    /// Read recent DeviceRecords for a project (most recent last).
    pub fn read_device_records(
        &self,
        host: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<DeviceRecord>> {
        let ledger_dir = device_pairing_dir_for_project(&self.data_dir, host);
        let ledger_file = ledger_dir.join("devices.jsonl");
        if !ledger_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&ledger_file)?;
        let mut entries: Vec<DeviceRecord> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let start = entries.len().saturating_sub(limit);
        entries = entries[start..].to_vec();
        Ok(entries)
    }

    /// Persist or replace a pending pairing code by code string.
    #[allow(clippy::too_many_arguments)]
    pub fn put_pairing_code(
        &self,
        code: &str,
        device_id: &str,
        device_name: Option<&str>,
        platform: Option<&str>,
        scopes_json: Option<&str>,
        daemon_base_url: Option<&str>,
        created_at: &str,
        expires_at: &str,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            r#"INSERT INTO pairing_codes
               (code, device_id, device_name, platform, scopes_json, daemon_base_url, created_at, expires_at, consumed)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)
               ON CONFLICT(code) DO UPDATE SET
                 device_id=excluded.device_id,
                 device_name=excluded.device_name,
                 platform=excluded.platform,
                 scopes_json=excluded.scopes_json,
                 daemon_base_url=excluded.daemon_base_url,
                 created_at=excluded.created_at,
                 expires_at=excluded.expires_at,
                 consumed=0"#,
            params![
                code, device_id, device_name, platform, scopes_json, daemon_base_url,
                created_at, expires_at,
            ],
        )?;
        Ok(())
    }

    /// Fetch a non-expired, non-consumed pairing code.
    pub fn get_pairing_code(&self, code: &str) -> anyhow::Result<Option<(String, String, String)>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            "SELECT device_id, expires_at, scopes_json FROM pairing_codes
             WHERE code = ?1 AND consumed = 0 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![code, now])?;
        if let Some(row) = rows.next()? {
            let device_id: String = row.get(0)?;
            let expires_at: String = row.get(1)?;
            let scopes: String = row.get(2).unwrap_or_else(|_| "[]".to_string());
            return Ok(Some((device_id, expires_at, scopes)));
        }
        Ok(None)
    }

    /// Mark a pairing code as consumed (idempotent).
    pub fn consume_pairing_code(&self, code: &str) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            "UPDATE pairing_codes SET consumed = 1 WHERE code = ?1",
            params![code],
        )?;
        Ok(())
    }

    /// Persist a connect session (rendezvous).
    #[allow(clippy::too_many_arguments)]
    pub fn put_connect_session(
        &self,
        connect_id: &str,
        device_id: Option<&str>,
        mac_nonce: Option<&str>,
        mac_pubkey: Option<&str>,
        mac_callback: Option<&str>,
        server_url: &str,
        scopes_json: Option<&str>,
        created_at: &str,
        expires_at: &str,
        room_claim_secret: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute(
            r#"INSERT INTO connect_sessions
               (connect_id, device_id, mac_nonce, mac_pubkey, mac_callback, server_url,
                scopes_json, status, created_at, expires_at, completed_at, room_claim_secret)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'waiting_for_mac', ?8, ?9, NULL, ?10)
               ON CONFLICT(connect_id) DO UPDATE SET
                 device_id=COALESCE(excluded.device_id, connect_sessions.device_id),
                 mac_nonce=COALESCE(excluded.mac_nonce, connect_sessions.mac_nonce),
                 mac_pubkey=COALESCE(excluded.mac_pubkey, connect_sessions.mac_pubkey),
                 mac_callback=COALESCE(excluded.mac_callback, connect_sessions.mac_callback),
                 server_url=excluded.server_url,
                 scopes_json=excluded.scopes_json,
                 expires_at=excluded.expires_at,
                 room_claim_secret=COALESCE(excluded.room_claim_secret, connect_sessions.room_claim_secret)"#,
            params![
                connect_id, device_id, mac_nonce, mac_pubkey, mac_callback, server_url,
                scopes_json, created_at, expires_at, room_claim_secret,
            ],
        )?;
        Ok(())
    }

    /// Fetch a non-expired connect session.
    /// Fetch a non-expired connect session.
    #[allow(clippy::type_complexity)] // tuple shape is internal; matches PairingStore caller
    pub fn get_connect_session(
        &self,
        connect_id: &str,
    ) -> anyhow::Result<Option<(String, String, Option<String>, String)>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            "SELECT server_url, expires_at, mac_callback, status FROM connect_sessions
             WHERE connect_id = ?1 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![connect_id, now])?;
        if let Some(row) = rows.next()? {
            let server_url: String = row.get(0)?;
            let expires_at: String = row.get(1)?;
            let mac_callback: Option<String> = row.get(2).ok();
            let status: String = row.get(3)?;
            return Ok(Some((server_url, expires_at, mac_callback, status)));
        }
        Ok(None)
    }

    /// V2 P0 round 2: full lookup including device_id, scopes, created_at,
    /// room_claim_secret. Used by /status rehydration.
    #[allow(clippy::type_complexity)]
    pub fn get_connect_session_with_meta(
        &self,
        connect_id: &str,
    ) -> anyhow::Result<
        Option<(
            String,
            String,
            String,
            Option<String>,
            String,
            String,
            String,
        )>,
    > {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT device_id, server_url, expires_at, mac_callback, status, scopes_json, room_claim_secret
             FROM connect_sessions
             WHERE connect_id = ?1 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![connect_id, now])?;
        if let Some(row) = rows.next()? {
            let device_id: Option<String> = row.get(0).ok();
            let server_url: String = row.get(1)?;
            let expires_at: String = row.get(2)?;
            let mac_callback: Option<String> = row.get(3).ok();
            let status: String = row.get(4)?;
            let scopes_json: Option<String> = row.get(5).ok();
            let room_claim_secret: Option<String> = row.get(6).ok();
            let scopes_json = scopes_json.unwrap_or_else(|| "[]".into());
            return Ok(Some((
                device_id.unwrap_or_default(),
                server_url,
                expires_at,
                mac_callback,
                status,
                scopes_json,
                room_claim_secret.unwrap_or_default(),
            )));
        }
        Ok(None)
    }

    /// V2 P0 round 2: look up a device_token row by device_id.
    pub fn get_device_token_by_device_id(
        &self,
        device_id: &str,
    ) -> anyhow::Result<Option<crate::types::DeviceToken>> {
        use crate::types::DeviceToken;
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT token, scopes_json, issued_at, expires_at, issued_to
             FROM device_tokens WHERE device_id = ?1
             ORDER BY issued_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![device_id])?;
        if let Some(row) = rows.next()? {
            let token: String = row.get(0)?;
            let scopes_json: Option<String> = row.get(1).ok();
            let issued_at: String = row.get(2)?;
            let expires_at: String = row.get(3)?;
            let issued_to: Option<String> = row.get(4).ok();
            let scopes: Vec<String> = scopes_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_default();
            let issued_at_dt = chrono::DateTime::parse_from_rfc3339(&issued_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now() + chrono::Duration::days(30));
            return Ok(Some(DeviceToken {
                device_id: device_id.to_string(),
                token,
                scopes,
                issued_at: issued_at_dt,
                expires_at: expires_at_dt,
                last_used_at: None,
                issued_to: issued_to.unwrap_or_else(|| "ledger".to_string()),
            }));
        }
        Ok(None)
    }

    /// Mark a connect session as completed.
    /// V2: Enumerate every non-expired connect_session row. Used by
    /// /v1/connect/rooms to rehydrate the room list after a daemon
    /// restart so VPS-created rooms are discoverable.
    pub fn list_connect_sessions(&self) -> anyhow::Result<Vec<(String, String, String, String)>> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT connect_id, server_url, expires_at, status FROM connect_sessions WHERE expires_at > ?1 AND status != 'completed' ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Delete expired, incomplete connect-session rooms during daemon startup.
    pub fn cleanup_expired_pairing_rooms(&self) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        let deleted = conn.execute(
            "DELETE FROM connect_sessions WHERE expires_at <= ?1 AND status != 'completed'",
            params![now],
        )?;
        Ok(deleted)
    }

    /// V2: Mark a connect_session as completed in the SQLite ledger. Used by
    /// /v1/connect/room/approve after the token is minted, so the durable
    /// record reflects the transition (and so the room is no longer
    /// rehydrated as in-flight on the next daemon restart).
    pub fn complete_connect_session(&self, connect_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE connect_sessions SET status = 'completed' WHERE connect_id = ?1",
            params![connect_id],
        )?;
        // best-effort token-delivery timestamp
        let _ = conn.execute(
            "UPDATE connect_sessions SET expires_at = ?1 WHERE connect_id = ?2 AND expires_at > ?1",
            params![now, connect_id],
        );
        Ok(())
    }

    /// V2 (privacy): Delete connect_session rows where the room expired
    /// without being completed AND the row is not currently bound to an
    /// active device. Returns the count of rows removed. Wipes any
    /// partial MAC negotiation data (mac_nonce, mac_pubkey, mac_callback).
    pub fn cleanup_expired_connect_sessions(&self) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        // Only delete rows that are expired AND not currently paired (no device_id).
        let removed = conn.execute(
            "DELETE FROM connect_sessions \
             WHERE expires_at < ?1 \
               AND (device_id IS NULL OR device_id = '') \
               AND status != 'completed'",
            params![now],
        )?;
        if removed > 0 {
            tracing::info!(
                removed,
                "cleanup_expired_connect_sessions: purged unpaired expired rooms"
            );
        }
        Ok(removed)
    }

    /// V2: Force a WAL checkpoint so all just-committed writes are visible
    /// to readers and the on-disk file is consistent. Called after every
    /// trust transition (room create, room join, room approve, token revoke).
    pub fn checkpoint_wal(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
        Ok(())
    }

    /// V2: List non-completed, non-expired connect_sessions. Used by
    /// rehydrate_pairing_state_from_ledger and by /v1/connect/rooms.
    pub fn list_active_connect_sessions(
        &self,
    ) -> anyhow::Result<Vec<(String, String, String, String)>> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut stmt = conn.prepare(
            "SELECT connect_id, server_url, expires_at, status FROM connect_sessions
             WHERE expires_at > ?1 ORDER BY created_at DESC",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = stmt.query_map(params![now], |row| {
            let connect_id: String = row.get(0)?;
            let server_url: String = row.get(1)?;
            let expires_at: String = row.get(2)?;
            let status: String = row.get(3)?;
            Ok((connect_id, server_url, expires_at, status))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// V2: Persist a device token so it survives daemon restart.
    pub fn put_device_token(
        &self,
        token: &str,
        device_id: &str,
        scopes_json: Option<&str>,
        issued_at: &str,
        expires_at: &str,
        issued_to: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        conn.execute(
            "INSERT OR REPLACE INTO device_tokens
             (token, device_id, scopes_json, issued_at, expires_at, last_used_at, issued_to)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)",
            params![
                token,
                device_id,
                scopes_json,
                issued_at,
                expires_at,
                issued_to
            ],
        )?;
        Ok(())
    }

    /// V2: Look up a device token by its Bearer string.
    pub fn get_device_token(&self, token: &str) -> anyhow::Result<Option<(String, String)>> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT device_id, expires_at FROM device_tokens WHERE token = ?1 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![token, now])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    /// V2: Load the FULL DeviceToken-shaped record (with scopes) for auth
    /// rehydration after daemon restart. Uses the storage JSON column to
    /// preserve the granted scopes exactly as minted.
    pub fn load_device_token_full(
        &self,
        token: &str,
    ) -> anyhow::Result<Option<PersistedDeviceToken>> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT device_id, scopes_json, issued_at, expires_at, issued_to FROM device_tokens
             WHERE token = ?1 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![token, now])?;
        if let Some(row) = rows.next()? {
            let device_id: String = row.get(0)?;
            let scopes_json: Option<String> = row.get(1).ok();
            let issued_at: String = row.get(2)?;
            let expires_at: String = row.get(3)?;
            let issued_to: Option<String> = row.get(4).ok();
            let scopes: Vec<String> = scopes_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
            let issued_at_dt = chrono::DateTime::parse_from_rfc3339(&issued_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now() + chrono::Duration::seconds(86400 * 30));
            Ok(Some(PersistedDeviceToken {
                device_id,
                scopes,
                issued_at: issued_at_dt,
                expires_at: expires_at_dt,
                issued_to: issued_to.unwrap_or_else(|| "ledger".to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    /// V2: Revoke a device token (used by /v1/device/pair/revoke).
    pub fn revoke_device_token(&self, token: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        conn.execute("DELETE FROM device_tokens WHERE token = ?1", params![token])?;
        Ok(())
    }

    pub fn list_device_tokens(&self) -> anyhow::Result<Vec<DeviceTokenRow>> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT token, device_id, scopes_json, issued_at, expires_at, issued_to
             FROM device_tokens WHERE expires_at > ?1",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = stmt.query_map(params![now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2).ok().flatten(),
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5).ok().flatten(),
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// V2: Revoke ALL tokens for a given device_id. Used by pair_revoke so
    /// that a daemon restart cannot resurrect a revoked device via the
    /// auth middleware's SQLite fallback.
    pub fn revoke_device_tokens_by_device(&self, device_id: &str) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let n = conn.execute(
            "DELETE FROM device_tokens WHERE device_id = ?1",
            params![device_id],
        )?;
        Ok(n)
    }

    /// V2 Invariant 6: revoke any prior active (non-expired) token for
    /// the same (device_id, host) tuple before minting a new one. Re-pair
    /// of the same Mac against the same VPS host should supersede the
    /// old token, not stack on top of it. Returns the number of rows
    /// deleted (0 or 1 in practice).
    ///
    /// Issued_to is stored as-is in device_tokens.issued_to. We treat
    /// NULL issued_to as 'any host' (defensive) and match on
    /// device_id+issued_to exactly. The partial UNIQUE index
    /// uniq_device_tokens_device_host_active guarantees no two active
    /// rows can coexist, so this is the canonical cleanup.
    pub fn revoke_active_token_for_device_host(
        &self,
        device_id: &str,
        host: &str,
    ) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        let n = conn.execute(
            "DELETE FROM device_tokens
             WHERE device_id = ?1 AND issued_to = ?2 AND expires_at > ?3",
            params![device_id, host, now],
        )?;
        Ok(n)
    }

    /// V2 Invariant 6 (stricter): revoke any prior active token for the
    /// same (mac_name, host) tuple by looking up all device_ids ever
    /// paired under that mac_name on that host. This catches re-pair
    /// across fresh device_id generations. Returns the number of rows
    /// deleted (0..=N).
    ///
    /// Used as a defense-in-depth alongside
    /// revoke_active_token_for_device_host, which catches re-pair
    /// within the same device_id (e.g. duplicate /join retries).
    ///
    /// Note: device_records is stored in a JSONL file (devices.jsonl),
    /// NOT a SQL table. We read it directly, collect distinct
    /// device_ids for (mac_name, host), then issue a single SQL DELETE
    /// against the device_tokens table for those device_ids.
    pub fn revoke_active_tokens_for_mac_host(
        &self,
        mac_name: &str,
        host: &str,
    ) -> anyhow::Result<usize> {
        if mac_name.is_empty() {
            return Ok(0);
        }
        // Step 1: read device_records JSONL ledger for this host.
        let records = self.read_device_records(host, usize::MAX)?;
        let device_ids: Vec<String> = records
            .into_iter()
            .filter(|r| r.name == mac_name && r.host == host && !r.revoked)
            .map(|r| r.device_id)
            .filter(|id| !id.is_empty())
            .collect();
        // Dedupe.
        let mut device_ids = device_ids;
        device_ids.sort();
        device_ids.dedup();
        if device_ids.is_empty() {
            return Ok(0);
        }
        // Step 2: build IN clause with one '?' per device_id, then
        // execute a single DELETE for all of them.
        let placeholders = vec!["?"; device_ids.len()].join(",");
        let sql = format!(
            "DELETE FROM device_tokens
             WHERE issued_to = ?1
               AND expires_at > ?2
               AND device_id IN ({})",
            placeholders
        );
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let now = chrono::Utc::now().to_rfc3339();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(host.to_string()));
        params.push(Box::new(now));
        for d in &device_ids {
            params.push(Box::new(d.clone()));
        }
        let param_refs: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        let n = conn.execute(&sql, &param_refs[..])?;
        Ok(n)
    }

    /// V2: Load the FULL connect_session row (including all fields used by
    /// /join, /approve, /status) for in-memory rehydrate on daemon startup.
    pub fn get_connect_session_full(
        &self,
        connect_id: &str,
    ) -> anyhow::Result<Option<PersistedConnectSessionFull>> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        let mut stmt = conn.prepare(
            "SELECT device_id, mac_nonce, mac_pubkey, mac_callback, server_url,
                    scopes_json, status, created_at, expires_at, room_claim_secret
             FROM connect_sessions WHERE connect_id = ?1 AND expires_at > ?2",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![connect_id, now])?;
        if let Some(row) = rows.next()? {
            let device_id: Option<String> = row.get(0).ok();
            let mac_nonce: Option<String> = row.get(1).ok();
            let mac_pubkey: Option<String> = row.get(2).ok();
            let mac_callback: Option<String> = row.get(3).ok();
            let server_url: String = row.get(4)?;
            let scopes_json: Option<String> = row.get(5).ok();
            let status: String = row.get(6)?;
            let created_at: String = row.get(7)?;
            let expires_at: String = row.get(8)?;
            let room_claim_secret: Option<String> = row.get(9).ok();
            let _ = (); // no-op marker
            let scopes: Vec<String> = scopes_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
            let created_at_dt = chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now() + chrono::Duration::seconds(300));
            Ok(Some(PersistedConnectSessionFull {
                connect_id: connect_id.to_string(),
                device_id: device_id.unwrap_or_default(),
                mac_name: String::new(),
                mac_nonce: mac_nonce.unwrap_or_default(),
                mac_pubkey,
                mac_callback,
                server_url,
                scopes,
                created_at: created_at_dt,
                expires_at: expires_at_dt,
                status,
                room_claim_secret: room_claim_secret.unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }
}

/// V2: Full connect_session record rehydrated from the SQLite ledger.
/// Mirrors ConnectSession but persisted via the connect_sessions table.
pub struct PersistedConnectSessionFull {
    pub connect_id: String,
    pub device_id: String,
    pub mac_name: String,
    pub mac_nonce: String,
    pub mac_pubkey: Option<String>,
    pub mac_callback: Option<String>,
    pub server_url: String,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
    /// V2 P0 round 2: room_claim_secret persisted to the ledger so
    /// the secret survives daemon restart. Empty string for rooms
    /// created by /firstrun (Mac-creates) — they have no secret.
    pub room_claim_secret: String,
}

fn curator_eval_ledger_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    data_dir.join(format!("curator-eval-ledger/{}", hash))
}

fn cognition_optimizer_artifacts_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    data_dir.join(format!("cognition-optimizer-artifacts/{}", hash))
}

impl SqlitePersistence {
    /// Append a CuratorEvalRun to the scope-bounded JSONL ledger.
    pub fn append_curator_eval_run(&self, run: &CuratorEvalRun) -> anyhow::Result<()> {
        let ledger_dir = curator_eval_ledger_dir_for_project(&self.data_dir, &run.project_root);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("eval-runs.jsonl");
        let line = serde_json::to_string(run)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        writeln!(file, "{}", line)?;
        debug!("Appended CuratorEvalRun to {:?}", ledger_file);
        Ok(())
    }

    /// Read recent CuratorEvalRuns for a project (most recent last).
    pub fn read_curator_eval_runs(
        &self,
        project_root: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<CuratorEvalRun>> {
        let ledger_dir = curator_eval_ledger_dir_for_project(&self.data_dir, project_root);
        let ledger_file = ledger_dir.join("eval-runs.jsonl");
        if !ledger_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&ledger_file)?;
        let mut entries: Vec<CuratorEvalRun> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let start = entries.len().saturating_sub(limit);
        entries = entries[start..].to_vec();
        Ok(entries)
    }

    /// Get the latest promoted CognitionOptimizerArtifact for a project+module.
    pub fn latest_promoted_artifact(
        &self,
        project_root: &str,
        module_name: &str,
    ) -> anyhow::Result<Option<CognitionOptimizerArtifact>> {
        let entries = self.read_cognition_optimizer_artifacts(project_root, module_name, 50)?;
        Ok(entries.into_iter().rev().find(|a| a.promoted))
    }

    /// Append a CognitionOptimizerArtifact to the scope-bounded JSONL ledger.
    pub fn append_cognition_optimizer_artifact(
        &self,
        artifact: &CognitionOptimizerArtifact,
    ) -> anyhow::Result<()> {
        let ledger_dir =
            cognition_optimizer_artifacts_dir_for_project(&self.data_dir, &artifact.project_root);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("artifacts.jsonl");
        let line = serde_json::to_string(artifact)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        writeln!(file, "{}", line)?;
        debug!("Appended CognitionOptimizerArtifact to {:?}", ledger_file);
        Ok(())
    }

    /// Read recent CognitionOptimizerArtifacts for a project+module.
    pub fn read_cognition_optimizer_artifacts(
        &self,
        project_root: &str,
        module_name: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<CognitionOptimizerArtifact>> {
        let ledger_dir =
            cognition_optimizer_artifacts_dir_for_project(&self.data_dir, project_root);
        let ledger_file = ledger_dir.join("artifacts.jsonl");
        if !ledger_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&ledger_file)?;
        let mut entries: Vec<CognitionOptimizerArtifact> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .filter(|a: &CognitionOptimizerArtifact| a.module_name == module_name)
            .collect();
        let start = entries.len().saturating_sub(limit);
        entries = entries[start..].to_vec();
        Ok(entries)
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
