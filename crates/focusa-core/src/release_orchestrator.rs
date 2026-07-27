//! Provider-neutral Master Release Cycle orchestration (Spec143 §§14-20).
//!
//! The kernel owns authority, ordering, exact-SHA evidence, and settlement.
//! Providers implement `ReleaseAdapter`; they never choose release state.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, ensure};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::release_calibration::ReleasePlanTuning;
use crate::release_cycle::{
    ReleaseCandidate, ReleaseEvidence, ReleaseStage, ReleaseSurfaceKind, ReleaseTopology,
};
use crate::release_ledger::{
    NoopReleaseCheckpointSink, RELEASE_CHECKPOINT_SCHEMA, ReleaseCheckpointSink,
    ReleaseRunCheckpoint,
};

pub use crate::release_planner::{bounded_surface_waves, surface_waves};
use crate::release_planner::{remaining_stages, reusable_artifact_id, stage_mutates};
pub use crate::release_protocol::*;

pub struct MasterReleaseOrchestrator;

impl MasterReleaseOrchestrator {
    pub async fn run<A: ReleaseAdapter>(
        candidate: ReleaseCandidate,
        topology: ReleaseTopology,
        adapter: &A,
        authority: ReleaseAuthority,
        mode: ReleaseRunMode,
        observed_at: &str,
        reusable_evidence: BTreeMap<ReleaseStage, ReleaseEvidence>,
    ) -> anyhow::Result<ReleaseRunResult> {
        Self::run_input(
            adapter,
            ReleaseRunInput {
                candidate,
                topology,
                authority,
                mode,
                observed_at: observed_at.into(),
                reusable_evidence,
                tuning: ReleasePlanTuning::default(),
                invocation_surface: ReleaseInvocationSurface::Headless,
                resume_receipts: Vec::new(),
            },
        )
        .await
    }

    pub async fn run_input<A: ReleaseAdapter>(
        adapter: &A,
        input: ReleaseRunInput,
    ) -> anyhow::Result<ReleaseRunResult> {
        Self::run_with_checkpoint_sink(adapter, input, &NoopReleaseCheckpointSink).await
    }

    pub async fn run_with_checkpoint_sink<A: ReleaseAdapter, S: ReleaseCheckpointSink>(
        adapter: &A,
        input: ReleaseRunInput,
        checkpoint_sink: &S,
    ) -> anyhow::Result<ReleaseRunResult> {
        let ReleaseRunInput {
            mut candidate,
            topology,
            authority,
            mode,
            observed_at,
            reusable_evidence,
            tuning,
            invocation_surface,
            resume_receipts,
        } = input;
        let observed_at = observed_at.as_str();
        authority.validate(&candidate, mode)?;
        let descriptor = adapter.descriptor();
        let plan = Self::plan_for_surface(
            &candidate,
            &topology,
            &descriptor,
            &reusable_evidence,
            &tuning,
            invocation_surface,
        )?;
        let mut checkpoint_sequence = checkpoint_sink.next_sequence()?;
        checkpoint_sink.append(&checkpoint(
            checkpoint_sequence,
            if mode == ReleaseRunMode::Plan {
                "planned"
            } else {
                "started"
            },
            observed_at,
            &candidate,
            &plan,
            &resume_receipts,
            &[],
        ))?;
        checkpoint_sequence += 1;
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
            let reasons = vec!["mutation_authority_missing".to_string()];
            checkpoint_sink.append(&checkpoint(
                checkpoint_sequence,
                "blocked",
                observed_at,
                &candidate,
                &plan,
                &resume_receipts,
                &reasons,
            ))?;
            return Ok(blocked_result(
                candidate,
                plan,
                None,
                "mutation_authority_missing",
            ));
        }

