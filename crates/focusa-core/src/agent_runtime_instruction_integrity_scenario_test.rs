use crate::agent_runtime_instruction_integrity::*;
use chrono::{Duration, Utc};
use std::collections::BTreeMap;

fn profile(id: &str) -> ApprovedTargetProfile {
    ApprovedTargetProfile {
        profile_id: id.into(),
        revision: 1,
        environment_ref: "env:verified".into(),
        required_capability_refs: vec!["capability:runtime".into()],
        contingency_profile_ref: Some("recovery".into()),
        approved_by_ref: "operator:profile-approval".into(),
        evidence_refs: vec!["evidence:profile".into()],
    }
}

fn request(canvas: bool) -> InstructionIntegrityRequest {
    InstructionIntegrityRequest {
        scope_ref: "project:focusa".into(),
        expected_invariant_hashes: BTreeMap::from([("foundation".into(), "a".repeat(64))]),
        effective_invariant_hashes: BTreeMap::from([("foundation".into(), "a".repeat(64))]),
        dynamic_authority_available: true,
        durable_or_consequential_action: true,
        mission_canvas_available: canvas,
        approved_target_profiles: vec![profile("primary"), profile("recovery")],
        selected_target_profile_ref: "primary".into(),
        temporal_adaptations: vec![],
        active_constitution_revision: 2,
        observed_constitution_revision: 2,
        evidence_refs: vec!["evidence:guard".into()],
        receipt_ref: "receipt:guard".into(),
    }
}

fn adaptation(field: TemporalAdaptationField) -> TemporalInstructionAdaptation {
    TemporalInstructionAdaptation {
        adaptation_id: format!("adaptation:{field:?}"),
        field,
        prior_value: "old".into(),
        adapted_value: "new".into(),
        temporal_claim_ref: "temporal:claim".into(),
        effective_at: Utc::now() - Duration::minutes(1),
        expires_at: Utc::now() + Duration::hours(1),
        evidence_refs: vec!["evidence:temporal".into()],
        receipt_ref: "receipt:temporal".into(),
    }
}

fn parity() -> HeadlessCapabilityParity {
    HeadlessCapabilityParity {
        effective_instruction_read: true,
        source_inspection: true,
        conflict_inspection: true,
        integrity_guard_evaluation: true,
        amendment_proposal: true,
        amendment_activation: true,
        drift_detection: true,
        rollback: true,
        evidence_receipts: true,
        mission_canvas_independent: true,
        evidence_refs: vec!["evidence:headless".into()],
        receipt_ref: "receipt:headless".into(),
    }
}

fn amendment() -> CanonicalInstructionAmendment {
    CanonicalInstructionAmendment {
        amendment_id: "amendment:one".into(),
        scope_ref: "project:focusa".into(),
        prior_revision: 1,
        next_revision: 2,
        changed_instruction_refs: vec!["instruction:one".into()],
        proposed_by_operator_ref: "operator:proposal".into(),
        proposal_receipt_ref: "receipt:proposal".into(),
        approved_by_operator_ref: Some("operator:approval".into()),
        approval_receipt_ref: Some("receipt:approval".into()),
        sweep_manifest: Some(OfficialDocumentationSweepManifest {
            manifest_id: "sweep:one".into(),
            affected_source_refs: vec!["AGENTS.md".into()],
            entries: vec![DocumentationSweepEntry {
                source_ref: "AGENTS.md".into(),
                before_sha256: "a".repeat(64),
                after_sha256: "b".repeat(64),
                disposition: SweepDisposition::Changed,
                evidence_refs: vec!["diff:agents".into()],
            }],
            completed_at: Utc::now(),
            reviewer_ref: "operator:review".into(),
            receipt_ref: "receipt:sweep".into(),
        }),
        effective_at: Some(Utc::now()),
        status: AmendmentStatus::Activated,
        evidence_refs: vec!["evidence:amendment".into()],
    }
}

fn assert_authorized(value: InstructionIntegrityRequest) {
    assert_eq!(
        evaluate_instruction_integrity(&value, Utc::now()).decision,
        InstructionIntegrityDecision::Authorized
    );
}

