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
              thread_id TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
            CREATE INDEX IF NOT EXISTS idx_events_machine ON events(machine_id);
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_thread ON events(thread_id);


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
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
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
              machine_id, instance_id, session_id, thread_id
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
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
            ],
        )?;

        Ok(())
    }
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
