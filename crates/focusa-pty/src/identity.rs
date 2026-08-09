//! Governed Attachment identity (mirror of the renderer contract).
//!
//! Identity is authoritative and exact: CWD, session, or remembered state
//! never grant authority. A process may only exist for a fully-qualified
//! project-scope Attachment whose every key field is present.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectScopeKey {
    pub scope_kind: String,
    pub scope_id: String,
    pub root_path: String,
    pub canonical_name: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScopeRef {
    pub scope_kind: String,
    pub scope_key: ProjectScopeKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkstreamKey {
    pub scope: ScopeRef,
    pub workstream_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AttachmentKey {
    pub attachment_id: String,
    pub continuity_id: Option<String>,
    pub instance_id: String,
    pub session_id: String,
    pub workspace_binding_id: String,
    pub workstream: WorkstreamKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PtyAttachmentIdentity {
    #[serde(flatten)]
    pub attachment_key: AttachmentKey,
    pub work_surface_id: String,
    pub runtime_object: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum IdentityValidationError {
    #[error("scope must be a project scope")]
    ScopeNotProject,
    #[error("scope key is missing fields")]
    ScopeKeyIncomplete,
    #[error("workstream id is missing")]
    MissingWorkstreamId,
    #[error("attachment id is missing")]
    MissingAttachmentId,
    #[error("workspace binding id is missing")]
    MissingWorkspaceBindingId,
    #[error("instance id is missing")]
    MissingInstanceId,
    #[error("session id is missing")]
    MissingSessionId,
    #[error("work surface id is missing")]
    MissingWorkSurfaceId,
}

impl AttachmentKey {
    /// True when every authority field is present and the scope is a project.
    pub fn is_exact(&self) -> bool {
        !self.attachment_id.is_empty()
            && !self.instance_id.is_empty()
            && !self.session_id.is_empty()
            && !self.workspace_binding_id.is_empty()
            && self.workstream.scope.scope_kind == "project"
            && !self.workstream.workstream_id.is_empty()
            && !self.workstream.scope.scope_key.scope_id.is_empty()
            && !self.workstream.scope.scope_key.root_path.is_empty()
            && !self.workstream.scope.scope_key.fingerprint.is_empty()
    }
}

impl PtyAttachmentIdentity {
    /// Fail-closed validation. Returns the first missing field reason.
    pub fn validate(&self) -> Result<(), IdentityValidationError> {
        if self.attachment_key.workstream.scope.scope_kind != "project" {
            return Err(IdentityValidationError::ScopeNotProject);
        }
        let scope_key = &self.attachment_key.workstream.scope.scope_key;
        if scope_key.scope_id.is_empty()
            || scope_key.root_path.is_empty()
            || scope_key.canonical_name.is_empty()
            || scope_key.fingerprint.is_empty()
        {
            return Err(IdentityValidationError::ScopeKeyIncomplete);
        }
        if self.attachment_key.workstream.workstream_id.is_empty() {
            return Err(IdentityValidationError::MissingWorkstreamId);
        }
        if self.attachment_key.attachment_id.is_empty() {
            return Err(IdentityValidationError::MissingAttachmentId);
        }
        if self.attachment_key.workspace_binding_id.is_empty() {
            return Err(IdentityValidationError::MissingWorkspaceBindingId);
        }
        if self.attachment_key.instance_id.is_empty() {
            return Err(IdentityValidationError::MissingInstanceId);
        }
        if self.attachment_key.session_id.is_empty() {
            return Err(IdentityValidationError::MissingSessionId);
        }
        if self.work_surface_id.is_empty() {
            return Err(IdentityValidationError::MissingWorkSurfaceId);
        }
        Ok(())
    }

    /// Stable registry key: canonical JSON serialization of the exact identity.
    pub fn registry_key(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn sample_identity() -> PtyAttachmentIdentity {
        PtyAttachmentIdentity {
            attachment_key: AttachmentKey {
                attachment_id: "attachment:pi".into(),
                continuity_id: Some("continuity:mission-canvas".into()),
                instance_id: "instance:pi".into(),
                session_id: "session:pi".into(),
                workspace_binding_id: "workspace:mission-canvas".into(),
                workstream: WorkstreamKey {
                    scope: ScopeRef {
                        scope_kind: "project".into(),
                        scope_key: ProjectScopeKey {
                            scope_kind: "project".into(),
                            scope_id: "project:focusa".into(),
                            root_path: "/example/focusa".into(),
                            canonical_name: "Focusa".into(),
                            fingerprint: "host-a:worktree-main".into(),
                        },
                    },
                    workstream_id: "ws:mission-canvas".into(),
                },
            },
            work_surface_id: "surface:pi".into(),
            runtime_object: Some("pi_session:session:pi".into()),
        }
    }

    #[test]
    fn exact_identity_validates() {
        assert_eq!(sample_identity().validate(), Ok(()));
    }

    #[test]
    fn missing_attachment_fails_closed() {
        let mut identity = sample_identity();
        identity.attachment_key.attachment_id = String::new();
        assert_eq!(
            identity.validate(),
            Err(IdentityValidationError::MissingAttachmentId)
        );
    }

    #[test]
    fn non_project_scope_fails_closed() {
        let mut identity = sample_identity();
        identity.attachment_key.workstream.scope.scope_kind = "host".into();
        assert_eq!(
            identity.validate(),
            Err(IdentityValidationError::ScopeNotProject)
        );
    }

    #[test]
    fn incomplete_scope_key_fails_closed() {
        let mut identity = sample_identity();
        identity.attachment_key.workstream.scope.scope_key.fingerprint = String::new();
        assert_eq!(
            identity.validate(),
            Err(IdentityValidationError::ScopeKeyIncomplete)
        );
    }

    #[test]
    fn registry_key_is_stable_and_distinct() {
        let a = sample_identity();
        let mut b = sample_identity();
        b.work_surface_id = "surface:other".into();
        let key_a = a.registry_key().unwrap();
        let key_b = b.registry_key().unwrap();
        assert_eq!(a.registry_key().unwrap(), key_a);
        assert_ne!(key_a, key_b);
    }
}
