use super::*;

use super::*;

fn claim(kind: TemporalClaimKind) -> TemporalClaim {
    let now = Utc::now();
    TemporalClaim {
        claim_id: "claim-1".into(),
        revision: 1,
        scope: TemporalScope {
            project_root: "/workspace/project".into(),
            continuity_id: "main".into(),
        },
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
    let scope = TemporalScope {
        project_root: "/workspace/project".into(),
        continuity_id: "main".into(),
    };
    let projection = project_temporal(scope, &[], Utc::now());
    assert_eq!(projection.deadline_status, DeadlineStatus::None);
    assert!(projection.active_commitment.is_none());
    assert!(projection.urgency.is_none());
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
    let scope = TemporalScope {
        project_root: root.to_string_lossy().to_string(),
        continuity_id: "main".into(),
    };
    let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
    let draft = TemporalEvent {
        event_id: "event-1".into(),
        sequence: 0,
        event_kind: TemporalEventKind::ClaimProposed,
        scope,
        claim: Some(claim(TemporalClaimKind::NoDeadline)),
        clock_sample: None,
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
