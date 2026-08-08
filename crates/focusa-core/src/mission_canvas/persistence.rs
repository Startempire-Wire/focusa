use std::{path::Path, sync::Arc};

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::layout_mutation::{LayoutMutationResult, LAYOUT_MUTATE_OPERATION};
use super::memory::{layout_memory_digest, validate_profile_layout_memory, ProfileLayoutMemory};
use super::model::{
    CompositionEvent, DomainPackInstallReceipt, GovernedDomainPackInstallReceipt,
    MissionCanvasScope, OmissionDiagnostic, ResolvedWorkspaceProjection, StoredDocument,
};
use super::profiles::DomainPack;
use super::reducer::{projection_digest, RecompositionEvidence, RecompositionReceipt};
use super::LAYOUT_MEMORY_UPDATE_OPERATION;
use crate::workstream_identity::WorkstreamKey;

const DOCUMENT_TABLES: &[&str] = &[
    "mission_canvas_profiles",
    "mission_canvas_activity_modes",
    "mission_canvas_registry_entries",
    "mission_canvas_layout_trees",
    "mission_canvas_layout_memory",
    "mission_canvas_drafts",
    "mission_canvas_host_lifecycle",
];

/// Every Mission Canvas row is partitioned by this exact Workstream owner.
/// Subordinate Attachment/Session/Surface values remain in the typed payload
/// and are validated at the read/write boundary; they are never used as a
/// fallback partition key.
const SCOPED_TABLES: &[&str] = &[
    "mission_canvas_profiles",
    "mission_canvas_activity_modes",
    "mission_canvas_registry_entries",
    "mission_canvas_layout_trees",
    "mission_canvas_layout_memory",
    "mission_canvas_drafts",
    "mission_canvas_host_lifecycle",
    "mission_canvas_projections",
    "mission_canvas_composition_events",
    "mission_canvas_domain_pack_installations",
];

