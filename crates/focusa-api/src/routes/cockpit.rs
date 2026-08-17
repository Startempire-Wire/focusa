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
    focusa_core::workset_store::ensure_schema(conn)?;
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
        let events =
            focusa_core::workset_store::list_events(conn, &workset_id).unwrap_or_default();
        let projection =
            focusa_core::workset_ledger::replay_projection(&definition, &events).ok();
        let (status, met, open) = match &projection {
            Some(p) => {
                let met: Vec<String> = p
                    .requirements
                    .values()
                    .filter(|req| req.disposition.is_some())
                    .map(|req| req.requirement_id.clone())
                    .collect();
                let open: Vec<String> = p
                    .requirements
                    .values()
                    .filter(|req| req.disposition.is_none())
                    .map(|req| req.requirement_id.clone())
                    .collect();
                (
                    if p.settled { "settled" } else { "in_progress" }.to_string(),
                    met,
                    open,
                )
            }
            None => (String::from("unparsable"), vec![], vec![]),
        };
        out.push(json!({
            "workset_id": workset_id,
            "revision": revision,
            "status": status,
            "met": met.len(),
            "open": open.len(),
            "digest": focusa_core::workset_ledger::workset_digest(&definition),
        }));
    }
    Ok(out)
}

fn callgraph_frontiers(conn: &rusqlite::Connection) -> anyhow::Result<Vec<Value>> {
    focusa_core::callgraph_store::ensure_schema(conn)?;
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
        let Some(stored) = focusa_core::callgraph_store::load_definition(conn, &graph_id, revision.try_into().unwrap_or(u64::MAX))?
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
    focusa_core::direction_ledger::ensure_schema(conn)?;
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
    focusa_core::background_job_store::ensure_schema(conn)?;
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

pub async fn projection(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
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
    })
    .await;
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
