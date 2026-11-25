//! Binding-qualified Genesis staging, publication, and readback.
use super::{
    AppState, Arc, Json, ProjectGenesisRequest, State, StatusCode, Utc, Value, clean, json, reject,
    start,
};
use crate::routes::workpoint::materialize_remote_genesis_events;
use crate::scope::{ScopeContext, ScopeSource};
use axum::http::HeaderMap;
use focusa_core::remote_workspace::{
    RemoteGenesisFence, RemoteGenesisPublication, inspect_attested_remote_workspace,
};
use focusa_core::scoped_state::ScopeRef;
use focusa_core::types::HltLedgerEntry;
use std::path::Path;

#[path = "project_genesis_remote_plan.rs"]
mod plan;
#[cfg(test)]
pub(super) use plan::build_remote_staged_packet;
#[cfg(not(test))]
use plan::build_remote_staged_packet;
use plan::{FrozenGenesisPlan, freeze_plan, project_remote_beads_tasks};

pub(super) async fn commit(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    req: ProjectGenesisRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use super::super::project_bootstrap::remote::{MAX_PROBE_AGE, resolve_authority};
    if req.takeover == Some(true) {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "remote_genesis_takeover_unsupported",
            "remote Genesis cannot take over another Workstream",
        ));
    }
    let authority = resolve_authority(
        state,
        headers,
        req.remote_binding_id.as_deref(),
        &req.project_root,
        &req.continuity_id,
    )
    .await?;
    let scope = authority.scope(&req.project_root, &req.continuity_id);
    let ledger_path = super::super::events_sqlite::focusa_db_path(&state.config.data_dir);
    let blocked = |_: anyhow::Error| {
        reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_genesis_commit_precondition_failed",
            "commit requires the original staged intent and fresh exact remote inventory",
        )
    };
    let inspection =
        inspect_attested_remote_workspace(&ledger_path, &scope, &authority.binding, MAX_PROBE_AGE)
            .await
            .map_err(blocked)?;
    let (fence, plan) = {
        let ledger = rusqlite::Connection::open_with_flags(
            ledger_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|error| blocked(error.into()))?;
        RemoteGenesisFence::load(
            &ledger,
            &scope,
            &authority.binding,
            MAX_PROBE_AGE,
            &req.idempotency_key,
            inspection.inventory(),
        )
        .map_err(blocked)?
    };
    let frozen: FrozenGenesisPlan =
        serde_json::from_value(plan).map_err(|error| blocked(error.into()))?;
    frozen.validate().map_err(blocked)?;
    let live_tasks =
        project_remote_beads_tasks(inspection.inventory().beads_export()).map_err(blocked)?;
    if !inspection.inventory().beads_present
        || !frozen
            .matches_current_first_task(&live_tasks)
            .map_err(blocked)?
    {
        return Err(reject(
            StatusCode::CONFLICT,
            "remote_genesis_first_task_changed",
            "the staged first task no longer matches the owner-verified queue",
        ));
    }
    let query = super::ProjectGenesisStatusQuery {
        project_root: req.project_root.clone(),
        remote_binding_id: req.remote_binding_id.clone(),
        continuity_id: Some(req.continuity_id.clone()),
    };
    let replay_key = req.idempotency_key.clone();
    let key = Some(replay_key.as_str());
    let existing = status(state, headers, &query, key).await?;
    if existing.0["status"] == "ready" {
        return Ok(existing);
    }

    let projection = fence.intent().projection_key();
    let scope_ref = ScopeRef::project(
        format!("remote-workspace:{}", scope.binding_id),
        scope.remote_root,
        &scope.project.project_id,
        &projection,
    )
    .map_err(|error| blocked(error.into()))?;
    let (req, packet) = frozen.into_compile_inputs(req, fence.intent().bootstrap_receipt_id);
    let (trajectory, workpoint) =
        super::compile_genesis_records(Path::new(&projection), &req, &packet, scope_ref);
    let hlt = HltLedgerEntry::new(
        projection.clone(),
        trajectory.long_term_goal.clone(),
        "remote_genesis_commit",
        1,
    )
    .with_scope(Some(req.continuity_id.clone()), None)
    .with_evidence(vec![format!(
        "remote-bootstrap:{}",
        fence.intent().bootstrap_receipt_id
    )]);
    let publication = RemoteGenesisPublication {
        trajectory_id: trajectory.trajectory_id.clone(),
        workpoint_id: workpoint.workpoint_id,
        ladder: crate::routes::trajectory::trajectory_commit_events(
            &trajectory,
            None,
            &hlt,
            &hlt.evidence_refs,
            Some(&req.idempotency_key),
        ),
        hlt,
    };
    let events = super::genesis_events(trajectory, workpoint);
    let context = ScopeContext {
        project_root: Some(projection),
        continuity_id: Some(req.continuity_id.clone()),
        source: ScopeSource::Query,
        ..ScopeContext::default()
    };
    let state_for_write = state.clone();
    let committed = tokio::spawn(async move {
        materialize_remote_genesis_events(
            context,
            &state_for_write,
            events,
            fence.with_publication(publication),
        )
        .await
    })
    .await
    .map_err(|_| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "remote_genesis_worker_lost",
            "check exact publication before attempting a retry",
        )
    })?;
    match committed {
        Ok(_) => status(state, headers, &query, key).await,
        Err(error) => {
            // A concurrent identical commit can settle first. Read only the
            // matching receipt; never re-materialize or accept a different key.
            if let Ok(observed) = status(state, headers, &query, key).await {
                if observed.0["status"] == "ready" {
                    return Ok(observed);
                }
            }
            Err(error)
        }
    }
}

