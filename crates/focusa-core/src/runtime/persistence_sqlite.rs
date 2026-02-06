//! Persistence — SQLite canonical store.
//!
//! Canonical persistence:
//! - append-only events table
//! - versioned state snapshots
//!
//! ECS objects remain filesystem-backed (see reference::store).

use crate::types::{EventLogEntry, FocusaConfig, FocusaState};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::debug;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const SCHEMA_VERSION: i64 = 1;

/// SQLite-backed persistence.
///
/// NOTE: Focusa daemon is single-writer, but API reads may happen concurrently.
/// We keep a single Connection behind a Mutex for now (simple + correct).
#[derive(Clone)]
pub struct SqlitePersistence {
    pub data_dir: PathBuf,
    conn: Arc<Mutex<Connection>>,
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

        let this = Self {
            data_dir,
            conn: Arc::new(Mutex::new(conn)),
        };

        this.init_schema()?;
        Ok(this)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");

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
                if parsed != SCHEMA_VERSION {
                    anyhow::bail!(
                        "unsupported schema_version {} (expected {})",
                        parsed,
                        SCHEMA_VERSION
                    );
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
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO peers(peer_id, name, endpoint, auth_token, created_at, status)
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending')
            ON CONFLICT(peer_id) DO UPDATE SET
                name=excluded.name,
                endpoint=excluded.endpoint,
                auth_token=excluded.auth_token
            "#,
            params![
                peer_id,
                name,
                endpoint,
                auth_token,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn remove_peer(&self, peer_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        conn.execute(
            "DELETE FROM peers WHERE peer_id = ?1",
            params![peer_id],
        )?;
        Ok(())
    }

    pub fn list_peers(&self) -> anyhow::Result<Vec<PeerRecord>> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
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
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        conn.execute(
            "UPDATE peers SET status = ?2, last_seen_at = ?3 WHERE peer_id = ?1",
            params![peer_id, status, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ─── Sync Cursors ──────────────────────────────────────────────────────

    pub fn get_cursor(&self, peer_id: &str) -> anyhow::Result<Option<SyncCursor>> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
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
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO sync_cursors(peer_id, last_event_id, last_event_ts, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(peer_id) DO UPDATE SET
                last_event_id=excluded.last_event_id,
                last_event_ts=excluded.last_event_ts,
                updated_at=excluded.updated_at
            "#,
            params![peer_id, last_event_id, last_event_ts, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ─── Events for Sync ───────────────────────────────────────────────────

    pub fn event_exists(&self, event_id: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM events WHERE event_id = ?1",
            params![event_id],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn events_since(
        &self,
        since_ts: Option<&str>,
        since_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<EventLogEntry>> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
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
        let rows = stmt.query_map(
            params![since_ts, since_id, limit as i64],
            |r| {
                let payload: String = r.get(4)?;
                let mut entry: EventLogEntry = serde_json::from_str(&payload)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;
                // Override stored columns with authoritative DB values
                entry.id = r.get::<_, String>(0)?.parse().map_err(|_| rusqlite::Error::InvalidColumnType(0, "event_id".into(), rusqlite::types::Type::Text))?;
                entry.timestamp = DateTime::parse_from_rfc3339(&r.get::<_, String>(1)?)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(1, "ts".into(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                entry.machine_id = r.get(5)?;
                entry.instance_id = r.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok());
                entry.session_id = r.get::<_, Option<String>>(7)?.and_then(|s| s.parse().ok());
                entry.thread_id = r.get::<_, Option<String>>(8)?.and_then(|s| s.parse().ok());
                entry.is_observation = r.get::<_, i32>(9)? != 0;
                Ok(entry)
            },
        )?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn save_state(&self, state: &FocusaState) -> anyhow::Result<()> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        let ts = Utc::now();
        let state_json = serde_json::to_string(state)?;
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
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
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
                Ok(s) => Ok(Some(s)),
                Err(_) => {
                    // Backward compatibility: older snapshots won't have newer fields.
                    // Fall back to a fresh state rather than failing daemon startup.
                    Ok(None)
                }
            },
        }
    }

    pub fn machine_id(&self) -> anyhow::Result<String> {
        let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
        let id: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'machine_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        id.ok_or_else(|| anyhow::anyhow!("machine_id missing from meta"))
    }

    pub fn append_event(&self, entry: &EventLogEntry) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(entry)?;

        // Avoid re-locking the same mutex (machine_id() also locks conn).
        let (conn, machine_id) = {
            let conn = self.conn.lock().expect("sqlite conn mutex poisoned");
            let machine_id = entry.machine_id.clone().or_else(|| {
                conn.query_row(
                    "SELECT value FROM meta WHERE key = 'machine_id'",
                    [],
                    |row| row.get(0),
                )
                .optional()
                .ok()
                .flatten()
            });
            (conn, machine_id.unwrap_or_else(|| "unknown".to_string()))
        };

        conn.execute(
            r#"
            INSERT INTO events(
              event_id, ts, origin, correlation_id, payload_json,
              machine_id, instance_id, session_id, thread_id, is_observation
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                entry.id.to_string(),
                entry.timestamp.to_rfc3339(),
                format!("{:?}", entry.origin),
                entry.correlation_id.clone(),
                payload_json,
                machine_id,
                entry.instance_id.map(|v| v.to_string()),
                entry.session_id.map(|v| v.to_string()),
                entry.thread_id.map(|v| v.to_string()),
                entry.is_observation as i32,
            ],
        )?;

        Ok(())
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

fn shellexpand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") && let Ok(home) = std::env::var("HOME") {
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
