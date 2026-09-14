use super::*;

fn artifact() -> SemanticArtifact {
    SemanticArtifact {
        artifact_id: "artifact:focusa:test".into(),
        kind: SemanticArtifactKind::Ontology,
        namespace_iri: "https://focusa.dev/ontology/test#".into(),
        version: 1,
        graph_iri: "urn:focusa:graph:test".into(),
        owner_scope_ref: "project:/srv/test#continuity:test".into(),
        statements: vec![SemanticStatement {
            subject: "urn:focusa:artifact:test".into(),
            predicate: "urn:focusa:predicate:status".into(),
            object: "\"active\"".into(),
            graph_iri: "urn:focusa:graph:test".into(),
        }],
        import_iris: vec!["https://www.w3.org/ns/prov-o".into()],
        signature_ref: "sig:test".into(),
        provenance: SemanticProvenance {
            source_ref: "spec144:test".into(),
            source_digest: "sha256:test".into(),
            generated_by: "focusa-core".into(),
            evidence_refs: vec!["test:semantic-integrity".into()],
        },
        state: SemanticArtifactState::Active,
    }
}

fn profile() -> ValidationProfile {
    ValidationProfile {
        profile_id: "profile:system".into(),
        family: ValidationProfileFamily::Intake,
        version: 1,
        shapes: vec![SemanticShape {
            shape_id: "shape:status".into(),
            target_class_iri: "urn:focusa:class:Artifact".into(),
            required_predicate_iris: vec!["urn:focusa:predicate:status".into()],
            allowed_predicate_iris: vec!["urn:focusa:predicate:status".into()],
            closed: true,
            severity: SemanticSeverity::Violation,
        }],
        import_allowlist: vec!["https://www.w3.org/ns/prov-o".into()],
        evidence_refs: vec!["spec144:profile".into()],
    }
}

#[test]
fn canonicalization_is_order_independent_and_digest_stable() {
    let mut left = artifact();
    left.statements.push(SemanticStatement {
        subject: "urn:focusa:artifact:test".into(),
        predicate: "urn:focusa:predicate:type".into(),
        object: "<urn:focusa:class:Artifact>".into(),
        graph_iri: left.graph_iri.clone(),
    });
    let mut right = left.clone();
    right.statements.reverse();
    let left = canonicalize_semantic_artifact(&left).unwrap();
    let right = canonicalize_semantic_artifact(&right).unwrap();
    assert_eq!(left.canonical_bytes, right.canonical_bytes);
    assert_eq!(left.sha256, right.sha256);
}

#[test]
fn cross_graph_and_duplicate_statements_fail_closed() {
    let mut cross_graph = artifact();
    cross_graph.statements[0].graph_iri = "urn:focusa:graph:foreign".into();
    assert_eq!(
        canonicalize_semantic_artifact(&cross_graph),
        Err(SemanticCanonicalizationError::CrossGraphStatement)
    );
    let mut duplicate = artifact();
    duplicate.statements.push(duplicate.statements[0].clone());
    assert_eq!(
        canonicalize_semantic_artifact(&duplicate),
        Err(SemanticCanonicalizationError::DuplicateStatement)
    );
}

#[test]
fn validation_profiles_enforce_shapes_imports_and_quarantine() {
    let valid = validate_semantic_artifact(&artifact(), &profile()).unwrap();
    assert!(valid.conforms);
    assert!(!valid.quarantine_required);

    let mut invalid = artifact();
    invalid.statements.clear();
    invalid
        .import_iris
        .push("https://untrusted.example/ontology".into());
    let report = validate_semantic_artifact(&invalid, &profile()).unwrap();
    assert!(!report.conforms);
    assert!(report.quarantine_required);
    assert_eq!(report.findings.len(), 2);
}

#[test]
fn work_contract_requires_independent_verification_and_nonconflicting_mutation() {
    let mut contract = SemanticWorkContract {
        contract_id: "contract:test".into(),
        work_item_ref: "bead:test".into(),
        project_scope_ref: "project:/srv/test#continuity:test".into(),
        deliverable_refs: vec!["artifact:test".into()],
        acceptance_criteria: vec!["canonical digest stable".into()],
        allowed_mutation_refs: vec!["crates/focusa-core".into()],
        prohibited_mutation_refs: vec!["production:data".into()],
        evidence_requirements: vec!["test:semantic-integrity".into()],
        receipt_destinations: vec!["evidence:spec144".into()],
        execution_pair: SemanticExecutionPair {
            action_plan_ref: "plan:action".into(),
            verification_plan_ref: "plan:verification".into(),
        },
        ontology_version_ref: "ontology:focusa@1".into(),
        validation_profile_refs: vec!["profile:system@1".into()],
    };
    assert_eq!(validate_semantic_work_contract(&contract), Ok(()));
    contract
        .prohibited_mutation_refs
        .push("crates/focusa-core".into());
    assert_eq!(
        validate_semantic_work_contract(&contract),
        Err(SemanticWorkContractError::MutationConflict)
    );
}