pub(super) async fn resume(
    state: &AppState,
    headers: &HeaderMap,
    req: ProjectGenesisRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.idempotency_key.trim().is_empty()
        || req.idempotency_key.len() > 256
        || req.takeover == Some(true)
    {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "remote_genesis_invalid_resume",
            "resume requires the original replay key and does not permit takeover",
        ));
    }
    let query = super::ProjectGenesisStatusQuery {
        project_root: req.project_root,
        remote_binding_id: req.remote_binding_id,
        continuity_id: Some(req.continuity_id),
    };
    status(state, headers, &query, Some(&req.idempotency_key)).await
}

pub(super) async fn status(
    state: &AppState,
    headers: &HeaderMap,
    query: &super::ProjectGenesisStatusQuery,
    expected_replay_key: Option<&str>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use super::super::project_bootstrap::remote::{MAX_PROBE_AGE, resolve_authority};
    let continuity = query.continuity_id.as_deref().ok_or_else(|| {
        reject(
            StatusCode::BAD_REQUEST,
            "remote_genesis_scope_required",
            "remote Genesis status requires an explicit continuity",
        )
    })?;
    let authority = resolve_authority(
        state,
        headers,
        query.remote_binding_id.as_deref(),
        &query.project_root,
        continuity,
    )
    .await?;
    let scope = authority.scope(&query.project_root, continuity);
    let ledger_path = super::super::events_sqlite::focusa_db_path(&state.config.data_dir);
    let observed = {
        let ledger = rusqlite::Connection::open_with_flags(
            ledger_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|_| {
            reject(
                StatusCode::SERVICE_UNAVAILABLE,
                "remote_genesis_ledger_unavailable",
                "controller ledger is unavailable",
            )
        })?;
        focusa_core::remote_workspace::read_remote_genesis_publication(
            &ledger,
            &scope,
            &authority.binding,
            Utc::now(),
            MAX_PROBE_AGE,
            expected_replay_key,
        )
    }
    .map_err(|error| {
        if error.to_string() == "remote_genesis_intent_conflict" {
            return reject(
                StatusCode::CONFLICT,
                "remote_genesis_intent_conflict",
                "the replay key does not match the staged intent",
            );
        }
        reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_genesis_verification_failed",
            "current attestation and exact Bootstrap receipt are required",
        )
    })?;
    if expected_replay_key.is_some() && observed.is_none() {
        return Err(reject(
            StatusCode::NOT_FOUND,
            "remote_genesis_intent_not_found",
            "no staged intent exists for this binding",
        ));
    }
    let mut packet = json!({
        "schema": "focusa.remote_genesis_status.v1", "status": if observed.is_some() { "remote_intent_staged" } else { "not_started" },
        "binding_id": scope.binding_id, "project_root": scope.remote_root, "continuity_id": scope.continuity_id,
        "canonical": false, "execution_admission": false, "readiness_receipt": null,
        "remote_genesis_intent": observed.as_ref().map(|(intent, _)| intent),
    });
    if let Some((intent, Some(publication))) = observed {
        let focusa = state.focusa.read().await;
        if !publication.verifies(&focusa, &intent, scope.continuity_id) {
            return Err(reject(
                StatusCode::CONFLICT,
                "remote_genesis_projection_mismatch",
                "stored receipt cannot prove the active Trajectory and first Workpoint",
            ));
        }
        packet["status"] = json!("ready");
        packet["canonical"] = json!(true);
        packet["readiness_receipt"] = json!({
            "receipt_id": intent.intent_id, "bootstrap_receipt_id": intent.bootstrap_receipt_id,
            "trajectory_id": publication.trajectory_id, "workpoint_id": publication.workpoint_id,
            "recorded_at": publication.hlt.timestamp,
        });
        let first = focusa
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == publication.workpoint_id)
            .ok_or_else(|| {
                reject(
                    StatusCode::CONFLICT,
                    "remote_genesis_projection_mismatch",
                    "first Workpoint disappeared",
                )
            })?;
        let expected_scope_id = format!("remote-workspace:{}", scope.binding_id);
        let projection_key = intent.projection_key();
        let trajectory = focusa
            .trajectory
            .records
            .iter()
            .find(|record| {
                record.trajectory_id == publication.trajectory_id
                    && record.scope_ref.as_ref().is_some_and(|reference| {
                        reference.scope_id == expected_scope_id
                            && reference.root_path == Path::new(scope.remote_root)
                            && reference.fingerprint == projection_key
                    })
            })
            .ok_or_else(|| {
                reject(
                    StatusCode::CONFLICT,
                    "remote_genesis_scope_mismatch",
                    "published Trajectory is not associated with this binding",
                )
            })?;
        packet["trajectory"] = json!({"trajectory_id": trajectory.trajectory_id, "hlt": trajectory.long_term_goal,
            "mlg": trajectory.mid_level_goal, "stg": trajectory.short_term_goal, "current_state": trajectory.current_state,
            "desired_end_state": trajectory.desired_end_state, "gap": trajectory.gap_summary,
            "active_waypoint_id": trajectory.active_waypoint_id,
        });
        let mut first_packet = crate::routes::workpoint::workpoint_packet(first);
        first_packet["project_root"] = json!(scope.remote_root);
        first_packet["remote_binding_id"] = json!(scope.binding_id);
        packet["first_workpoint"] = first_packet;
        packet["next_action"] =
            json!("attach the exact remote Workstream before executing its first Workpoint");
    }
    Ok(Json(packet))
}

