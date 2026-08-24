use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{ApprovalId, SilentSessionId, SilentSessionRunId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionRouteScope {
    Read,
    Stream,
    Create,
    Control,
    Config,
    Admin,
    Forensics,
}

impl SilentSessionRouteScope {
    pub fn as_str(self) -> &'static str {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionRole {
    Viewer,
    Operator,
    Administrator,
    Runner,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    pub principal_id: String,
    pub actor: String,
    pub role: SilentSessionRole,
    pub os_user: String,
    pub scopes: BTreeSet<SilentSessionRouteScope>,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionAction {
    List,
    Show,
    FollowStream,
    Create,
    Preflight,
    Start,
    SendInput,
    Pause,
    Resume,
    Interrupt,
    Cancel,
    Restart,
    PreviewConfig,
    ReviseConfig,
    RollbackConfig,
    Adopt,
    ForceKill,
    ReadRawForensics,
    Export,
    EvidenceHold,
    Delete,
    Purge,
}

impl SilentSessionAction {
    pub fn required_scope(self) -> SilentSessionRouteScope {
        match self {
            Self::List | Self::Show | Self::Export => SilentSessionRouteScope::Read,
            Self::FollowStream => SilentSessionRouteScope::Stream,
            Self::Create | Self::Preflight | Self::Start => SilentSessionRouteScope::Create,
            Self::SendInput
            | Self::Pause
            | Self::Resume
            | Self::Interrupt
            | Self::Cancel
            | Self::Restart => SilentSessionRouteScope::Control,
            Self::PreviewConfig | Self::ReviseConfig | Self::RollbackConfig => {
                SilentSessionRouteScope::Config
            }
            Self::Adopt | Self::ForceKill | Self::EvidenceHold | Self::Delete | Self::Purge => {
                SilentSessionRouteScope::Admin
            }
            Self::ReadRawForensics => SilentSessionRouteScope::Forensics,
        }
    }

    pub fn requires_approval(self) -> bool {
        matches!(
            self,
            Self::Start
                | Self::SendInput
                | Self::Interrupt
                | Self::Cancel
                | Self::Restart
                | Self::ReviseConfig
                | Self::RollbackConfig
                | Self::Adopt
                | Self::ForceKill
                | Self::ReadRawForensics
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationTarget {
    pub project_root: String,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub owner_os_user: String,
    pub writer_principal_id: Option<String>,
    pub config_hash: String,
    pub model_binding: String,
    pub workspace: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextAuthorityVerdict {
    NotRequired,
    Allowed,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedAuthorityFacts {
    pub project_permission: bool,
    pub continuity_permission: bool,
    pub work_item_permission: bool,
    pub writer_ownership: bool,
    pub authorized_project_root: String,
    pub authorized_continuity_id: String,
    pub authorized_work_item_ref: Option<String>,
    pub writer_principal_id: Option<String>,
    pub context_authority: ContextAuthorityVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DurableApprovalRecord {
    pub approval_id: ApprovalId,
    pub operator_actor: String,
    pub action: SilentSessionAction,
    pub project_root: String,
    pub continuity_id: String,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub config_hash: String,
    pub action_digest: String,
    pub model_binding: String,
    pub workspace: String,
    pub risk_class: String,
    pub expires_at: DateTime<Utc>,
    pub permitted_side_effects: Vec<String>,
    #[serde(default)]
    pub issuance_idempotency_key: String,
    #[serde(default)]
    pub issuance_request_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionAuthorizationRequest {
    pub principal: AuthenticatedPrincipal,
    pub action: SilentSessionAction,
    pub target: AuthorizationTarget,
    pub authority: VerifiedAuthorityFacts,
    pub approval: Option<DurableApprovalRecord>,
    pub approval_durably_verified: bool,
    pub legacy_approved: bool,
    pub requested_side_effects: Vec<String>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizedProjection {
    Full,
    RedactedSummary,
    RawForensics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub projection: Option<AuthorizedProjection>,
    pub reason: String,
    pub approval_id: Option<ApprovalId>,
}

pub fn authorize_silent_session_action(
    request: &SilentSessionAuthorizationRequest,
) -> AuthorizationDecision {
    authorize_silent_session_action_internal(request, true)
}

/// Pre-authorize issuance of a durable approval without pretending that an
/// approval is already persisted. All principal, scope, role, authority,
/// writer, and cross-user checks remain identical to action authorization;
/// only the final approval-record verification is deferred until the caller
/// constructs and durably stores the exact digest-bound record.
pub fn authorize_silent_session_approval_issuance(
    request: &SilentSessionAuthorizationRequest,
) -> AuthorizationDecision {
    if !request.action.requires_approval() {
        return AuthorizationDecision {
            allowed: false,
            projection: None,
            reason: "approval issuance is valid only for approval-required actions".into(),
            approval_id: None,
        };
    }
    authorize_silent_session_action_internal(request, false)
}

fn authorize_silent_session_action_internal(
    request: &SilentSessionAuthorizationRequest,
    verify_approval_record: bool,
) -> AuthorizationDecision {
    let deny = |reason: &str| AuthorizationDecision {
        allowed: false,
        projection: None,
        reason: reason.into(),
        approval_id: None,
    };
    if !request.principal.authenticated {
        return deny("principal is not authenticated");
    }
    let required = request.action.required_scope();
    if !request.principal.scopes.contains(&required) {
        return deny("required route scope is missing");
    }
    if !role_allows(request.principal.role, request.action) {
        return deny("principal role does not permit action");
    }
    if !request.authority.project_permission
        || !request.authority.continuity_permission
        || !request.authority.work_item_permission
        || request.authority.authorized_project_root != request.target.project_root
        || request.authority.authorized_continuity_id != request.target.continuity_id
        || request.authority.authorized_work_item_ref != request.target.work_item_ref
    {
        return deny("project, continuity or work-item permission denied");
    }
    if request.authority.context_authority == ContextAuthorityVerdict::Denied {
        return deny("Context Authority denied action");
    }
    if requires_writer(request.action)
        && (!request.authority.writer_ownership
            || request.authority.writer_principal_id.as_deref()
                != Some(request.principal.principal_id.as_str())
            || request.target.writer_principal_id != request.authority.writer_principal_id)
    {
        return deny("writer ownership is required");
    }
    let cross_user = request.principal.os_user != request.target.owner_os_user;
    if cross_user {
        if request.action == SilentSessionAction::FollowStream {
            return deny("cross-user stream access is denied");
        }
        if !request
            .principal
            .scopes
            .contains(&SilentSessionRouteScope::Admin)
        {
            return deny("cross-user access requires admin scope");
        }
    }
    let approval_id = if request.action.requires_approval() && verify_approval_record {
        match verify_approval(request) {
            Ok(id) => Some(id),
            Err(reason) => return deny(reason),
        }
    } else {
        None
    };
    let projection = if request.action == SilentSessionAction::ReadRawForensics {
        if !request
            .principal
            .scopes
            .contains(&SilentSessionRouteScope::Admin)
            && cross_user
        {
            return deny("cross-user raw output requires admin and forensic scopes");
        }
        AuthorizedProjection::RawForensics
    } else if cross_user {
        AuthorizedProjection::RedactedSummary
    } else {
        AuthorizedProjection::Full
    };
    AuthorizationDecision {
        allowed: true,
        projection: Some(projection),
        reason: "durable authorization context verified".into(),
        approval_id,
    }
}

pub fn action_digest(request: &SilentSessionAuthorizationRequest) -> String {
    let payload = serde_json::json!({
        "action": request.action,
        "project_root": request.target.project_root,
        "continuity_id": request.target.continuity_id,
        "work_item_ref": request.target.work_item_ref,
        "session_id": request.target.session_id,
        "run_id": request.target.run_id,
        "config_hash": request.target.config_hash,
        "model_binding": request.target.model_binding,
        "workspace": request.target.workspace,
        "requested_side_effects": request.requested_side_effects,
    });
    let bytes = serde_json::to_vec(&payload).expect("authorization digest payload serializes");
    hex::encode(Sha256::digest(bytes))
}

fn verify_approval(
    request: &SilentSessionAuthorizationRequest,
) -> Result<ApprovalId, &'static str> {
    if !request.approval_durably_verified {
        return Err("durably verified approval is required");
    }
    let approval = request
        .approval
        .as_ref()
        .ok_or("approval record is missing")?;
    if approval.expires_at < request.now {
        return Err("approval record is expired");
    }
    if approval.operator_actor.trim().is_empty() || approval.risk_class.trim().is_empty() {
        return Err("approval actor and risk class are required");
    }
    if approval.action != request.action
        || approval.project_root != request.target.project_root
        || approval.continuity_id != request.target.continuity_id
        || approval.session_id != request.target.session_id
        || approval.run_id != request.target.run_id
        || approval.config_hash != request.target.config_hash
        || approval.model_binding != request.target.model_binding
        || approval.workspace != request.target.workspace
        || approval.action_digest != action_digest(request)
    {
        return Err("approval record does not match requested action");
    }
    let permitted = approval
        .permitted_side_effects
        .iter()
        .collect::<BTreeSet<_>>();
    if request
        .requested_side_effects
        .iter()
        .any(|effect| !permitted.contains(effect))
    {
        return Err("requested side effect is not approved");
    }
    Ok(approval.approval_id)
}

fn requires_writer(action: SilentSessionAction) -> bool {
    matches!(
        action,
        SilentSessionAction::Start
            | SilentSessionAction::SendInput
            | SilentSessionAction::Restart
            | SilentSessionAction::ReviseConfig
            | SilentSessionAction::RollbackConfig
    )
}

fn role_allows(role: SilentSessionRole, action: SilentSessionAction) -> bool {
    match role {
        SilentSessionRole::Viewer => matches!(
            action,
            SilentSessionAction::List
                | SilentSessionAction::Show
                | SilentSessionAction::FollowStream
        ),
        SilentSessionRole::Operator => !matches!(
            action,
            SilentSessionAction::Adopt
                | SilentSessionAction::ForceKill
                | SilentSessionAction::ReadRawForensics
                | SilentSessionAction::EvidenceHold
                | SilentSessionAction::Delete
                | SilentSessionAction::Purge
        ),
        SilentSessionRole::Administrator => true,
        SilentSessionRole::Runner => false,
    }
}
