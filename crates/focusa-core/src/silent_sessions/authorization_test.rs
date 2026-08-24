use std::collections::BTreeSet;

use chrono::{Duration, Utc};

use crate::{
    runtime::persistence_sqlite::SqlitePersistence, silent_sessions::*, types::FocusaConfig,
};

fn persistence() -> SqlitePersistence {
    let dir = std::env::temp_dir().join(format!("focusa-auth-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    SqlitePersistence::new(&FocusaConfig {
        data_dir: dir.to_string_lossy().into_owned(),
        ..FocusaConfig::default()
    })
    .unwrap()
}

fn request(action: SilentSessionAction) -> SilentSessionAuthorizationRequest {
    SilentSessionAuthorizationRequest {
        principal: AuthenticatedPrincipal {
            principal_id: "principal:operator".into(),
            actor: "actor:operator".into(),
            role: SilentSessionRole::Operator,
            os_user: "wirebot".into(),
            scopes: [
                SilentSessionRouteScope::Read,
                SilentSessionRouteScope::Stream,
                SilentSessionRouteScope::Create,
                SilentSessionRouteScope::Control,
                SilentSessionRouteScope::Config,
            ]
            .into_iter()
            .collect(),
            authenticated: true,
        },
        action,
        target: AuthorizationTarget {
            project_root: "/srv/focusa".into(),
            continuity_id: "continuity:proof".into(),
            work_item_ref: Some("focusa-proof".into()),
            session_id: Some(SilentSessionId::new()),
            run_id: Some(SilentSessionRunId::new()),
            owner_os_user: "wirebot".into(),
            writer_principal_id: Some("principal:operator".into()),
            config_hash: "config:abc".into(),
            model_binding: "provider/model".into(),
            workspace: "/srv/focusa-worktree".into(),
        },
        authority: VerifiedAuthorityFacts {
            project_permission: true,
            continuity_permission: true,
            work_item_permission: true,
            writer_ownership: true,
            authorized_project_root: "/srv/focusa".into(),
            authorized_continuity_id: "continuity:proof".into(),
            authorized_work_item_ref: Some("focusa-proof".into()),
            writer_principal_id: Some("principal:operator".into()),
            context_authority: ContextAuthorityVerdict::Allowed,
        },
        approval: None,
        approval_durably_verified: false,
        legacy_approved: true,
        requested_side_effects: vec!["write_worktree".into()],
        now: Utc::now(),
    }
}

fn approve(request: &mut SilentSessionAuthorizationRequest) {
    request.approval_durably_verified = true;
    request.approval = Some(DurableApprovalRecord {
        approval_id: ApprovalId::new(),
        operator_actor: "actor:operator".into(),
        action: request.action,
        project_root: request.target.project_root.clone(),
        continuity_id: request.target.continuity_id.clone(),
        session_id: request.target.session_id,
        run_id: request.target.run_id,
        config_hash: request.target.config_hash.clone(),
        action_digest: action_digest(request),
        model_binding: request.target.model_binding.clone(),
        workspace: request.target.workspace.clone(),
        risk_class: "write".into(),
        expires_at: request.now + Duration::minutes(5),
        permitted_side_effects: request.requested_side_effects.clone(),
        issuance_idempotency_key: "approval:test".into(),
        issuance_request_hash: "request:test".into(),
    });
}

#[test]
fn exact_route_scope_names_and_action_mapping_are_stable() {
    let names = [
        SilentSessionRouteScope::Read,
        SilentSessionRouteScope::Stream,
        SilentSessionRouteScope::Create,
        SilentSessionRouteScope::Control,
        SilentSessionRouteScope::Config,
        SilentSessionRouteScope::Admin,
        SilentSessionRouteScope::Forensics,
    ]
    .map(SilentSessionRouteScope::as_str);
    assert_eq!(
        names,
        [
            "silent_sessions:read",
            "silent_sessions:stream",
            "silent_sessions:create",
            "silent_sessions:control",
            "silent_sessions:config",
            "silent_sessions:admin",
            "silent_sessions:forensics",
        ]
    );
    assert_eq!(
        SilentSessionAction::SendInput.required_scope(),
        SilentSessionRouteScope::Control
    );
    assert_eq!(
        SilentSessionAction::RollbackConfig.required_scope(),
        SilentSessionRouteScope::Config
    );
}

#[test]
fn approval_issuance_runs_full_policy_before_record_verification() {
    for action in [
        SilentSessionAction::Start,
        SilentSessionAction::SendInput,
        SilentSessionAction::Cancel,
    ] {
        let input = request(action);
        assert!(!authorize_silent_session_action(&input).allowed);
        assert!(authorize_silent_session_approval_issuance(&input).allowed);
    }
    let pause = request(SilentSessionAction::Pause);
    let decision = authorize_silent_session_approval_issuance(&pause);
    assert!(!decision.allowed);
    assert!(decision.reason.contains("approval-required"));

    let mut denied = request(SilentSessionAction::Start);
    denied.authority.writer_ownership = false;
    assert!(!authorize_silent_session_approval_issuance(&denied).allowed);
}

