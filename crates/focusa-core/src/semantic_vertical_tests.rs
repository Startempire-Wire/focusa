use super::*;
use chrono::Duration;
use ed25519_dalek::{Signer, SigningKey};

fn calibration(id: &str, provider: &str, now: DateTime<Utc>) -> VerifierCalibration {
    VerifierCalibration {
        verifier_id: id.into(),
        provider_class: provider.into(),
        domain: "semantic".into(),
        sample_count: 100,
        precision: 0.95,
        recall: 0.90,
        brier_score: 0.10,
        evidence_refs: vec!["evidence:calibration".into()],
        valid_until: now + Duration::days(30),
    }
}

fn content(now: DateTime<Utc>) -> VerticalBundleContent {
    VerticalBundleContent {
        bundle_id: "vertical-semantic".into(),
        domain: "semantic".into(),
        version: "1.0.0".into(),
        ontology_refs: BTreeSet::from(["ontology:v1".into()]),
        shape_refs: BTreeSet::from(["shapes:v1".into()]),
        domain_graph: BTreeMap::from([("claim:a".into(), BTreeSet::from(["claim:b".into()]))]),
        evidence_index: BTreeMap::from([("claim:a".into(), vec!["evidence:a".into()])]),
        calibration_profile_refs: BTreeSet::from(["calibration:v1".into()]),
        contradictions: vec![],
        valid_from: now - Duration::hours(1),
        expires_at: now + Duration::days(30),
    }
}

#[test]
fn lifecycle_rerouting_is_bounded_and_preserves_findings_and_obligations() {
    let mut lifecycle = VerificationLifecycle {
        state: LifecycleState::VerificationBlocked,
        snapshot_hash: "sha256:one".into(),
        obligation_ids: BTreeSet::from(["obl:one".into()]),
        preserved_finding_ids: BTreeSet::from(["finding:one".into()]),
        reroute_count: 0,
        max_reroutes: 1,
    };
    lifecycle
        .reroute("sha256:two", ["obl:two".into()], ["finding:two".into()])
        .unwrap();
    assert_eq!(lifecycle.state, LifecycleState::VerificationPlanned);
    assert_eq!(lifecycle.obligation_ids.len(), 2);
    assert_eq!(lifecycle.preserved_finding_ids.len(), 2);
    assert_eq!(
        lifecycle.reroute("sha256:three", [], []),
        Err(VerticalGovernanceError::RerouteBudgetExhausted)
    );
    assert_eq!(lifecycle.state, LifecycleState::OscillationDetected);
}

#[test]
fn champion_challenger_requires_calibration_and_provider_independence() {
    let now = Utc::now();
    let valid = VerifierCohort {
        champion: calibration("champion", "provider-a", now),
        challengers: vec![calibration("challenger", "provider-b", now)],
    };
    assert_eq!(valid.validate(now), Ok(()));
    let coupled = VerifierCohort {
        champion: calibration("champion", "provider-a", now),
        challengers: vec![calibration("challenger", "provider-a", now)],
    };
    assert_eq!(
        coupled.validate(now),
        Err(VerticalGovernanceError::ChallengerNotIndependent)
    );
}

#[test]
fn learning_requires_causal_transfer_evidence_and_never_auto_mutates() {
    let proposal = LearningProposal {
        proposal_id: "learn-1".into(),
        source_domain: "semantic".into(),
        target_domain: "release".into(),
        causal_evidence_refs: vec!["evidence:causal".into()],
        transfer_evidence_refs: vec![],
        operator_approved: false,
        automatic_policy_mutation: false,
    };
    assert_eq!(
        proposal.validate(),
        Err(VerticalGovernanceError::MissingTransferEvidence)
    );
    let mut unsafe_proposal = proposal;
    unsafe_proposal
        .transfer_evidence_refs
        .push("evidence:transfer".into());
    assert_eq!(
        unsafe_proposal.authorize_promotion(),
        Err(VerticalGovernanceError::OperatorApprovalRequired)
    );
    unsafe_proposal.automatic_policy_mutation = true;
    assert_eq!(
        unsafe_proposal.validate(),
        Err(VerticalGovernanceError::AutomaticMutationForbidden)
    );
}

#[test]
fn vertical_activation_requires_digest_signature_window_and_resolved_critical_claims() {
    let now = Utc::now();
    let content = content(now);
    let mut bundle = SignedVerticalBundle {
        content_hash: vertical_content_hash(&content),
        content,
        signer_id: "release-authority".into(),
        signature: "signature:v1".into(),
        signature_verified: true,
    };
    assert_eq!(validate_vertical_activation(&bundle, now), Ok(()));
    bundle.signature_verified = false;
    assert_eq!(
        validate_vertical_activation(&bundle, now),
        Err(VerticalGovernanceError::SignatureUnverified)
    );
    bundle.signature_verified = true;
    bundle.content.contradictions.push(DomainContradiction {
        contradiction_id: "contradiction:critical".into(),
        left_claim_ref: "claim:a".into(),
        right_claim_ref: "claim:b".into(),
        severity: ContradictionSeverity::Critical,
        resolution_ref: None,
    });
    bundle.content_hash = vertical_content_hash(&bundle.content);
    assert_eq!(
        validate_vertical_activation(&bundle, now),
        Err(VerticalGovernanceError::UnresolvedContradiction(
            "contradiction:critical".into()
        ))
    );
}

#[test]
fn trusted_activation_cryptographically_verifies_bundle_digest() {
    let now = Utc::now();
    let content = content(now);
    let content_hash = vertical_content_hash(&content);
    let signing = SigningKey::from_bytes(&[9; 32]);
    let signature = hex::encode(signing.sign(content_hash.as_bytes()).to_bytes());
    let mut bundle = SignedVerticalBundle {
        content,
        content_hash,
        signer_id: "trusted-key".into(),
        signature,
        signature_verified: false,
    };
    let trusted = BTreeMap::from([(
        "trusted-key".into(),
        hex::encode(signing.verifying_key().to_bytes()),
    )]);
    assert_eq!(
        validate_vertical_activation_with_trust(&bundle, now, &trusted),
        Ok(())
    );
    bundle.content_hash.push('0');
    assert_eq!(
        validate_vertical_activation_with_trust(&bundle, now, &trusted),
        Err(VerticalGovernanceError::SignatureVerificationFailed)
    );
}
