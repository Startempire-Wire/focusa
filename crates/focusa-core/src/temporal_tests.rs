use super::*;
use crate::temporal_clock::*;
use crate::temporal_operations::{
    HumanCalendarContext, TemporalExecutionGuard, TemporalPriorityFrame, authorize_temporal_action,
};

fn claim(kind: TemporalClaimKind) -> TemporalClaim {
    let now = Utc::now();
    TemporalClaim {
        claim_id: "claim-1".into(),
        revision: 1,
        scope: TemporalScope::project("/workspace/project", "main"),
        kind,
        status: TemporalClaimStatus::Proposed,
        subject_ref: "release".into(),
        target_at: None,
        duration_ms: None,
        timezone: "America/Los_Angeles".into(),
        source: "operator".into(),
        source_ref: None,
        operator_confirmed: false,
        confidence: TemporalConfidence::Medium,
        uncertainty: None,
        observed_at: now,
        effective_at: now,
        expires_at: None,
        supersedes_revision: None,
        evidence_refs: Vec::new(),
        reason_code: None,
    }
}

#[test]
fn no_deadline_is_truthful_without_fabricated_urgency() {
    let scope = TemporalScope::project("/workspace/project", "main");
    let projection = project_temporal(scope, &[], Utc::now());
    assert_eq!(projection.deadline_status, DeadlineStatus::None);
    assert!(projection.active_commitment.is_none());
    assert!(projection.urgency.is_none());
}

#[test]
fn latest_breached_revision_replaces_the_prior_canonical_commitment() {
    let now = Utc::now();
    let mut committed = claim(TemporalClaimKind::ExternalCommitment);
    committed.status = TemporalClaimStatus::Canonical;
    committed.operator_confirmed = true;
    committed.target_at = Some(now - chrono::Duration::minutes(1));
    committed.effective_at = now - chrono::Duration::minutes(2);
    let mut breached = committed.clone();
    breached.revision = 2;
    breached.status = TemporalClaimStatus::Breached;
    breached.supersedes_revision = Some(1);
    let event = |id: &str, claim: TemporalClaim, recorded_at| TemporalEvent {
        event_id: id.into(),
        sequence: 0,
        event_kind: TemporalEventKind::ClaimRevised,
        scope: claim.scope.clone(),
        claim: Some(claim),
        clock_sample: None,
        metadata: Default::default(),
        signature: None,
        predecessor_digest: None,
        recorded_at,
        idempotency_key: id.into(),
        digest: String::new(),
    };
    let scope = committed.scope.clone();
    let projection = project_temporal(
        scope,
        &[
            event("committed", committed, now - chrono::Duration::seconds(1)),
            event("breached", breached, now),
        ],
        now,
    );
    assert_eq!(projection.deadline_status, DeadlineStatus::Breached);
    assert_eq!(projection.active_commitment.unwrap().revision, 2);
}

#[test]
fn commitment_requires_operator_confirmation_and_target() {
    let mut value = claim(TemporalClaimKind::ExternalCommitment);
    assert_eq!(
        validate_claim(&value, None),
        Err(TemporalValidationError::CommitmentRequiresConfirmation)
    );
    value.operator_confirmed = true;
    assert_eq!(
        validate_claim(&value, None),
        Err(TemporalValidationError::CommitmentRequiresTarget)
    );
    value.target_at = Some(Utc::now());
    assert!(validate_claim(&value, None).is_ok());
}

#[test]
fn revision_requires_monotonic_supersession() {
    let previous = claim(TemporalClaimKind::Estimate);
    let mut next = previous.clone();
    next.revision = 2;
    assert_eq!(
        validate_claim(&next, Some(&previous)),
        Err(TemporalValidationError::SupersessionRequired)
    );
    next.supersedes_revision = Some(1);
    assert!(validate_claim(&next, Some(&previous)).is_ok());
}

