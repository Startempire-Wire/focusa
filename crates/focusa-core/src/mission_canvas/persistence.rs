use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use thiserror::Error;

use super::model::{
    CompositionEvent, MissionCanvasScope, ResolvedWorkspaceProjection, StoredDocument,
};

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
        MissionCanvasScope {
            project_root: "/tmp/focusa".into(),
            continuity_id: "mission-canvas".into(),
            instance_id: Some("instance:1".into()),
            session_id: "session:1".into(),
            attachment_id: "attachment:1".into(),
            working_subpath_id: Some("primary".into()),
        }
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
