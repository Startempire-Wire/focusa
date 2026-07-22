use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ControlBody {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    #[serde(default)]
    pub approval_id: Option<ApprovalId>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Copy)]
enum ControlKind {
    Pause,
    Resume,
    Interrupt,
    Cancel,
}

impl ControlKind {
    fn action(self) -> SilentSessionAction {
        match self {
            Self::Pause => SilentSessionAction::Pause,
            Self::Resume => SilentSessionAction::Resume,
            Self::Interrupt => SilentSessionAction::Interrupt,
            Self::Cancel => SilentSessionAction::Cancel,
        }
    }

    fn event_kind(self) -> &'static str {
        match self {
            Self::Pause => "pause_requested",
            Self::Resume => "resume_requested",
            Self::Interrupt => "interrupt_requested",
            Self::Cancel => "cancel_requested",
        }
    }

    fn side_effect(self) -> &'static str {
        match self {
            Self::Pause => "runner_pause_request",
            Self::Resume => "runner_resume_request",
            Self::Interrupt => "runner_interrupt_request",
            Self::Cancel => "runner_cancel_request",
        }
    }

    fn requires_approval(self) -> bool {
        matches!(self, Self::Interrupt | Self::Cancel)
    }

    fn target(self, current: SilentSessionLifecycle) -> Option<SilentSessionLifecycle> {
        match (self, current) {
            (Self::Pause, SilentSessionLifecycle::Running) => Some(SilentSessionLifecycle::Pausing),
            (
                Self::Pause,
                SilentSessionLifecycle::WaitingInput | SilentSessionLifecycle::Blocked,
            ) => Some(SilentSessionLifecycle::Paused),
            (Self::Resume, SilentSessionLifecycle::Paused) => {
                Some(SilentSessionLifecycle::Resuming)
            }
            (
                Self::Interrupt | Self::Cancel,
                SilentSessionLifecycle::Running
                | SilentSessionLifecycle::WaitingInput
                | SilentSessionLifecycle::Blocked
                | SilentSessionLifecycle::Paused,
            ) => Some(SilentSessionLifecycle::Cancelling),
            _ => None,
        }
    }
}

pub(super) async fn pause(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<ControlBody>,
) -> ApiResponse {
    control(state, headers, session_id, body, ControlKind::Pause).await
}

pub(super) async fn resume(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<ControlBody>,
) -> ApiResponse {
    control(state, headers, session_id, body, ControlKind::Resume).await
}

pub(super) async fn interrupt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<ControlBody>,
) -> ApiResponse {
    control(state, headers, session_id, body, ControlKind::Interrupt).await
}

pub(super) async fn cancel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<ControlBody>,
) -> ApiResponse {
    control(state, headers, session_id, body, ControlKind::Cancel).await
}

