//! Canonical Spec 158 scope identity types.
//!
//! This module is the migration target for the legacy flat `scoped_state::ScopeRef`.
//! During dual-read migration it wraps the validated legacy project/host keys without
//! allowing a filesystem path, continuity id, or current-session heuristic to become
//! canonical scope authority.

use crate::scoped_state::{ProjectRootKey, ScopeKeyError, ScopeKind, ScopeRef as LegacyScopeRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A validated host scope key used by canonical [`ScopeRef`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HostScopeKey(pub LegacyScopeRef);

impl HostScopeKey {
    pub fn new(scope: LegacyScopeRef) -> Result<Self, ScopeKeyError> {
        if scope.scope_kind != ScopeKind::Host {
            return Err(ScopeKeyError::KindMismatch {
                expected: ScopeKind::Host,
                found: scope.scope_kind,
            });
        }
        scope.validate()?;
        Ok(Self(scope))
    }

    pub fn storage_key(&self) -> String {
        self.0.storage_key()
    }
}

/// Canonical project-or-host scope discriminator.
///
/// The enum shape prevents callers from treating a path string as a complete scope.
/// Project keys retain the required scope id and fingerprint, which distinguish the
/// same path across hosts, worktrees, and workspace bindings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "scope_kind", content = "scope_key", rename_all = "snake_case")]
pub enum ScopeRef {
    Project(ProjectRootKey),
    Host(HostScopeKey),
}

/// Durable identity of one cognitive Workstream within a project or host scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct WorkstreamId(String);

impl WorkstreamId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ScopeKeyError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(ScopeKeyError::Missing("workstream_id"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkstreamId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

macro_rules! subordinate_id {
    ($name:ident, $field:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ScopeKeyError> {
                let value = value.into().trim().to_string();
                if value.is_empty() {
                    return Err(ScopeKeyError::Missing($field));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

subordinate_id!(ContinuityId, "continuity_id");
subordinate_id!(InstanceId, "instance_id");
subordinate_id!(SessionId, "session_id");
subordinate_id!(AttachmentId, "attachment_id");
subordinate_id!(WorkspaceBindingId, "workspace_binding_id");
subordinate_id!(WorkSurfaceId, "work_surface_id");

/// Stable identity of a runtime object subordinate to a Workstream.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RuntimeObjectRef {
    pub runtime_kind: String,
    pub runtime_id: String,
}

impl RuntimeObjectRef {
    pub fn new(
        runtime_kind: impl Into<String>,
        runtime_id: impl Into<String>,
    ) -> Result<Self, ScopeKeyError> {
        let runtime_kind = runtime_kind.into().trim().to_string();
        let runtime_id = runtime_id.into().trim().to_string();
        if runtime_kind.is_empty() {
            return Err(ScopeKeyError::Missing("runtime_kind"));
        }
        if runtime_id.is_empty() {
            return Err(ScopeKeyError::Missing("runtime_id"));
        }
        Ok(Self {
            runtime_kind,
            runtime_id,
        })
    }
}

/// Canonical Workstream map key. Continuity and session identity are deliberately
/// subordinate and therefore cannot participate in this key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WorkstreamKey {
    pub scope: ScopeRef,
    pub workstream_id: WorkstreamId,
}

impl WorkstreamKey {
    pub fn new(scope: ScopeRef, workstream_id: WorkstreamId) -> Self {
        Self {
            scope,
            workstream_id,
        }
    }

    pub fn storage_key(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(Sha256::digest(bytes))
    }
}

/// Exact runtime attachment ownership. Every runtime identifier is subordinate
/// to one durable Workstream and one explicit Desktop workspace binding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttachmentKey {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub instance_id: InstanceId,
    pub session_id: SessionId,
    pub attachment_id: AttachmentId,
    pub workspace_binding_id: WorkspaceBindingId,
}

impl AttachmentKey {
    pub fn new(
        workstream: WorkstreamKey,
        continuity_id: Option<ContinuityId>,
        instance_id: InstanceId,
        session_id: SessionId,
        attachment_id: AttachmentId,
        workspace_binding_id: WorkspaceBindingId,
    ) -> Self {
        Self {
            workstream,
            continuity_id,
            instance_id,
            session_id,
            attachment_id,
            workspace_binding_id,
        }
    }

    pub fn validate_owner(&self, expected: &WorkstreamKey) -> Result<(), ScopeKeyError> {
        if &self.workstream != expected {
            return Err(ScopeKeyError::ScopeMismatch);
        }
        Ok(())
    }

    pub fn storage_key(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(Sha256::digest(bytes))
    }
}

impl ScopeRef {
    pub fn project(scope: LegacyScopeRef) -> Result<Self, ScopeKeyError> {
        scope.validate()?;
        ProjectRootKey::new(scope).map(Self::Project)
    }

    pub fn host(scope: LegacyScopeRef) -> Result<Self, ScopeKeyError> {
        HostScopeKey::new(scope).map(Self::Host)
    }

    pub fn storage_key(&self) -> String {
        match self {
            Self::Project(key) => key.0.storage_key(),
            Self::Host(key) => key.storage_key(),
        }
    }

    pub fn legacy_scope(&self) -> &LegacyScopeRef {
        match self {
            Self::Project(key) => &key.0,
            Self::Host(key) => &key.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(fingerprint: &str) -> LegacyScopeRef {
        LegacyScopeRef::project("project:focusa", "/workspace/focusa", "Focusa", fingerprint)
            .unwrap()
    }

    #[test]
    fn project_scope_requires_validated_project_key() {
        let scope = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        assert!(matches!(scope, ScopeRef::Project(_)));
    }

    #[test]
    fn identical_paths_with_different_host_worktree_fingerprints_are_distinct() {
        let first = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        let second = ScopeRef::project(project("host-b:worktree-main")).unwrap();
        assert_ne!(first, second);
        assert_ne!(first.storage_key(), second.storage_key());
    }

    #[test]
    fn project_scope_cannot_be_constructed_from_host_scope() {
        let host =
            LegacyScopeRef::host("host:build", "/workspace/focusa", "Builder", "host-a").unwrap();
        assert!(matches!(
            ScopeRef::project(host),
            Err(ScopeKeyError::KindMismatch { .. })
        ));
    }

    #[test]
    fn two_workstreams_under_one_project_remain_distinct_keys() {
        let scope = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        let first = WorkstreamKey::new(scope.clone(), WorkstreamId::parse("planning").unwrap());
        let second = WorkstreamKey::new(scope, WorkstreamId::parse("delivery").unwrap());
        assert_ne!(first, second);
        assert_ne!(first.storage_key(), second.storage_key());
    }

    #[test]
    fn continuity_is_not_part_of_serialized_workstream_identity() {
        let scope = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        let key = WorkstreamKey::new(scope, WorkstreamId::parse("delivery").unwrap());
        let encoded = serde_json::to_value(key).unwrap();
        assert_eq!(encoded["workstream_id"], "delivery");
        assert!(encoded.get("continuity_id").is_none());
        assert!(encoded.get("session_id").is_none());
    }

    fn attachment(workstream: WorkstreamKey) -> AttachmentKey {
        AttachmentKey::new(
            workstream,
            Some(ContinuityId::parse("continuity-a").unwrap()),
            InstanceId::parse("instance-a").unwrap(),
            SessionId::parse("session-a").unwrap(),
            AttachmentId::parse("attachment-a").unwrap(),
            WorkspaceBindingId::parse("workspace-a").unwrap(),
        )
    }

    #[test]
    fn attachment_accepts_only_its_owning_workstream() {
        let scope = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        let owner = WorkstreamKey::new(scope.clone(), WorkstreamId::parse("delivery").unwrap());
        let other = WorkstreamKey::new(scope, WorkstreamId::parse("planning").unwrap());
        let key = attachment(owner.clone());
        assert_eq!(key.validate_owner(&owner), Ok(()));
        assert_eq!(
            key.validate_owner(&other),
            Err(ScopeKeyError::ScopeMismatch)
        );
    }

    #[test]
    fn attachment_serializes_the_complete_owner_chain() {
        let scope = ScopeRef::project(project("host-a:worktree-main")).unwrap();
        let key = attachment(WorkstreamKey::new(
            scope,
            WorkstreamId::parse("delivery").unwrap(),
        ));
        let encoded = serde_json::to_value(key).unwrap();
        assert_eq!(encoded["workstream"]["workstream_id"], "delivery");
        assert_eq!(encoded["continuity_id"], "continuity-a");
        assert_eq!(encoded["workspace_binding_id"], "workspace-a");
    }
}
