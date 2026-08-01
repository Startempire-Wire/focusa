use crate::semantic_migration::*;
use crate::semantic_pair::SEMANTIC_PAIR_SCHEMA_VERSION;

fn legacy_bytes() -> Vec<u8> {
    serde_json::to_vec(&LegacySemanticPairV1 {
        schema_version: 1,
        pair_id: "pair-1".into(),
        attempt_id: "attempt-1".into(),
        builder: "builder".into(),
        started_at: "2026-01-01T00:00:00Z".into(),
        project_root: "/project".into(),
        continuity_id: "continuity".into(),
        snapshot_id: "snapshot-1".into(),
        snapshot_hash: "sha256:snapshot".into(),
    })
    .unwrap()
}

#[test]
fn compatibility_read_is_truthful_for_old_current_and_future_versions() {
    let old = compatibility_read(&legacy_bytes()).unwrap();
    assert!(matches!(old, CompatibilityRead::MigrationRequired(_)));

    let plan = plan_v1_migration(&legacy_bytes(), "migration", true).unwrap();
    let current = serde_json::to_vec(&plan.aggregate).unwrap();
    assert!(matches!(
        compatibility_read(&current).unwrap(),
        CompatibilityRead::Current(_)
    ));

    let future = serde_json::json!({
        "schema_version": SEMANTIC_PAIR_SCHEMA_VERSION + 1,
        "unknown_future_payload": { "must_not": "decode" }
    });
    assert!(matches!(
        compatibility_read(&serde_json::to_vec(&future).unwrap()).unwrap(),
        CompatibilityRead::Quarantined(SemanticStoreState::QuarantinedFutureVersion { .. })
    ));
}

#[test]
fn dry_run_produces_receipt_without_claiming_apply() {
    let plan = plan_v1_migration(&legacy_bytes(), "migration-dry", true).unwrap();
    assert!(plan.receipt.dry_run);
    assert!(!plan.receipt.applied);
    assert_eq!(plan.receipt.to_version, SEMANTIC_PAIR_SCHEMA_VERSION);
    assert!(matches!(
        plan.applied_receipt(),
        Err(MigrationError::DryRunCannotApply)
    ));
}

#[test]
fn apply_receipt_and_rollback_are_bounded_to_exact_source() {
    let plan = plan_v1_migration(&legacy_bytes(), "migration-apply", false).unwrap();
    let receipt = plan.applied_receipt().unwrap();
    assert!(receipt.applied);
    assert_eq!(plan.rollback(&receipt).unwrap(), legacy_bytes());

    let mut wrong = receipt;
    wrong.rollback_boundary.push_str("tampered");
    assert!(matches!(
        plan.rollback(&wrong),
        Err(MigrationError::RollbackBoundaryMismatch)
    ));
}

#[test]
fn invalid_version_reports_degraded_instead_of_ready() {
    let bytes = serde_json::to_vec(&serde_json::json!({ "schema_version": 0 })).unwrap();
    assert!(matches!(
        inspect_version(&bytes).unwrap(),
        SemanticStoreState::Degraded { .. }
    ));
}