async fn control(
    state: Arc<AppState>,
    headers: HeaderMap,
    session_id: SilentSessionId,
    body: ControlBody,
    kind: ControlKind,
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
    let request_hash = request_hash(&body, kind);
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
        return success(
            StatusCode::OK,
            "replayed",
            &session,
            existing.id.to_string(),
            true,
            &principal,
            kind,
            body.approval_id,
        );
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(config)) => config,
        Ok(None) => return after_principal(not_found("config_revision_id"), &principal),
        Err(error) => return after_principal(persistence_failure(error), &principal),
    };
    let approval = if kind.requires_approval() {
        let Some(approval_id) = body.approval_id else {
            return after_principal(
                failure(
                    StatusCode::FORBIDDEN,
                    "approval_required",
                    "approval_not_supplied",
                    "Supply a durable approval bound to this exact control request.",
                ),
                &principal,
            );
        };
        match load_durable_approval(&state.persistence, approval_id) {
            Ok(Some(approval)) => Some(approval),
            Ok(None) => {
                return after_principal(
                    failure(
                        StatusCode::FORBIDDEN,
                        "approval_required",
                        "approval_not_found",
                        "Create a durable approval matching this exact control request.",
                    ),
                    &principal,
                );
            }
            Err(error) => return after_principal(persistence_failure(error), &principal),
        }
    } else {
        None
    };
    if let Err(response) = authorize_control(&principal, &session, &run, &config, kind, approval) {
        return after_principal(*response, &principal);
    }
    let Some(target) = kind.target(session.lifecycle) else {
        return after_principal(invalid_transition(session.lifecycle, kind), &principal);
    };
    let transition = match reduce_lifecycle(
        session.lifecycle,
        target,
        &TransitionEvidence::default(),
    ) {
        Ok(transition) => transition,
        Err(error) => {
            tracing::warn!(error = %error, session_id = %session_id, "control transition rejected");
            return after_principal(invalid_transition(session.lifecycle, kind), &principal);
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
        kind: kind.event_kind().into(),
        payload: json!({
            "request_hash": request_hash,
            "from": transition.from,
            "to": transition.to,
            "side_effect": kind.side_effect(),
            "approval_id": body.approval_id,
        }),
        idempotency_key: body.idempotency_key,
        previous_event_hash: previous.map(|event| event.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: now,
    };
    if let Err(error) = append_reducer_event_and_project(&state.persistence, &mut event, &session) {
        return after_principal(persistence_failure(error), &principal);
    }
    success(
        StatusCode::ACCEPTED,
        kind.event_kind(),
        &session,
        event.id.to_string(),
        false,
        &principal,
        kind,
        body.approval_id,
    )
}

fn authorize_control(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    run: &SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
    kind: ControlKind,
    approval: Option<DurableApprovalRecord>,
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
        action: kind.action(),
        target,
        authority,
        approval_durably_verified: approval.is_some(),
        approval,
        legacy_approved: false,
        requested_side_effects: vec![kind.side_effect().into()],
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
    principal: &ApiRequestPrincipal,
    kind: ControlKind,
    approval_id: Option<ApprovalId>,
) -> ApiResponse {
    let projection = authorized_projection(principal, session, SilentSessionAction::Show)
        .unwrap_or_else(|| json!({"id": session.id, "projection": "redacted_summary"}));
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({"session": projection, "event_id": event_id, "idempotent_replay": replayed}),
    );
    if let Some(approval_id) = approval_id {
        envelope.receipt_refs.push(approval_id.to_string());
    }
    envelope.side_effects.extend([
        ApiSideEffect {
            effect: "authorization_principal_upsert".into(),
            status: "completed".into(),
            target_ref: Some(principal.principal.principal_id.clone()),
        },
        ApiSideEffect {
            effect: kind.side_effect().into(),
            status: if replayed { "replayed" } else { "queued" }.into(),
            target_ref: Some(session.id.to_string()),
        },
    ]);
    (code, Json(envelope))
}

fn invalid_transition(current: SilentSessionLifecycle, kind: ControlKind) -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "invalid_transition",
        "lifecycle_conflict",
        "Refresh status before retrying the lifecycle action.",
    );
    response.1.0.misuse_hint = Some(format!("cannot apply {kind:?} from {current:?}"));
    response
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

fn request_hash(body: &ControlBody, kind: ControlKind) -> String {
    let bytes = serde_json::to_vec(&(kind.event_kind(), body))
        .expect("ControlBody serialization cannot fail");
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_targets_are_reducer_compatible() {
        assert_eq!(
            ControlKind::Pause.target(SilentSessionLifecycle::Running),
            Some(SilentSessionLifecycle::Pausing)
        );
        assert_eq!(
            ControlKind::Pause.target(SilentSessionLifecycle::WaitingInput),
            Some(SilentSessionLifecycle::Paused)
        );
        assert_eq!(
            ControlKind::Resume.target(SilentSessionLifecycle::Paused),
            Some(SilentSessionLifecycle::Resuming)
        );
        assert_eq!(
            ControlKind::Resume.target(SilentSessionLifecycle::Running),
            None
        );
        assert_eq!(
            ControlKind::Interrupt.target(SilentSessionLifecycle::Running),
            Some(SilentSessionLifecycle::Cancelling)
        );
        assert_eq!(
            ControlKind::Cancel.target(SilentSessionLifecycle::Paused),
            Some(SilentSessionLifecycle::Cancelling)
        );
        assert!(ControlKind::Interrupt.requires_approval());
        assert!(ControlKind::Cancel.requires_approval());
    }
}
