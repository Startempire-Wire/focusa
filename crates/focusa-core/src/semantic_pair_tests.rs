use crate::semantic_pair::*;
use std::collections::BTreeMap;

fn item(id: &str) -> SemanticItem {
    SemanticItem {
        id: id.to_string(),
        statement: format!("statement for {id}"),
        status: "open".to_string(),
        artifact_refs: vec![],
        attributes: BTreeMap::new(),
    }
}

#[test]
fn aggregate_covers_the_complete_lifecycle_and_is_stably_hashable() {
    let mut pair = SemanticPair::empty(
        "pair-1",
        BuilderAttempt {
            attempt_id: "attempt-1".into(),
            builder: "builder".into(),
            started_at: "2026-01-01T00:00:00Z".into(),
        },
        BuilderContext {
            project_root: "/project".into(),
            continuity_id: "continuity".into(),
            ..Default::default()
        },
        ImmutableSnapshot {
            snapshot_id: "snapshot-1".into(),
            captured_at: "2026-01-01T00:00:01Z".into(),
            content_hash: "sha256:snapshot".into(),
            artifact_refs: vec![],
        },
    );
    pair.obligations.push(item("o"));
    pair.plans.push(item("p"));
    pair.assignments.push(item("a"));
    pair.findings.push(item("f"));
    pair.responses.push(item("r"));
    pair.dispositions.push(item("d"));
    pair.validations.push(item("v"));
    pair.reroutes.push(item("rr"));
    pair.settlements.push(item("s"));
    pair.receipts.push(SemanticReceipt {
        receipt_id: "receipt".into(),
        kind: "settlement".into(),
        issued_at: "2026-01-01T00:00:02Z".into(),
        evidence_refs: vec![],
        attributes: BTreeMap::new(),
    });

    pair.validate().unwrap();
    assert_eq!(
        pair.canonical_hash().unwrap(),
        pair.canonical_hash().unwrap()
    );
}

#[test]
fn large_inline_artifacts_are_rejected_but_handles_are_accepted() {
    let mut value = item("large");
    value.statement = "x".repeat(MAX_INLINE_TEXT_BYTES + 1);
    assert!(matches!(
        value.validate(),
        Err(SemanticPairError::InlineArtifactTooLarge { .. })
    ));

    value.statement = "external artifact".into();
    value.artifact_refs.push(ArtifactHandleRef {
        handle: "artifact://store/large".into(),
        content_hash: "sha256:abc".into(),
        byte_len: 50_000_000,
        media_type: "application/octet-stream".into(),
    });
    value.validate().unwrap();
}
