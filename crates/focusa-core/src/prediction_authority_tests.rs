use crate::{prediction_authority::*, prediction_authority_ledger::*};

use chrono::{DateTime, Duration, Utc};

fn epistemic_scope(scope_id: &str, root: &str, continuity_id: &str) -> EpistemicScope {
    crate::scoped_state::WorkstreamKey::new(
        crate::scoped_state::ScopeRef::project(
            scope_id,
            root,
            scope_id,
            format!("fingerprint:{scope_id}"),
        )
        .unwrap(),
        continuity_id,
    )
    .unwrap()
}

fn host_epistemic_scope(scope_id: &str, continuity_id: &str) -> EpistemicScope {
    crate::scoped_state::WorkstreamKey::new(
        crate::scoped_state::ScopeRef::host(
            scope_id,
            "/",
            scope_id,
            format!("fingerprint:{scope_id}"),
        )
        .unwrap(),
        continuity_id,
    )
    .unwrap()
}

fn scope_evidence(
    evidence_id: &str,
    kind: crate::prediction_migration::ScopeAttributionEvidenceKind,
    scope: EpistemicScope,
) -> crate::prediction_migration::ScopeAttributionEvidence {
    crate::prediction_migration::ScopeAttributionEvidence {
        evidence_id: evidence_id.into(),
        kind,
        candidate_scope: Some(scope),
        source_ref: evidence_id.into(),
        source_digest: "a".repeat(64),
        observed_at: now(),
    }
}

fn migration_plan(
    plan_id: &str,
    source_ref: &str,
    source_sha256: String,
    evidence: Vec<crate::prediction_migration::ScopeAttributionEvidence>,
) -> crate::prediction_migration::LegacyScopeMigrationPlan {
    crate::prediction_migration::plan_legacy_scope_migration(
        plan_id,
        source_ref,
        source_sha256,
        now(),
        Some(format!("vector:{source_ref}")),
        evidence,
        format!("migration:{source_ref}"),
        format!("receipt:{plan_id}"),
    )
    .unwrap()
}

fn now() -> DateTime<Utc> {
    "2026-07-27T00:00:00Z".parse().unwrap()
}

fn commitment() -> PredictionCommitment {
    PredictionCommitment {
        commitment_id: "commitment:1".into(),
        question_id: "question:1".into(),
        predicted_outcome: "yes".into(),
        confidence: ConfidenceDimensions {
            forecast_probability: 0.9,
            evidence_confidence: 0.7,
            source_reliability: 0.6,
            model_confidence: 0.8,
            resolution_confidence: None,
        },
        information_set: InformationSetRef {
            information_set_id: "info:1".into(),
            version: 1,
            as_of_claim_ref: "temporal:as-of:1".into(),
            evidence_refs: vec!["evidence:info".into()],
        },
        resolver_policy_ref: "resolver:1@1".into(),
        scoring_policy_ref: "brier@1".into(),
        committed_at: now(),
        evidence_refs: vec!["evidence:commitment".into()],
        receipt_ref: "receipt:commitment".into(),
    }
}

fn policy() -> ScoringPolicy {
    ScoringPolicy {
        policy_id: "brier".into(),
        version: 1,
        scorer: ScorerKind::BrierBinary,
        direction: ScoreDirection::LowerIsBetter,
        range_min: 0.0,
        range_max: 1.0,
        assumptions: vec!["binary outcome".into()],
        frozen_at: now(),
        evidence_refs: vec!["evidence:policy".into()],
    }
}

fn resolution(outcome: &str) -> OutcomeResolution {
    OutcomeResolution {
        resolution_id: "resolution:1".into(),
        claim_id: "claim:1".into(),
        resolved_outcome: outcome.into(),
        resolver_policy_ref: "resolver:1@1".into(),
        resolved_at: now(),
        resolution_confidence: 0.95,
        evidence_refs: vec!["evidence:outcome".into()],
        receipt_ref: "receipt:resolution".into(),
    }
}

#[test]
fn frozen_scorer_registry_is_versioned_and_immutable() {
    let mut registry = ScorerRegistry::default();
    assert_eq!(registry.register(policy()).unwrap(), "brier@1");
    assert!(
        registry
            .register(policy())
            .unwrap_err()
            .contains("already exists")
    );
}

