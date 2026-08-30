use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ApprovalId, AuthorizationTarget, ContextAuthorityVerdict, EVENT_SCHEMA_VERSION,
    SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionEvent,
    SilentSessionEventId, SilentSessionId, SilentSessionLifecycle, SilentSessionRole,
    SilentSessionRunId, TransitionEvidence, VerifiedAuthorityFacts,
    append_reducer_event_and_project, authorize_silent_session_action, load_config_revision,
    load_durable_approval, load_run, load_session, load_session_events, reduce_lifecycle,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal,
        ensure_silent_session_temporal_guard, failure, persistence_failure,
        silent_session_temporal_context,
    },
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

const START_SIDE_EFFECT: &str = "runner_start_request";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct StartBody {
    pub run_id: SilentSessionRunId,
    pub generation: focusa_core::silent_sessions::RunGeneration,
    pub approval_id: ApprovalId,
    pub idempotency_key: String,
}

pub(super) async fn start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<StartBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if body.idempotency_key.trim().is_empty() || body.idempotency_key.len() > 200 {
        return after_principal(
            failure(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "invalid_idempotency_key",
                "Supply a non-empty idempotency_key no longer than 200 bytes.",
            ),
            &principal,
        );
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
            return after_principal(
                failure(
                    StatusCode::CONFLICT,
                    "idempotency_conflict",
                    "idempotency_key_reused",
                    "Retry with the original request or a new idempotency_key.",
                ),
                &principal,
            );
        }
        return lifecycle_success(
            StatusCode::OK,
            "replayed",
            &session,
            existing.id.to_string(),
            true,
            &principal,
            body.approval_id,
            silent_session_temporal_context(&session, Some(&run), None, Some(events.len())),
            None,
        );
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(config)) => config,
        Ok(None) => return after_principal(not_found("config_revision_id"), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    let approval = match load_durable_approval(&state.persistence, body.approval_id) {
        Ok(Some(approval)) => approval,
        Ok(None) => {
            return after_principal(
                approval_required(session_id, body.run_id, body.generation),
                &principal,
            );
        }
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    if let Err(response) = authorize_start(&principal, &session, &run, &config, approval) {
        return after_principal(*response, &principal);
    }
    let temporal_guard =
        match ensure_silent_session_temporal_guard(&session, "silent-session:start") {
            Ok(context) => context,
            Err(response) => return after_principal(*response, &principal),
        };
    let transition = match reduce_lifecycle(
        session.lifecycle,
        SilentSessionLifecycle::Validating,
        &TransitionEvidence::default(),
    ) {
        Ok(transition) => transition,
        Err(error) => {
            tracing::warn!(error = %error, session_id = %session_id, "start transition rejected");
            return after_principal(
                failure(
                    StatusCode::CONFLICT,
                    "invalid_transition",
                    "lifecycle_conflict",
                    "Refresh status before retrying the lifecycle action.",
                ),
                &principal,
            );
        }
    };
    let now = Utc::now();
    session.lifecycle = transition.to;
    session.updated_at = now;
    let previous = events.last();
    let mut event = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(run.id),
        sequence: previous.map_or(1, |event| event.sequence + 1),
        kind: "start_requested".into(),
        payload: json!({
            "request_hash": request_hash,
            "from": transition.from,
            "to": transition.to,
            "approval_id": body.approval_id,
            "side_effect": START_SIDE_EFFECT,
        }),
        idempotency_key: body.idempotency_key,
        previous_event_hash: previous.map(|event| event.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: now,
    };
    if let Err(error) = append_reducer_event_and_project(&state.persistence, &mut event, &session) {
        return after_principal(persistence_failure(error), &principal);
    }
    let temporal_context = silent_session_temporal_context(
        &session,
        Some(&run),
        Some(&config.config),
        Some(events.len() + 1),
    );
    lifecycle_success(
        StatusCode::ACCEPTED,
        "start_requested",
        &session,
        event.id.to_string(),
        false,
        &principal,
        body.approval_id,
        temporal_context,
        Some(temporal_guard),
    )
}

fn approval_required(
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: focusa_core::silent_sessions::RunGeneration,
) -> ApiResponse {
    let mut response = failure(
        StatusCode::FORBIDDEN,
        "approval_required",
        "approval_not_found",
        "Create a durable approval matching this exact start request.",
    );
    response.1.0.next_tools.push(format!(
        "focusa silent approval preview {session_id} --request-file <approval-request.json>"
    ));
    response.1.0.recovery_hint = Some(format!(
        "Bind approval to run_id={run_id}, generation={}, action=start, and side effect={START_SIDE_EFFECT}.",
        generation.get()
    ));
    response
}

fn authorize_start(
    request_principal: &ApiRequestPrincipal,
    session: &focusa_core::silent_sessions::SilentSession,
    run: &focusa_core::silent_sessions::SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
    approval: focusa_core::silent_sessions::DurableApprovalRecord,
) -> Result<(), Box<ApiResponse>> {
    let principal = &request_principal.principal;
    let administrator = principal.role == SilentSessionRole::Administrator;
    let controller = !session.controller_principal_id.is_empty()
        && session.controller_principal_id == principal.principal_id;
    let permission = controller || administrator;
    let workspace = config
        .config
        .workspace
        .source_root
        .clone()
        .unwrap_or_else(|| session.authority.project_root.clone());
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
        workspace,
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
        action: SilentSessionAction::Start,
        target,
        authority,
        approval: Some(approval),
        approval_durably_verified: true,
        legacy_approved: false,
        requested_side_effects: vec![START_SIDE_EFFECT.into()],
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
fn lifecycle_success(
    code: StatusCode,
    status: &str,
    session: &focusa_core::silent_sessions::SilentSession,
    event_id: String,
    replayed: bool,
    principal: &ApiRequestPrincipal,
    approval_id: ApprovalId,
    temporal_context: Value,
    temporal_guard: Option<Value>,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({
            "session": session,
            "event_id": event_id,
            "idempotent_replay": replayed,
            "temporal_context":temporal_context,
            "mutation_temporal_guard":temporal_guard,
        }),
    );
    envelope.side_effects.extend([
        ApiSideEffect {
            effect: "authorization_principal_upsert".into(),
            status: "completed".into(),
            target_ref: Some(principal.principal.principal_id.clone()),
        },
        ApiSideEffect {
            effect: START_SIDE_EFFECT.into(),
            status: if replayed { "replayed" } else { "queued" }.into(),
            target_ref: Some(session.id.to_string()),
        },
    ]);
    envelope.receipt_refs.push(approval_id.to_string());
    (code, Json(envelope))
}

fn stale_target(reason: &str) -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and retry with the current session_id, run_id and generation.",
    );
    response.1.0.stale = true;
    response.1.0.misuse_hint = Some(reason.into());
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

