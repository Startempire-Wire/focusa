use std::collections::BTreeSet;

use chrono::{Duration, Utc};
use serde_json::json;

use crate::silent_sessions::*;

#[test]
fn runner_commands_are_bound_authenticated_expiring_and_replay_safe() {
    let runner = RunnerIdentity {
        principal_id: "runner:proof".into(),
        os_user: "wirebot".into(),
        socket_scope: "uid:1000/session:proof".into(),
    };
    let now = Utc::now();
    let key = b"test-only-runner-authentication-key";
    let command = AuthenticatedRunnerCommand::issue(
        RunnerCommandClaims {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            runner: runner.clone(),
            action_digest: "action:digest".into(),
            nonce: "nonce:unique".into(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        },
        b"bounded-command-payload",
        key,
    )
    .unwrap();
    let serialized = serde_json::to_string(&command).unwrap();
    assert!(!serialized.contains("bounded-command-payload"));
    assert!(!serialized.contains("test-only-runner-authentication-key"));

    let mut consumed = BTreeSet::new();
    command
        .authenticate(&runner, now, key, &mut consumed)
        .unwrap();
    assert_eq!(
        command.authenticate(&runner, now, key, &mut consumed),
        Err(RunnerAuthenticationError::Replay)
    );

    let mut fresh = BTreeSet::new();
    let wrong_runner = RunnerIdentity {
        os_user: "root".into(),
        ..runner.clone()
    };
    assert_eq!(
        command.authenticate(&wrong_runner, now, key, &mut fresh),
        Err(RunnerAuthenticationError::IdentityMismatch)
    );
    assert_eq!(
        command.authenticate(&runner, now + Duration::minutes(1), key, &mut fresh,),
        Err(RunnerAuthenticationError::Expired)
    );
}

#[test]
fn runner_command_tampering_fails_authentication() {
    let runner = RunnerIdentity {
        principal_id: "runner:proof".into(),
        os_user: "wirebot".into(),
        socket_scope: "uid:1000/session:proof".into(),
    };
    let now = Utc::now();
    let mut command = AuthenticatedRunnerCommand::issue(
        RunnerCommandClaims {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            runner: runner.clone(),
            action_digest: "action:digest".into(),
            nonce: "nonce:tamper".into(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        },
        b"payload",
        b"key",
    )
    .unwrap();
    command.action_digest = "tampered".into();
    assert_eq!(
        command.authenticate(&runner, now, b"key", &mut BTreeSet::new()),
        Err(RunnerAuthenticationError::InvalidTag)
    );
}

#[test]
fn control_audit_preserves_facts_and_redacts_all_secret_classes() {
    let input = ControlAuditInput {
        audit_id: ControlAuditId::new(),
        occurred_at: Utc::now(),
        actor: "actor:proof".into(),
        action: SilentSessionAction::ReviseConfig,
        project_root: "/srv/focusa".into(),
        continuity_id: "continuity:proof".into(),
        session_id: Some(SilentSessionId::new()),
        run_id: Some(SilentSessionRunId::new()),
        approval_id: Some(ApprovalId::new()),
        decision: AuthorizationDecision {
            allowed: true,
            projection: Some(AuthorizedProjection::Full),
            reason: "verified".into(),
            approval_id: None,
        },
        details: json!({
            "authorization": "Bearer raw-token",
            "provider_credential": "provider-secret",
            "secret_environment_value": "env-secret",
            "private_key": "-----BEGIN PRIVATE KEY----- raw-key",
            "connector_secret": "connector-secret",
            "auth_profile_ref": "auth:reference",
            "max_tokens": 1000,
            "nested": {"api_token": "api-secret"}
        }),
    };
    let record = redact_control_audit(input);
    let serialized = serde_json::to_string(&record).unwrap();
    for forbidden in [
        "raw-token",
        "provider-secret",
        "env-secret",
        "raw-key",
        "connector-secret",
        "api-secret",
    ] {
        assert!(!serialized.contains(forbidden));
    }
    assert!(serialized.contains("auth:reference"));
    assert!(serialized.contains("1000"));
    assert!(record.redaction_classes.contains(&"auth_header".into()));
    assert!(
        record
            .redaction_classes
            .contains(&"private_key_material".into())
    );
}

#[test]
fn runner_command_rejects_payload_substitution_before_nonce_consumption() {
    let runner = RunnerIdentity {
        principal_id: "runner:payload-proof".into(),
        os_user: "wirebot".into(),
        socket_scope: "uid:1000/session:payload-proof".into(),
    };
    let now = Utc::now();
    let key = b"test-only-runner-authentication-key";
    let command = AuthenticatedRunnerCommand::issue(
        RunnerCommandClaims {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            runner: runner.clone(),
            action_digest: "action:digest".into(),
            nonce: "nonce:payload-proof".into(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        },
        b"original-payload",
        key,
    )
    .unwrap();
    let mut consumed = BTreeSet::new();
    assert_eq!(
        command.authenticate_payload(&runner, now, key, &mut consumed, b"substituted-payload",),
        Err(RunnerAuthenticationError::PayloadMismatch)
    );
    assert!(consumed.is_empty());
}