#[derive(Debug, Error)]
pub enum MissionCanvasStoreError {
    #[error("mission canvas I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mission canvas SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("mission canvas serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("mission canvas scope validation failed: {0}")]
    InvalidScope(&'static str),
    #[error("mission canvas projection validation failed: {0}")]
    InvalidProjection(&'static str),
    #[error("mission canvas data belongs to a different Workstream")]
    WorkstreamMismatch,
    #[error("mission canvas projection and event revisions do not match")]
    ProjectionEventRevisionMismatch,
    #[error("unknown mission canvas document table: {0}")]
    UnknownTable(String),
    #[error("revision conflict: expected {expected}, observed {observed}")]
    RevisionConflict { expected: u64, observed: u64 },
    #[error("layout-memory idempotency key conflicts with an existing request")]
    LayoutMemoryIdempotencyConflict,
    #[error("layout mutation idempotency key conflicts with an existing request")]
    LayoutMutationIdempotencyConflict,
    #[error("layout-memory document is invalid: {0}")]
    InvalidLayoutMemory(&'static str),
    #[error("document already exists at revision {0}")]
    AlreadyExists(u64),
    #[error("invalid host lifecycle document: {0}")]
    InvalidHostLifecycleDocument(String),
    #[error("domain pack already installed: {0}")]
    DomainPackAlreadyInstalled(String),
    #[error("domain pack idempotency key conflicts with an existing request")]
    DomainPackIdempotencyConflict,
    #[error("domain pack document already exists: {0}")]
    DomainPackDocumentAlreadyExists(String),
}

pub type Result<T> = std::result::Result<T, MissionCanvasStoreError>;

#[derive(Clone)]
pub struct MissionCanvasStore {
    connection: Arc<Mutex<Connection>>,
}

impl MissionCanvasStore {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        std::fs::create_dir_all(data_dir.as_ref())?;
        let connection = Connection::open(data_dir.as_ref().join("mission-canvas.sqlite3"))?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let store = Self {
            connection: Arc::new(Mutex::new(Connection::open_in_memory()?)),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn migrate(&self) -> Result<()> {
        let connection = self.connection.lock();
        connection.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS mission_canvas_schema_version (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                version INTEGER NOT NULL
            );
            INSERT INTO mission_canvas_schema_version(singleton, version)
            VALUES (1, 1)
            ON CONFLICT(singleton) DO UPDATE SET version = MAX(version, excluded.version);

            CREATE TABLE IF NOT EXISTS mission_canvas_profiles (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_activity_modes (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_registry_entries (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_layout_trees (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_layout_memory (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_drafts (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_host_lifecycle (
                scope_key TEXT NOT NULL,
                document_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0),
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, document_id)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_projections (
                scope_key TEXT PRIMARY KEY,
                projection_revision INTEGER NOT NULL CHECK (projection_revision >= 0),
                layout_revision INTEGER NOT NULL CHECK (layout_revision >= 0),
                projection_digest TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_composition_events (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                scope_key TEXT NOT NULL,
                event_kind TEXT NOT NULL,
                projection_revision INTEGER NOT NULL CHECK (projection_revision >= 0),
                layout_revision INTEGER NOT NULL CHECK (layout_revision >= 0),
                causation_id TEXT,
                correlation_id TEXT,
                occurred_at TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                evidence_refs_json TEXT NOT NULL,
                receipt_refs_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_mission_canvas_events_scope_sequence
                ON mission_canvas_composition_events(scope_key, sequence);
            CREATE TABLE IF NOT EXISTS mission_canvas_domain_pack_installations (
                scope_key TEXT NOT NULL,
                pack_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                request_digest TEXT NOT NULL,
                receipt_ref TEXT NOT NULL,
                receipt_json TEXT NOT NULL,
                event_cursor TEXT NOT NULL,
                authority_ref TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                PRIMARY KEY(scope_key, pack_id),
                UNIQUE(scope_key, idempotency_key)
            );
            CREATE TABLE IF NOT EXISTS mission_canvas_legacy_quarantine (
                quarantine_id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_table TEXT NOT NULL,
                legacy_scope_key TEXT NOT NULL,
                row_ref TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                reason TEXT NOT NULL,
                quarantined_at TEXT NOT NULL,
                UNIQUE(source_table, legacy_scope_key, row_ref)
            );
            CREATE INDEX IF NOT EXISTS idx_mission_canvas_legacy_quarantine_source
                ON mission_canvas_legacy_quarantine(source_table, legacy_scope_key);
            "#,
        )?;
        quarantine_legacy_rows(&connection)?;
        Ok(())
    }

    fn ensure_table(table: &str) -> Result<()> {
        if DOCUMENT_TABLES.contains(&table) {
            Ok(())
        } else {
            Err(MissionCanvasStoreError::UnknownTable(table.to_owned()))
        }
    }

    pub fn get_document(
        &self,
        table: &str,
        scope: &MissionCanvasScope,
        document_id: &str,
    ) -> Result<Option<StoredDocument>> {
        Self::ensure_table(table)?;
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let sql = format!(
            "SELECT revision, payload_json, updated_at FROM {table} WHERE scope_key = ?1 AND document_id = ?2"
        );
        let row: Option<(u64, String, String)> = connection
            .query_row(&sql, params![scope_key, document_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .optional()?;
        row.map(|(revision, payload, updated_at)| {
            Ok(StoredDocument {
                document_id: document_id.to_owned(),
                scope: scope.clone(),
                revision,
                payload: serde_json::from_str(&payload)?,
                updated_at,
            })
        })
        .transpose()
    }

    pub fn list_documents(
        &self,
        table: &str,
        scope: &MissionCanvasScope,
    ) -> Result<Vec<StoredDocument>> {
        Self::ensure_table(table)?;
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let sql = format!(
            "SELECT document_id, revision, payload_json, updated_at FROM {table} WHERE scope_key = ?1 ORDER BY document_id"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(params![scope_key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut documents = Vec::new();
        for row in rows {
            let (document_id, revision, payload, updated_at) = row?;
            documents.push(StoredDocument {
                document_id,
                scope: scope.clone(),
                revision,
                payload: serde_json::from_str(&payload)?,
                updated_at,
            });
        }
        Ok(documents)
    }

    pub fn put_document(
        &self,
        table: &str,
        document: &StoredDocument,
        expected_revision: Option<u64>,
        event: &CompositionEvent,
    ) -> Result<u64> {
        Self::ensure_table(table)?;
        let document_scope_key = canonical_scope_key(&document.scope)?;
        canonical_scope_key(&event.scope)?;
        if document.scope.workstream != event.scope.workstream {
            return Err(MissionCanvasStoreError::WorkstreamMismatch);
        }
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let current = current_document_revision(
            &transaction,
            table,
            &document_scope_key,
            &document.document_id,
        )?;
        check_revision(current, expected_revision)?;
        let sql = format!(
            "INSERT INTO {table}(scope_key, document_id, revision, payload_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(scope_key, document_id) DO UPDATE SET revision=excluded.revision, payload_json=excluded.payload_json, updated_at=excluded.updated_at"
        );
        transaction.execute(
            &sql,
            params![
                document_scope_key,
                document.document_id,
                document.revision,
                serde_json::to_string(&document.payload)?,
                document.updated_at,
            ],
        )?;
        append_event_transaction(&transaction, event)?;
        transaction.commit()?;
        Ok(document.revision)
    }

    /// Persist one generated ProfileLayoutMemory update with its exact
    /// Workstream, Evidence, Receipt, and durable event in one transaction.
    /// The operation is idempotent across restarts: a matching causation key
    /// and request digest returns the original Receipt without appending a
    /// second event or overwriting a newer memory revision.
    pub fn update_layout_memory(
        &self,
        memory: &ProfileLayoutMemory,
        expected_revision: u64,
        request_digest: &str,
        authority_ref: &str,
    ) -> Result<RecompositionReceipt> {
        Self::ensure_table("mission_canvas_layout_memory")?;
        let scope_key = canonical_scope_key(&memory.scope)?;
        if request_digest.trim().is_empty() || authority_ref.trim().is_empty() {
            return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                "request_digest_or_authority_missing",
            ));
        }
        validate_profile_layout_memory(
            memory,
            &memory.scope,
            &memory.profile_id,
            &memory.activity_mode_id,
            &memory.viewport_class,
        )
        .map_err(MissionCanvasStoreError::InvalidLayoutMemory)?;
        let next_revision =
            expected_revision
                .checked_add(1)
                .ok_or(MissionCanvasStoreError::RevisionConflict {
                    expected: expected_revision,
                    observed: u64::MAX,
                })?;
        if memory.memory_revision != next_revision {
            return Err(MissionCanvasStoreError::RevisionConflict {
                expected: next_revision,
                observed: memory.memory_revision,
            });
        }

        let digest = layout_memory_digest(memory)?;
        let document_id = memory.memory_id.clone();
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;

        let existing_event: Option<(String, String)> = transaction
            .query_row(
                "SELECT event_kind, payload_json FROM mission_canvas_composition_events WHERE scope_key = ?1 AND causation_id = ?2 ORDER BY sequence DESC LIMIT 1",
                params![scope_key, memory.idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((event_kind, payload_json)) = existing_event {
            let payload: Value = serde_json::from_str(&payload_json)?;
            let same_operation = event_kind == "layout_changed"
                && payload.get("operation_id").and_then(Value::as_str)
                    == Some(LAYOUT_MEMORY_UPDATE_OPERATION);
            if same_operation
                && payload.get("request_digest").and_then(Value::as_str) == Some(request_digest)
            {
                let receipt: RecompositionReceipt =
                    serde_json::from_value(payload.get("receipt").cloned().ok_or(
                        MissionCanvasStoreError::InvalidLayoutMemory("idempotent_receipt_missing"),
                    )?)?;
                if receipt.scope != memory.scope
                    || receipt.idempotency_key != memory.idempotency_key
                {
                    return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                        "idempotent_receipt_scope_mismatch",
                    ));
                }
                transaction.rollback()?;
                return Ok(receipt);
            }
            return Err(MissionCanvasStoreError::LayoutMemoryIdempotencyConflict);
        }

        let current: Option<(u64, String)> = transaction
            .query_row(
                "SELECT revision, payload_json FROM mission_canvas_layout_memory WHERE scope_key = ?1 AND document_id = ?2",
                params![scope_key, document_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match current {
            Some((observed_revision, payload_json)) => {
                if observed_revision != expected_revision {
                    return Err(MissionCanvasStoreError::RevisionConflict {
                        expected: expected_revision,
                        observed: observed_revision,
                    });
                }
                let stored: ProfileLayoutMemory = serde_json::from_str(&payload_json)?;
                validate_profile_layout_memory(
                    &stored,
                    &memory.scope,
                    &memory.profile_id,
                    &memory.activity_mode_id,
                    &memory.viewport_class,
                )
                .map_err(MissionCanvasStoreError::InvalidLayoutMemory)?;
                if stored.memory_revision != observed_revision {
                    return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                        "stored_revision_mismatch",
                    ));
                }
            }
            None if expected_revision != 0 => {
                return Err(MissionCanvasStoreError::RevisionConflict {
                    expected: expected_revision,
                    observed: 0,
                });
            }
            None => {}
        }

        let scope_fragment = hex::encode(Sha256::digest(scope_key.as_bytes()));
        let memory_fragment = hex::encode(Sha256::digest(memory.memory_id.as_bytes()));
        let evidence_id = format!(
            "recomposition-evidence:layout-memory:{scope_fragment}:{memory_fragment}:{}",
            memory.memory_revision
        );
        let receipt_id = format!(
            "recomposition-receipt:layout-memory:{scope_fragment}:{memory_fragment}:{}",
            memory.memory_revision
        );
        let candidate_contribution_ids = memory
            .placements
            .iter()
            .map(|placement| placement.contribution_id.clone())
            .chain(memory.absent_contribution_ids.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let issued_at = memory.updated_at.clone();
        let evidence = RecompositionEvidence {
            evidence_id: evidence_id.clone(),
            scope: memory.scope.clone(),
            trigger: "preference_change".into(),
            input_projection_digest: None,
            output_projection_digest: digest.clone(),
            rule_revision: "layout-memory:v1".into(),
            candidate_contribution_ids,
            eligibility_decisions: vec![],
            observed_at: issued_at.clone(),
        };
        let mut receipt = RecompositionReceipt {
            receipt_id: receipt_id.clone(),
            scope: memory.scope.clone(),
            accepted: true,
            projection_revision: memory.memory_revision,
            layout_revision: memory.memory_revision,
            projection_digest: digest,
            event_cursor: "event:pending".into(),
            evidence_id: evidence_id.clone(),
            idempotency_key: memory.idempotency_key.clone(),
            issued_at: issued_at.clone(),
        };
        let idempotency_fragment = hex::encode(Sha256::digest(memory.idempotency_key.as_bytes()));
        let event_id =
            format!("projection-event:layout-memory:{scope_fragment}:{idempotency_fragment}");
        let mut event = CompositionEvent {
            event_id: event_id.clone(),
            event_kind: "layout_changed".into(),
            scope: memory.scope.clone(),
            projection_revision: memory.memory_revision,
            layout_revision: memory.memory_revision,
            causation_id: Some(memory.idempotency_key.clone()),
            correlation_id: Some(authority_ref.to_owned()),
            occurred_at: issued_at,
            payload: json!({
                "operation_id": LAYOUT_MEMORY_UPDATE_OPERATION,
                "memory_id": memory.memory_id,
                "memory_revision": memory.memory_revision,
                "request_digest": request_digest,
                "authority_ref": authority_ref,
                "evidence": evidence,
                "receipt": receipt,
            }),
            evidence_refs: vec![evidence_id],
            receipt_refs: vec![receipt_id],
        };
        let sequence = append_event_transaction(&transaction, &event)?;
        receipt.event_cursor = format!("event:{sequence}");
        event.payload["receipt"] = serde_json::to_value(&receipt)?;
        transaction.execute(
            "UPDATE mission_canvas_composition_events SET payload_json = ?1 WHERE event_id = ?2",
            params![serde_json::to_string(&event.payload)?, event_id],
        )?;
        transaction.execute(
            r#"INSERT INTO mission_canvas_layout_memory(
                    scope_key, document_id, revision, payload_json, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(scope_key, document_id) DO UPDATE SET
                    revision=excluded.revision, payload_json=excluded.payload_json,
                    updated_at=excluded.updated_at"#,
            params![
                scope_key,
                document_id,
                memory.memory_revision,
                serde_json::to_string(memory)?,
                memory.updated_at,
            ],
        )?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Find a previously committed layout mutation for an exact Workstream.
    /// This read is used by Core before optimistic-concurrency validation so an
    /// exact retry replays its original result rather than being mistaken for
    /// a stale command. The write path repeats the check inside its transaction.
    pub fn find_layout_mutation_replay(
        &self,
        scope: &MissionCanvasScope,
        idempotency_key: &str,
        request_digest: &str,
    ) -> Result<Option<LayoutMutationResult>> {
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT event_kind, payload_json FROM mission_canvas_composition_events WHERE scope_key = ?1 AND causation_id = ?2 ORDER BY sequence DESC LIMIT 1",
                params![scope_key, idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((event_kind, payload_json)) = row else {
            return Ok(None);
        };
        let payload: Value = serde_json::from_str(&payload_json)?;
        if event_kind != "layout_changed"
            || payload.get("operation_id").and_then(Value::as_str) != Some(LAYOUT_MUTATE_OPERATION)
            || payload.get("request_digest").and_then(Value::as_str) != Some(request_digest)
        {
            return Err(MissionCanvasStoreError::LayoutMutationIdempotencyConflict);
        }
        let result: LayoutMutationResult =
            serde_json::from_value(payload.get("result").cloned().ok_or(
                MissionCanvasStoreError::InvalidLayoutMemory("layout_mutation_result_missing"),
            )?)?;
        if result.scope != *scope
            || result.accepted != true
            || result.event_cursor.trim().is_empty()
        {
            return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                "layout_mutation_result_scope_invalid",
            ));
        }
        Ok(Some(result))
    }

    /// Persist one canonical layout mutation, its direct generated result, and
    /// the durable event atomically. The event sequence becomes the returned
    /// cursor and is written back into the projection before commit, so a
    /// renderer cannot observe a result whose layout cursor is only process
    /// local. Matching Workstream-scoped retries return the original result.
    pub fn save_layout_mutation(
        &self,
        projection: &ResolvedWorkspaceProjection,
        expected_projection_revision: u64,
        command_id: &str,
        idempotency_key: &str,
        request_digest: &str,
        event: &CompositionEvent,
        evidence_ref: &str,
        receipt_ref: &str,
    ) -> Result<LayoutMutationResult> {
        if command_id.trim().is_empty()
            || idempotency_key.trim().is_empty()
            || request_digest.trim().is_empty()
            || evidence_ref.trim().is_empty()
            || receipt_ref.trim().is_empty()
        {
            return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                "layout_mutation_metadata_missing",
            ));
        }
        if event.scope != projection.scope
            || event.causation_id.as_deref() != Some(idempotency_key)
            || event.projection_revision != projection.projection_revision
            || event.layout_revision != projection.layout_revision
        {
            return Err(MissionCanvasStoreError::ProjectionEventRevisionMismatch);
        }
        if event.payload.get("operation_id").and_then(Value::as_str)
            != Some(LAYOUT_MUTATE_OPERATION)
        {
            return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                "layout_mutation_operation_invalid",
            ));
        }
        let scope_key = canonical_scope_key(&projection.scope)?;
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;

        let existing: Option<(String, String)> = transaction
            .query_row(
                "SELECT event_kind, payload_json FROM mission_canvas_composition_events WHERE scope_key = ?1 AND causation_id = ?2 ORDER BY sequence DESC LIMIT 1",
                params![scope_key, idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((event_kind, payload_json)) = existing {
            let payload: Value = serde_json::from_str(&payload_json)?;
            if event_kind == "layout_changed"
                && payload.get("operation_id").and_then(Value::as_str)
                    == Some(LAYOUT_MUTATE_OPERATION)
                && payload.get("request_digest").and_then(Value::as_str) == Some(request_digest)
            {
                let result: LayoutMutationResult =
                    serde_json::from_value(payload.get("result").cloned().ok_or(
                        MissionCanvasStoreError::InvalidLayoutMemory(
                            "layout_mutation_result_missing",
                        ),
                    )?)?;
                if result.scope != projection.scope || result.command_id != command_id {
                    return Err(MissionCanvasStoreError::InvalidLayoutMemory(
                        "layout_mutation_result_scope_invalid",
                    ));
                }
                transaction.rollback()?;
                return Ok(result);
            }
            return Err(MissionCanvasStoreError::LayoutMutationIdempotencyConflict);
        }

        let current: Option<(u64, u64)> = transaction
            .query_row(
                "SELECT projection_revision, layout_revision FROM mission_canvas_projections WHERE scope_key = ?1",
                params![scope_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match current {
            Some((observed_projection, observed_layout))
                if observed_projection == expected_projection_revision =>
            {
                if projection.projection_revision != observed_projection + 1
                    || projection.layout_revision != observed_layout + 1
                {
                    return Err(MissionCanvasStoreError::ProjectionEventRevisionMismatch);
                }
            }
            Some((observed_projection, _)) => {
                return Err(MissionCanvasStoreError::RevisionConflict {
                    expected: expected_projection_revision,
                    observed: observed_projection,
                });
            }
            None => {
                return Err(MissionCanvasStoreError::RevisionConflict {
                    expected: expected_projection_revision,
                    observed: 0,
                });
            }
        }

        let mut event = event.clone();
        let sequence = append_event_transaction(&transaction, &event)?;
        let mut next_projection = projection.clone();
        next_projection.durable_event_cursor = format!("event:{sequence}");
        next_projection.projection_digest = projection_digest(&next_projection)?;
        let result = LayoutMutationResult {
            scope: next_projection.scope.clone(),
            command_id: command_id.to_owned(),
            accepted: true,
            projection_revision: next_projection.projection_revision,
            layout_revision: next_projection.layout_revision,
            projection_digest: next_projection.projection_digest.clone(),
            event_cursor: next_projection.durable_event_cursor.clone(),
            error_ref: None,
            evidence_ref: Some(evidence_ref.to_owned()),
            receipt_ref: Some(receipt_ref.to_owned()),
        };
        event.payload["projection_digest"] = json!(next_projection.projection_digest);
        event.payload["result"] = serde_json::to_value(&result)?;
        transaction.execute(
            "UPDATE mission_canvas_composition_events SET payload_json = ?1 WHERE event_id = ?2",
            params![serde_json::to_string(&event.payload)?, event.event_id],
        )?;
        validate_projection_for_store(&next_projection)?;
        transaction.execute(
            r#"INSERT INTO mission_canvas_projections(
                    scope_key, projection_revision, layout_revision, projection_digest, payload_json, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(scope_key) DO UPDATE SET
                    projection_revision=excluded.projection_revision,
                    layout_revision=excluded.layout_revision,
                    projection_digest=excluded.projection_digest,
                    payload_json=excluded.payload_json,
                    updated_at=excluded.updated_at"#,
            params![
                scope_key,
                next_projection.projection_revision,
                next_projection.layout_revision,
                next_projection.projection_digest,
                serde_json::to_string(&next_projection)?,
                next_projection.resolved_at.as_deref().unwrap_or(&event.occurred_at),
            ],
        )?;
        transaction.commit()?;
        Ok(result)
    }

    /// Persist one exact rich-host lifecycle command and its event atomically.
    /// A retry with the same Workstream-scoped idempotency key returns the
    /// original document without appending a second lifecycle event. The
    /// durable event sequence is written back into the generated state cursor
    /// before the transaction commits.
    pub fn put_idempotent_lifecycle_document(
        &self,
        document: &StoredDocument,
        idempotency_key: &str,
        event: &CompositionEvent,
    ) -> Result<StoredDocument> {
        Self::ensure_table("mission_canvas_host_lifecycle")?;
        if idempotency_key.trim().is_empty() {
            return Err(MissionCanvasStoreError::InvalidHostLifecycleDocument(
                "idempotency key is empty".into(),
            ));
        }
        if event.scope != document.scope {
            return Err(MissionCanvasStoreError::InvalidHostLifecycleDocument(
                "lifecycle event scope differs from document scope".into(),
            ));
        }
        let payload_key = document
            .payload
            .get("idempotency_key")
            .and_then(Value::as_str);
        if payload_key != Some(idempotency_key) {
            return Err(MissionCanvasStoreError::InvalidHostLifecycleDocument(
                "document idempotency key differs from command".into(),
            ));
        }
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let scope_key = canonical_scope_key(&document.scope)?;
        let current: Option<(u64, String, String)> = transaction
            .query_row(
                "SELECT revision, payload_json, updated_at FROM mission_canvas_host_lifecycle WHERE scope_key = ?1 AND document_id = ?2",
                params![scope_key, document.document_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        if let Some((revision, payload_json, updated_at)) = current {
            let payload: Value = serde_json::from_str(&payload_json)?;
            let existing_key = payload
                .get("idempotency_key")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty());
            if existing_key == Some(idempotency_key) {
                transaction.rollback()?;
                return Ok(StoredDocument {
                    document_id: document.document_id.clone(),
                    scope: document.scope.clone(),
                    revision,
                    payload,
                    updated_at,
                });
            }
            check_revision(Some(revision), Some(document.revision.saturating_sub(1)))?;
        } else if document.revision != 1 {
            check_revision(None, Some(document.revision.saturating_sub(1)))?;
        }

        let sequence = append_event_transaction(&transaction, event)?;
        let mut payload = document.payload.clone();
        let state = payload
            .get_mut("state")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                MissionCanvasStoreError::InvalidHostLifecycleDocument(
                    "lifecycle envelope must contain an object state".into(),
                )
            })?;
        state.insert(
            "durable_event_cursor".into(),
            json!(format!("event:{sequence}")),
        );
        let payload_json = serde_json::to_string(&payload)?;
        transaction.execute(
            r#"INSERT INTO mission_canvas_host_lifecycle(
                    scope_key, document_id, revision, payload_json, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(scope_key, document_id) DO UPDATE SET
                    revision=excluded.revision, payload_json=excluded.payload_json,
                    updated_at=excluded.updated_at"#,
            params![
                scope_key,
                document.document_id,
                document.revision,
                payload_json,
                document.updated_at,
            ],
        )?;
        transaction.commit()?;
        Ok(StoredDocument {
            document_id: document.document_id.clone(),
            scope: document.scope.clone(),
            revision: document.revision,
            payload,
            updated_at: document.updated_at.clone(),
        })
    }

    /// Atomically persist every document and lifecycle event for one domain
    /// pack, then commit its governed receipt/idempotency record.  A retry with
    /// the same exact scope, pack, idempotency key, and request digest returns
    /// the original receipt without appending duplicate events.
    pub fn install_domain_pack(
        &self,
        scope: &MissionCanvasScope,
        pack: &DomainPack,
        idempotency_key: &str,
        request_digest: &str,
        authority_ref: &str,
        issued_at: &str,
    ) -> Result<DomainPackInstallReceipt> {
        let scope_key = canonical_scope_key(scope)?;
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;

        let existing_by_key: Option<(String, String, String)> = transaction
            .query_row(
                "SELECT pack_id, request_digest, receipt_json FROM mission_canvas_domain_pack_installations WHERE scope_key = ?1 AND idempotency_key = ?2",
                params![scope_key, idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        if let Some((existing_pack_id, existing_digest, receipt_json)) = existing_by_key {
            if existing_pack_id == pack.pack_id && existing_digest == request_digest {
                return Ok(serde_json::from_str(&receipt_json)?);
            }
            return Err(MissionCanvasStoreError::DomainPackIdempotencyConflict);
        }

        let already_installed: Option<String> = transaction
            .query_row(
                "SELECT idempotency_key FROM mission_canvas_domain_pack_installations WHERE scope_key = ?1 AND pack_id = ?2",
                params![scope_key, pack.pack_id],
                |row| row.get(0),
            )
            .optional()?;
        if already_installed.is_some() {
            return Err(MissionCanvasStoreError::DomainPackAlreadyInstalled(
                pack.pack_id.clone(),
            ));
        }

        let receipt_ref = format!("receipt:domain-pack:{}:{}", pack.pack_id, request_digest);
        let documents = domain_pack_documents(pack)?;
        let mut event_cursor = 0_u64;
        for (table, document_id, revision, payload, event_kind) in documents {
            let current = current_document_revision(&transaction, table, &scope_key, &document_id)?;
            if current.is_some() {
                return Err(MissionCanvasStoreError::DomainPackDocumentAlreadyExists(
                    document_id,
                ));
            }
            let updated_at = issued_at.to_owned();
            let sql = format!(
                "INSERT INTO {table}(scope_key, document_id, revision, payload_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)"
            );
            transaction.execute(
                &sql,
                params![
                    scope_key,
                    document_id,
                    revision,
                    serde_json::to_string(&payload)?,
                    updated_at,
                ],
            )?;
            let event = CompositionEvent {
                event_id: format!(
                    "mission-canvas:domain-pack:{}:{}:{}",
                    scope_key, pack.pack_id, document_id
                ),
                event_kind: event_kind.to_owned(),
                scope: scope.clone(),
                projection_revision: 0,
                layout_revision: 0,
                causation_id: Some(idempotency_key.to_owned()),
                correlation_id: Some(pack.pack_id.clone()),
                occurred_at: issued_at.to_owned(),
                payload: serde_json::json!({
                    "document_id": document_id,
                    "pack_id": pack.pack_id,
                    "request_digest": request_digest,
                }),
                evidence_refs: vec![format!("evidence:domain-pack:{}", pack.pack_id)],
                receipt_refs: vec![receipt_ref.clone()],
            };
            event_cursor = append_event_transaction(&transaction, &event)?;
        }

        let receipt = DomainPackInstallReceipt {
            schema: "focusa.mission_canvas.domain_pack_install_receipt.v1".into(),
            workstream: scope.workstream.clone(),
            installed: true,
            pack_id: pack.pack_id.clone(),
            receipt_ref: receipt_ref.clone(),
        };
        let governed = GovernedDomainPackInstallReceipt {
            receipt: receipt.clone(),
            idempotency_key: idempotency_key.to_owned(),
            request_digest: request_digest.to_owned(),
            authority_ref: authority_ref.to_owned(),
            event_cursor: format!("mission-canvas:domain-pack:{event_cursor}"),
            issued_at: issued_at.to_owned(),
        };
        transaction.execute(
            r#"INSERT INTO mission_canvas_domain_pack_installations(
                    scope_key, pack_id, idempotency_key, request_digest, receipt_ref,
                    receipt_json, event_cursor, authority_ref, issued_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                scope_key,
                pack.pack_id,
                idempotency_key,
                request_digest,
                receipt_ref,
                serde_json::to_string(&governed.receipt)?,
                governed.event_cursor,
                governed.authority_ref,
                governed.issued_at,
            ],
        )?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Load the one canonical projection partition owned by this exact
    /// Workstream. Optional Attachment/Surface authority is checked after the
    /// Workstream lookup; it is never used to guess another row.
    pub fn load_projection(
        &self,
        scope: &MissionCanvasScope,
    ) -> Result<Option<ResolvedWorkspaceProjection>> {
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let payload: Option<String> = connection
            .query_row(
                "SELECT payload_json FROM mission_canvas_projections WHERE scope_key = ?1",
                params![scope_key],
                |row| row.get(0),
            )
            .optional()?;
        let projection: Option<ResolvedWorkspaceProjection> = payload
            .map(|value| serde_json::from_str(&value).map_err(MissionCanvasStoreError::from))
            .transpose()?;
        if let Some(projection) = projection {
            validate_projection_for_store(&projection)?;
            projection
                .validate_scope(scope)
                .map_err(MissionCanvasStoreError::InvalidProjection)?;
            Ok(Some(projection))
        } else {
            Ok(None)
        }
    }

    /// Workstream-only read for aggregate consumers. A caller that has
    /// subordinate identity must use [`Self::load_projection`] so the exact
    /// Attachment/Surface relationship is checked as well.
    pub fn load_projection_for_workstream(
        &self,
        workstream: &WorkstreamKey,
    ) -> Result<Option<ResolvedWorkspaceProjection>> {
        let scope = MissionCanvasScope::new(workstream.clone(), None)
            .map_err(MissionCanvasStoreError::InvalidScope)?;
        self.load_projection(&scope)
    }

    /// Compatibility name retained for existing API consumers. The storage
    /// owner is now the WorkstreamKey, not the flattened authority context.
    pub fn get_projection(
        &self,
        scope: &MissionCanvasScope,
    ) -> Result<Option<ResolvedWorkspaceProjection>> {
        self.load_projection(scope)
    }

    /// Load one exact-Workstream CanvasDraftState document and fail closed on
    /// missing, foreign, or incomplete authority. A stored draft that claims
    /// subordinate Attachment/Surface identity must match the requested scope;
    /// an aggregate read never guesses an Attachment or Surface.
    pub fn load_draft(
        &self,
        scope: &MissionCanvasScope,
        draft_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        if draft_id.trim().is_empty() {
            return Err(MissionCanvasStoreError::InvalidScope("draft_id is required"));
        }
        let document = self.get_document(
            "mission_canvas_drafts",
            scope,
            &format!("draft:{draft_id}"),
        )?;
        let Some(document) = document else {
            return Ok(None);
        };
        let draft = &document.payload;
        let draft_object = draft.as_object().ok_or_else(|| {
            MissionCanvasStoreError::InvalidScope("draft payload is not a JSON object")
        })?;
        let draft_workstream = draft_object.get("workstream").ok_or_else(|| {
            MissionCanvasStoreError::InvalidScope("draft payload is missing exact Workstream identity")
        })?;
        let scope_workstream = serde_json::to_value(&scope.workstream)
            .map_err(MissionCanvasStoreError::Serialization)?;
        if draft_workstream != &scope_workstream {
            return Err(MissionCanvasStoreError::WorkstreamMismatch);
        }
        if let Some(attachment) = draft_object.get("attachment") {
            let expected_attachment = scope
                .attachment
                .as_ref()
                .map(|attachment| serde_json::to_value(attachment).map_err(MissionCanvasStoreError::Serialization))
                .transpose()?;
            match (attachment, expected_attachment) {
                (Value::Null, None) => {}
                (Value::Null, Some(_)) => {
                    return Err(MissionCanvasStoreError::InvalidScope("draft claims attachment authority but scope lacks it"))
                }
                (value, None) => {
                    if !value.is_null() {
                        return Err(MissionCanvasStoreError::InvalidScope("draft carries attachment identity outside the request scope"));
                    }
                }
                (value, Some(expected)) => {
                    if value != &expected {
                        return Err(MissionCanvasStoreError::WorkstreamMismatch);
                    }
                }
            }
        }
        Ok(Some(draft.clone()))
    }

    /// Save one Workstream-owned projection and its lifecycle event atomically.
    /// Evidence, receipt, omission diagnostics, layout memory references, and
    /// the projection revision remain in the canonical payload/event rows.
    pub fn save_projection(
        &self,
        projection: &ResolvedWorkspaceProjection,
        expected_revision: Option<u64>,
        event: &CompositionEvent,
    ) -> Result<u64> {
        let workstream_scope_key = validate_projection_for_store(projection)?;
        canonical_scope_key(&event.scope)?;
        if projection.scope.workstream != event.scope.workstream {
            return Err(MissionCanvasStoreError::WorkstreamMismatch);
        }
        if projection.projection_revision != event.projection_revision
            || projection.layout_revision != event.layout_revision
        {
            return Err(MissionCanvasStoreError::ProjectionEventRevisionMismatch);
        }
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let current: Option<u64> = transaction
            .query_row(
                "SELECT projection_revision FROM mission_canvas_projections WHERE scope_key = ?1",
                params![workstream_scope_key],
                |row| row.get(0),
            )
            .optional()?;
        check_revision(current, expected_revision)?;
        transaction.execute(
            r#"INSERT INTO mission_canvas_projections(
                    scope_key, projection_revision, layout_revision, projection_digest, payload_json, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(scope_key) DO UPDATE SET
                    projection_revision=excluded.projection_revision,
                    layout_revision=excluded.layout_revision,
                    projection_digest=excluded.projection_digest,
                    payload_json=excluded.payload_json,
                    updated_at=excluded.updated_at"#,
            params![
                workstream_scope_key,
                projection.projection_revision,
                projection.layout_revision,
                projection.projection_digest,
                serde_json::to_string(projection)?,
                projection.resolved_at.as_deref().unwrap_or(&event.occurred_at),
            ],
        )?;
        append_event_transaction(&transaction, event)?;
        transaction.commit()?;
        Ok(projection.projection_revision)
    }

    /// Compatibility name retained for existing API consumers.
    pub fn put_projection(
        &self,
        projection: &ResolvedWorkspaceProjection,
        expected_revision: Option<u64>,
        event: &CompositionEvent,
    ) -> Result<u64> {
        self.save_projection(projection, expected_revision, event)
    }

    pub fn append_event(&self, event: &CompositionEvent) -> Result<u64> {
        let connection = self.connection.lock();
        append_event_transaction(&connection, event)
    }

    pub fn events_after(
        &self,
        scope: &MissionCanvasScope,
        sequence: u64,
        limit: usize,
    ) -> Result<Vec<(u64, CompositionEvent)>> {
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let mut statement = connection.prepare(
            r#"SELECT sequence, event_id, event_kind, projection_revision, layout_revision,
                      causation_id, correlation_id, occurred_at, payload_json, evidence_refs_json, receipt_refs_json
               FROM mission_canvas_composition_events
               WHERE scope_key = ?1 AND sequence > ?2
               ORDER BY sequence ASC LIMIT ?3"#,
        )?;
        let rows = statement.query_map(params![scope_key, sequence, limit as u64], |row| {
            Ok((
                row.get::<_, u64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, u64>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;
        let mut events = Vec::new();
        for row in rows {
            let (
                sequence,
                event_id,
                event_kind,
                projection_revision,
                layout_revision,
                causation_id,
                correlation_id,
                occurred_at,
                payload,
                evidence,
                receipts,
            ) = row?;
            events.push((
                sequence,
                CompositionEvent {
                    event_id,
                    event_kind,
                    scope: scope.clone(),
                    projection_revision,
                    layout_revision,
                    causation_id,
                    correlation_id,
                    occurred_at,
                    payload: serde_json::from_str(&payload)?,
                    evidence_refs: serde_json::from_str(&evidence)?,
                    receipt_refs: serde_json::from_str(&receipts)?,
                },
            ));
        }
        Ok(events)
    }

    /// Return the latest durable composition-event sequence for one exact
    /// Workstream.  A cursor is scoped to this stream; callers must not use a
    /// global/latest-record fallback when deciding whether a cursor is valid.
    pub fn latest_event_sequence(&self, scope: &MissionCanvasScope) -> Result<u64> {
        let scope_key = canonical_scope_key(scope)?;
        let connection = self.connection.lock();
        let sequence: Option<u64> = connection.query_row(
            "SELECT MAX(sequence) FROM mission_canvas_composition_events WHERE scope_key = ?1",
            params![scope_key],
            |row| row.get(0),
        )?;
        Ok(sequence.unwrap_or_default())
    }
}

/// Return the only storage partition accepted for canonical Mission Canvas
/// state. WorkstreamKey::storage_key is a stable hash of the generated
/// Workstream identity; project_root, continuity, session, and selected-surface
/// values cannot form a substitute key.
fn canonical_scope_key(scope: &MissionCanvasScope) -> Result<String> {
    scope
        .validate()
        .map_err(MissionCanvasStoreError::InvalidScope)?;
    if scope.work_surface_id.is_some() && scope.attachment.is_none() {
        return Err(MissionCanvasStoreError::InvalidScope("attachment_missing"));
    }
    Ok(scope.workstream.storage_key())
}

fn validate_projection_for_store(projection: &ResolvedWorkspaceProjection) -> Result<String> {
    let scope_key = canonical_scope_key(&projection.scope)?;
    projection
        .validate_scope(&projection.scope)
        .map_err(MissionCanvasStoreError::InvalidProjection)?;
    Ok(scope_key)
}

/// Preserve pre-Workstream rows as immutable migration evidence. They are
/// deliberately not re-keyed by parsing project/continuity/session fields:
/// ambiguous legacy ownership is quarantined rather than guessed.
fn quarantine_legacy_rows(connection: &Connection) -> Result<()> {
    for table in SCOPED_TABLES {
        let table_name = *table;
        let payload_column = if table_name == "mission_canvas_domain_pack_installations" {
            "receipt_json"
        } else {
            "payload_json"
        };
        let sql = format!(
            "SELECT rowid, scope_key, {payload_column} FROM {table_name} \
             WHERE length(scope_key) <> 64 \
                OR lower(scope_key) <> scope_key \
                OR scope_key GLOB '*[^0-9a-f]*'"
        );
        let rows = {
            let mut statement = connection.prepare(&sql)?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };
        for (row_id, legacy_scope_key, payload_json) in rows {
            connection.execute(
                r#"INSERT OR IGNORE INTO mission_canvas_legacy_quarantine(
                        source_table, legacy_scope_key, row_ref, payload_json, reason, quarantined_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                params![
                    table_name,
                    legacy_scope_key,
                    format!("rowid:{row_id}"),
                    payload_json,
                    "legacy_scope_key_not_workstream_key",
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }
    }
    Ok(())
}

fn domain_pack_documents(
    pack: &DomainPack,
) -> Result<Vec<(&'static str, String, u64, serde_json::Value, String)>> {
    let mut documents = vec![(
        "mission_canvas_profiles",
        format!("profile:{}", pack.profile.profile_id),
        pack.profile.revision,
        serde_json::to_value(&pack.profile)?,
        "profile_changed".to_owned(),
    )];
    documents.extend(
        pack.activities
            .iter()
            .map(|activity| {
                Ok((
                    "mission_canvas_activity_modes",
                    format!("activity:{}", activity.activity_mode_id),
                    activity.revision,
                    serde_json::to_value(activity)?,
                    "activity_mode_changed".to_owned(),
                ))
            })
            .collect::<Result<Vec<_>>>()?,
    );
    for entry in &pack.registry_entries {
        let table = domain_pack_registry_table(&entry.registry_kind)
            .ok_or_else(|| MissionCanvasStoreError::UnknownTable(entry.registry_kind.clone()))?;
        documents.push((
            table,
            format!("registry:{}", entry.entry_id),
            entry.revision,
            serde_json::to_value(entry)?,
            "candidate_discovered".to_owned(),
        ));
    }
    Ok(documents)
}

fn domain_pack_registry_table(registry_kind: &str) -> Option<&'static str> {
    match registry_kind {
        "PanelRegistry"
        | "HomeCanvasRegistry"
        | "WorkSurfaceRendererRegistry"
        | "ArtifactRendererRegistry"
        | "TerminologyRegistry"
        | "DomainSemanticBindingRegistry" => Some("mission_canvas_registry_entries"),
        _ => None,
    }
}

fn current_document_revision(
    transaction: &Transaction<'_>,
    table: &str,
    scope_key: &str,
    document_id: &str,
) -> Result<Option<u64>> {
    let sql = format!("SELECT revision FROM {table} WHERE scope_key = ?1 AND document_id = ?2");
    Ok(transaction
        .query_row(&sql, params![scope_key, document_id], |row| row.get(0))
        .optional()?)
}

fn check_revision(current: Option<u64>, expected: Option<u64>) -> Result<()> {
    match (current, expected) {
        (None, None) => Ok(()),
        (Some(observed), None) => Err(MissionCanvasStoreError::AlreadyExists(observed)),
        (Some(observed), Some(expected)) if observed == expected => Ok(()),
        (Some(observed), Some(expected)) => {
            Err(MissionCanvasStoreError::RevisionConflict { expected, observed })
        }
        (None, Some(expected)) => Err(MissionCanvasStoreError::RevisionConflict {
            expected,
            observed: 0,
        }),
    }
}

fn append_event_transaction(connection: &Connection, event: &CompositionEvent) -> Result<u64> {
    let scope_key = canonical_scope_key(&event.scope)?;
    connection.execute(
        r#"INSERT INTO mission_canvas_composition_events(
                event_id, scope_key, event_kind, projection_revision, layout_revision,
                causation_id, correlation_id, occurred_at, payload_json, evidence_refs_json, receipt_refs_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        params![
            event.event_id,
            scope_key,
            event.event_kind,
            event.projection_revision,
            event.layout_revision,
            event.causation_id,
            event.correlation_id,
            event.occurred_at,
            serde_json::to_string(&event.payload)?,
            serde_json::to_string(&event.evidence_refs)?,
            serde_json::to_string(&event.receipt_refs)?,
        ],
    )?;
    Ok(connection.last_insert_rowid() as u64)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn scope() -> MissionCanvasScope {
        let legacy = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        let workstream = crate::workstream_identity::WorkstreamKey::new(
            crate::workstream_identity::ScopeRef::project(legacy).unwrap(),
            crate::workstream_identity::WorkstreamId::parse("ws:mission-canvas").unwrap(),
        );
        let continuity =
            crate::workstream_identity::ContinuityId::parse("continuity:mission-canvas").unwrap();
        let attachment = crate::workstream_identity::AttachmentKey::new(
            workstream.clone(),
            Some(continuity),
            crate::workstream_identity::InstanceId::parse("instance:1").unwrap(),
            crate::workstream_identity::SessionId::parse("session:1").unwrap(),
            crate::workstream_identity::AttachmentId::parse("attachment:1").unwrap(),
            crate::workstream_identity::WorkspaceBindingId::parse("workspace:mission-canvas")
                .unwrap(),
        );
        MissionCanvasScope::from_parts(
            workstream,
            None,
            Some(attachment),
            Some(
                crate::workstream_identity::WorkspaceBindingId::parse("workspace:mission-canvas")
                    .unwrap(),
            ),
            None,
            Some(crate::workstream_identity::WorkSurfaceId::parse("surface:primary").unwrap()),
        )
        .unwrap()
    }

    fn event(id: &str, revision: u64) -> CompositionEvent {
        CompositionEvent {
            event_id: id.into(),
            event_kind: "layout_changed".into(),
            scope: scope(),
            projection_revision: revision,
            layout_revision: revision,
            causation_id: None,
            correlation_id: None,
            occurred_at: "2026-07-30T12:00:00Z".into(),
            payload: json!({"revision": revision}),
            evidence_refs: vec![],
            receipt_refs: vec![],
        }
    }

    #[test]
    fn migrates_all_composition_tables_and_writes_atomically() {
        let store = MissionCanvasStore::open_in_memory().unwrap();
        let document = StoredDocument {
            document_id: "profile:software".into(),
            scope: scope(),
            revision: 1,
            payload: json!({"profile_id": "software"}),
            updated_at: "2026-07-30T12:00:00Z".into(),
        };
        store
            .put_document(
                "mission_canvas_profiles",
                &document,
                None,
                &event("event:1", 1),
            )
            .unwrap();
        assert_eq!(
            store
                .get_document("mission_canvas_profiles", &scope(), "profile:software")
                .unwrap()
                .unwrap(),
            document
        );
        assert_eq!(store.events_after(&scope(), 0, 10).unwrap().len(), 1);
    }

    #[test]
    fn rejects_stale_transaction_without_appending_event() {
        let store = MissionCanvasStore::open_in_memory().unwrap();
        let mut document = StoredDocument {
            document_id: "draft:1".into(),
            scope: scope(),
            revision: 1,
            payload: json!({"content": "one"}),
            updated_at: "2026-07-30T12:00:00Z".into(),
        };
        store
            .put_document(
                "mission_canvas_drafts",
                &document,
                None,
                &event("event:1", 1),
            )
            .unwrap();
        document.revision = 2;
        let error = store
            .put_document(
                "mission_canvas_drafts",
                &document,
                Some(0),
                &event("event:2", 2),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            MissionCanvasStoreError::RevisionConflict { .. }
        ));
        assert_eq!(store.events_after(&scope(), 0, 10).unwrap().len(), 1);
    }

    #[test]
    fn survives_restart_and_rejects_concurrent_stale_writer() {
        let directory = std::env::temp_dir().join(format!(
            "focusa-mission-canvas-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let document = StoredDocument {
            document_id: "layout-memory:1".into(),
            scope: scope(),
            revision: 1,
            payload: json!({"placements": []}),
            updated_at: "2026-07-30T12:00:00Z".into(),
        };
        {
            let store = MissionCanvasStore::open(&directory).unwrap();
            store
                .put_document(
                    "mission_canvas_layout_memory",
                    &document,
                    None,
                    &event("event:restart:1", 1),
                )
                .unwrap();
        }
        let store = MissionCanvasStore::open(&directory).unwrap();
        assert_eq!(
            store
                .get_document(
                    "mission_canvas_layout_memory",
                    &scope(),
                    &document.document_id
                )
                .unwrap()
                .unwrap(),
            document
        );
        let mut first = document.clone();
        first.revision = 2;
        first.payload = json!({"placements": ["a"]});
        let mut second = first.clone();
        second.payload = json!({"placements": ["b"]});
        let left = store.clone();
        let right = store.clone();
        let left_result = std::thread::spawn(move || {
            left.put_document(
                "mission_canvas_layout_memory",
                &first,
                Some(1),
                &event("event:concurrent:left", 2),
            )
        })
        .join()
        .unwrap();
        let right_result = std::thread::spawn(move || {
            right.put_document(
                "mission_canvas_layout_memory",
                &second,
                Some(1),
                &event("event:concurrent:right", 2),
            )
        })
        .join()
        .unwrap();
        assert_ne!(left_result.is_ok(), right_result.is_ok());
        assert_eq!(store.events_after(&scope(), 0, 10).unwrap().len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// Hostile Workstream isolation and fail-closed persistence coverage for
    /// CORE-005. This module name is also the bounded runtime-test selector.
    mod mission_canvas_store_workstream_isolation {
        use serde_json::json;

        use super::*;

        fn workstream(id: &str) -> WorkstreamKey {
            let base = scope().workstream;
            WorkstreamKey::new(
                base.scope,
                crate::workstream_identity::WorkstreamId::parse(id).unwrap(),
            )
        }

        fn aggregate_scope(id: &str) -> MissionCanvasScope {
            MissionCanvasScope::new(workstream(id), None).unwrap()
        }

        fn projection(
            scope: MissionCanvasScope,
            revision: u64,
            marker: &str,
        ) -> ResolvedWorkspaceProjection {
            ResolvedWorkspaceProjection {
                schema: "focusa.resolved_workspace_projection.v1".into(),
                scope,
                workspace_profile_id: "software".into(),
                workspace_profile_revision: 1,
                activity_mode_id: "overview".into(),
                activity_mode_revision: 1,
                focused_work_surface_id: None,
                canonical_read_model_revision: revision,
                candidate_contribution_ids: vec![format!("candidate:{marker}")],
                eligible_contributions: vec![],
                omission_diagnostics: vec![OmissionDiagnostic {
                    contribution_id: format!("contribution:{marker}"),
                    reason: "capability_not_present".into(),
                    rule_revision: "resolver:v1".into(),
                    projection_revision: revision,
                    canonical_input_refs: vec![],
                    details_ref: Some(format!("diagnostic:{marker}")),
                    observed_at: "2026-07-30T12:00:00Z".into(),
                }],
                layout_tree: json!({
                    "kind": "single",
                    "node_id": format!("layout:{marker}"),
                    "contribution_id": format!("contribution:{marker}")
                }),
                operation_bindings: vec![],
                focused_semantic_target: format!("semantic:{marker}"),
                projection_revision: revision,
                layout_revision: revision,
                durable_event_cursor: format!("event:{marker}:{revision}"),
                projection_digest: format!("sha256:{marker}"),
                resolved_at: Some("2026-07-30T12:00:00Z".into()),
                evidence_refs: vec![format!("evidence:{marker}")],
                receipt_refs: vec![format!("receipt:{marker}")],
            }
        }

        fn event_for(scope: MissionCanvasScope, id: &str, revision: u64) -> CompositionEvent {
            CompositionEvent {
                event_id: id.into(),
                event_kind: "projection_resolved".into(),
                scope,
                projection_revision: revision,
                layout_revision: revision,
                causation_id: None,
                correlation_id: None,
                occurred_at: "2026-07-30T12:00:00Z".into(),
                payload: json!({"revision": revision}),
                evidence_refs: vec![format!("evidence:{id}")],
                receipt_refs: vec![format!("receipt:{id}")],
            }
        }

        #[test]
        fn reading_workstream_a_cannot_return_workstream_b_data() {
            let store = MissionCanvasStore::open_in_memory().unwrap();
            let a = aggregate_scope("ws:alpha");
            let b = aggregate_scope("ws:beta");
            store
                .save_projection(
                    &projection(a.clone(), 1, "alpha"),
                    None,
                    &event_for(a.clone(), "event:alpha", 1),
                )
                .unwrap();
            store
                .save_projection(
                    &projection(b.clone(), 1, "beta"),
                    None,
                    &event_for(b.clone(), "event:beta", 1),
                )
                .unwrap();

            let loaded_a = store.load_projection(&a).unwrap().unwrap();
            let loaded_b = store
                .load_projection_for_workstream(&b.workstream)
                .unwrap()
                .unwrap();
            assert_eq!(loaded_a.scope.workstream, a.workstream);
            assert_eq!(loaded_a.evidence_refs, vec!["evidence:alpha"]);
            assert_eq!(loaded_b.scope.workstream, b.workstream);
            assert_eq!(loaded_b.receipt_refs, vec!["receipt:beta"]);
            assert!(store
                .load_projection(&aggregate_scope("ws:missing"))
                .unwrap()
                .is_none());
            assert_eq!(store.events_after(&a, 0, 10).unwrap().len(), 1);
            assert!(store
                .events_after(&aggregate_scope("ws:missing"), 0, 10)
                .unwrap()
                .is_empty());
        }

        #[test]
        fn layout_memory_evidence_receipt_and_diagnostic_rows_are_workstream_partitioned() {
            let store = MissionCanvasStore::open_in_memory().unwrap();
            let a = aggregate_scope("ws:alpha");
            let b = aggregate_scope("ws:beta");
            for (scope, marker, event_id) in [
                (a.clone(), "alpha", "event:memory:alpha"),
                (b.clone(), "beta", "event:memory:beta"),
            ] {
                let document = StoredDocument {
                    document_id: "layout-memory:software:overview".into(),
                    scope: scope.clone(),
                    revision: 1,
                    payload: json!({
                        "placements": [marker],
                        "evidence_refs": [format!("evidence:{marker}")],
                        "receipt_refs": [format!("receipt:{marker}")],
                        "omission_diagnostics": [{"reason": "capability_not_present"}]
                    }),
                    updated_at: "2026-07-30T12:00:00Z".into(),
                };
                store
                    .put_document(
                        "mission_canvas_layout_memory",
                        &document,
                        None,
                        &event_for(scope, event_id, 1),
                    )
                    .unwrap();
            }
            let loaded_a = store
                .get_document(
                    "mission_canvas_layout_memory",
                    &a,
                    "layout-memory:software:overview",
                )
                .unwrap()
                .unwrap();
            let loaded_b = store
                .get_document(
                    "mission_canvas_layout_memory",
                    &b,
                    "layout-memory:software:overview",
                )
                .unwrap()
                .unwrap();
            assert_eq!(loaded_a.payload["placements"][0], "alpha");
            assert_eq!(loaded_a.payload["evidence_refs"][0], "evidence:alpha");
            assert_eq!(loaded_b.payload["placements"][0], "beta");
            assert_eq!(loaded_b.payload["receipt_refs"][0], "receipt:beta");
        }

        #[test]
        fn foreign_scope_and_surface_without_attachment_fail_closed() {
            let store = MissionCanvasStore::open_in_memory().unwrap();
            let a = aggregate_scope("ws:alpha");
            let b = aggregate_scope("ws:beta");
            let projection_a = projection(a.clone(), 1, "alpha");
            let foreign_event = event_for(b, "event:foreign", 1);
            assert!(matches!(
                store.save_projection(&projection_a, None, &foreign_event),
                Err(MissionCanvasStoreError::WorkstreamMismatch)
            ));
            assert!(store.load_projection(&a).unwrap().is_none());

            let invalid_scope = MissionCanvasScope::from_parts(
                a.workstream.clone(),
                None,
                None,
                None,
                None,
                Some(crate::workstream_identity::WorkSurfaceId::parse("surface:orphan").unwrap()),
            )
            .unwrap();
            let invalid_projection = projection(invalid_scope.clone(), 1, "orphan");
            assert!(matches!(
                store.save_projection(
                    &invalid_projection,
                    None,
                    &event_for(invalid_scope, "event:orphan", 1),
                ),
                Err(MissionCanvasStoreError::InvalidScope("attachment_missing"))
            ));
        }

        #[test]
        fn stale_revision_and_foreign_cursor_never_overwrite_or_leak() {
            let store = MissionCanvasStore::open_in_memory().unwrap();
            let a = aggregate_scope("ws:alpha");
            let b = aggregate_scope("ws:beta");
            store
                .save_projection(
                    &projection(a.clone(), 1, "alpha"),
                    None,
                    &event_for(a.clone(), "event:alpha", 1),
                )
                .unwrap();
            let stale = store.save_projection(
                &projection(a.clone(), 2, "alpha-new"),
                Some(0),
                &event_for(a.clone(), "event:stale", 2),
            );
            assert!(matches!(
                stale,
                Err(MissionCanvasStoreError::RevisionConflict {
                    expected: 0,
                    observed: 1
                })
            ));
            assert_eq!(
                store
                    .load_projection(&a)
                    .unwrap()
                    .unwrap()
                    .projection_revision,
                1
            );
            assert_eq!(store.events_after(&a, 0, 10).unwrap().len(), 1);
            assert_eq!(store.latest_event_sequence(&b).unwrap(), 0);
            assert!(store.events_after(&b, 0, 10).unwrap().is_empty());
        }

        #[test]
        fn legacy_scope_rows_are_quarantined_and_never_rekeyed_by_guess() {
            let store = MissionCanvasStore::open_in_memory().unwrap();
            let legacy_scope = scope();
            let legacy_key = legacy_scope.storage_key();
            let payload =
                serde_json::to_string(&projection(aggregate_scope("ws:legacy"), 1, "legacy"))
                    .unwrap();
            {
                let connection = store.connection.lock();
                connection
                    .execute(
                        "INSERT INTO mission_canvas_projections(scope_key, projection_revision, layout_revision, projection_digest, payload_json, updated_at) VALUES (?1, 1, 1, ?2, ?3, ?4)",
                        params![
                            legacy_key,
                            "sha256:legacy",
                            payload,
                            "2026-07-30T12:00:00Z"
                        ],
                    )
                    .unwrap();
            }
            store.migrate().unwrap();
            assert!(store
                .load_projection(&aggregate_scope("ws:legacy"))
                .unwrap()
                .is_none());
            let connection = store.connection.lock();
            let quarantined: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM mission_canvas_legacy_quarantine WHERE source_table = 'mission_canvas_projections'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(quarantined, 1);
        }
    }
}