pub(super) async fn start_or_preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let Some(binding_id) = req.remote_binding_id.as_deref() else {
        return start(State(state), Json(req)).await;
    };
    use super::super::project_bootstrap::remote::{MAX_PROBE_AGE, resolve_authority};
    use focusa_core::remote_workspace::{
        inspect_attested_remote_workspace, read_remote_bootstrap, stage_remote_genesis,
    };
    let authority = resolve_authority(
        &state,
        &headers,
        Some(binding_id),
        &req.project_root,
        &req.continuity_id,
    )
    .await?;
    let scope = authority.scope(&req.project_root, &req.continuity_id);
    let ledger_path = super::super::events_sqlite::focusa_db_path(&state.config.data_dir);
    let blocked = |_: anyhow::Error| {
        reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_genesis_verification_failed",
            "verified remote binding, Bootstrap receipt and owner inventory are required",
        )
    };
    let inspection =
        inspect_attested_remote_workspace(&ledger_path, &scope, &authority.binding, MAX_PROBE_AGE)
            .await
            .map_err(blocked)?;
    let ledger = rusqlite::Connection::open_with_flags(
        &ledger_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|error| blocked(error.into()))?;
    let receipt = read_remote_bootstrap(
        &ledger,
        &scope,
        &authority.binding,
        Utc::now(),
        MAX_PROBE_AGE,
    )
    .map_err(blocked)?
    .ok_or_else(|| {
        reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_genesis_bootstrap_required",
            "apply remote Bootstrap first",
        )
    })?;
    if !inspection.inventory().beads_present || clean(Some(&req.idempotency_key)).is_none() {
        return Err(reject(
            StatusCode::PRECONDITION_FAILED,
            "remote_genesis_inputs_missing",
            "existing Beads export and an idempotency key are required",
        ));
    }
    let mut packet = build_remote_staged_packet(&req, None, inspection.inventory().beads_export())
        .map_err(|_| {
            reject(
                StatusCode::PRECONDITION_FAILED,
                "remote_genesis_beads_invalid",
                "existing Beads export could not be decoded",
            )
        })?;
    packet["planning_status"] = packet["status"].clone();
    packet["status"] = json!("remote_preview");
    packet["canonical"] = json!(false);
    packet["execution_admission"] = json!(false);
    if req.confirm == Some(true) && packet["planning_status"] == "staged" {
        if req.takeover == Some(true) {
            return Err(reject(
                StatusCode::BAD_REQUEST,
                "remote_genesis_takeover_unsupported",
                "remote staging cannot replace an existing intent",
            ));
        }
        let plan = freeze_plan(&packet).map_err(blocked)?;
        let mut writer = rusqlite::Connection::open_with_flags(
            &ledger_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .map_err(|error| blocked(error.into()))?;
        let intent = stage_remote_genesis(
            &mut writer,
            &scope,
            &inspection,
            MAX_PROBE_AGE,
            receipt.receipt_id,
            &req.idempotency_key,
            &plan,
        )
        .map_err(|error| {
            if error.to_string() == "remote_genesis_intent_conflict" {
                reject(
                    StatusCode::CONFLICT,
                    "remote_genesis_intent_conflict",
                    "the binding already has a different staged intent",
                )
            } else {
                blocked(error)
            }
        })?;
        packet["status"] = json!("remote_intent_staged");
        packet["remote_genesis_intent"] = json!(intent);
    }

    packet["remote_binding_id"] = json!(binding_id);
    packet["remote_bootstrap_receipt_id"] = json!(receipt.receipt_id);
    if packet["status"] == "remote_intent_staged" {
        packet["next_action"] = json!("commit Project Genesis using the original replay key");
    }
    Ok(Json(packet))
}
