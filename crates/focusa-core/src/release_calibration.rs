//! Deterministic benchmark calibration for increasingly efficient releases.
//!
//! Every adjustment is an evidence-backed experiment. The next observation
//! promotes or rolls it back; project/profile histories never mix.

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::release_cycle::{ReleaseBenchmark, ReleaseStage};

pub const RELEASE_CALIBRATION_OBSERVATION_SCHEMA: &str =
    "focusa.release_calibration_observation.v1";
pub const RELEASE_CALIBRATION_DECISION_SCHEMA: &str = "focusa.release_calibration_decision.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationOutcome {
    Baseline,
    Proposed,
    Promoted,
    RolledBack,
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleasePlanTuning {
    pub tuning_id: String,
    pub max_parallel_operations: u16,
    pub reuse_exact_sha_evidence: bool,
    pub preflight_before_immutable_tag: bool,
    pub priority_stage: Option<ReleaseStage>,
    pub strategy: String,
}

impl Default for ReleasePlanTuning {
    fn default() -> Self {
        Self {
            tuning_id: "baseline".into(),
            max_parallel_operations: 1,
            reuse_exact_sha_evidence: true,
            preflight_before_immutable_tag: true,
            priority_stage: None,
            strategy: "establish_baseline".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseCalibrationObservation {
    pub schema: String,
    pub release_id: String,
    pub project_id: String,
    pub profile: String,
    pub exact_sha: String,
    pub observed_at: String,
    pub applied_tuning_id: String,
    pub benchmark: ReleaseBenchmark,
    pub token_cost: u64,
    pub monetary_cost_microunits: u64,
    pub evidence_refs: Vec<String>,
}

impl ReleaseCalibrationObservation {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_CALIBRATION_OBSERVATION_SCHEMA,
            "unsupported calibration observation schema"
        );
        ensure!(!self.release_id.trim().is_empty(), "release_id is required");
        ensure!(!self.project_id.trim().is_empty(), "project_id is required");
        ensure!(!self.profile.trim().is_empty(), "profile is required");
        ensure!(!self.exact_sha.trim().is_empty(), "exact_sha is required");
        ensure!(
            !self.observed_at.trim().is_empty(),
            "observed_at is required"
        );
        ensure!(
            !self.applied_tuning_id.trim().is_empty(),
            "applied_tuning_id is required"
        );
        ensure!(
            self.benchmark.total_elapsed_ms > 0,
            "elapsed benchmark is required"
        );
        ensure!(
            !self.evidence_refs.is_empty(),
            "calibration observation requires evidence"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseCalibrationDecision {
    pub schema: String,
    pub decision_id: String,
    pub project_id: String,
    pub profile: String,
    pub based_on_release_ids: Vec<String>,
    pub outcome: CalibrationOutcome,
    pub elapsed_change_percent: Option<f64>,
    pub first_pass_change: Option<f64>,
    pub cost_change_percent: Option<f64>,
    pub active_tuning: ReleasePlanTuning,
    pub next_tuning: ReleasePlanTuning,
    pub reason_codes: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCalibrationPolicy {
    pub min_samples_for_experiment: usize,
    pub max_parallel_operations: u16,
    pub max_regression_percent: f64,
    pub first_pass_tolerance: f64,
}

impl Default for ReleaseCalibrationPolicy {
    fn default() -> Self {
        Self {
            min_samples_for_experiment: 2,
            max_parallel_operations: 8,
            max_regression_percent: 5.0,
            first_pass_tolerance: 0.01,
        }
    }
}

pub struct ReleaseCalibrator;

impl ReleaseCalibrator {
    pub fn decide(
        history: &[ReleaseCalibrationObservation],
        active_tuning: &ReleasePlanTuning,
        policy: &ReleaseCalibrationPolicy,
    ) -> anyhow::Result<ReleaseCalibrationDecision> {
        ensure!(
            !history.is_empty(),
            "calibration requires at least one observation"
        );
        for observation in history {
            observation.validate()?;
        }
        let latest = history.last().expect("non-empty history");
        ensure!(
            history
                .iter()
                .all(|item| item.project_id == latest.project_id && item.profile == latest.profile),
            "calibration history crosses project/profile authority"
        );
        let previous = history.iter().rev().nth(1);
        let mut outcome = CalibrationOutcome::Baseline;
        let mut reasons = Vec::new();
        let mut elapsed_change = None;
        let mut first_pass_change = None;
        let mut cost_change = None;

        if let Some(previous) = previous {
            elapsed_change = Some(percent_change(
                previous.benchmark.total_elapsed_ms,
                latest.benchmark.total_elapsed_ms,
            ));
            first_pass_change = Some(
                latest.benchmark.first_pass_gate_success_rate
                    - previous.benchmark.first_pass_gate_success_rate,
            );
            cost_change = Some(percent_change(
                previous.monetary_cost_microunits.max(1),
                latest.monetary_cost_microunits.max(1),
            ));
            if latest.applied_tuning_id == active_tuning.tuning_id
                && active_tuning.tuning_id != "baseline"
            {
                let elapsed_regression =
                    elapsed_change.unwrap_or_default() > policy.max_regression_percent;
                let reliability_regression =
                    first_pass_change.unwrap_or_default() < -policy.first_pass_tolerance;
                if elapsed_regression || reliability_regression {
                    outcome = CalibrationOutcome::RolledBack;
                    reasons.push(if elapsed_regression {
                        "elapsed_regression".into()
                    } else {
                        "first_pass_regression".into()
                    });
                } else {
                    outcome = CalibrationOutcome::Promoted;
                    reasons.push("experiment_improved_or_held_quality".into());
                }
            }
        }

        let base = if outcome == CalibrationOutcome::RolledBack {
            ReleasePlanTuning::default()
        } else {
            active_tuning.clone()
        };
        let mut next = base.clone();
        if history.len() < policy.min_samples_for_experiment {
            reasons.push("more_baseline_samples_required".into());
        } else {
            let benchmark = &latest.benchmark;
            let retry_ratio = benchmark.retry_ms as f64 / benchmark.total_elapsed_ms as f64;
            let queue_ratio = benchmark.queue_ms as f64 / benchmark.total_elapsed_ms as f64;
            let priority = benchmark
                .stages
                .iter()
                .max_by_key(|stage| stage.elapsed_ms)
                .map(|stage| stage.stage);
            next.tuning_id = Uuid::now_v7().to_string();
            next.priority_stage = priority;
            if benchmark.retries > 0 || benchmark.human_interventions > 0 || retry_ratio > 0.10 {
                next.strategy = "stabilize_critical_stage".into();
                reasons.push("retry_or_intervention_dominates".into());
            } else if queue_ratio > 0.20 {
                next.max_parallel_operations =
                    next.max_parallel_operations.saturating_sub(1).max(1);
                next.strategy = "reduce_runner_queue_contention".into();
                reasons.push("queue_time_dominates".into());
            } else if benchmark.flow_efficiency < 0.80
                && next.max_parallel_operations < policy.max_parallel_operations
            {
                next.max_parallel_operations += 1;
                next.strategy = "parallelize_independent_topology_waves".into();
                reasons.push("flow_efficiency_below_target".into());
            } else {
                next.strategy = "prewarm_critical_stage_cache".into();
                reasons.push("critical_path_cache_experiment".into());
            }
            if next != base {
                outcome = CalibrationOutcome::Proposed;
            }
        }

        let evidence_refs = history
            .iter()
            .rev()
            .take(2)
            .flat_map(|item| item.evidence_refs.clone())
            .collect();
        Ok(ReleaseCalibrationDecision {
            schema: RELEASE_CALIBRATION_DECISION_SCHEMA.into(),
            decision_id: Uuid::now_v7().to_string(),
            project_id: latest.project_id.clone(),
            profile: latest.profile.clone(),
            based_on_release_ids: history
                .iter()
                .rev()
                .take(2)
                .map(|item| item.release_id.clone())
                .collect(),
            outcome,
            elapsed_change_percent: elapsed_change,
            first_pass_change,
            cost_change_percent: cost_change,
            active_tuning: active_tuning.clone(),
            next_tuning: next,
            reason_codes: reasons,
            evidence_refs,
        })
    }
}

pub struct ReleaseCalibrationLedger;

impl ReleaseCalibrationLedger {
    pub fn append(
        path: impl AsRef<Path>,
        observation: &ReleaseCalibrationObservation,
    ) -> anyhow::Result<()> {
        observation.validate()?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("open calibration ledger {}", path.display()))?;
        serde_json::to_writer(&mut file, observation)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    }

    pub fn read(
        path: impl AsRef<Path>,
        project_id: &str,
        profile: &str,
    ) -> anyhow::Result<Vec<ReleaseCalibrationObservation>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(path)?;
        let mut output = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let item: ReleaseCalibrationObservation = serde_json::from_str(&line)
                .with_context(|| format!("invalid calibration ledger line {}", index + 1))?;
            item.validate()?;
            if item.project_id == project_id && item.profile == profile {
                output.push(item);
            }
        }
        Ok(output)
    }
}

fn percent_change(previous: u64, current: u64) -> f64 {
    if previous == 0 {
        return 0.0;
    }
    (current as f64 - previous as f64) * 100.0 / previous as f64
}

#[cfg(test)]
#[path = "release_calibration_test.rs"]
mod tests;