#[test]
fn approved_boolean_alone_never_authorizes_and_digest_is_exact() {
    let mut input = request(SilentSessionAction::SendInput);
    let denied = authorize_silent_session_action(&input);
    assert!(!denied.allowed);
    assert!(denied.reason.contains("durably verified"));

    approve(&mut input);
    let allowed = authorize_silent_session_action(&input);
    assert!(allowed.allowed);
    assert_eq!(allowed.projection, Some(AuthorizedProjection::Full));
    assert_eq!(
        allowed.approval_id,
        input.approval.as_ref().map(|record| record.approval_id)
    );

    input.target.config_hash = "config:tampered".into();
    let denied = authorize_silent_session_action(&input);
    assert!(!denied.allowed);
    assert!(denied.reason.contains("does not match"));
}

#[test]
fn authority_writer_and_expiration_checks_fail_closed() {
    let mut input = request(SilentSessionAction::Start);
    approve(&mut input);
    input.authority.writer_ownership = false;
    assert!(!authorize_silent_session_action(&input).allowed);

    input.authority.writer_ownership = true;
    input.authority.context_authority = ContextAuthorityVerdict::Denied;
    assert!(!authorize_silent_session_action(&input).allowed);

    input.authority.context_authority = ContextAuthorityVerdict::Allowed;
    input.approval.as_mut().unwrap().expires_at = input.now - Duration::seconds(1);
    assert!(!authorize_silent_session_action(&input).allowed);
}

#[test]
fn cross_user_streams_are_denied_and_admin_views_are_redacted() {
    let mut show = request(SilentSessionAction::Show);
    show.principal.role = SilentSessionRole::Administrator;
    show.principal.os_user = "root".into();
    show.principal.scopes.insert(SilentSessionRouteScope::Admin);
    let decision = authorize_silent_session_action(&show);
    assert!(decision.allowed);
    assert_eq!(
        decision.projection,
        Some(AuthorizedProjection::RedactedSummary)
    );

    show.action = SilentSessionAction::FollowStream;
    assert!(!authorize_silent_session_action(&show).allowed);
}

#[test]
fn principals_approvals_audits_and_runner_nonces_are_durable() {
    let persistence = persistence();
    let now = Utc::now();
    let mut input = request(SilentSessionAction::SendInput);
    approve(&mut input);
    save_authorization_principal(&persistence, &input.principal, now).unwrap();
    let approval = input.approval.clone().unwrap();
    save_durable_approval(&persistence, &approval).unwrap();
    assert_eq!(
        load_authorization_principal(&persistence, &input.principal.principal_id).unwrap(),
        Some(input.principal.clone())
    );
    assert_eq!(
        load_durable_approval(&persistence, approval.approval_id).unwrap(),
        Some(approval.clone())
    );
    assert_eq!(
        load_durable_approval_by_idempotency(
            &persistence,
            &approval.operator_actor,
            &approval.issuance_idempotency_key,
        )
        .unwrap(),
        Some(approval)
    );

    let audit = redact_control_audit(ControlAuditInput {
        audit_id: ControlAuditId::new(),
        occurred_at: now,
        actor: input.principal.actor.clone(),
        action: input.action,
        project_root: input.target.project_root.clone(),
        continuity_id: input.target.continuity_id.clone(),
        session_id: input.target.session_id,
        run_id: input.target.run_id,
        approval_id: input.approval.as_ref().map(|record| record.approval_id),
        decision: authorize_silent_session_action(&input),
        details: serde_json::json!({"authorization":"Bearer must-not-persist"}),
    });
    append_redacted_control_audit(&persistence, &audit).unwrap();
    let loaded = load_redacted_control_audit(&persistence, audit.audit_id)
        .unwrap()
        .unwrap();
    assert_eq!(loaded, audit);
    assert!(
        !serde_json::to_string(&loaded)
            .unwrap()
            .contains("must-not-persist")
    );

    let runner = RunnerIdentity {
        principal_id: "runner:proof".into(),
        os_user: "wirebot".into(),
        socket_scope: "uid:proof".into(),
    };
    let command = AuthenticatedRunnerCommand::issue(
        RunnerCommandClaims {
            session_id: input.target.session_id.unwrap(),
            run_id: input.target.run_id.unwrap(),
            runner,
            action_digest: action_digest(&input),
            nonce: "nonce:durable".into(),
            issued_at: now,
            expires_at: now + Duration::minutes(1),
        },
        b"payload",
        b"key",
    )
    .unwrap();
    assert!(consume_runner_nonce(&persistence, &command, now).unwrap());
    assert!(!consume_runner_nonce(&persistence, &command, now).unwrap());
}

#[test]
fn missing_auth_scope_or_role_is_denied_before_approval() {
    let mut input = request(SilentSessionAction::SendInput);
    input.principal.authenticated = false;
    assert!(!authorize_silent_session_action(&input).allowed);

    input.principal.authenticated = true;
    input.principal.scopes = BTreeSet::new();
    assert!(!authorize_silent_session_action(&input).allowed);

    input
        .principal
        .scopes
        .insert(SilentSessionRouteScope::Control);
    input.principal.role = SilentSessionRole::Viewer;
    assert!(!authorize_silent_session_action(&input).allowed);
}
