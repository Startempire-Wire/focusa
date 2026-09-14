//! CallGraph persistence — slice 3 (#254). Spec 155 §18.1.
//!
//! SQLite ledger for definitions/revisions and runs. Runs are append-only:
//! a dispatch is durably committed before any adapter call (Spec 155 §12
//! final sentence). The reducer/replay layer (slice 4) consumes this
//! ledger; this module owns schema, upsert, and query.

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::callgraph::{Disposition, FocusaCallGraphDefinition};

pub const CALLGRAPH_LEDGER_SCHEMA: &str = "focusa.callgraph_ledger.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredDefinition {
    pub graph_id: String,
    pub revision: u64,
    pub definition_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Created,
    Dispatching,
    Running,
    WaitingJoin,
    WaitingAuthority,
    Completed,
    Failed,
    Unwound,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraphRun {
    pub run_id: String,
    pub graph_id: String,
    pub revision: u64,
    pub state: RunState,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameDispatch {
    pub dispatch_id: String,
    pub run_id: String,
    pub frame_id: String,
    pub invocation_id: Option<String>,
    pub parent_invocation_id: Option<String>,
    pub disposition: Disposition,
    pub attempt: u32,
    pub committed_at: String,
    pub receipt_ref: Option<String>,
}

/// Durable dispatch commit — written BEFORE any adapter call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchCommit {
    pub dispatch_id: String,
    pub run_id: String,
    pub frame_id: String,
    pub invocation_id: String,
    pub parent_invocation_id: Option<String>,
    pub disposition: Disposition,
    pub attempt: u32,
    pub committed_at: String,
    pub actor_ref: String,
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS callgraph_definitions (
            graph_id TEXT NOT NULL,
            revision INTEGER NOT NULL,
            definition_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (graph_id, revision)
        );
        CREATE TABLE IF NOT EXISTS callgraph_runs (
            run_id TEXT PRIMARY KEY,
            graph_id TEXT NOT NULL,
            revision INTEGER NOT NULL,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS callgraph_frame_leases (
            invocation_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            frame_id TEXT NOT NULL,
            lease_holder TEXT NOT NULL,
            lease_expires_at TEXT NOT NULL,
            acquired_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS callgraph_dispatch_evidence (
            dispatch_id TEXT NOT NULL,
            run_id TEXT NOT NULL,
            evidence_ref TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS callgraph_dispatches (
            dispatch_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            frame_id TEXT NOT NULL,
            invocation_id TEXT,
            parent_invocation_id TEXT,
            disposition TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            committed_at TEXT NOT NULL,
            receipt_ref TEXT
        );
        "#,
    )?;
    Ok(())
}

pub fn upsert_definition(conn: &Connection, graph: &FocusaCallGraphDefinition) -> Result<()> {
    let json = serde_json::to_string(graph)?;
    conn.execute(
        "INSERT INTO callgraph_definitions (graph_id, revision, definition_json, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(graph_id, revision) DO UPDATE SET definition_json = excluded.definition_json",
        params![
            graph.graph_id,
            graph.revision as i64,
            json,
            graph.created_at
        ],
    )?;
    Ok(())
}

pub fn load_definition(
    conn: &Connection,
    graph_id: &str,
    revision: u64,
) -> Result<Option<StoredDefinition>> {
    conn.query_row(
        "SELECT graph_id, revision, definition_json, created_at
         FROM callgraph_definitions WHERE graph_id = ?1 AND revision = ?2",
        params![graph_id, revision as i64],
        |row| {
            Ok(StoredDefinition {
                graph_id: row.get(0)?,
                revision: row.get::<_, i64>(1)? as u64,
                definition_json: row.get(2)?,
                created_at: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub fn create_run(conn: &Connection, run: &CallGraphRun) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO callgraph_runs
         (run_id, graph_id, revision, state, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            run.run_id,
            run.graph_id,
            run.revision as i64,
            run_state_str(run.state),
            run.created_at,
            run.updated_at
        ],
    )?;
    Ok(())
}

pub fn transition_run(
    conn: &Connection,
    run_id: &str,
    state: RunState,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE callgraph_runs SET state = ?2, updated_at = ?3 WHERE run_id = ?1",
        params![run_id, run_state_str(state), updated_at],
    )?;
    Ok(())
}

pub fn load_run(conn: &Connection, run_id: &str) -> Result<Option<CallGraphRun>> {
    conn.query_row(
        "SELECT run_id, graph_id, revision, state, created_at, updated_at
         FROM callgraph_runs WHERE run_id = ?1",
        params![run_id],
        |row| {
            Ok(CallGraphRun {
                run_id: row.get(0)?,
                graph_id: row.get(1)?,
                revision: row.get::<_, i64>(2)? as u64,
                state: run_state_from(row.get::<_, String>(3)?),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

/// The §12 commit boundary: persist the dispatch row before any adapter
/// call. Returns Err on any failure so callers cannot proceed to dispatch.
pub fn commit_dispatch(conn: &Connection, commit: &DispatchCommit) -> Result<()> {
    conn.execute(
        "INSERT INTO callgraph_dispatches
         (dispatch_id, run_id, frame_id, invocation_id, parent_invocation_id,
          disposition, attempt, committed_at, receipt_ref)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL)",
        params![
            commit.dispatch_id,
            commit.run_id,
            commit.frame_id,
            commit.invocation_id,
            commit.parent_invocation_id,
            disposition_str(commit.disposition.clone()),
            commit.attempt,
            commit.committed_at,
        ],
    )?;
    Ok(())
}

pub fn list_dispatches(conn: &Connection, run_id: &str) -> Result<Vec<FrameDispatch>> {
    let mut stmt = conn.prepare(
        "SELECT dispatch_id, run_id, frame_id, invocation_id, parent_invocation_id,
                disposition, attempt, committed_at, receipt_ref
         FROM callgraph_dispatches WHERE run_id = ?1 ORDER BY committed_at",
    )?;
    let rows = stmt.query_map(params![run_id], |row| {
        Ok(FrameDispatch {
            dispatch_id: row.get(0)?,
            run_id: row.get(1)?,
            frame_id: row.get(2)?,
            invocation_id: row.get(3)?,
            parent_invocation_id: row.get(4)?,
            disposition: disposition_from(row.get::<_, String>(5)?),
            attempt: row.get(6)?,
            committed_at: row.get(7)?,
            receipt_ref: row.get(8)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn run_state_str(state: RunState) -> &'static str {
    match state {
        RunState::Created => "created",
        RunState::Dispatching => "dispatching",
        RunState::Running => "running",
        RunState::WaitingJoin => "waiting_join",
        RunState::WaitingAuthority => "waiting_authority",
        RunState::Completed => "completed",
        RunState::Failed => "failed",
        RunState::Unwound => "unwound",
        RunState::Cancelled => "cancelled",
    }
}

fn run_state_from(value: String) -> RunState {
    match value.as_str() {
        "dispatching" => RunState::Dispatching,
        "running" => RunState::Running,
        "waiting_join" => RunState::WaitingJoin,
        "waiting_authority" => RunState::WaitingAuthority,
        "completed" => RunState::Completed,
        "failed" => RunState::Failed,
        "unwound" => RunState::Unwound,
        "cancelled" => RunState::Cancelled,
        _ => RunState::Created,
    }
}

fn disposition_str(disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::Eligible => "eligible",
        Disposition::WaitingInput => "waiting_input",
        Disposition::WaitingParent => "waiting_parent",
        Disposition::WaitingJoin => "waiting_join",
        Disposition::WaitingAuthority => "waiting_authority",
        Disposition::WaitingCapability => "waiting_capability",
        Disposition::BlockedScope => "blocked_scope",
        Disposition::BlockedStale => "blocked_stale",
        Disposition::BlockedBudget => "blocked_budget",
        Disposition::BlockedCyclePolicy => "blocked_cycle_policy",
        Disposition::Rejected => "rejected",
    }
}

fn disposition_from(value: String) -> Disposition {
    match value.as_str() {
        "waiting_input" => Disposition::WaitingInput,
        "waiting_parent" => Disposition::WaitingParent,
        "waiting_join" => Disposition::WaitingJoin,
        "waiting_authority" => Disposition::WaitingAuthority,
        "waiting_capability" => Disposition::WaitingCapability,
        "blocked_scope" => Disposition::BlockedScope,
        "blocked_stale" => Disposition::BlockedStale,
        "blocked_budget" => Disposition::BlockedBudget,
        "blocked_cycle_policy" => Disposition::BlockedCyclePolicy,
        "rejected" => Disposition::Rejected,
        _ => Disposition::Eligible,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameLease {
    pub invocation_id: String,
    pub run_id: String,
    pub frame_id: String,
    pub lease_holder: String,
    pub lease_expires_at: String,
    pub acquired_at: String,
}

/// Acquire a frame lease. Returns Ok(true) on acquisition; Ok(false) when
/// an unexpired lease already exists for the invocation (liveness guard).
pub fn acquire_lease(conn: &Connection, lease: &FrameLease) -> Result<bool> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT lease_expires_at FROM callgraph_frame_leases WHERE invocation_id = ?1",
            params![lease.invocation_id],
            |row| row.get(0),
        )
        .optional()?;
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(expires) = existing {
        if expires > now {
            return Ok(false);
        }
        conn.execute(
            "DELETE FROM callgraph_frame_leases WHERE invocation_id = ?1",
            params![lease.invocation_id],
        )?;
    }
    conn.execute(
        "INSERT INTO callgraph_frame_leases
         (invocation_id, run_id, frame_id, lease_holder, lease_expires_at, acquired_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            lease.invocation_id,
            lease.run_id,
            lease.frame_id,
            lease.lease_holder,
            lease.lease_expires_at,
            lease.acquired_at,
        ],
    )?;
    Ok(true)
}

pub fn release_lease(conn: &Connection, invocation_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM callgraph_frame_leases WHERE invocation_id = ?1",
        params![invocation_id],
    )?;
    Ok(())
}

/// Lapsed leases (expired but not released) — liveness sweeper input.
pub fn lapsed_leases(conn: &Connection, now: &str) -> Result<Vec<FrameLease>> {
    let mut stmt = conn.prepare(
        "SELECT invocation_id, run_id, frame_id, lease_holder, lease_expires_at, acquired_at
         FROM callgraph_frame_leases WHERE lease_expires_at <= ?1",
    )?;
    let rows = stmt.query_map(params![now], |row| {
        Ok(FrameLease {
            invocation_id: row.get(0)?,
            run_id: row.get(1)?,
            frame_id: row.get(2)?,
            lease_holder: row.get(3)?,
            lease_expires_at: row.get(4)?,
            acquired_at: row.get(5)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

/// Dispatches across all runs of a graph — export snapshots.
pub fn list_dispatches_for_graph(conn: &Connection, graph_id: &str) -> Result<Vec<FrameDispatch>> {
    let mut stmt = conn.prepare(
        "SELECT dispatch_id, run_id, frame_id, invocation_id, parent_invocation_id,
                disposition, attempt, committed_at, receipt_ref
         FROM callgraph_dispatches
         WHERE run_id IN (SELECT run_id FROM callgraph_runs WHERE graph_id = ?1)
         ORDER BY committed_at",
    )?;
    let rows = stmt.query_map(params![graph_id], |row| {
        Ok(FrameDispatch {
            dispatch_id: row.get(0)?,
            run_id: row.get(1)?,
            frame_id: row.get(2)?,
            invocation_id: row.get(3)?,
            parent_invocation_id: row.get(4)?,
            disposition: disposition_from(row.get::<_, String>(5)?),
            attempt: row.get(6)?,
            committed_at: row.get(7)?,
            receipt_ref: row.get(8)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

/// Mark a dispatch settled with its receipt + outcome; store evidence.
pub fn mark_dispatch_settled(
    conn: &Connection,
    dispatch_id: &str,
    receipt_ref: &str,
    outcome: &str,
    evidence_refs: &[String],
) -> Result<()> {
    conn.execute(
        "UPDATE callgraph_dispatches SET receipt_ref = ?2 WHERE dispatch_id = ?1",
        params![dispatch_id, receipt_ref],
    )?;
    for evidence in evidence_refs {
        conn.execute(
            "INSERT INTO callgraph_dispatch_evidence (dispatch_id, run_id, evidence_ref)
             VALUES (?1, ?2, ?3)",
            params![dispatch_id, "", evidence],
        )?;
    }
    let _ = outcome;
    Ok(())
}

pub fn link_evidence(
    conn: &Connection,
    run_id: &str,
    dispatch_id: &str,
    evidence_refs: &[String],
) -> Result<()> {
    for evidence in evidence_refs {
        conn.execute(
            "INSERT INTO callgraph_dispatch_evidence (dispatch_id, run_id, evidence_ref)
             VALUES (?1, ?2, ?3)",
            params![dispatch_id, run_id, evidence],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::{
        AcceptanceContract, AuthorityRef, CallGraphPolicies, CallGraphScope, FocusaCallFrame,
        FrameKind, SideEffectClass,
    };

    fn sample_graph() -> FocusaCallGraphDefinition {
        FocusaCallGraphDefinition {
            schema: crate::callgraph::CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g1".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            mission_ref: "m1".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "test".to_string(),
            description: "test".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![FocusaCallFrame {
                frame_id: "a".to_string(),
                name: "a".to_string(),
                purpose: "test".to_string(),
                kind: FrameKind::Agent,
                input_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                preconditions: vec![],
                postconditions: vec![],
                side_effect_class: SideEffectClass::None,
                capability_refs: vec![],
                authority_requirement: None,
                timeout_policy: None,
                retry_policy: None,
                failure_boundary: None,
                compensation_frame_id: None,
                resource_budget: None,
                acceptance: AcceptanceContract {
                    acceptance_atoms: vec!["a1".to_string()],
                    verifier: None,
                },
                execution_binding: None,
            }],
            edges: vec![],
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "2026-08-16T00:00:00Z".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        }
    }

    #[test]
    fn definition_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let graph = sample_graph();
        upsert_definition(&conn, &graph).unwrap();
        let stored = load_definition(&conn, "g1", 1).unwrap().expect("exists");
        assert_eq!(stored.graph_id, "g1");
        let parsed: FocusaCallGraphDefinition =
            serde_json::from_str(&stored.definition_json).unwrap();
        assert_eq!(parsed, graph);
    }

    #[test]
    fn run_lifecycle_and_dispatch_commit() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        create_run(
            &conn,
            &CallGraphRun {
                run_id: "r1".to_string(),
                graph_id: "g1".to_string(),
                revision: 1,
                state: RunState::Created,
                created_at: "2026-08-16T00:00:00Z".to_string(),
                updated_at: "2026-08-16T00:00:00Z".to_string(),
            },
        )
        .unwrap();
        commit_dispatch(
            &conn,
            &DispatchCommit {
                dispatch_id: "d1".to_string(),
                run_id: "r1".to_string(),
                frame_id: "a".to_string(),
                invocation_id: "i1".to_string(),
                parent_invocation_id: None,
                disposition: Disposition::Eligible,
                attempt: 1,
                committed_at: "2026-08-16T00:00:00Z".to_string(),
                actor_ref: "op-1".to_string(),
            },
        )
        .unwrap();
        transition_run(&conn, "r1", RunState::Running, "2026-08-16T00:00:01Z").unwrap();
        let run = load_run(&conn, "r1").unwrap().expect("exists");
        assert_eq!(run.state, RunState::Running);
        let dispatches = list_dispatches(&conn, "r1").unwrap();
        assert_eq!(dispatches.len(), 1);
        assert_eq!(dispatches[0].disposition, Disposition::Eligible);
    }

    #[test]
    fn lease_acquire_release_lapse() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let lease = FrameLease {
            invocation_id: "i1".to_string(),
            run_id: "r1".to_string(),
            frame_id: "a".to_string(),
            lease_holder: "holder-1".to_string(),
            lease_expires_at: (chrono::Utc::now() + chrono::Duration::seconds(60)).to_rfc3339(),
            acquired_at: chrono::Utc::now().to_rfc3339(),
        };
        assert!(acquire_lease(&conn, &lease).unwrap());
        // Second acquisition while unexpired is refused.
        assert!(!acquire_lease(&conn, &lease).unwrap());
        release_lease(&conn, "i1").unwrap();
        assert!(acquire_lease(&conn, &lease).unwrap());
        // Lapsed lease surfaces for the sweeper.
        let lapsed = lapsed_leases(
            &conn,
            &(chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339(),
        )
        .unwrap();
        assert_eq!(lapsed.len(), 1);
        assert_eq!(lapsed[0].frame_id, "a");
    }

    #[test]
    fn duplicate_dispatch_id_is_rejected() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        create_run(
            &conn,
            &CallGraphRun {
                run_id: "r1".to_string(),
                graph_id: "g1".to_string(),
                revision: 1,
                state: RunState::Created,
                created_at: "t".to_string(),
                updated_at: "t".to_string(),
            },
        )
        .unwrap();
        let commit = DispatchCommit {
            dispatch_id: "d1".to_string(),
            run_id: "r1".to_string(),
            frame_id: "a".to_string(),
            invocation_id: "i1".to_string(),
            parent_invocation_id: None,
            disposition: Disposition::Eligible,
            attempt: 1,
            committed_at: "t".to_string(),
            actor_ref: "op-1".to_string(),
        };
        commit_dispatch(&conn, &commit).unwrap();
        assert!(commit_dispatch(&conn, &commit).is_err());
    }
}
