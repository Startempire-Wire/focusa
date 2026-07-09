//! Smoke tests for focusa-bench crate.

use super::*;
use serde_json::json;

#[test]
fn canonical_pool_has_150_tasks() {
    let pool = TaskPool::canonical();
    assert_eq!(pool.count(), 150, "150 tasks expected");
    assert_eq!(pool.public_count(), 78, "78 public tasks (13 per kind * 6 kinds = 78)");
    assert_eq!(pool.private_count(), 72, "72 private tasks (12 per kind * 6 kinds = 72)");
}

#[test]
fn canonical_model_matrix_has_six_models() {
    let matrix = ModelMatrix::canonical();
    assert!(matrix.count() >= 4, "Expected 4+ models in canonical matrix");
    assert!(matrix.get("claude-sonnet-4").is_some());
    assert!(matrix.get("gpt-4o").is_some());
}

#[test]
fn arm_configs_have_correct_attributes() {
    let no_focusa = ArmConfig::for_arm(Arm::NoFocusa);
    assert!(!no_focusa.focusa_tools_registered);
    assert!(!no_focusa.emit_focusa_agent_prompt);

    let full_focusa = ArmConfig::for_arm(Arm::FullFocusa);
    assert!(full_focusa.focusa_tools_registered);
    assert!(full_focusa.emit_focusa_agent_prompt);
    assert!(full_focusa.focusa_workpoint_required);
    assert!(full_focusa.evidence_chain_required);
}

#[test]
fn agent_power_index_computes_correctly() {
    let outcomes = vec![(true, 1u8), (true, 2), (false, 3), (true, 4), (false, 5)];
    let api = AgentPowerIndex::from_outcomes(&outcomes);
    assert_eq!(api.tasks_attempted, 5);
    assert_eq!(api.tasks_completed, 3);
    assert!((api.success_rate - 0.6).abs() < 0.01);
    // weighted_score = (1+2+0+4+0) / (1+2+3+4+5) = 7/15 ≈ 0.467
    assert!((api.weighted_score - 0.467).abs() < 0.01);
}

#[test]
fn focusa_uplift_score_detects_helps() {
    let full = vec![true, true, true, false, false];
    let baseline = vec![true, false, false, false, false];
    let uplift = FocusaUpliftScore::from_comparison(&full, &baseline);
    assert_eq!(uplift.tasks_helped, 2);  // indices 1, 2
    assert_eq!(uplift.tasks_hurt, 0);
    assert_eq!(uplift.tasks_neutral, 3);
}

#[test]
fn pass_at_n_known_value() {
    // 5 total, 3 successful, n=1: 1 - C(2,1)/C(5,1) = 1 - 2/5 = 0.6
    let p = PassAtN::from_observations(1, 3, 5);
    assert!((p.pass_at_n - 0.6).abs() < 0.01);
}

#[test]
fn time_horizon_median_and_p95() {
    let times = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let th = TimeHorizon::from_observations(&times);
    assert_eq!(th.tasks_measured, 10);
    assert!(th.median_seconds > 4.0 && th.median_seconds < 7.0);
    assert!(th.p95_seconds > 8.0);
}

#[test]
fn eval_ledger_hash_chain_works() {
    let mut ledger = EvalLedger::new();
    let e1 = ledger.append(LedgerKind::Run, "run-1", "full_focusa", "claude-sonnet-4", json!({"task": "t1"}));
    assert_eq!(e1.prev_hash, "genesis");
    let e2 = ledger.append(LedgerKind::Event, "run-1", "full_focusa", "claude-sonnet-4", json!({"event": "started"}));
    assert_eq!(e2.prev_hash, e1.entry_hash, "e2 should chain from e1");
    let e3 = ledger.append(LedgerKind::Complete, "run-1", "full_focusa", "claude-sonnet-4", json!({"passed": 78}));
    assert_eq!(e3.prev_hash, e2.entry_hash, "e3 should chain from e2");
    assert_eq!(ledger.entry_count(), 3);
    // Verify chain integrity: each entry's prev_hash matches the previous entry's entry_hash.
    let entries: Vec<&LedgerEntry> = ledger.entries.values().collect();
    assert_eq!(entries[0].prev_hash, "genesis");
    assert_eq!(entries[1].prev_hash, entries[0].entry_hash);
    assert_eq!(entries[2].prev_hash, entries[1].entry_hash);
}

#[test]
fn groundedness_score_computes() {
    let g = GroundednessScore::from_observations(100, 85);
    assert_eq!(g.claims_total, 100);
    assert!((g.score - 0.85).abs() < 0.01);
}

#[test]
fn snapshot_can_add_claim() {
    let mut snap = PublicSnapshot::new("test snapshot", json!({"agent_power_index": 0.65}));
    snap.add_rule("$.api_key", snapshot::RedactionStrategy::Mask);
    let claim = snap.claim("agent_power_index", 0.65, 100);
    assert_eq!(claim.value, 0.65);
    assert_eq!(claim.n, 100);
}

#[test]
fn benchmark_report_adds_claims() {
    let mut report = BenchmarkReport::new("Test Report");
    report.add_claim("agent_power_index", 0.7, 0.6, 0.8, 100);
    report.add_claim("uplift_score", 0.15, 0.10, 0.20, 50);
    assert_eq!(report.claims.len(), 2);
}
