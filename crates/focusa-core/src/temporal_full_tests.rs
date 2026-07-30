use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Utc};
use ed25519_dalek::SigningKey;

use crate::{
    temporal::*, temporal_authority::*, temporal_clock::*, temporal_deadline::*,
    temporal_foundation::*, temporal_high_consequence::*, temporal_integrity::*,
    temporal_operations::*, temporal_progress::*,
};

fn scope() -> TemporalScope {
    let mut scope = TemporalScope::project("/workspace/project", "main");
    scope.host_id = Some("host-a".into());
    scope.operator_id = Some("operator-a".into());
    scope.workpoint_id = Some("wp-a".into());
    scope.item_id = Some("item-a".into());
    scope.task_id = Some("task-a".into());
    scope
}

fn clock_sample(id: &str, boot: &str, confidence: TemporalConfidence) -> TemporalClockSample {
    TemporalClockSample {
        sample_id: id.into(),
        domain: TemporalClockDomain::MonotonicActive,
        wall_utc: Utc::now(),
        monotonic_ns: Some(100),
        suspend_aware_ns: Some(100),
        boot_id: Some(boot.into()),
        timezone: "UTC".into(),
        tzdb_version: Some("2026a".into()),
        source: "clock_gettime".into(),
        observed_offset_ns: Some(0),
        measurement_uncertainty_ns: 10,
        confidence,
    }
}

fn uncertainty() -> ClockUncertaintyBudget {
    ClockUncertaintyBudget {
        method: "NIST-TN-1297".into(),
        standard_uncertainty_ns: 10.0,
        expanded_uncertainty_ns: 20.0,
        coverage_factor: 2.0,
        coverage_probability: 0.95,
        offset_ns: 0,
        delay_ns: 1,
        jitter_ns: 1,
        dispersion_ns: 1,
        root_distance_ns: 3,
        frequency_error_ppb: 0.1,
        sample_age_ms: 1,
        calibration_lineage: vec!["calibration:host-network-adapter-provider".into()],
    }
}

fn lineage() -> TemporalVersionLineage {
    TemporalVersionLineage {
        schema_version: "temporal.v1".into(),
        policy_version: "policy.v1".into(),
        adapter_version: "adapter.v1".into(),
        calendar_version: Some("gregorian.v1".into()),
        tzdb_version: Some("2026a".into()),
        estimator_version: Some("estimator.v1".into()),
        clock_profile_version: "clock.v1".into(),
    }
}

#[test]
fn exact_scope_requires_every_applicable_dimension() {
    let required = RequiredTemporalScope {
        host: true,
        operator: true,
        project: true,
        continuity: true,
        workpoint: true,
        item: true,
        task: true,
    };
    assert_eq!(validate_exact_scope(&scope(), &required), Ok(()));
    let mut missing = scope();
    missing.task_id = None;
    assert_eq!(
        validate_exact_scope(&missing, &required),
        Err(TemporalScopeError::MissingTask)
    );
}

#[test]
fn reboot_suspend_and_wall_discontinuity_never_become_false_exact_elapsed() {
    let pair = ClockSamplePair {
        before: clock_sample("a", "boot-a", TemporalConfidence::Verified),
        after: clock_sample("b", "boot-b", TemporalConfidence::Verified),
        elapsed_lower_ns: 100,
        elapsed_upper_ns: None,
        uncertainty: uncertainty(),
        crosses_boot_epoch: true,
        crosses_suspend: true,
        lineage: lineage(),
    };
    assert_eq!(
        validate_clock_sample_pair(&pair),
        Err(ClockSamplePairError::MissingCrossEpochBound)
    );
}

