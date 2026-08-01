use super::*;

fn requirement() -> VerificationRequirement {
    VerificationRequirement {
        requirement_id: "REQ-1".into(),
        criterion_refs: vec!["criterion:a".into()],
        risk_classes: vec!["semantic_integrity".into()],
        mandatory: true,
    }
}

fn profile(id: &str) -> VerifierCapabilityProfile {
    VerifierCapabilityProfile {
        verifier_id: id.into(),
        provider_class: "independent-provider".into(),
        risk_classes: BTreeSet::from(["semantic_integrity".into()]),
        live_provider: true,
        valid_tool_path: true,
        calibrated: true,
        conformance_proven: true,
        deprecated: false,
        approved_tool_ids: vec!["semantic-read".into()],
    }
}

fn snapshot() -> VerificationSnapshot {
    VerificationSnapshot::freeze(
        "snapshot-1",
        BTreeMap::from([("source.ttl".into(), "sha256:source".into())]),
        "sha256:criteria",
        "registry-v1",
    )
}

#[test]
fn obligation_compilation_is_deterministic_and_model_additive_only() {
    let suggestion = ModelSuggestedObligation {
        suggestion_id: "suggestion-1".into(),
        requirement_id: "REQ-1".into(),
        criterion_ref: "criterion:model".into(),
        risk_class: "semantic_integrity".into(),
    };
    let first = compile_obligations(&[requirement()], std::slice::from_ref(&suggestion));
    let second = compile_obligations(&[requirement()], &[suggestion]);
    assert_eq!(first, second);
    assert_eq!(first.requirement_coverage["REQ-1"].len(), 2);
    assert!(first.obligations.iter().any(|item| item.mandatory));
    assert_eq!(first.suggestions_added, vec!["suggestion-1"]);
}

#[test]
fn router_requires_live_conformant_independent_capability() {
    let obligations = compile_obligations(&[requirement()], &[]).obligations;
    let error = route_verification("builder", &snapshot(), &obligations, &[profile("builder")])
        .unwrap_err();
    assert!(matches!(error, VerificationError::UncoveredMandatory(_)));

    let plan =
        route_verification("builder", &snapshot(), &obligations, &[profile("verifier")]).unwrap();
    assert_eq!(plan.assignments.len(), 1);
    assert_eq!(plan.assignments[0].verifier_identity, "verifier");
    assert!(!plan.assignments[0].writer_lease);
    assert_eq!(plan.assignments[0].approved_tool_ids, vec!["semantic-read"]);
}

#[test]
fn settlement_fails_closed_on_stale_snapshot_writer_or_open_finding() {
    let obligations = compile_obligations(&[requirement()], &[]).obligations;
    let mut plan =
        route_verification("builder", &snapshot(), &obligations, &[profile("verifier")]).unwrap();
    let mandatory = BTreeSet::from([obligations[0].obligation_id.clone()]);
    let response = VerificationResponse {
        assignment_id: plan.assignments[0].assignment_id.clone(),
        snapshot_hash: plan.snapshot_hash.clone(),
        verdicts: BTreeMap::from([(
            obligations[0].obligation_id.clone(),
            ObligationVerdict::Pass,
        )]),
        findings: vec![],
    };
    assert_eq!(
        settle_verification(&plan, std::slice::from_ref(&response), &mandatory),
        Ok(())
    );

    plan.assignments[0].writer_lease = true;
    assert_eq!(
        settle_verification(&plan, std::slice::from_ref(&response), &mandatory),
        Err(VerificationError::WriterLeaseConflict)
    );
    plan.assignments[0].writer_lease = false;

    let mut stale = response.clone();
    stale.snapshot_hash = "sha256:stale".into();
    assert_eq!(
        settle_verification(&plan, &[stale], &mandatory),
        Err(VerificationError::SnapshotChanged)
    );

    let mut unknown = response.clone();
    unknown.assignment_id = "unknown".into();
    assert_eq!(
        settle_verification(&plan, &[unknown], &mandatory),
        Err(VerificationError::UnknownAssignment("unknown".into()))
    );

    let mut blocked = response;
    blocked.findings.push(VerificationFinding {
        finding_id: "finding-1".into(),
        obligation_id: obligations[0].obligation_id.clone(),
        blocking: true,
        evidence_refs: vec!["evidence:1".into()],
        disposition: FindingDisposition::Open,
    });
    assert_eq!(
        settle_verification(&plan, &[blocked], &mandatory),
        Err(VerificationError::BlockingFinding("finding-1".into()))
    );
}

#[test]
fn frozen_snapshot_digest_is_order_independent() {
    let a = VerificationSnapshot::freeze(
        "snapshot",
        BTreeMap::from([("b".into(), "2".into()), ("a".into(), "1".into())]),
        "criteria",
        "registry",
    );
    let b = VerificationSnapshot::freeze(
        "snapshot",
        BTreeMap::from([("a".into(), "1".into()), ("b".into(), "2".into())]),
        "criteria",
        "registry",
    );
    assert_eq!(a.content_hash, b.content_hash);
    assert!(a.frozen);
}
