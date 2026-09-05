//! Cockpit projection — the hand-in-glove operator surface.
//!
//! Gap E of docs/170: one typed, bounded payload joining the workset
//! projection, open CallGraph run frontiers, direction steers, and the
//! background-job board (with EMAs) so the operator and extensions see
//! the whole flywheel from a single read. Read-only, ledger-backed.
//!
//! This route composes the same stores the family routes already own;
//! no new write surface.

use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

const MAX_WORKSETS: usize = 50;
const MAX_RUNS: usize = 20;
const MAX_JOBS: usize = 30;

fn workset_summaries(conn: &rusqlite::Connection) -> anyhow::Result<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT workset_id, revision, definition_json FROM worksets ORDER BY workset_id, revision",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (workset_id, revision, definition_json) = row?;
        let definition: focusa_core::workset_ledger::WorksetDefinition =
            serde_json::from_str(&definition_json)?;
        let events = focusa_core::workset_store::list_events(conn, &workset_id)?;
        let projection = focusa_core::workset_ledger::replay_projection(&definition, &events)
            .map_err(anyhow::Error::msg)?;
        let met = projection
            .requirements
            .values()
            .filter(|req| req.disposition.is_some())
            .count();
        let open = projection
            .requirements
            .values()
            .filter(|req| req.disposition.is_none())
            .count();
        let status = if projection.settled {
            "settled"
        } else {
            "in_progress"
        };
        out.push(json!({
            "workset_id": workset_id,
            "revision": revision,
            "status": status,
            "met": met,
            "open": open,
            "digest": focusa_core::workset_ledger::workset_digest(&definition),
        }));
    }
    Ok(out)
}

fn callgraph_frontiers(conn: &rusqlite::Connection) -> anyhow::Result<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT run_id, graph_id, revision, state FROM callgraph_runs WHERE state IN ('running','paused') ORDER BY updated_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([MAX_RUNS as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (run_id, graph_id, revision, state) = row?;
        let Some(stored) = focusa_core::callgraph_store::load_definition(
            conn,
            &graph_id,
            revision.try_into().unwrap_or(u64::MAX),
        )?
        else {
            continue;
        };
        let graph: focusa_core::callgraph::FocusaCallGraphDefinition =
            serde_json::from_str(&stored.definition_json)?;
        let dispatches = focusa_core::callgraph_store::list_dispatches(conn, &run_id)?;
        let frontier = focusa_core::callgraph::replay_frontier(&graph, &dispatches);
        out.push(json!({
            "run_id": run_id,
            "graph_id": graph_id,
            "state": state,
            "frontier": frontier,
        }));
    }
    Ok(out)
}

fn direction_steers(conn: &rusqlite::Connection) -> anyhow::Result<Vec<Value>> {
    Ok(focusa_core::direction_ledger::list_operations(conn)?
        .into_iter()
        .filter(|receipt| {
            matches!(
                receipt.operation,
                focusa_core::direction_operations::DirectionOperation::Steer { .. }
            )
        })
        .map(|receipt| {
            let focusa_core::direction_operations::DirectionOperation::Steer {
                target_ref,
                direction,
                rationale,
                ..
            } = receipt.operation
            else {
                unreachable!()
            };
            json!({
                "target_ref": target_ref,
                "direction": direction,
                "rationale": rationale,
                "recorded_at": receipt.recorded_at,
            })
        })
        .collect())
}