fn after_principal(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

fn request_hash(body: &StartBody) -> String {
    let bytes = serde_json::to_vec(body).expect("StartBody serialization cannot fail");
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use focusa_core::silent_sessions::RunGeneration;

    use super::*;

    #[test]
    fn start_request_hash_is_stable_and_content_bound() {
        let original = StartBody {
            run_id: SilentSessionRunId::new(),
            generation: RunGeneration::first(),
            approval_id: ApprovalId::new(),
            idempotency_key: "start-1".into(),
        };
        let mut changed = original.clone();
        changed.idempotency_key = "start-2".into();
        assert_eq!(request_hash(&original), request_hash(&original));
        assert_ne!(request_hash(&original), request_hash(&changed));
    }

    #[test]
    fn missing_start_approval_returns_exact_next_command() {
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let generation = RunGeneration::first();
        let response = approval_required(session_id, run_id, generation);
        assert_eq!(response.0, StatusCode::FORBIDDEN);
        assert_eq!(response.1.0.status, "approval_required");
        assert_eq!(
            response.1.0.failure_class.as_deref(),
            Some("approval_not_found")
        );
        assert_eq!(response.1.0.next_tools.len(), 1);
        assert!(response.1.0.next_tools[0].contains(&session_id.to_string()));
        assert!(response.1.0.recovery_hint.as_deref().is_some_and(|hint| {
            hint.contains(&run_id.to_string()) && hint.contains("runner_start_request")
        }));
    }

    #[test]
    fn stale_target_envelope_is_canonical_and_non_retrying() {
        let response = stale_target("generation mismatch");
        assert_eq!(response.0, StatusCode::CONFLICT);
        assert!(response.1.0.stale);
        assert!(!response.1.0.retry.retryable);
        assert_eq!(
            response.1.0.failure_class.as_deref(),
            Some("exact_target_mismatch")
        );
    }
}
