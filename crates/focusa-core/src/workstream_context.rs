//! Exact, fail-closed Workstream request-context extraction for Spec 158.
//!
//! A canonical request carries an exact WorkstreamKey or an exact AttachmentKey.
//! No presentation, filesystem, recency, or subordinate runtime identifier is an
//! ownership resolver.

use crate::scoped_state::{AuthorityEnvelope, AuthorityStatus};
use crate::silent_sessions::{AuthenticatedPrincipal, SilentSessionRole, VerifiedAuthorityFacts};
use crate::workstream_identity::{AttachmentKey, ContinuityId, WorkspaceBindingId, WorkstreamKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const WORKSTREAM_OPERATION_REQUEST_SCHEMA: &str = "focusa.workstream_operation_request.v1";

fn default_request_schema() -> String {
    WORKSTREAM_OPERATION_REQUEST_SCHEMA.to_string()
}

fn default_request_input() -> Value {
    Value::Null
}

/// The stable category of the actor that issued a canonical request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Operator,
    Agent,
    Pi,
    Desktop,
    Web,
    Service,
}

/// Typed actor ownership for a canonical request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActorRef {
    pub actor_type: ActorType,
    pub actor_id: String,
}

impl ActorRef {
    pub fn new(
        actor_type: ActorType,
        actor_id: impl Into<String>,
    ) -> Result<Self, WorkstreamContextError> {
        let actor_id = actor_id.into().trim().to_string();
        if actor_id.is_empty() {
            return Err(WorkstreamContextError::MissingActor);
        }
        Ok(Self {
            actor_type,
            actor_id,
        })
    }

    /// Adapt the existing API authentication owner without making an API
    /// principal the Workstream identity itself.
    pub fn from_authenticated_principal(
        principal: &AuthenticatedPrincipal,
    ) -> Result<Self, WorkstreamContextError> {
        let actor_type = match principal.role {
            SilentSessionRole::Runner => ActorType::Service,
            SilentSessionRole::Viewer
            | SilentSessionRole::Operator
            | SilentSessionRole::Administrator => ActorType::Operator,
        };
        Self::new(actor_type, principal.actor.clone())
    }
}

/// Typed authority carried by a canonical request.
///
/// `AuthorityEnvelope` is the existing scoped authority owner. Optional
/// `VerifiedAuthorityFacts` retain the existing Silent Session/API decision
/// facts when a request has them; the bounded wrapper gives Workstream requests
/// one concrete authority seam without creating a second permission system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityContext {
    pub authority_ref: String,
    pub envelope: AuthorityEnvelope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_facts: Option<VerifiedAuthorityFacts>,
}

impl AuthorityContext {
    pub fn canonical(authority_ref: impl Into<String>, why: impl Into<String>) -> Self {
        Self {
            authority_ref: authority_ref.into(),
            envelope: AuthorityEnvelope {
                status: AuthorityStatus::Canonical,
                why: why.into(),
            },
            verified_facts: None,
        }
    }

    pub fn from_verified_facts(
        authority_ref: impl Into<String>,
        why: impl Into<String>,
        verified_facts: VerifiedAuthorityFacts,
    ) -> Self {
        Self {
            authority_ref: authority_ref.into(),
            envelope: AuthorityEnvelope {
                status: AuthorityStatus::Canonical,
                why: why.into(),
            },
            verified_facts: Some(verified_facts),
        }
    }

