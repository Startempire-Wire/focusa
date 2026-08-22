//! CallGraph HTTP surface (#254 slice 2) — Spec 155 §19.1 (first routes).
//!
//! Validation and eligibility are pure core functions; this module wraps
//! them in typed HTTP responses. Definition storage and the run ledger
//! arrive in slice 3+.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use focusa_core::callgraph::{
    Disposition, FocusaCallGraphDefinition, eligibility_for_frame, validate_graph,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/callgraphs/validate", post(validate))
        .route("/v1/callgraphs/eligibility", post(eligibility))
        .route(
            "/v1/callgraphs",
            post(create_definition).get(list_definitions),
        )
        .route(
            "/v1/callgraphs/{graph_id}/runs/preflight",
            post(preflight_run),
        )
        .route("/v1/callgraphs/{graph_id}/runs", post(create_run))
        .route("/v1/callgraph-runs/{run_id}", get(get_run))
        .route("/v1/callgraphs/{graph_id}/export", get(export_graph))
        .route(
            "/v1/callgraph-items/{graph_id}/{frame_id}",
            get(get_item_envelope),
        )
        .route("/v1/callgraph-runs/{run_id}/control", post(control_run))
        .route("/v1/callgraph-runs/{run_id}/settle", post(settle_frame))
        .route("/v1/callgraph-runs/{run_id}/events", get(get_run_events))
        .route(
            "/v1/callgraph-runs/{run_id}/flowmesh-bindings/preflight",
            post(flowmesh_preflight),
        )
        .route(
            "/v1/callgraph-runs/{run_id}/flowmesh-bindings/execute",
            post(flowmesh_execute),
        )
        .route(
            "/v1/callgraph-runs/{run_id}/evidence/link",
            post(link_evidence),
        )
        .route("/v1/callgraph-runs/{run_id}/paths", get(get_run_paths))
        .route("/v1/callgraph-runs/{run_id}/frontier", get(get_run_paths))
}

#[derive(Deserialize)]
pub struct EligibilityBody {
    pub graph: FocusaCallGraphDefinition,
    pub frame_id: String,
    #[serde(default)]
    pub parent_frame_id: Option<String>,
    #[serde(default)]
    pub settled_edges: Vec<String>,
}

async fn validate(
    State(_state): State<Arc<AppState>>,
    Json(graph): Json<FocusaCallGraphDefinition>,
) -> Json<Value> {
    let report = validate_graph(&graph);
    Json(json!({
        "status": if report.valid { "valid" } else { "invalid" },
        "valid": report.valid,
        "issues": report.issues,
        "graph_id": graph.graph_id,
        "revision": graph.revision,
    }))
}

async fn eligibility(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<EligibilityBody>,
) -> Json<Value> {
    let settled: HashSet<String> = body.settled_edges.into_iter().collect();
    let disposition = eligibility_for_frame(
        &body.graph,
        &body.frame_id,
        body.parent_frame_id.as_deref(),
        &settled,
    );
    Json(json!({
        "status": "computed",
        "frame_id": body.frame_id,
        "disposition": disposition,
    }))
}

