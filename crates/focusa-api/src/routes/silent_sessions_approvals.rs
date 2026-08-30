use std::{collections::BTreeSet, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Duration, Utc};
use focusa_core::silent_sessions::{
    ApprovalId, AuthorizationTarget, ContextAuthorityVerdict, DurableApprovalRecord, RunGeneration,
    SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionId, SilentSessionRole,
    SilentSessionRouteScope, SilentSessionRunId, VerifiedAuthorityFacts, action_digest,
    load_config_revision, load_durable_approval, load_run, load_session, save_durable_approval,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
        persistence_failure,
    },
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

const MAX_APPROVAL_LIFETIME: Duration = Duration::hours(24);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApprovalRequest {
    pub approval_id: ApprovalId,
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    pub action: SilentSessionAction,
    pub risk_class: String,
    pub expires_at: DateTime<Utc>,
    pub requested_side_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApprovalCreateBody {
    pub request: ApprovalRequest,
    pub expected_action_digest: String,
}

pub(super) async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<ApprovalRequest>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let approval = match build_approval(&state, &principal, session_id, &body) {
        Ok(approval) => approval,
        Err(response) => return disclose_principal_side_effect(*response, &principal),
    };
    let create_request = ApprovalCreateBody {
        request: body,
        expected_action_digest: approval.action_digest.clone(),
    };
    let mut envelope = SilentSessionApiEnvelope::canonical(
        "approval_preview",
        json!({
            "approval": approval,
            "create_request": create_request,
            "persisted": false
        }),
    );
    envelope.next_tools.push(format!(
        "focusa silent approval create {} --request-file <data.create_request.json>",
        session_id
    ));
    disclose_principal_side_effect((StatusCode::OK, Json(envelope)), &principal)
}

pub(super) async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<ApprovalCreateBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    // Keep derivation and persistence in the same serialized mutation window so
    // the durable record cannot be bound to authority facts changed in between.
    let _write_guard = state.write_serial_lock.lock().await;
    let approval = match build_approval(&state, &principal, session_id, &body.request) {
        Ok(approval) => approval,
        Err(response) => return disclose_principal_side_effect(*response, &principal),
    };
    if body.expected_action_digest != approval.action_digest {
        return disclose_principal_side_effect(
            failure(
                StatusCode::CONFLICT,
                "stale_target",
                "approval_digest_mismatch",
                "Run approval preview again and submit its exact action digest.",
            ),
            &principal,
        );
    }
    match load_durable_approval(&state.persistence, approval.approval_id) {
        Ok(Some(existing)) if existing == approval => {
            return approval_success(StatusCode::OK, "replayed", existing, true, &principal);
        }
        Ok(Some(_)) => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::CONFLICT,
                    "idempotency_conflict",
                    "approval_id_reused",
                    "Retry with the original request or a new UUIDv7 approval_id.",
                ),
                &principal,
            );
        }
        Ok(None) => {}
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    }
    if let Err(error) = save_durable_approval(&state.persistence, &approval) {
        return disclose_principal_side_effect(persistence_failure(error), &principal);
    }
    approval_success(
        StatusCode::CREATED,
        "approval_created",
        approval,
        false,
        &principal,
    )
}

fn build_approval(
    state: &AppState,
    principal: &ApiRequestPrincipal,
    session_id: SilentSessionId,
    request: &ApprovalRequest,
) -> Result<DurableApprovalRecord, Box<ApiResponse>> {
    if principal.principal.role != SilentSessionRole::Administrator
        || !principal
            .principal
            .scopes
            .contains(&SilentSessionRouteScope::Admin)
    {
        return Err(Box::new(failure(
            StatusCode::FORBIDDEN,
            "forbidden",
            "administrator_approval_required",
            "Use an authenticated administrator principal to create a durable approval.",
        )));
    }
    let now = Utc::now();
    let risk_class = validate_approval_request(request, now)?;
    let session = load_session(&state.persistence, session_id)
        .map_err(|error| Box::new(persistence_failure(error)))?
        .ok_or_else(|| Box::new(not_found("session_id")))?;
    let run = load_run(&state.persistence, request.run_id)
        .map_err(|error| Box::new(persistence_failure(error)))?
        .ok_or_else(|| Box::new(not_found("run_id")))?;
    if session.current_run_generation != request.generation {
        return Err(Box::new(stale("session generation is no longer current")));
    }
    guard_exact_target(
        ExactSessionRunTarget {
            session_id,
            run_id: request.run_id,
            generation: request.generation,
        },
        &run,
    )
    .map_err(|error| Box::new(stale(&format!("exact target rejected: {error:?}"))))?;
    let config = load_config_revision(&state.persistence, run.config_revision_id)
        .map_err(|error| Box::new(persistence_failure(error)))?
        .ok_or_else(|| Box::new(not_found("config_revision_id")))?;
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
    let authorization = SilentSessionAuthorizationRequest {
        principal: principal.principal.clone(),
        action: request.action,
        authority: VerifiedAuthorityFacts {
            project_permission: true,
            continuity_permission: true,
            work_item_permission: true,
            writer_ownership: false,
            authorized_project_root: target.project_root.clone(),
            authorized_continuity_id: target.continuity_id.clone(),
            authorized_work_item_ref: target.work_item_ref.clone(),
            writer_principal_id: None,
            context_authority: ContextAuthorityVerdict::Allowed,
        },
        target: target.clone(),
        approval: None,
        approval_durably_verified: false,
        legacy_approved: false,
        requested_side_effects: request.requested_side_effects.clone(),
        now,
    };
    Ok(DurableApprovalRecord {
        approval_id: request.approval_id,
        operator_actor: principal.principal.actor.clone(),
        action: request.action,
        project_root: target.project_root,
        continuity_id: target.continuity_id,
        session_id: target.session_id,
        run_id: target.run_id,
        config_hash: target.config_hash,
        action_digest: action_digest(&authorization),
        model_binding: target.model_binding,
        workspace: target.workspace,
        risk_class,
        expires_at: request.expires_at,
        permitted_side_effects: request.requested_side_effects.clone(),
    })
}

