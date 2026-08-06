//! Exact, fail-closed Workstream request-context extraction for Spec 158.

use crate::workstream_identity::{AttachmentKey, ContinuityId, WorkspaceBindingId, WorkstreamKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkstreamContextError {
    #[error("request does not contain exact Workstream authority")]
    MissingWorkstream,
    #[error("request Workstream and attachment owner disagree")]
    WorkstreamMismatch,
    #[error("request continuity and attachment continuity disagree")]
    ContinuityMismatch,
    #[error("request workspace binding and attachment workspace binding disagree")]
    WorkspaceBindingMismatch,
}

/// Canonical request context resolved before reducer execution.
///
/// Actor and authority payloads remain generic until their existing authority
/// contracts are migrated; neither participates in Workstream selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamContext<A, U> {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub actor: A,
    pub authority: U,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamContextInput<A, U> {
    pub explicit_workstream: Option<WorkstreamKey>,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub actor: A,
    pub authority: U,
}

impl<A, U> WorkstreamContext<A, U> {
    pub fn extract(input: WorkstreamContextInput<A, U>) -> Result<Self, WorkstreamContextError> {
        let attachment_owner = input
            .attachment
            .as_ref()
            .map(|attachment| attachment.workstream.clone());

        let workstream = match (&input.explicit_workstream, &attachment_owner) {
            (Some(explicit), Some(owner)) if explicit != owner => {
                return Err(WorkstreamContextError::WorkstreamMismatch)
            }
            (Some(explicit), _) => explicit.clone(),
            (None, Some(owner)) => owner.clone(),
            (None, None) => return Err(WorkstreamContextError::MissingWorkstream),
        };

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

        Ok(Self {
            workstream,
            continuity_id: input.continuity_id.or(attachment_continuity),
            attachment: input.attachment,
            workspace_binding_id: input.workspace_binding_id.or(attachment_binding),
            actor: input.actor,
            authority: input.authority,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{AttachmentId, InstanceId, ScopeRef, SessionId, WorkstreamId};

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
    fn exact_attachment_resolves_its_owner_without_ui_or_session_fallback() {
        let owner = workstream("delivery");
        let context = WorkstreamContext::extract(WorkstreamContextInput {
            explicit_workstream: None,
            continuity_id: None,
            attachment: Some(attachment(owner.clone())),
            workspace_binding_id: None,
            actor: "operator",
            authority: "confirmed",
        })
        .unwrap();
        assert_eq!(context.workstream, owner);
        assert_eq!(context.continuity_id.unwrap().as_str(), "continuity-a");
    }

    #[test]
    fn ambiguous_workstream_ownership_fails_closed() {
        let result = WorkstreamContext::extract(WorkstreamContextInput {
            explicit_workstream: Some(workstream("planning")),
            continuity_id: None,
            attachment: Some(attachment(workstream("delivery"))),
            workspace_binding_id: None,
            actor: "operator",
            authority: "confirmed",
        });
        assert_eq!(result, Err(WorkstreamContextError::WorkstreamMismatch));
    }

    #[test]
    fn continuity_or_session_without_workstream_cannot_resolve_context() {
        let result = WorkstreamContext::extract(WorkstreamContextInput {
            explicit_workstream: None,
            continuity_id: Some(ContinuityId::parse("continuity-a").unwrap()),
            attachment: None,
            workspace_binding_id: None,
            actor: "operator",
            authority: "confirmed",
        });
        assert_eq!(result, Err(WorkstreamContextError::MissingWorkstream));
    }
}
