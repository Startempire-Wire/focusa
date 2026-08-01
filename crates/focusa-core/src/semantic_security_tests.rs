use super::*;
use ed25519_dalek::{Signer, SigningKey};

fn fixture() -> (SemanticSecurityPolicy, SemanticSecurityEnvelope) {
    let signing = SigningKey::from_bytes(&[7; 32]);
    let artifact_digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let signature_hex = hex::encode(signing.sign(artifact_digest.as_bytes()).to_bytes());
    let policy = SemanticSecurityPolicy {
        project_root: "/project".into(),
        continuity_id: "continuity-1".into(),
        trusted_origins: BTreeSet::from(["https://schemas.focusa.dev".into()]),
        trusted_keys: BTreeMap::from([(
            "release-key".into(),
            hex::encode(signing.verifying_key().to_bytes()),
        )]),
        allowed_evidence_classes: BTreeSet::from(["public".into(), "internal".into()]),
        budget: SemanticResourceBudget {
            max_nodes: 10_000,
            max_edges: 40_000,
            max_depth: 16,
            max_reasoning_steps: 100_000,
            max_memory_bytes: 64 * 1024 * 1024,
            max_result_bytes: 1024 * 1024,
            timeout_ms: 5_000,
        },
    };
    let envelope = SemanticSecurityEnvelope {
        project_root: "/project".into(),
        continuity_id: "continuity-1".into(),
        origin: "https://schemas.focusa.dev".into(),
        artifact_digest: artifact_digest.into(),
        signing_key_id: "release-key".into(),
        signature_hex,
        import_origins: BTreeSet::from(["https://schemas.focusa.dev".into()]),
        hot_import_requested: false,
        shacl_sparql_present: false,
        recursive_shape_depth: 4,
        node_count: 100,
        edge_count: 300,
        reasoning_steps: 1_000,
        estimated_memory_bytes: 1_000_000,
        estimated_result_bytes: 10_000,
        requested_timeout_ms: 1_000,
        predicates: BTreeSet::from(["https://schemas.focusa.dev/hasEvidence".into()]),
        textual_payloads: vec!["bounded semantic claim".into()],
        evidence_data_classes: BTreeSet::from(["internal".into()]),
    };
    (policy, envelope)
}

#[test]
fn trusted_signed_bounded_exact_scope_envelope_produces_chained_receipt() {
    let (policy, envelope) = fixture();
    let first = validate_semantic_security(&policy, &envelope, None).unwrap();
    assert!(first.signature_verified && first.scope_verified && first.budget_verified);
    assert!(is_sha256(&first.receipt_digest));
    let second =
        validate_semantic_security(&policy, &envelope, Some(&first.receipt_digest)).unwrap();
    assert_eq!(second.previous_receipt_digest, Some(first.receipt_digest));
    assert_ne!(second.receipt_digest, second.envelope_digest);
}

#[test]
fn scope_origin_and_signature_fail_closed() {
    let (policy, mut envelope) = fixture();
    envelope.continuity_id = "foreign".into();
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::ScopeMismatch)
    );
    let (_, mut envelope) = fixture();
    envelope.origin = "https://evil.example".into();
    assert!(matches!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::UntrustedOrigin(_))
    ));
    let (_, mut envelope) = fixture();
    envelope.artifact_digest.push('0');
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::InvalidSignature)
    );
}

#[test]
fn executable_shapes_identity_merges_secrets_and_ineligible_evidence_are_rejected() {
    let (policy, mut envelope) = fixture();
    envelope.shacl_sparql_present = true;
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::ShaclSparqlProhibited)
    );
    let (_, mut envelope) = fixture();
    envelope
        .predicates
        .insert("http://www.w3.org/2002/07/owl#sameAs".into());
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::CanonicalSameAsProhibited)
    );
    let (_, mut envelope) = fixture();
    envelope
        .textual_payloads
        .push("api_key=do-not-store".into());
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::SecretMaterialProhibited)
    );
    let (_, mut envelope) = fixture();
    envelope.evidence_data_classes.insert("restricted".into());
    assert!(matches!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::EvidenceClassDenied(_))
    ));
}

#[test]
fn oversized_and_recursive_work_is_rejected_before_execution() {
    let (policy, mut envelope) = fixture();
    envelope.node_count = policy.budget.max_nodes + 1;
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::BudgetExceeded("nodes"))
    );
    let (_, mut envelope) = fixture();
    envelope.recursive_shape_depth = policy.budget.max_depth + 1;
    assert_eq!(
        validate_semantic_security(&policy, &envelope, None),
        Err(SemanticSecurityError::RecursiveShapeDenied)
    );
}
