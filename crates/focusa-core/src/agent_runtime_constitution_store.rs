//! Spec 140 SQLite persistence for runtime-constitution facts and events.

use crate::agent_runtime_constitution::RuntimeConstitutionEvent;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RUNTIME_CONSTITUTION_SCHEMA_VERSION: i64 = 1;

pub const RUNTIME_CONSTITUTION_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS runtime_constitution_schema_meta (
  singleton INTEGER PRIMARY KEY CHECK(singleton = 1), schema_version INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS runtime_constitutions (
  constitution_id TEXT NOT NULL, version TEXT NOT NULL, project_ref TEXT NOT NULL,
  lifecycle TEXT NOT NULL, payload_json TEXT NOT NULL, content_sha256 TEXT NOT NULL,
  created_at TEXT NOT NULL, PRIMARY KEY(constitution_id, version)
);
CREATE TABLE IF NOT EXISTS instruction_sources (
  source_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL, content_sha256 TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS instruction_claims (
  claim_id TEXT PRIMARY KEY, source_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS instruction_conflicts (
  conflict_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS instruction_resolutions (
  resolution_id TEXT PRIMARY KEY, conflict_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS operating_contracts (
  contract_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS prompt_assembly_plans (
  plan_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS prompt_variants (
  variant_id TEXT PRIMARY KEY, plan_id TEXT NOT NULL, target TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS prompt_evaluations (
  evaluation_id TEXT PRIMARY KEY, variant_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS skill_activation_plans (
  plan_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tool_routing_plans (
  plan_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS enforcement_plans (
  plan_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS validation_matrices (
  matrix_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS contract_impact_assessments (
  assessment_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS delivery_manifests (
  manifest_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS runtime_constitution_events (
  event_id TEXT PRIMARY KEY, constitution_id TEXT NOT NULL, sequence INTEGER NOT NULL,
  idempotency_key TEXT NOT NULL, kind TEXT NOT NULL, payload_json TEXT NOT NULL,
  previous_event_hash TEXT, event_hash TEXT NOT NULL, occurred_at TEXT NOT NULL,
  UNIQUE(constitution_id, sequence), UNIQUE(constitution_id, idempotency_key),
  UNIQUE(constitution_id, event_hash)
);
CREATE INDEX IF NOT EXISTS runtime_constitution_events_order
  ON runtime_constitution_events(constitution_id, sequence, event_hash);
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRuntimeConstitutionEvent {
    pub event_id: String,
    pub constitution_id: String,
    pub sequence: i64,
    pub idempotency_key: String,
    pub kind: String,
    pub payload_json: String,
    pub previous_event_hash: Option<String>,
    pub event_hash: String,
    pub occurred_at: String,
}

pub fn migrate_runtime_constitution_schema(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute_batch(RUNTIME_CONSTITUTION_SCHEMA_SQL)?;
    transaction.execute(
        "INSERT INTO runtime_constitution_schema_meta(singleton,schema_version) VALUES(1,?1) \
         ON CONFLICT(singleton) DO UPDATE SET schema_version=excluded.schema_version",
        [RUNTIME_CONSTITUTION_SCHEMA_VERSION],
    )?;
    transaction.commit()?;
    Ok(())
}

pub fn append_runtime_constitution_event(
    connection: &mut Connection,
    event_id: &str,
    constitution_id: &str,
    idempotency_key: &str,
    event: &RuntimeConstitutionEvent,
) -> Result<StoredRuntimeConstitutionEvent> {
    let transaction = connection.transaction()?;
    if let Some(existing) = transaction
        .query_row(
            "SELECT event_id,constitution_id,sequence,idempotency_key,kind,payload_json,previous_event_hash,event_hash,occurred_at \
             FROM runtime_constitution_events WHERE constitution_id=?1 AND idempotency_key=?2",
            params![constitution_id, idempotency_key],
            row_to_event,
        )
        .optional()?
    {
        transaction.commit()?;
        return Ok(existing);
    }
    let previous = transaction
        .query_row(
            "SELECT sequence,event_hash FROM runtime_constitution_events WHERE constitution_id=?1 ORDER BY sequence DESC LIMIT 1",
            [constitution_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    let sequence = previous.as_ref().map_or(1, |(value, _)| value + 1);
    let previous_event_hash = previous.map(|(_, hash)| hash);
    let payload_json = serde_json::to_string(event)?;
    let kind = event.event_name().to_string();
    let occurred_at = Utc::now().to_rfc3339();
    let event_hash = calculate_hash(
        constitution_id,
        sequence,
        idempotency_key,
        &kind,
        &payload_json,
        previous_event_hash.as_deref(),
    );
    transaction.execute(
        "INSERT INTO runtime_constitution_events(event_id,constitution_id,sequence,idempotency_key,kind,payload_json,previous_event_hash,event_hash,occurred_at) \
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![event_id, constitution_id, sequence, idempotency_key, kind, payload_json, previous_event_hash, event_hash, occurred_at],
    )?;
    let stored = transaction
        .query_row(
            "SELECT event_id,constitution_id,sequence,idempotency_key,kind,payload_json,previous_event_hash,event_hash,occurred_at \
             FROM runtime_constitution_events WHERE event_id=?1",
            [event_id],
            row_to_event,
        )
        .with_context(|| format!("read appended runtime constitution event {event_id}"))?;
    transaction.commit()?;
    Ok(stored)
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredRuntimeConstitutionEvent> {
    Ok(StoredRuntimeConstitutionEvent {
        event_id: row.get(0)?,
        constitution_id: row.get(1)?,
        sequence: row.get(2)?,
        idempotency_key: row.get(3)?,
        kind: row.get(4)?,
        payload_json: row.get(5)?,
        previous_event_hash: row.get(6)?,
        event_hash: row.get(7)?,
        occurred_at: row.get(8)?,
    })
}

fn calculate_hash(
    constitution_id: &str,
    sequence: i64,
    idempotency_key: &str,
    kind: &str,
    payload_json: &str,
    previous_event_hash: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    for value in [
        constitution_id,
        &sequence.to_string(),
        idempotency_key,
        kind,
        payload_json,
        previous_event_hash.unwrap_or("GENESIS"),
    ] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hex::encode(hasher.finalize())
}
