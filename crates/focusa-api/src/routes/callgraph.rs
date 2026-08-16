//! CallGraph HTTP surface (#254 slice 2) — Spec 155 §19.1 (first routes).
//!
//! Validation and eligibility are pure core functions; this module wraps
//! them in typed HTTP responses. Definition storage and the run ledger
//! arrive in slice 3+.

use axum::extract::State;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use focusa_core::callgraph::{
    eligibility_for_frame, validate_graph, Disposition, FocusaCallGraphDefinition,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/callgraphs/validate", post(validate))
        .route("/v1/callgraphs/eligibility", post(eligibility))
        .route("/v1/callgraphs", post(create_definition).get(list_definitions))
        .route(
            "/v1/callgraphs/{graph_id}/runs/preflight",
            post(preflight_run),
        )
        .route("/v1/callgraphs/{graph_id}/runs", post(create_run))
        .route("/v1/callgraph-runs/{run_id}", get(get_run))
        .route("/v1/callgraphs/{graph_id}/export", get(export_graph))
        .route("/v1/callgraph-items/{graph_id}/{frame_id}", get(get_item_envelope))
        .route("/v1/callgraph-runs/{run_id}/control", post(control_run))
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(rows.collect::<rusqlite::Result<Vec<i64>>>()?
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        let stored =
            focusa_core::callgraph_store::load_definition(&conn, &graph_id, revision)?;
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
            let disposition = eligibility_for_frame(
                &graph,
                entry,
                None,
                &std::collections::HashSet::new(),
            );
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
                let graph: FocusaCallGraphDefinition = serde_json::from_str(&stored.definition_json)
                    .map_err(|error| anyhow::anyhow!("stored definition unparsable: {error}"))?;
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
                    let disposition =
                        eligibility_for_frame(&graph, entry, None, &settled);
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
                    let route =
                        focusa_core::callgraph::route_frame(frame, &adapter_capabilities);
                    let routed_adapter = match &route {
                        focusa_core::callgraph::RouteDecision::Routed { adapter_id, .. } => {
                            adapter_id.clone()
                        }
                        focusa_core::callgraph::RouteDecision::WaitingCapability => {
                            "pending-capability".to_string()
                        }
                        focusa_core::callgraph::RouteDecision::Rejected => {
                            "rejected".to_string()
                        }
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
                            lease_expires_at: (chrono::Utc::now()
                                + chrono::Duration::seconds(300))
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
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
        let envelope =
            focusa_core::callgraph_envelope::build_item_envelope(&graph, &frame, None);
        Ok(json!({
            "status": "ok",
            "envelope": envelope,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error("route", &error.to_string())),
        Err(error) => Json(focusa_core::error_envelope::internal_error("join", &format!("join error: {error}"))),
    }
}
