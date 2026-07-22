use super::{
    silent_sessions::{
        ApiResponse, authorized_projection, disclose_principal_side_effect,
        durable_request_principal, failure, persistence_failure,
    },
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};
use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ApprovalId, AuthorizationTarget, ContextAuthorityVerdict, DurableApprovalRecord,
    EVENT_SCHEMA_VERSION, RunGeneration, SilentSession, SilentSessionAction,
    SilentSessionAuthorizationRequest, SilentSessionEvent, SilentSessionEventId, SilentSessionId,
    SilentSessionLifecycle, SilentSessionRole, SilentSessionRun, SilentSessionRunId,
    TransitionEvidence, VerifiedAuthorityFacts, append_reducer_event_and_project,
    authorize_silent_session_action, load_config_revision, load_durable_approval, load_run,
    load_session, load_session_events, reduce_lifecycle,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const ADOPT_EFFECT: &str = "session_control_adoption";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct AdoptBody {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    pub approval_id: ApprovalId,
    pub idempotency_key: String,
}

pub(super) async fn adopt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<AdoptBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return *r,
    };
    if body.idempotency_key.trim().is_empty() || body.idempotency_key.len() > 200 {
        return after(
            failure(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "invalid_idempotency_key",
                "Supply a valid idempotency key.",
            ),
            &principal,
        );
    }
    let hash = request_hash(&body);
    let _guard = state.write_serial_lock.lock().await;
    let mut session = match load_session(&state.persistence, session_id) {
        Ok(Some(v)) => v,
        Ok(None) => return after(not_found("session_id"), &principal),
        Err(e) => return after(persistence_failure(e), &principal),
    };
    let run = match load_run(&state.persistence, body.run_id) {
        Ok(Some(v)) => v,
        Ok(None) => return after(not_found("run_id"), &principal),
        Err(e) => return after(persistence_failure(e), &principal),
    };
    if session.current_run_generation != body.generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id: body.run_id,
                generation: body.generation,
            },
            &run,
        )
        .is_err()
    {
        return after(stale(), &principal);
    }
    let events = match load_session_events(&state.persistence, session_id) {
        Ok(v) => v,
        Err(e) => return after(persistence_failure(e), &principal),
    };
    if let Some(existing) = events
        .iter()
        .find(|e| e.idempotency_key == body.idempotency_key)
    {
        if existing.payload.get("request_hash").and_then(Value::as_str) != Some(hash.as_str()) {
            return after(
                failure(
                    StatusCode::CONFLICT,
                    "idempotency_conflict",
                    "idempotency_key_reused",
                    "Use the original request or a new key.",
                ),
                &principal,
            );
        }
        return success(
            StatusCode::OK,
            "replayed",
            &session,
            existing.id.to_string(),
            true,
            &principal,
            body.approval_id,
        );
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(v)) => v,
        Ok(None) => return after(not_found("config_revision_id"), &principal),
        Err(e) => return after(persistence_failure(e), &principal),
    };
    let approval = match load_durable_approval(&state.persistence, body.approval_id) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return after(
                failure(
                    StatusCode::FORBIDDEN,
                    "approval_required",
                    "approval_not_found",
                    "Create an exact durable adoption approval.",
                ),
                &principal,
            );
        }
        Err(e) => return after(persistence_failure(e), &principal),
    };
    if let Err(r) = authorize(&principal, &session, &run, &config, approval) {
        return after(*r, &principal);
    }
    let transition = match reduce_lifecycle(
        session.lifecycle,
        SilentSessionLifecycle::Recovering,
        &TransitionEvidence::default(),
    ) {
        Ok(v) => v,
        Err(_) => {
            return after(
                failure(
                    StatusCode::CONFLICT,
                    "invalid_transition",
                    "lifecycle_conflict",
                    "Adoption requires an orphaned session.",
                ),
                &principal,
            );
        }
    };
    let previous_controller = session.controller_principal_id.clone();
    session.controller_principal_id = principal.principal.principal_id.clone();
    session.lifecycle = transition.to;
    session.updated_at = Utc::now();
    let previous = events.last();
    let mut event = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(run.id),
        sequence: previous.map_or(1, |e| e.sequence + 1),
        kind: "session_adopted".into(),
        payload: json!({"request_hash": hash, "previous_controller_principal_id": previous_controller, "controller_principal_id": session.controller_principal_id, "approval_id": body.approval_id}),
        idempotency_key: body.idempotency_key,
        previous_event_hash: previous.map(|e| e.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: session.updated_at,
    };
    if let Err(e) = append_reducer_event_and_project(&state.persistence, &mut event, &session) {
        return after(persistence_failure(e), &principal);
    }
    success(
        StatusCode::ACCEPTED,
        "adopted",
        &session,
        event.id.to_string(),
        false,
        &principal,
        body.approval_id,
    )
}

fn authorize(
    p: &ApiRequestPrincipal,
    session: &SilentSession,
    run: &SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
    approval: DurableApprovalRecord,
) -> Result<(), Box<ApiResponse>> {
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
        project_permission: true,
        continuity_permission: true,
        work_item_permission: true,
        writer_ownership: false,
        authorized_project_root: target.project_root.clone(),
        authorized_continuity_id: target.continuity_id.clone(),
        authorized_work_item_ref: target.work_item_ref.clone(),
        writer_principal_id: None,
        context_authority: ContextAuthorityVerdict::Allowed,
    };
    let decision = authorize_silent_session_action(&SilentSessionAuthorizationRequest {
        principal: p.principal.clone(),
        action: SilentSessionAction::Adopt,
        target,
        authority,
        approval: Some(approval),
        approval_durably_verified: true,
        legacy_approved: false,
        requested_side_effects: vec![ADOPT_EFFECT.into()],
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
    replayed: bool,
    p: &ApiRequestPrincipal,
    approval: ApprovalId,
) -> ApiResponse {
    let projection = authorized_projection(p, session, SilentSessionAction::Show)
        .unwrap_or_else(|| json!({"id": session.id, "projection": "redacted_summary"}));
    let mut env = SilentSessionApiEnvelope::canonical(
        status,
        json!({"session": projection, "event_id": event_id, "idempotent_replay": replayed}),
    );
    env.receipt_refs.push(approval.to_string());
    env.side_effects.extend([
        ApiSideEffect {
            effect: "authorization_principal_upsert".into(),
            status: "completed".into(),
            target_ref: Some(p.principal.principal_id.clone()),
        },
        ApiSideEffect {
            effect: ADOPT_EFFECT.into(),
            status: if replayed { "replayed" } else { "completed" }.into(),
            target_ref: Some(session.id.to_string()),
        },
    ]);
    (code, Json(env))
}
fn after(r: ApiResponse, p: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(r, p)
}
fn not_found(t: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {t}."),
    )
}
fn stale() -> ApiResponse {
    let mut r = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and retry with the current exact target.",
    );
    r.1.0.stale = true;
    r
}
fn request_hash(body: &AdoptBody) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(body).expect("AdoptBody serializes"),
    ))
}
