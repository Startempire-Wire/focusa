//! Workstream-rooted canonical runtime — slice 1 (#125).
//!
//! One canonical runtime root per workstream: state, evidence, and
//! compaction partitions under the daemon data dir. Remote authority comes
//! from RemoteWorkspaceBinding (#89, docs/162); this module owns the
//! workstream's own identity, scope, and persistence. Design:
//! docs/164-workstream-rooted-canonical-runtime-design.md.
//!
//! Invariants:
//! 1. State mutations are workstream-scoped — a write must name the root.
//! 2. Compaction never mixes workstreams.
//! 3. Continuation resolves the workstream root first, the session second.
//! 4. Remote workstreams hold no authority state on the remote host.

use anyhow::{anyhow, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const WORKSTREAM_SCHEMA: &str = "focusa.workstream_root.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkstreamState {
    Active,
    Suspended,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootScope {
    pub scope_kind: String,
    pub remote_binding_id: Option<String>,
    pub canonical_root: String,
    pub working_subpath: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuity {
    pub continuity_id: String,
    pub principal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePartition {
    pub state_ref: String,
    pub evidence_ref: String,
    pub compaction_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkstreamRoot {
    pub schema: String,
    pub workstream_id: String,
    pub root_scope: RootScope,
    pub continuity: Continuity,
    pub runtime: RuntimePartition,
    pub state: WorkstreamState,
    pub created_at: String,
    pub updated_at: String,
}

impl WorkstreamRoot {
    /// Invariant 3: continuation identity resolves the workstream root
    /// first — this key is the canonical resolution order.
    pub fn resolution_key(&self) -> String {
        format!(
            "{}|{}|{}",
            self.workstream_id, self.continuity.continuity_id, self.root_scope.canonical_root
        )
    }

    /// Invariant 4: remote workstreams reference a binding; the runtime
    /// truth lives on the controller.
    pub fn is_remote(&self) -> bool {
        self.root_scope.scope_kind == "remote"
    }
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS workstream_roots (
           workstream_id TEXT PRIMARY KEY,
           continuity_id TEXT NOT NULL,
           canonical_root TEXT NOT NULL,
           state TEXT NOT NULL,
           root_json TEXT NOT NULL,
           created_at TEXT NOT NULL,
           updated_at TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_wsr_continuity ON workstream_roots(continuity_id);
         CREATE INDEX IF NOT EXISTS idx_wsr_state ON workstream_roots(state);",
    )?;
    Ok(())
}

/// Create or update a workstream root. Identity (workstream_id,
/// continuity_id, canonical_root) is immutable; conflicts are refused.
pub fn upsert_workstream(conn: &Connection, root: &WorkstreamRoot) -> Result<(bool, WorkstreamRoot)> {
    ensure_schema(conn)?;
    if root.schema != WORKSTREAM_SCHEMA {
        return Err(anyhow!("workstream schema must be {WORKSTREAM_SCHEMA}"));
    }
    if let Some((stored_continuity, stored_root)) = conn
        .query_row(
            "SELECT continuity_id, canonical_root FROM workstream_roots WHERE workstream_id = ?1",
            [&root.workstream_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if stored_continuity != root.continuity.continuity_id || stored_root != root.root_scope.canonical_root {
            return Err(anyhow!(
                "workstream identity is immutable: continuity/canonical_root differ from stored"
            ));
        }
    }
    let now = chrono::Utc::now().to_rfc3339();
    let root_json = serde_json::to_string(root)?;
    let existed: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM workstream_roots WHERE workstream_id = ?1)",
            [&root.workstream_id],
            |row| row.get(0),
        )?;
    conn.execute(
        "INSERT INTO workstream_roots
           (workstream_id, continuity_id, canonical_root, state, root_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(workstream_id) DO UPDATE SET
           state = excluded.state,
           root_json = excluded.root_json,
           updated_at = excluded.updated_at",
        rusqlite::params![
            root.workstream_id,
            root.continuity.continuity_id,
            root.root_scope.canonical_root,
            serde_json::to_string(&root.state)?,
            root_json,
            now,
            now,
        ],
    )?;
    Ok((!existed, root.clone()))
}

/// Invariant 1: mutations name the workstream root — this lookup is the
/// only canonical entry point for state access.
pub fn load_workstream(conn: &Connection, workstream_id: &str) -> Result<Option<WorkstreamRoot>> {
    ensure_schema(conn)?;
    let raw: Option<String> = conn
        .query_row(
            "SELECT root_json FROM workstream_roots WHERE workstream_id = ?1",
            [workstream_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|text| serde_json::from_str(&text).ok()))
}

pub fn list_workstreams(conn: &Connection) -> Result<Vec<WorkstreamRoot>> {
    ensure_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT root_json FROM workstream_roots ORDER BY updated_at DESC",
    )?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|raw| serde_json::from_str(&raw).ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn root(id: &str, continuity: &str, canonical_root: &str) -> WorkstreamRoot {
        WorkstreamRoot {
            schema: WORKSTREAM_SCHEMA.to_string(),
            workstream_id: id.to_string(),
            root_scope: RootScope {
                scope_kind: if canonical_root.starts_with("/home") {
                    "remote".into()
                } else {
                    "local".into()
                },
                remote_binding_id: None,
                canonical_root: canonical_root.to_string(),
                working_subpath: None,
            },
            continuity: Continuity {
                continuity_id: continuity.to_string(),
                principal: Some("team:planmarr".into()),
            },
            runtime: RuntimePartition {
                state_ref: format!("workstreams/{id}/state.sqlite"),
                evidence_ref: format!("workstreams/{id}/evidence"),
                compaction_ref: format!("workstreams/{id}/compaction"),
            },
            state: WorkstreamState::Active,
            created_at: "2026-08-15T00:00:00Z".into(),
            updated_at: "2026-08-15T00:00:00Z".into(),
        }
    }

    #[test]
    fn upsert_load_list_round_trip() {
        let conn = conn();
        let (created, _) = upsert_workstream(&conn, &root("ws1", "ptm-main", "/root/ws1")).unwrap();
        assert!(created);
        let loaded = load_workstream(&conn, "ws1").unwrap().unwrap();
        assert_eq!(loaded.continuity.continuity_id, "ptm-main");
        assert_eq!(list_workstreams(&conn).unwrap().len(), 1);
    }

    #[test]
    fn identity_is_immutable() {
        let conn = conn();
        upsert_workstream(&conn, &root("ws1", "ptm-main", "/root/ws1")).unwrap();
        let error = upsert_workstream(&conn, &root("ws1", "other-continuity", "/root/ws1")).unwrap_err();
        assert!(error.to_string().contains("immutable"));
    }

    #[test]
    fn resolution_key_orders_root_first() {
        let root = root("ws1", "ptm-main", "/home/planmarr/plan-the-marriage");
        assert!(root.resolution_key().starts_with("ws1|ptm-main|/home/planmarr"));
        assert!(root.is_remote());
    }
}