fn approval_success(
    status_code: StatusCode,
    status: &str,
    approval: DurableApprovalRecord,
    replayed: bool,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    let approval_id = approval.approval_id;
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({"approval": approval, "persisted": true, "idempotent_replay": replayed}),
    );
    envelope.side_effects.push(ApiSideEffect {
        effect: "durable_approval_create".into(),
        status: if replayed { "replayed" } else { "persisted" }.into(),
        target_ref: Some(approval_id.to_string()),
    });
    envelope
        .receipt_refs
        .push(format!("silent-session-approval:{approval_id}"));
    disclose_principal_side_effect((status_code, Json(envelope)), principal)
}

fn validate_approval_request(
    request: &ApprovalRequest,
    now: DateTime<Utc>,
) -> Result<String, Box<ApiResponse>> {
    if !request.action.requires_approval() {
        return Err(Box::new(failure(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "approval_not_required",
            "Select an action whose contract requires durable approval.",
        )));
    }
    let risk_class = request.risk_class.trim();
    if risk_class.is_empty() {
        return Err(Box::new(invalid("risk_class is required")));
    }
    if request.expires_at <= now || request.expires_at > now + MAX_APPROVAL_LIFETIME {
        return Err(Box::new(invalid(
            "expires_at must be in the future and no more than 24 hours away",
        )));
    }
    if request.requested_side_effects.is_empty()
        || request
            .requested_side_effects
            .iter()
            .any(|effect| effect.trim().is_empty())
        || request
            .requested_side_effects
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != request.requested_side_effects.len()
    {
        return Err(Box::new(invalid(
            "requested_side_effects must be non-empty, non-blank, and unique",
        )));
    }
    Ok(risk_class.to_string())
}

fn invalid(message: &str) -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "invalid_approval_request",
        message,
    )
}

fn not_found(field: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "silent_session_target_not_found",
        &format!("No record exists for {field}."),
    )
}

fn stale(message: &str) -> ApiResponse {
    failure(
        StatusCode::CONFLICT,
        "stale_target",
        "silent_session_target_stale",
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(now: DateTime<Utc>) -> ApprovalRequest {
        ApprovalRequest {
            approval_id: ApprovalId::new(),
            run_id: SilentSessionRunId::new(),
            generation: RunGeneration::first(),
            action: SilentSessionAction::Start,
            risk_class: "read_only_verifier".into(),
            expires_at: now + Duration::hours(1),
            requested_side_effects: vec!["runner_start_request".into()],
        }
    }

    #[test]
    fn client_cannot_supply_server_authority_fields_or_flatten_create_request() {
        let now = Utc::now();
        let mut value = serde_json::to_value(request(now)).unwrap();
        value["operator_actor"] = json!("forged-operator");
        assert!(serde_json::from_value::<ApprovalRequest>(value).is_err());

        let flat_create = json!({
            "approval_id": ApprovalId::new(),
            "run_id": SilentSessionRunId::new(),
            "generation": 1,
            "action": "start",
            "risk_class": "read_only_verifier",
            "expires_at": now + Duration::hours(1),
            "requested_side_effects": ["runner_start_request"],
            "expected_action_digest": "digest"
        });
        assert!(serde_json::from_value::<ApprovalCreateBody>(flat_create).is_err());

        let exact = ApprovalCreateBody {
            request: request(now),
            expected_action_digest: "digest".into(),
        };
        let encoded = serde_json::to_value(&exact).unwrap();
        assert!(encoded.get("request").is_some());
        assert_eq!(
            serde_json::from_value::<ApprovalCreateBody>(encoded)
                .unwrap()
                .expected_action_digest,
            "digest"
        );
    }

    #[test]
    fn approval_request_validation_is_bounded_and_action_specific() {
        let now = Utc::now();
        assert_eq!(
            validate_approval_request(&request(now), now).unwrap(),
            "read_only_verifier"
        );

        let mut no_approval_action = request(now);
        no_approval_action.action = SilentSessionAction::Preflight;
        assert_eq!(
            validate_approval_request(&no_approval_action, now)
                .unwrap_err()
                .1
                .0
                .failure_class
                .as_deref(),
            Some("approval_not_required")
        );

        let mut duplicate_effect = request(now);
        duplicate_effect
            .requested_side_effects
            .push("runner_start_request".into());
        assert!(validate_approval_request(&duplicate_effect, now).is_err());

        let mut expired = request(now);
        expired.expires_at = now;
        assert!(validate_approval_request(&expired, now).is_err());

        let mut too_long = request(now);
        too_long.expires_at = now + Duration::hours(25);
        assert!(validate_approval_request(&too_long, now).is_err());
    }
}
