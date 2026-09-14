//! Verified AgentBootstrap and project-mutation barrier for daemon-native
//! Silent Sessions.
//!
//! Bootstrap composes existing ProjectIdentity, Trajectory, Workpoint, and
//! Context Cognition authority; it does not create a new authority source.
//! A project mutation grant is minted only while the bootstrap receipt, writer
//! lease, Context Authority verdict, and runtime-observed model all match the
//! exact session run and remain fresh.

use crate::silent_session::{
    ModelBinding, OperatorAskBinding, SILENT_SESSION_LEASE_SCHEMA, SilentSessionId,
    SilentSessionLease, SilentSessionRunId, WorkpointBinding,
};
use crate::silent_session_authorization::{ContextAuthorityActionClass, ContextAuthorityGrant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::{Uuid, Version};

pub const AGENT_BOOTSTRAP_PACKET_SCHEMA: &str = "focusa.agent_bootstrap_packet.v1";
pub const AGENT_BOOTSTRAP_VERIFICATION_SCHEMA: &str = "focusa.agent_bootstrap_verification.v1";

/// Hash-bound ProjectIdentity snapshot. The canonical identity remains the
/// ProjectIdentity record referenced by `project_identity_ref`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIdentityBootstrapBinding {
    pub project_identity_ref: String,
    pub project_root: PathBuf,
    pub fingerprint: String,
    pub snapshot_ref: String,
    pub snapshot_sha256: String,
    pub verified_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrajectoryBootstrapStatus {
    CanonicalAdvisory,
    GenericDegraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrajectoryBootstrapBinding {
    pub trajectory_ref: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub snapshot_ref: String,
    pub snapshot_sha256: String,
    pub generated_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub status: TrajectoryBootstrapStatus,
    pub waypoints: Vec<String>,
    pub active_gap: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointBootstrapBinding {
    pub workpoint_ref: WorkpointBinding,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub snapshot_ref: String,
    pub snapshot_sha256: String,
    pub generated_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
}

/// A bounded Context Cognition projection. Its authority flags are explicit so
/// selected context can never silently become project-mutation authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBootstrapBinding {
    pub context_packet_ref: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub trajectory_ref: String,
    pub workpoint_ref: WorkpointBinding,
    pub source_snapshot_ref: String,
    pub packet_sha256: String,
    pub generated_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub advisory: bool,
    pub canonical: bool,
    pub canonical_mutation_allowed: bool,
    pub selected_context: Vec<String>,
    pub excluded_context: Vec<String>,
    pub risk_refs: Vec<String>,
    pub valid_next_tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionOntologyBindings {
    pub agent_identity_ref: String,
    pub actor_instance_ref: String,
    pub role_profile_ref: String,
    pub capability_profile_ref: String,
    pub permission_profile_ref: String,
    pub responsibility_ref: String,
    pub handoff_boundary_ref: String,
    pub execution_context_ref: String,
    pub tool_surface_ref: String,
    pub affordance_ref: String,
    pub resource_ref: String,
    pub cost_model_ref: String,
    pub reliability_profile_ref: String,
    pub reversibility_profile_ref: String,
    pub work_item_ref: String,
    pub action_intent_ref: String,
    pub blocker_ref: String,
    pub verification_record_ref: String,
    pub evidence_artifact_ref: String,
}

impl SilentSessionOntologyBindings {
    fn validate(&self) -> Result<(), AgentBootstrapBarrierError> {
        let refs = [
            self.agent_identity_ref.as_str(),
            self.actor_instance_ref.as_str(),
            self.role_profile_ref.as_str(),
            self.capability_profile_ref.as_str(),
            self.permission_profile_ref.as_str(),
            self.responsibility_ref.as_str(),
            self.handoff_boundary_ref.as_str(),
            self.execution_context_ref.as_str(),
            self.tool_surface_ref.as_str(),
            self.affordance_ref.as_str(),
            self.resource_ref.as_str(),
            self.cost_model_ref.as_str(),
            self.reliability_profile_ref.as_str(),
            self.reversibility_profile_ref.as_str(),
            self.work_item_ref.as_str(),
            self.action_intent_ref.as_str(),
            self.blocker_ref.as_str(),
            self.verification_record_ref.as_str(),
            self.evidence_artifact_ref.as_str(),
        ];
        if refs.iter().any(|value| value.trim().is_empty()) {
            return Err(AgentBootstrapBarrierError::MissingField("ontology_ref"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapWorkspaceBinding {
    pub workspace_ref: String,
    pub workspace_root: PathBuf,
}

/// One exact, bounded startup packet for a single Silent Session run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBootstrapPacket {
    pub schema: String,
    pub packet_id: Uuid,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub generated_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub project_identity: ProjectIdentityBootstrapBinding,
    pub continuity_id: String,
    pub trajectory: TrajectoryBootstrapBinding,
    pub workpoint: WorkpointBootstrapBinding,
    pub operator_ask: OperatorAskBinding,
    pub context: ContextBootstrapBinding,
    pub ontology: SilentSessionOntologyBindings,
    pub work_item_ref: Option<String>,
    pub workspace: BootstrapWorkspaceBinding,
    pub model: ModelBinding,
    pub role_ref: String,
    pub mission: String,
    pub exact_next_action: String,
    pub active_object_refs: Vec<String>,
    pub hook_refs: Vec<String>,
    pub blockers: Vec<String>,
    pub do_not_drift: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub proof_gaps: Vec<String>,
    pub completion_expectations: Vec<String>,
}

impl AgentBootstrapPacket {
    pub fn sha256(&self) -> Result<String, AgentBootstrapBarrierError> {
        let bytes = serde_json::to_vec(self)
            .map_err(|_| AgentBootstrapBarrierError::SerializationFailed)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), AgentBootstrapBarrierError> {
        self.operator_ask
            .validate()
            .map_err(|_| AgentBootstrapBarrierError::ScopeMismatch("operator_ask"))?;
        self.ontology.validate()?;
        if self.trajectory.status == TrajectoryBootstrapStatus::GenericDegraded {
            return Err(AgentBootstrapBarrierError::GenericTrajectoryBlocked);
        }
        if self.trajectory.waypoints.is_empty()
            || self
                .trajectory
                .waypoints
                .iter()
                .any(|value| value.trim().is_empty())
            || self.trajectory.active_gap.trim().is_empty()
        {
            return Err(AgentBootstrapBarrierError::MissingField(
                "trajectory_waypoints_or_gap",
            ));
        }
        if self.schema != AGENT_BOOTSTRAP_PACKET_SCHEMA {
            return Err(AgentBootstrapBarrierError::UnsupportedSchema("packet"));
        }
        if self.packet_id.get_version() != Some(Version::SortRand)
            || !self.session_id.is_uuid_v7()
            || !self.run_id.is_uuid_v7()
            || self.generation == 0
        {
            return Err(AgentBootstrapBarrierError::InvalidIdentity);
        }
        if !safe_absolute_root(&self.project_identity.project_root)
            || !safe_absolute_root(&self.workspace.workspace_root)
        {
            return Err(AgentBootstrapBarrierError::UnsafeRoot);
        }
        require_fields(&[
            (
                "project_identity_ref",
                &self.project_identity.project_identity_ref,
            ),
            ("project_fingerprint", &self.project_identity.fingerprint),
            ("project_snapshot_ref", &self.project_identity.snapshot_ref),
            ("continuity_id", &self.continuity_id),
            ("trajectory_ref", &self.trajectory.trajectory_ref),
            ("trajectory_snapshot_ref", &self.trajectory.snapshot_ref),
            ("workpoint_id", &self.workpoint.workpoint_ref.workpoint_id),
            ("workpoint_snapshot_ref", &self.workpoint.snapshot_ref),
            ("context_packet_ref", &self.context.context_packet_ref),
            ("context_snapshot_ref", &self.context.source_snapshot_ref),
            ("workspace_ref", &self.workspace.workspace_ref),
            ("model_provider", &self.model.provider),
            ("model", &self.model.model),
            ("role_ref", &self.role_ref),
            ("mission", &self.mission),
            ("exact_next_action", &self.exact_next_action),
        ])?;
        for (name, hash) in [
            (
                "project_identity",
                self.project_identity.snapshot_sha256.as_str(),
            ),
            ("trajectory", self.trajectory.snapshot_sha256.as_str()),
            ("workpoint", self.workpoint.snapshot_sha256.as_str()),
            ("context", self.context.packet_sha256.as_str()),
        ] {
            if !valid_sha256(hash) {
                return Err(AgentBootstrapBarrierError::InvalidSnapshotHash(name));
            }
        }
        if self.trajectory.project_identity_ref != self.project_identity.project_identity_ref
            || self.workpoint.project_identity_ref != self.project_identity.project_identity_ref
            || self.context.project_identity_ref != self.project_identity.project_identity_ref
        {
            return Err(AgentBootstrapBarrierError::ScopeMismatch(
                "project_identity",
            ));
        }
        if self.trajectory.continuity_id != self.continuity_id
            || self.workpoint.continuity_id != self.continuity_id
            || self.context.continuity_id != self.continuity_id
        {
            return Err(AgentBootstrapBarrierError::ScopeMismatch("continuity"));
        }
        if self.context.trajectory_ref != self.trajectory.trajectory_ref {
            return Err(AgentBootstrapBarrierError::ScopeMismatch("trajectory"));
        }
        if self.context.workpoint_ref != self.workpoint.workpoint_ref {
            return Err(AgentBootstrapBarrierError::ScopeMismatch("workpoint"));
        }
        if !self.context.advisory
            || self.context.canonical
            || self.context.canonical_mutation_allowed
        {
            return Err(AgentBootstrapBarrierError::ContextAuthorityEscalation);
        }
        if self.context.selected_context.is_empty() {
            return Err(AgentBootstrapBarrierError::MissingField("selected_context"));
        }
        if self.do_not_drift.is_empty() {
            return Err(AgentBootstrapBarrierError::MissingField("do_not_drift"));
        }
        if self.evidence_refs.is_empty() && self.proof_gaps.is_empty() {
            return Err(AgentBootstrapBarrierError::MissingField(
                "evidence_refs_or_proof_gaps",
            ));
        }
        if self.completion_expectations.is_empty() {
            return Err(AgentBootstrapBarrierError::MissingField(
                "completion_expectations",
            ));
        }
        if self.generated_at > now
            || self.project_identity.verified_at > now
            || self.trajectory.generated_at > now
            || self.workpoint.generated_at > now
            || self.context.generated_at > now
        {
            return Err(AgentBootstrapBarrierError::SourceNotYetValid);
        }
        for (name, fresh_until) in [
            ("packet", self.fresh_until),
            ("project_identity", self.project_identity.fresh_until),
            ("trajectory", self.trajectory.fresh_until),
            ("workpoint", self.workpoint.fresh_until),
            ("context", self.context.fresh_until),
        ] {
            if fresh_until <= now || self.fresh_until > fresh_until {
                return Err(AgentBootstrapBarrierError::StaleSource(name));
            }
        }
        Ok(())
    }
}

/// Hash and exact source bindings produced only after packet verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBootstrapVerification {
    pub schema: String,
    pub verification_id: Uuid,
    pub packet_id: Uuid,
    pub packet_sha256: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_identity_ref: String,
    pub trajectory_ref: String,
    pub workpoint_ref: WorkpointBinding,
    pub operator_ask_ref: String,
    pub operator_ask_sha256: String,
    pub operator_ask_revision: u64,
    pub context_packet_ref: String,
    pub verified_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
}

impl AgentBootstrapVerification {
    pub fn verify_for(
        &self,
        packet: &AgentBootstrapPacket,
        now: DateTime<Utc>,
    ) -> Result<(), AgentBootstrapBarrierError> {
        packet.validate(now)?;
        if self.schema != AGENT_BOOTSTRAP_VERIFICATION_SCHEMA {
            return Err(AgentBootstrapBarrierError::UnsupportedSchema(
                "verification",
            ));
        }
        if self.verification_id.get_version() != Some(Version::SortRand)
            || self.verified_at > now
            || self.fresh_until <= now
            || self.fresh_until > packet.fresh_until
            || self.packet_id != packet.packet_id
            || self.packet_sha256 != packet.sha256()?
            || self.session_id != packet.session_id
            || self.run_id != packet.run_id
            || self.generation != packet.generation
            || self.project_identity_ref != packet.project_identity.project_identity_ref
            || self.trajectory_ref != packet.trajectory.trajectory_ref
            || self.workpoint_ref != packet.workpoint.workpoint_ref
            || self.operator_ask_ref != packet.operator_ask.ask_ref
            || self.operator_ask_sha256 != packet.operator_ask.text_sha256
            || self.operator_ask_revision != packet.operator_ask.revision
            || self.context_packet_ref != packet.context.context_packet_ref
        {
            return Err(AgentBootstrapBarrierError::VerificationMismatch);
        }
        Ok(())
    }
}

pub fn verify_agent_bootstrap_packet(
    packet: &AgentBootstrapPacket,
    now: DateTime<Utc>,
) -> Result<AgentBootstrapVerification, AgentBootstrapBarrierError> {
    packet.validate(now)?;
    Ok(AgentBootstrapVerification {
        schema: AGENT_BOOTSTRAP_VERIFICATION_SCHEMA.into(),
        verification_id: Uuid::now_v7(),
        packet_id: packet.packet_id,
        packet_sha256: packet.sha256()?,
        session_id: packet.session_id,
        run_id: packet.run_id,
        generation: packet.generation,
        project_identity_ref: packet.project_identity.project_identity_ref.clone(),
        trajectory_ref: packet.trajectory.trajectory_ref.clone(),
        workpoint_ref: packet.workpoint.workpoint_ref.clone(),
        operator_ask_ref: packet.operator_ask.ask_ref.clone(),
        operator_ask_sha256: packet.operator_ask.text_sha256.clone(),
        operator_ask_revision: packet.operator_ask.revision,
        context_packet_ref: packet.context.context_packet_ref.clone(),
        verified_at: now,
        fresh_until: packet.fresh_until,
    })
}

#[derive(Serialize)]
struct ContextAuthorityActionDigest<'a> {
    bootstrap_packet_sha256: String,
    action_class: ContextAuthorityActionClass,
    action: &'a str,
}

pub fn context_authority_action_digest(
    packet: &AgentBootstrapPacket,
    action_class: ContextAuthorityActionClass,
    action: &str,
) -> Result<String, AgentBootstrapBarrierError> {
    if action.trim().is_empty() {
        return Err(AgentBootstrapBarrierError::ContextAuthorityDenied);
    }
    let material = ContextAuthorityActionDigest {
        bootstrap_packet_sha256: packet.sha256()?,
        action_class,
        action,
    };
    let bytes = serde_json::to_vec(&material)
        .map_err(|_| AgentBootstrapBarrierError::SerializationFailed)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn verify_context_authority_for_action(
    packet: &AgentBootstrapPacket,
    authority: &ContextAuthorityGrant,
    action_class: ContextAuthorityActionClass,
    action: &str,
    now: DateTime<Utc>,
) -> Result<(), AgentBootstrapBarrierError> {
    let packet_workpoint_ref = packet.workpoint.workpoint_ref.workpoint_id.as_str();
    let expected_digest = context_authority_action_digest(packet, action_class, action)?;
    if !authority.allowed
        || authority.verdict_ref.trim().is_empty()
        || authority.action_class != action_class
        || authority.action != action
        || authority.action_digest != expected_digest
        || authority.issued_at > now
        || authority.expires_at <= now
        || authority.expires_at - authority.issued_at > chrono::Duration::minutes(5)
        || authority.project_identity_ref != packet.project_identity.project_identity_ref
        || authority.continuity_id != packet.continuity_id
        || authority.workpoint_ref.as_deref() != Some(packet_workpoint_ref)
    {
        return Err(AgentBootstrapBarrierError::ContextAuthorityDenied);
    }
    Ok(())
}

pub struct MajorTransitionBarrierRequest<'a> {
    pub packet: &'a AgentBootstrapPacket,
    pub bootstrap_verification: &'a AgentBootstrapVerification,
    pub context_authority: &'a ContextAuthorityGrant,
    pub action_class: ContextAuthorityActionClass,
    pub action: &'a str,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMajorTransitionGrant {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub action_class: ContextAuthorityActionClass,
    pub action: String,
    pub action_digest: String,
    pub context_authority_ref: String,
    pub valid_until: DateTime<Utc>,
}

pub fn verify_major_transition_barrier(
    request: &MajorTransitionBarrierRequest<'_>,
) -> Result<VerifiedMajorTransitionGrant, AgentBootstrapBarrierError> {
    request
        .bootstrap_verification
        .verify_for(request.packet, request.now)?;
    verify_context_authority_for_action(
        request.packet,
        request.context_authority,
        request.action_class,
        request.action,
        request.now,
    )?;
    Ok(VerifiedMajorTransitionGrant {
        session_id: request.packet.session_id,
        run_id: request.packet.run_id,
        generation: request.packet.generation,
        action_class: request.action_class,
        action: request.action.to_owned(),
        action_digest: request.context_authority.action_digest.clone(),
        context_authority_ref: request.context_authority.verdict_ref.clone(),
        valid_until: [
            request.packet.fresh_until,
            request.bootstrap_verification.fresh_until,
            request.context_authority.expires_at,
        ]
        .into_iter()
        .min()
        .expect("major transition has three expiry bounds"),
    })
}

pub struct ProjectMutationBarrierRequest<'a> {
    pub packet: &'a AgentBootstrapPacket,
    pub bootstrap_verification: &'a AgentBootstrapVerification,
    pub lease: &'a SilentSessionLease,
    pub context_authority: &'a ContextAuthorityGrant,
    pub action_class: ContextAuthorityActionClass,
    pub action: &'a str,
    pub actor_instance_ref: &'a str,
    pub requested_model: &'a ModelBinding,
    pub effective_model: Option<&'a ModelBinding>,
    pub observed_model: Option<&'a ModelBinding>,
    pub now: DateTime<Utc>,
}

/// Non-serializable type-state token required by runner project-mutation APIs.
/// Callers cannot reconstruct it from a stale persisted boolean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProjectMutationGrant {
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: u64,
    project_root: PathBuf,
    project_identity_ref: String,
    continuity_id: String,
    workspace_ref: String,
    workspace_root: PathBuf,
    bootstrap_packet_sha256: String,
    context_authority_ref: String,
    model: ModelBinding,
    lease_fencing_token: u64,
    verified_at: DateTime<Utc>,
    valid_until: DateTime<Utc>,
}

impl VerifiedProjectMutationGrant {
    pub fn session_id(&self) -> SilentSessionId {
        self.session_id
    }

    pub fn run_id(&self) -> SilentSessionRunId {
        self.run_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn bootstrap_packet_sha256(&self) -> &str {
        &self.bootstrap_packet_sha256
    }

    pub fn context_authority_ref(&self) -> &str {
        &self.context_authority_ref
    }

    pub fn model(&self) -> &ModelBinding {
        &self.model
    }

    pub fn lease_fencing_token(&self) -> u64 {
        self.lease_fencing_token
    }

    pub fn verified_at(&self) -> DateTime<Utc> {
        self.verified_at
    }

    pub fn valid_until(&self) -> DateTime<Utc> {
        self.valid_until
    }

    /// Re-check the exact execution scope and freshness at the last responsible
    /// moment before an OS-backed project mutation.
    pub fn verify_execution_scope(
        &self,
        project_root: &Path,
        project_identity_ref: &str,
        workspace_root: &Path,
        now: DateTime<Utc>,
    ) -> Result<(), AgentBootstrapBarrierError> {
        if now >= self.valid_until {
            return Err(AgentBootstrapBarrierError::GrantExpired);
        }
        if project_root != self.project_root
            || project_identity_ref != self.project_identity_ref
            || workspace_root != self.workspace_root
        {
            return Err(AgentBootstrapBarrierError::ScopeMismatch("execution_scope"));
        }
        Ok(())
    }
}

pub fn verify_project_mutation_barrier(
    request: &ProjectMutationBarrierRequest<'_>,
) -> Result<VerifiedProjectMutationGrant, AgentBootstrapBarrierError> {
    let packet = request.packet;
    request
        .bootstrap_verification
        .verify_for(packet, request.now)?;

    let lease = request.lease;
    if lease.schema != SILENT_SESSION_LEASE_SCHEMA
        || !lease.lease_id.is_uuid_v7()
        || lease.session_id != packet.session_id
        || lease.project_root != packet.project_identity.project_root
        || lease.project_identity_ref != packet.project_identity.project_identity_ref
        || lease.continuity_id != packet.continuity_id
        || lease.work_item_ref != packet.work_item_ref
        || lease.workspace_ref != packet.workspace.workspace_ref
        || lease.owner_actor_instance_ref != request.actor_instance_ref
        || lease.writer_role.trim().is_empty()
        || lease.fencing_token == 0
        || lease.acquired_at > lease.heartbeat_at
        || lease.heartbeat_at > request.now
        || lease.expires_at <= request.now
    {
        return Err(AgentBootstrapBarrierError::LeaseDenied);
    }

    let authority = request.context_authority;
    verify_context_authority_for_action(
        packet,
        authority,
        request.action_class,
        request.action,
        request.now,
    )?;

    let Some(effective_model) = request.effective_model else {
        return Err(AgentBootstrapBarrierError::ModelNotVerified);
    };
    let Some(observed_model) = request.observed_model else {
        return Err(AgentBootstrapBarrierError::ModelNotVerified);
    };
    if request.requested_model != &packet.model
        || effective_model != &packet.model
        || observed_model != &packet.model
    {
        return Err(AgentBootstrapBarrierError::ModelNotVerified);
    }

    let valid_until = [
        packet.fresh_until,
        request.bootstrap_verification.fresh_until,
        lease.expires_at,
        authority.expires_at,
    ]
    .into_iter()
    .min()
    .expect("mutation barrier has four expiry bounds");

    Ok(VerifiedProjectMutationGrant {
        session_id: packet.session_id,
        run_id: packet.run_id,
        generation: packet.generation,
        project_root: packet.project_identity.project_root.clone(),
        project_identity_ref: packet.project_identity.project_identity_ref.clone(),
        continuity_id: packet.continuity_id.clone(),
        workspace_ref: packet.workspace.workspace_ref.clone(),
        workspace_root: packet.workspace.workspace_root.clone(),
        bootstrap_packet_sha256: request.bootstrap_verification.packet_sha256.clone(),
        context_authority_ref: authority.verdict_ref.clone(),
        model: packet.model.clone(),
        lease_fencing_token: lease.fencing_token,
        verified_at: request.now,
        valid_until,
    })
}

fn require_fields(fields: &[(&'static str, &str)]) -> Result<(), AgentBootstrapBarrierError> {
    fields
        .iter()
        .find(|(_, value)| value.trim().is_empty())
        .map_or(Ok(()), |(name, _)| {
            Err(AgentBootstrapBarrierError::MissingField(name))
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_absolute_root(path: &Path) -> bool {
    path.is_absolute() && path.parent().is_some_and(|parent| parent != path)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AgentBootstrapBarrierError {
    #[error("unsupported AgentBootstrap {0} schema")]
    UnsupportedSchema(&'static str),
    #[error("AgentBootstrap identity must use UUIDv7 and a positive generation")]
    InvalidIdentity,
    #[error("AgentBootstrap project and workspace roots must be safe absolute paths")]
    UnsafeRoot,
    #[error("AgentBootstrap is missing required field {0}")]
    MissingField(&'static str),
    #[error("AgentBootstrap {0} snapshot hash is invalid")]
    InvalidSnapshotHash(&'static str),
    #[error("AgentBootstrap scope mismatch: {0}")]
    ScopeMismatch(&'static str),
    #[error("generic trajectory is degraded context and cannot launch canonical work")]
    GenericTrajectoryBlocked,
    #[error("Context Cognition attempted to become mutation authority")]
    ContextAuthorityEscalation,
    #[error("AgentBootstrap source timestamp is in the future")]
    SourceNotYetValid,
    #[error("AgentBootstrap source is stale: {0}")]
    StaleSource(&'static str),
    #[error("AgentBootstrap serialization failed")]
    SerializationFailed,
    #[error("AgentBootstrap verification does not match the exact packet")]
    VerificationMismatch,
    #[error("fresh exact writer lease is required")]
    LeaseDenied,
    #[error("fresh exact Context Authority verdict is required")]
    ContextAuthorityDenied,
    #[error("requested, effective, and runtime-observed model must match bootstrap")]
    ModelNotVerified,
    #[error("project mutation grant expired")]
    GrantExpired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::SilentSessionLeaseId;
    use chrono::{Duration, TimeZone};

    #[derive(Clone)]
    struct Fixture {
        packet: AgentBootstrapPacket,
        verification: AgentBootstrapVerification,
        lease: SilentSessionLease,
        authority: ContextAuthorityGrant,
        requested: ModelBinding,
        effective: Option<ModelBinding>,
        observed: Option<ModelBinding>,
        now: DateTime<Utc>,
        actor_instance_ref: String,
    }

    impl Fixture {
        fn request(&self) -> ProjectMutationBarrierRequest<'_> {
            ProjectMutationBarrierRequest {
                packet: &self.packet,
                bootstrap_verification: &self.verification,
                lease: &self.lease,
                context_authority: &self.authority,
                action_class: ContextAuthorityActionClass::GeneratedCodeOverwrite,
                action: "project_mutation",
                actor_instance_ref: &self.actor_instance_ref,
                requested_model: &self.requested,
                effective_model: self.effective.as_ref(),
                observed_model: self.observed.as_ref(),
                now: self.now,
            }
        }
    }

    fn ontology() -> SilentSessionOntologyBindings {
        SilentSessionOntologyBindings {
            agent_identity_ref: "agent:spec133".into(),
            actor_instance_ref: "actor-instance:spec133".into(),
            role_profile_ref: "role:implementer".into(),
            capability_profile_ref: "capability:rust".into(),
            permission_profile_ref: "permission:isolated-worktree".into(),
            responsibility_ref: "responsibility:work-item".into(),
            handoff_boundary_ref: "handoff:operator".into(),
            execution_context_ref: "execution-context:spec133".into(),
            tool_surface_ref: "tool-surface:runner".into(),
            affordance_ref: "affordance:project-mutation".into(),
            resource_ref: "resource:local-runner".into(),
            cost_model_ref: "cost-model:test".into(),
            reliability_profile_ref: "reliability:strict".into(),
            reversibility_profile_ref: "reversibility:worktree".into(),
            work_item_ref: "focusa-a6yq6.4.4".into(),
            action_intent_ref: "action-intent:guarded-mutation".into(),
            blocker_ref: "blocker:none".into(),
            verification_record_ref: "verification:bootstrap".into(),
            evidence_artifact_ref: "evidence:bootstrap".into(),
        }
    }

    fn fixture() -> Fixture {
        let now = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).unwrap();
        let fresh_until = now + Duration::minutes(5);
        let project_identity_ref = "project:focusa:verified".to_owned();
        let continuity_id = "continuity:spec133".to_owned();
        let trajectory_ref = "trajectory:133".to_owned();
        let workpoint_ref = WorkpointBinding {
            workpoint_id: "workpoint:133-3.4".into(),
            revision: Some(7),
        };
        let model = ModelBinding {
            provider: "openai-codex".into(),
            model: "gpt-test".into(),
            thinking: Some("high".into()),
        };
        let packet = AgentBootstrapPacket {
            schema: AGENT_BOOTSTRAP_PACKET_SCHEMA.into(),
            packet_id: Uuid::now_v7(),
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 1,
            generated_at: now,
            fresh_until,
            project_identity: ProjectIdentityBootstrapBinding {
                project_identity_ref: project_identity_ref.clone(),
                project_root: crate::test_support::absolute_path("silent-bootstrap-project"),
                fingerprint: "focusa-fingerprint".into(),
                snapshot_ref: "project-snapshot:1".into(),
                snapshot_sha256: "1".repeat(64),
                verified_at: now,
                fresh_until,
            },
            continuity_id: continuity_id.clone(),
            trajectory: TrajectoryBootstrapBinding {
                trajectory_ref: trajectory_ref.clone(),
                project_identity_ref: project_identity_ref.clone(),
                continuity_id: continuity_id.clone(),
                snapshot_ref: "trajectory-snapshot:1".into(),
                snapshot_sha256: "2".repeat(64),
                generated_at: now,
                fresh_until,
                status: TrajectoryBootstrapStatus::CanonicalAdvisory,
                waypoints: vec!["close the active gap".into()],
                active_gap: "bootstrap authority is unverified".into(),
            },
            workpoint: WorkpointBootstrapBinding {
                workpoint_ref: workpoint_ref.clone(),
                project_identity_ref: project_identity_ref.clone(),
                continuity_id: continuity_id.clone(),
                snapshot_ref: "workpoint-snapshot:7".into(),
                snapshot_sha256: "3".repeat(64),
                generated_at: now,
                fresh_until,
            },
            operator_ask: OperatorAskBinding::capture(
                "ask:bootstrap-test",
                "execute exact governed task",
                1,
                now,
            ),
            context: ContextBootstrapBinding {
                context_packet_ref: "context-packet:1".into(),
                project_identity_ref: project_identity_ref.clone(),
                continuity_id: continuity_id.clone(),
                trajectory_ref,
                workpoint_ref: workpoint_ref.clone(),
                source_snapshot_ref: "context-snapshot:1".into(),
                packet_sha256: "4".repeat(64),
                generated_at: now,
                fresh_until,
                advisory: true,
                canonical: false,
                canonical_mutation_allowed: false,
                selected_context: vec!["docs/spec133".into(), "src/adapter".into()],
                excluded_context: vec!["unrelated/project".into()],
                risk_refs: vec!["risk:project-mutation".into()],
                valid_next_tools: vec!["tool:workpoint-checkpoint".into()],
            },
            ontology: ontology(),
            work_item_ref: Some("focusa-a6yq6.4.4".into()),
            workspace: BootstrapWorkspaceBinding {
                workspace_ref: "workspace:spec133".into(),
                workspace_root: crate::test_support::absolute_path("silent-bootstrap-worktree"),
            },
            model: model.clone(),
            role_ref: "role:implementer".into(),
            mission: "implement the bounded Spec 133 slice".into(),
            exact_next_action: "prove the AgentBootstrap mutation barrier".into(),
            active_object_refs: vec!["spec:133".into()],
            hook_refs: vec!["hook:before-mutation".into()],
            blockers: vec![],
            do_not_drift: vec!["do not deploy".into()],
            evidence_refs: vec!["test:bootstrap-barrier".into()],
            proof_gaps: vec![],
            completion_expectations: vec!["focused tests pass".into()],
        };
        let verification = verify_agent_bootstrap_packet(&packet, now).unwrap();
        let actor_instance_ref = "actor-instance:spec133".to_owned();
        let lease = SilentSessionLease {
            schema: SILENT_SESSION_LEASE_SCHEMA.into(),
            lease_id: SilentSessionLeaseId::new(),
            session_id: packet.session_id,
            project_root: packet.project_identity.project_root.clone(),
            project_identity_ref: project_identity_ref.clone(),
            continuity_id: continuity_id.clone(),
            work_item_ref: packet.work_item_ref.clone(),
            workspace_ref: packet.workspace.workspace_ref.clone(),
            path_intents: vec![PathBuf::from("crates/focusa-core")],
            mutation_mode: crate::silent_session::WriterMutationMode::IsolatedWorktree,
            writer_role: "primary".into(),
            owner_actor_instance_ref: actor_instance_ref.clone(),
            fencing_token: 11,
            acquired_at: now - Duration::seconds(30),
            heartbeat_at: now,
            expires_at: now + Duration::minutes(2),
            adoption_policy: "operator_only".into(),
        };
        let authority = ContextAuthorityGrant {
            verdict_ref: "context-authority:spec133".into(),
            allowed: true,
            project_identity_ref,
            continuity_id,
            workpoint_ref: Some(workpoint_ref.workpoint_id),
            action_class: ContextAuthorityActionClass::GeneratedCodeOverwrite,
            action: "project_mutation".into(),
            action_digest: context_authority_action_digest(
                &packet,
                ContextAuthorityActionClass::GeneratedCodeOverwrite,
                "project_mutation",
            )
            .unwrap(),
            issued_at: now,
            expires_at: now + Duration::minutes(1),
        };
        Fixture {
            packet,
            verification,
            lease,
            authority,
            requested: model.clone(),
            effective: Some(model.clone()),
            observed: Some(model),
            now,
            actor_instance_ref,
        }
    }

    fn guarded_mutation(
        fixture: &Fixture,
        mutation_count: &mut usize,
    ) -> Result<VerifiedProjectMutationGrant, AgentBootstrapBarrierError> {
        let grant = verify_project_mutation_barrier(&fixture.request())?;
        *mutation_count += 1;
        Ok(grant)
    }

    #[test]
    fn bootstrap_verification_binds_all_authority_sources_and_blocks_generic_trajectory() {
        let fixture = fixture();
        fixture
            .verification
            .verify_for(&fixture.packet, fixture.now)
            .unwrap();

        let mut tampered = fixture.packet.clone();
        tampered.mission.push_str(" drift");
        assert_eq!(
            fixture.verification.verify_for(&tampered, fixture.now),
            Err(AgentBootstrapBarrierError::VerificationMismatch)
        );

        let mut changed_ask = fixture.packet.clone();
        changed_ask.operator_ask =
            OperatorAskBinding::capture("ask:changed", "newer exact operator ask", 2, fixture.now);
        assert_eq!(
            fixture.verification.verify_for(&changed_ask, fixture.now),
            Err(AgentBootstrapBarrierError::VerificationMismatch)
        );

        let mut generic = fixture.packet.clone();
        generic.trajectory.status = TrajectoryBootstrapStatus::GenericDegraded;
        assert_eq!(
            verify_agent_bootstrap_packet(&generic, fixture.now),
            Err(AgentBootstrapBarrierError::GenericTrajectoryBlocked)
        );

        let mut escalated = fixture.packet;
        escalated.context.canonical_mutation_allowed = true;
        assert_eq!(
            verify_agent_bootstrap_packet(&escalated, fixture.now),
            Err(AgentBootstrapBarrierError::ContextAuthorityEscalation)
        );
    }

    #[test]
    fn major_transition_requires_fresh_action_specific_bootstrap_authority() {
        let fixture = fixture();
        let mut authority = fixture.authority.clone();
        authority.action_class = ContextAuthorityActionClass::DaemonOrServiceRestart;
        authority.action = "restart_daemon".into();
        authority.action_digest = context_authority_action_digest(
            &fixture.packet,
            authority.action_class,
            &authority.action,
        )
        .unwrap();
        let request = MajorTransitionBarrierRequest {
            packet: &fixture.packet,
            bootstrap_verification: &fixture.verification,
            context_authority: &authority,
            action_class: ContextAuthorityActionClass::DaemonOrServiceRestart,
            action: "restart_daemon",
            now: fixture.now,
        };
        let grant = verify_major_transition_barrier(&request).unwrap();
        assert_eq!(grant.action, "restart_daemon");
        assert_eq!(grant.action_digest, authority.action_digest);

        let wrong_action = MajorTransitionBarrierRequest {
            action_class: ContextAuthorityActionClass::Release,
            action: "release",
            ..request
        };
        assert_eq!(
            verify_major_transition_barrier(&wrong_action),
            Err(AgentBootstrapBarrierError::ContextAuthorityDenied)
        );
    }

    #[test]
    fn no_project_mutation_runs_until_bootstrap_lease_authority_and_model_pass() {
        let mut mutation_count = 0;

        let mut invalid_bootstrap = fixture();
        invalid_bootstrap.packet.exact_next_action = "changed after verification".into();
        assert!(guarded_mutation(&invalid_bootstrap, &mut mutation_count).is_err());

        let mut stale_lease = fixture();
        stale_lease.lease.expires_at = stale_lease.now;
        assert_eq!(
            guarded_mutation(&stale_lease, &mut mutation_count),
            Err(AgentBootstrapBarrierError::LeaseDenied)
        );

        let mut stale_authority = fixture();
        stale_authority.authority.expires_at = stale_authority.now;
        assert_eq!(
            guarded_mutation(&stale_authority, &mut mutation_count),
            Err(AgentBootstrapBarrierError::ContextAuthorityDenied)
        );

        let mut unobserved_model = fixture();
        unobserved_model.observed = None;
        assert_eq!(
            guarded_mutation(&unobserved_model, &mut mutation_count),
            Err(AgentBootstrapBarrierError::ModelNotVerified)
        );
        assert_eq!(mutation_count, 0);

        let valid = fixture();
        let grant = guarded_mutation(&valid, &mut mutation_count).unwrap();
        assert_eq!(mutation_count, 1);
        assert_eq!(grant.run_id(), valid.packet.run_id);
        assert_eq!(grant.lease_fencing_token(), 11);
        assert_eq!(grant.model(), &valid.packet.model);
    }

    #[test]
    fn mutation_grant_is_exact_scope_bound_and_expires_at_earliest_barrier() {
        let fixture = fixture();
        let grant = verify_project_mutation_barrier(&fixture.request()).unwrap();
        assert_eq!(grant.valid_until(), fixture.authority.expires_at);
        assert!(
            grant
                .verify_execution_scope(
                    &fixture.packet.project_identity.project_root,
                    &fixture.packet.project_identity.project_identity_ref,
                    &fixture.packet.workspace.workspace_root,
                    fixture.now,
                )
                .is_ok()
        );
        assert_eq!(
            grant.verify_execution_scope(
                &fixture.packet.project_identity.project_root,
                &fixture.packet.project_identity.project_identity_ref,
                &crate::test_support::absolute_path("silent-bootstrap-other-worktree"),
                fixture.now,
            ),
            Err(AgentBootstrapBarrierError::ScopeMismatch("execution_scope"))
        );
        assert_eq!(
            grant.verify_execution_scope(
                &fixture.packet.project_identity.project_root,
                &fixture.packet.project_identity.project_identity_ref,
                &fixture.packet.workspace.workspace_root,
                grant.valid_until(),
            ),
            Err(AgentBootstrapBarrierError::GrantExpired)
        );
    }
}