/// Persist a validated definition (Spec 155 §19.1 POST /v1/callgraphs).
async fn create_definition(
    State(state): State<Arc<AppState>>,
    Json(graph): Json<FocusaCallGraphDefinition>,
) -> Json<Value> {
    let report = validate_graph(&graph);
    if !report.valid {
        return Json(json!({
            "status": "rejected_invalid",
            "issues": report.issues,
        }));
    }
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let graph_id = graph.graph_id.clone();
    let revision = graph.revision;
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        focusa_core::callgraph_store::upsert_definition(&conn, &graph)?;
        Ok(())
    })
    .await;
    match result {
        Ok(Ok(())) => Json(json!({
            "status": "stored",
            "graph_id": graph_id,
            "revision": revision,
        })),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

/// List stored definition revisions (Spec 155 §19.1 GET /v1/callgraphs).
#[derive(Deserialize)]
pub struct ListDefinitionsQuery {
    pub graph_id: String,
}

async fn list_definitions(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<ListDefinitionsQuery>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let graph_id = query.graph_id.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u64>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT revision FROM callgraph_definitions WHERE graph_id = ?1 ORDER BY revision",
        )?;
        let rows = stmt.query_map([query.graph_id], |row| row.get::<_, i64>(0))?;
        Ok(rows
            .collect::<rusqlite::Result<Vec<i64>>>()?
            .into_iter()
            .map(|revision| revision as u64)
            .collect())
    })
    .await;
    match result {
        Ok(Ok(revisions)) => Json(json!({
            "status": "ok",
            "graph_id": graph_id,
            "revisions": revisions,
        })),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

/// Preflight a run for a stored graph revision (Spec 155 §19.1).
async fn preflight_run(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(graph_id): axum::extract::Path<String>,
    Json(body): Json<PreflightBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let revision = body.revision;
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let stored = focusa_core::callgraph_store::load_definition(&conn, &graph_id, revision)?;
        let Some(stored) = stored else {
            return Ok(json!({
                "status": "rejected_missing_definition",
                "graph_id": graph_id,
                "revision": revision,
            }));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        let report = validate_graph(&graph);
        if !report.valid {
            return Ok(json!({
                "status": "rejected_invalid",
                "issues": report.issues,
            }));
        }
        let mut blockers = Vec::new();
        for entry in &graph.entry_frame_ids {
            let disposition =
                eligibility_for_frame(&graph, entry, None, &std::collections::HashSet::new());
            if disposition != Disposition::Eligible {
                blockers.push(json!({
                    "frame_id": entry,
                    "disposition": disposition,
                }));
            }
        }
        Ok(json!({
            "status": if blockers.is_empty() { "preflighted" } else { "blocked" },
            "graph_id": graph_id,
            "revision": revision,
            "blockers": blockers,
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
            &format!("join error: {error}"),
        )),
    }
}

#[derive(Deserialize)]
pub struct PreflightBody {
    pub revision: u64,
}

/// Create a run for a stored, preflightable graph revision (Spec 155 §19.1).
async fn create_run(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(graph_id): axum::extract::Path<String>,
    Json(body): Json<PreflightBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let revision = body.revision;
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(stored) =
            focusa_core::callgraph_store::load_definition(&conn, &graph_id, revision)?
        else {
            return Ok(json!({
                "status": "rejected_missing_definition",
                "graph_id": graph_id,
                "revision": revision,
            }));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        if !validate_graph(&graph).valid {
            return Ok(json!({"status": "rejected_invalid"}));
        }
        let run_id = uuid::Uuid::now_v7().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        focusa_core::callgraph_store::create_run(
            &conn,
            &focusa_core::callgraph_store::CallGraphRun {
                run_id: run_id.clone(),
                graph_id: graph_id.clone(),
                revision,
                state: focusa_core::callgraph_store::RunState::Created,
                created_at: now.clone(),
                updated_at: now,
            },
        )?;
        Ok(json!({
            "status": "created",
            "run_id": run_id,
            "graph_id": graph_id,
            "revision": revision,
            "next": "post /v1/callgraph-runs/{run_id}/control to dispatch the entry frontier",
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
            &format!("join error: {error}"),
        )),
    }
}

/// Read a run's ledger row (Spec 155 §19.1 GET /v1/callgraph-runs/{run_id}).
async fn get_run(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id)? else {
            return Ok(json!({"status": "missing", "run_id": run_id}));
        };
        let dispatches = focusa_core::callgraph_store::list_dispatches(&conn, &run_id)?;
        Ok(json!({
            "status": "ok",
            "run": run,
            "dispatches": dispatches,
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
            &format!("join error: {error}"),
        )),
    }
}

/// Frame settlement (Spec 155 §17/§19.1): a receipt settles the
/// invocation, marks the dispatch receipted, and transitions the run when
/// every dispatch is settled. Evidence links land on the settlement.
#[derive(Deserialize)]
pub struct SettleBody {
    pub invocation_id: String,
    pub receipt_ref: String,
    pub outcome: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

async fn settle_frame(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<SettleBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let events_tx = state.events_tx.clone();
    let invocation_id = body.invocation_id.clone();
    let receipt_ref = body.receipt_ref.clone();
    let outcome = body.outcome.clone();
    let evidence_refs = body.evidence_refs.clone();
    let run_id_inner = run_id.clone();
    let run_id_for_event = run_id.clone();
    let event_invocation_id = invocation_id.clone();
    let event_receipt_ref = receipt_ref.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id_inner)? else {
            return Ok(json!({"status": "missing", "run_id": run_id_inner}));
        };
        let mut dispatches = focusa_core::callgraph_store::list_dispatches(&conn, &run_id_inner)?;
        let Some(dispatch) = dispatches
            .iter_mut()
            .find(|d| d.invocation_id.as_deref() == Some(invocation_id.as_str()))
        else {
            return Ok(json!({"status": "missing_invocation", "invocation_id": invocation_id}));
        };
        if dispatch.receipt_ref.is_some() {
            return Ok(json!({"status": "already_settled", "dispatch": dispatch}));
        }
        focusa_core::callgraph_store::mark_dispatch_settled(
            &conn,
            &dispatch.dispatch_id,
            &receipt_ref,
            &outcome,
            &evidence_refs,
        )?;
        focusa_core::callgraph_store::release_lease(&conn, &invocation_id)?;
        let all_settled = focusa_core::callgraph_store::list_dispatches(&conn, &run_id_inner)?
            .iter()
            .all(|d| d.receipt_ref.is_some());
        let now = chrono::Utc::now().to_rfc3339();
        if all_settled {
            focusa_core::callgraph_store::transition_run(
                &conn,
                &run_id_inner,
                focusa_core::callgraph_store::RunState::Completed,
                &now,
            )?;
        }
        // §16 compensation: a failed settlement unrolls the graph — every
        // declared compensation target becomes a committed dispatch.
        if outcome == "failed" {
            let Some(stored) =
                focusa_core::callgraph_store::load_definition(&conn, &run.graph_id, run.revision)?
            else {
                return Ok(json!({"status": "missing_definition"}));
            };
            let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
                .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
            let failed_frame = dispatch.frame_id.clone();
            let steps = focusa_core::callgraph::plan_unwind(&graph, &failed_frame);
            for step in steps {
                if let Some(target) = step.compensation_target {
                    focusa_core::callgraph_store::commit_dispatch(
                        &conn,
                        &focusa_core::callgraph_store::DispatchCommit {
                            dispatch_id: uuid::Uuid::now_v7().to_string(),
                            run_id: run_id_inner.clone(),
                            frame_id: target,
                            invocation_id: uuid::Uuid::now_v7().to_string(),
                            parent_invocation_id: Some(invocation_id.clone()),
                            disposition: focusa_core::callgraph::Disposition::Eligible,
                            attempt: 1,
                            committed_at: now.clone(),
                            actor_ref: "compensation".to_string(),
                        },
                    )?;
                }
            }
        }
        let _ = &run;
        Ok(json!({
            "status": "settled",
            "dispatch_id": dispatch.dispatch_id,
            "invocation_id": invocation_id,
            "receipt_ref": receipt_ref,
            "run_settled": all_settled,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => {
            if let Ok(serialized) =
                serde_json::to_string(&focusa_core::types::FocusaEvent::CallGraphFrameSettled {
                    run_id: run_id_for_event,
                    frame_id: "settled".to_string(),
                    invocation_id: event_invocation_id,
                    receipt_ref: event_receipt_ref,
                })
            {
                let _ = events_tx.send(serialized);
            }
            Json(payload)
        }
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

/// Flow Mesh binding preflight (§13.2): validate the binding against the
/// frame, then execute commits a dispatch with the binding recorded.
async fn flowmesh_preflight(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id)? else {
            return Ok(json!({"status": "missing", "run_id": run_id}));
        };
        let Some(stored) = focusa_core::callgraph_store::load_definition(&conn, &run.graph_id, run.revision)? else {
            return Ok(json!({"status": "missing_definition"}));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        let frame_id = body.get("frame_id").and_then(|v| v.as_str()).unwrap_or("");
        let Some(frame) = graph.frames.iter().find(|f| f.frame_id == frame_id) else {
            return Ok(json!({"status": "missing_frame", "frame_id": frame_id}));
        };
        let binding: focusa_core::callgraph::FlowMeshBinding =
            serde_json::from_value(body.get("binding").cloned().unwrap_or(serde_json::json!({})))
                .map_err(|error| anyhow::anyhow!("binding unparsable: {error}"))?;
        match focusa_core::callgraph::validate_flowmesh_binding(&binding, frame) {
            Ok(()) => Ok(json!({"status": "preflighted", "frame_id": frame_id, "binding_id": binding.binding_id})),
            Err(reason) => Ok(json!({
                "status": "rejected",
                "failure_class": "flowmesh_binding_invalid",
                "retry_posture": "do_not_retry_unchanged",
                "safe_recovery": "bind a flowmesh_task frame with a complete binding",
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

/// Flow Mesh binding execute (§13.4): commit a dispatch bound to the
/// validated binding — the durable commit boundary precedes any Flow
/// Mesh call, exactly like the entry frontier.
async fn flowmesh_execute(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id)? else {
            return Ok(json!({"status": "missing", "run_id": run_id}));
        };
        let frame_id = body.get("frame_id").and_then(|v| v.as_str()).unwrap_or("");
        let binding_id = body
            .get("binding_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if frame_id.is_empty() || binding_id.is_empty() {
            return Ok(json!({"status": "rejected", "error": "frame_id + binding_id required"}));
        }
        let dispatch_id = uuid::Uuid::now_v7().to_string();
        let invocation_id = uuid::Uuid::now_v7().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        focusa_core::callgraph_store::commit_dispatch(
            &conn,
            &focusa_core::callgraph_store::DispatchCommit {
                dispatch_id: dispatch_id.clone(),
                run_id: run_id.clone(),
                frame_id: frame_id.to_string(),
                invocation_id: invocation_id.clone(),
                parent_invocation_id: None,
                disposition: focusa_core::callgraph::Disposition::Eligible,
                attempt: 1,
                committed_at: now,
                actor_ref: format!("flowmesh:{binding_id}"),
            },
        )?;
        Ok(json!({
            "status": "dispatched",
            "dispatch_id": dispatch_id,
            "invocation_id": invocation_id,
            "binding_id": binding_id,
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

/// Run events (§19.1): the dispatch ledger as an ordered event list.
async fn get_run_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let dispatches = focusa_core::callgraph_store::list_dispatches(&conn, &run_id)?;
        Ok(json!({"status": "ok", "run_id": run_id, "events": dispatches}))
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

/// Evidence link (Spec 155 §17): bind evidence refs to a settled dispatch.
async fn link_evidence(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let dispatch_id = body
            .get("dispatch_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let evidence = body
            .get("evidence_refs")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        focusa_core::callgraph_store::link_evidence(&conn, &run_id, dispatch_id, &evidence)?;
        Ok(json!({"status": "linked", "dispatch_id": dispatch_id, "evidence_refs": evidence}))
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

/// Read run paths + frontier (Spec 155 §19.1).
async fn get_run_paths(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id)? else {
            return Ok(json!({"status": "missing", "run_id": run_id}));
        };
        let Some(stored) =
            focusa_core::callgraph_store::load_definition(&conn, &run.graph_id, run.revision)?
        else {
            return Ok(json!({"status": "missing_definition"}));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        let dispatches = focusa_core::callgraph_store::list_dispatches(&conn, &run_id)?;
        let frontier = focusa_core::callgraph::replay_frontier(&graph, &dispatches);
        let paths: Vec<Value> = graph
            .entry_frame_ids
            .iter()
            .map(|entry| {
                let mut path = vec![entry.clone()];
                let mut current = entry.clone();
                loop {
                    let next = graph
                        .edges
                        .iter()
                        .find(|e| e.from_frame_id == current)
                        .map(|e| e.to_frame_id.clone());
                    match next {
                        Some(next) => {
                            path.push(next.clone());
                            current = next;
                        }
                        None => break,
                    }
                }
                json!({"path_id": format!("path-{entry}"), "invocation_ids": path})
            })
            .collect();
        Ok(json!({
            "status": "ok",
            "run_id": run_id,
            "paths": paths,
            "frontier": frontier,
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

/// Dispatch control (Spec 155 §19.1 POST /v1/callgraph-runs/{run_id}/control).
/// `dispatch_entry_frontier` commits a durable dispatch row per eligible
/// entry frame BEFORE any adapter activity — the §12 commit boundary.
#[derive(Deserialize)]
pub struct ControlBody {
    pub action: String,
    #[serde(default)]
    pub actor_ref: Option<String>,
}

async fn control_run(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<ControlBody>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let events_tx = state.events_tx.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(run) = focusa_core::callgraph_store::load_run(&conn, &run_id)? else {
            return Ok(json!({"status": "missing", "run_id": run_id}));
        };
        match body.action.as_str() {
            "dispatch_entry_frontier" => {
                let Some(stored) = focusa_core::callgraph_store::load_definition(
                    &conn,
                    &run.graph_id,
                    run.revision,
                )?
                else {
                    return Ok(json!({
                        "status": "rejected_missing_definition",
                        "graph_id": run.graph_id,
                        "revision": run.revision,
                    }));
                };
                let graph: FocusaCallGraphDefinition =
                    serde_json::from_str(&stored.definition_json).map_err(|error| {
                        anyhow::anyhow!("stored definition unparsable: {error}")
                    })?;
                let settled = std::collections::HashSet::new();
                let now = chrono::Utc::now().to_rfc3339();
                let actor = body.actor_ref.unwrap_or_else(|| "daemon".to_string());
                let mut dispatched = Vec::new();
                let mut blocked = Vec::new();
                // Adapter registry: route each entry frame against the
                // registered capability sets (slice 10).
                let adapter_capabilities: Vec<focusa_core::callgraph::AdapterCapability> =
                    focusa_core::adapter_registry::list_adapters(&conn)?
                        .into_iter()
                        .map(|record| focusa_core::callgraph::AdapterCapability {
                            adapter_id: record.adapter_id,
                            model: record.model,
                            capabilities: record.capabilities,
                            healthy: record.healthy,
                        })
                        .collect();
                for entry in &graph.entry_frame_ids {
                    let disposition = eligibility_for_frame(&graph, entry, None, &settled);
                    if disposition != Disposition::Eligible {
                        blocked.push(json!({
                            "frame_id": entry,
                            "disposition": disposition,
                        }));
                        continue;
                    }
                    let frame = graph
                        .frames
                        .iter()
                        .find(|frame| frame.frame_id == *entry)
                        .expect("entry frame exists");
                    let route = focusa_core::callgraph::route_frame(frame, &adapter_capabilities);
                    let routed_adapter = match &route {
                        focusa_core::callgraph::RouteDecision::Routed { adapter_id, .. } => {
                            adapter_id.clone()
                        }
                        focusa_core::callgraph::RouteDecision::WaitingCapability => {
                            "pending-capability".to_string()
                        }
                        focusa_core::callgraph::RouteDecision::Rejected => "rejected".to_string(),
                    };
                    let dispatch_id = uuid::Uuid::now_v7().to_string();
                    let invocation_id = uuid::Uuid::now_v7().to_string();
                    focusa_core::callgraph_store::commit_dispatch(
                        &conn,
                        &focusa_core::callgraph_store::DispatchCommit {
                            dispatch_id: dispatch_id.clone(),
                            run_id: run_id.clone(),
                            frame_id: entry.clone(),
                            invocation_id: invocation_id.clone(),
                            parent_invocation_id: None,
                            disposition: Disposition::Eligible,
                            attempt: 1,
                            committed_at: now.clone(),
                            actor_ref: routed_adapter,
                        },
                    )?;
                    focusa_core::callgraph_store::acquire_lease(
                        &conn,
                        &focusa_core::callgraph_store::FrameLease {
                            invocation_id: invocation_id.clone(),
                            run_id: run_id.clone(),
                            frame_id: entry.clone(),
                            lease_holder: actor.clone(),
                            lease_expires_at: (chrono::Utc::now() + chrono::Duration::seconds(300))
                                .to_rfc3339(),
                            acquired_at: now.clone(),
                        },
                    )?;
                    dispatched.push(json!({
                        "dispatch_id": dispatch_id,
                        "invocation_id": invocation_id,
                        "frame_id": entry,
                    }));
                    if let Ok(serialized) = serde_json::to_string(
                        &focusa_core::types::FocusaEvent::CallGraphFrameDispatched {
                            run_id: run_id.clone(),
                            dispatch_id,
                            frame_id: entry.clone(),
                            invocation_id: invocation_id.clone(),
                            adapter_id: "pending".to_string(),
                            model: "pending".to_string(),
                            attempt: 1,
                        },
                    ) {
                        let _ = events_tx.send(serialized);
                    }
                }
                if !dispatched.is_empty() {
                    focusa_core::callgraph_store::transition_run(
                        &conn,
                        &run_id,
                        focusa_core::callgraph_store::RunState::Running,
                        &now,
                    )?;
                }
                Ok(json!({
                    "status": if blocked.is_empty() { "dispatched" } else { "partial" },
                    "run_id": run_id,
                    "dispatched": dispatched,
                    "blocked": blocked,
                }))
            }
            other => Ok(json!({
                "status": "unknown_action",
                "action": other,
                "supported": ["dispatch_entry_frontier"],
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
            &format!("join error: {error}"),
        )),
    }
}

/// Export a stored definition through one typed projection (#287).
#[derive(Deserialize)]
pub struct ExportQuery {
    pub revision: u64,
    #[serde(default = "default_export_format")]
    pub format: String,
}

fn default_export_format() -> String {
    "jsonl".to_string()
}

async fn export_graph(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(graph_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<ExportQuery>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(stored) =
            focusa_core::callgraph_store::load_definition(&conn, &graph_id, query.revision)?
        else {
            return Ok(json!({
                "status": "missing",
                "graph_id": graph_id,
                "revision": query.revision,
            }));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        let dispatches: Vec<focusa_core::callgraph_store::FrameDispatch> =
            focusa_core::callgraph_store::list_dispatches_for_graph(&conn, &graph_id)
                .unwrap_or_default();
        let (format_name, lossless, omissions) = match query.format.as_str() {
            "jsonl" => ("jsonl".to_string(), true, vec![]),
            "todo.txt" => (
                "todo.txt".to_string(),
                false,
                vec!["edge semantics flattened to dep: tags".to_string()],
            ),
            "dot" => ("dot".to_string(), true, vec![]),
            "csv" => ("csv".to_string(), true, vec![]),
            "tsv" => ("tsv".to_string(), true, vec![]),
            "mermaid" => ("mermaid".to_string(), true, vec![]),
            other => {
                return Ok(json!({
                    "status": "unknown_format",
                    "format": other,
                    "supported": ["jsonl", "todo.txt", "dot", "csv", "tsv", "mermaid"],
                }));
            }
        };
        let projection = focusa_core::callgraph_export::CallGraphExportProjection::new(
            graph,
            dispatches,
            &format_name,
            lossless,
            omissions,
        );
        let body = match query.format.as_str() {
            "jsonl" => focusa_core::callgraph_export::export_jsonl(&projection),
            "todo.txt" => focusa_core::callgraph_export::export_todo_txt(&projection),
            "dot" => focusa_core::callgraph_export::export_dot(&projection),
            "csv" => focusa_core::callgraph_export::export_csv(&projection, ','),
            "tsv" => focusa_core::callgraph_export::export_csv(&projection, '\t'),
            "mermaid" => focusa_core::callgraph_export::export_mermaid(&projection),
            _ => unreachable!("format validated above"),
        };
        Ok(json!({
            "status": "ok",
            "format": format_name,
            "manifest": projection.manifest,
            "body": body,
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
            &format!("join error: {error}"),
        )),
    }
}

/// Read the canonical Item Envelope for one frame (#289).
async fn get_item_envelope(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((graph_id, frame_id)): axum::extract::Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<ExportQuery>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::callgraph_store::ensure_schema(&conn)?;
        let Some(stored) =
            focusa_core::callgraph_store::load_definition(&conn, &graph_id, query.revision)?
        else {
            return Ok(json!({
                "status": "missing",
                "graph_id": graph_id,
                "revision": query.revision,
            }));
        };
        let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
            .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
        let Some(frame) = graph
            .frames
            .iter()
            .find(|frame| frame.frame_id == frame_id)
            .cloned()
        else {
            return Ok(json!({
                "status": "missing_frame",
                "frame_id": frame_id,
            }));
        };
        let envelope = focusa_core::callgraph_envelope::build_item_envelope(&graph, &frame, None);
        Ok(json!({
            "status": "ok",
            "envelope": envelope,
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
            &format!("join error: {error}"),
        )),
    }
}
