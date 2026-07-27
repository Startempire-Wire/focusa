//! Provider-neutral canonical release-cycle primitives (Spec143 §§14-15).
//!
//! This module owns release semantics, not provider execution. GitHub Actions,
//! local CI, package registries, and deployment systems are adapters that must
//! return exact-SHA evidence into these types.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};

pub const RELEASE_TOPOLOGY_SCHEMA: &str = "focusa.release_topology.v1";
pub const RELEASE_CANDIDATE_SCHEMA: &str = "focusa.release_candidate.v1";
pub const RELEASE_INTELLIGENCE_SCHEMA: &str = "focusa.release_intelligence.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStage {
    Plan,
    Locked,
    CandidateSnapshotted,
    Preflighted,
    Built,
    Packaged,
    Provenanced,
    DraftPublished,
    CanaryDeployed,
    Verified,
    Promoted,
    Closed,
    RolledBack,
    Cancelled,
}

impl ReleaseStage {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Plan => Some(Self::Locked),
            Self::Locked => Some(Self::CandidateSnapshotted),
            Self::CandidateSnapshotted => Some(Self::Preflighted),
            Self::Preflighted => Some(Self::Built),
            Self::Built => Some(Self::Packaged),
            Self::Packaged => Some(Self::Provenanced),
            Self::Provenanced => Some(Self::DraftPublished),
            Self::DraftPublished => Some(Self::CanaryDeployed),
            Self::CanaryDeployed => Some(Self::Verified),
            Self::Verified => Some(Self::Promoted),
            Self::Promoted => Some(Self::Closed),
            Self::Closed | Self::RolledBack | Self::Cancelled => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::RolledBack | Self::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSurfaceKind {
    Library,
    Package,
    Cli,
    Daemon,
    Tui,
    AgentExtension,
    Desktop,
    Mobile,
    Service,
    Container,
    Web,
    Installer,
    Documentation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSurface {
    pub surface_id: String,
    pub kind: ReleaseSurfaceKind,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub required_gates: Vec<String>,
    pub artifact_identity: String,
    pub deployment_target: Option<String>,
    #[serde(default)]
    pub canary_required: bool,
    #[serde(default)]
    pub rollback_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTopology {
    pub schema: String,
    pub project_id: String,
    pub profile: String,
    pub provider: String,
    pub surfaces: Vec<ReleaseSurface>,
    #[serde(default)]
    pub global_gates: Vec<String>,
}

impl ReleaseTopology {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_TOPOLOGY_SCHEMA,
            "unsupported release topology schema"
        );
        ensure!(!self.project_id.trim().is_empty(), "project_id is required");
        ensure!(
            !self.profile.trim().is_empty(),
            "release profile is required"
        );
        ensure!(
            !self.provider.trim().is_empty(),
            "release provider is required"
        );
        ensure!(
            !self.surfaces.is_empty(),
            "at least one release surface is required"
        );

        let mut ids = BTreeSet::new();
        for surface in &self.surfaces {
            ensure!(
                !surface.surface_id.trim().is_empty(),
                "surface_id is required"
            );
            ensure!(
                ids.insert(surface.surface_id.as_str()),
                "duplicate release surface {}",
                surface.surface_id
            );
            ensure!(
                !surface.artifact_identity.trim().is_empty(),
                "surface {} lacks artifact identity",
                surface.surface_id
            );
            ensure!(
                !surface.required_gates.is_empty(),
                "surface {} lacks required gates",
                surface.surface_id
            );
            ensure!(
                !surface.rollback_required || surface.deployment_target.is_some(),
                "surface {} requires rollback without a deployment target",
                surface.surface_id
            );
        }
        for surface in &self.surfaces {
            for dependency in &surface.depends_on {
                ensure!(
                    ids.contains(dependency.as_str()),
                    "surface {} has unknown dependency {}",
                    surface.surface_id,
                    dependency
                );
                ensure!(
                    dependency != &surface.surface_id,
                    "surface {} depends on itself",
                    surface.surface_id
                );
            }
        }
        ensure!(
            !self.has_cycle(),
            "release surface dependency graph contains a cycle"
        );
        Ok(())
    }

    fn has_cycle(&self) -> bool {
        let graph: BTreeMap<&str, Vec<&str>> = self
            .surfaces
            .iter()
            .map(|surface| {
                (
                    surface.surface_id.as_str(),
                    surface.depends_on.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        fn visit<'a>(
            node: &'a str,
            graph: &BTreeMap<&'a str, Vec<&'a str>>,
            visiting: &mut BTreeSet<&'a str>,
            visited: &mut BTreeSet<&'a str>,
        ) -> bool {
            if visited.contains(node) {
                return false;
            }
            if !visiting.insert(node) {
                return true;
            }
            if graph
                .get(node)
                .into_iter()
                .flatten()
                .any(|dependency| visit(dependency, graph, visiting, visited))
            {
                return true;
            }
            visiting.remove(node);
            visited.insert(node);
            false
        }
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        graph
            .keys()
            .any(|node| visit(node, &graph, &mut visiting, &mut visited))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEvidence {
    pub stage: ReleaseStage,
    pub exact_sha: String,
    pub observed_at: String,
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub invalidates: Vec<String>,
}

impl ReleaseEvidence {
    pub fn validate(&self, candidate_sha: &str) -> anyhow::Result<()> {
        ensure!(
            self.exact_sha == candidate_sha,
            "release evidence SHA differs from candidate SHA"
        );
        ensure!(
            !self.observed_at.trim().is_empty(),
            "release evidence timestamp is required"
        );
        ensure!(
            !self.evidence_refs.is_empty(),
            "release stage requires evidence refs"
        );
        ensure!(
            self.evidence_refs
                .iter()
                .all(|reference| !reference.trim().is_empty()),
            "release evidence refs cannot be empty"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseFixLane {
    pub failed_gate: String,
    pub affected_surfaces: Vec<String>,
    pub expected_proof: Vec<String>,
    pub invalidated_evidence: Vec<String>,
    pub new_candidate_required: bool,
    pub operator_amendment_ref: Option<String>,
}

impl ReleaseFixLane {
    pub fn validate(&self, topology: &ReleaseTopology) -> anyhow::Result<()> {
        ensure!(
            !self.failed_gate.trim().is_empty(),
            "failed gate is required"
        );
        ensure!(
            !self.affected_surfaces.is_empty(),
            "affected surfaces are required"
        );
        ensure!(
            !self.expected_proof.is_empty(),
            "expected fix proof is required"
        );
        let known: BTreeSet<_> = topology
            .surfaces
            .iter()
            .map(|surface| surface.surface_id.as_str())
            .collect();
        ensure!(
            self.affected_surfaces
                .iter()
                .all(|surface| known.contains(surface.as_str())),
            "fix lane contains an unknown surface"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseStageTiming {
    pub stage: ReleaseStage,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub elapsed_ms: u64,
    pub queue_ms: u64,
    pub retry_ms: u64,
    pub useful_work_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseBenchmark {
    pub total_elapsed_ms: u64,
    pub useful_work_ms: u64,
    pub queue_ms: u64,
    pub retry_ms: u64,
    pub human_interventions: u32,
    pub retries: u32,
    pub first_pass_gate_success_rate: f64,
    pub flow_efficiency: f64,
    pub critical_path: Vec<String>,
    pub missed_target_reason_codes: Vec<String>,
    pub stages: Vec<ReleaseStageTiming>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseCandidate {
    pub schema: String,
    pub candidate_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub workpoint_id: String,
    pub version: String,
    pub exact_sha: String,
    pub topology_ref: String,
    pub stage: ReleaseStage,
    pub locked_scope_refs: Vec<String>,
    pub evidence: Vec<ReleaseEvidence>,
    pub admitted_fixes: Vec<ReleaseFixLane>,
    pub benchmark: Option<ReleaseBenchmark>,
}

impl ReleaseCandidate {
    pub fn validate_identity(&self) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_CANDIDATE_SCHEMA,
            "unsupported release candidate schema"
        );
        ensure!(
            !self.candidate_id.trim().is_empty(),
            "candidate_id is required"
        );
        ensure!(
            !self.project_root.trim().is_empty(),
            "project_root is required"
        );
        ensure!(
            !self.continuity_id.trim().is_empty(),
            "continuity_id is required"
        );
        ensure!(
            !self.workpoint_id.trim().is_empty(),
            "release-scoped workpoint_id is required"
        );
        ensure!(!self.version.trim().is_empty(), "version is required");
        ensure!(self.exact_sha.len() >= 7, "exact candidate SHA is required");
        ensure!(
            !self.topology_ref.trim().is_empty(),
            "topology_ref is required"
        );
        ensure!(
            !self.locked_scope_refs.is_empty(),
            "release lock requires scope refs"
        );
        Ok(())
    }

    pub fn advance(&mut self, to: ReleaseStage, evidence: ReleaseEvidence) -> anyhow::Result<()> {
        self.validate_identity()?;
        let expected = self.stage.next().context("release candidate is terminal")?;
        ensure!(
            to == expected,
            "illegal release transition {:?} -> {:?}; expected {:?}",
            self.stage,
            to,
            expected
        );
        ensure!(
            evidence.stage == to,
            "release evidence stage does not match transition"
        );
        evidence.validate(&self.exact_sha)?;
        self.evidence.push(evidence);
        self.stage = to;
        Ok(())
    }

    pub fn admit_fix(
        &mut self,
        topology: &ReleaseTopology,
        fix: ReleaseFixLane,
    ) -> anyhow::Result<()> {
        self.validate_identity()?;
        ensure!(
            !self.stage.is_terminal(),
            "terminal release cannot admit a fix"
        );
        fix.validate(topology)?;
        self.admitted_fixes.push(fix);
        Ok(())
    }

    pub fn terminate(&mut self, to: ReleaseStage, evidence: ReleaseEvidence) -> anyhow::Result<()> {
        ensure!(
            matches!(to, ReleaseStage::RolledBack | ReleaseStage::Cancelled),
            "invalid release termination state"
        );
        ensure!(
            !self.stage.is_terminal(),
            "release candidate is already terminal"
        );
        evidence.validate(&self.exact_sha)?;
        self.evidence.push(evidence);
        self.stage = to;
        Ok(())
    }
}

#[cfg(test)]
#[path = "release_cycle_test.rs"]
mod tests;