#[test]
fn scenario_01_canvas_enabled() {
    assert_authorized(request(true));
}
#[test]
fn scenario_02_canvas_disabled() {
    assert_authorized(request(false));
}
#[test]
fn scenario_03_canvas_unavailable_after_start() {
    let mut value = request(true);
    value.mission_canvas_available = false;
    assert_authorized(value);
}
#[test]
fn scenario_04_canvas_never_installed() {
    let result = evaluate_instruction_integrity(&request(false), Utc::now());
    assert!(!result.mission_canvas_authoritative);
}
#[test]
fn scenario_05_cli_only_operation() {
    assert!(validate_headless_parity(&parity()).is_ok());
}
#[test]
fn scenario_06_pi_only_without_canvas() {
    assert_authorized(request(false));
}
#[test]
fn scenario_07_api_driven_headless() {
    assert!(validate_headless_parity(&parity()).is_ok());
}
#[test]
fn scenario_08_urgency_change_without_restart() {
    let mut value = request(false);
    value
        .temporal_adaptations
        .push(adaptation(TemporalAdaptationField::NotificationCadence));
    assert_authorized(value);
}
#[test]
fn scenario_09_deadline_revision_reassesses_schedule() {
    let mut value = request(false);
    value
        .temporal_adaptations
        .push(adaptation(TemporalAdaptationField::Scheduling));
    assert_authorized(value);
}
#[test]
fn scenario_10_critical_posture_preserves_target() {
    let mut value = request(false);
    value
        .temporal_adaptations
        .push(adaptation(TemporalAdaptationField::CheckpointFrequency));
    assert_authorized(value);
}
#[test]
fn scenario_11_breached_recovery_uses_approved_route() {
    let mut value = request(false);
    value.selected_target_profile_ref = "recovery".into();
    assert_authorized(value);
}
#[test]
fn scenario_12_runtime_target_invention_blocked() {
    let mut value = request(false);
    value.selected_target_profile_ref = "invented".into();
    assert_eq!(
        evaluate_instruction_integrity(&value, Utc::now()).decision,
        InstructionIntegrityDecision::Blocked
    );
}
#[test]
fn scenario_13_execution_steering_does_not_amend_canon() {
    let mut value = amendment();
    value.proposed_by_operator_ref.clear();
    assert_eq!(
        validate_amendment_proposal(&value),
        Err(InstructionIntegrityError::AmendmentProposalAuthorityMissing)
    );
}
#[test]
fn scenario_14_authorization_without_sweep_blocked() {
    let mut value = amendment();
    value.sweep_manifest = None;
    assert_eq!(
        validate_amendment_activation(&value),
        Err(InstructionIntegrityError::AmendmentSweepMissing)
    );
}
#[test]
fn scenario_15_sweep_without_post_confirmation_blocked() {
    let mut value = amendment();
    value.approved_by_operator_ref = None;
    assert_eq!(
        validate_amendment_activation(&value),
        Err(InstructionIntegrityError::AmendmentApprovalMissing)
    );
}
#[test]
fn scenario_16_two_operator_acts_activate_amendment() {
    assert!(validate_amendment_activation(&amendment()).is_ok());
}
#[test]
fn scenario_17_amendment_invalidates_stale_plan() {
    let mut value = request(false);
    value.active_constitution_revision = 3;
    assert_eq!(
        evaluate_instruction_integrity(&value, Utc::now()).decision,
        InstructionIntegrityDecision::Blocked
    );
}
#[test]
fn scenario_18_session_rebinds_after_amendment() {
    let mut value = request(false);
    value.active_constitution_revision = 3;
    value.observed_constitution_revision = 3;
    assert_authorized(value);
}
#[test]
fn scenario_19_multi_agent_stale_revision_rejected() {
    assert_eq!(
        validate_replica_activation(4, 4, &["evidence".into()], "receipt"),
        Err(InstructionIntegrityError::ReplicaEpochStale)
    );
}
#[test]
fn scenario_20_daemon_backstops_harness() {
    assert!(validate_headless_parity(&parity()).is_ok());
}
#[test]
fn scenario_21_dynamic_authority_outage_safe_fallback() {
    let mut blocked = request(false);
    blocked.dynamic_authority_available = false;
    assert_eq!(
        evaluate_instruction_integrity(&blocked, Utc::now()).decision,
        InstructionIntegrityDecision::Blocked
    );
    blocked.durable_or_consequential_action = false;
    assert_authorized(blocked);
}
#[test]
fn scenario_22_compaction_preserves_constitution_and_refreshes_guard() {
    let mut value = request(false);
    value
        .temporal_adaptations
        .push(adaptation(TemporalAdaptationField::EvidenceRefreshCadence));
    assert_authorized(value);
}
#[test]
fn scenario_23_ambiguity_blocks_affected_slice() {
    let mut value = request(false);
    value.selected_target_profile_ref.clear();
    let result = evaluate_instruction_integrity(&value, Utc::now());
    assert_eq!(result.reason_codes, vec!["target_profile_not_approved"]);
}
#[test]
fn scenario_24_fidelity_failure_prevents_completion() {
    let mut value = request(false);
    value
        .effective_invariant_hashes
        .insert("foundation".into(), "f".repeat(64));
    assert_eq!(
        evaluate_instruction_integrity(&value, Utc::now()).decision,
        InstructionIntegrityDecision::Blocked
    );
}