#[test]
fn ledger_fsyncs_causal_batch_and_replays_idempotently() {
    let root = std::env::temp_dir().join(format!("focusa-temporal-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&root).unwrap();
    let scope = TemporalScope::project(root.to_string_lossy(), "main");
    let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
    let draft = TemporalEvent {
        event_id: "event-1".into(),
        sequence: 0,
        event_kind: TemporalEventKind::ClaimProposed,
        scope,
        claim: Some(claim(TemporalClaimKind::NoDeadline)),
        clock_sample: None,
        metadata: Default::default(),
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let first = ledger.append_batch("batch-1", vec![draft]).unwrap();
    let replay = ledger.append_batch("batch-1", first.clone()).unwrap();
    assert_eq!(first, replay);
    assert_eq!(ledger.read_all().unwrap().len(), 1);
    assert!(verify_event_chain(&ledger.read_all().unwrap()));
    std::fs::remove_dir_all(root).unwrap();
}

fn clock_sample(sample_id: &str, boot_id: &str, monotonic_ns: u128) -> TemporalClockSample {
    TemporalClockSample {
        sample_id: sample_id.into(),
        domain: TemporalClockDomain::MonotonicActive,
        wall_utc: Utc::now(),
        monotonic_ns: Some(monotonic_ns),
        suspend_aware_ns: Some(monotonic_ns),
        boot_id: Some(boot_id.into()),
        timezone: "UTC".into(),
        tzdb_version: Some("2026a".into()),
        source: "clock_gettime".into(),
        observed_offset_ns: Some(0),
        measurement_uncertainty_ns: 10,
        confidence: TemporalConfidence::High,
    }
}

fn uncertainty_budget() -> ClockUncertaintyBudget {
    ClockUncertaintyBudget {
        method: "NIST-TN-1297".into(),
        standard_uncertainty_ns: 10.0,
        expanded_uncertainty_ns: 20.0,
        coverage_factor: 2.0,
        coverage_probability: 0.95,
        offset_ns: 0,
        delay_ns: 5,
        jitter_ns: 2,
        dispersion_ns: 3,
        root_distance_ns: 8,
        frequency_error_ppb: 0.1,
        sample_age_ms: 1,
        calibration_lineage: vec!["calibration:clock-profile-v1".into()],
    }
}

fn version_lineage() -> TemporalVersionLineage {
    TemporalVersionLineage {
        schema_version: "temporal.v1".into(),
        policy_version: "policy.v1".into(),
        adapter_version: "linux-clock.v1".into(),
        calendar_version: Some("gregorian.v1".into()),
        tzdb_version: Some("2026a".into()),
        estimator_version: Some("elapsed.v1".into()),
        clock_profile_version: "clock-profile.v1".into(),
    }
}

#[test]
fn cross_boot_elapsed_requires_an_uncertainty_bearing_upper_bound() {
    let pair = ClockSamplePair {
        before: clock_sample("before", "boot-a", 100),
        after: clock_sample("after", "boot-b", 10),
        elapsed_lower_ns: 1,
        elapsed_upper_ns: None,
        uncertainty: uncertainty_budget(),
        crosses_boot_epoch: true,
        crosses_suspend: false,
        lineage: version_lineage(),
    };
    assert_eq!(
        validate_clock_sample_pair(&pair),
        Err(ClockSamplePairError::MissingCrossEpochBound)
    );
}

#[test]
fn clock_sample_pair_rejects_negative_or_unscientific_bounds() {
    let mut pair = ClockSamplePair {
        before: clock_sample("before", "boot-a", 100),
        after: clock_sample("after", "boot-a", 200),
        elapsed_lower_ns: 100,
        elapsed_upper_ns: Some(90),
        uncertainty: uncertainty_budget(),
        crosses_boot_epoch: false,
        crosses_suspend: false,
        lineage: version_lineage(),
    };
    assert_eq!(
        validate_clock_sample_pair(&pair),
        Err(ClockSamplePairError::NegativeElapsed)
    );
    pair.elapsed_upper_ns = Some(110);
    pair.uncertainty.coverage_probability = 1.5;
    assert_eq!(
        validate_clock_sample_pair(&pair),
        Err(ClockSamplePairError::InvalidUncertainty)
    );
}

#[test]
fn legacy_project_scope_deserializes_without_inventing_finer_authority() {
    let scope: TemporalScope = serde_json::from_value(serde_json::json!({
        "project_root": "/workspace/project",
        "continuity_id": "main"
    }))
    .unwrap();
    assert_eq!(scope, TemporalScope::project("/workspace/project", "main"));
    assert!(scope.host_id.is_none());
    assert!(scope.operator_id.is_none());
    assert!(scope.workpoint_id.is_none());
    assert!(scope.item_id.is_none());
    assert!(scope.task_id.is_none());
}

#[test]
fn consequential_action_requires_fresh_matching_execution_guard() {
    let now = Utc::now();
    let scope = TemporalScope::project("/workspace/project", "main");
    let ask_digest = "sha256:operator-ask".to_string();
    let calendar = HumanCalendarContext {
        context_id: "calendar-1".into(),
        operator_id: "operator-1".into(),
        timezone: "America/Los_Angeles".into(),
        tzdb_version: "2025b".into(),
        availability_policy_ref: "availability:v1".into(),
        quiet_hours_policy_ref: "quiet-hours:v1".into(),
        resolved_boundary_refs: vec!["civil-time:resolved".into()],
        generated_at: now,
        expires_at: now + chrono::Duration::minutes(5),
        private_detail_rehydrate_refs: vec!["operator-profile:v1".into()],
    };
    let frame = TemporalPriorityFrame {
        frame_id: "frame-1".into(),
        scope: scope.clone(),
        operator_ask_digest: ask_digest.clone(),
        primary_objective_ref: "workpoint:verified".into(),
        approaching_deadline_refs: vec![],
        conflict_state: crate::temporal_operations::DeadlineConflictState::Feasible,
        consequence_summary: "operator-selected verified release gate".into(),
        safer_sequence_refs: vec!["release:verify".into()],
        generated_at: now,
        expires_at: now + chrono::Duration::minutes(5),
        evidence_refs: vec!["workpoint:verified".into()],
    };
    let guard = TemporalExecutionGuard {
        guard_id: "guard-1".into(),
        scope: scope.clone(),
        priority_frame_ref: "frame-1".into(),
        authorized_action_refs: vec!["release:verify".into()],
        deterministic_critical_path: true,
        preauthorized: true,
        issued_at: now,
        expires_at: now + chrono::Duration::minutes(5),
        policy_version: "temporal-guard-v1".into(),
        receipt_ref: "operator-approval:v1".into(),
    };
    assert!(
        authorize_temporal_action(
            &calendar,
            &frame,
            Some(&guard),
            &scope,
            &ask_digest,
            "release:verify",
            now
        )
        .is_ok()
    );
    assert!(
        authorize_temporal_action(
            &calendar,
            &frame,
            Some(&guard),
            &scope,
            "sha256:changed-ask",
            "release:verify",
            now
        )
        .is_err()
    );
    assert!(
        authorize_temporal_action(
            &calendar,
            &frame,
            Some(&guard),
            &scope,
            &ask_digest,
            "release:publish",
            now
        )
        .is_err()
    );
}
