use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use thiserror::Error;

use super::model::{
    CompositionEvent, DomainPackInstallReceipt, GovernedDomainPackInstallReceipt,
    MissionCanvasScope, ResolvedWorkspaceProjection, StoredDocument,
};
use super::profiles::DomainPack;

const DOCUMENT_TABLES: &[&str] = &[
    "mission_canvas_profiles",
    "mission_canvas_activity_modes",
    "mission_canvas_registry_entries",
    "mission_canvas_layout_trees",
    "mission_canvas_layout_memory",
    "mission_canvas_drafts",
    "mission_canvas_host_lifecycle",
];

#[derive(Debug, Error)]
pub enum MissionCanvasStoreError {
    #[error("mission canvas I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mission canvas SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("mission canvas serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unknown mission canvas document table: {0}")]
    UnknownTable(String),
    #[error("revision conflict: expected {expected}, observed {observed}")]
    RevisionConflict { expected: u64, observed: u64 },
    #[error("document already exists at revision {0}")]
    AlreadyExists(u64),
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
            "#,
        )?;
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
        let connection = self.connection.lock();
        let sql = format!(
            "SELECT revision, payload_json, updated_at FROM {table} WHERE scope_key = ?1 AND document_id = ?2"
        );
        let row: Option<(u64, String, String)> = connection
            .query_row(&sql, params![scope.storage_key(), document_id], |row| {
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
        let connection = self.connection.lock();
        let sql = format!(
            "SELECT document_id, revision, payload_json, updated_at FROM {table} WHERE scope_key = ?1 ORDER BY document_id"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(params![scope.storage_key()], |row| {
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
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let current = current_document_revision(
            &transaction,
            table,
            &document.scope.storage_key(),
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
                document.scope.storage_key(),
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
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let scope_key = scope.storage_key();

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

    pub fn get_projection(
        &self,
        scope: &MissionCanvasScope,
    ) -> Result<Option<ResolvedWorkspaceProjection>> {
        let connection = self.connection.lock();
        let payload: Option<String> = connection
            .query_row(
                "SELECT payload_json FROM mission_canvas_projections WHERE scope_key = ?1",
                params![scope.storage_key()],
                |row| row.get(0),
            )
            .optional()?;
        payload
            .map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    pub fn put_projection(
        &self,
        projection: &ResolvedWorkspaceProjection,
        expected_revision: Option<u64>,
        event: &CompositionEvent,
    ) -> Result<u64> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let current: Option<u64> = transaction
            .query_row(
                "SELECT projection_revision FROM mission_canvas_projections WHERE scope_key = ?1",
                params![projection.scope.storage_key()],
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
                projection.scope.storage_key(),
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
        let connection = self.connection.lock();
        let mut statement = connection.prepare(
            r#"SELECT sequence, event_id, event_kind, projection_revision, layout_revision,
                      causation_id, correlation_id, occurred_at, payload_json, evidence_refs_json, receipt_refs_json
               FROM mission_canvas_composition_events
               WHERE scope_key = ?1 AND sequence > ?2
               ORDER BY sequence ASC LIMIT ?3"#,
        )?;
        let rows = statement.query_map(
            params![scope.storage_key(), sequence, limit as u64],
            |row| {
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
            },
        )?;
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
        let connection = self.connection.lock();
        let sequence: Option<u64> = connection.query_row(
            "SELECT MAX(sequence) FROM mission_canvas_composition_events WHERE scope_key = ?1",
            params![scope.storage_key()],
            |row| row.get(0),
        )?;
        Ok(sequence.unwrap_or_default())
    }
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
    connection.execute(
        r#"INSERT INTO mission_canvas_composition_events(
                event_id, scope_key, event_kind, projection_revision, layout_revision,
                causation_id, correlation_id, occurred_at, payload_json, evidence_refs_json, receipt_refs_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        params![
            event.event_id,
            event.scope.storage_key(),
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
}
