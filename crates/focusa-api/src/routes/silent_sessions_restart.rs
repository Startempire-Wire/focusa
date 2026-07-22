use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ActorInstanceId, ApprovalId, AuthorizationTarget, ContextAuthorityVerdict,
    DurableApprovalRecord, EVENT_SCHEMA_VERSION, RunGeneration, SilentSession, SilentSessionAction,
    SilentSessionAuthorizationRequest, SilentSessionEvent, SilentSessionEventId, SilentSessionId,
    SilentSessionLifecycle, SilentSessionRole, SilentSessionRun, SilentSessionRunId,
    TransitionEvidence, VerifiedAuthorityFacts, append_restart_event_and_project,
    authorize_silent_session_action, load_config_revision, load_durable_approval, load_run,
    load_session, load_session_events, reduce_lifecycle,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, authorized_projection, disclose_principal_side_effect,
        durable_request_principal, failure, persistence_failure,
    },
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

const RESTART_SIDE_EFFECT: &str = "runner_restart_request";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RestartBody {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    pub approval_id: ApprovalId,
    pub idempotency_key: String,
}

pub(super) async fn restart(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<RestartBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if body.idempotency_key.trim().is_empty() || body.idempotency_key.len() > 200 {
        return after_principal(invalid_idempotency(), &principal);
    }
    let request_hash = request_hash(&body);
    let _write_guard = state.write_serial_lock.lock().await;
    let mut session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after_principal(not_found("session_id"), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    let run = match load_run(&state.persistence, body.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after_principal(not_found("run_id"), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    if session.current_run_generation != body.generation {
        return after_principal(
            stale_target("session generation is no longer current"),
            &principal,
        );
    }
    if let Err(error) = guard_exact_target(
        ExactSessionRunTarget {
            session_id,
            run_id: body.run_id,
            generation: body.generation,
        },
        &run,
    ) {
        return after_principal(
            stale_target(&format!("exact target rejected: {error:?}")),
            &principal,
        );
    }
    let events = match load_session_events(&state.persistence, session_id) {
        Ok(events) => events,
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    if let Some(existing) = events
        .iter()
        .find(|event| event.idempotency_key == body.idempotency_key)
    {
        if existing.payload.get("request_hash").and_then(Value::as_str)
            != Some(request_hash.as_str())
        {
            return after_principal(idempotency_conflict(), &principal);
        }
        return success(
            StatusCode::OK,
            "replayed",
            &session,
            existing.id.to_string(),
            existing.run_id,
            true,
            &principal,
            body.approval_id,
        );
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(config)) => config,
        Ok(None) => return after_principal(not_found("config_revision_id"), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    let approval = match load_durable_approval(&state.persistence, body.approval_id) {
        Ok(Some(approval)) => approval,
        Ok(None) => return after_principal(approval_missing(), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    if let Err(response) = authorize_restart(&principal, &session, &run, &config, approval) {
        return after_principal(*response, &principal);
    }
    let transition = match reduce_lifecycle(
        session.lifecycle,
        SilentSessionLifecycle::Draft,
        &TransitionEvidence::default(),
    ) {
        Ok(transition) => transition,
        Err(error) => {
            tracing::warn!(error = %error, session_id = %session_id, "restart transition rejected");
            return after_principal(invalid_transition(session.lifecycle), &principal);
        }
    };
    let now = Utc::now();
    let mut previous_run = run.clone();
    previous_run.ended_at = Some(now);
    let next_generation = match run.generation.next() {
        Ok(generation) => generation,
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    let next_run = SilentSessionRun {
        silent_session_schema_version: 1,
        id: SilentSessionRunId::new(),
        silent_session_id: session.id,
        generation: next_generation,
        actor_instance_id: ActorInstanceId::new(),
        config_revision_id: run.config_revision_id,
        protocol_versions: run.protocol_versions.clone(),
        started_at: now,
        ended_at: None,
    };
    session.current_run_generation = next_generation;
    session.lifecycle = transition.to;
    session.updated_at = now;
    let previous_event = events.last();
    let mut event = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(next_run.id),
        sequence: previous_event.map_or(1, |event| event.sequence + 1),
        kind: "restart_requested".into(),
        payload: json!({
            "request_hash": request_hash,
            "previous_run_id": run.id,
            "next_run_id": next_run.id,
            "previous_generation": run.generation,
            "next_generation": next_generation,
            "approval_id": body.approval_id,
            "side_effect": RESTART_SIDE_EFFECT,
        }),
        idempotency_key: body.idempotency_key,
        previous_event_hash: previous_event.map(|event| event.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: now,
    };
    if let Err(error) = append_restart_event_and_project(
        &state.persistence,
        &mut event,
        &session,
        &previous_run,
        &next_run,
    ) {
        return after_principal(persistence_failure(error), &principal);
    }
    success(
        StatusCode::ACCEPTED,
        "restart_requested",
        &session,
        event.id.to_string(),
        Some(next_run.id),
        false,
        &principal,
        body.approval_id,
    )
}

fn authorize_restart(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    run: &SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
    approval: DurableApprovalRecord,
) -> Result<(), Box<ApiResponse>> {
    let principal = &request_principal.principal;
    let administrator = principal.role == SilentSessionRole::Administrator;
    let controller = !session.controller_principal_id.is_empty()
        && session.controller_principal_id == principal.principal_id;
    let permission = controller || administrator;
    let target = AuthorizationTarget {
        project_root: session.authority.project_root.clone(),
        continuity_id: session.authority.continuity_id.clone(),
        work_item_ref: session.work_item_ref.clone(),
        session_id: Some(session.id),
        run_id: Some(run.id),
        owner_os_user: session.owner_os_user.clone(),
        writer_principal_id: Some(session.controller_principal_id.clone()),
        config_hash: config.redacted_config_hash.clone(),
        model_binding: format!(
            "{}:{}",
            config.config.model.provider, config.config.model.model
        ),
        workspace: config
            .config
            .workspace
            .source_root
            .clone()
            .unwrap_or_else(|| session.authority.project_root.clone()),
    };
    let authority = VerifiedAuthorityFacts {
        project_permission: permission,
        continuity_permission: permission,
        work_item_permission: permission,
        writer_ownership: controller,
        authorized_project_root: target.project_root.clone(),
        authorized_continuity_id: target.continuity_id.clone(),
        authorized_work_item_ref: target.work_item_ref.clone(),
        writer_principal_id: controller.then(|| principal.principal_id.clone()),
        context_authority: ContextAuthorityVerdict::Allowed,
    };
    let decision = authorize_silent_session_action(&SilentSessionAuthorizationRequest {
        principal: principal.clone(),
        action: SilentSessionAction::Restart,
        target,
        authority,
        approval: Some(approval),
        approval_durably_verified: true,
        legacy_approved: false,
        requested_side_effects: vec![RESTART_SIDE_EFFECT.into()],
        now: Utc::now(),
    });
    if decision.allowed {
        Ok(())
    } else {
        Err(Box::new(failure(
            StatusCode::FORBIDDEN,
            "forbidden",
            "authorization_denied",
            &decision.reason,
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn success(
    code: StatusCode,
    status: &str,
    session: &SilentSession,
    event_id: String,
    run_id: Option<SilentSessionRunId>,
    replayed: bool,
    principal: &ApiRequestPrincipal,
    approval_id: ApprovalId,
) -> ApiResponse {
    let projection = authorized_projection(principal, session, SilentSessionAction::Show)
        .unwrap_or_else(|| json!({"id": session.id, "projection": "redacted_summary"}));
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({
            "session": projection, "run_id": run_id, "generation": session.current_run_generation,
            "event_id": event_id, "idempotent_replay": replayed
        }),
    );
    envelope.receipt_refs.push(approval_id.to_string());
    envelope.side_effects.extend([
        ApiSideEffect {
            effect: "authorization_principal_upsert".into(),
            status: "completed".into(),
            target_ref: Some(principal.principal.principal_id.clone()),
        },
        ApiSideEffect {
            effect: RESTART_SIDE_EFFECT.into(),
            status: if replayed { "replayed" } else { "queued" }.into(),
            target_ref: Some(session.id.to_string()),
        },
    ]);
    (code, Json(envelope))
}

fn after_principal(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}
fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}
fn approval_missing() -> ApiResponse {
    failure(
        StatusCode::FORBIDDEN,
        "approval_required",
        "approval_not_found",
        "Create a durable approval matching this exact restart request.",
    )
}
fn invalid_idempotency() -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "invalid_idempotency_key",
        "Supply a non-empty idempotency_key no longer than 200 bytes.",
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
fn invalid_transition(current: SilentSessionLifecycle) -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "invalid_transition",
        "lifecycle_conflict",
        "Restart only after the current run reaches a terminal state.",
    );
    response.1.0.misuse_hint = Some(format!("cannot restart from {current:?}"));
    response
}
fn stale_target(reason: &str) -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and retry with the current exact target.",
    );
    response.1.0.stale = true;
    response.1.0.misuse_hint = Some(reason.into());
    response
}
fn request_hash(body: &RestartBody) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(body).expect("RestartBody serializes"),
    ))
}
