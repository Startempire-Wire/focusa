use std::{fs, path::PathBuf};

use anyhow::Context;
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::runtime::persistence_sqlite::SqlitePersistence;

use super::{SilentSession, SilentSessionConfigRevision, SilentSessionEvent, SilentSessionRun};

pub const SILENT_SESSION_DB_SCHEMA_VERSION: i64 = 5;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS silent_session_control_schema_meta (
  version INTEGER NOT NULL,
  migrated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS silent_session_controls (
  silent_session_id TEXT PRIMARY KEY,
  project_root TEXT NOT NULL,
  continuity_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  work_item_ref TEXT,
  mission TEXT NOT NULL,
  active_config_revision_id TEXT NOT NULL,
  current_run_generation INTEGER NOT NULL CHECK(current_run_generation > 0),
  lifecycle TEXT NOT NULL,
  health TEXT NOT NULL,
  semantic_activity TEXT NOT NULL,
  snapshot_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_silent_session_controls_authority
  ON silent_session_controls(project_root, continuity_id, updated_at);
CREATE TABLE IF NOT EXISTS silent_session_daemon_runs (
  run_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_generation INTEGER NOT NULL CHECK(run_generation > 0),
  actor_instance_id TEXT NOT NULL,
  config_revision_id TEXT NOT NULL,
  protocol_versions_json TEXT NOT NULL,
  run_json TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  UNIQUE(silent_session_id, run_generation)
);
CREATE INDEX IF NOT EXISTS idx_silent_session_daemon_runs_session
  ON silent_session_daemon_runs(silent_session_id, run_generation);
CREATE TABLE IF NOT EXISTS silent_session_control_config_revisions (
  config_revision_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  revision INTEGER NOT NULL CHECK(revision > 0),
  config_schema_version INTEGER NOT NULL,
  config_json TEXT NOT NULL,
  redacted_config_hash TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(silent_session_id, revision)
);
CREATE TABLE IF NOT EXISTS silent_session_control_events (
  event_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT,
  sequence INTEGER NOT NULL CHECK(sequence > 0),
  event_schema_version INTEGER NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  previous_event_hash TEXT,
  event_hash TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  UNIQUE(silent_session_id, sequence),
  UNIQUE(silent_session_id, idempotency_key),
  UNIQUE(silent_session_id, event_hash)
);
CREATE INDEX IF NOT EXISTS idx_silent_session_control_events_chain
  ON silent_session_control_events(silent_session_id, sequence, event_hash);
CREATE TABLE IF NOT EXISTS silent_session_control_stream_indexes (
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT NOT NULL,
  stream_name TEXT NOT NULL,
  chunk_sequence INTEGER NOT NULL,
  chunk_ref TEXT NOT NULL,
  byte_start INTEGER NOT NULL,
  byte_end INTEGER NOT NULL,
  chunk_hash TEXT NOT NULL,
  codec_version INTEGER NOT NULL,
  first_event_sequence INTEGER NOT NULL,
  last_event_sequence INTEGER NOT NULL,
  event_count INTEGER NOT NULL,
  uncompressed_bytes INTEGER NOT NULL,
  compressed_bytes INTEGER NOT NULL,
  redaction_applied INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY(silent_session_id, run_id, stream_name, chunk_sequence)
);
CREATE TABLE IF NOT EXISTS silent_session_control_checkpoints (
  checkpoint_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT,
  checkpoint_kind TEXT NOT NULL,
  event_sequence INTEGER NOT NULL,
  checkpoint_json TEXT NOT NULL,
  checkpoint_hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_silent_session_control_checkpoints_latest
  ON silent_session_control_checkpoints(silent_session_id, run_id, event_sequence DESC);
CREATE TABLE IF NOT EXISTS silent_session_control_leases (
  lease_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT NOT NULL,
  owner_actor_instance_id TEXT NOT NULL,
  fencing_token INTEGER NOT NULL CHECK(fencing_token > 0),
  status TEXT NOT NULL,
  lease_json TEXT NOT NULL,
  issued_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  UNIQUE(silent_session_id, fencing_token)
);
CREATE INDEX IF NOT EXISTS idx_silent_session_control_leases_active
  ON silent_session_control_leases(silent_session_id, status, expires_at);
CREATE TABLE IF NOT EXISTS silent_session_control_notifications (
  notification_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  event_id TEXT,
  notification_type TEXT NOT NULL,
  channel TEXT NOT NULL,
  status TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  delivered_at TEXT
);
CREATE TABLE IF NOT EXISTS silent_session_control_completion_evaluations (
  completion_evaluation_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT NOT NULL,
  decision TEXT NOT NULL,
  evaluation_json TEXT NOT NULL,
  receipt_ready INTEGER NOT NULL,
  evaluated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_silent_session_completion_latest
  ON silent_session_control_completion_evaluations(silent_session_id, evaluated_at DESC);
CREATE TABLE IF NOT EXISTS silent_session_control_backend_bindings (
  binding_id TEXT PRIMARY KEY,
  silent_session_id TEXT NOT NULL REFERENCES silent_session_controls(silent_session_id),
  run_id TEXT NOT NULL,
  backend_kind TEXT NOT NULL,
  backend_identity TEXT NOT NULL,
  protocol_version INTEGER NOT NULL,
  binding_json TEXT NOT NULL,
  bound_at TEXT NOT NULL,
  released_at TEXT,
  UNIQUE(silent_session_id, run_id, backend_kind)
);
CREATE TABLE IF NOT EXISTS silent_session_control_principals (
  principal_id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  os_user TEXT NOT NULL,
  role TEXT NOT NULL,
  principal_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS silent_session_control_approvals (
  approval_id TEXT PRIMARY KEY,
  operator_actor TEXT NOT NULL,
  action TEXT NOT NULL,
  project_root TEXT NOT NULL,
  continuity_id TEXT NOT NULL,
  session_id TEXT,
  run_id TEXT,
  action_digest TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  approval_json TEXT NOT NULL,
  issuance_idempotency_key TEXT,
  issuance_request_hash TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_silent_session_control_approvals_idempotency
  ON silent_session_control_approvals(operator_actor,issuance_idempotency_key)
  WHERE issuance_idempotency_key IS NOT NULL AND issuance_idempotency_key <> '';
CREATE INDEX IF NOT EXISTS idx_silent_session_control_approvals_scope
  ON silent_session_control_approvals(project_root,continuity_id,expires_at);
CREATE TABLE IF NOT EXISTS silent_session_control_audits (
  audit_id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  project_root TEXT NOT NULL,
  continuity_id TEXT NOT NULL,
  session_id TEXT,
  run_id TEXT,
  audit_json TEXT NOT NULL,
  occurred_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS silent_session_control_runner_nonces (
  runner_principal_id TEXT NOT NULL,
  nonce TEXT NOT NULL,
  command_id TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  consumed_at TEXT NOT NULL,
  PRIMARY KEY(runner_principal_id,nonce)
);
CREATE TABLE IF NOT EXISTS silent_session_control_retention (
  session_id TEXT PRIMARY KEY REFERENCES silent_session_controls(silent_session_id) ON DELETE CASCADE,
  evidence_hold INTEGER NOT NULL DEFAULT 0,
  hold_reason TEXT,
  hold_expires_at TEXT,
  deleted_at TEXT,
  delete_reason TEXT,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS silent_session_control_retention_operations (
  session_id TEXT NOT NULL,
  action TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  response_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY(session_id,action,principal_id,idempotency_key)
);
"#;

const MIGRATION_V2_SQL: &str = r#"
ALTER TABLE silent_session_daemon_runs ADD COLUMN run_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE silent_session_control_leases ADD COLUMN lease_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN codec_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN first_event_sequence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN last_event_sequence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN event_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN uncompressed_bytes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN compressed_bytes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE silent_session_control_stream_indexes ADD COLUMN redaction_applied INTEGER NOT NULL DEFAULT 1;
"#;

const MIGRATION_V5_SQL: &str = r#"
ALTER TABLE silent_session_control_approvals ADD COLUMN issuance_idempotency_key TEXT;
ALTER TABLE silent_session_control_approvals ADD COLUMN issuance_request_hash TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_silent_session_control_approvals_idempotency
  ON silent_session_control_approvals(operator_actor,issuance_idempotency_key)
  WHERE issuance_idempotency_key IS NOT NULL AND issuance_idempotency_key <> '';
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationMode {
    Apply,
    DryRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationOutcome {
    pub previous_version: i64,
    pub target_version: i64,
    pub applied: bool,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendOutcome {
    Appended,
    Replayed,
}

pub fn migrate_silent_session_schema(
    persistence: &SqlitePersistence,
    mode: MigrationMode,
) -> anyhow::Result<MigrationOutcome> {
    let db_path = persistence.data_dir.join("focusa.sqlite");
    persistence.with_connection_mut(|connection| {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let meta_exists: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='silent_session_control_schema_meta'",
            [],
            |row| row.get(0),
        )?;
        let previous_version = if meta_exists == 1 {
            connection.query_row(
                "SELECT version FROM silent_session_control_schema_meta LIMIT 1",
                [],
                |row| row.get::<_, i64>(0),
            )?
        } else {
            0
        };
        if previous_version > SILENT_SESSION_DB_SCHEMA_VERSION {
            anyhow::bail!("unsupported silent session schema version {previous_version}");
        }
        if previous_version == SILENT_SESSION_DB_SCHEMA_VERSION {
            verify_schema(connection)?;
            return Ok(MigrationOutcome {
                previous_version,
                target_version: SILENT_SESSION_DB_SCHEMA_VERSION,
                applied: false,
                backup_path: None,
            });
        }

        let backup_path = if mode == MigrationMode::Apply && db_path.exists() {
            connection.execute_batch("PRAGMA wal_checkpoint(FULL);")?;
            let path = persistence.data_dir.join(format!(
                "focusa.sqlite.pre-silent-session-v{}.backup",
                SILENT_SESSION_DB_SCHEMA_VERSION
            ));
            fs::copy(&db_path, &path)
                .with_context(|| format!("back up SQLite database to {}", path.display()))?;
            Some(path)
        } else {
            None
        };

        let transaction = connection.transaction()?;
        transaction.execute_batch(SCHEMA_SQL)?;
        if previous_version == 1 {
            transaction.execute_batch(MIGRATION_V2_SQL)?;
        }
        if (1..5).contains(&previous_version) {
            transaction.execute_batch(MIGRATION_V5_SQL)?;
        }
        transaction.execute("DELETE FROM silent_session_control_schema_meta", [])?;
        transaction.execute(
            "INSERT INTO silent_session_control_schema_meta(version, migrated_at) VALUES (?1, ?2)",
            params![SILENT_SESSION_DB_SCHEMA_VERSION, Utc::now().to_rfc3339()],
        )?;
        verify_schema(&transaction)?;
        if mode == MigrationMode::Apply {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }
        Ok(MigrationOutcome {
            previous_version,
            target_version: SILENT_SESSION_DB_SCHEMA_VERSION,
            applied: mode == MigrationMode::Apply,
            backup_path,
        })
    })
}

pub fn append_reducer_event_and_project(
    persistence: &SqlitePersistence,
    event: &mut SilentSessionEvent,
    projection: &SilentSession,
) -> anyhow::Result<AppendOutcome> {
    append_event_projection_and_revision(persistence, event, projection, None, None, None)
}

pub fn append_create_event_and_project(
    persistence: &SqlitePersistence,
    event: &mut SilentSessionEvent,
    projection: &SilentSession,
    revision: &SilentSessionConfigRevision,
    run: &SilentSessionRun,
) -> anyhow::Result<AppendOutcome> {
    if revision.silent_session_id != projection.id
        || projection.active_config_revision_id != revision.id
    {
        anyhow::bail!("config revision does not match the created session projection");
    }
    if run.silent_session_id != projection.id
        || run.config_revision_id != revision.id
        || run.generation != projection.current_run_generation
        || event.run_id != Some(run.id)
    {
        anyhow::bail!("initial run does not match the created session projection");
    }
    append_event_projection_and_revision(
        persistence,
        event,
        projection,
        Some(revision),
        Some(run),
        None,
    )
}

pub fn append_config_revision_event_and_project(
    persistence: &SqlitePersistence,
    event: &mut SilentSessionEvent,
    projection: &SilentSession,
    revision: &SilentSessionConfigRevision,
) -> anyhow::Result<AppendOutcome> {
    if revision.silent_session_id != projection.id || event.silent_session_id != projection.id {
        anyhow::bail!("config revision does not match the session projection");
    }
    append_event_projection_and_revision(persistence, event, projection, Some(revision), None, None)
}

pub fn append_restart_event_and_project(
    persistence: &SqlitePersistence,
    event: &mut SilentSessionEvent,
    projection: &SilentSession,
    previous_run: &SilentSessionRun,
    next_run: &SilentSessionRun,
) -> anyhow::Result<AppendOutcome> {
    if previous_run.silent_session_id != projection.id
        || next_run.silent_session_id != projection.id
        || previous_run.ended_at.is_none()
        || previous_run.generation.next()? != next_run.generation
        || next_run.generation != projection.current_run_generation
        || next_run.config_revision_id != projection.active_config_revision_id
        || event.run_id != Some(next_run.id)
    {
        anyhow::bail!("restart runs do not match the rolled-over session projection");
    }
    append_event_projection_and_revision(
        persistence,
        event,
        projection,
        None,
        Some(next_run),
        Some(previous_run),
    )
}

fn append_event_projection_and_revision(
    persistence: &SqlitePersistence,
    event: &mut SilentSessionEvent,
    projection: &SilentSession,
    revision: Option<&SilentSessionConfigRevision>,
    run_to_insert: Option<&SilentSessionRun>,
    run_to_update: Option<&SilentSessionRun>,
) -> anyhow::Result<AppendOutcome> {
    if event.silent_session_id != projection.id {
        anyhow::bail!("event and projection silent_session_id mismatch");
    }
    if event.sequence == 0 || event.idempotency_key.trim().is_empty() {
        anyhow::bail!("event sequence and idempotency key must be valid");
    }

    persistence.with_connection_mut(|connection| {
        let transaction = connection.transaction()?;
        let replayed = transaction
            .query_row(
                "SELECT event_hash,kind,payload_json FROM silent_session_control_events WHERE silent_session_id=?1 AND idempotency_key=?2",
                params![projection.id.to_string(), event.idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        if let Some((existing_hash, existing_kind, existing_payload)) = replayed {
            if event.kind != existing_kind
                || serde_json::to_string(&event.payload)? != existing_payload
            {
                anyhow::bail!("idempotency key reused with different event content");
            }
            event.event_hash = existing_hash;
            transaction.rollback()?;
            return Ok(AppendOutcome::Replayed);
        }

        let chain_head = transaction
            .query_row(
                "SELECT sequence,event_hash FROM silent_session_control_events WHERE silent_session_id=?1 ORDER BY sequence DESC LIMIT 1",
                [projection.id.to_string()],
                |row| Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let (expected_sequence, expected_previous) = chain_head
            .map(|(sequence, hash)| (sequence + 1, Some(hash)))
            .unwrap_or((1, None));
        if event.sequence != expected_sequence || event.previous_event_hash != expected_previous {
            anyhow::bail!("event sequence or previous hash does not match the canonical chain head");
        }
        event.event_hash = calculate_event_hash(event)?;

        upsert_projection(&transaction, projection)?;
        if let Some(revision) = revision {
            transaction.execute(
                r#"INSERT INTO silent_session_control_config_revisions(
                   config_revision_id,silent_session_id,revision,config_schema_version,config_json,
                   redacted_config_hash,created_by,created_at
                   ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                params![
                    revision.id.to_string(),
                    revision.silent_session_id.to_string(),
                    revision.revision,
                    revision.config_schema_version,
                    serde_json::to_string(&revision.config)?,
                    revision.redacted_config_hash,
                    revision.created_by.to_string(),
                    revision.created_at.to_rfc3339(),
                ],
            )?;
        }
        if let Some(run) = run_to_update {
            let changed = transaction.execute(
                "UPDATE silent_session_daemon_runs SET run_json=?1,ended_at=?2 WHERE run_id=?3",
                params![
                    serde_json::to_string(run)?,
                    run.ended_at.map(|value| value.to_rfc3339()),
                    run.id.to_string(),
                ],
            )?;
            if changed != 1 {
                anyhow::bail!("previous restart run does not exist");
            }
        }
        if let Some(run) = run_to_insert {
            transaction.execute(
                r#"INSERT INTO silent_session_daemon_runs(
                   run_id,silent_session_id,run_generation,actor_instance_id,config_revision_id,
                   protocol_versions_json,run_json,started_at,ended_at
                   ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
                params![
                    run.id.to_string(),
                    run.silent_session_id.to_string(),
                    run.generation.get(),
                    run.actor_instance_id.to_string(),
                    run.config_revision_id.to_string(),
                    serde_json::to_string(&run.protocol_versions)?,
                    serde_json::to_string(run)?,
                    run.started_at.to_rfc3339(),
                    run.ended_at.map(|value| value.to_rfc3339()),
                ],
            )?;
        }
        transaction.execute(
            r#"INSERT INTO silent_session_control_events(
              event_id,silent_session_id,run_id,sequence,event_schema_version,kind,payload_json,
              idempotency_key,previous_event_hash,event_hash,occurred_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
            params![
                event.id.to_string(),
                event.silent_session_id.to_string(),
                event.run_id.map(|id| id.to_string()),
                event.sequence,
                event.event_schema_version,
                event.kind,
                serde_json::to_string(&event.payload)?,
                event.idempotency_key,
                event.previous_event_hash,
                event.event_hash,
                event.occurred_at.to_rfc3339(),
            ],
        )?;
        transaction.commit()?;
        Ok(AppendOutcome::Appended)
    })
}

fn upsert_projection(
    transaction: &rusqlite::Transaction<'_>,
    projection: &SilentSession,
) -> anyhow::Result<()> {
    let changed = transaction.execute(
        r#"INSERT INTO silent_session_controls(
          silent_session_id,project_root,continuity_id,display_name,work_item_ref,mission,
          active_config_revision_id,current_run_generation,lifecycle,health,semantic_activity,
          snapshot_json,created_at,updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
        ON CONFLICT(silent_session_id) DO UPDATE SET
          display_name=excluded.display_name,work_item_ref=excluded.work_item_ref,
          mission=excluded.mission,active_config_revision_id=excluded.active_config_revision_id,
          current_run_generation=excluded.current_run_generation,lifecycle=excluded.lifecycle,
          health=excluded.health,semantic_activity=excluded.semantic_activity,
          snapshot_json=excluded.snapshot_json,updated_at=excluded.updated_at
        WHERE silent_session_controls.project_root=excluded.project_root
          AND silent_session_controls.continuity_id=excluded.continuity_id"#,
        params![
            projection.id.to_string(),
            projection.authority.project_root,
            projection.authority.continuity_id,
            projection.display_name,
            projection.work_item_ref,
            projection.mission,
            projection.active_config_revision_id.to_string(),
            projection.current_run_generation.get(),
            enum_json(projection.lifecycle)?,
            enum_json(projection.health)?,
            enum_json(projection.semantic_activity)?,
            serde_json::to_string(projection)?,
            projection.created_at.to_rfc3339(),
            projection.updated_at.to_rfc3339(),
        ],
    )?;
    if changed != 1 {
        anyhow::bail!("session projection authority does not match canonical scope");
    }
    Ok(())
}

fn calculate_event_hash(event: &SilentSessionEvent) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(event.silent_session_id.to_string());
    hasher.update(b"\n");
    hasher.update(event.sequence.to_string());
    hasher.update(b"\n");
    hasher.update(event.previous_event_hash.as_deref().unwrap_or("GENESIS"));
    hasher.update(b"\n");
    hasher.update(&event.kind);
    hasher.update(b"\n");
    hasher.update(serde_json::to_vec(&event.payload)?);
    hasher.update(b"\n");
    hasher.update(&event.idempotency_key);
    Ok(hex::encode(hasher.finalize()))
}

fn enum_json<T: Serialize>(value: T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(&value)?.trim_matches('"').to_string())
}

fn verify_schema(connection: &rusqlite::Connection) -> anyhow::Result<()> {
    const REQUIRED_TABLES: [&str; 16] = [
        "silent_session_controls",
        "silent_session_daemon_runs",
        "silent_session_control_config_revisions",
        "silent_session_control_events",
        "silent_session_control_stream_indexes",
        "silent_session_control_checkpoints",
        "silent_session_control_leases",
        "silent_session_control_notifications",
        "silent_session_control_completion_evaluations",
        "silent_session_control_backend_bindings",
        "silent_session_control_principals",
        "silent_session_control_approvals",
        "silent_session_control_audits",
        "silent_session_control_runner_nonces",
        "silent_session_control_retention",
        "silent_session_control_retention_operations",
    ];
    for table in REQUIRED_TABLES {
        let exists: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )?;
        if exists != 1 {
            anyhow::bail!("silent session migration missing table {table}");
        }
    }
    for (table, column) in [
        ("silent_session_daemon_runs", "run_json"),
        ("silent_session_control_leases", "lease_json"),
        ("silent_session_control_stream_indexes", "codec_version"),
        (
            "silent_session_control_stream_indexes",
            "last_event_sequence",
        ),
        ("silent_session_control_stream_indexes", "redaction_applied"),
    ] {
        let exists: i64 = connection.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name=?2",
            params![table, column],
            |row| row.get(0),
        )?;
        if exists != 1 {
            anyhow::bail!("silent session migration missing column {table}.{column}");
        }
    }
    Ok(())
}
