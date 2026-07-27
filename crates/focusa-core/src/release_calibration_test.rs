use super::*;
use crate::release_cycle::ReleaseStageTiming;

fn benchmark(elapsed: u64, queue: u64, retry: u64, flow: f64, first_pass: f64) -> ReleaseBenchmark {
    ReleaseBenchmark {
        total_elapsed_ms: elapsed,
        useful_work_ms: elapsed.saturating_sub(queue + retry),
        queue_ms: queue,
        retry_ms: retry,
        human_interventions: u32::from(retry > 0),
        retries: u32::from(retry > 0),
        first_pass_gate_success_rate: first_pass,
        flow_efficiency: flow,
        critical_path: vec!["build".into()],
        missed_target_reason_codes: vec![],
        stages: vec![ReleaseStageTiming {
            stage: ReleaseStage::Built,
            started_at: "2026-01-01T00:00:00Z".into(),
            completed_at: Some("2026-01-01T00:01:00Z".into()),
            elapsed_ms: elapsed / 2,
            queue_ms: queue,
            retry_ms: retry,
            useful_work_ms: elapsed.saturating_sub(queue + retry) / 2,
        }],
    }
}

fn observation(
    id: &str,
    tuning: &str,
    elapsed: u64,
    queue: u64,
    retry: u64,
    flow: f64,
    first_pass: f64,
) -> ReleaseCalibrationObservation {
    ReleaseCalibrationObservation {
        schema: RELEASE_CALIBRATION_OBSERVATION_SCHEMA.into(),
        release_id: id.into(),
        project_id: "project".into(),
        profile: "profile".into(),
        exact_sha: format!("sha-{id}"),
        observed_at: "2026-01-01T00:00:00Z".into(),
        applied_tuning_id: tuning.into(),
        benchmark: benchmark(elapsed, queue, retry, flow, first_pass),
        token_cost: 100,
        monetary_cost_microunits: elapsed,
        evidence_refs: vec![format!("run:{id}")],
    }
}

#[test]
fn two_baselines_propose_a_real_next_cycle_change() {
    let history = vec![
        observation("r1", "baseline", 1000, 50, 0, 0.50, 1.0),
        observation("r2", "baseline", 950, 40, 0, 0.55, 1.0),
    ];
    let decision = ReleaseCalibrator::decide(
        &history,
        &ReleasePlanTuning::default(),
        &ReleaseCalibrationPolicy::default(),
    )
    .unwrap();
    assert_eq!(decision.outcome, CalibrationOutcome::Proposed);
    assert_eq!(decision.next_tuning.max_parallel_operations, 2);
    assert_eq!(
        decision.next_tuning.strategy,
        "parallelize_independent_topology_waves"
    );
    assert_ne!(decision.next_tuning.tuning_id, "baseline");
}

#[test]
fn successful_experiment_is_promoted_before_next_proposal() {
    let active = ReleasePlanTuning {
        tuning_id: "experiment-1".into(),
        max_parallel_operations: 2,
        reuse_exact_sha_evidence: true,
        preflight_before_immutable_tag: true,
        priority_stage: Some(ReleaseStage::Built),
        strategy: "parallelize_independent_topology_waves".into(),
    };
    let history = vec![
        observation("r1", "baseline", 1000, 50, 0, 0.60, 0.99),
        observation("r2", "experiment-1", 800, 40, 0, 0.82, 1.0),
    ];
    let decision = ReleaseCalibrator::decide(
        &history,
        &active,
        &ReleaseCalibrationPolicy {
            min_samples_for_experiment: 3,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(decision.outcome, CalibrationOutcome::Promoted);
    assert!(decision.elapsed_change_percent.unwrap() < 0.0);
    assert_eq!(decision.next_tuning, active);
}

#[test]
fn regressing_experiment_rolls_back() {
    let active = ReleasePlanTuning {
        tuning_id: "experiment-1".into(),
        max_parallel_operations: 4,
        reuse_exact_sha_evidence: true,
        preflight_before_immutable_tag: true,
        priority_stage: None,
        strategy: "parallelize".into(),
    };
    let history = vec![
        observation("r1", "baseline", 1000, 50, 0, 0.80, 1.0),
        observation("r2", "experiment-1", 1200, 50, 0, 0.70, 0.90),
    ];
    let decision = ReleaseCalibrator::decide(
        &history,
        &active,
        &ReleaseCalibrationPolicy {
            min_samples_for_experiment: 3,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(decision.outcome, CalibrationOutcome::RolledBack);
    assert_eq!(decision.next_tuning, ReleasePlanTuning::default());
}

#[test]
fn queue_contention_reduces_parallelism() {
    let active = ReleasePlanTuning {
        tuning_id: "current".into(),
        max_parallel_operations: 4,
        reuse_exact_sha_evidence: true,
        preflight_before_immutable_tag: true,
        priority_stage: None,
        strategy: "parallel".into(),
    };
    let history = vec![
        observation("r1", "baseline", 1000, 300, 0, 0.85, 1.0),
        observation("r2", "baseline", 1000, 350, 0, 0.85, 1.0),
    ];
    let decision =
        ReleaseCalibrator::decide(&history, &active, &ReleaseCalibrationPolicy::default()).unwrap();
    assert_eq!(decision.next_tuning.max_parallel_operations, 3);
    assert_eq!(
        decision.next_tuning.strategy,
        "reduce_runner_queue_contention"
    );
}

#[test]
fn mixed_project_history_is_rejected() {
    let mut other = observation("r2", "baseline", 900, 0, 0, 0.9, 1.0);
    other.project_id = "other".into();
    let error = ReleaseCalibrator::decide(
        &[observation("r1", "baseline", 1000, 0, 0, 0.8, 1.0), other],
        &ReleasePlanTuning::default(),
        &ReleaseCalibrationPolicy::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("crosses project/profile"));
}

#[test]
fn append_only_ledger_filters_typed_scope() {
    let path = std::env::temp_dir().join(format!("focusa-calibration-{}.jsonl", Uuid::now_v7()));
    let one = observation("r1", "baseline", 1000, 0, 0, 0.8, 1.0);
    let mut other = observation("r2", "baseline", 900, 0, 0, 0.9, 1.0);
    other.project_id = "other".into();
    ReleaseCalibrationLedger::append(&path, &one).unwrap();
    ReleaseCalibrationLedger::append(&path, &other).unwrap();
    let scoped = ReleaseCalibrationLedger::read(&path, "project", "profile").unwrap();
    assert_eq!(scoped, vec![one]);
    std::fs::remove_file(path).unwrap();
}
