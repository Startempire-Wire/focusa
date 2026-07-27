//! Provider-neutral Master Release Cycle orchestration (Spec143 §§14-20).
//!
//! The kernel owns authority, ordering, exact-SHA evidence, and settlement.
//! Providers implement `ReleaseAdapter`; they never choose release state.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, ensure};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    fn validate(&self, candidate: &ReleaseCandidate, mode: ReleaseRunMode) -> anyhow::Result<()> {
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
        ensure!(
            !topology
                .surfaces
                .iter()
                .any(|surface| surface.rollback_required)
                || self.supports_rollback,
            "topology requires rollback but adapter cannot provide it"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseStageRequest {
    pub candidate_id: String,
    pub exact_sha: String,
    pub version: String,
    pub project_root: String,
    pub topology: ReleaseTopology,
    pub stage: ReleaseStage,
    pub surface_waves: Vec<Vec<String>>,
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
    fn validate(
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
            ReleaseStage::Built | ReleaseStage::Packaged | ReleaseStage::Provenanced
        ) {
            ensure!(
                self.artifact_set_id
                    .as_deref()
                    .is_some_and(|id| !id.trim().is_empty()),
                "artifact stage requires immutable artifact_set_id"
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
    pub stages: Vec<ReleaseStage>,
    pub surface_waves: Vec<Vec<String>>,
    pub reused_stages: Vec<ReleaseStage>,
    pub mutating_stages: Vec<ReleaseStage>,
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

pub struct MasterReleaseOrchestrator;

impl MasterReleaseOrchestrator {
    pub fn plan(
        candidate: &ReleaseCandidate,
        topology: &ReleaseTopology,
        adapter: &ReleaseAdapterDescriptor,
        reusable_evidence: &BTreeMap<ReleaseStage, ReleaseEvidence>,
    ) -> anyhow::Result<ReleaseExecutionPlan> {
        candidate.validate_identity()?;
        topology.validate()?;
        adapter.validate_for(topology)?;
        let stages = remaining_stages(candidate.stage, topology)?;
        let reused_stages = stages
            .iter()
            .filter(|stage| {
                reusable_evidence.get(stage).is_some_and(|evidence| {
                    evidence.validate(&candidate.exact_sha).is_ok() && evidence.stage == **stage
                })
            })
            .copied()
            .collect();
        let mutating_stages = stages
            .iter()
            .filter(|stage| stage_mutates(**stage))
            .copied()
            .collect();
        Ok(ReleaseExecutionPlan {
            schema: RELEASE_PLAN_SCHEMA.into(),
            candidate_id: candidate.candidate_id.clone(),
            exact_sha: candidate.exact_sha.clone(),
            adapter_id: adapter.adapter_id.clone(),
            stages,
            surface_waves: surface_waves(topology)?,
            reused_stages,
            mutating_stages,
        })
    }

    pub async fn run<A: ReleaseAdapter>(
        mut candidate: ReleaseCandidate,
        topology: ReleaseTopology,
        adapter: &A,
        authority: ReleaseAuthority,
        mode: ReleaseRunMode,
        observed_at: &str,
        reusable_evidence: BTreeMap<ReleaseStage, ReleaseEvidence>,
    ) -> anyhow::Result<ReleaseRunResult> {
        authority.validate(&candidate, mode)?;
        let descriptor = adapter.descriptor();
        let plan = Self::plan(&candidate, &topology, &descriptor, &reusable_evidence)?;
        if mode == ReleaseRunMode::Plan {
            return Ok(ReleaseRunResult {
                schema: RELEASE_RUN_SCHEMA.into(),
                status: "planned".into(),
                candidate,
                plan,
                receipts: Vec::new(),
                blocked_stage: None,
                blocked_reasons: Vec::new(),
            });
        }
        if !authority.mutation_allowed && !plan.mutating_stages.is_empty() {
            return Ok(blocked_result(
                candidate,
                plan,
                None,
                "mutation_authority_missing",
            ));
        }

        let mut receipts = Vec::new();
        for stage in plan.stages.clone() {
            let receipt = if let Some(evidence) = reusable_evidence.get(&stage) {
                ReleaseStageReceipt {
                    stage,
                    outcome: AdapterOutcome::Passed,
                    evidence: evidence.clone(),
                    adapter_id: descriptor.adapter_id.clone(),
                    artifact_set_id: reusable_artifact_id(stage, &candidate),
                    rollback_ref: None,
                    elapsed_ms: 0,
                    queue_ms: 0,
                    retry_ms: 0,
                    reason_codes: vec!["exact_sha_evidence_reused".into()],
                }
            } else if stage == ReleaseStage::CanaryDeployed
                && !topology
                    .surfaces
                    .iter()
                    .any(|surface| surface.canary_required)
            {
                ReleaseStageReceipt {
                    stage,
                    outcome: AdapterOutcome::Skipped,
                    evidence: ReleaseEvidence {
                        stage,
                        exact_sha: candidate.exact_sha.clone(),
                        observed_at: observed_at.into(),
                        evidence_refs: vec!["focusa:canary:not-required".into()],
                        invalidates: Vec::new(),
                    },
                    adapter_id: descriptor.adapter_id.clone(),
                    artifact_set_id: None,
                    rollback_ref: None,
                    elapsed_ms: 0,
                    queue_ms: 0,
                    retry_ms: 0,
                    reason_codes: vec!["canary_not_required".into()],
                }
            } else {
                let request = ReleaseStageRequest {
                    candidate_id: candidate.candidate_id.clone(),
                    exact_sha: candidate.exact_sha.clone(),
                    version: candidate.version.clone(),
                    project_root: candidate.project_root.clone(),
                    topology: topology.clone(),
                    stage,
                    surface_waves: plan.surface_waves.clone(),
                    approval_refs: authority.approval_refs.clone(),
                };
                adapter
                    .execute(request.clone())
                    .await
                    .with_context(|| format!("adapter failed at {stage:?}"))?
            };
            let request = ReleaseStageRequest {
                candidate_id: candidate.candidate_id.clone(),
                exact_sha: candidate.exact_sha.clone(),
                version: candidate.version.clone(),
                project_root: candidate.project_root.clone(),
                topology: topology.clone(),
                stage,
                surface_waves: plan.surface_waves.clone(),
                approval_refs: authority.approval_refs.clone(),
            };
            receipt.validate(&request, &descriptor)?;
            if receipt.outcome == AdapterOutcome::Blocked {
                let reasons = if receipt.reason_codes.is_empty() {
                    vec!["adapter_blocked".into()]
                } else {
                    receipt.reason_codes.clone()
                };
                receipts.push(receipt);
                return Ok(ReleaseRunResult {
                    schema: RELEASE_RUN_SCHEMA.into(),
                    status: "blocked".into(),
                    candidate,
                    plan,
                    receipts,
                    blocked_stage: Some(stage),
                    blocked_reasons: reasons,
                });
            }
            candidate.advance(stage, receipt.evidence.clone())?;
            receipts.push(receipt);
        }
        Ok(ReleaseRunResult {
            schema: RELEASE_RUN_SCHEMA.into(),
            status: "closed".into(),
            candidate,
            plan,
            receipts,
            blocked_stage: None,
            blocked_reasons: Vec::new(),
        })
    }
}

fn blocked_result(
    candidate: ReleaseCandidate,
    plan: ReleaseExecutionPlan,
    blocked_stage: Option<ReleaseStage>,
    reason: &str,
) -> ReleaseRunResult {
    ReleaseRunResult {
        schema: RELEASE_RUN_SCHEMA.into(),
        status: "blocked".into(),
        candidate,
        plan,
        receipts: Vec::new(),
        blocked_stage,
        blocked_reasons: vec![reason.into()],
    }
}

fn canonical_stages(_topology: &ReleaseTopology) -> Vec<ReleaseStage> {
    let mut stages = vec![
        ReleaseStage::Locked,
        ReleaseStage::CandidateSnapshotted,
        ReleaseStage::Preflighted,
        ReleaseStage::Built,
        ReleaseStage::Packaged,
        ReleaseStage::Provenanced,
        ReleaseStage::DraftPublished,
    ];
    stages.push(ReleaseStage::CanaryDeployed);
    stages.extend([
        ReleaseStage::Verified,
        ReleaseStage::Promoted,
        ReleaseStage::Closed,
    ]);
    stages
}

fn remaining_stages(
    current: ReleaseStage,
    topology: &ReleaseTopology,
) -> anyhow::Result<Vec<ReleaseStage>> {
    ensure!(
        !current.is_terminal(),
        "terminal candidate cannot be resumed"
    );
    Ok(canonical_stages(topology)
        .into_iter()
        .filter(|stage| *stage > current)
        .collect())
}

fn stage_mutates(stage: ReleaseStage) -> bool {
    matches!(
        stage,
        ReleaseStage::Built
            | ReleaseStage::Packaged
            | ReleaseStage::Provenanced
            | ReleaseStage::DraftPublished
            | ReleaseStage::CanaryDeployed
            | ReleaseStage::Promoted
            | ReleaseStage::Closed
    )
}

fn reusable_artifact_id(stage: ReleaseStage, candidate: &ReleaseCandidate) -> Option<String> {
    matches!(
        stage,
        ReleaseStage::Built | ReleaseStage::Packaged | ReleaseStage::Provenanced
    )
    .then(|| format!("reused:{}:{}", candidate.exact_sha, candidate.version))
}

pub fn surface_waves(topology: &ReleaseTopology) -> anyhow::Result<Vec<Vec<String>>> {
    topology.validate()?;
    let mut remaining: BTreeMap<String, BTreeSet<String>> = topology
        .surfaces
        .iter()
        .map(|surface| {
            (
                surface.surface_id.clone(),
                surface.depends_on.iter().cloned().collect(),
            )
        })
        .collect();
    let mut completed = BTreeSet::new();
    let mut waves = Vec::new();
    while !remaining.is_empty() {
        let wave: Vec<_> = remaining
            .iter()
            .filter(|(_, deps)| deps.is_subset(&completed))
            .map(|(id, _)| id.clone())
            .collect();
        ensure!(!wave.is_empty(), "release topology dependency cycle");
        for id in &wave {
            remaining.remove(id);
            completed.insert(id.clone());
        }
        waves.push(wave);
    }
    Ok(waves)
}

#[cfg(test)]
#[path = "release_orchestrator_test.rs"]
mod tests;
