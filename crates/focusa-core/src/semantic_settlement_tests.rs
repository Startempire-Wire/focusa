use super::*;

fn input() -> SettlementInput {
    SettlementInput {
        contract_revision: 2,
        verified_contract_revision: 2,
        workpoint_revision: 3,
        verified_workpoint_revision: 3,
        final_snapshot_hash: "sha256:snapshot".into(),
        verified_snapshot_hash: "sha256:snapshot".into(),
        mandatory_requirement_ids: BTreeSet::from(["REQ-1".into(), "REQ-2".into()]),
        passed_requirement_ids: BTreeSet::from(["REQ-1".into(), "REQ-2".into()]),
        evidence: vec![SettlementEvidence {
            evidence_ref: "evidence:all".into(),
            requirement_ids: BTreeSet::from(["REQ-1".into(), "REQ-2".into()]),
            fresh: true,
            validation_receipt_ref: Some("receipt:validation".into()),
        }],
        verifier_calibration_valid: true,
        verifier_eligible: true,
        verifier_independent: true,
        temporal_authority_valid: true,
        epistemic_authority_valid: true,
        pack_conflicts: BTreeSet::new(),
        required_approval_ids: BTreeSet::new(),
        received_approval_ids: BTreeSet::new(),
        migration_verified: true,
        client_parity_verified: true,
        receipt_ready: true,
        runtime_variance_ids: BTreeSet::new(),
        partial_settlement_allowed: false,
    }
}

#[test]
fn full_settlement_requires_every_mandatory_requirement_and_all_authorities() {
    let result = evaluate_settlement(&input());
    assert_eq!(result.status, SettlementStatus::SettledFull);
    assert!(result.closure_allowed);
    assert!(result.unsettled_requirement_ids.is_empty());
    assert!(result.blocker_codes.is_empty());
}

#[test]
fn partial_settlement_names_unsettled_requirements_and_never_closes_parent() {
    let mut input = input();
    input.passed_requirement_ids.remove("REQ-2");
    input.partial_settlement_allowed = true;
    let result = evaluate_settlement(&input);
    assert_eq!(result.status, SettlementStatus::SettledPartial);
    assert_eq!(
        result.settled_requirement_ids,
        BTreeSet::from(["REQ-1".into()])
    );
    assert_eq!(
        result.unsettled_requirement_ids,
        BTreeSet::from(["REQ-2".into()])
    );
    assert!(!result.closure_allowed);
}

#[test]
fn changed_revisions_stale_evidence_variance_and_pack_conflicts_block() {
    let mut input = input();
    input.contract_revision += 1;
    input.workpoint_revision += 1;
    input.final_snapshot_hash = "sha256:changed".into();
    input.evidence[0].fresh = false;
    input.pack_conflicts.insert("pack:conflict".into());
    input.runtime_variance_ids.insert("variance:1".into());
    let result = evaluate_settlement(&input);
    assert_eq!(result.status, SettlementStatus::Blocked);
    for code in [
        "contract_revision_changed",
        "workpoint_revision_changed",
        "snapshot_changed",
        "evidence_unfresh_or_unvalidated",
        "pack_conflict",
        "runtime_variance",
    ] {
        assert!(result.blocker_codes.contains(&code.to_string()), "{code}");
    }
    assert!(!result.closure_allowed);
}

#[test]
fn missing_approval_is_operator_required_and_false_completion_checks_block() {
    let mut approval = input();
    approval
        .required_approval_ids
        .insert("approval:release".into());
    assert_eq!(
        evaluate_settlement(&approval).status,
        SettlementStatus::OperatorRequired
    );
    let mut blocked = input();
    blocked.verifier_calibration_valid = false;
    blocked.verifier_eligible = false;
    blocked.verifier_independent = false;
    blocked.temporal_authority_valid = false;
    blocked.epistemic_authority_valid = false;
    blocked.migration_verified = false;
    blocked.client_parity_verified = false;
    blocked.receipt_ready = false;
    let result = evaluate_settlement(&blocked);
    assert_eq!(result.status, SettlementStatus::Blocked);
    assert_eq!(result.blocker_codes.len(), 8);
}
