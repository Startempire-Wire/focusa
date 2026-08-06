//! Canonical Spec 158 scope identity types.
//!
//! This module is the migration target for the legacy flat `scoped_state::ScopeRef`.
//! During dual-read migration it wraps the validated legacy project/host keys without
//! allowing a filesystem path, continuity id, or current-session heuristic to become
//! canonical scope authority.

use crate::scoped_state::{ProjectRootKey, ScopeKeyError, ScopeKind, ScopeRef as LegacyScopeRef};
use serde::{Deserialize, Serialize};

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
}