#[test]
fn source_spoof_replay_disagreement_and_unsettled_quarantine_fail_closed() {
    let profile = ClockTrustProfile {
        profile_id: "trusted".into(),
        required_source_count: 2,
        required_independent_source_count: 2,
        required_authentication: ClockAuthenticationPolicy::Nts,
        disagreement_threshold_ns: 10,
        max_sync_age_ms: 100,
        max_holdover_ms: 100,
        max_offset_ns: 100,
        max_root_distance_ns: 100,
        on_disagreement: ClockDisagreementAction::Block,
    };
    let source = |id: &str, class: &str, replay: bool| ClockSourceObservation {
        source_id: id.into(),
        diversity_class: class.into(),
        authenticated: true,
        replay_protected: replay,
        request_response_bound: true,
        observed_at: Utc::now(),
        synchronization_age_ms: 1,
        offset_ns: 0,
        delay_ns: 1,
        jitter_ns: 1,
        dispersion_ns: 1,
        root_distance_ns: 2,
        frequency_error_ppb: 1,
        status: ClockSourceStatus::Healthy,
        quarantine_reason: None,
        recovery_evidence_refs: vec![],
    };
    assert_eq!(
        evaluate_clock_sources(
            &profile,
            &[source("a", "one", true), source("b", "two", false)]
        ),
        Err(ClockSourceTrustError::ReplayProtectionMissing)
    );
}

#[test]
fn dst_fold_gap_and_material_tzdb_change_require_versioned_resolution_receipts() {
    let intent = CivilTimeIntent {
        intent_id: "civil".into(),
        original_expression: "first 01:30 after DST fallback".into(),
        timezone: "America/Los_Angeles".into(),
        tzdb_version: "2026a".into(),
        calendar: "gregorian".into(),
        calendar_version: "v1".into(),
        jurisdiction: Some("US-CA".into()),
        jurisdiction_rule_version: Some("2026".into()),
        fold_policy: "first".into(),
        gap_policy: "reject".into(),
        recurrence_rule: None,
        floating: false,
        resolved_instants: vec![Utc::now()],
        resolution_receipt_refs: vec![],
        supersedes_resolution_ref: None,
    };
    assert_eq!(
        validate_civil_time_intent(&intent),
        Err(DeadlineError::MissingCivilTimeVersion)
    );
}

#[test]
fn uncertainty_crossing_deadline_is_never_reported_on_time() {
    let now = Utc::now();
    let contract = DeadlineContract {
        contract_id: "d".into(),
        scope: scope(),
        kind: DeadlineContractKind::ExternalDeadline,
        readiness_at: Some(now - Duration::minutes(1)),
        deadline_at: now,
        boundary_policy: DeadlineBoundaryPolicy::Inclusive,
        source_authority: "operator".into(),
        immutable_external_boundary: true,
        inheritance_source_ref: None,
        working_window_ref: None,
        conflict_refs: vec![],
        uncertainty: None,
        revision: 1,
        reducer_receipt_ref: "receipt".into(),
        cas_token: "cas".into(),
    };
    assert_eq!(
        compare_deadline(
            &contract,
            Some(now - Duration::seconds(1)),
            Some(now + Duration::seconds(1))
        ),
        DeadlineComparison::PossiblyCrossed
    );
}

#[test]
fn cancellation_race_and_retry_preserve_original_parent_budget() {
    let budget = ChildTimeoutBudget {
        budget_id: "child".into(),
        parent_budget_ref: "parent".into(),
        original_deadline_monotonic_ns: 1_000,
        dispatched_at_monotonic_ns: 100,
        remaining_ns: 900,
        elapsed_deducted_ns: 100,
        retry_count: 1,
        cancellation_requested: true,
        cancellation_acknowledged: true,
        cancellation_effective: false,
        possible_effect_requires_reconciliation: true,
    };
    assert_eq!(
        validate_child_budget(&budget),
        Err(ChildBudgetError::RetryDeadlineReset)
    );
}

