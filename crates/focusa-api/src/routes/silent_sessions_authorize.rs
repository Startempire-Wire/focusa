use axum::http::StatusCode;
use chrono::Utc;
use focusa_core::silent_sessions::{
    AuthorizationTarget, ContextAuthorityVerdict, DurableApprovalRecord, SilentSession,
    SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionRole, SilentSessionRun,
    VerifiedAuthorityFacts, authorize_silent_session_action,
};

use crate::middleware::principal::ApiRequestPrincipal;

use super::silent_sessions::{ApiResponse, failure};

pub(super) fn authorization_context(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    run: &SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
) -> (AuthorizationTarget, VerifiedAuthorityFacts) {
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
    (target, authority)
}

pub(super) fn authorize_mutation(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    run: &SilentSessionRun,
    config: &focusa_core::silent_sessions::SilentSessionConfigRevision,
    action: SilentSessionAction,
    requested_side_effects: Vec<String>,
    approval: Option<DurableApprovalRecord>,
) -> Result<(), Box<ApiResponse>> {
    let principal = &request_principal.principal;
    let (target, authority) = authorization_context(request_principal, session, run, config);
    let decision = authorize_silent_session_action(&SilentSessionAuthorizationRequest {
        principal: principal.clone(),
        action,
        target,
        authority,
        approval_durably_verified: approval.is_some(),
        approval,
        legacy_approved: false,
        requested_side_effects,
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
