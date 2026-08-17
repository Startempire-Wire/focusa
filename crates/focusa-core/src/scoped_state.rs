//! Typed scoped state and deterministic CRDT reconciliation — Spec 104.
//!
//! Canonical state must be partitioned by a verified root scope first and a
//! workstream discriminator second. Continuity never establishes root authority.

use crate::scope_safety::classify_project_root;
use crate::sync::VectorClock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const SCOPED_STATE_SCHEMA_V1: &str = "focusa.scoped_state.v1";
pub const SCOPED_RESULT_SCHEMA_V1: &str = "focusa.scoped_result.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Project,
    Host,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScopeRef {
    pub scope_kind: ScopeKind,
    pub scope_id: String,
    pub root_path: PathBuf,
    pub canonical_name: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProjectRootKey(pub ScopeRef);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WorkstreamKey {
    pub root_scope: ScopeRef,
    pub continuity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttachmentKey {
    pub workstream: WorkstreamKey,
    pub instance_id: String,
    pub session_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScopeKeyError {
    #[error("scope field {0} is required")]
    Missing(&'static str),
    #[error("unsafe project root: {0}")]
    UnsafeProjectRoot(String),
    #[error("scope kind mismatch: expected {expected:?}, found {found:?}")]
    KindMismatch {
        expected: ScopeKind,
        found: ScopeKind,
    },
    #[error("scoped records belong to different workstreams")]
    ScopeMismatch,
    #[error("scoped records have different record ids")]
    RecordMismatch,
}

fn required(value: impl Into<String>, field: &'static str) -> Result<String, ScopeKeyError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        Err(ScopeKeyError::Missing(field))
    } else {
        Ok(value)
    }
}

fn normalized_path(path: impl AsRef<Path>) -> PathBuf {
    let value = path.as_ref().to_string_lossy();
    let normalized = value.trim().trim_end_matches('/');
    PathBuf::from(if normalized.is_empty() {
        "/"
    } else {
        normalized
    })
}

impl ScopeRef {
    pub fn project(
        scope_id: impl Into<String>,
        root_path: impl AsRef<Path>,
        canonical_name: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Result<Self, ScopeKeyError> {
        let root_path = normalized_path(root_path);
        if !classify_project_root(root_path.to_string_lossy().as_ref()).is_safe() {
            return Err(ScopeKeyError::UnsafeProjectRoot(
                root_path.display().to_string(),
            ));
        }
        Ok(Self {
            scope_kind: ScopeKind::Project,
            scope_id: required(scope_id, "scope_id")?,
            root_path,
            canonical_name: required(canonical_name, "canonical_name")?,
            fingerprint: required(fingerprint, "fingerprint")?,
        })
    }

    pub fn host(
        scope_id: impl Into<String>,
        root_path: impl AsRef<Path>,
        canonical_name: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Result<Self, ScopeKeyError> {
        Ok(Self {
            scope_kind: ScopeKind::Host,
            scope_id: required(scope_id, "scope_id")?,
            root_path: normalized_path(root_path),
            canonical_name: required(canonical_name, "canonical_name")?,
            fingerprint: required(fingerprint, "fingerprint")?,
        })
    }

    pub fn validate(&self) -> Result<(), ScopeKeyError> {
        required(self.scope_id.clone(), "scope_id")?;
        required(self.canonical_name.clone(), "canonical_name")?;
        required(self.fingerprint.clone(), "fingerprint")?;
        if self.root_path.as_os_str().is_empty() {
            return Err(ScopeKeyError::Missing("root_path"));
        }
        if self.scope_kind == ScopeKind::Project
            && !classify_project_root(self.root_path.to_string_lossy().as_ref()).is_safe()
        {
            return Err(ScopeKeyError::UnsafeProjectRoot(
                self.root_path.display().to_string(),
            ));
        }
        Ok(())
    }

    pub fn storage_key(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(Sha256::digest(bytes))
    }
}

impl ProjectRootKey {
    pub fn new(scope: ScopeRef) -> Result<Self, ScopeKeyError> {
        if scope.scope_kind != ScopeKind::Project {
            return Err(ScopeKeyError::KindMismatch {
                expected: ScopeKind::Project,
                found: scope.scope_kind,
            });
        }
        Ok(Self(scope))
    }
}

impl WorkstreamKey {
    pub fn new(
        root_scope: ScopeRef,
        continuity_id: impl Into<String>,
    ) -> Result<Self, ScopeKeyError> {
        Ok(Self {
            root_scope,
            continuity_id: required(continuity_id, "continuity_id")?,
        })
    }

    pub fn validate(&self) -> Result<(), ScopeKeyError> {
        self.root_scope.validate()?;
        required(self.continuity_id.clone(), "continuity_id")?;
        Ok(())
    }

    pub fn storage_key(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(Sha256::digest(bytes))
    }
}

impl AttachmentKey {
    pub fn new(
        workstream: WorkstreamKey,
        instance_id: impl Into<String>,
        session_id: impl Into<String>,
        attachment_id: impl Into<String>,
    ) -> Result<Self, ScopeKeyError> {
        Ok(Self {
            workstream,
            instance_id: required(instance_id, "instance_id")?,
            session_id: required(session_id, "session_id")?,
            attachment_id: required(attachment_id, "attachment_id")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopedCrdtRecord<T> {
    pub schema: String,
    pub scope: WorkstreamKey,
    pub record_id: String,
    pub actor_id: String,
    pub vector_clock: VectorClock,
    pub lamport_ts: u64,
    pub updated_at: DateTime<Utc>,
    pub tombstone: bool,
    pub value: T,
}

fn payload_hash<T: Serialize>(value: &T) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(value).unwrap_or_default(),
    ))
}

impl<T> ScopedCrdtRecord<T>
where
    T: Clone + Serialize,
{
    pub fn new(
        scope: WorkstreamKey,
        record_id: impl Into<String>,
        actor_id: impl Into<String>,
        value: T,
    ) -> Result<Self, ScopeKeyError> {
        let actor_id = required(actor_id, "actor_id")?;
        let mut vector_clock = VectorClock::new();
        vector_clock.increment(&actor_id);
        Ok(Self {
            schema: SCOPED_STATE_SCHEMA_V1.to_string(),
            scope,
            record_id: required(record_id, "record_id")?,
            actor_id,
            vector_clock,
            lamport_ts: 1,
            updated_at: Utc::now(),
            tombstone: false,
            value,
        })
    }

    pub fn revise(
        &self,
        actor_id: impl Into<String>,
        value: T,
        tombstone: bool,
    ) -> Result<Self, ScopeKeyError> {
        let actor_id = required(actor_id, "actor_id")?;
        let mut next = self.clone();
        next.actor_id = actor_id.clone();
        next.vector_clock.increment(&actor_id);
        next.lamport_ts = self.lamport_ts.saturating_add(1);
        next.updated_at = Utc::now();
        next.tombstone = tombstone;
        next.value = value;
        Ok(next)
    }

    pub fn reconcile(&self, other: &Self) -> Result<Self, ScopeKeyError> {
        if self.scope != other.scope {
            return Err(ScopeKeyError::ScopeMismatch);
        }
        if self.record_id != other.record_id {
            return Err(ScopeKeyError::RecordMismatch);
        }
        use std::cmp::Ordering;
        let ordering = self.vector_clock.compare(&other.vector_clock);
        let winner = match ordering {
            Some(Ordering::Greater) => self,
            Some(Ordering::Less) => other,
            Some(Ordering::Equal) | None => {
                let left = (
                    self.lamport_ts,
                    self.updated_at,
                    self.actor_id.as_str(),
                    payload_hash(&(self.tombstone, &self.value)),
                );
                let right = (
                    other.lamport_ts,
                    other.updated_at,
                    other.actor_id.as_str(),
                    payload_hash(&(other.tombstone, &other.value)),
                );
                if left >= right { self } else { other }
            }
        };
        let mut merged = winner.clone();
        merged.vector_clock.merge(&self.vector_clock);
        merged.vector_clock.merge(&other.vector_clock);
        merged.lamport_ts = self.lamport_ts.max(other.lamport_ts);
        merged.updated_at = self.updated_at.max(other.updated_at);
        Ok(merged)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopedCrdtMap<T> {
    pub schema: String,
    pub scope: WorkstreamKey,
    pub records: BTreeMap<String, ScopedCrdtRecord<T>>,
}

impl<T> ScopedCrdtMap<T>
where
    T: Clone + Serialize,
{
    pub fn new(scope: WorkstreamKey) -> Self {
        Self {
            schema: SCOPED_STATE_SCHEMA_V1.to_string(),
            scope,
            records: BTreeMap::new(),
        }
    }

    pub fn apply(&mut self, incoming: ScopedCrdtRecord<T>) -> Result<(), ScopeKeyError> {
        if incoming.scope != self.scope {
            return Err(ScopeKeyError::ScopeMismatch);
        }
        let next = match self.records.get(&incoming.record_id) {
            Some(current) => current.reconcile(&incoming)?,
            None => incoming,
        };
        self.records.insert(next.record_id.clone(), next);
        Ok(())
    }

    pub fn reconcile(&mut self, other: &Self) -> Result<(), ScopeKeyError> {
        if self.scope != other.scope {
            return Err(ScopeKeyError::ScopeMismatch);
        }
        for record in other.records.values() {
            self.apply(record.clone())?;
        }
        Ok(())
    }

    pub fn active_records(&self) -> impl Iterator<Item = &ScopedCrdtRecord<T>> {
        self.records.values().filter(|record| !record.tombstone)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStatus {
    Canonical,
    Advisory,
    Blocked,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityEnvelope {
    pub status: AuthorityStatus,
    pub why: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HumanReadableSummary {
    pub status: String,
    pub summary: String,
    pub next_action: String,
    pub why: String,
    pub evidence_refs: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopedResultEnvelope<T> {
    pub schema: String,
    pub scope: WorkstreamKey,
    pub authority: AuthorityEnvelope,
    pub human: HumanReadableSummary,
    #[serde(default)]
    pub human_readable: String,
    pub data: T,
}

impl<T> ScopedResultEnvelope<T> {
    pub fn new(
        scope: WorkstreamKey,
        authority: AuthorityEnvelope,
        human: HumanReadableSummary,
        data: T,
    ) -> Self {
        let authority_label = match authority.status {
            AuthorityStatus::Canonical => "canonical",
            AuthorityStatus::Advisory => "advisory",
            AuthorityStatus::Blocked => "blocked",
            AuthorityStatus::Degraded => "degraded",
        };
        let human_readable = format!(
            "{}: {} Scope: {} · {}. Authority: {}. Next: {}. Why: {}",
            human.status,
            human.summary,
            scope.root_scope.canonical_name,
            scope.continuity_id,
            authority_label,
            human.next_action,
            if human.why.trim().is_empty() {
                &authority.why
            } else {
                &human.why
            }
        );
        Self {
            schema: SCOPED_RESULT_SCHEMA_V1.to_string(),
            scope,
            authority,
            human,
            human_readable,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_scope(name: &str) -> ScopeRef {
        ScopeRef::project(
            format!("project:{name}"),
            format!("/workspace/{name}"),
            name,
            format!("sha256:{name}"),
        )
        .unwrap()
    }

    fn workstream(project: &str, continuity: &str) -> WorkstreamKey {
        WorkstreamKey::new(project_scope(project), continuity).unwrap()
    }

    #[test]
    fn broad_root_is_never_project_scope_but_can_be_explicit_host_scope() {
        assert!(ScopeRef::project("project:root", "/root", "root", "fp").is_err());
        let host = ScopeRef::host("host:one", "/root", "operator-host", "sha256:host").unwrap();
        assert_eq!(host.scope_kind, ScopeKind::Host);
    }

    #[test]
    fn continuity_is_required_but_secondary_to_root_scope() {
        assert!(WorkstreamKey::new(project_scope("a"), "").is_err());
        let a = workstream("a", "same-continuity");
        let b = workstream("b", "same-continuity");
        assert_ne!(a, b);
        assert_ne!(a.storage_key(), b.storage_key());
    }

    #[test]
    fn causal_revision_wins_and_scope_mismatch_blocks() {
        let scope = workstream("a", "cont-a");
        let first = ScopedCrdtRecord::new(scope.clone(), "r1", "agent-a", "one").unwrap();
        let second = first.revise("agent-a", "two", false).unwrap();
        assert_eq!(first.reconcile(&second).unwrap().value, "two");
        let wrong =
            ScopedCrdtRecord::new(workstream("b", "cont-a"), "r1", "agent-b", "bad").unwrap();
        assert_eq!(first.reconcile(&wrong), Err(ScopeKeyError::ScopeMismatch));
    }

    #[test]
    fn concurrent_merge_is_commutative_and_idempotent() {
        let scope = workstream("a", "cont-a");
        let left = ScopedCrdtRecord::new(scope.clone(), "r1", "agent-a", "left").unwrap();
        let right = ScopedCrdtRecord::new(scope, "r1", "agent-b", "right").unwrap();
        let lr = left.reconcile(&right).unwrap();
        let rl = right.reconcile(&left).unwrap();
        assert_eq!(lr, rl);
        assert_eq!(lr.reconcile(&lr).unwrap(), lr);
    }

    #[test]
    fn scoped_map_never_merges_cross_project_records() {
        let mut a = ScopedCrdtMap::new(workstream("a", "cont"));
        let b = ScopedCrdtMap::<String>::new(workstream("b", "cont"));
        assert_eq!(a.reconcile(&b), Err(ScopeKeyError::ScopeMismatch));
    }

    #[test]
    fn result_envelope_preserves_human_and_machine_views() {
        let scope = workstream("a", "cont");
        let envelope = ScopedResultEnvelope::new(
            scope,
            AuthorityEnvelope {
                status: AuthorityStatus::Canonical,
                why: "verified scope".into(),
            },
            HumanReadableSummary {
                status: "completed".into(),
                summary: "Scoped state updated".into(),
                next_action: "Continue".into(),
                why: "CRDT merge accepted".into(),
                evidence_refs: vec!["test:scoped".into()],
                warnings: vec![],
            },
            serde_json::json!({"record_id":"r1"}),
        );
        let json = serde_json::to_value(envelope).unwrap();
        assert_eq!(json["human"]["summary"], "Scoped state updated");
        assert!(
            json["human_readable"]
                .as_str()
                .is_some_and(|text| text.contains("completed: Scoped state updated"))
        );
        assert!(
            json["human_readable"]
                .as_str()
                .is_some_and(|text| text.contains("Scope: a · cont"))
        );
        assert!(
            json["human_readable"]
                .as_str()
                .is_some_and(|text| text.contains("Next: Continue"))
        );
        assert_eq!(json["data"]["record_id"], "r1");
    }
}
