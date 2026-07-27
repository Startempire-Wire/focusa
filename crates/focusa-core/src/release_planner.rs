//! Deterministic topology planning for the Master Release Cycle.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::ensure;

use crate::release_calibration::ReleasePlanTuning;
use crate::release_cycle::{ReleaseCandidate, ReleaseEvidence, ReleaseStage, ReleaseTopology};
use crate::release_orchestrator::MasterReleaseOrchestrator;
use crate::release_protocol::{
    RELEASE_PLAN_SCHEMA, ReleaseAdapterDescriptor, ReleaseExecutionPlan, ReleaseInvocationSurface,
    canonical_stages,
};

impl MasterReleaseOrchestrator {
    pub fn plan(
        candidate: &ReleaseCandidate,
        topology: &ReleaseTopology,
        adapter: &ReleaseAdapterDescriptor,
        reusable_evidence: &BTreeMap<ReleaseStage, ReleaseEvidence>,
        tuning: &ReleasePlanTuning,
    ) -> anyhow::Result<ReleaseExecutionPlan> {
        Self::plan_for_surface(
            candidate,
            topology,
            adapter,
            reusable_evidence,
            tuning,
            ReleaseInvocationSurface::Headless,
        )
    }

    pub fn plan_for_surface(
        candidate: &ReleaseCandidate,
        topology: &ReleaseTopology,
        adapter: &ReleaseAdapterDescriptor,
        reusable_evidence: &BTreeMap<ReleaseStage, ReleaseEvidence>,
        tuning: &ReleasePlanTuning,
        invocation_surface: ReleaseInvocationSurface,
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
            invocation_surface,
            stages,
            surface_waves: bounded_surface_waves(topology, tuning.max_parallel_operations)?,
            reused_stages,
            mutating_stages,
            tuning: tuning.clone(),
        })
    }
}

pub(crate) fn remaining_stages(
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

pub(crate) fn stage_mutates(stage: ReleaseStage) -> bool {
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

pub(crate) fn reusable_artifact_id(
    stage: ReleaseStage,
    candidate: &ReleaseCandidate,
) -> Option<String> {
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

pub fn bounded_surface_waves(
    topology: &ReleaseTopology,
    max_parallel_operations: u16,
) -> anyhow::Result<Vec<Vec<String>>> {
    ensure!(
        max_parallel_operations > 0,
        "max_parallel_operations must be positive"
    );
    let limit = usize::from(max_parallel_operations);
    Ok(surface_waves(topology)?
        .into_iter()
        .flat_map(|wave| {
            wave.chunks(limit)
                .map(<[String]>::to_vec)
                .collect::<Vec<_>>()
        })
        .collect())
}
