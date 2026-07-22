use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ActorInstanceId, ApprovalId, ConfigLayer, ConfigRevisionId, EVENT_SCHEMA_VERSION,
    RunGeneration, SilentSessionAction, SilentSessionConfig, SilentSessionConfigRevision,
    SilentSessionEvent, SilentSessionEventId, SilentSessionId, SilentSessionRunId,
    append_config_revision_event_and_project, append_reducer_event_and_project,
    load_config_revision, load_durable_approval, load_run, load_session, load_session_events,
    preview_config_revision,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
        persistence_failure,
    },
    silent_sessions_authorize::authorize_mutation,
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MutationTarget {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    approval_id: ApprovalId,
    idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RevisionBody {
    #[serde(flatten)]
    target: MutationTarget,
    requested_config: SilentSessionConfig,
    #[serde(default)]
    layers: Vec<ConfigLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RollbackBody {
    #[serde(flatten)]
    target: MutationTarget,
    target_revision_id: ConfigRevisionId,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/silent-sessions/{session_id}/config/revisions",
            post(revise),
        )
        .route(
            "/v1/silent-sessions/{session_id}/config/rollback",
            post(rollback),
        )
}

async fn revise(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<RevisionBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let (mut session, run, current, events) =
        match load_context(&state, session_id, &body.target, &principal) {
            Ok(context) => context,
            Err(response) => return *response,
        };
    let request_hash = hash_request("revision", &body);
    if let Some(response) = replay(&events, &body.target, &request_hash, &principal) {
        return response;
    }
    let plan =
        match preview_config_revision(current.config.clone(), body.requested_config, body.layers) {
            Ok(plan) => plan,
            Err(error) => return after(config_failure(error.to_string()), &principal),
        };
    if plan.effective_diff.is_empty() {
        return after(
            config_failure("revision has no effective changes".into()),
            &principal,
        );
    }
    let mut side_effects = vec![format!("config_revision_persist:{request_hash}")];
    side_effects.push(if plan.restart_required_fields.is_empty() {
        format!("runner_hot_config_request:{request_hash}")
    } else {
        format!("runner_restart_plan_request:{request_hash}")
    });
    let approval = match load_durable_approval(&state.persistence, body.target.approval_id) {
        Ok(Some(approval)) => approval,
        Ok(None) => return after(approval_required(), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if let Err(response) = authorize_mutation(
        &principal,
        &session,
        &run,
        &current,
        SilentSessionAction::ReviseConfig,
        side_effects.clone(),
        Some(approval),
    ) {
        return after(*response, &principal);
    }
    let now = Utc::now();
    session.updated_at = now;
    let revision = SilentSessionConfigRevision {
        config_schema_version: current.config_schema_version,
        id: plan.revision_id,
        silent_session_id: session.id,
        revision: current.revision + 1,
        config: plan.candidate.resolved_effective_config.clone(),
        redacted_config_hash: plan.candidate.redacted_config_hash.clone(),
        created_by: ActorInstanceId::new(),
        created_at: now,
    };
    let mut event = mutation_event(
        &session,
        run.id,
        &events,
        "config.revision_proposed",
        &body.target,
        json!({
            "request_hash": request_hash,
            "revision_id": revision.id,
            "revision": revision.revision,
            "effective_diff": plan.effective_diff,
            "hot_fields": plan.hot_fields,
            "restart_required_fields": plan.restart_required_fields,
            "side_effects": side_effects,
            "approval_id": body.target.approval_id,
            "application_status": "pending",
        }),
    );
    if let Err(error) = append_config_revision_event_and_project(
        &state.persistence,
        &mut event,
        &session,
        &revision,
    ) {
        return after(persistence_failure(error), &principal);
    }
    success(
        "config_revision_requested",
        event.id.to_string(),
        &body.target,
        false,
        json!({
            "revision_id": revision.id,
            "revision": revision.revision,
            "restart_required": !plan.restart_required_fields.is_empty(),
            "application_status": "pending",
        }),
        side_effects,
        &principal,
    )
}

async fn rollback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<RollbackBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let (mut session, run, current, events) =
        match load_context(&state, session_id, &body.target, &principal) {
            Ok(context) => context,
            Err(response) => return *response,
        };
    let request_hash = hash_request("rollback", &body);
    if let Some(response) = replay(&events, &body.target, &request_hash, &principal) {
        return response;
    }
    let target_revision = match load_config_revision(&state.persistence, body.target_revision_id) {
        Ok(Some(revision)) if revision.silent_session_id == session.id => revision,
        Ok(Some(_)) | Ok(None) => return after(not_found("target_revision_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if target_revision.revision >= current.revision {
        return after(
            config_failure("rollback target must precede the active revision".into()),
            &principal,
        );
    }
    let side_effects = vec![
        format!("config_rollback_request:{request_hash}"),
        format!("runner_restart_plan_request:{request_hash}"),
    ];
    let approval = match load_durable_approval(&state.persistence, body.target.approval_id) {
        Ok(Some(approval)) => approval,
        Ok(None) => return after(approval_required(), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if let Err(response) = authorize_mutation(
        &principal,
        &session,
        &run,
        &current,
        SilentSessionAction::RollbackConfig,
        side_effects.clone(),
        Some(approval),
    ) {
        return after(*response, &principal);
    }
    session.updated_at = Utc::now();
    let mut event = mutation_event(
        &session,
        run.id,
        &events,
        "config.rollback_requested",
        &body.target,
        json!({
            "request_hash": request_hash,
            "from_revision_id": current.id,
            "target_revision_id": target_revision.id,
            "target_revision": target_revision.revision,
            "side_effects": side_effects,
            "approval_id": body.target.approval_id,
            "application_status": "pending",
        }),
    );
    if let Err(error) = append_reducer_event_and_project(&state.persistence, &mut event, &session) {
        return after(persistence_failure(error), &principal);
    }
    success(
        "config_rollback_requested",
        event.id.to_string(),
        &body.target,
        false,
        json!({
            "from_revision_id": current.id,
            "target_revision_id": target_revision.id,
            "restart_required": true,
            "application_status": "pending",
        }),
        side_effects,
        &principal,
    )
}

#[allow(clippy::type_complexity)]
fn load_context(
    state: &Arc<AppState>,
    session_id: SilentSessionId,
    target: &MutationTarget,
    principal: &ApiRequestPrincipal,
) -> Result<
    (
        focusa_core::silent_sessions::SilentSession,
        focusa_core::silent_sessions::SilentSessionRun,
        SilentSessionConfigRevision,
        Vec<SilentSessionEvent>,
    ),
    Box<ApiResponse>,
> {
    let session = load_session(&state.persistence, session_id)
        .map_err(|error| Box::new(after(persistence_failure(error), principal)))?
        .ok_or_else(|| Box::new(after(not_found("session_id"), principal)))?;
    let run = load_run(&state.persistence, target.run_id)
        .map_err(|error| Box::new(after(persistence_failure(error), principal)))?
        .ok_or_else(|| Box::new(after(not_found("run_id"), principal)))?;
    if session.current_run_generation != target.generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id: target.run_id,
                generation: target.generation,
            },
            &run,
        )
        .is_err()
    {
        return Err(Box::new(after(stale_target(), principal)));
    }
    let current = load_config_revision(&state.persistence, session.active_config_revision_id)
        .map_err(|error| Box::new(after(persistence_failure(error), principal)))?
        .ok_or_else(|| Box::new(after(not_found("active_config_revision_id"), principal)))?;
    let events = load_session_events(&state.persistence, session_id)
        .map_err(|error| Box::new(after(persistence_failure(error), principal)))?;
    Ok((session, run, current, events))
}

fn mutation_event(
    session: &focusa_core::silent_sessions::SilentSession,
    run_id: SilentSessionRunId,
    events: &[SilentSessionEvent],
    kind: &str,
    target: &MutationTarget,
    payload: Value,
) -> SilentSessionEvent {
    let previous = events.last();
    SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(run_id),
        sequence: previous.map_or(1, |event| event.sequence + 1),
        kind: kind.into(),
        payload,
        idempotency_key: target.idempotency_key.clone(),
        previous_event_hash: previous.map(|event| event.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: session.updated_at,
    }
}

fn replay(
    events: &[SilentSessionEvent],
    target: &MutationTarget,
    request_hash: &str,
    principal: &ApiRequestPrincipal,
) -> Option<ApiResponse> {
    let existing = events
        .iter()
        .find(|event| event.idempotency_key == target.idempotency_key)?;
    if existing.payload.get("request_hash").and_then(Value::as_str) != Some(request_hash) {
        return Some(after(idempotency_conflict(), principal));
    }
    let side_effects = existing
        .payload
        .get("side_effects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect();
    Some(success(
        "replayed",
        existing.id.to_string(),
        target,
        true,
        json!({"application_status": "pending"}),
        side_effects,
        principal,
    ))
}

#[allow(clippy::too_many_arguments)]
fn success(
    status: &str,
    event_id: String,
    target: &MutationTarget,
    replayed: bool,
    detail: Value,
    side_effects: Vec<String>,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({
            "event_id": event_id,
            "run_id": target.run_id,
            "generation": target.generation,
            "replayed": replayed,
            "detail": detail,
        }),
    );
    envelope
        .side_effects
        .extend(side_effects.into_iter().map(|effect| ApiSideEffect {
            effect,
            status: "requested".into(),
            target_ref: Some(target.run_id.to_string()),
        }));
    envelope.receipt_refs.push(target.approval_id.to_string());
    after((StatusCode::ACCEPTED, Json(envelope)), principal)
}

fn hash_request(kind: &str, body: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(&json!({"kind": kind, "body": body}))
        .expect("config mutation request serializes");
    hex::encode(Sha256::digest(bytes))
}

fn config_failure(reason: String) -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "config_rejected",
        "validation_rejected",
        &reason,
    )
}

fn approval_required() -> ApiResponse {
    failure(
        StatusCode::FORBIDDEN,
        "approval_required",
        "approval_not_found",
        "Create a durable approval matching this payload-bound config request.",
    )
}

fn idempotency_conflict() -> ApiResponse {
    failure(
        StatusCode::CONFLICT,
        "idempotency_conflict",
        "idempotency_key_reused",
        "Retry with the original request or a new idempotency_key.",
    )
}

fn stale_target() -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and retry with the current exact target.",
    );
    response.1.0.stale = true;
    response
}

fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}

fn after(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

#[cfg(test)]
#[path = "silent_sessions_config_mutation_test.rs"]
mod tests;