#[test]
fn stale_uncertain_out_of_scope_market_dispatch_is_blocked() {
    let policy = DispatchAgePolicy {
        maximum_clock_uncertainty_ns: 10,
        maximum_market_data_age_ms: 10,
        maximum_decision_age_ms: 10,
        maximum_dispatch_age_ms: 10,
        risk_limit_policy_ref: "risk:v1".into(),
    };
    let observation = DispatchAgeObservation {
        clock_uncertainty_ns: 11,
        market_data_age_ms: 0,
        decision_age_ms: 0,
        dispatch_age_ms: 0,
        in_scope: false,
        within_risk_limits: false,
    };
    assert_eq!(
        authorize_dispatch(&policy, &observation),
        Err(HighConsequenceError::UncertaintyExceeded)
    );
}

#[test]
fn alert_overload_obeys_backpressure_and_notification_budget() {
    let policy = TemporalPulsePolicy {
        policy_id: "pulse".into(),
        minimum_dwell_ms: 1,
        debounce_ms: 1,
        hysteresis_ms: 1,
        maximum_notifications_per_hour: 4,
        maximum_pending_notifications: 1,
        protected_focus: false,
        safety_authority_immutable: true,
    };
    let state = TemporalPulseState {
        pending_notifications: 1,
        urgency_level: 4,
        ..Default::default()
    };
    assert_eq!(
        temporal_pulse_decision(&policy, &state, Utc::now()),
        PulseDecision::SuppressForBackpressure
    );
}

#[test]
fn temporal_event_signature_detects_tampering_and_preserves_chain_authority() {
    let signing_key = SigningKey::from_bytes(&[42; 32]);
    let mut event = TemporalEvent {
        event_id: "event".into(),
        sequence: 1,
        event_kind: TemporalEventKind::DeadlineCompared,
        scope: scope(),
        claim: None,
        clock_sample: None,
        metadata: BTreeMap::new(),
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: "key".into(),
        digest: String::new(),
    };
    sign_temporal_event(&mut event, "host-key", &signing_key);
    assert_eq!(
        verify_temporal_event_signature(&event, Some("host-key")),
        Ok(())
    );
    event
        .metadata
        .insert("tampered".into(), serde_json::json!(true));
    assert_eq!(
        verify_temporal_event_signature(&event, Some("host-key")),
        Err(TemporalIntegrityError::DigestMismatch)
    );
}

#[test]
fn live_activation_requires_complete_evidence_and_explicit_approval() {
    let firewall = ActivationFirewall {
        current_level: ActivationLevel::Canary,
        requested_level: ActivationLevel::Live,
        requirement_refs: vec!["req:a".into(), "req:b".into()],
        evidence_refs: vec!["evidence:a".into()],
        approval_receipt_refs: vec![],
        deterministic_loop_has_llm: false,
    };
    assert_eq!(
        authorize_activation(&firewall),
        Err(HighConsequenceError::ActivationFirewallOpen)
    );
}

#[test]
fn closure_keeps_factual_completion_separate_from_disposition_and_temporal_failure() {
    let posture = ClosureTemporalPosture {
        factual_status: "accepted_risk".into(),
        operator_disposition: Some("accepted".into()),
        amendment_ref: None,
        degraded_posture: Some("late".into()),
        rollup_eligible: true,
        temporal_failure: true,
        spec131_closure_ref: "closure:131".into(),
        spec137_temporal_refs: vec!["breach:137".into()],
        receipt_refs: vec!["receipt".into()],
    };
    assert_eq!(
        validate_closure_posture(&posture),
        Err(ClosurePostureError::NonCompletionMasqueradesAsCompletion)
    );
}

#[test]
fn signed_control_requires_distinct_source_auth_and_ledger_integrity_tests() {
    let controls = SignedTemporalLedgerControl {
        signed_event_kinds: BTreeSet::from(["clock_sample".into(), "deadline".into()]),
        hash_chain_verified: true,
        source_authentication_test_refs: vec!["test:auth".into()],
        ledger_integrity_test_refs: vec![],
    };
    assert_eq!(
        validate_ledger_controls(&controls),
        Err(HighConsequenceError::LedgerIntegrityMissing)
    );
}
