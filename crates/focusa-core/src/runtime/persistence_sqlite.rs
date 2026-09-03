//! Persistence — SQLite canonical store.
//!
//! Canonical persistence:
//! - append-only events table
//! - versioned state snapshots
//!
//! ECS objects remain filesystem-backed (see reference::store).

use crate::clt::retain_hot_window;
use crate::semantic_migration::{MigrationPlan, MigrationReceipt};
use crate::semantic_replay::{SemanticEventEnvelope, replay as replay_semantic_events};
use crate::silent_session::{
    SilentSession, SilentSessionConfigRevision, SilentSessionConfigRevisionId, SilentSessionEvent,
    SilentSessionEventId, SilentSessionId, SilentSessionLifecycleState, SilentSessionRun,
    SilentSessionRunId,
};
use crate::silent_session_authorization::{SilentSessionApproval, SilentSessionPrincipal};
use crate::silent_session_config::redacted_config_hash;
use crate::silent_session_writer::WriterLeaseRegistry;
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

const SCHEMA_VERSION: i64 = 6;

fn hot_clt_snapshot_max_nodes() -> usize {
    std::env::var("FOCUSA_HOT_CLT_MAX_NODES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1_000)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntitlementLimitReservationOutcome {
    Reserved,
    IdempotentReplay,
    Exhausted,
}

#[derive(Clone)]
pub struct SqlitePersistence {
    pub data_dir: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl SqlitePersistence {
    pub fn reserve_entitlement_limit(
        &self,
        reservation_id: &str,
        lease_id: &str,
        lease_sequence: u64,
        limit_bucket: &str,
        units: u64,
        available: u64,
    ) -> anyhow::Result<EntitlementLimitReservationOutcome> {
        anyhow::ensure!(
            !reservation_id.trim().is_empty(),
            "reservation_id is required"
        );
        anyhow::ensure!(!lease_id.trim().is_empty(), "lease_id is required");
        anyhow::ensure!(!limit_bucket.trim().is_empty(), "limit_bucket is required");
        anyhow::ensure!(units > 0, "reservation units must be positive");
        let mut connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let transaction = connection.transaction()?;
        let existing: Option<(String, String, u64, String, u64)> = transaction
            .query_row(
                "SELECT lease_id, status, lease_sequence, limit_bucket, units FROM entitlement_limit_reservations WHERE reservation_id = ?1",
                params![reservation_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .optional()?;
        let existing_released = if let Some((
            existing_lease,
            status,
            existing_sequence,
            existing_bucket,
            existing_units,
        )) = existing
        {
            anyhow::ensure!(
                existing_lease == lease_id
                    && existing_sequence == lease_sequence
                    && existing_bucket == limit_bucket
                    && existing_units == units,
                "entitlement reservation idempotency conflict"
            );
            if status != "released" {
                transaction.commit()?;
                return Ok(EntitlementLimitReservationOutcome::IdempotentReplay);
            }
            true
        } else {
            false
        };
        let consumed: u64 = transaction.query_row(
            "SELECT COALESCE(SUM(units), 0) FROM entitlement_limit_reservations WHERE lease_id = ?1 AND lease_sequence = ?2 AND limit_bucket = ?3 AND status IN ('reserved', 'committed')",
            params![lease_id, lease_sequence, limit_bucket],
            |row| row.get(0),
        )?;
        if consumed.saturating_add(units) > available {
            transaction.commit()?;
            return Ok(EntitlementLimitReservationOutcome::Exhausted);
        }
        if existing_released {
            transaction.execute(
                "UPDATE entitlement_limit_reservations SET status = 'reserved', settled_at = NULL, created_at = ?2 WHERE reservation_id = ?1",
                params![reservation_id, Utc::now().to_rfc3339()],
            )?;
        } else {
            transaction.execute(
                "INSERT INTO entitlement_limit_reservations (reservation_id, lease_id, lease_sequence, limit_bucket, units, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'reserved', ?6)",
                params![reservation_id, lease_id, lease_sequence, limit_bucket, units, Utc::now().to_rfc3339()],
            )?;
        }
        transaction.commit()?;
        Ok(EntitlementLimitReservationOutcome::Reserved)
    }

    pub fn settle_entitlement_limit(
        &self,
        reservation_id: &str,
        commit: bool,
    ) -> anyhow::Result<()> {
        let connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        connection.execute(
            "UPDATE entitlement_limit_reservations SET status = ?2, settled_at = ?3 WHERE reservation_id = ?1 AND status = 'reserved'",
            params![reservation_id, if commit { "committed" } else { "released" }, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn save_runtime_constitution(
        &self,
        constitution: &crate::agent_runtime_constitution::ProjectAgentRuntimeConstitution,
    ) -> anyhow::Result<()> {
        let connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::agent_runtime_constitution_store::save_runtime_constitution(
            &connection,
            constitution,
        )
    }

    pub fn load_runtime_constitution(
        &self,
        constitution_id: &str,
    ) -> anyhow::Result<Option<crate::agent_runtime_constitution::ProjectAgentRuntimeConstitution>>
    {
        let connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::agent_runtime_constitution_store::load_runtime_constitution(
            &connection,
            constitution_id,
        )
    }

    pub fn append_runtime_constitution_event(
        &self,
        event_id: &str,
        constitution_id: &str,
        idempotency_key: &str,
        event: &crate::agent_runtime_constitution::RuntimeConstitutionEvent,
    ) -> anyhow::Result<crate::agent_runtime_constitution_store::StoredRuntimeConstitutionEvent>
    {
        let mut connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::agent_runtime_constitution_store::append_runtime_constitution_event(
            &mut connection,
            event_id,
            constitution_id,
            idempotency_key,
            event,
        )
    }

    pub fn runtime_constitution_events(
        &self,
        constitution_id: &str,
    ) -> anyhow::Result<Vec<crate::agent_runtime_constitution_store::StoredRuntimeConstitutionEvent>>
    {
        let connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::agent_runtime_constitution_store::runtime_constitution_events(
            &connection,
            constitution_id,
        )
    }

    pub fn latest_runtime_constitution_event(
        &self,
        constitution_id: &str,
    ) -> anyhow::Result<
        Option<crate::agent_runtime_constitution_store::StoredRuntimeConstitutionEvent>,
    > {
        let connection = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::agent_runtime_constitution_store::latest_runtime_constitution_event(
            &connection,
            constitution_id,
        )
    }
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

fn table_has_columns(conn: &Connection, table: &str, columns: &[&str]) -> anyhow::Result<bool> {
    for column in columns {
        let present: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name=?2",
            params![table, column],
            |row| row.get(0),
        )?;
        if present == 0 {
            return Ok(false);
        }
    }
    Ok(true)
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
        let mut conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Existing installations can carry an `events` table created before
        // scoped runtime columns were introduced. `CREATE TABLE IF NOT EXISTS`
        // never upgrades that table, and the indexes below otherwise reference
        // missing columns before startup can bind the health endpoint.
        let events_exist: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'",
            [],
            |row| row.get(0),
        )?;
        if events_exist != 0 {
            let migration = conn.transaction()?;
            for (column, definition) in [
                ("machine_id", "machine_id TEXT"),
                ("instance_id", "instance_id TEXT"),
                ("session_id", "session_id TEXT"),
                ("thread_id", "thread_id TEXT"),
                (
                    "is_observation",
                    "is_observation INTEGER NOT NULL DEFAULT 0",
                ),
            ] {
                let present: i64 = migration.query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name=?1",
                    [column],
                    |row| row.get(0),
                )?;
                if present == 0 {
                    migration
                        .execute(&format!("ALTER TABLE events ADD COLUMN {definition}"), [])?;
                }
            }
            migration.commit()?;
        }

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

            -- V3: Spec 133 canonical SilentSession projections and append-only event chain.
            CREATE TABLE IF NOT EXISTS runtime_silent_sessions (
              session_id TEXT PRIMARY KEY,
              project_root TEXT NOT NULL,
              continuity_id TEXT NOT NULL,
              lifecycle_state TEXT NOT NULL,
              projection_json TEXT NOT NULL,
              projection_version INTEGER NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_silent_sessions_scope
              ON runtime_silent_sessions(project_root, continuity_id);

            -- V5: exact durable run projection and generation guard source.
            CREATE TABLE IF NOT EXISTS runtime_silent_session_runs (
              run_id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              generation INTEGER NOT NULL CHECK(generation > 0),
              run_json TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(session_id, generation),
              FOREIGN KEY(session_id) REFERENCES runtime_silent_sessions(session_id) ON DELETE RESTRICT
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_silent_session_runs_target
              ON runtime_silent_session_runs(session_id, run_id, generation);

            CREATE TABLE IF NOT EXISTS runtime_silent_session_events (
              event_id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              run_id TEXT NOT NULL,
              seq INTEGER NOT NULL,
              occurred_at TEXT NOT NULL,
              event_json TEXT NOT NULL,
              payload_sha256 TEXT NOT NULL,
              previous_hash TEXT NOT NULL,
              event_hash TEXT NOT NULL,
              UNIQUE(session_id, run_id, seq),
              FOREIGN KEY(session_id) REFERENCES runtime_silent_sessions(session_id) ON DELETE RESTRICT
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_silent_session_events_order
              ON runtime_silent_session_events(session_id, run_id, seq);

            -- V6: append-only transactional Silent Session config authority.
            CREATE TABLE IF NOT EXISTS runtime_silent_session_config_revisions (
              revision_id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              parent_revision_id TEXT,
              revision_json TEXT NOT NULL,
              effective_hash TEXT NOT NULL,
              applied_at TEXT NOT NULL,
              FOREIGN KEY(session_id) REFERENCES runtime_silent_sessions(session_id) ON DELETE RESTRICT
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_silent_session_config_revisions_history
              ON runtime_silent_session_config_revisions(session_id, applied_at, revision_id);

            -- V4: durable Spec 133 control-plane identities and one-shot approvals.
            CREATE TABLE IF NOT EXISTS runtime_silent_session_principals (
              actor_instance_ref TEXT PRIMARY KEY,
              actor_ref TEXT NOT NULL,
              principal_json TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS runtime_silent_session_approvals (
              approval_id TEXT PRIMARY KEY,
              action_digest TEXT NOT NULL,
              approval_json TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              consumed_at TEXT
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_runtime_silent_session_approval_digest
              ON runtime_silent_session_approvals(action_digest);
            CREATE TABLE IF NOT EXISTS runtime_silent_session_action_redemptions (
              action_digest TEXT PRIMARY KEY,
              approval_id TEXT NOT NULL,
              redeemed_at TEXT NOT NULL,
              FOREIGN KEY(approval_id) REFERENCES runtime_silent_session_approvals(approval_id) ON DELETE RESTRICT
            );
            CREATE TABLE IF NOT EXISTS runtime_silent_session_runner_identities (
              runner_id TEXT PRIMARY KEY,
              verifying_key_base64 TEXT NOT NULL,
              os_user TEXT NOT NULL,
              project_identity_ref TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS runtime_silent_session_writer_lease_registry (
              singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
              revision INTEGER NOT NULL,
              registry_json TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entitlement_limit_reservations (
              reservation_id TEXT PRIMARY KEY,
              lease_id TEXT NOT NULL,
              lease_sequence INTEGER NOT NULL,
              limit_bucket TEXT NOT NULL,
              units INTEGER NOT NULL CHECK(units > 0),
              status TEXT NOT NULL CHECK(status IN ('reserved', 'committed', 'released')),
              created_at TEXT NOT NULL,
              settled_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_entitlement_limit_reservations_lease
              ON entitlement_limit_reservations(lease_id, lease_sequence, limit_bucket, status);

            CREATE TABLE IF NOT EXISTS snapshots (
              name TEXT PRIMARY KEY,
              version INTEGER NOT NULL,
              ts TEXT NOT NULL,
              state_json TEXT NOT NULL
            );
            "#,
        )?;

        // V5 briefly used canonical Spec133 table names for a separate runtime
        // projection schema. Copy only tables that prove that V5 shape; retained
        // Spec133 tables use different identity columns and remain untouched.
        // INSERT OR IGNORE makes this a one-time, idempotent bridge.
        if table_has_columns(
            &conn,
            "silent_sessions",
            &[
                "session_id",
                "project_root",
                "continuity_id",
                "lifecycle_state",
                "projection_json",
                "projection_version",
                "updated_at",
            ],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_sessions
                  (session_id, project_root, continuity_id, lifecycle_state,
                   projection_json, projection_version, updated_at)
                SELECT session_id, project_root, continuity_id, lifecycle_state,
                       projection_json, projection_version, updated_at
                FROM silent_sessions;
                "#,
            )?;
        }
        if table_has_columns(&conn, "silent_session_runs", &["session_id", "generation"])? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_runs
                  (run_id, session_id, generation, run_json, updated_at)
                SELECT run_id, session_id, generation, run_json, updated_at
                FROM silent_session_runs;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_events",
            &[
                "event_id",
                "session_id",
                "run_id",
                "seq",
                "occurred_at",
                "event_json",
                "payload_sha256",
                "previous_hash",
                "event_hash",
            ],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_events
                  (event_id, session_id, run_id, seq, occurred_at, event_json,
                   payload_sha256, previous_hash, event_hash)
                SELECT event_id, session_id, run_id, seq, occurred_at, event_json,
                       payload_sha256, previous_hash, event_hash
                FROM silent_session_events;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_config_revisions",
            &[
                "revision_id",
                "session_id",
                "parent_revision_id",
                "effective_hash",
            ],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_config_revisions
                  (revision_id, session_id, parent_revision_id, revision_json, effective_hash, applied_at)
                SELECT revision_id, session_id, parent_revision_id, revision_json, effective_hash, applied_at
                FROM silent_session_config_revisions;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_principals",
            &["actor_instance_ref", "actor_ref", "principal_json"],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_principals
                  (actor_instance_ref, actor_ref, principal_json, updated_at)
                SELECT actor_instance_ref, actor_ref, principal_json, updated_at
                FROM silent_session_principals;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_approvals",
            &["action_digest", "approval_json", "consumed_at"],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_approvals
                  (approval_id, action_digest, approval_json, expires_at, consumed_at)
                SELECT approval_id, action_digest, approval_json, expires_at, consumed_at
                FROM silent_session_approvals;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_action_redemptions",
            &["action_digest", "approval_id", "redeemed_at"],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_action_redemptions
                  (action_digest, approval_id, redeemed_at)
                SELECT action_digest, approval_id, redeemed_at
                FROM silent_session_action_redemptions;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_runner_identities",
            &["runner_id", "verifying_key_base64", "project_identity_ref"],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_runner_identities
                  (runner_id, verifying_key_base64, os_user, project_identity_ref, updated_at)
                SELECT runner_id, verifying_key_base64, os_user, project_identity_ref, updated_at
                FROM silent_session_runner_identities;
                "#,
            )?;
        }
        if table_has_columns(
            &conn,
            "silent_session_writer_lease_registry",
            &["singleton", "revision", "registry_json"],
        )? {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO runtime_silent_session_writer_lease_registry
                  (singleton, revision, registry_json, updated_at)
                SELECT singleton, revision, registry_json, updated_at
                FROM silent_session_writer_lease_registry;
                "#,
            )?;
        }

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

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS semantic_pair_events (
                pair_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                event_id TEXT NOT NULL UNIQUE,
                envelope_json TEXT NOT NULL,
                event_hash TEXT NOT NULL,
                PRIMARY KEY (pair_id, sequence)
            );
            CREATE INDEX IF NOT EXISTS idx_semantic_pair_events_pair
                ON semantic_pair_events(pair_id, sequence);
            CREATE TABLE IF NOT EXISTS semantic_pair_scope_index (
                storage_key TEXT PRIMARY KEY,
                project_root TEXT NOT NULL,
                continuity_id TEXT NOT NULL,
                logical_pair_id TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_semantic_pair_scope_lookup
                ON semantic_pair_scope_index(project_root, continuity_id, logical_pair_id);
            CREATE TABLE IF NOT EXISTS semantic_pair_migrations (
                migration_id TEXT PRIMARY KEY,
                pair_id TEXT NOT NULL,
                receipt_json TEXT NOT NULL,
                aggregate_json TEXT NOT NULL,
                event_head_sequence INTEGER,
                rolled_back INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS semantic_pair_quarantine (
                pair_id TEXT PRIMARY KEY,
                found_version INTEGER NOT NULL,
                payload BLOB NOT NULL,
                reason TEXT NOT NULL,
                quarantined_at TEXT NOT NULL
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
            .project_root
            .as_deref()
            .or_else(|| {
                entry.correlation_id.as_deref().and_then(|value| {
                    value
                        .split('|')
                        .find_map(|part| part.strip_prefix("project_root="))
                })
            })
            .unwrap_or("unscoped_project_root")
            .to_string();
        let workstream_key = entry
            .continuity_id
            .as_deref()
            .or_else(|| {
                entry.correlation_id.as_deref().and_then(|value| {
                    value
                        .split('|')
                        .find_map(|part| part.strip_prefix("continuity_id="))
                })
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

use crate::types::{HltLedgerEntry, TrajectoryLadderEvent};

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
    /// Append one atomic logical batch to the project-scoped Trajectory Ladder ledger.
    /// Events must share one project scope; callers provide causal ordering and Lamport values.
    pub fn append_trajectory_ladder_events(
        &self,
        events: &[TrajectoryLadderEvent],
    ) -> anyhow::Result<()> {
        let Some(first) = events.first() else {
            return Ok(());
        };
        if events
            .iter()
            .any(|event| event.project_root != first.project_root)
        {
            anyhow::bail!("trajectory ladder batch crosses project scope");
        }
        let ledger_dir = trajectory_ledger_dir_for_project(&self.data_dir, &first.project_root);
        std::fs::create_dir_all(&ledger_dir)?;
        let ledger_file = ledger_dir.join("events.jsonl");
        let mut payload = Vec::new();
        for event in events {
            serde_json::to_writer(&mut payload, event)?;
            payload.push(b'\n');
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_file)?;
        use std::io::Write;
        file.write_all(&payload)?;
        file.sync_data()?;
        debug!(
            "Appended {} Trajectory Ladder events to {:?}",
            events.len(),
            ledger_file
        );
        Ok(())
    }

    /// Read canonical Ladder events and project legacy HLT entries into the same schema.
    /// Corrupt canonical lines fail closed; compatibility projection never mutates on read.
    pub fn read_trajectory_ladder_events(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<TrajectoryLadderEvent>> {
        let ledger_file = self.trajectory_ledger_path_for_project(project_root);
        let mut events = Vec::new();
        if ledger_file.exists() {
            let content = std::fs::read_to_string(&ledger_file)?;
            for (index, line) in content.lines().enumerate() {
                let event: TrajectoryLadderEvent = serde_json::from_str(line).map_err(|error| {
                    anyhow::anyhow!(
                        "invalid Trajectory Ladder event at {}:{}: {}",
                        ledger_file.display(),
                        index + 1,
                        error
                    )
                })?;
                events.push(event);
            }
        }

        let existing_ids: std::collections::HashSet<String> =
            events.iter().map(|event| event.event_id.clone()).collect();
        for legacy in self.read_hlt_ledger_entries(project_root, continuity_id, 500)? {
            let event = TrajectoryLadderEvent::from_hlt_ledger(&legacy);
            if !existing_ids.contains(&event.event_id) {
                events.push(event);
            }
        }
        events.retain(|event| {
            event.project_root == project_root
                && continuity_id
                    .is_none_or(|expected| event.continuity_id.as_deref() == Some(expected))
        });
        events.sort_by(|left, right| {
            left.lamport_ts
                .cmp(&right.lamport_ts)
                .then_with(|| left.timestamp.cmp(&right.timestamp))
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        let bounded_limit = limit.clamp(1, 500);
        let start = events.len().saturating_sub(bounded_limit);
        Ok(events[start..].to_vec())
    }

    pub fn trajectory_ledger_path_for_project(&self, project_root: &str) -> PathBuf {
        trajectory_ledger_dir_for_project(&self.data_dir, project_root).join("events.jsonl")
    }

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

fn trajectory_ledger_dir_for_project(data_dir: &Path, project_root: &str) -> PathBuf {
    let digest = Sha256::digest(project_root.as_bytes());
    let hash = hex::encode(&digest[..8]);
    data_dir.join(format!("trajectory-ledger/{hash}"))
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

    pub fn put_silent_session_principal(
        &self,
        principal: &SilentSessionPrincipal,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        conn.execute(
            r#"INSERT INTO runtime_silent_session_principals(actor_instance_ref, actor_ref, principal_json, updated_at)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(actor_instance_ref) DO UPDATE SET actor_ref=excluded.actor_ref,
                 principal_json=excluded.principal_json, updated_at=excluded.updated_at"#,
            params![principal.actor_instance_ref, principal.actor_ref, serde_json::to_string(principal)?, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn load_silent_session_principal(
        &self,
        actor_instance_ref: &str,
    ) -> anyhow::Result<Option<SilentSessionPrincipal>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let json: Option<String> = conn
            .query_row(
                "SELECT principal_json FROM runtime_silent_session_principals WHERE actor_instance_ref=?1",
                [actor_instance_ref],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    pub fn put_silent_session_approval(
        &self,
        approval: &SilentSessionApproval,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let changed = conn.execute(
            r#"INSERT INTO runtime_silent_session_approvals(approval_id, action_digest, approval_json, expires_at, consumed_at)
               VALUES (?1, ?2, ?3, ?4, NULL)
               ON CONFLICT(approval_id) DO UPDATE SET action_digest=excluded.action_digest,
                 approval_json=excluded.approval_json, expires_at=excluded.expires_at
               WHERE runtime_silent_session_approvals.consumed_at IS NULL"#,
            params![approval.approval_id, approval.action_digest, serde_json::to_string(approval)?, approval.expires_at.to_rfc3339()],
        )?;
        anyhow::ensure!(
            changed == 1,
            "consumed silent-session approval is immutable"
        );
        Ok(())
    }

    pub fn load_silent_session_approval(
        &self,
        approval_id: &str,
    ) -> anyhow::Result<Option<SilentSessionApproval>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let json: Option<String> = conn
            .query_row(
                "SELECT approval_json FROM runtime_silent_session_approvals WHERE approval_id=?1 AND consumed_at IS NULL",
                [approval_id],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    pub fn redeem_silent_session_approval(
        &self,
        approval_id: &str,
        action_digest: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<SilentSessionApproval> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let (json, stored_digest, expires_at, consumed_at): (String, String, String, Option<String>) = tx.query_row(
            "SELECT approval_json, action_digest, expires_at, consumed_at FROM runtime_silent_session_approvals WHERE approval_id=?1",
            [approval_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        anyhow::ensure!(
            consumed_at.is_none(),
            "silent-session approval already consumed"
        );
        anyhow::ensure!(
            stored_digest == action_digest,
            "silent-session approval digest mismatch"
        );
        let expiry = DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc);
        anyhow::ensure!(expiry > now, "silent-session approval expired");
        tx.execute(
            "INSERT INTO runtime_silent_session_action_redemptions(action_digest, approval_id, redeemed_at) VALUES (?1,?2,?3)",
            params![action_digest, approval_id, now.to_rfc3339()],
        )?;
        tx.execute(
            "UPDATE runtime_silent_session_approvals SET consumed_at=?2 WHERE approval_id=?1 AND consumed_at IS NULL",
            params![approval_id, now.to_rfc3339()],
        )?;
        tx.commit()?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn put_silent_session_runner_identity(
        &self,
        runner_id: &str,
        verifying_key_base64: &str,
        os_user: &str,
        project_identity_ref: &str,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            !runner_id.trim().is_empty() && !verifying_key_base64.trim().is_empty(),
            "runner identity and key are required"
        );
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        conn.execute(
            r#"INSERT INTO runtime_silent_session_runner_identities(runner_id, verifying_key_base64, os_user, project_identity_ref, updated_at)
               VALUES (?1,?2,?3,?4,?5)
               ON CONFLICT(runner_id) DO UPDATE SET verifying_key_base64=excluded.verifying_key_base64,
                 os_user=excluded.os_user, project_identity_ref=excluded.project_identity_ref,
                 updated_at=excluded.updated_at"#,
            params![runner_id, verifying_key_base64, os_user, project_identity_ref, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn load_silent_session_runner_identity(
        &self,
        runner_id: &str,
    ) -> anyhow::Result<Option<(String, String, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        conn.query_row(
            "SELECT verifying_key_base64, os_user, project_identity_ref FROM runtime_silent_session_runner_identities WHERE runner_id=?1",
            [runner_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional().map_err(Into::into)
    }

    pub fn load_silent_session_writer_lease_registry(
        &self,
    ) -> anyhow::Result<(u64, WriterLeaseRegistry)> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let stored = conn
            .query_row(
                "SELECT revision, registry_json FROM runtime_silent_session_writer_lease_registry WHERE singleton=1",
                [],
                |row| Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        match stored {
            Some((revision, json)) => {
                let registry: WriterLeaseRegistry = serde_json::from_str(&json)?;
                registry.validate()?;
                Ok((revision, registry))
            }
            None => Ok((0, WriterLeaseRegistry::default())),
        }
    }

    pub fn persist_silent_session_writer_lease_registry_cas(
        &self,
        expected_revision: u64,
        registry: &WriterLeaseRegistry,
    ) -> anyhow::Result<u64> {
        registry.validate()?;
        let json = serde_json::to_string(registry)?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let current = tx
            .query_row(
                "SELECT revision FROM runtime_silent_session_writer_lease_registry WHERE singleton=1",
                [],
                |row| row.get::<_, u64>(0),
            )
            .optional()?
            .unwrap_or(0);
        anyhow::ensure!(
            current == expected_revision,
            "silent-session writer lease registry CAS conflict"
        );
        let next = current
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("writer lease registry revision exhausted"))?;
        tx.execute(
            r#"INSERT INTO runtime_silent_session_writer_lease_registry(singleton, revision, registry_json, updated_at)
               VALUES (1,?1,?2,?3)
               ON CONFLICT(singleton) DO UPDATE SET revision=excluded.revision,
                 registry_json=excluded.registry_json, updated_at=excluded.updated_at"#,
            params![next, json, Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        Ok(next)
    }

    /// Atomically append a Spec 133 event and advance its session projection.
    pub fn persist_silent_session_event(
        &self,
        session: &SilentSession,
        event: &SilentSessionEvent,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            event.session_id == session.session_id,
            "silent-session event scope mismatch"
        );
        let projection_json = serde_json::to_string(session)?;
        let event_json = serde_json::to_string(event)?;
        let payload_sha256 = sha256_hex(event_json.as_bytes());
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        tx.execute(
            r#"INSERT INTO runtime_silent_sessions(session_id, project_root, continuity_id, lifecycle_state, projection_json, projection_version, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(session_id) DO UPDATE SET project_root=excluded.project_root,
                 continuity_id=excluded.continuity_id, lifecycle_state=excluded.lifecycle_state,
                 projection_json=excluded.projection_json, projection_version=excluded.projection_version,
                 updated_at=excluded.updated_at"#,
            params![session.session_id.to_string(), session.project_root.to_string_lossy(),
                session.continuity_id, format!("{:?}", session.lifecycle_state), projection_json,
                i64::try_from(event.seq)?, Utc::now().to_rfc3339()],
        )?;
        let replay: Option<String> = tx
            .query_row(
                "SELECT event_json FROM runtime_silent_session_events WHERE event_id=?1",
                [event.event_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing) = replay {
            anyhow::ensure!(
                existing == event_json,
                "silent-session event id replay conflict"
            );
            tx.commit()?;
            return Ok(());
        }
        let previous: Option<(i64, String)> = tx.query_row(
            "SELECT seq, event_hash FROM runtime_silent_session_events WHERE session_id=?1 AND run_id=?2 ORDER BY seq DESC LIMIT 1",
            params![event.session_id.to_string(), event.run_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
        if let Some((last_seq, _)) = previous.as_ref() {
            anyhow::ensure!(
                i64::try_from(event.seq)? == last_seq + 1,
                "silent-session event sequence gap"
            );
        }
        let previous_hash = previous
            .map(|(_, hash)| hash)
            .unwrap_or_else(|| "GENESIS".into());
        let event_hash = event_chain_hash(
            &previous_hash,
            &event.event_id.to_string(),
            &event.occurred_at.to_rfc3339(),
            &payload_sha256,
        );
        tx.execute(
            "INSERT INTO runtime_silent_session_events(event_id, session_id, run_id, seq, occurred_at, event_json, payload_sha256, previous_hash, event_hash) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![event.event_id.to_string(), event.session_id.to_string(), event.run_id.to_string(),
                i64::try_from(event.seq)?, event.occurred_at.to_rfc3339(), event_json,
                payload_sha256, previous_hash, event_hash],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Atomically create a canonical draft session, its first run generation,
    /// and genesis event while redeeming the exact durable approval.
    #[allow(clippy::too_many_arguments)]
    pub fn persist_silent_session_create(
        &self,
        approval_id: &str,
        action_digest: &str,
        authorized_at: DateTime<Utc>,
        session: &SilentSession,
        run: &SilentSessionRun,
        event: &SilentSessionEvent,
        initial_config_revision: &SilentSessionConfigRevision,
        effective_config_hash: &str,
    ) -> anyhow::Result<()> {
        session
            .validate()
            .map_err(|error| anyhow::anyhow!("invalid silent-session projection: {error:?}"))?;
        run.validate(session)
            .map_err(|error| anyhow::anyhow!("invalid silent-session run: {error:?}"))?;
        event
            .validate(session, run)
            .map_err(|error| anyhow::anyhow!("invalid silent-session event: {error:?}"))?;
        anyhow::ensure!(
            session.lifecycle_state == SilentSessionLifecycleState::Draft,
            "silent-session create requires draft lifecycle"
        );
        anyhow::ensure!(
            session.active_run_id == Some(run.run_id)
                && run.generation == 1
                && run.current_event_seq == 1
                && event.seq == 1,
            "silent-session create requires generation-one genesis event"
        );
        anyhow::ensure!(
            initial_config_revision.session_id == session.session_id
                && initial_config_revision.revision_id == session.config_revision_id
                && initial_config_revision.parent_revision_id.is_none()
                && initial_config_revision.applied_at.is_some(),
            "silent-session create requires matching initial config revision"
        );
        anyhow::ensure!(
            redacted_config_hash(&initial_config_revision.config)? == effective_config_hash,
            "silent-session initial config hash mismatch"
        );

        let projection_json = serde_json::to_string(session)?;
        let run_json = serde_json::to_string(run)?;
        let event_json = serde_json::to_string(event)?;
        let config_revision_json = serde_json::to_string(initial_config_revision)?;
        let payload_sha256 = sha256_hex(event_json.as_bytes());
        let event_hash = event_chain_hash(
            "GENESIS",
            &event.event_id.to_string(),
            &event.occurred_at.to_rfc3339(),
            &payload_sha256,
        );
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let (stored_digest, expires_at, consumed_at): (String, String, Option<String>) = tx
            .query_row(
                "SELECT action_digest, expires_at, consumed_at FROM runtime_silent_session_approvals WHERE approval_id=?1",
                [approval_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
        anyhow::ensure!(
            consumed_at.is_none(),
            "silent-session approval already consumed"
        );
        anyhow::ensure!(
            stored_digest == action_digest,
            "silent-session approval digest mismatch"
        );
        let expiry = DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc);
        anyhow::ensure!(expiry > authorized_at, "silent-session approval expired");

        tx.execute(
            "INSERT INTO runtime_silent_session_action_redemptions(action_digest, approval_id, redeemed_at) VALUES (?1,?2,?3)",
            params![action_digest, approval_id, authorized_at.to_rfc3339()],
        )?;
        let consumed = tx.execute(
            "UPDATE runtime_silent_session_approvals SET consumed_at=?2 WHERE approval_id=?1 AND consumed_at IS NULL",
            params![approval_id, authorized_at.to_rfc3339()],
        )?;
        anyhow::ensure!(consumed == 1, "silent-session approval redemption conflict");
        tx.execute(
            "INSERT INTO runtime_silent_sessions(session_id, project_root, continuity_id, lifecycle_state, projection_json, projection_version, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![session.session_id.to_string(), session.project_root.to_string_lossy(),
                session.continuity_id, format!("{:?}", session.lifecycle_state), projection_json,
                i64::try_from(event.seq)?, authorized_at.to_rfc3339()],
        )?;
        tx.execute(
            "INSERT INTO runtime_silent_session_config_revisions(revision_id, session_id, parent_revision_id, revision_json, effective_hash, applied_at) VALUES (?1,?2,NULL,?3,?4,?5)",
            params![initial_config_revision.revision_id.to_string(), session.session_id.to_string(),
                config_revision_json, effective_config_hash, authorized_at.to_rfc3339()],
        )?;
        tx.execute(
            "INSERT INTO runtime_silent_session_runs(run_id, session_id, generation, run_json, updated_at) VALUES (?1,?2,?3,?4,?5)",
            params![run.run_id.to_string(), session.session_id.to_string(),
                i64::try_from(run.generation)?, run_json, authorized_at.to_rfc3339()],
        )?;
        tx.execute(
            "INSERT INTO runtime_silent_session_events(event_id, session_id, run_id, seq, occurred_at, event_json, payload_sha256, previous_hash, event_hash) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![event.event_id.to_string(), event.session_id.to_string(), event.run_id.to_string(),
                i64::try_from(event.seq)?, event.occurred_at.to_rfc3339(), event_json,
                payload_sha256, "GENESIS", event_hash],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Atomically compare-and-swap one lifecycle projection and append its
    /// canonical event. Both the lifecycle state and run event cursor fence
    /// concurrent controls; the run generation fences controls delayed across
    /// restart/adoption.
    #[allow(clippy::too_many_arguments)]
    pub fn persist_silent_session_lifecycle_cas(
        &self,
        expected_lifecycle: SilentSessionLifecycleState,
        expected_generation: u64,
        expected_run_id: SilentSessionRunId,
        approval_id: &str,
        action_digest: &str,
        authorized_at: DateTime<Utc>,
        session: &SilentSession,
        run: &SilentSessionRun,
        event: &SilentSessionEvent,
    ) -> anyhow::Result<()> {
        session
            .validate()
            .map_err(|error| anyhow::anyhow!("invalid silent-session projection: {error:?}"))?;
        run.validate(session)
            .map_err(|error| anyhow::anyhow!("invalid silent-session run: {error:?}"))?;
        event
            .validate(session, run)
            .map_err(|error| anyhow::anyhow!("invalid silent-session event: {error:?}"))?;
        let creates_generation = run.run_id != expected_run_id;
        anyhow::ensure!(
            (!creates_generation && run.generation == expected_generation)
                || (creates_generation && run.generation == expected_generation.saturating_add(1)),
            "silent-session generation conflict"
        );
        anyhow::ensure!(
            run.current_event_seq == event.seq && (!creates_generation || event.seq == 1),
            "silent-session run cursor must advance to event sequence"
        );

        let projection_json = serde_json::to_string(session)?;
        let run_json = serde_json::to_string(run)?;
        let event_json = serde_json::to_string(event)?;
        let payload_sha256 = sha256_hex(event_json.as_bytes());
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;

        let (stored_digest, expires_at, consumed_at): (String, String, Option<String>) = tx
            .query_row(
                "SELECT action_digest, expires_at, consumed_at FROM runtime_silent_session_approvals WHERE approval_id=?1",
                [approval_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
        anyhow::ensure!(
            consumed_at.is_none(),
            "silent-session approval already consumed"
        );
        anyhow::ensure!(
            stored_digest == action_digest,
            "silent-session approval digest mismatch"
        );
        let expiry = DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc);
        anyhow::ensure!(expiry > authorized_at, "silent-session approval expired");

        let stored_session_json: String = tx.query_row(
            "SELECT projection_json FROM runtime_silent_sessions WHERE session_id=?1",
            [session.session_id.to_string()],
            |row| row.get(0),
        )?;
        let stored_session: SilentSession = serde_json::from_str(&stored_session_json)?;
        anyhow::ensure!(
            stored_session.lifecycle_state == expected_lifecycle,
            "silent-session lifecycle conflict"
        );
        anyhow::ensure!(
            stored_session.project_root == session.project_root
                && stored_session.project_identity_ref == session.project_identity_ref
                && stored_session.continuity_id == session.continuity_id,
            "silent-session scope mutation"
        );

        let (stored_generation, stored_run_json): (i64, String) = tx.query_row(
            "SELECT generation, run_json FROM runtime_silent_session_runs WHERE session_id=?1 AND run_id=?2",
            params![session.session_id.to_string(), expected_run_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let stored_run: SilentSessionRun = serde_json::from_str(&stored_run_json)?;
        let latest_generation: i64 = tx.query_row(
            "SELECT MAX(generation) FROM runtime_silent_session_runs WHERE session_id=?1",
            [session.session_id.to_string()],
            |row| row.get(0),
        )?;
        anyhow::ensure!(
            u64::try_from(stored_generation)? == expected_generation
                && stored_run.generation == expected_generation
                && u64::try_from(latest_generation)? == expected_generation,
            "silent-session generation conflict"
        );
        anyhow::ensure!(
            (creates_generation && event.seq == 1)
                || (!creates_generation
                    && event.seq == stored_run.current_event_seq.saturating_add(1)),
            "silent-session event cursor conflict"
        );

        let previous: Option<(i64, String)> = tx
            .query_row(
                "SELECT seq, event_hash FROM runtime_silent_session_events WHERE session_id=?1 AND run_id=?2 ORDER BY seq DESC LIMIT 1",
                params![event.session_id.to_string(), event.run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (last_seq, previous_hash) = previous.unwrap_or((0, "GENESIS".into()));
        anyhow::ensure!(
            i64::try_from(event.seq)? == last_seq + 1,
            "silent-session event sequence gap"
        );
        let event_hash = event_chain_hash(
            &previous_hash,
            &event.event_id.to_string(),
            &event.occurred_at.to_rfc3339(),
            &payload_sha256,
        );

        tx.execute(
            "INSERT INTO runtime_silent_session_action_redemptions(action_digest, approval_id, redeemed_at) VALUES (?1,?2,?3)",
            params![action_digest, approval_id, authorized_at.to_rfc3339()],
        )?;
        let consumed = tx.execute(
            "UPDATE runtime_silent_session_approvals SET consumed_at=?2 WHERE approval_id=?1 AND consumed_at IS NULL",
            params![approval_id, authorized_at.to_rfc3339()],
        )?;
        anyhow::ensure!(consumed == 1, "silent-session approval redemption conflict");
        tx.execute(
            "UPDATE runtime_silent_sessions SET lifecycle_state=?2, projection_json=?3, projection_version=?4, updated_at=?5 WHERE session_id=?1",
            params![session.session_id.to_string(), format!("{:?}", session.lifecycle_state), projection_json,
                i64::try_from(event.seq)?, Utc::now().to_rfc3339()],
        )?;
        if creates_generation {
            tx.execute(
                "INSERT INTO runtime_silent_session_runs(run_id, session_id, generation, run_json, updated_at) VALUES (?1,?2,?3,?4,?5)",
                params![run.run_id.to_string(), session.session_id.to_string(),
                    i64::try_from(run.generation)?, run_json, Utc::now().to_rfc3339()],
            )?;
        } else {
            tx.execute(
                "UPDATE runtime_silent_session_runs SET run_json=?3, updated_at=?4 WHERE session_id=?1 AND run_id=?2 AND generation=?5",
                params![session.session_id.to_string(), run.run_id.to_string(), run_json,
                    Utc::now().to_rfc3339(), i64::try_from(expected_generation)?],
            )?;
        }
        tx.execute(
            "INSERT INTO runtime_silent_session_events(event_id, session_id, run_id, seq, occurred_at, event_json, payload_sha256, previous_hash, event_hash) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![event.event_id.to_string(), event.session_id.to_string(), event.run_id.to_string(),
                i64::try_from(event.seq)?, event.occurred_at.to_rfc3339(), event_json,
                payload_sha256, previous_hash, event_hash],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_silent_session(
        &self,
        session_id: SilentSessionId,
    ) -> anyhow::Result<Option<SilentSession>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let json: Option<String> = conn
            .query_row(
                "SELECT projection_json FROM runtime_silent_sessions WHERE session_id=?1",
                [session_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    /// Return a bounded newest-first projection of durable logical sessions.
    /// Callers receive canonical records only; process runs remain separate and
    /// must be addressed by their exact run identity.
    pub fn list_silent_sessions(&self, limit: usize) -> anyhow::Result<Vec<SilentSession>> {
        anyhow::ensure!(limit > 0, "silent-session list limit must be positive");
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let mut statement = conn.prepare(
            "SELECT projection_json FROM runtime_silent_sessions ORDER BY updated_at DESC, session_id DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([i64::try_from(limit)?], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    /// Insert the initial config authority record for an existing session.
    /// The revision identity must match the session projection so config reads
    /// can never silently fall back to an unversioned request.
    pub fn put_initial_silent_session_config_revision(
        &self,
        session: &SilentSession,
        revision: &SilentSessionConfigRevision,
        effective_hash: &str,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            revision.session_id == session.session_id
                && revision.revision_id == session.config_revision_id
                && revision.parent_revision_id.is_none(),
            "invalid initial silent-session config revision"
        );
        let revision_json = serde_json::to_string(revision)?;
        let applied_at = revision
            .applied_at
            .ok_or_else(|| anyhow::anyhow!("config revision must have applied_at"))?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let stored_projection: String = conn.query_row(
            "SELECT projection_json FROM runtime_silent_sessions WHERE session_id=?1",
            [session.session_id.to_string()],
            |row| row.get(0),
        )?;
        let stored: SilentSession = serde_json::from_str(&stored_projection)?;
        anyhow::ensure!(
            stored.config_revision_id == revision.revision_id,
            "silent-session config projection conflict"
        );
        conn.execute(
            "INSERT INTO runtime_silent_session_config_revisions(revision_id, session_id, parent_revision_id, revision_json, effective_hash, applied_at) VALUES (?1,?2,NULL,?3,?4,?5)",
            params![revision.revision_id.to_string(), session.session_id.to_string(), revision_json,
                effective_hash, applied_at.to_rfc3339()],
        )?;
        Ok(())
    }

    /// Atomically compare-and-swap the current config revision and session
    /// projection. A stale writer cannot append history or move the projection.
    #[allow(clippy::too_many_arguments)]
    pub fn persist_silent_session_config_revision_cas(
        &self,
        expected_revision_id: SilentSessionConfigRevisionId,
        approval_id: &str,
        action_digest: &str,
        authorized_at: DateTime<Utc>,
        session: &SilentSession,
        revision: &SilentSessionConfigRevision,
        effective_hash: &str,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            revision.session_id == session.session_id
                && revision.revision_id == session.config_revision_id
                && revision.parent_revision_id == Some(expected_revision_id),
            "invalid silent-session config revision transition"
        );
        let revision_json = serde_json::to_string(revision)?;
        let projection_json = serde_json::to_string(session)?;
        let applied_at = revision
            .applied_at
            .ok_or_else(|| anyhow::anyhow!("config revision must have applied_at"))?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let (stored_digest, expires_at, consumed_at): (String, String, Option<String>) = tx
            .query_row(
                "SELECT action_digest, expires_at, consumed_at FROM runtime_silent_session_approvals WHERE approval_id=?1",
                [approval_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
        anyhow::ensure!(
            consumed_at.is_none(),
            "silent-session approval already consumed"
        );
        anyhow::ensure!(
            stored_digest == action_digest,
            "silent-session approval digest mismatch"
        );
        let expiry = DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc);
        anyhow::ensure!(expiry > authorized_at, "silent-session approval expired");
        let stored_projection: String = tx.query_row(
            "SELECT projection_json FROM runtime_silent_sessions WHERE session_id=?1",
            [session.session_id.to_string()],
            |row| row.get(0),
        )?;
        let stored: SilentSession = serde_json::from_str(&stored_projection)?;
        anyhow::ensure!(
            stored.config_revision_id == expected_revision_id,
            "silent-session config revision conflict"
        );
        let parent_exists: Option<String> = tx
            .query_row(
                "SELECT revision_id FROM runtime_silent_session_config_revisions WHERE session_id=?1 AND revision_id=?2",
                params![session.session_id.to_string(), expected_revision_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        anyhow::ensure!(
            parent_exists.is_some(),
            "silent-session config parent missing"
        );
        tx.execute(
            "INSERT INTO runtime_silent_session_action_redemptions(action_digest, approval_id, redeemed_at) VALUES (?1,?2,?3)",
            params![action_digest, approval_id, authorized_at.to_rfc3339()],
        )?;
        let consumed = tx.execute(
            "UPDATE runtime_silent_session_approvals SET consumed_at=?2 WHERE approval_id=?1 AND consumed_at IS NULL",
            params![approval_id, authorized_at.to_rfc3339()],
        )?;
        anyhow::ensure!(consumed == 1, "silent-session approval redemption conflict");
        tx.execute(
            "INSERT INTO runtime_silent_session_config_revisions(revision_id, session_id, parent_revision_id, revision_json, effective_hash, applied_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![revision.revision_id.to_string(), session.session_id.to_string(),
                expected_revision_id.to_string(), revision_json, effective_hash,
                applied_at.to_rfc3339()],
        )?;
        let changed = tx.execute(
            "UPDATE runtime_silent_sessions SET projection_json=?3, updated_at=?4 WHERE session_id=?1 AND json_extract(projection_json, '$.config_revision_id')=?2",
            params![session.session_id.to_string(), expected_revision_id.to_string(),
                projection_json, Utc::now().to_rfc3339()],
        )?;
        anyhow::ensure!(changed == 1, "silent-session config revision conflict");
        tx.commit()?;
        Ok(())
    }

    pub fn load_silent_session_config_revision(
        &self,
        session_id: SilentSessionId,
        revision_id: SilentSessionConfigRevisionId,
    ) -> anyhow::Result<Option<(SilentSessionConfigRevision, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT revision_json, effective_hash FROM runtime_silent_session_config_revisions WHERE session_id=?1 AND revision_id=?2",
                params![session_id.to_string(), revision_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        row.map(|(json, hash)| Ok((serde_json::from_str(&json)?, hash)))
            .transpose()
    }

    pub fn load_silent_session_config_history(
        &self,
        session_id: SilentSessionId,
    ) -> anyhow::Result<Vec<(SilentSessionConfigRevision, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let mut statement = conn.prepare(
            "SELECT revision_json, effective_hash FROM runtime_silent_session_config_revisions WHERE session_id=?1 ORDER BY applied_at, revision_id",
        )?;
        let rows = statement.query_map([session_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.map(|row| {
            let (json, hash) = row?;
            Ok((serde_json::from_str(&json)?, hash))
        })
        .collect()
    }

    /// Persist the exact process-run projection used by API generation guards.
    /// The owning session must already be durable, and a generation can never
    /// be rebound to another run identity.
    pub fn put_silent_session_run(
        &self,
        session: &SilentSession,
        run: &SilentSessionRun,
    ) -> anyhow::Result<()> {
        run.validate(session)
            .map_err(|error| anyhow::anyhow!("invalid silent-session run: {error:?}"))?;
        let run_json = serde_json::to_string(run)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let durable_session: Option<String> = conn
            .query_row(
                "SELECT session_id FROM runtime_silent_sessions WHERE session_id=?1",
                [session.session_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        anyhow::ensure!(
            durable_session.is_some(),
            "silent-session run requires durable owning session"
        );
        let changed = conn.execute(
            r#"INSERT INTO runtime_silent_session_runs(run_id, session_id, generation, run_json, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(run_id) DO UPDATE SET run_json=excluded.run_json,
                 updated_at=excluded.updated_at
               WHERE runtime_silent_session_runs.session_id=excluded.session_id
                 AND runtime_silent_session_runs.generation=excluded.generation"#,
            params![
                run.run_id.to_string(),
                run.session_id.to_string(),
                i64::try_from(run.generation)?,
                run_json,
                Utc::now().to_rfc3339()
            ],
        )?;
        anyhow::ensure!(changed == 1, "silent-session run identity conflict");
        Ok(())
    }

    pub fn load_silent_session_run(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
    ) -> anyhow::Result<Option<SilentSessionRun>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let json: Option<String> = conn
            .query_row(
                "SELECT run_json FROM runtime_silent_session_runs WHERE session_id=?1 AND run_id=?2",
                params![session_id.to_string(), run_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    /// Load one exact run's event stream after an optional emitted event ID.
    /// Unknown or cross-run cursors fail closed instead of silently replaying
    /// from genesis, which makes `Last-Event-ID` retries unambiguous.
    pub fn load_silent_session_run_events_after(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        after_event_id: Option<SilentSessionEventId>,
    ) -> anyhow::Result<Vec<SilentSessionEvent>> {
        self.load_silent_session_run_events_after_bounded(
            session_id,
            run_id,
            after_event_id,
            usize::MAX,
        )
    }

    /// Load a bounded exact-run event page after an opaque emitted event ID.
    /// Unknown and cross-run cursors fail closed rather than replaying genesis.
    pub fn load_silent_session_run_events_after_bounded(
        &self,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        after_event_id: Option<SilentSessionEventId>,
        limit: usize,
    ) -> anyhow::Result<Vec<SilentSessionEvent>> {
        anyhow::ensure!(limit > 0, "silent-session event limit must be positive");
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let after_seq = match after_event_id {
            Some(event_id) => Some(
                conn.query_row(
                    "SELECT seq FROM runtime_silent_session_events WHERE session_id=?1 AND run_id=?2 AND event_id=?3",
                    params![
                        session_id.to_string(),
                        run_id.to_string(),
                        event_id.to_string()
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .ok_or_else(|| anyhow::anyhow!("silent-session event cursor not found for exact run"))?,
            ),
            None => None,
        };
        let mut statement = conn.prepare(
            "SELECT event_json FROM runtime_silent_session_events WHERE session_id=?1 AND run_id=?2 AND seq>?3 ORDER BY seq LIMIT ?4",
        )?;
        let sql_limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = statement.query_map(
            params![
                session_id.to_string(),
                run_id.to_string(),
                after_seq.unwrap_or(0),
                sql_limit
            ],
            |row| row.get::<_, String>(0),
        )?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn load_silent_session_events(
        &self,
        session_id: SilentSessionId,
    ) -> anyhow::Result<Vec<SilentSessionEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let mut statement = conn.prepare(
            "SELECT event_json FROM runtime_silent_session_events WHERE session_id=?1 ORDER BY run_id, seq",
        )?;
        let rows = statement.query_map([session_id.to_string()], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    #[cfg(test)]
    pub(crate) fn corrupt_silent_session_event_hash_for_test(
        &self,
        event_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        conn.execute(
            "UPDATE runtime_silent_session_events SET event_hash='tampered' WHERE event_id=?1",
            [event_id],
        )?;
        Ok(())
    }

    pub fn verify_silent_session_event_chain(
        &self,
        session_id: SilentSessionId,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let mut statement = conn.prepare(
            "SELECT event_id, run_id, seq, occurred_at, payload_sha256, previous_hash, event_hash FROM runtime_silent_session_events WHERE session_id=?1 ORDER BY run_id, seq")?;
        let mut rows = statement.query([session_id.to_string()])?;
        let mut prior_run = String::new();
        let mut prior_hash = "GENESIS".to_string();
        let mut prior_seq: Option<i64> = None;
        while let Some(row) = rows.next()? {
            let (event_id, run_id, seq, occurred_at, payload_sha256, stored_previous, stored_hash):
                (String, String, i64, String, String, String, String) =
                (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?);
            if run_id != prior_run {
                prior_run = run_id;
                prior_hash = "GENESIS".into();
                prior_seq = None;
            }
            anyhow::ensure!(
                stored_previous == prior_hash,
                "silent-session event chain predecessor mismatch"
            );
            if let Some(previous_seq) = prior_seq {
                anyhow::ensure!(
                    seq == previous_seq + 1,
                    "silent-session event chain sequence gap"
                );
            }
            let expected = event_chain_hash(&prior_hash, &event_id, &occurred_at, &payload_sha256);
            anyhow::ensure!(
                stored_hash == expected,
                "silent-session event chain hash mismatch"
            );
            prior_hash = stored_hash;
            prior_seq = Some(seq);
        }
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

    /// Atomically append a validated suffix to an unscoped semantic pair stream.
    pub fn append_semantic_pair_events(
        &self,
        pair_id: &str,
        events: &[SemanticEventEnvelope],
    ) -> anyhow::Result<()> {
        if events.iter().any(|envelope| envelope.pair_id != pair_id) {
            anyhow::bail!("semantic event pair id does not match append target");
        }
        self.append_semantic_pair_events_internal(pair_id, None, events)
    }

    /// Atomically append a validated suffix under an opaque exact-scope storage
    /// key while preserving the logical pair id inside signed event envelopes.
    pub fn append_scoped_semantic_pair_events(
        &self,
        storage_key: &str,
        events: &[SemanticEventEnvelope],
    ) -> anyhow::Result<()> {
        self.append_semantic_pair_events_internal(storage_key, None, events)
    }

    pub fn append_exact_scope_semantic_pair_events(
        &self,
        storage_key: &str,
        project_root: &str,
        continuity_id: &str,
        logical_pair_id: &str,
        events: &[SemanticEventEnvelope],
    ) -> anyhow::Result<()> {
        self.append_semantic_pair_events_internal(
            storage_key,
            Some((project_root, continuity_id, logical_pair_id)),
            events,
        )
    }

    fn append_semantic_pair_events_internal(
        &self,
        storage_key: &str,
        scope_index: Option<(&str, &str, &str)>,
        events: &[SemanticEventEnvelope],
    ) -> anyhow::Result<()> {
        if storage_key.is_empty() {
            anyhow::bail!("semantic scoped storage key is required");
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let existing = load_semantic_events_from_connection(&conn, storage_key)?;
        let logical_pair_id = existing
            .first()
            .or_else(|| events.first())
            .map(|event| event.pair_id.as_str());
        if logical_pair_id.is_some_and(|pair_id| {
            existing
                .iter()
                .chain(events)
                .any(|event| event.pair_id != pair_id)
        }) {
            anyhow::bail!("semantic scoped stream contains mixed logical pair ids");
        }
        let mut candidate = existing;
        candidate.extend_from_slice(events);
        replay_semantic_events(&candidate)
            .map_err(|error| anyhow::anyhow!("semantic replay validation failed: {error}"))?;

        let tx = conn.transaction()?;
        if let Some((project_root, continuity_id, logical_pair_id)) = scope_index {
            tx.execute(
                "INSERT INTO semantic_pair_scope_index(storage_key, project_root, continuity_id, logical_pair_id, updated_at) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(storage_key) DO UPDATE SET project_root=excluded.project_root, continuity_id=excluded.continuity_id, logical_pair_id=excluded.logical_pair_id, updated_at=excluded.updated_at",
                params![storage_key, project_root, continuity_id, logical_pair_id, Utc::now().to_rfc3339()],
            )?;
        }
        for envelope in events {
            tx.execute(
                "INSERT INTO semantic_pair_events(pair_id, sequence, event_id, envelope_json, event_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    storage_key,
                    envelope.sequence as i64,
                    envelope.event_id,
                    serde_json::to_string(envelope)?,
                    envelope.hash,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_semantic_pair_events(
        &self,
        pair_id: &str,
    ) -> anyhow::Result<Vec<SemanticEventEnvelope>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let events = load_semantic_events_from_connection(&conn, pair_id)?;
        if !events.is_empty() {
            replay_semantic_events(&events)
                .map_err(|error| anyhow::anyhow!("semantic replay integrity failure: {error}"))?;
        }
        Ok(events)
    }

    pub fn list_exact_scope_semantic_pair_streams(
        &self,
        project_root: &str,
        continuity_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<(String, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let mut statement = conn.prepare(
            "SELECT storage_key, logical_pair_id FROM semantic_pair_scope_index WHERE project_root=?1 AND continuity_id=?2 ORDER BY updated_at DESC LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![project_root, continuity_id, limit.clamp(1, 100) as i64],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Store an apply receipt and migrated aggregate in one transaction. A dry
    /// run returns its truthful receipt and performs no write.
    pub fn apply_semantic_pair_migration(
        &self,
        plan: &MigrationPlan,
    ) -> anyhow::Result<MigrationReceipt> {
        if plan.receipt.dry_run {
            return Ok(plan.receipt.clone());
        }
        let receipt = plan.applied_receipt()?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let head: Option<i64> = tx
            .query_row(
                "SELECT MAX(sequence) FROM semantic_pair_events WHERE pair_id=?1",
                [&receipt.pair_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        tx.execute(
            "INSERT INTO semantic_pair_migrations(migration_id, pair_id, receipt_json, aggregate_json, event_head_sequence) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                receipt.migration_id,
                receipt.pair_id,
                serde_json::to_string(&receipt)?,
                serde_json::to_string(&plan.aggregate)?,
                head,
            ],
        )?;
        tx.commit()?;
        Ok(receipt)
    }

    pub fn load_migrated_semantic_pair(
        &self,
        pair_id: &str,
    ) -> anyhow::Result<Option<(crate::semantic_pair::SemanticPair, MigrationReceipt)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT aggregate_json, receipt_json FROM semantic_pair_migrations WHERE pair_id=?1 AND rolled_back=0 ORDER BY rowid DESC LIMIT 1",
                [pair_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        row.map(|(aggregate, receipt)| {
            Ok((
                serde_json::from_str(&aggregate)?,
                serde_json::from_str(&receipt)?,
            ))
        })
        .transpose()
    }

    /// Quarantine opaque future-version bytes. They are never decoded or
    /// admitted to replay by this runtime.
    pub fn quarantine_semantic_pair(
        &self,
        pair_id: &str,
        found_version: u32,
        payload: &[u8],
        reason: &str,
    ) -> anyhow::Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        conn.execute(
            "INSERT OR REPLACE INTO semantic_pair_quarantine(pair_id, found_version, payload, reason, quarantined_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![pair_id, found_version, payload, reason, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn semantic_pair_quarantine_version(&self, pair_id: &str) -> anyhow::Result<Option<u32>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let version: Option<i64> = conn
            .query_row(
                "SELECT found_version FROM semantic_pair_quarantine WHERE pair_id=?1",
                [pair_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(version.map(|value| value as u32))
    }

    /// Rollback is allowed only while the event head still matches the head at
    /// apply time. This prevents a migration rollback from erasing later work.
    pub fn rollback_semantic_pair_migration(
        &self,
        migration_id: &str,
        rollback_boundary: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("db lock poisoned"))?;
        let tx = conn.transaction()?;
        let row: Option<(String, String, Option<i64>, i64)> = tx
            .query_row(
                "SELECT pair_id, receipt_json, event_head_sequence, rolled_back FROM semantic_pair_migrations WHERE migration_id=?1",
                [migration_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let (pair_id, receipt_json, apply_head, rolled_back) =
            row.ok_or_else(|| anyhow::anyhow!("semantic migration receipt not found"))?;
        let receipt: MigrationReceipt = serde_json::from_str(&receipt_json)?;
        if rolled_back != 0 || receipt.rollback_boundary != rollback_boundary {
            anyhow::bail!("semantic migration rollback boundary mismatch");
        }
        let current_head: Option<i64> = tx
            .query_row(
                "SELECT MAX(sequence) FROM semantic_pair_events WHERE pair_id=?1",
                [&pair_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        if current_head != apply_head {
            anyhow::bail!("semantic migration rollback blocked by later events");
        }
        tx.execute(
            "UPDATE semantic_pair_migrations SET rolled_back=1 WHERE migration_id=?1",
            [migration_id],
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn load_semantic_events_from_connection(
    conn: &Connection,
    pair_id: &str,
) -> anyhow::Result<Vec<SemanticEventEnvelope>> {
    let mut statement = conn.prepare(
        "SELECT envelope_json FROM semantic_pair_events WHERE pair_id=?1 ORDER BY sequence",
    )?;
    let rows = statement.query_map([pair_id], |row| row.get::<_, String>(0))?;
    rows.map(|row| {
        let json = row?;
        serde_json::from_str(&json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    })
    .collect::<Result<Vec<_>, _>>()
    .map_err(Into::into)
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

#[cfg(test)]
mod trajectory_ladder_ledger_tests {
    use super::*;
    use crate::types::{TrajectoryConfidence, TrajectoryLadderEventKind, TrajectoryLadderLevel};

    fn test_persistence() -> (SqlitePersistence, PathBuf) {
        let root = std::env::temp_dir().join(format!("focusa-ladder-ledger-{}", Uuid::now_v7()));
        let config = FocusaConfig {
            data_dir: root.display().to_string(),
            ..FocusaConfig::default()
        };
        let persistence = SqlitePersistence::new(&config).expect("test persistence");
        (persistence, root)
    }

    fn event(
        project_root: &str,
        continuity_id: &str,
        event_id: &str,
        level: TrajectoryLadderLevel,
        lamport_ts: u64,
    ) -> TrajectoryLadderEvent {
        TrajectoryLadderEvent {
            schema_version: TrajectoryLadderEvent::SCHEMA_VERSION.to_string(),
            event_id: event_id.to_string(),
            trajectory_id: "trajectory:test".to_string(),
            project_root: project_root.to_string(),
            continuity_id: Some(continuity_id.to_string()),
            session_id: None,
            hlt_version: 1,
            causal_parent_event_id: None,
            event_kind: TrajectoryLadderEventKind::Committed,
            level,
            object_id: None,
            old_value: serde_json::Value::Null,
            new_value: serde_json::json!("value"),
            actor: "test".to_string(),
            source: "test".to_string(),
            authority: "canonical_explicit".to_string(),
            provenance: "test".to_string(),
            confidence: TrajectoryConfidence::High,
            reason: None,
            evidence_refs: vec![],
            idempotency_key: Some(event_id.to_string()),
            lamport_ts,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn ladder_ledger_is_scope_bounded_and_deduplicates_legacy_hlt_projection() {
        let (persistence, root) = test_persistence();
        let project_root = "/projects/focusa";
        let legacy = HltLedgerEntry::new(project_root.to_string(), "HLT".to_string(), "test", 1)
            .with_scope(Some("continuity:a".to_string()), None);
        persistence
            .append_hlt_ledger_entry(&legacy)
            .expect("append legacy HLT");
        let canonical_hlt = event(
            project_root,
            "continuity:a",
            &format!("legacy-hlt:{}", legacy.event_id),
            TrajectoryLadderLevel::Hlt,
            1,
        );
        let waypoint = event(
            project_root,
            "continuity:a",
            "event:waypoint",
            TrajectoryLadderLevel::Waypoint,
            2,
        );
        persistence
            .append_trajectory_ladder_events(&[canonical_hlt, waypoint])
            .expect("append Ladder batch");

        let events = persistence
            .read_trajectory_ladder_events(project_root, Some("continuity:a"), 50)
            .expect("read Ladder");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].level, TrajectoryLadderLevel::Hlt);
        assert_eq!(events[1].level, TrajectoryLadderLevel::Waypoint);
        assert!(
            persistence
                .read_trajectory_ladder_events(project_root, Some("continuity:b"), 50)
                .expect("other continuity")
                .is_empty()
        );
        drop(persistence);
        std::fs::remove_dir_all(root).expect("clean test data");
    }

    #[test]
    fn ladder_ledger_rejects_cross_project_batch() {
        let (persistence, root) = test_persistence();
        let first = event(
            "/projects/a",
            "continuity:a",
            "event:a",
            TrajectoryLadderLevel::Hlt,
            1,
        );
        let second = event(
            "/projects/b",
            "continuity:a",
            "event:b",
            TrajectoryLadderLevel::Mlg,
            2,
        );
        let error = persistence
            .append_trajectory_ladder_events(&[first, second])
            .expect_err("cross-project batch must fail");
        assert!(error.to_string().contains("crosses project scope"));
        drop(persistence);
        std::fs::remove_dir_all(root).expect("clean test data");
    }
}
