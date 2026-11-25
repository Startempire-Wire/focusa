//! Canonical temporal work-item state; native SQLite snapshots carry it with events.

use super::persistence_sqlite::SqlitePersistence;
use anyhow::{Context, Result, bail, ensure};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

const LEGACY_DIR: &str = "temporal-work-items";
const MIGRATION: &str = "temporal_work_items_migrated_v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItem {
    pub item_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub title: String,
    pub status: String,
    pub revision: u64,
    pub estimate_ms: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub evidence_refs: Vec<String>,
    pub last_reason: Option<String>,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug)]
pub struct WorkItemSnapshot {
    pub items: Vec<WorkItem>,
    scope: (String, String),
    version: Option<i64>,
}

fn legacy_files(data_dir: &Path) -> Result<Vec<PathBuf>> {
    let directory = data_dir.join(LEGACY_DIR);
    match fs::symlink_metadata(&directory) {
        Ok(metadata) => ensure!(
            metadata.is_dir(),
            "legacy temporal work-item path is not a directory"
        ),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|ext| ext == "json") {
            ensure!(
                entry.file_type()?.is_file(),
                "legacy temporal work-item record is not a regular file"
            );
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(files)
}

fn validate_scope(items: &[WorkItem], root: &str, continuity: &str) -> Result<()> {
    ensure!(
        items
            .iter()
            .all(|item| item.project_root == root && item.continuity_id == continuity),
        "temporal work-item record contains a different scope"
    );
    Ok(())
}

/// True only for snapshots that explicitly include authoritative temporal items.
/// The table and the marker must agree; legacy databases have neither.
pub(super) fn has_coverage(conn: &Connection) -> Result<bool> {
    let table_exists = |name: &str| -> rusqlite::Result<bool> {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1)",
            [name],
            |row| row.get(0),
        )
    };
    let marker: Option<String> = if table_exists("meta")? {
        conn.query_row("SELECT value FROM meta WHERE key=?1", [MIGRATION], |row| {
            row.get(0)
        })
        .optional()?
    } else {
        None
    };
    match (
        marker.as_deref(),
        table_exists("temporal_work_item_scopes")?,
    ) {
        (Some("1"), true) => Ok(true),
        (None, false) => Ok(false),
        _ => bail!("invalid temporal work-item coverage: migration marker and table disagree"),
    }
}

pub(super) fn verify_coverage(conn: &Connection) -> Result<bool> {
    if !has_coverage(conn)? {
        return Ok(false);
    }
    let mut statement = conn.prepare(
        "SELECT project_root, continuity_id, version, items_json FROM temporal_work_item_scopes",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let root: String = row.get(0)?;
        let continuity: String = row.get(1)?;
        let version: i64 = row.get(2)?;
        let json: String = row.get(3)?;
        ensure!(version > 0, "invalid temporal work-item scope version");
        validate_scope(
            &serde_json::from_str::<Vec<WorkItem>>(&json)?,
            &root,
            &continuity,
        )?;
    }
    Ok(true)
}

pub(super) fn require_native_capture(conn: &Connection, data_dir: &Path) -> Result<bool> {
    let covered = verify_coverage(conn)?;
    ensure!(
        covered || legacy_files(data_dir)?.is_empty(),
        "native backup would omit legacy temporal work items; migrate them by starting the upgraded daemon first"
    );
    Ok(covered)
}

pub(super) fn migrate(conn: &mut Connection, data_dir: &Path) -> Result<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if has_coverage(&tx)? {
        return Ok(());
    }
    tx.execute_batch(
        "CREATE TABLE temporal_work_item_scopes (
        project_root TEXT NOT NULL,
        continuity_id TEXT NOT NULL,
        version INTEGER NOT NULL CHECK(version > 0),
        items_json TEXT NOT NULL CHECK(json_type(items_json) = 'array'),
        PRIMARY KEY(project_root, continuity_id)
    ) WITHOUT ROWID",
    )?;
    for path in legacy_files(data_dir)? {
        let items: Vec<WorkItem> =
            serde_json::from_slice(&fs::read(&path)?).with_context(|| {
                format!(
                    "invalid legacy temporal work-item record {}",
                    path.display()
                )
            })?;
        let Some(first) = items.first() else { continue };
        let (root, continuity) = (&first.project_root, &first.continuity_id);
        validate_scope(&items, root, continuity)?;
        let digest = hex::encode(Sha256::digest(format!("{root}\0{continuity}")));
        ensure!(
            path.file_stem().and_then(|name| name.to_str()) == Some(digest.as_str()),
            "legacy temporal work-item file does not match its recorded scope"
        );
        tx.execute(
            "INSERT INTO temporal_work_item_scopes VALUES (?1, ?2, 1, ?3)",
            params![root, continuity, serde_json::to_string(&items)?],
        )?;
    }
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, '1')",
        [MIGRATION],
    )?;
    tx.commit()?;
    Ok(())
}

impl SqlitePersistence {
    pub fn read_temporal_work_items(
        &self,
        root: &str,
        continuity: &str,
    ) -> Result<WorkItemSnapshot> {
        self.with_connection_mut(|db| {
            let row: Option<(i64, String)> = db.query_row(
                "SELECT version, items_json FROM temporal_work_item_scopes WHERE project_root=?1 AND continuity_id=?2",
                params![root, continuity], |r| Ok((r.get(0)?, r.get(1)?)),
            ).optional()?;
            let (version, items) = match row {
                Some((version, json)) => (Some(version), serde_json::from_str::<Vec<WorkItem>>(&json)?),
                None => (None, Vec::new()),
            };
            validate_scope(&items, root, continuity)?;
            Ok(WorkItemSnapshot { items, scope: (root.to_owned(), continuity.to_owned()), version })
        })
    }

    /// A concurrent request must reread/requalify rather than replace admitted state.
    pub fn commit_temporal_work_items(&self, snapshot: WorkItemSnapshot) -> Result<bool> {
        let (root, continuity) = snapshot.scope;
        validate_scope(&snapshot.items, &root, &continuity)?;
        let json = serde_json::to_string(&snapshot.items)?;
        self.with_connection_mut(|db| {
            let modified = match snapshot.version {
                Some(version) => db.execute(
                    "UPDATE temporal_work_item_scopes SET version=version+1, items_json=?3 WHERE project_root=?1 AND continuity_id=?2 AND version=?4",
                    params![root, continuity, json, version],
                )?,
                None => db.execute(
                    "INSERT INTO temporal_work_item_scopes VALUES (?1, ?2, 1, ?3) ON CONFLICT(project_root, continuity_id) DO NOTHING",
                    params![root, continuity, json],
                )?,
            };
            Ok(modified == 1)
        })
    }
}
