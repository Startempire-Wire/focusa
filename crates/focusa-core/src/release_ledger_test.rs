use super::*;
use crate::{
    release_calibration::ReleasePlanTuning,
    release_cycle::{RELEASE_CANDIDATE_SCHEMA, ReleaseStage},
    release_orchestrator::{RELEASE_PLAN_SCHEMA, ReleaseInvocationSurface},
};

fn candidate(stage: ReleaseStage) -> ReleaseCandidate {
    ReleaseCandidate {
        schema: RELEASE_CANDIDATE_SCHEMA.into(),
        candidate_id: "candidate-1".into(),
        project_root: "/project".into(),
        continuity_id: "release".into(),
        workpoint_id: "work".into(),
        version: "1".into(),
        exact_sha: "0123456789abcdef".into(),
        topology_ref: "topology".into(),
        stage,
        locked_scope_refs: vec!["scope:1".into()],
        evidence: vec![],
        admitted_fixes: vec![],
        benchmark: None,
    }
}

fn plan() -> ReleaseExecutionPlan {
    ReleaseExecutionPlan {
        schema: RELEASE_PLAN_SCHEMA.into(),
        candidate_id: "candidate-1".into(),
        exact_sha: "0123456789abcdef".into(),
        adapter_id: "adapter".into(),
        invocation_surface: ReleaseInvocationSurface::Headless,
        stages: vec![ReleaseStage::Locked],
        surface_waves: vec![],
        reused_stages: vec![],
        mutating_stages: vec![],
        tuning: ReleasePlanTuning::default(),
    }
}

fn checkpoint(sequence: u64, stage: ReleaseStage) -> ReleaseRunCheckpoint {
    ReleaseRunCheckpoint {
        schema: RELEASE_CHECKPOINT_SCHEMA.into(),
        sequence,
        status: "running".into(),
        observed_at: "2026-01-01T00:00:00Z".into(),
        candidate: candidate(stage),
        plan: plan(),
        receipts: vec![],
        blocked_reasons: vec![],
    }
}

#[test]
fn append_only_ledger_resumes_from_latest_exact_candidate() {
    let path = std::env::temp_dir().join(format!(
        "focusa-release-ledger-{}.jsonl",
        uuid::Uuid::now_v7()
    ));
    let ledger =
        JsonlReleaseRunLedger::new(&path, "/project", "candidate-1", "0123456789abcdef").unwrap();
    ledger.append(&checkpoint(0, ReleaseStage::Plan)).unwrap();
    ledger.append(&checkpoint(1, ReleaseStage::Locked)).unwrap();
    assert_eq!(ledger.next_sequence().unwrap(), 2);
    assert_eq!(
        ledger.latest().unwrap().unwrap().candidate.stage,
        ReleaseStage::Locked
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn ledger_rejects_sequence_and_scope_forks() {
    let path = std::env::temp_dir().join(format!(
        "focusa-release-ledger-{}.jsonl",
        uuid::Uuid::now_v7()
    ));
    let ledger =
        JsonlReleaseRunLedger::new(&path, "/project", "candidate-1", "0123456789abcdef").unwrap();
    assert!(
        ledger
            .append(&checkpoint(2, ReleaseStage::Plan))
            .unwrap_err()
            .to_string()
            .contains("sequence")
    );
    let wrong =
        JsonlReleaseRunLedger::new(&path, "/other", "candidate-1", "0123456789abcdef").unwrap();
    assert!(
        wrong
            .append(&checkpoint(0, ReleaseStage::Plan))
            .unwrap_err()
            .to_string()
            .contains("project")
    );
}

#[test]
fn ledger_path_must_be_absolute() {
    let error = match JsonlReleaseRunLedger::new(
        "relative.jsonl",
        "/project",
        "candidate-1",
        "0123456789abcdef",
    ) {
        Ok(_) => panic!("relative ledger unexpectedly accepted"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("absolute"));
}