#[test]
fn evaluation_requires_frozen_policy_and_preserves_confidence_semantics() {
    let evaluation = evaluate_binary(
        &commitment(),
        &resolution("no"),
        &policy(),
        "evaluation:1",
        now(),
        "receipt:evaluation",
    )
    .unwrap();
    assert!(!evaluation.correct);
    assert!((evaluation.canonical_score - 0.81).abs() < 1e-9);
}

#[test]
fn high_confidence_miss_creates_structured_learning_candidate() {
    let commitment = commitment();
    let evaluation = evaluate_binary(
        &commitment,
        &resolution("no"),
        &policy(),
        "evaluation:1",
        now(),
        "receipt:evaluation",
    )
    .unwrap();
    let candidate = high_confidence_miss_candidate(
        &commitment,
        &evaluation,
        "candidate:1",
        now(),
        now() + Duration::days(7),
        now() + Duration::days(30),
    )
    .unwrap();
    assert_eq!(candidate.reason_code, "high_confidence_miss");
    assert!(!candidate.evidence_refs.is_empty());
}

#[test]
fn prose_only_metrics_cannot_promote_learning() {
    let candidate = LearningCandidate {
        candidate_id: "candidate:1".into(),
        evaluation_id: "evaluation:1".into(),
        reason_code: "miss".into(),
        hypothesis: "h".into(),
        applicability: Applicability {
            includes: vec!["context:1".into()],
            excludes: vec![],
        },
        created_at: now(),
        review_at: now() + Duration::days(1),
        expires_at: now() + Duration::days(30),
        evidence_refs: vec!["evidence:1".into()],
    };
    let decision = decide_promotion(
        &candidate,
        &[],
        &["better".into()],
        false,
        "decision:1",
        now(),
        "receipt:decision",
    );
    assert_eq!(decision.decision, PromotionDecisionKind::Rejected);
    assert!(decision.reason_codes.contains(&"prose_only_metrics".into()));
}

#[test]
fn negative_effect_blocks_promotion() {
    let candidate = LearningCandidate {
        candidate_id: "candidate:1".into(),
        evaluation_id: "evaluation:1".into(),
        reason_code: "miss".into(),
        hypothesis: "h".into(),
        applicability: Applicability {
            includes: vec!["context:1".into()],
            excludes: vec![],
        },
        created_at: now(),
        review_at: now() + Duration::days(1),
        expires_at: now() + Duration::days(30),
        evidence_refs: vec!["evidence:1".into()],
    };
    let metric = TypedMetric {
        metric_id: "loss".into(),
        value: 0.2,
        baseline: 0.4,
        sample_size: 10,
        higher_is_better: false,
    };
    assert_eq!(
        decide_promotion(
            &candidate,
            &[metric],
            &[],
            true,
            "decision:1",
            now(),
            "receipt:1"
        )
        .decision,
        PromotionDecisionKind::Rejected
    );
}

