//! Workset API — #271 slice 1. Definitions, append-only events, and the
//! replay projection. No scheduling/execution (authority separation:
//! CallGraph owns execution).

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use focusa_core::workset_ledger::{WorksetDefinition, WorksetEvent, replay_projection};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/worksets", post(create_definition).get(list))
        .route("/v1/worksets/{workset_id}/events", post(append_event))
        .route("/v1/worksets/{workset_id}/projection", get(get_projection))
        .route("/v1/worksets/{workset_id}/freshness", get(get_freshness))
        .route(
            "/v1/worksets/{workset_id}/transition",
            post(evaluate_transition_route),
        )
}

async fn create_definition(
    State(state): State<Arc<AppState>>,
    Json(definition): Json<WorksetDefinition>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        focusa_core::workset_store::upsert_definition(&conn, &definition)?;
        Ok(json!({
            "status": "stored",
            "workset_id": definition.workset_id,
            "revision": definition.revision,
            "digest": focusa_core::workset_ledger::workset_digest(&definition),
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

async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT workset_id, revision, definition_json FROM worksets ORDER BY workset_id, revision",
        )?;
        let rows = stmt.query_map([], |row| {
            let definition: WorksetDefinition = serde_json::from_str(&row.get::<_, String>(2)?)
                .unwrap_or_else(|_| WorksetDefinition {
                    schema: focusa_core::workset_ledger::WORKSET_LEDGER_SCHEMA.to_string(),
                    workset_id: "unparsable".to_string(),
                    revision: 0,
                    scope: focusa_core::workset_ledger::WorksetScope { project_root: String::new(), continuity_id: String::new() },
                    completion_contract: focusa_core::workset_ledger::CompletionContract { required_requirement_ids: vec![], release_gate_ref: None },
                });
            let events = focusa_core::workset_store::list_events(&conn, &definition.workset_id).unwrap_or_default();
            let projection = focusa_core::workset_ledger::replay_projection(&definition, &events).ok();
            Ok(json!({
                "workset_id": definition.workset_id,
                "revision": row.get::<_, i64>(1)?,
                "digest": focusa_core::workset_ledger::workset_digest(&definition),
                "requirement_count": projection.as_ref().map(|p| p.requirements.len()).unwrap_or(0),
                "membership_count": projection.as_ref().map(|p| p.membership.len()).unwrap_or(0),
                "settled": projection.as_ref().map(|p| p.settled).unwrap_or(false),
            }))
        })?;
        Ok(json!({"status": "ok", "worksets": rows.collect::<rusqlite::Result<Vec<_>>>()?}))
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

async fn append_event(
    State(state): State<Arc<AppState>>,
    Path(workset_id): Path<String>,
    Json(event): Json<WorksetEvent>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        let seq = focusa_core::workset_store::append_event(&conn, &workset_id, &event)?;
        Ok(json!({"status": "appended", "workset_id": workset_id, "seq": seq}))
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

#[derive(serde::Deserialize)]
pub struct TransitionBody {
    pub from: focusa_core::workset_transitions::WorksetState,
    pub to: focusa_core::workset_transitions::WorksetState,
}

async fn evaluate_transition_route(
    State(state): State<Arc<AppState>>,
    Path(workset_id): Path<String>,
    Json(body): Json<TransitionBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        let definition = conn
            .query_row(
                "SELECT definition_json FROM worksets WHERE workset_id = ?1 ORDER BY revision DESC LIMIT 1",
                [&workset_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|raw| serde_json::from_str::<WorksetDefinition>(&raw).ok());
        let Some(definition) = definition else {
            return Ok(json!({"status": "missing", "workset_id": workset_id}));
        };
        let events = focusa_core::workset_store::list_events(&conn, &workset_id)?;
        match focusa_core::workset_transitions::evaluate_transition(
            &definition,
            &events,
            body.from,
            body.to,
        ) {
            Ok(verdict) => Ok(json!({"status": "evaluated", "verdict": verdict})),
            Err(reason) => Ok(json!({"status": "replay_rejected", "error": reason})),
        }
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

async fn get_freshness(
    State(state): State<Arc<AppState>>,
    Path(workset_id): Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        let definition = conn
            .query_row(
                "SELECT definition_json FROM worksets WHERE workset_id = ?1 ORDER BY revision DESC LIMIT 1",
                [&workset_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|raw| serde_json::from_str::<WorksetDefinition>(&raw).ok());
        let Some(definition) = definition else {
            return Ok(json!({"status": "missing", "workset_id": workset_id}));
        };
        let events = focusa_core::workset_store::list_events(&conn, &workset_id)?;
        match focusa_core::workset_freshness::canonical_stamp(&definition, &events) {
            Ok(stamp) => Ok(json!({"status": "ok", "stamp": stamp})),
            Err(reason) => Ok(json!({"status": "replay_rejected", "error": reason})),
        }
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

async fn get_projection(
    State(state): State<Arc<AppState>>,
    Path(workset_id): Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::workset_store::ensure_schema(&conn)?;
        let definition = conn
            .query_row(
                "SELECT definition_json FROM worksets WHERE workset_id = ?1 ORDER BY revision DESC LIMIT 1",
                [&workset_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|raw| serde_json::from_str::<WorksetDefinition>(&raw).ok());
        let Some(definition) = definition else {
            return Ok(json!({"status": "missing", "workset_id": workset_id}));
        };
        let events = focusa_core::workset_store::list_events(&conn, &workset_id)?;
        match replay_projection(&definition, &events) {
            Ok(projection) => Ok(json!({"status": "ok", "projection": projection})),
            Err(reason) => Ok(json!({
                "status": "replay_rejected",
                "failure_class": "workset_ledger_invalid",
                "retry_posture": "do_not_retry_unchanged",
                "safe_recovery": "inspect the ledger events for out-of-order dispositions",
                "error": reason,
            })),
        }
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
