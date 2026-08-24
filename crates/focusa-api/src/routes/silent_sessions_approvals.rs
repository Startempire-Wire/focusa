use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::{Duration, Utc};
use focusa_core::silent_sessions::{
    ApprovalId, DurableApprovalRecord, SilentSessionAction, SilentSessionAuthorizationRequest,
    SilentSessionId, SilentSessionLifecycle, action_digest,
    authorize_silent_session_approval_issuance, load_config_revision,
    load_durable_approval_by_idempotency, load_run, load_session, save_authorization_principal,
    save_durable_approval,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::server::AppState;

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
        persistence_failure,
    },
    silent_sessions_approval_payload::{
        DeliveryKind, delivery_request_hash_for_approval, validate_approval_payload,
    },
    silent_sessions_authorize::authorization_context,
    silent_sessions_contract::{
        ApiSideEffect, ApprovalCreateRequest, ApprovalCreateResponse, ApprovalRequestAction,
        ExactSessionRunTarget, SILENT_SESSION_APPROVAL_REQUEST_SCHEMA_V1,
        SILENT_SESSION_APPROVAL_RESPONSE_SCHEMA_V1, SilentSessionApiEnvelope, guard_exact_target,
    },
};

const APPROVAL_TTL_MINUTES: i64 = 5;