#[test]
fn append_only_recovery_and_scope_isolation_are_deterministic() {
    let scope_a = epistemic_scope("project:a", "/a", "a");
    let scope_b = epistemic_scope("project:b", "/b", "b");
    let question = |id: &str| PredictionQuestion {
        question_id: id.into(),
        subject_ref: "s".into(),
        outcome_space: vec!["yes".into(), "no".into()],
        created_at: now(),
        horizon_claim_ref: "temporal:horizon".into(),
        evidence_refs: vec!["evidence:1".into()],
    };
    let events = [
        ScopedAuthorityEvent {
            event_id: "event:1".into(),
            sequence: 1,
            scope: scope_a.clone(),
            recorded_at: now(),
            event: PredictionAuthorityEvent::Question(question("q:a")),
            evidence_refs: vec!["evidence:1".into()],
            receipt_ref: "receipt:1".into(),
        },
        ScopedAuthorityEvent {
            event_id: "event:2".into(),
            sequence: 2,
            scope: scope_b.clone(),
            recorded_at: now(),
            event: PredictionAuthorityEvent::Question(question("q:b")),
            evidence_refs: vec!["evidence:2".into()],
            receipt_ref: "receipt:2".into(),
        },
    ];
    let jsonl = events
        .iter()
        .map(|event| serde_json::to_string(event).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let ledger = PredictionAuthorityLedger::recover(&jsonl).unwrap();
    assert!(ledger.project(&scope_a).questions.contains_key("q:a"));
    assert!(!ledger.project(&scope_a).questions.contains_key("q:b"));
    assert!(
        ledger
            .clone()
            .append(events[0].clone())
            .unwrap_err()
            .contains("already exists")
    );
}

#[test]
fn action_prediction_and_actual_delta_lifecycle_is_temporally_ordered() {
    let prediction_temporal =
        crate::temporal_clock::capture_temporal_action_envelope("UTC", None).unwrap();
    let mut action_start_temporal = prediction_temporal.clone();
    action_start_temporal.envelope_id = "temporal:action-start".into();
    action_start_temporal.monotonic_ns += 1_000;
    action_start_temporal.captured_at_utc += Duration::microseconds(1);
    action_start_temporal.utc_unix_ns += 1_000;
    let mut prediction = commitment();
    prediction.committed_at = prediction_temporal.captured_at_utc;
    let linked = ActionPredictionCommitment {
        action_id: "action:1".into(),
        action_kind: "lookup".into(),
        action_scope_ref: "scope:project+continuity".into(),
        prediction_temporal: prediction_temporal.clone(),
        action_start_temporal: action_start_temporal.clone(),
        commitment: prediction.clone(),
        duration_baseline: crate::temporal_progress::DurationPredictionBaseline {
            estimate_ns: 2_000,
            lower_bound_ns: 1_000,
            upper_bound_ns: Some(4_000),
            source: "learned_cohort".into(),
            sample_count: 10,
            cohort_key: "cache:hot".into(),
        },
        pattern_cohort_keys: vec!["cache:hot".into()],
    };
    assert_eq!(validate_action_prediction_commitment(&linked), Ok(()));
    let mut backfilled = linked.clone();
    backfilled.prediction_temporal.monotonic_ns = action_start_temporal.monotonic_ns + 1;
    assert_eq!(
        validate_action_prediction_commitment(&backfilled),
        Err(ActionPredictionGateError::PredictionAfterActionStart)
    );

    let mut completed = action_start_temporal.clone();
    completed.envelope_id = "temporal:completed".into();
    completed.monotonic_ns += 3_000;
    completed.captured_at_utc += Duration::microseconds(3);
    completed.utc_unix_ns += 3_000;
    let trace = crate::temporal_progress::ActionTimingTrace {
        trace_id: "trace:1".into(),
        action_id: "action:1".into(),
        prediction_id: prediction.commitment_id.clone(),
        started_temporal_envelope_ref: action_start_temporal.envelope_id.clone(),
        completed_temporal_envelope_ref: completed.envelope_id.clone(),
        started_monotonic_ns: action_start_temporal.monotonic_ns,
        completed_monotonic_ns: completed.monotonic_ns,
        total_elapsed_ns: 3_000,
        spans: vec![],
        attributed_union_ns: 0,
        unattributed_ns: 3_000,
        reconciliation_delta_ns: 0,
        evidence_refs: vec!["evidence:timing".into()],
    };
    let claim = OutcomeClaim {
        claim_id: "claim:1".into(),
        commitment_id: prediction.commitment_id.clone(),
        claimed_outcome: "yes".into(),
        claimed_at: completed.captured_at_utc,
        source_ref: "action:1".into(),
        evidence_refs: vec!["evidence:actual".into()],
    };
    let observation = crate::outcome_resolution::ActionOutcomeObservation {
        observation_id: "observation:1".into(),
        action_id: "action:1".into(),
        commitment_id: prediction.commitment_id.clone(),
        predicted_outcome: "yes".into(),
        actual_outcome: "yes".into(),
        outcome_match_score: 1.0,
        completed_temporal: completed,
        timing_trace: trace,
        expected_duration_ns: 2_000,
        actual_duration_ns: 3_000,
        duration_delta_ns: 1_000,
        outcome_claim: claim,
        evidence_refs: vec!["evidence:settlement".into()],
    };
    assert_eq!(
        crate::outcome_resolution::validate_action_outcome_observation(&linked, &observation),
        Ok(())
    );
    let mut wrong_delta = observation;
    wrong_delta.duration_delta_ns = 0;
    assert_eq!(
        crate::outcome_resolution::validate_action_outcome_observation(&linked, &wrong_delta),
        Err(crate::outcome_resolution::ActionOutcomeObservationError::InvalidDurationDelta)
    );
}

#[test]
fn singleton_migration_requires_converged_authoritative_typed_scope_evidence() {
    use crate::prediction_migration::*;
    let destination = epistemic_scope("project:migration", "/project", "main");
    let payload = serde_json::json!({"legacy":true});
    let event = migrate_legacy_record(
        "migration:1",
        LegacyEpistemicSource::PredictionValueV1,
        "legacy:1",
        &payload,
        destination.clone(),
        1,
        vec!["lineage:1".into()],
        vec!["evidence:source".into()],
        "receipt:legacy",
        now(),
    )
    .unwrap();
    let PredictionAuthorityEvent::LegacyMigration(record) = &event.event else {
        panic!("legacy")
    };
    let evidence = scope_evidence(
        "evidence:typed-scope",
        ScopeAttributionEvidenceKind::TypedScopeIdentity,
        destination.clone(),
    );
    let plan = migration_plan(
        "plan:1",
        "legacy:1",
        record.source_sha256.clone(),
        vec![evidence],
    );
    assert_eq!(
        plan.disposition,
        LegacyScopeMigrationDisposition::ScopedCanonical
    );
    let migrated = apply_legacy_scope_migration_plan(event, &plan)
        .unwrap()
        .unwrap();
    let PredictionAuthorityEvent::LegacyMigration(ref record) = migrated.event else {
        panic!("legacy")
    };
    assert_eq!(
        record.authority_status,
        LegacyAuthorityStatus::ScopedCanonicalMigration
    );
    assert_eq!(record.destination_scope, Some(destination.clone()));

    let weak = scope_evidence(
        "evidence:path-similarity",
        ScopeAttributionEvidenceKind::PathSimilarity,
        destination.clone(),
    );
    let weak_plan = migration_plan("plan:weak", "legacy:weak", "c".repeat(64), vec![weak]);
    assert_eq!(
        weak_plan.disposition,
        LegacyScopeMigrationDisposition::QuarantinedNoAuthoritativeEvidence
    );
    assert!(
        apply_legacy_scope_migration_plan(migrated.clone(), &weak_plan)
            .unwrap()
            .is_none()
    );

    let host = host_epistemic_scope("host:operator", "main");
    let conflict_plan = migration_plan(
        "plan:conflict",
        "legacy:conflict",
        "d".repeat(64),
        vec![
            scope_evidence(
                "evidence:project",
                ScopeAttributionEvidenceKind::VerifiedProjectMarker,
                destination,
            ),
            scope_evidence(
                "evidence:host",
                ScopeAttributionEvidenceKind::VerifiedHostIdentity,
                host,
            ),
        ],
    );
    assert_eq!(
        conflict_plan.disposition,
        LegacyScopeMigrationDisposition::QuarantinedConflictingEvidence
    );
    assert!(
        apply_legacy_scope_migration_plan(migrated, &conflict_plan)
            .unwrap()
            .is_none()
    );
}

#[test]
fn recurring_action_delta_pattern_requires_unique_evidence_backed_samples() {
    let pattern = crate::metacognitive_learning::ActionDeltaPattern {
        pattern_id: "pattern:cache-miss".into(),
        cohort_key: "lookup:provider-a:cache-miss".into(),
        action_ids: vec!["action:1".into(), "action:2".into(), "action:3".into()],
        observation_refs: vec!["obs:1".into(), "obs:2".into(), "obs:3".into()],
        mean_duration_delta_ns: 2_000,
        mean_outcome_match_ppm: 900_000,
        evidence_refs: vec!["evidence:pattern".into()],
    };
    assert_eq!(
        crate::metacognitive_learning::validate_action_delta_pattern(&pattern),
        Ok(())
    );
    let mut duplicate = pattern;
    duplicate.action_ids[2] = "action:1".into();
    assert_eq!(
        crate::metacognitive_learning::validate_action_delta_pattern(&duplicate),
        Err(crate::metacognitive_learning::LearningAuthorityError::InvalidPatternSamples)
    );
}
