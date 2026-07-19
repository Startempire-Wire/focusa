//! Spec 133 durable authorization for daemon-native Silent Sessions.
//! Authorization is fail-closed: a legacy `approved=true` flag is never sufficient.

use crate::silent_session::{SilentSessionId, SilentSessionLease, SilentSessionRunId};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;

pub const SILENT_SESSION_APPROVAL_SCHEMA: &str = "focusa.silent_session_approval.v1";
pub const SILENT_SESSION_RUNNER_CONTROL_SCHEMA: &str = "focusa.silent_session_runner_control.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionScope {
    Read,
    Stream,
    Create,
    Control,
    Config,
    Admin,
    Forensics,
}

impl SilentSessionScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "silent_sessions:read",
            Self::Stream => "silent_sessions:stream",
            Self::Create => "silent_sessions:create",
            Self::Control => "silent_sessions:control",
            Self::Config => "silent_sessions:config",
            Self::Admin => "silent_sessions:admin",
            Self::Forensics => "silent_sessions:forensics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionRole {
    Operator,
    Administrator,
    ForensicOperator,
    Runner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionPrincipal {
    pub actor_ref: String,
    pub actor_instance_ref: String,
    pub role: SilentSessionRole,
    pub os_user: String,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: Option<String>,
    pub scopes: BTreeSet<SilentSessionScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextAuthorityGrant {
    pub verdict_ref: String,
    pub allowed: bool,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionDigestMaterial {
    pub action: String,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub config_hash: String,
    pub model_binding: String,
    pub workspace_ref: String,
    pub permitted_side_effects: Vec<String>,
}

impl ActionDigestMaterial {
    pub fn digest(&self) -> Result<String, AuthorizationError> {
        let encoded = serde_json::to_vec(self).map_err(|_| AuthorizationError::InvalidDigest)?;
        Ok(hex::encode(Sha256::digest(encoded)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionApproval {
    pub schema: String,
    pub approval_id: String,
    pub operator_actor_ref: String,
    pub action: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: Option<String>,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub config_hash: String,
    pub action_digest: String,
    pub model_binding: String,
    pub workspace_ref: String,
    pub risk_class: String,
    pub expires_at: DateTime<Utc>,
    pub permitted_side_effects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SilentSessionAuthorizationRequest<'a> {
    pub legacy_approved: bool,
    pub required_scope: SilentSessionScope,
    pub principal: Option<&'a SilentSessionPrincipal>,
    pub approval: Option<&'a SilentSessionApproval>,
    pub action: &'a ActionDigestMaterial,
    pub context_authority: Option<&'a ContextAuthorityGrant>,
    pub lease: Option<&'a SilentSessionLease>,
    pub session_owner_os_user: &'a str,
    pub now: DateTime<Utc>,
    pub require_context_authority: bool,
    pub require_lease: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationGrant {
    pub approval_id: String,
    pub actor_ref: String,
    pub action_digest: String,
    pub context_authority_ref: Option<String>,
    pub lease_fencing_token: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthorizationError {
    #[error("an authenticated principal is required")]
    MissingPrincipal,
    #[error("legacy approved=true is insufficient without a durable approval")]
    MissingApproval,
    #[error("the principal lacks the exact route scope")]
    MissingScope,
    #[error("the approval has expired")]
    ApprovalExpired,
    #[error("the approval does not match the authenticated actor")]
    ActorMismatch,
    #[error("the authorization project does not match")]
    ProjectMismatch,
    #[error("the authorization continuity does not match")]
    ContinuityMismatch,
    #[error("the authorization Workpoint does not match")]
    WorkpointMismatch,
    #[error("the authorization action does not match")]
    ActionMismatch,
    #[error("the action digest is invalid or stale")]
    InvalidDigest,
    #[error("cross-user access is denied")]
    CrossUserDenied,
    #[error("a fresh Context Authority verdict is required")]
    ContextAuthorityDenied,
    #[error("a valid writer lease is required")]
    InvalidLease,
    #[error("runner control authentication failed")]
    RunnerAuthenticationFailed,
}

pub fn authorize_silent_session(
    request: &SilentSessionAuthorizationRequest<'_>,
) -> Result<AuthorizationGrant, AuthorizationError> {
    let principal = request
        .principal
        .ok_or(AuthorizationError::MissingPrincipal)?;
    let approval = request
        .approval
        .ok_or(AuthorizationError::MissingApproval)?;

    // Keep the legacy bit intentionally non-authoritative. This branch documents that
    // even true proceeds only through the durable checks below.
    let _legacy_approved = request.legacy_approved;

    if !principal.scopes.contains(&request.required_scope) {
        return Err(AuthorizationError::MissingScope);
    }
    if approval.schema != SILENT_SESSION_APPROVAL_SCHEMA || approval.expires_at <= request.now {
        return Err(AuthorizationError::ApprovalExpired);
    }
    if approval.operator_actor_ref != principal.actor_ref {
        return Err(AuthorizationError::ActorMismatch);
    }
    if principal.os_user != request.session_owner_os_user {
        return Err(AuthorizationError::CrossUserDenied);
    }
    if principal.project_root != request.action.project_root
        || principal.project_identity_ref != request.action.project_identity_ref
        || approval.project_identity_ref != request.action.project_identity_ref
    {
        return Err(AuthorizationError::ProjectMismatch);
    }
    if principal.continuity_id != request.action.continuity_id
        || approval.continuity_id != request.action.continuity_id
    {
        return Err(AuthorizationError::ContinuityMismatch);
    }
    if approval.workpoint_ref != principal.workpoint_ref {
        return Err(AuthorizationError::WorkpointMismatch);
    }
    if approval.action != request.action.action
        || approval.session_id != request.action.session_id
        || approval.run_id != request.action.run_id
        || approval.config_hash != request.action.config_hash
        || approval.model_binding != request.action.model_binding
        || approval.workspace_ref != request.action.workspace_ref
        || approval.permitted_side_effects != request.action.permitted_side_effects
    {
        return Err(AuthorizationError::ActionMismatch);
    }
    let digest = request.action.digest()?;
    if approval.action_digest != digest {
        return Err(AuthorizationError::InvalidDigest);
    }

    let context_authority_ref = if request.require_context_authority {
        let verdict = request
            .context_authority
            .ok_or(AuthorizationError::ContextAuthorityDenied)?;
        if !verdict.allowed || verdict.expires_at <= request.now {
            return Err(AuthorizationError::ContextAuthorityDenied);
        }
        if verdict.project_identity_ref != principal.project_identity_ref {
            return Err(AuthorizationError::ProjectMismatch);
        }
        if verdict.continuity_id != principal.continuity_id {
            return Err(AuthorizationError::ContinuityMismatch);
        }
        if verdict.workpoint_ref != principal.workpoint_ref {
            return Err(AuthorizationError::WorkpointMismatch);
        }
        Some(verdict.verdict_ref.clone())
    } else {
        None
    };

    let lease_fencing_token = if request.require_lease {
        let lease = request.lease.ok_or(AuthorizationError::InvalidLease)?;
        if lease.expires_at <= request.now
            || lease.fencing_token == 0
            || lease.owner_actor_instance_ref != principal.actor_instance_ref
            || lease.project_root != principal.project_root
            || lease.project_identity_ref != principal.project_identity_ref
            || lease.continuity_id != principal.continuity_id
            || Some(lease.session_id) != request.action.session_id
        {
            return Err(AuthorizationError::InvalidLease);
        }
        Some(lease.fencing_token)
    } else {
        None
    };

    Ok(AuthorizationGrant {
        approval_id: approval.approval_id.clone(),
        actor_ref: principal.actor_ref.clone(),
        action_digest: digest,
        context_authority_ref,
        lease_fencing_token,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossUserView {
    Full,
    RedactedSummary,
}

pub fn authorize_list_view(
    principal: &SilentSessionPrincipal,
    owner_os_user: &str,
) -> Result<CrossUserView, AuthorizationError> {
    if principal.os_user == owner_os_user {
        return principal
            .scopes
            .contains(&SilentSessionScope::Read)
            .then_some(CrossUserView::Full)
            .ok_or(AuthorizationError::MissingScope);
    }
    if principal.role == SilentSessionRole::Administrator
        && principal.scopes.contains(&SilentSessionScope::Admin)
    {
        return Ok(CrossUserView::RedactedSummary);
    }
    Err(AuthorizationError::CrossUserDenied)
}

pub fn authorize_stream(
    principal: &SilentSessionPrincipal,
    owner_os_user: &str,
) -> Result<(), AuthorizationError> {
    if principal.os_user != owner_os_user {
        return Err(AuthorizationError::CrossUserDenied);
    }
    principal
        .scopes
        .contains(&SilentSessionScope::Stream)
        .then_some(())
        .ok_or(AuthorizationError::MissingScope)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerControlCommand {
    pub schema: String,
    pub runner_id: String,
    pub os_user: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub session_id: SilentSessionId,
    pub action: String,
    pub action_digest: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}

impl RunnerControlCommand {
    pub fn signing_bytes(&self) -> Result<Vec<u8>, AuthorizationError> {
        serde_json::to_vec(self).map_err(|_| AuthorizationError::RunnerAuthenticationFailed)
    }
}

pub fn verify_runner_control(
    command: &RunnerControlCommand,
    signature_base64: &str,
    trusted_key: &VerifyingKey,
    expected_os_user: &str,
    expected_project_identity_ref: &str,
    now: DateTime<Utc>,
) -> Result<(), AuthorizationError> {
    if command.schema != SILENT_SESSION_RUNNER_CONTROL_SCHEMA
        || command.expires_at <= now
        || command.os_user != expected_os_user
        || command.project_identity_ref != expected_project_identity_ref
        || command.nonce.is_empty()
    {
        return Err(AuthorizationError::RunnerAuthenticationFailed);
    }
    let raw = BASE64
        .decode(signature_base64)
        .map_err(|_| AuthorizationError::RunnerAuthenticationFailed)?;
    let signature =
        Signature::from_slice(&raw).map_err(|_| AuthorizationError::RunnerAuthenticationFailed)?;
    trusted_key
        .verify(&command.signing_bytes()?, &signature)
        .map_err(|_| AuthorizationError::RunnerAuthenticationFailed)
}

pub fn redact_control_audit(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                    let secret = normalized.contains("token")
                        || normalized.contains("password")
                        || normalized.contains("secret")
                        || normalized.contains("credential")
                        || normalized.contains("privatekey")
                        || normalized == "authorization"
                        || normalized == "authheader";
                    (
                        key.clone(),
                        if secret {
                            Value::String("[REDACTED]".to_owned())
                        } else {
                            redact_control_audit(value)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_control_audit).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::SilentSessionLeaseId;
    use chrono::Duration;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn fixture() -> (
        SilentSessionPrincipal,
        ActionDigestMaterial,
        SilentSessionApproval,
        ContextAuthorityGrant,
        SilentSessionLease,
        DateTime<Utc>,
    ) {
        let now = Utc::now();
        let session_id = SilentSessionId::new();
        let action = ActionDigestMaterial {
            action: "start".into(),
            project_root: PathBuf::from("/work/focusa"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "continuity:one".into(),
            session_id: Some(session_id),
            run_id: None,
            config_hash: "config-hash".into(),
            model_binding: "anthropic/claude".into(),
            workspace_ref: "/work/focusa".into(),
            permitted_side_effects: vec!["write_workspace".into()],
        };
        let principal = SilentSessionPrincipal {
            actor_ref: "actor:operator".into(),
            actor_instance_ref: "actor-instance:1".into(),
            role: SilentSessionRole::Operator,
            os_user: "alice".into(),
            project_root: PathBuf::from("/work/focusa"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "continuity:one".into(),
            workpoint_ref: Some("workpoint:1".into()),
            scopes: [SilentSessionScope::Create, SilentSessionScope::Stream]
                .into_iter()
                .collect(),
        };
        let approval = SilentSessionApproval {
            schema: SILENT_SESSION_APPROVAL_SCHEMA.into(),
            approval_id: "approval:1".into(),
            operator_actor_ref: principal.actor_ref.clone(),
            action: action.action.clone(),
            project_identity_ref: action.project_identity_ref.clone(),
            continuity_id: action.continuity_id.clone(),
            workpoint_ref: principal.workpoint_ref.clone(),
            session_id: action.session_id,
            run_id: action.run_id,
            config_hash: action.config_hash.clone(),
            action_digest: action.digest().unwrap(),
            model_binding: action.model_binding.clone(),
            workspace_ref: action.workspace_ref.clone(),
            risk_class: "mutating".into(),
            expires_at: now + Duration::minutes(5),
            permitted_side_effects: action.permitted_side_effects.clone(),
        };
        let context = ContextAuthorityGrant {
            verdict_ref: "context-authority:1".into(),
            allowed: true,
            project_identity_ref: principal.project_identity_ref.clone(),
            continuity_id: principal.continuity_id.clone(),
            workpoint_ref: principal.workpoint_ref.clone(),
            expires_at: now + Duration::minutes(5),
        };
        let lease = SilentSessionLease {
            schema: "focusa.silent_session_lease.v1".into(),
            lease_id: SilentSessionLeaseId::new(),
            session_id,
            project_root: principal.project_root.clone(),
            project_identity_ref: principal.project_identity_ref.clone(),
            continuity_id: principal.continuity_id.clone(),
            work_item_ref: None,
            workspace_ref: action.workspace_ref.clone(),
            path_intents: vec![],
            writer_role: "primary".into(),
            owner_actor_instance_ref: principal.actor_instance_ref.clone(),
            fencing_token: 7,
            acquired_at: now,
            heartbeat_at: now,
            expires_at: now + Duration::minutes(1),
            adoption_policy: "operator_only".into(),
        };
        (principal, action, approval, context, lease, now)
    }

    #[test]
    fn approved_true_without_principal_or_durable_approval_is_rejected() {
        let (_, action, _, _, _, now) = fixture();
        let request = SilentSessionAuthorizationRequest {
            legacy_approved: true,
            required_scope: SilentSessionScope::Create,
            principal: None,
            approval: None,
            action: &action,
            context_authority: None,
            lease: None,
            session_owner_os_user: "alice",
            now,
            require_context_authority: false,
            require_lease: false,
        };
        assert_eq!(
            authorize_silent_session(&request),
            Err(AuthorizationError::MissingPrincipal)
        );
    }

    #[test]
    fn exact_scope_and_all_authority_bindings_are_required() {
        let (principal, action, approval, context, lease, now) = fixture();
        let request = SilentSessionAuthorizationRequest {
            legacy_approved: true,
            required_scope: SilentSessionScope::Create,
            principal: Some(&principal),
            approval: Some(&approval),
            action: &action,
            context_authority: Some(&context),
            lease: Some(&lease),
            session_owner_os_user: "alice",
            now,
            require_context_authority: true,
            require_lease: true,
        };
        let grant = authorize_silent_session(&request).unwrap();
        assert_eq!(grant.lease_fencing_token, Some(7));
        assert_eq!(
            grant.context_authority_ref.as_deref(),
            Some("context-authority:1")
        );

        let mut wrong_scope = principal.clone();
        wrong_scope.scopes.remove(&SilentSessionScope::Create);
        let denied = SilentSessionAuthorizationRequest {
            principal: Some(&wrong_scope),
            ..request
        };
        assert_eq!(
            authorize_silent_session(&denied),
            Err(AuthorizationError::MissingScope)
        );
    }

    #[test]
    fn expired_or_replayed_action_digest_is_rejected() {
        let (principal, mut action, approval, context, lease, now) = fixture();
        action.action = "cancel".into();
        let request = SilentSessionAuthorizationRequest {
            legacy_approved: true,
            required_scope: SilentSessionScope::Create,
            principal: Some(&principal),
            approval: Some(&approval),
            action: &action,
            context_authority: Some(&context),
            lease: Some(&lease),
            session_owner_os_user: "alice",
            now,
            require_context_authority: true,
            require_lease: true,
        };
        assert!(matches!(
            authorize_silent_session(&request),
            Err(AuthorizationError::ActionMismatch | AuthorizationError::InvalidDigest)
        ));

        let original_action = ActionDigestMaterial {
            action: "start".into(),
            ..action.clone()
        };
        let expired = SilentSessionAuthorizationRequest {
            action: &original_action,
            now: approval.expires_at,
            ..request
        };
        assert_eq!(
            authorize_silent_session(&expired),
            Err(AuthorizationError::ApprovalExpired)
        );
    }

    #[test]
    fn cross_user_list_is_redacted_for_admin_and_stream_is_denied() {
        let (mut principal, _, _, _, _, _) = fixture();
        principal.role = SilentSessionRole::Administrator;
        principal.scopes.insert(SilentSessionScope::Admin);
        assert_eq!(
            authorize_list_view(&principal, "bob"),
            Ok(CrossUserView::RedactedSummary)
        );
        assert_eq!(
            authorize_stream(&principal, "bob"),
            Err(AuthorizationError::CrossUserDenied)
        );
    }

    #[test]
    fn runner_controls_require_trusted_signature_scope_and_freshness() {
        let (_, action, _, _, _, now) = fixture();
        let signing_key = SigningKey::from_bytes(&[23; 32]);
        let action_digest = action.digest().unwrap();
        let command = RunnerControlCommand {
            schema: SILENT_SESSION_RUNNER_CONTROL_SCHEMA.into(),
            runner_id: "runner:1".into(),
            os_user: "alice".into(),
            project_identity_ref: action.project_identity_ref,
            continuity_id: action.continuity_id,
            session_id: action.session_id.unwrap(),
            action: action.action,
            action_digest,
            nonce: "nonce:unique".into(),
            expires_at: now + Duration::minutes(1),
        };
        let signature = signing_key.sign(&command.signing_bytes().unwrap());
        let encoded = BASE64.encode(signature.to_bytes());
        assert_eq!(
            verify_runner_control(
                &command,
                &encoded,
                &signing_key.verifying_key(),
                "alice",
                "project:focusa",
                now,
            ),
            Ok(())
        );
        assert_eq!(
            verify_runner_control(
                &command,
                &encoded,
                &signing_key.verifying_key(),
                "bob",
                "project:focusa",
                now,
            ),
            Err(AuthorizationError::RunnerAuthenticationFailed)
        );
    }

    #[test]
    fn control_audit_redacts_nested_secret_classes() {
        let audit = json!({
            "actor": "actor:operator",
            "authorization": "Bearer secret",
            "config": {"provider_credentials": "secret", "model": "safe"},
            "items": [{"private_key": "secret", "secret_ref": "also-redacted"}]
        });
        let redacted = redact_control_audit(&audit);
        assert_eq!(redacted["actor"], "actor:operator");
        assert_eq!(redacted["authorization"], "[REDACTED]");
        assert_eq!(redacted["config"]["provider_credentials"], "[REDACTED]");
        assert_eq!(redacted["config"]["model"], "safe");
        assert_eq!(redacted["items"][0]["private_key"], "[REDACTED]");
        assert_eq!(redacted["items"][0]["secret_ref"], "[REDACTED]");
    }
}
