use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ActorInstanceId, AppendOutcome, AuthorizationTarget, ConfigLayer, ConfigRevisionId,
    ContextAuthorityVerdict, EVENT_SCHEMA_VERSION, ProtocolVersions, SilentSession,
    SilentSessionAction, SilentSessionAuthority, SilentSessionAuthorizationRequest,
    SilentSessionConfig, SilentSessionConfigRevision, SilentSessionEvent, SilentSessionEventId,
    SilentSessionRun, SilentSessionRunId, VerifiedAuthorityFacts, append_create_event_and_project,
    authorize_silent_session_action, load_session_by_idempotency_key,
    resolve_silent_session_config,
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
    silent_sessions_contract::{ApiSideEffect, SilentSessionApiEnvelope},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct PreflightBody {
    pub config: SilentSessionConfig,
    #[serde(default)]
    pub layers: Vec<ConfigLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CreateBody {
    pub config: SilentSessionConfig,
    #[serde(default)]
    pub layers: Vec<ConfigLayer>,
    pub idempotency_key: String,
}

pub(super) async fn preflight(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<PreflightBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let effective = match resolve_silent_session_config(body.config, body.layers) {
        Ok(effective) => effective,
        Err(error) => {
            return disclose_principal_side_effect(config_failure(error), &principal);
        }
    };
    if let Err(response) = authorize_config(
        &principal,
        SilentSessionAction::Preflight,
        &effective.resolved_effective_config,
        &effective.redacted_config_hash,
    ) {
        return disclose_principal_side_effect(*response, &principal);
    }
    let mut envelope = SilentSessionApiEnvelope::canonical("preflight_ok", json!(effective));
    principal_side_effect(&mut envelope, &principal);
    (StatusCode::OK, Json(envelope))
}

pub(super) async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let idempotency_key = body.idempotency_key.trim();
    if idempotency_key.is_empty() || idempotency_key.len() > 200 {
        return disclose_principal_side_effect(
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
    match load_session_by_idempotency_key(&state.persistence, idempotency_key) {
        Ok(Some((session, payload))) => {
            if payload.get("request_hash").and_then(Value::as_str) != Some(request_hash.as_str()) {
                return disclose_principal_side_effect(
                    failure(
                        StatusCode::CONFLICT,
                        "idempotency_conflict",
                        "idempotency_key_reused",
                        "Retry with the original request or a new idempotency_key.",
                    ),
                    &principal,
                );
            }
            let mut envelope = SilentSessionApiEnvelope::canonical(
                "replayed",
                json!({
                    "session": session,
                    "run_id": payload.get("run_id"),
                    "run_generation": payload.get("run_generation"),
                    "idempotent_replay": true
                }),
            );
            principal_side_effect(&mut envelope, &principal);
            envelope.side_effects.push(ApiSideEffect {
                effect: "silent_session_create".into(),
                status: "replayed".into(),
                target_ref: Some(session.id.to_string()),
            });
            return (StatusCode::OK, Json(envelope));
        }
        Ok(None) => {}
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    }
    let effective = match resolve_silent_session_config(body.config, body.layers) {
        Ok(effective) => effective,
        Err(error) => {
            return disclose_principal_side_effect(config_failure(error), &principal);
        }
    };
    let config = &effective.resolved_effective_config;
    if let Err(response) = authorize_config(
        &principal,
        SilentSessionAction::Create,
        config,
        &effective.redacted_config_hash,
    ) {
        return disclose_principal_side_effect(*response, &principal);
    }
    let authority = match SilentSessionAuthority::new(
        config.identity.project_root.clone(),
        config.identity.continuity_id.clone(),
    ) {
        Ok(authority) => authority,
        Err(error) => {
            return disclose_principal_side_effect(config_failure(error), &principal);
        }
    };
    let now = Utc::now();
    let revision_id = ConfigRevisionId::new();
    let mut session = match SilentSession::draft_owned(
        authority,
        principal.principal.principal_id.clone(),
        principal.principal.os_user.clone(),
        config.identity.display_name.clone(),
        config.identity.mission.clone(),
        revision_id,
        now,
    ) {
        Ok(session) => session,
        Err(error) => {
            return disclose_principal_side_effect(config_failure(error), &principal);
        }
    };
    session.work_item_ref = config.identity.work_item_ref.clone();
    let actor = ActorInstanceId::new();
    let revision = SilentSessionConfigRevision {
        config_schema_version: 1,
        id: revision_id,
        silent_session_id: session.id,
        revision: 1,
        config: config.clone(),
        redacted_config_hash: effective.redacted_config_hash.clone(),
        created_by: actor,
        created_at: now,
    };
    let run = SilentSessionRun {
        silent_session_schema_version: 1,
        id: SilentSessionRunId::new(),
        silent_session_id: session.id,
        generation: session.current_run_generation,
        actor_instance_id: actor,
        config_revision_id: revision.id,
        protocol_versions: ProtocolVersions::default(),
        started_at: now,
        ended_at: None,
    };
    let mut event = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(run.id),
        sequence: 1,
        kind: "session_drafted".into(),
        payload: json!({
            "request_hash": request_hash,
            "config_revision_id": revision.id,
            "run_id": run.id,
            "run_generation": run.generation,
            "creator_principal_id": session.creator_principal_id,
        }),
        idempotency_key: idempotency_key.into(),
        previous_event_hash: None,
        event_hash: String::new(),
        occurred_at: now,
    };
    let outcome = match append_create_event_and_project(
        &state.persistence,
        &mut event,
        &session,
        &revision,
        &run,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    let mut envelope = SilentSessionApiEnvelope::canonical(
        "created",
        json!({
            "session": session,
            "config_revision_id": revision.id,
            "run": run,
            "redacted_config_hash": revision.redacted_config_hash,
            "event_id": event.id,
            "idempotent_replay": outcome == AppendOutcome::Replayed,
        }),
    );
    principal_side_effect(&mut envelope, &principal);
    envelope.side_effects.extend([
        ApiSideEffect {
            effect: "silent_session_projection".into(),
            status: "created".into(),
            target_ref: Some(session.id.to_string()),
        },
        ApiSideEffect {
            effect: "initial_run".into(),
            status: "created".into(),
            target_ref: Some(run.id.to_string()),
        },
        ApiSideEffect {
            effect: "config_revision".into(),
            status: "created".into(),
            target_ref: Some(revision.id.to_string()),
        },
        ApiSideEffect {
            effect: "reducer_event".into(),
            status: "appended".into(),
            target_ref: Some(event.id.to_string()),
        },
    ]);
    (StatusCode::CREATED, Json(envelope))
}

fn authorize_config(
    request_principal: &ApiRequestPrincipal,
    action: SilentSessionAction,
    config: &SilentSessionConfig,
    config_hash: &str,
) -> Result<(), Box<ApiResponse>> {
    let principal = &request_principal.principal;
    let target = AuthorizationTarget {
        project_root: config.identity.project_root.clone(),
        continuity_id: config.identity.continuity_id.clone(),
        work_item_ref: config.identity.work_item_ref.clone(),
        session_id: None,
        run_id: None,
        owner_os_user: principal.os_user.clone(),
        writer_principal_id: None,
        config_hash: config_hash.into(),
        model_binding: format!("{}:{}", config.model.provider, config.model.model),
        workspace: config
            .workspace
            .source_root
            .clone()
            .unwrap_or_else(|| config.identity.project_root.clone()),
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
        principal: principal.clone(),
        action,
        target,
        authority,
        approval: None,
        approval_durably_verified: false,
        legacy_approved: false,
        requested_side_effects: Vec::new(),
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

fn request_hash(body: &CreateBody) -> String {
    let bytes = serde_json::to_vec(body).expect("CreateBody serialization cannot fail");
    hex::encode(Sha256::digest(bytes))
}

fn config_failure(error: impl std::fmt::Display) -> ApiResponse {
    tracing::warn!(error = %error, "Silent Session configuration rejected");
    failure(
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_config",
        "config_validation",
        "Correct the configuration and rerun preflight.",
    )
}

fn principal_side_effect(
    envelope: &mut SilentSessionApiEnvelope<Value>,
    principal: &ApiRequestPrincipal,
) {
    envelope.side_effects.push(ApiSideEffect {
        effect: "authorization_principal_upsert".into(),
        status: "completed".into(),
        target_ref: Some(principal.principal.principal_id.clone()),
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use focusa_core::silent_sessions::{
        AuthenticatedPrincipal, HarnessConfig, HarnessKind, IdentityConfig, ModelConfig,
        ModelFallbackPolicy, ModelSelectionPolicy, NativeResumePolicy, SilentSessionRole,
        SilentSessionRouteScope,
    };

    use super::*;
    use crate::middleware::principal::ApiPrincipalSource;

    fn config() -> SilentSessionConfig {
        SilentSessionConfig::new(
            IdentityConfig {
                display_name: "proof".into(),
                project_root: "/repo/focusa".into(),
                continuity_id: "continuity:test".into(),
                work_item_ref: Some("focusa-test".into()),
                mission: "prove create".into(),
                agent_identity_ref: "agent:test".into(),
                role_profile_ref: None,
            },
            HarnessConfig {
                kind: HarnessKind::Pi,
                adapter_version: "1".into(),
                native_resume_policy: NativeResumePolicy::Prefer,
            },
            ModelConfig {
                provider: "provider".into(),
                model: "model".into(),
                thinking: None,
                selection_policy: ModelSelectionPolicy::Exact,
                fallback_policy: ModelFallbackPolicy::Disabled,
                allowed_fallbacks: Vec::new(),
                auth_profile_ref: "operator".into(),
                require_entitlement_preflight: true,
                require_runtime_model_confirmation: true,
            },
        )
    }

    fn principal(scopes: BTreeSet<SilentSessionRouteScope>) -> ApiRequestPrincipal {
        ApiRequestPrincipal {
            principal: AuthenticatedPrincipal {
                principal_id: "principal:test".into(),
                actor: "actor:test".into(),
                role: SilentSessionRole::Operator,
                os_user: "wirebot".into(),
                scopes,
                authenticated: true,
            },
            source: ApiPrincipalSource::PairedDevice,
            capability_grants: ["read:*".into(), "write:*".into()].into_iter().collect(),
        }
    }

    #[test]
    fn create_authorization_requires_exact_create_scope() {
        let allowed = principal([SilentSessionRouteScope::Create].into_iter().collect());
        assert!(authorize_config(&allowed, SilentSessionAction::Create, &config(), "hash").is_ok());
        let denied = principal([SilentSessionRouteScope::Read].into_iter().collect());
        assert!(authorize_config(&denied, SilentSessionAction::Create, &config(), "hash").is_err());
    }

    #[test]
    fn create_request_hash_is_stable_and_content_bound() {
        let original = CreateBody {
            config: config(),
            layers: Vec::new(),
            idempotency_key: "create-1".into(),
        };
        let mut changed = original.clone();
        changed.config.identity.mission = "different".into();
        assert_eq!(request_hash(&original), request_hash(&original));
        assert_ne!(request_hash(&original), request_hash(&changed));
    }
}