        let mut immutable_artifact_set_id: Option<String> = None;
        for receipt in &resume_receipts {
            ensure!(
                receipt.evidence.exact_sha == candidate.exact_sha,
                "resume receipt SHA mismatch"
            );
            ensure!(
                receipt.adapter_id == descriptor.adapter_id,
                "resume receipt adapter mismatch"
            );
            if let Some(identity) = &receipt.artifact_set_id {
                if let Some(expected) = &immutable_artifact_set_id {
                    ensure!(
                        identity == expected,
                        "resume receipts contain different artifact sets"
                    );
                } else {
                    immutable_artifact_set_id = Some(identity.clone());
                }
            }
        }
        let mut receipts = resume_receipts;
        for stage in plan.stages.clone() {
            let receipt = if let Some(evidence) = reusable_evidence.get(&stage) {
                ReleaseStageReceipt {
                    stage,
                    outcome: AdapterOutcome::Passed,
                    evidence: evidence.clone(),
                    adapter_id: descriptor.adapter_id.clone(),
                    artifact_set_id: reusable_artifact_id(stage, &candidate)
                        .or_else(|| immutable_artifact_set_id.clone()),
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
                    artifact_set_id: immutable_artifact_set_id.clone(),
                    rollback_ref: None,
                    elapsed_ms: 0,
                    queue_ms: 0,
                    retry_ms: 0,
                    reason_codes: vec!["canary_not_required".into()],
                }
            } else {
                let request = ReleaseStageRequest {
                    candidate_id: candidate.candidate_id.clone(),
                    idempotency_key: operation_idempotency_key(&candidate, stage),
                    exact_sha: candidate.exact_sha.clone(),
                    version: candidate.version.clone(),
                    project_root: candidate.project_root.clone(),
                    topology: topology.clone(),
                    stage,
                    surface_waves: plan.surface_waves.clone(),
                    tuning: plan.tuning.clone(),
                    immutable_artifact_set_id: immutable_artifact_set_id.clone(),
                    approval_refs: authority.approval_refs.clone(),
                };
                adapter
                    .execute(request.clone())
                    .await
                    .with_context(|| format!("adapter failed at {stage:?}"))?
            };
            let request = ReleaseStageRequest {
                candidate_id: candidate.candidate_id.clone(),
                idempotency_key: operation_idempotency_key(&candidate, stage),
                exact_sha: candidate.exact_sha.clone(),
                version: candidate.version.clone(),
                project_root: candidate.project_root.clone(),
                topology: topology.clone(),
                stage,
                surface_waves: plan.surface_waves.clone(),
                tuning: plan.tuning.clone(),
                immutable_artifact_set_id: immutable_artifact_set_id.clone(),
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
                if rollback_required(&candidate, &topology) {
                    let rollback_request = ReleaseStageRequest {
                        candidate_id: candidate.candidate_id.clone(),
                        idempotency_key: operation_idempotency_key(
                            &candidate,
                            ReleaseStage::RolledBack,
                        ),
                        exact_sha: candidate.exact_sha.clone(),
                        version: candidate.version.clone(),
                        project_root: candidate.project_root.clone(),
                        topology: topology.clone(),
                        stage: ReleaseStage::RolledBack,
                        surface_waves: plan.surface_waves.clone(),
                        tuning: plan.tuning.clone(),
                        immutable_artifact_set_id: immutable_artifact_set_id.clone(),
                        approval_refs: authority.approval_refs.clone(),
                    };
                    let rollback = adapter
                        .execute(rollback_request.clone())
                        .await
                        .context("adapter rollback failed")?;
                    rollback.validate(&rollback_request, &descriptor)?;
                    ensure!(
                        rollback.outcome == AdapterOutcome::Passed,
                        "release rollback did not pass"
                    );
                    candidate.terminate(ReleaseStage::RolledBack, rollback.evidence.clone())?;
                    receipts.push(rollback);
                    checkpoint_sink.append(&checkpoint(
                        checkpoint_sequence,
                        "rolled_back",
                        observed_at,
                        &candidate,
                        &plan,
                        &receipts,
                        &reasons,
                    ))?;
                    return Ok(ReleaseRunResult {
                        schema: RELEASE_RUN_SCHEMA.into(),
                        status: "rolled_back".into(),
                        candidate,
                        plan,
                        receipts,
                        blocked_stage: Some(stage),
                        blocked_reasons: reasons,
                    });
                }
                checkpoint_sink.append(&checkpoint(
                    checkpoint_sequence,
                    "blocked",
                    observed_at,
                    &candidate,
                    &plan,
                    &receipts,
                    &reasons,
                ))?;
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
            if let Some(identity) = &receipt.artifact_set_id {
                if let Some(expected) = &immutable_artifact_set_id {
                    ensure!(
                        identity == expected,
                        "immutable artifact set changed between release stages"
                    );
                } else {
                    immutable_artifact_set_id = Some(identity.clone());
                }
            }
            candidate.advance(stage, receipt.evidence.clone())?;
            receipts.push(receipt);
            checkpoint_sink.append(&checkpoint(
                checkpoint_sequence,
                if candidate.stage == ReleaseStage::Closed {
                    "closed"
                } else {
                    "running"
                },
                observed_at,
                &candidate,
                &plan,
                &receipts,
                &[],
            ))?;
            checkpoint_sequence += 1;
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

fn checkpoint(
    sequence: u64,
    status: &str,
    observed_at: &str,
    candidate: &ReleaseCandidate,
    plan: &ReleaseExecutionPlan,
    receipts: &[ReleaseStageReceipt],
    blocked_reasons: &[String],
) -> ReleaseRunCheckpoint {
    ReleaseRunCheckpoint {
        schema: RELEASE_CHECKPOINT_SCHEMA.into(),
        sequence,
        status: status.into(),
        observed_at: observed_at.into(),
        candidate: candidate.clone(),
        plan: plan.clone(),
        receipts: receipts.to_vec(),
        blocked_reasons: blocked_reasons.to_vec(),
    }
}

fn operation_idempotency_key(candidate: &ReleaseCandidate, stage: ReleaseStage) -> String {
    format!(
        "{}:{}:{stage:?}",
        candidate.candidate_id, candidate.exact_sha
    )
}

fn rollback_required(candidate: &ReleaseCandidate, topology: &ReleaseTopology) -> bool {
    candidate.stage >= ReleaseStage::CanaryDeployed
        && candidate.stage < ReleaseStage::Closed
        && topology
            .surfaces
            .iter()
            .any(|surface| surface.rollback_required)
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

#[cfg(test)]
#[path = "release_orchestrator_test.rs"]
mod tests;
