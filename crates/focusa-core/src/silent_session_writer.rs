//! Writer admission and lease fencing for Silent Sessions.

use crate::silent_session::{
    SILENT_SESSION_LEASE_SCHEMA, SilentSession, SilentSessionId, SilentSessionLease,
    SilentSessionLeaseId, WriterMutationMode,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

use crate::license::{
    evaluate_entitlement_execution,
    EntitlementExecutionContext,
    EntitlementExecutionPolicy,
};

const ENTITLEMENT_ROUTE_UNCLASSIFIED: &str = "ENTITLEMENT_ROUTE_UNCLASSIFIED";

pub const WRITER_ADMISSION_SCHEMA: &str = "focusa.writer_admission_decision.v1";
pub const WRITER_LEASE_REGISTRY_SCHEMA: &str = "focusa.writer_lease_registry.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriterActorKind {
    Foreground,
    WorkLoop,
    SilentSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterClaim {
    pub actor_kind: WriterActorKind,
    pub actor_instance_ref: String,
    pub session_id: Option<SilentSessionId>,
    pub project_root: PathBuf,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub workspace_ref: String,
    pub path_intents: Vec<PathBuf>,
    pub mutation_mode: WriterMutationMode,
    pub workspace_dirty: bool,
    pub lease_id: Option<SilentSessionLeaseId>,
    pub fencing_token: Option<u64>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl WriterClaim {
    fn active_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_none_or(|expiry| expiry > now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterAdmissionCandidate {
    pub actor_instance_ref: String,
    pub session_id: SilentSessionId,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub workspace_ref: String,
    pub path_intents: Vec<PathBuf>,
    pub mutation_mode: WriterMutationMode,
    pub workspace_dirty: bool,
    pub explicit_shared_approval_ref: Option<String>,
    pub writer_role: String,
    pub adoption_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriterAdmissionDenial {
    InvalidScope,
    WorkItemConflict,
    WorkspaceConflict,
    DirtySecondWriter,
    PathIntentConflict,
    ExplicitSharedApprovalMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterAdmissionDecision {
    pub schema: String,
    pub admitted: bool,
    pub read_only: bool,
    pub renewal: bool,
    pub denials: Vec<WriterAdmissionDenial>,
    pub conflicting_actor_refs: Vec<String>,
    pub conflicting_lease_ids: Vec<SilentSessionLeaseId>,
    pub isolated_worktree_required: bool,
}

fn writer_entitlement_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.silent_session.writer_admission",
        focusa_license::OperationClass::ValueMutation,
        focusa_license::CapabilityFamily::Automation,
        Some("focusa.agent.silent_sessions"),
        Some("silent_session_admissions"),
        focusa_license::RecoveryAllowance::None,
    )
}

fn evaluate_writer_entitlement(
    entitlement_guard: &focusa_license::LicenseGuard,
) -> Result<(), WriterLeaseError> {
    evaluate_entitlement_execution(
        entitlement_guard,
        &writer_entitlement_policy(),
        EntitlementExecutionContext::default(),
    )
    .map(|_| ())
    .map_err(|error| WriterLeaseError::EntitlementDenied {
        code: error.code,
        message: error.message,
        required_feature: error.required_feature,
        limit_bucket: error.limit_bucket,
    })
}

pub fn analyze_writer_admission(
    candidate: &WriterAdmissionCandidate,
    existing: &[WriterClaim],
    now: DateTime<Utc>,
) -> WriterAdmissionDecision {
    let mut denials = Vec::new();
    let mut conflicting_actor_refs = Vec::new();
    let mut conflicting_lease_ids = Vec::new();
    let read_only = candidate.mutation_mode == WriterMutationMode::ReadOnlyShared;
    let valid_scope = candidate.session_id.is_uuid_v7()
        && candidate.project_root.is_absolute()
        && !candidate.project_identity_ref.trim().is_empty()
        && !candidate.continuity_id.trim().is_empty()
        && !candidate.workspace_ref.trim().is_empty()
        && !candidate.actor_instance_ref.trim().is_empty()
        && !candidate.writer_role.trim().is_empty()
        && safe_path_intents(&candidate.path_intents);
    if !valid_scope {
        denials.push(WriterAdmissionDenial::InvalidScope);
    }
    if candidate.mutation_mode == WriterMutationMode::ExplicitShared
        && candidate
            .explicit_shared_approval_ref
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        denials.push(WriterAdmissionDenial::ExplicitSharedApprovalMissing);
    }

    let mut renewal = false;
    if !read_only {
        for claim in existing.iter().filter(|claim| claim.active_at(now)) {
            if claim.project_root != candidate.project_root
                || claim.continuity_id != candidate.continuity_id
                || claim.mutation_mode == WriterMutationMode::ReadOnlyShared
            {
                continue;
            }
            let same_owner = claim.actor_instance_ref == candidate.actor_instance_ref;
            let same_workspace = claim.workspace_ref == candidate.workspace_ref;
            let same_work_item =
                claim.work_item_ref.is_some() && claim.work_item_ref == candidate.work_item_ref;
            if same_owner && same_workspace && same_work_item {
                renewal = true;
                continue;
            }

            let explicitly_shareable = same_workspace
                && claim.mutation_mode == WriterMutationMode::ExplicitShared
                && candidate.mutation_mode == WriterMutationMode::ExplicitShared
                && candidate.explicit_shared_approval_ref.is_some();
            if same_work_item {
                denials.push(WriterAdmissionDenial::WorkItemConflict);
            }
            if same_workspace && !explicitly_shareable {
                denials.push(WriterAdmissionDenial::WorkspaceConflict);
                if claim.workspace_dirty || candidate.workspace_dirty {
                    denials.push(WriterAdmissionDenial::DirtySecondWriter);
                }
            }
            if explicitly_shareable
                && path_intents_overlap(&claim.path_intents, &candidate.path_intents)
            {
                denials.push(WriterAdmissionDenial::PathIntentConflict);
            }
            if same_work_item || same_workspace {
                conflicting_actor_refs.push(claim.actor_instance_ref.clone());
                if let Some(lease_id) = claim.lease_id {
                    conflicting_lease_ids.push(lease_id);
                }
            }
        }
    }

    denials.sort();
    denials.dedup();
    conflicting_actor_refs.sort();
    conflicting_actor_refs.dedup();
    conflicting_lease_ids.sort();
    conflicting_lease_ids.dedup();
    WriterAdmissionDecision {
        schema: WRITER_ADMISSION_SCHEMA.into(),
        admitted: denials.is_empty(),
        read_only,
        renewal,
        isolated_worktree_required: denials.iter().any(|denial| {
            matches!(
                denial,
                WriterAdmissionDenial::WorkspaceConflict
                    | WriterAdmissionDenial::DirtySecondWriter
                    | WriterAdmissionDenial::PathIntentConflict
            )
        }),
        denials,
        conflicting_actor_refs,
        conflicting_lease_ids,
    }
}

fn safe_path_intents(paths: &[PathBuf]) -> bool {
    paths.iter().all(|path| {
        !path.as_os_str().is_empty()
            && !path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
    })
}

fn path_intents_overlap(left: &[PathBuf], right: &[PathBuf]) -> bool {
    left.iter()
        .any(|left| right.iter().any(|right| path_prefix_overlap(left, right)))
}

fn path_prefix_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterLeaseRegistry {
    pub schema: String,
    pub next_fencing_token: u64,
    pub leases: Vec<SilentSessionLease>,
}

impl Default for WriterLeaseRegistry {
    fn default() -> Self {
        Self {
            schema: WRITER_LEASE_REGISTRY_SCHEMA.into(),
            next_fencing_token: 1,
            leases: Vec::new(),
        }
    }
}

impl WriterLeaseRegistry {
    pub fn validate(&self) -> Result<(), WriterLeaseError> {
        let max_issued = self
            .leases
            .iter()
            .map(|lease| lease.fencing_token)
            .max()
            .unwrap_or(0);
        if self.schema != WRITER_LEASE_REGISTRY_SCHEMA
            || self.next_fencing_token == 0
            || self.next_fencing_token <= max_issued
        {
            return Err(WriterLeaseError::InvalidRegistry);
        }
        Ok(())
    }

    pub fn active_claims(&self, now: DateTime<Utc>) -> Vec<WriterClaim> {
        self.leases
            .iter()
            .filter(|lease| lease.expires_at > now)
            .map(|lease| WriterClaim {
                actor_kind: WriterActorKind::SilentSession,
                actor_instance_ref: lease.owner_actor_instance_ref.clone(),
                session_id: Some(lease.session_id),
                project_root: lease.project_root.clone(),
                continuity_id: lease.continuity_id.clone(),
                work_item_ref: lease.work_item_ref.clone(),
                workspace_ref: lease.workspace_ref.clone(),
                path_intents: lease.path_intents.clone(),
                mutation_mode: lease.mutation_mode,
                workspace_dirty: false,
                lease_id: Some(lease.lease_id),
                fencing_token: Some(lease.fencing_token),
                expires_at: Some(lease.expires_at),
            })
            .collect()
    }

    pub fn acquire_with_entitlement(
        &mut self,
        session: &SilentSession,
        candidate: &WriterAdmissionCandidate,
        external_claims: &[WriterClaim],
        now: DateTime<Utc>,
        ttl: Duration,
        entitlement_guard: &focusa_license::LicenseGuard,
    ) -> Result<SilentSessionLease, WriterLeaseError> {
        evaluate_writer_entitlement(entitlement_guard)?;
        self.acquire(session, candidate, external_claims, now, ttl)
    }

    pub fn acquire(
        &mut self,
        session: &SilentSession,
        candidate: &WriterAdmissionCandidate,
        external_claims: &[WriterClaim],
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> Result<SilentSessionLease, WriterLeaseError> {
        if ttl <= Duration::zero() {
            return Err(WriterLeaseError::InvalidTtl);
        }
        self.leases.retain(|lease| lease.expires_at > now);
        let mut claims = self.active_claims(now);
        claims.extend_from_slice(external_claims);
        let decision = analyze_writer_admission(candidate, &claims, now);
        if !decision.admitted || decision.read_only {
            return Err(WriterLeaseError::AdmissionDenied(decision));
        }
        if candidate.session_id != session.session_id
            || candidate.project_root != session.project_root
            || candidate.project_identity_ref != session.project_identity_ref
            || candidate.continuity_id != session.continuity_id
            || candidate.work_item_ref != session.work_item_ref
        {
            return Err(WriterLeaseError::SessionScopeMismatch);
        }
        let token = self.allocate_token()?;
        let lease = SilentSessionLease {
            schema: SILENT_SESSION_LEASE_SCHEMA.into(),
            lease_id: SilentSessionLeaseId::new(),
            session_id: session.session_id,
            project_root: session.project_root.clone(),
            project_identity_ref: session.project_identity_ref.clone(),
            continuity_id: session.continuity_id.clone(),
            work_item_ref: session.work_item_ref.clone(),
            workspace_ref: candidate.workspace_ref.clone(),
            path_intents: candidate.path_intents.clone(),
            mutation_mode: candidate.mutation_mode,
            writer_role: candidate.writer_role.clone(),
            owner_actor_instance_ref: candidate.actor_instance_ref.clone(),
            fencing_token: token,
            acquired_at: now,
            heartbeat_at: now,
            expires_at: now + ttl,
            adoption_policy: candidate.adoption_policy.clone(),
        };
        lease
            .validate(session)
            .map_err(|_| WriterLeaseError::InvalidLease)?;
        self.leases.push(lease.clone());
        Ok(lease)
    }

    pub fn renew(
        &mut self,
        lease_id: SilentSessionLeaseId,
        owner_actor_instance_ref: &str,
        fencing_token: u64,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> Result<SilentSessionLease, WriterLeaseError> {
        if ttl <= Duration::zero() {
            return Err(WriterLeaseError::InvalidTtl);
        }
        let index = self
            .leases
            .iter()
            .position(|lease| lease.lease_id == lease_id)
            .ok_or(WriterLeaseError::LeaseNotFound)?;
        if self.leases[index].owner_actor_instance_ref != owner_actor_instance_ref
            || self.leases[index].fencing_token != fencing_token
            || self.leases[index].expires_at <= now
        {
            return Err(WriterLeaseError::StaleFencingToken);
        }
        let next_token = self.allocate_token()?;
        let lease = &mut self.leases[index];
        lease.fencing_token = next_token;
        lease.heartbeat_at = now;
        lease.expires_at = now + ttl;
        Ok(lease.clone())
    }

    pub fn release(
        &mut self,
        lease_id: SilentSessionLeaseId,
        owner_actor_instance_ref: &str,
        fencing_token: u64,
    ) -> Result<SilentSessionLease, WriterLeaseError> {
        let index = self
            .leases
            .iter()
            .position(|lease| lease.lease_id == lease_id)
            .ok_or(WriterLeaseError::LeaseNotFound)?;
        if self.leases[index].owner_actor_instance_ref != owner_actor_instance_ref
            || self.leases[index].fencing_token != fencing_token
        {
            return Err(WriterLeaseError::StaleFencingToken);
        }
        Ok(self.leases.remove(index))
    }

    fn allocate_token(&mut self) -> Result<u64, WriterLeaseError> {
        let token = self.next_fencing_token;
        if token == 0 {
            return Err(WriterLeaseError::FencingTokenExhausted);
        }
        self.next_fencing_token = token
            .checked_add(1)
            .ok_or(WriterLeaseError::FencingTokenExhausted)?;
        Ok(token)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WriterLeaseError {
    #[error("writer lease admission was denied")]
    AdmissionDenied(WriterAdmissionDecision),
    #[error("writer lease candidate does not match canonical session scope")]
    SessionScopeMismatch,
    #[error("writer lease TTL must be positive")]
    InvalidTtl,
    #[error("writer lease is invalid")]
    InvalidLease,
    #[error("writer lease registry is invalid")]
    InvalidRegistry,
    #[error("writer lease was not found")]
    LeaseNotFound,
    #[error("writer lease owner or fencing token is stale")]
    StaleFencingToken,
    #[error("writer fencing token source is exhausted")]
    FencingTokenExhausted,
    #[error("writer entitlement was denied ({code}): {message}")]
    EntitlementDenied {
        code: String,
        message: String,
        required_feature: Option<String>,
        limit_bucket: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::*;

    fn session() -> SilentSession {
        let now = Utc::now();
        SilentSession {
            schema: SILENT_SESSION_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            session_id: SilentSessionId::new(),
            display_name: "writer-test".into(),
            created_at: now,
            created_by_actor_ref: "operator:test".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "runner".into(),
            project_root: PathBuf::from("/projects/focusa"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "workloop-completion".into(),
            trajectory_ref: None,
            workpoint_ref: None,
            work_item_ref: Some("focusa-a6yq6.6.1".into()),
            operator_ask: crate::silent_session::OperatorAskBinding::capture(
                "ask:writer-test",
                "prove writer admission",
                1,
                Utc::now(),
            ),
            mission: "prove writer admission".into(),
            lifecycle_state: SilentSessionLifecycleState::Running,
            health: SilentSessionHealth::Healthy,
            semantic_observation: None,
            active_run_id: None,
            config_revision_id: SilentSessionConfigRevisionId::new(),
            writer_lease_ref: None,
            retention_policy_ref: "retention:test".into(),
            receipt_refs: vec![],
        }
    }

    fn candidate(session: &SilentSession) -> WriterAdmissionCandidate {
        WriterAdmissionCandidate {
            actor_instance_ref: "actor:owner".into(),
            session_id: session.session_id,
            project_root: session.project_root.clone(),
            project_identity_ref: session.project_identity_ref.clone(),
            continuity_id: session.continuity_id.clone(),
            work_item_ref: session.work_item_ref.clone(),
            workspace_ref: "workspace:primary".into(),
            path_intents: vec![PathBuf::from("crates/focusa-core")],
            mutation_mode: WriterMutationMode::ExclusiveExisting,
            workspace_dirty: true,
            explicit_shared_approval_ref: None,
            writer_role: "primary".into(),
            adoption_policy: "signed_match_only".into(),
        }
    }

    #[test]
    fn dirty_workspace_allows_sole_owner_renewal_but_blocks_second_writer() {
        let now = Utc::now();
        let session = session();
        let owner = candidate(&session);
        let claim = WriterClaim {
            actor_kind: WriterActorKind::Foreground,
            actor_instance_ref: owner.actor_instance_ref.clone(),
            session_id: None,
            project_root: owner.project_root.clone(),
            continuity_id: owner.continuity_id.clone(),
            work_item_ref: owner.work_item_ref.clone(),
            workspace_ref: owner.workspace_ref.clone(),
            path_intents: owner.path_intents.clone(),
            mutation_mode: owner.mutation_mode,
            workspace_dirty: true,
            lease_id: None,
            fencing_token: Some(4),
            expires_at: None,
        };
        let renewal = analyze_writer_admission(&owner, std::slice::from_ref(&claim), now);
        assert!(renewal.admitted && renewal.renewal);

        let mut second = owner;
        second.actor_instance_ref = "actor:second".into();
        let denied = analyze_writer_admission(&second, &[claim], now);
        assert!(!denied.admitted);
        assert!(
            denied
                .denials
                .contains(&WriterAdmissionDenial::DirtySecondWriter)
        );
        assert!(denied.isolated_worktree_required);
    }

    #[test]
    fn isolated_or_read_only_workspaces_avoid_false_conflicts_and_shared_paths_are_fenced() {
        let now = Utc::now();
        let session = session();
        let owner = candidate(&session);
        let claim = WriterClaim {
            actor_kind: WriterActorKind::WorkLoop,
            actor_instance_ref: "actor:workloop".into(),
            session_id: None,
            project_root: owner.project_root.clone(),
            continuity_id: owner.continuity_id.clone(),
            work_item_ref: Some("other-item".into()),
            workspace_ref: "workspace:primary".into(),
            path_intents: vec![PathBuf::from("crates/focusa-core")],
            mutation_mode: WriterMutationMode::ExplicitShared,
            workspace_dirty: false,
            lease_id: None,
            fencing_token: Some(7),
            expires_at: None,
        };

        let mut isolated = owner.clone();
        isolated.actor_instance_ref = "actor:isolated".into();
        isolated.work_item_ref = Some("isolated-item".into());
        isolated.workspace_ref = "workspace:isolated".into();
        isolated.mutation_mode = WriterMutationMode::IsolatedWorktree;
        isolated.workspace_dirty = false;
        assert!(analyze_writer_admission(&isolated, std::slice::from_ref(&claim), now).admitted);

        let mut read_only = isolated.clone();
        read_only.mutation_mode = WriterMutationMode::ReadOnlyShared;
        read_only.workspace_ref = claim.workspace_ref.clone();
        assert!(analyze_writer_admission(&read_only, std::slice::from_ref(&claim), now).admitted);

        let mut shared = isolated;
        shared.workspace_ref = claim.workspace_ref.clone();
        shared.mutation_mode = WriterMutationMode::ExplicitShared;
        shared.explicit_shared_approval_ref = Some("approval:shared".into());
        let denied = analyze_writer_admission(&shared, &[claim], now);
        assert!(
            denied
                .denials
                .contains(&WriterAdmissionDenial::PathIntentConflict)
        );
    }

    #[test]
    fn lease_registry_issues_monotonic_tokens_and_fences_stale_renew_release() {
        let now = Utc::now();
        let session = session();
        let candidate = candidate(&session);
        let mut registry = WriterLeaseRegistry::default();
        let lease = registry
            .acquire(&session, &candidate, &[], now, Duration::seconds(30))
            .unwrap();
        assert_eq!(lease.fencing_token, 1);
        assert!(matches!(
            registry.renew(
                lease.lease_id,
                &lease.owner_actor_instance_ref,
                999,
                now,
                Duration::seconds(30)
            ),
            Err(WriterLeaseError::StaleFencingToken)
        ));
        let renewed = registry
            .renew(
                lease.lease_id,
                &lease.owner_actor_instance_ref,
                lease.fencing_token,
                now + Duration::seconds(1),
                Duration::seconds(30),
            )
            .unwrap();
        assert_eq!(renewed.fencing_token, 2);
        assert!(matches!(
            registry.release(
                renewed.lease_id,
                &renewed.owner_actor_instance_ref,
                lease.fencing_token
            ),
            Err(WriterLeaseError::StaleFencingToken)
        ));
        registry
            .release(
                renewed.lease_id,
                &renewed.owner_actor_instance_ref,
                renewed.fencing_token,
            )
            .unwrap();
        assert!(registry.leases.is_empty());

        let expired = registry
            .acquire(&session, &candidate, &[], now, Duration::seconds(1))
            .unwrap();
        let mut replacement = candidate;
        replacement.actor_instance_ref = "actor:replacement".into();
        let replacement = registry
            .acquire(
                &session,
                &replacement,
                &[],
                now + Duration::seconds(2),
                Duration::seconds(30),
            )
            .expect("expired lease must not block a new exact writer");
        assert!(replacement.fencing_token > expired.fencing_token);
        assert_eq!(registry.leases.len(), 1);
    }
}
