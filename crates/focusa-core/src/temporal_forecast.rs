//! Spec137 calibrated forecasting and release-cycle timing from observed evidence.

use crate::temporal::{TemporalConfidence, TemporalScope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePhase {
    CadenceWait,
    Freeze,
    Build,
    Sign,
    Publish,
    Deploy,
    ArtifactPropagation,
    UpdateRollout,
    CanaryObservation,
    RollbackDecision,
    RollbackRecovery,
    Approval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedDuration {
    pub observation_id: String,
    pub scope: TemporalScope,
    pub phase: ReleasePhase,
    pub duration_ms: u64,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastRange {
    pub phase: ReleasePhase,
    pub sample_count: usize,
    pub minimum_ms: u64,
    pub p50_ms: u64,
    pub p80_ms: u64,
    pub p95_ms: u64,
    pub maximum_ms: u64,
    pub coverage_probability: f64,
    pub confidence: TemporalConfidence,
    pub method: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTimingStep {
    pub step_id: String,
    pub phase: ReleasePhase,
    pub depends_on: Vec<String>,
    pub forecast_ms: u64,
    pub earliest_start_ms: u64,
    pub earliest_finish_ms: u64,
    pub latest_start_ms: u64,
    pub slack_ms: u64,
    pub critical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTimingPlan {
    pub scope: TemporalScope,
    pub generated_at: DateTime<Utc>,
    pub steps: Vec<ReleaseTimingStep>,
    pub critical_path_ms: u64,
    pub observation_window_ms: u64,
    pub rollback_window_ms: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastCalibration {
    pub phase: ReleasePhase,
    pub forecast_p50_ms: u64,
    pub forecast_p95_ms: u64,
    pub actual_ms: u64,
    pub absolute_error_ms: u64,
    pub within_p95: bool,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissedTargetReceipt {
    pub receipt_id: String,
    pub scope: TemporalScope,
    pub target_ref: String,
    pub target_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    pub lateness_ms: u64,
    pub reason_codes: Vec<String>,
    pub lost_time_ms: u64,
    pub evidence_refs: Vec<String>,
    pub blame_assigned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoraTemporalMetrics {
    pub lead_time_ms: Option<u64>,
    pub deployment_duration_ms: Option<u64>,
    pub recovery_time_ms: Option<u64>,
    pub observation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForecastError {
    NoObservedHistory,
    ScopeMismatch,
    DuplicateStep,
    MissingDependency(String),
    DependencyCycle,
}

fn quantile(sorted: &[u64], probability: f64) -> u64 {
    let index = ((sorted.len() - 1) as f64 * probability).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

pub fn forecast_phase(
    scope: &TemporalScope,
    phase: ReleasePhase,
    observations: &[ObservedDuration],
) -> Result<ForecastRange, ForecastError> {
    let selected = observations
        .iter()
        .filter(|observation| &observation.scope == scope && observation.phase == phase)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(ForecastError::NoObservedHistory);
    }
    let mut durations = selected
        .iter()
        .map(|observation| observation.duration_ms)
        .collect::<Vec<_>>();
    durations.sort_unstable();
    let confidence = match durations.len() {
        0..=2 => TemporalConfidence::Low,
        3..=7 => TemporalConfidence::Medium,
        8..=19 => TemporalConfidence::High,
        _ => TemporalConfidence::Verified,
    };
    Ok(ForecastRange {
        phase,
        sample_count: durations.len(),
        minimum_ms: durations[0],
        p50_ms: quantile(&durations, 0.50),
        p80_ms: quantile(&durations, 0.80),
        p95_ms: quantile(&durations, 0.95),
        maximum_ms: *durations.last().expect("non-empty durations"),
        coverage_probability: 0.95,
        confidence,
        method: "empirical_nearest_rank".into(),
        evidence_refs: selected
            .iter()
            .flat_map(|observation| observation.evidence_refs.clone())
            .collect(),
    })
}

pub fn build_release_timing_plan(
    scope: TemporalScope,
    definitions: Vec<(String, ReleasePhase, Vec<String>, u64)>,
    generated_at: DateTime<Utc>,
) -> Result<ReleaseTimingPlan, ForecastError> {
    let mut by_id = BTreeMap::new();
    for (id, phase, dependencies, forecast_ms) in definitions {
        if by_id
            .insert(id.clone(), (phase, dependencies, forecast_ms))
            .is_some()
        {
            return Err(ForecastError::DuplicateStep);
        }
    }
    for (_, dependencies, _) in by_id.values() {
        for dependency in dependencies {
            if !by_id.contains_key(dependency) {
                return Err(ForecastError::MissingDependency(dependency.clone()));
            }
        }
    }
    let mut finished = BTreeMap::<String, u64>::new();
    let mut order = Vec::new();
    while order.len() < by_id.len() {
        let ready = by_id
            .iter()
            .filter(|(id, (_, dependencies, _))| {
                !finished.contains_key(*id)
                    && dependencies
                        .iter()
                        .all(|dependency| finished.contains_key(dependency))
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(ForecastError::DependencyCycle);
        }
        for id in ready {
            let (_, dependencies, duration) = &by_id[&id];
            let start = dependencies
                .iter()
                .filter_map(|dependency| finished.get(dependency))
                .copied()
                .max()
                .unwrap_or(0);
            finished.insert(id.clone(), start + duration);
            order.push(id);
        }
    }
    let critical_path_ms = finished.values().copied().max().unwrap_or(0);
    let mut steps = order
        .iter()
        .map(|id| {
            let (phase, dependencies, duration) = &by_id[id];
            let start = dependencies
                .iter()
                .filter_map(|dependency| finished.get(dependency))
                .copied()
                .max()
                .unwrap_or(0);
            let finish = start + duration;
            ReleaseTimingStep {
                step_id: id.clone(),
                phase: *phase,
                depends_on: dependencies.clone(),
                forecast_ms: *duration,
                earliest_start_ms: start,
                earliest_finish_ms: finish,
                latest_start_ms: critical_path_ms.saturating_sub(*duration),
                slack_ms: critical_path_ms.saturating_sub(finish),
                critical: finish == critical_path_ms,
            }
        })
        .collect::<Vec<_>>();
    let critical_ids = steps
        .iter()
        .filter(|step| step.critical)
        .map(|step| step.step_id.clone())
        .collect::<BTreeSet<_>>();
    for step in &mut steps {
        if step
            .depends_on
            .iter()
            .any(|dependency| critical_ids.contains(dependency))
            && step.slack_ms == 0
        {
            step.critical = true;
        }
    }
    let observation_window_ms = steps
        .iter()
        .filter(|step| step.phase == ReleasePhase::CanaryObservation)
        .map(|step| step.forecast_ms)
        .sum();
    let rollback_window_ms = steps
        .iter()
        .filter(|step| {
            matches!(
                step.phase,
                ReleasePhase::RollbackDecision | ReleasePhase::RollbackRecovery
            )
        })
        .map(|step| step.forecast_ms)
        .sum();
    Ok(ReleaseTimingPlan {
        scope,
        generated_at,
        steps,
        critical_path_ms,
        observation_window_ms,
        rollback_window_ms,
        warnings: Vec::new(),
    })
}

pub fn calibrate(range: &ForecastRange, actual_ms: u64) -> ForecastCalibration {
    let absolute_error_ms = range.p50_ms.abs_diff(actual_ms);
    let denominator = actual_ms.max(1) as f64;
    ForecastCalibration {
        phase: range.phase,
        forecast_p50_ms: range.p50_ms,
        forecast_p95_ms: range.p95_ms,
        actual_ms,
        absolute_error_ms,
        within_p95: actual_ms <= range.p95_ms,
        score: (1.0 - absolute_error_ms as f64 / denominator).clamp(0.0, 1.0),
    }
}

pub fn dora_metrics(observations: &[ObservedDuration]) -> DoraTemporalMetrics {
    let sum = |phase| {
        observations
            .iter()
            .filter(|observation| observation.phase == phase)
            .map(|observation| observation.duration_ms)
            .sum::<u64>()
    };
    let build_to_deploy = [
        ReleasePhase::Build,
        ReleasePhase::Sign,
        ReleasePhase::Publish,
        ReleasePhase::Deploy,
    ]
    .into_iter()
    .map(sum)
    .sum::<u64>();
    let recovery = sum(ReleasePhase::RollbackDecision) + sum(ReleasePhase::RollbackRecovery);
    DoraTemporalMetrics {
        lead_time_ms: (!observations.is_empty()).then_some(build_to_deploy),
        deployment_duration_ms: observations
            .iter()
            .any(|observation| observation.phase == ReleasePhase::Deploy)
            .then(|| sum(ReleasePhase::Deploy)),
        recovery_time_ms: (recovery > 0).then_some(recovery),
        observation_count: observations.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> TemporalScope {
        TemporalScope {
            project_root: "/workspace/project".into(),
            continuity_id: "main".into(),
        }
    }

    #[test]
    fn forecast_requires_history_and_returns_range_not_exact_guess() {
        assert_eq!(
            forecast_phase(&scope(), ReleasePhase::Build, &[]),
            Err(ForecastError::NoObservedHistory)
        );
        let now = Utc::now();
        let observations = [10, 20, 30, 40, 100]
            .into_iter()
            .enumerate()
            .map(|(index, duration)| ObservedDuration {
                observation_id: format!("o{index}"),
                scope: scope(),
                phase: ReleasePhase::Build,
                duration_ms: duration,
                outcome: "success".into(),
                reason_code: None,
                started_at: now,
                completed_at: now,
                evidence_refs: vec![format!("run:{index}")],
            })
            .collect::<Vec<_>>();
        let range = forecast_phase(&scope(), ReleasePhase::Build, &observations).unwrap();
        assert_eq!(range.p50_ms, 30);
        assert_eq!(range.p95_ms, 100);
        assert_eq!(range.method, "empirical_nearest_rank");
    }

    #[test]
    fn release_plan_exposes_critical_path_observation_and_rollback_windows() {
        let plan = build_release_timing_plan(
            scope(),
            vec![
                ("build".into(), ReleasePhase::Build, vec![], 10),
                ("sign".into(), ReleasePhase::Sign, vec!["build".into()], 5),
                (
                    "deploy".into(),
                    ReleasePhase::Deploy,
                    vec!["sign".into()],
                    8,
                ),
                (
                    "observe".into(),
                    ReleasePhase::CanaryObservation,
                    vec!["deploy".into()],
                    20,
                ),
                (
                    "rollback".into(),
                    ReleasePhase::RollbackRecovery,
                    vec!["deploy".into()],
                    7,
                ),
            ],
            Utc::now(),
        )
        .unwrap();
        assert_eq!(plan.critical_path_ms, 43);
        assert_eq!(plan.observation_window_ms, 20);
        assert_eq!(plan.rollback_window_ms, 7);
    }
}