    fn validate(&self) -> Result<(), WorkstreamContextError> {
        if self.authority_ref.trim().is_empty() {
            return Err(WorkstreamContextError::MissingAuthority);
        }
        if self.envelope.status != AuthorityStatus::Canonical {
            return Err(WorkstreamContextError::AuthorityDenied);
        }
        if self.envelope.why.trim().is_empty() {
            return Err(WorkstreamContextError::InvalidAuthority);
        }
        if let Some(facts) = self.verified_facts.as_ref() {
            if !facts.project_permission
                || !facts.continuity_permission
                || !facts.work_item_permission
            {
                return Err(WorkstreamContextError::AuthorityDenied);
            }
            if facts.authorized_project_root.trim().is_empty()
                || facts.authorized_continuity_id.trim().is_empty()
            {
                return Err(WorkstreamContextError::InvalidAuthority);
            }
            if facts.context_authority == crate::silent_sessions::ContextAuthorityVerdict::Denied {
                return Err(WorkstreamContextError::AuthorityDenied);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkstreamContextError {
    #[error("request does not contain an actor reference")]
    MissingActor,
    #[error("request actor reference is invalid")]
    InvalidActor,
    #[error("request does not contain canonical authority")]
    MissingAuthority,
    #[error("request authority is denied or non-canonical")]
    AuthorityDenied,
    #[error("request authority is invalid")]
    InvalidAuthority,
    #[error("request envelope schema is invalid")]
    InvalidEnvelope,
    #[error("request does not contain exact Workstream authority")]
    MissingWorkstream,
    #[error("request Workstream and attachment owner disagree")]
    WorkstreamMismatch,
    #[error("request WorkstreamKey is invalid")]
    InvalidWorkstream,
    #[error("request continuity and attachment continuity disagree")]
    ContinuityMismatch,
    #[error("request workspace binding and attachment workspace binding disagree")]
    WorkspaceBindingMismatch,
}

/// The canonical request envelope consumed before a Workstream reducer event is
/// constructed. The `workstream` field is optional only because an exact
/// AttachmentKey can carry the owner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamRequestEnvelope {
    #[serde(default = "default_request_schema")]
    pub schema: String,
    #[serde(rename = "workstream")]
    pub workstream: Option<WorkstreamKey>,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub actor: ActorRef,
    pub authority: AuthorityContext,
    #[serde(default)]
    pub command_id: String,
    #[serde(default = "default_request_input")]
    pub input: Value,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub expected_revision: Option<u64>,
    /// Fencing cursor for authority-sensitive commands.  A request that does
    /// not carry the current cursor cannot acquire reducer authority.
    #[serde(default)]
    pub expected_fencing_token: Option<u64>,
}

/// Compatibility name for callers that construct the extraction input directly.
/// It is the canonical request envelope, not a second request path.
pub type WorkstreamContextInput = WorkstreamRequestEnvelope;

impl WorkstreamRequestEnvelope {
    pub fn new(
        workstream: Option<WorkstreamKey>,
        attachment: Option<AttachmentKey>,
        actor: ActorRef,
        authority: AuthorityContext,
    ) -> Self {
        Self {
            schema: default_request_schema(),
            workstream,
            continuity_id: None,
            attachment,
            workspace_binding_id: None,
            actor,
            authority,
            command_id: String::new(),
            input: Value::Null,
            idempotency_key: None,
            expected_revision: None,
            expected_fencing_token: None,
        }
    }

    pub fn with_expected_fencing_token(mut self, expected_fencing_token: u64) -> Self {
        self.expected_fencing_token = Some(expected_fencing_token);
        self
    }

    pub fn for_workstream(
        workstream: WorkstreamKey,
        actor: ActorRef,
        authority: AuthorityContext,
    ) -> Self {
        Self::new(Some(workstream), None, actor, authority)
    }

    pub fn for_attachment(
        attachment: AttachmentKey,
        actor: ActorRef,
        authority: AuthorityContext,
    ) -> Self {
        Self::new(None, Some(attachment), actor, authority)
    }

    pub fn with_expected_revision(mut self, expected_revision: u64) -> Self {
        self.expected_revision = Some(expected_revision);
        self
    }
}

/// Canonical request context resolved before reducer execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamContext {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub actor: ActorRef,
    pub authority: AuthorityContext,
}

impl WorkstreamContext {
    pub fn extract(input: WorkstreamRequestEnvelope) -> Result<Self, WorkstreamContextError> {
        if input.schema != WORKSTREAM_OPERATION_REQUEST_SCHEMA {
            return Err(WorkstreamContextError::InvalidEnvelope);
        }
        if input.actor.actor_id.trim().is_empty() {
            return Err(WorkstreamContextError::InvalidActor);
        }
        input.authority.validate()?;

        let attachment_owner = input
            .attachment
            .as_ref()
            .map(|attachment| attachment.workstream.clone());

        let workstream = match (&input.workstream, &attachment_owner) {
            (Some(explicit), Some(owner)) if explicit != owner => {
                return Err(WorkstreamContextError::WorkstreamMismatch);
            }
            (Some(explicit), _) => explicit.clone(),
            (None, Some(owner)) => owner.clone(),
            (None, None) => return Err(WorkstreamContextError::MissingWorkstream),
        };
        validate_workstream_key(&workstream)?;

        if let Some(attachment) = input.attachment.as_ref() {
            validate_workstream_key(&attachment.workstream)?;
            attachment
                .validate_owner(&workstream)
                .map_err(|_| WorkstreamContextError::WorkstreamMismatch)?;
        }

        let attachment_continuity = input
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone());
        if matches!(
            (&input.continuity_id, &attachment_continuity),
            (Some(requested), Some(attached)) if requested != attached
        ) {
            return Err(WorkstreamContextError::ContinuityMismatch);
        }

        let attachment_binding = input
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone());
        if matches!(
            (&input.workspace_binding_id, &attachment_binding),
            (Some(requested), Some(attached)) if requested != attached
        ) {
            return Err(WorkstreamContextError::WorkspaceBindingMismatch);
        }

        let context = Self {
            workstream,
            continuity_id: input.continuity_id.or(attachment_continuity),
            attachment: input.attachment,
            workspace_binding_id: input.workspace_binding_id.or(attachment_binding),
            actor: input.actor,
            authority: input.authority,
        };
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), WorkstreamContextError> {
        if self.actor.actor_id.trim().is_empty() {
            return Err(WorkstreamContextError::InvalidActor);
        }
        self.authority.validate()?;
        validate_workstream_key(&self.workstream)?;
        if let Some(attachment) = self.attachment.as_ref() {
            attachment
                .validate_owner(&self.workstream)
                .map_err(|_| WorkstreamContextError::WorkstreamMismatch)?;
            if matches!(
                (&self.continuity_id, &attachment.continuity_id),
                (Some(requested), Some(attached)) if requested != attached
            ) {
                return Err(WorkstreamContextError::ContinuityMismatch);
            }
            if self.workspace_binding_id.as_ref() != Some(&attachment.workspace_binding_id) {
                return Err(WorkstreamContextError::WorkspaceBindingMismatch);
            }
        }
        Ok(())
    }