fn background_board(conn: &rusqlite::Connection) -> anyhow::Result<Value> {
    let jobs = focusa_core::background_job_store::list_jobs(conn)?;
    let active = jobs
        .iter()
        .filter(|job| job.completed_at.is_none())
        .take(MAX_JOBS)
        .map(|job| {
            json!({
                "job_id": job.job_id,
                "name": job.name,
                "status": job.status,
                "started_at": job.started_at,
                "eta_ms": focusa_core::background_job_store::eta_ms_for(conn, &job.name).ok().flatten(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "active": active.len(),
        "jobs": active,
    }))
}

// Schema initialization belongs to the owning writers, never to a read.
fn read_projection(path: &std::path::Path) -> anyhow::Result<Value> {
    let conn =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let worksets = workset_summaries(&conn)
        .map(|items| items.into_iter().take(MAX_WORKSETS).collect::<Vec<_>>())?;
    let callgraph = callgraph_frontiers(&conn)?;
    let steers = direction_steers(&conn)?;
    let background = background_board(&conn)?;
    Ok(json!({
        "status": "ok",
        "worksets": worksets,
        "callgraph": callgraph,
        "steers": steers,
        "background": background,
    }))
}

pub async fn projection(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || read_projection(&path)).await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("{error}"),
        )),
    }
}

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/cockpit/projection", axum::routing::get(projection))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("focusa-cockpit-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn absent_database_is_not_created() {
        let dir = fixture_dir();
        let path = dir.join("focusa.sqlite");
        assert!(read_projection(&path).is_err());
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn read_collectors_do_not_initialize_missing_schemas() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        assert!(workset_summaries(&conn).is_err());
        assert!(callgraph_frontiers(&conn).is_err());
        assert!(direction_steers(&conn).is_err());
        assert!(background_board(&conn).is_err());
        let count: i64 = conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn missing_schema_preserves_existing_database() {
        let dir = fixture_dir();
        let path = dir.join("focusa.sqlite");
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute("CREATE TABLE sentinel (value TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO sentinel VALUES ('preserve-me')", [])
            .unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        assert!(read_projection(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn initialized_database_can_be_read_without_changes() {
        let dir = fixture_dir();
        let path = dir.join("focusa.sqlite");
        let conn = rusqlite::Connection::open(&path).unwrap();
        focusa_core::workset_store::ensure_schema(&conn).unwrap();
        focusa_core::callgraph_store::ensure_schema(&conn).unwrap();
        focusa_core::direction_ledger::ensure_schema(&conn).unwrap();
        focusa_core::background_job_store::ensure_schema(&conn).unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        let result = read_projection(&path).unwrap();
        assert_eq!(result["status"], "ok");
        assert_eq!(result["worksets"], json!([]));
        assert_eq!(result["background"]["active"], 0);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    fn workset_fixture() -> rusqlite::Connection {
        use focusa_core::workset_ledger::{CompletionContract, WorksetDefinition, WorksetScope};
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        focusa_core::workset_store::ensure_schema(&conn).unwrap();
        let definition = WorksetDefinition {
            schema: focusa_core::workset_ledger::WORKSET_LEDGER_SCHEMA.to_string(),
            workset_id: "test-workset".to_string(),
            revision: 1,
            scope: WorksetScope {
                project_root: "/test-project".to_string(),
                continuity_id: "test-continuity".to_string(),
            },
            completion_contract: CompletionContract {
                required_requirement_ids: vec!["requirement-1".to_string()],
                release_gate_ref: None,
            },
        };
        focusa_core::workset_store::upsert_definition(&conn, &definition).unwrap();
        conn
    }

    #[test]
    fn invalid_history_cannot_be_reported_as_empty_success() {
        use focusa_core::workset_ledger::{RequirementDisposition, WorksetEvent};
        let conn = workset_fixture();
        focusa_core::workset_store::append_event(
            &conn,
            "test-workset",
            &WorksetEvent::RequirementDisposed {
                requirement_id: "never-admitted".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: None,
            },
        )
        .unwrap();
        assert!(
            workset_summaries(&conn)
                .unwrap_err()
                .to_string()
                .contains("disposed before admission")
        );
        assert_eq!(
            focusa_core::workset_store::list_events(&conn, "test-workset")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn unknown_definition_schema_is_not_success() {
        let conn = workset_fixture();
        let mut definition = focusa_core::workset_store::load_definition(&conn, "test-workset", 1)
            .unwrap()
            .unwrap();
        definition.schema = "unknown-schema".to_string();
        focusa_core::workset_store::upsert_definition(&conn, &definition).unwrap();
        assert!(
            workset_summaries(&conn)
                .unwrap_err()
                .to_string()
                .contains("unexpected schema")
        );
    }

    #[test]
    fn corrupt_workset_events_cannot_be_reported_as_empty_success() {
        let conn = workset_fixture();
        assert_eq!(workset_summaries(&conn).unwrap().len(), 1);
        conn.execute(
            "INSERT INTO workset_events (workset_id, event_json, recorded_at)
             VALUES ('test-workset', '{bad-json', 'test-fixture')",
            [],
        )
        .unwrap();
        assert_eq!(
            workset_summaries(&conn).unwrap_err().to_string(),
            "invalid stored Workset event"
        );
        let raw: String = conn
            .query_row(
                "SELECT event_json FROM workset_events WHERE workset_id = 'test-workset'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(raw, "{bad-json");
    }
}