pub(super) async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<ApprovalCreateRequest>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if body.schema != SILENT_SESSION_APPROVAL_REQUEST_SCHEMA_V1 {
        return after(
            validation_failure("unsupported approval request schema"),
            &principal,
        );
    }
    if body.idempotency_key.trim().is_empty() || body.idempotency_key.len() > 200 {
        return after(
            validation_failure("idempotency_key must contain 1..=200 bytes"),
            &principal,
        );
    }
    if !body.risk_acknowledged {
        return after(
            validation_failure("risk_acknowledged must be true"),
            &principal,
        );
    }
    let request_hash = issuance_request_hash(session_id, &body);
    let _write_guard = state.write_serial_lock.lock().await;
    match load_durable_approval_by_idempotency(
        &state.persistence,
        &principal.principal.actor,
        body.idempotency_key.trim(),
    ) {
        Ok(Some(existing)) => {
            if existing.issuance_request_hash != request_hash {
                return after(idempotency_conflict(), &principal);
            }
            return approval_success(
                StatusCode::OK,
                "replayed",
                &body,
                session_id,
                &existing,
                true,
                &principal,
            );
        }
        Ok(None) => {}
        Err(error) => return after(persistence_failure(error), &principal),
    }

    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after(not_found("session_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    let run = match load_run(&state.persistence, body.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after(not_found("run_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
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
        return after(stale_target(), &principal);
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(config)) => config,
        Ok(None) => return after(not_found("config_revision_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };

    let approval_id = ApprovalId::new();
    let (action, side_effects) = match approval_effects(&body, &session, approval_id) {
        Ok(result) => result,
        Err(response) => return after(*response, &principal),
    };
    let (target, authority) = authorization_context(&principal, &session, &run, &config);
    let now = Utc::now();
    let issuance_request = SilentSessionAuthorizationRequest {
        principal: principal.principal.clone(),
        action,
        target: target.clone(),
        authority,
        approval: None,
        approval_durably_verified: false,
        legacy_approved: false,
        requested_side_effects: side_effects.clone(),
        now,
    };
    let decision = authorize_silent_session_approval_issuance(&issuance_request);
    if !decision.allowed {
        return after(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                &decision.reason,
            ),
            &principal,
        );
    }
    let approval = DurableApprovalRecord {
        approval_id,
        operator_actor: principal.principal.actor.clone(),
        action,
        project_root: target.project_root,
        continuity_id: target.continuity_id,
        session_id: target.session_id,
        run_id: target.run_id,
        config_hash: target.config_hash,
        action_digest: action_digest(&issuance_request),
        model_binding: target.model_binding,
        workspace: target.workspace,
        risk_class: risk_class(body.action).into(),
        expires_at: now + Duration::minutes(APPROVAL_TTL_MINUTES),
        permitted_side_effects: side_effects,
        issuance_idempotency_key: body.idempotency_key.trim().into(),
        issuance_request_hash: request_hash,
    };
    if let Err(error) = save_authorization_principal(&state.persistence, &principal.principal, now)
    {
        return after(persistence_failure(error), &principal);
    }
    if let Err(error) = save_durable_approval(&state.persistence, &approval) {
        return after(persistence_failure(error), &principal);
    }
    approval_success(
        StatusCode::CREATED,
        "approved",
        &body,
        session_id,
        &approval,
        false,
        &principal,
    )
}

fn approval_effects(
    body: &ApprovalCreateRequest,
    session: &focusa_core::silent_sessions::SilentSession,
    approval_id: ApprovalId,
) -> Result<(SilentSessionAction, Vec<String>), Box<ApiResponse>> {
    match body.action {
        ApprovalRequestAction::Start => {
            require_empty_payload(body)?;
            if session.lifecycle != SilentSessionLifecycle::Draft {
                return Err(Box::new(invalid_lifecycle("start", session.lifecycle)));
            }
            Ok((
                SilentSessionAction::Start,
                vec!["runner_start_request".into()],
            ))
        }
        ApprovalRequestAction::Cancel => {
            require_empty_payload(body)?;
            if !matches!(
                session.lifecycle,
                SilentSessionLifecycle::Running
                    | SilentSessionLifecycle::WaitingInput
                    | SilentSessionLifecycle::Blocked
                    | SilentSessionLifecycle::Paused
            ) {
                return Err(Box::new(invalid_lifecycle("cancel", session.lifecycle)));
            }
            Ok((
                SilentSessionAction::Cancel,
                vec!["runner_cancel_request".into()],
            ))
        }
        action => {
            let kind = delivery_kind(action).expect("delivery action maps to kind");
            let payload = body.payload.as_ref().ok_or_else(|| {
                Box::new(validation_failure("delivery approval requires payload"))
            })?;
            validate_approval_payload(kind, payload)?;
            if !kind.accepts(session.lifecycle) {
                return Err(Box::new(invalid_lifecycle(
                    kind.as_str(),
                    session.lifecycle,
                )));
            }
            let hash = delivery_request_hash_for_approval(
                body.run_id,
                body.generation,
                approval_id,
                kind,
                payload,
            );
            Ok((SilentSessionAction::SendInput, kind.side_effects(&hash)))
        }
    }
}

fn delivery_kind(action: ApprovalRequestAction) -> Option<DeliveryKind> {
    match action {
        ApprovalRequestAction::Input => Some(DeliveryKind::Input),
        ApprovalRequestAction::Steer => Some(DeliveryKind::Steer),
        ApprovalRequestAction::FollowUp => Some(DeliveryKind::FollowUp),
        ApprovalRequestAction::Keys => Some(DeliveryKind::Keys),
        ApprovalRequestAction::Start | ApprovalRequestAction::Cancel => None,
    }
}

fn require_empty_payload(body: &ApprovalCreateRequest) -> Result<(), Box<ApiResponse>> {
    if body.payload.as_ref().is_some_and(|value| !value.is_null()) {
        Err(Box::new(validation_failure(
            "start and cancel approvals do not accept payload",
        )))
    } else {
        Ok(())
    }
}

fn risk_class(action: ApprovalRequestAction) -> &'static str {
    match action {
        ApprovalRequestAction::Start => "durable_write",
        ApprovalRequestAction::Cancel => "external_effect",
        ApprovalRequestAction::Input
        | ApprovalRequestAction::Steer
        | ApprovalRequestAction::FollowUp
        | ApprovalRequestAction::Keys => "external_effect",
    }
}

fn issuance_request_hash(session_id: SilentSessionId, body: &ApprovalCreateRequest) -> String {
    let bytes = serde_json::to_vec(&json!({"session_id": session_id, "request": body}))
        .expect("approval request serializes");
    hex::encode(Sha256::digest(bytes))
}