    pub fn validate_for_workstream(
        &self,
        expected: &WorkstreamKey,
    ) -> Result<(), WorkstreamContextError> {
        self.validate()?;
        if &self.workstream != expected {
            return Err(WorkstreamContextError::WorkstreamMismatch);
        }
        Ok(())
    }
}

fn validate_workstream_key(key: &WorkstreamKey) -> Result<(), WorkstreamContextError> {
    if key.workstream_id.as_str().trim().is_empty() {
        return Err(WorkstreamContextError::InvalidWorkstream);
    }
    key.legacy_scope()
        .validate()
        .map_err(|_| WorkstreamContextError::InvalidWorkstream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{AttachmentId, InstanceId, ScopeRef, SessionId, WorkstreamId};
    use std::collections::BTreeSet;

    fn actor() -> ActorRef {
        ActorRef::new(ActorType::Operator, "actor:operator").unwrap()
    }

    fn authority() -> AuthorityContext {
        AuthorityContext::canonical("authority:test", "exact authority test fixture")
    }

    fn workstream(id: &str) -> WorkstreamKey {
        let scope = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamKey::new(
            ScopeRef::project(scope).unwrap(),
            WorkstreamId::parse(id).unwrap(),
        )
    }

    fn attachment(owner: WorkstreamKey) -> AttachmentKey {
        AttachmentKey::new(
            owner,
            Some(ContinuityId::parse("continuity-a").unwrap()),
            InstanceId::parse("instance-a").unwrap(),
            SessionId::parse("session-a").unwrap(),
            AttachmentId::parse("attachment-a").unwrap(),
            WorkspaceBindingId::parse("workspace-a").unwrap(),
        )
    }

    #[test]
    fn exact_workstream_request_resolves_with_concrete_actor_and_authority() {
        let owner = workstream("delivery");
        let context = WorkstreamContext::extract(WorkstreamRequestEnvelope::for_workstream(
            owner.clone(),
            actor(),
            authority(),
        ))
        .unwrap();
        assert_eq!(context.workstream, owner);
        assert_eq!(context.actor.actor_type, ActorType::Operator);
        assert_eq!(
            context.authority.envelope.status,
            AuthorityStatus::Canonical
        );
    }

    #[test]
    fn exact_attachment_resolves_its_owner_without_inference() {
        let owner = workstream("delivery");
        let context = WorkstreamContext::extract(WorkstreamRequestEnvelope::for_attachment(
            attachment(owner.clone()),
            actor(),
            authority(),
        ))
        .unwrap();
        assert_eq!(context.workstream, owner);
        assert_eq!(context.continuity_id.unwrap().as_str(), "continuity-a");
    }

    #[test]
    fn ambiguous_workstream_ownership_fails_closed() {
        let result = WorkstreamContext::extract(WorkstreamRequestEnvelope::new(
            Some(workstream("planning")),
            Some(attachment(workstream("delivery"))),
            actor(),
            authority(),
        ));
        assert_eq!(result, Err(WorkstreamContextError::WorkstreamMismatch));
    }

    #[test]
    fn continuity_only_request_cannot_resolve_context() {
        let mut request = WorkstreamRequestEnvelope::new(None, None, actor(), authority());
        request.continuity_id = Some(ContinuityId::parse("continuity-a").unwrap());
        assert_eq!(
            WorkstreamContext::extract(request),
            Err(WorkstreamContextError::MissingWorkstream)
        );
    }

    #[test]
    fn session_only_request_cannot_resolve_context() {
        let mut request = WorkstreamRequestEnvelope::new(None, None, actor(), authority());
        request.input = serde_json::json!({ "session_id": "session-only" });
        assert_eq!(
            WorkstreamContext::extract(request),
            Err(WorkstreamContextError::MissingWorkstream)
        );
    }

    #[test]
    fn missing_actor_and_authority_fail_closed() {
        let mut request =
            WorkstreamRequestEnvelope::for_workstream(workstream("delivery"), actor(), authority());
        request.actor.actor_id.clear();
        assert_eq!(
            WorkstreamContext::extract(request),
            Err(WorkstreamContextError::InvalidActor)
        );

        let mut request =
            WorkstreamRequestEnvelope::for_workstream(workstream("delivery"), actor(), authority());
        request.authority.authority_ref.clear();
        assert_eq!(
            WorkstreamContext::extract(request),
            Err(WorkstreamContextError::MissingAuthority)
        );
    }

    #[test]
    fn non_canonical_authority_fails_closed() {
        let mut blocked = authority();
        blocked.envelope.status = AuthorityStatus::Blocked;
        assert_eq!(
            WorkstreamContext::extract(WorkstreamRequestEnvelope::for_workstream(
                workstream("delivery"),
                actor(),
                blocked,
            )),
            Err(WorkstreamContextError::AuthorityDenied)
        );
    }

    #[test]
    fn conflicting_attachment_metadata_fails_closed() {
        let mut request = WorkstreamRequestEnvelope::for_attachment(
            attachment(workstream("delivery")),
            actor(),
            authority(),
        );
        request.continuity_id = Some(ContinuityId::parse("continuity-foreign").unwrap());
        assert_eq!(
            WorkstreamContext::extract(request),
            Err(WorkstreamContextError::ContinuityMismatch)
        );
    }

    #[test]
    fn request_envelope_has_no_presentation_or_runtime_owner_fallback() {
        let request =
            WorkstreamRequestEnvelope::for_workstream(workstream("delivery"), actor(), authority());
        let encoded = serde_json::to_value(request).unwrap();
        for field in [
            "ui_selection",
            "focused_work_surface_id",
            "current_project",
            "cwd",
            "latest_record",
            "similarity",
        ] {
            assert!(
                encoded.get(field).is_none(),
                "unexpected fallback field: {field}"
            );
        }
    }

    #[test]
    fn authenticated_api_principal_adapts_to_typed_actor_ref() {
        let principal = AuthenticatedPrincipal {
            principal_id: "principal:operator".into(),
            actor: "actor:operator".into(),
            role: SilentSessionRole::Operator,
            os_user: "operator".into(),
            scopes: BTreeSet::new(),
            authenticated: true,
        };
        assert_eq!(
            ActorRef::from_authenticated_principal(&principal)
                .unwrap()
                .actor_id,
            "actor:operator"
        );
    }
}
