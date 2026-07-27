//! Typed provider-neutral protocol for the Master Release Cycle.

use std::collections::BTreeSet;

use anyhow::ensure;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::release_calibration::ReleasePlanTuning;
use crate::release_cycle::{
    ReleaseCandidate, ReleaseEvidence, ReleaseStage, ReleaseSurfaceKind, ReleaseTopology,
};

pub const RELEASE_ADAPTER_SCHEMA: &str = "focusa.release_adapter.v1";
pub const RELEASE_PLAN_SCHEMA: &str = "focusa.release_execution_plan.v1";
pub const RELEASE_RUN_SCHEMA: &str = "focusa.release_run_result.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseRunMode {
    Plan,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseInvocationSurface {
    Canvas,
    Terminal,
    Headless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterOutcome {
    Passed,
    Skipped,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseAuthority {
    pub project_root: String,
    pub continuity_id: String,
    pub operator_confirmed: bool,
    pub mutation_allowed: bool,
    #[serde(default)]
    pub approval_refs: Vec<String>,
}

impl ReleaseAuthority {
    pub(crate) fn validate(
        &self,
        candidate: &ReleaseCandidate,
        mode: ReleaseRunMode,
    ) -> anyhow::Result<()> {
        ensure!(
            self.project_root == candidate.project_root,
            "release authority project mismatch"
        );
        ensure!(
            self.continuity_id == candidate.continuity_id,
            "release authority continuity mismatch"
        );
        if mode == ReleaseRunMode::Execute {
            ensure!(
                self.operator_confirmed,
                "release execution requires operator confirmation"
            );
            ensure!(
                !self.approval_refs.is_empty(),
                "release execution requires approval evidence"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseAdapterDescriptor {
    pub schema: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub supported_profiles: Vec<String>,
    pub supported_surface_kinds: Vec<ReleaseSurfaceKind>,
    pub supported_stages: Vec<ReleaseStage>,
    pub supports_canary: bool,
    pub supports_rollback: bool,
}

impl ReleaseAdapterDescriptor {
    pub fn validate_for(&self, topology: &ReleaseTopology) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_ADAPTER_SCHEMA,
            "unsupported release adapter schema"
        );
        ensure!(!self.adapter_id.trim().is_empty(), "adapter_id is required");
        ensure!(
            !self.adapter_version.trim().is_empty(),
            "adapter_version is required"
        );
        ensure!(
            self.supported_profiles
                .iter()
                .any(|profile| profile == &topology.profile),
            "adapter does not support topology profile"
        );
        let kinds: BTreeSet<_> = self.supported_surface_kinds.iter().copied().collect();
        ensure!(
            topology
                .surfaces
                .iter()
                .all(|surface| kinds.contains(&surface.kind)),
            "adapter does not support every topology surface kind"
        );
        let stages: BTreeSet<_> = self.supported_stages.iter().copied().collect();
        ensure!(
            canonical_stages(topology)
                .iter()
                .all(|stage| stages.contains(stage)),
            "adapter does not support every required release stage"
        );
        ensure!(
            !topology
                .surfaces
                .iter()
                .any(|surface| surface.canary_required)
                || self.supports_canary,
            "topology requires canary but adapter cannot provide it"
        );
        let rollback_required = topology
            .surfaces
            .iter()
            .any(|surface| surface.rollback_required);
        ensure!(
            !rollback_required || self.supports_rollback,
            "topology requires rollback but adapter cannot provide it"
        );
        ensure!(
            !rollback_required || stages.contains(&ReleaseStage::RolledBack),
            "topology requires rollback but adapter lacks rolled_back stage"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseStageRequest {
    pub candidate_id: String,
    pub idempotency_key: String,
    pub exact_sha: String,
    pub version: String,
    pub project_root: String,
    pub topology: ReleaseTopology,
    pub stage: ReleaseStage,
    pub surface_waves: Vec<Vec<String>>,
    pub tuning: ReleasePlanTuning,
    pub immutable_artifact_set_id: Option<String>,
    pub approval_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseStageReceipt {
    pub stage: ReleaseStage,
    pub outcome: AdapterOutcome,
    pub evidence: ReleaseEvidence,
    pub adapter_id: String,
    pub artifact_set_id: Option<String>,
    pub rollback_ref: Option<String>,
    pub elapsed_ms: u64,
    pub queue_ms: u64,
    pub retry_ms: u64,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl ReleaseStageReceipt {
    pub(crate) fn validate(
        &self,
        request: &ReleaseStageRequest,
        descriptor: &ReleaseAdapterDescriptor,
    ) -> anyhow::Result<()> {
        ensure!(
            self.stage == request.stage,
            "adapter receipt stage mismatch"
        );
        ensure!(
            self.adapter_id == descriptor.adapter_id,
            "adapter receipt identity mismatch"
        );
        self.evidence.validate(&request.exact_sha)?;
        ensure!(
            self.evidence.stage == request.stage,
            "adapter evidence stage mismatch"
        );
        if matches!(
            self.stage,
            ReleaseStage::Built
                | ReleaseStage::Packaged
                | ReleaseStage::Provenanced
                | ReleaseStage::DraftPublished
                | ReleaseStage::CanaryDeployed
                | ReleaseStage::Verified
                | ReleaseStage::Promoted
        ) {
            ensure!(
                self.artifact_set_id
                    .as_deref()
                    .is_some_and(|id| !id.trim().is_empty()),
                "artifact-consuming stage requires immutable artifact_set_id"
            );
        }
        Ok(())
    }
}

#[async_trait]
pub trait ReleaseAdapter: Send + Sync {
    fn descriptor(&self) -> ReleaseAdapterDescriptor;
    async fn execute(&self, request: ReleaseStageRequest) -> anyhow::Result<ReleaseStageReceipt>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseExecutionPlan {
    pub schema: String,
    pub candidate_id: String,
    pub exact_sha: String,
    pub adapter_id: String,
    pub invocation_surface: ReleaseInvocationSurface,
    pub stages: Vec<ReleaseStage>,
    pub surface_waves: Vec<Vec<String>>,
    pub reused_stages: Vec<ReleaseStage>,
    pub mutating_stages: Vec<ReleaseStage>,
    pub tuning: ReleasePlanTuning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRunResult {
    pub schema: String,
    pub status: String,
    pub candidate: ReleaseCandidate,
    pub plan: ReleaseExecutionPlan,
    pub receipts: Vec<ReleaseStageReceipt>,
    pub blocked_stage: Option<ReleaseStage>,
    pub blocked_reasons: Vec<String>,
}

pub(crate) fn canonical_stages(_topology: &ReleaseTopology) -> Vec<ReleaseStage> {
    vec![
        ReleaseStage::Locked,
        ReleaseStage::CandidateSnapshotted,
        ReleaseStage::Preflighted,
        ReleaseStage::Built,
        ReleaseStage::Packaged,
        ReleaseStage::Provenanced,
        ReleaseStage::DraftPublished,
        ReleaseStage::CanaryDeployed,
        ReleaseStage::Verified,
        ReleaseStage::Promoted,
        ReleaseStage::Closed,
    ]
}