fn approval_success(
    code: StatusCode,
    status: &str,
    body: &ApprovalCreateRequest,
    session_id: SilentSessionId,
    approval: &DurableApprovalRecord,
    replayed: bool,
    principal: &crate::middleware::principal::ApiRequestPrincipal,
) -> ApiResponse {
    let response = ApprovalCreateResponse {
        schema: SILENT_SESSION_APPROVAL_RESPONSE_SCHEMA_V1.into(),
        status: status.into(),
        approval_id: approval.approval_id,
        action: body.action,
        session_id,
        run_id: body.run_id,
        generation: body.generation,
        expires_at: approval.expires_at,
        receipt_ref: format!("approval:{}", approval.approval_id),
        action_idempotency_key: format!("approval-action:{}", approval.approval_id),
    };
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({
            "approval": response,
            "idempotent_replay": replayed,
        }),
    );
    envelope
        .receipt_refs
        .push(format!("approval:{}", approval.approval_id));
    envelope.side_effects.push(ApiSideEffect {
        effect: "durable_approval".into(),
        status: if replayed { "replayed" } else { "persisted" }.into(),
        target_ref: Some(approval.approval_id.to_string()),
    });
    after((code, Json(envelope)), principal)
}

fn validation_failure(reason: &str) -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "validation_rejected",
        reason,
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
        "Refresh status and request approval for the current exact target.",
    );
    response.1.0.stale = true;
    response
}
fn invalid_lifecycle(action: &str, lifecycle: SilentSessionLifecycle) -> ApiResponse {
    failure(
        StatusCode::CONFLICT,
        "invalid_state",
        "invalid_lifecycle_transition",
        &format!("cannot approve {action} from {lifecycle:?}"),
    )
}
fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}
fn after(
    response: ApiResponse,
    principal: &crate::middleware::principal::ApiRequestPrincipal,
) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::silent_sessions::{RunGeneration, SilentSessionRunId};

    fn request(action: ApprovalRequestAction, payload: Option<Value>) -> ApprovalCreateRequest {
        ApprovalCreateRequest {
            schema: SILENT_SESSION_APPROVAL_REQUEST_SCHEMA_V1.into(),
            action,
            run_id: SilentSessionRunId::new(),
            generation: RunGeneration::new(1).unwrap(),
            idempotency_key: "approval:test:1".into(),
            risk_acknowledged: true,
            payload,
        }
    }

    #[test]
    fn delivery_variants_map_to_exact_kinds_and_validate_payloads() {
        let cases = [
            (
                ApprovalRequestAction::Input,
                DeliveryKind::Input,
                json!({"text":"go"}),
            ),
            (
                ApprovalRequestAction::Steer,
                DeliveryKind::Steer,
                json!({"instruction":"turn"}),
            ),
            (
                ApprovalRequestAction::FollowUp,
                DeliveryKind::FollowUp,
                json!({"prompt":"next"}),
            ),
            (
                ApprovalRequestAction::Keys,
                DeliveryKind::Keys,
                json!({"keys":["Enter"]}),
            ),
        ];
        for (action, expected, payload) in cases {
            let kind = delivery_kind(action).unwrap();
            assert_eq!(kind, expected);
            assert!(validate_approval_payload(kind, &payload).is_ok());
        }
        assert!(
            validate_approval_payload(DeliveryKind::Input, &json!({"text":"go","extra":true}))
                .is_err()
        );
        assert!(validate_approval_payload(DeliveryKind::Keys, &json!({"keys":[]})).is_err());
    }

    #[test]
    fn issuance_idempotency_hash_binds_session_action_and_payload() {
        let session = SilentSessionId::new();
        let original = request(
            ApprovalRequestAction::Steer,
            Some(json!({"instruction":"left"})),
        );
        let mut changed = original.clone();
        changed.payload = Some(json!({"instruction":"right"}));
        assert_eq!(
            issuance_request_hash(session, &original),
            issuance_request_hash(session, &original)
        );
        assert_ne!(
            issuance_request_hash(session, &original),
            issuance_request_hash(session, &changed)
        );
        assert_ne!(
            issuance_request_hash(session, &original),
            issuance_request_hash(SilentSessionId::new(), &original)
        );
    }

    #[test]
    fn start_and_cancel_reject_client_payload() {
        for action in [ApprovalRequestAction::Start, ApprovalRequestAction::Cancel] {
            assert!(require_empty_payload(&request(action, None)).is_ok());
            assert!(
                require_empty_payload(&request(action, Some(json!({"client":"effect"})))).is_err()
            );
        }
    }
}
