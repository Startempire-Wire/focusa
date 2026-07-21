use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::silent_sessions::{
    AuthorizationTarget, AuthorizedProjection, ContextAuthorityVerdict, SilentSession,
    SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionId, SilentSessionRole,
    SilentSessionRouteScope, VerifiedAuthorityFacts, authorize_silent_session_action,
    list_sessions, load_session, save_authorization_principal,
};
use serde_json::{Value, json};

use crate::{
    middleware::principal::{ApiRequestPrincipal, request_principal},
    server::AppState,
};

use super::silent_sessions_contract::{ApiSideEffect, RetryDirective, SilentSessionApiEnvelope};

pub(super) type ApiResponse = (StatusCode, Json<SilentSessionApiEnvelope<Value>>);

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/silent-sessions/preflight",
            post(super::silent_sessions_create::preflight),
        )
        .route(
            "/v1/silent-sessions",
            get(list).post(super::silent_sessions_create::create),
        )
        .route("/v1/silent-sessions/{session_id}", get(show))
}

async fn list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if !principal
        .principal
        .scopes
        .contains(&SilentSessionRouteScope::Read)
    {
        return disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal lacks silent_sessions:read.",
            ),
            &principal,
        );
    }
    let sessions = match list_sessions(&state.persistence) {
        Ok(sessions) => sessions,
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    let data = sessions
        .iter()
        .filter_map(|session| authorized_projection(&principal, session, SilentSessionAction::List))
        .collect::<Vec<_>>();
    success_with_principal("listed", json!(data), &principal)
}

async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "not_found",
                    "No Silent Session exists for the requested session_id.",
                ),
                &principal,
            );
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    match authorized_projection(&principal, &session, SilentSessionAction::Show) {
        Some(data) => success_with_principal("found", data, &principal),
        None => disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal is not authorized for this Silent Session.",
            ),
            &principal,
        ),
    }
}

pub(super) async fn durable_request_principal(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<ApiRequestPrincipal, Box<ApiResponse>> {
    let principal = request_principal(headers).await.ok_or_else(|| {
        Box::new(failure(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "authentication_required",
            "Present a valid admin or paired-device bearer token.",
        ))
    })?;
    save_authorization_principal(&state.persistence, &principal.principal, Utc::now())
        .map_err(|error| Box::new(persistence_failure(error)))?;
    Ok(principal)
}

fn authorized_projection(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    action: SilentSessionAction,
) -> Option<Value> {
    let principal = &request_principal.principal;
    let administrator = principal.role == SilentSessionRole::Administrator;
    let creator = !session.creator_principal_id.is_empty()
        && session.creator_principal_id == principal.principal_id;
    let permission = creator || administrator;
    let target = AuthorizationTarget {
        project_root: session.authority.project_root.clone(),
        continuity_id: session.authority.continuity_id.clone(),
        work_item_ref: session.work_item_ref.clone(),
        session_id: Some(session.id),
        run_id: None,
        owner_os_user: session.owner_os_user.clone(),
        writer_principal_id: None,
        config_hash: String::new(),
        model_binding: String::new(),
        workspace: session.authority.project_root.clone(),
    };
    let authority = VerifiedAuthorityFacts {
        project_permission: permission,
        continuity_permission: permission,
        work_item_permission: permission,
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
    match decision.projection? {
        AuthorizedProjection::Full => serde_json::to_value(session).ok(),
        AuthorizedProjection::RedactedSummary => Some(json!({
            "silent_session_schema_version": session.silent_session_schema_version,
            "id": session.id,
            "display_name": session.display_name,
            "lifecycle": session.lifecycle,
            "health": session.health,
            "semantic_activity": session.semantic_activity,
            "current_run_generation": session.current_run_generation,
            "created_at": session.created_at,
            "updated_at": session.updated_at,
            "projection": "redacted_summary"
        })),
        AuthorizedProjection::RawForensics => None,
    }
}

fn success_with_principal(
    status: &str,
    data: Value,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    disclose_principal_side_effect(
        (
            StatusCode::OK,
            Json(SilentSessionApiEnvelope::canonical(status, data)),
        ),
        principal,
    )
}

pub(super) fn disclose_principal_side_effect(
    mut response: ApiResponse,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    response.1.0.side_effects.push(ApiSideEffect {
        effect: "authorization_principal_upsert".into(),
        status: "completed".into(),
        target_ref: Some(principal.principal.principal_id.clone()),
    });
    response
}

pub(super) fn persistence_failure(error: impl std::fmt::Display) -> ApiResponse {
    tracing::error!(error = %error, "Silent Session persistence operation failed");
    failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        "persistence_error",
        "persistence_failure",
        "Retry after checking daemon persistence health.",
    )
}

pub(super) fn failure(
    status_code: StatusCode,
    status: &str,
    failure_class: &str,
    recovery_hint: &str,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::failure(
        status,
        failure_class,
        RetryDirective {
            retryable: status_code.is_server_error(),
            after_ms: status_code.is_server_error().then_some(1_000),
            idempotency_key_required: false,
        },
    );
    envelope.recovery_hint = Some(recovery_hint.into());
    (status_code, Json(envelope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::silent_sessions::{
        ConfigRevisionId, SilentSessionAuthority, SilentSessionLifecycle,
    };

    fn request_principal(role: SilentSessionRole, os_user: &str) -> ApiRequestPrincipal {
        ApiRequestPrincipal {
            principal: focusa_core::silent_sessions::AuthenticatedPrincipal {
                principal_id: "principal:test".into(),
                actor: "actor:test".into(),
                role,
                os_user: os_user.into(),
                scopes: [
                    SilentSessionRouteScope::Read,
                    SilentSessionRouteScope::Admin,
                ]
                .into_iter()
                .collect(),
                authenticated: true,
            },
            source: crate::middleware::principal::ApiPrincipalSource::AdminToken,
        }
    }

    fn owned_session(owner: &str) -> SilentSession {
        SilentSession::draft_owned(
            SilentSessionAuthority::new("/repo/focusa", "continuity:test").unwrap(),
            "principal:test",
            owner,
            "session",
            "secret mission",
            ConfigRevisionId::new(),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn creator_receives_full_projection() {
        let session = owned_session("wirebot");
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Operator, "wirebot"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["mission"], "secret mission");
    }

    #[test]
    fn cross_user_admin_receives_redacted_projection() {
        let session = owned_session("wirebot");
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Administrator, "root"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["projection"], "redacted_summary");
        assert!(value.get("mission").is_none());
    }

    #[test]
    fn legacy_unknown_owner_never_receives_full_projection() {
        let mut session = owned_session("wirebot");
        session.creator_principal_id.clear();
        session.owner_os_user.clear();
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Administrator, "root"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["projection"], "redacted_summary");
        assert_eq!(session.lifecycle, SilentSessionLifecycle::Draft);
    }
}
