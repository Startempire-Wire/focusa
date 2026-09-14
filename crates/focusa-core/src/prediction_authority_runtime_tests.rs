use crate::{
    prediction_authority::*,
    prediction_authority_storage::{PersistentPredictionAuthorityLedger, PredictionStorageError},
};
use chrono::{Duration, Utc};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);

fn scope() -> EpistemicScope {
    let root = std::env::temp_dir().join(format!(
        "focusa-spec138-runtime-{}-{}",
        std::process::id(),
        NEXT_SCOPE.fetch_add(1, Ordering::Relaxed)
    ));
    crate::scoped_state::WorkstreamKey::new(
        crate::scoped_state::ScopeRef::project(
            "project:spec138-runtime",
            &root,
            "spec138-runtime",
            format!("fingerprint:{}", root.display()),
        )
        .unwrap(),
        "runtime-tests",
    )
    .unwrap()
}

fn envelope(
    scope: &EpistemicScope,
    sequence: u64,
    event: PredictionAuthorityEvent,
) -> ScopedAuthorityEvent {
    ScopedAuthorityEvent {
        event_id: format!("event:{sequence}"),
        sequence,
        scope: scope.clone(),
        recorded_at: Utc::now(),
        event,
        evidence_refs: vec![format!("evidence:{sequence}")],
        receipt_ref: format!("receipt:{sequence}"),
    }
}

#[test]
fn restart_replay_and_projection_preserve_all_runtime_authority_variants() {
    let scope = scope();
    let now = Utc::now();
    let events = vec![
        envelope(
            &scope,
            1,
            PredictionAuthorityEvent::OutcomeClaim(OutcomeClaim {
                claim_id: "claim:1".into(),
                commitment_id: "commitment:1".into(),
                claimed_outcome: "success".into(),
                claimed_at: now,
                source_ref: "resolver:1".into(),
                evidence_refs: vec!["evidence:claim".into()],
            }),
        ),
        envelope(
            &scope,
            2,
            PredictionAuthorityEvent::ScoringPolicy(ScoringPolicy {
                policy_id: "brier".into(),
                version: 1,
                scorer: ScorerKind::BrierBinary,
                direction: ScoreDirection::LowerIsBetter,
                range_min: 0.0,
                range_max: 1.0,
                assumptions: vec!["binary outcome".into()],
                frozen_at: now,
                evidence_refs: vec!["evidence:policy".into()],
            }),
        ),
        envelope(
            &scope,
            3,
            PredictionAuthorityEvent::LearningCandidate(LearningCandidate {
                candidate_id: "candidate:1".into(),
                evaluation_id: "evaluation:1".into(),
                reason_code: "high_confidence_miss".into(),
                hypothesis: "source reliability was overstated".into(),
                applicability: Applicability {
                    includes: vec!["context:1".into()],
                    excludes: vec!["context:2".into()],
                },
                created_at: now,
                review_at: now + Duration::days(1),
                expires_at: now + Duration::days(30),
                evidence_refs: vec!["evidence:candidate".into()],
            }),
        ),
        envelope(
            &scope,
            4,
            PredictionAuthorityEvent::PromotionDecision(PromotionDecision {
                decision_id: "decision:1".into(),
                candidate_id: "candidate:1".into(),
                decision: PromotionDecisionKind::Rejected,
                reason_codes: vec!["baseline_not_beaten".into()],
                decided_at: now,
                receipt_ref: "receipt:decision".into(),
            }),
        ),
        envelope(
            &scope,
            5,
            PredictionAuthorityEvent::TransferPrediction(TransferPrediction {
                transfer_id: "transfer:1".into(),
                learning_id: "learning:1".into(),
                source_context_ref: "context:source".into(),
                target_context_ref: "context:target".into(),
                expected_metric_delta: 0.1,
                window_claim_ref: "temporal:window".into(),
                exclusions: vec!["regime:other".into()],
                evidence_refs: vec!["evidence:transfer".into()],
            }),
        ),
    ];
    let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
    let ledger = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
    ledger.append_batch(events.clone()).unwrap();

    let restarted = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
    let projection = restarted.projection().unwrap();
    assert_eq!(projection.sequence, 5);
    assert!(projection.outcome_claims.contains_key("claim:1"));
    assert!(projection.scoring_policies.contains_key("brier@1"));
    assert!(projection.learning_candidates.contains_key("candidate:1"));
    assert!(projection.promotion_decisions.contains_key("decision:1"));
    assert!(projection.transfer_predictions.contains_key("transfer:1"));
    assert_eq!(
        restarted.append_batch(vec![events[4].clone()]),
        Err(PredictionStorageError::DuplicateEvent)
    );
    let replay = restarted.read_all().unwrap();
    assert_eq!(replay.len(), 5);
    assert_eq!(
        replay[4].predecessor_digest.as_deref(),
        Some(replay[3].digest.as_str())
    );
    let _ = std::fs::remove_dir_all(&scope.root_scope.root_path);
}

#[test]
fn semantically_invalid_variant_fails_closed_without_partial_append() {
    let scope = scope();
    let ledger = PersistentPredictionAuthorityLedger::for_project(scope.clone()).unwrap();
    let invalid = envelope(
        &scope,
        1,
        PredictionAuthorityEvent::TransferPrediction(TransferPrediction {
            transfer_id: "transfer:invalid".into(),
            learning_id: "learning:1".into(),
            source_context_ref: "same-context".into(),
            target_context_ref: "same-context".into(),
            expected_metric_delta: f64::NAN,
            window_claim_ref: "".into(),
            exclusions: vec![],
            evidence_refs: vec![],
        }),
    );
    assert!(matches!(
        ledger.append_batch(vec![invalid]),
        Err(PredictionStorageError::InvalidPrimitive(_))
    ));
    assert!(ledger.read_all().unwrap().is_empty());
    assert!(!ledger.path().exists());
}
