use super::*;
use crate::semantic_integrity::{SemanticArtifactKind, SemanticProvenance};

fn artifact(
    id: &str,
    namespace: &str,
    version: u64,
    state: SemanticArtifactState,
) -> SemanticArtifact {
    SemanticArtifact {
        artifact_id: id.into(),
        kind: SemanticArtifactKind::Ontology,
        namespace_iri: namespace.into(),
        version,
        graph_iri: format!("urn:focusa:graph:{id}:{version}"),
        owner_scope_ref: "project:/srv/test#continuity:test".into(),
        statements: vec![],
        import_iris: vec![],
        signature_ref: "sig:test".into(),
        provenance: SemanticProvenance {
            source_ref: "spec144:test".into(),
            source_digest: "sha256:test".into(),
            generated_by: "focusa-core".into(),
            evidence_refs: vec!["test:semantic-registry".into()],
        },
        state,
    }
}

fn registration(artifact: SemanticArtifact, prefix: &str) -> SemanticRegistryEvent {
    SemanticRegistryEvent::ArtifactRegistered {
        namespace: NamespaceRegistration {
            prefix: prefix.into(),
            namespace_iri: artifact.namespace_iri.clone(),
            owner_ref: "focusa-core".into(),
            artifact_id: artifact.artifact_id.clone(),
            artifact_version: artifact.version,
            evidence_refs: vec!["spec144:namespace".into()],
        },
        graph: NamedGraphRegistration {
            graph_iri: artifact.graph_iri.clone(),
            kind: NamedGraphKind::Registry,
            owner_scope_ref: artifact.owner_scope_ref.clone(),
            artifact_id: artifact.artifact_id.clone(),
            artifact_version: artifact.version,
            epistemic_class: EpistemicClass::DeterministicAsserted,
            evidence_refs: vec!["spec144:graph".into()],
        },
        artifact,
    }
}

#[test]
fn registry_is_append_only_versioned_and_collision_safe() {
    let mut registry = SemanticRegistry::default();
    registry
        .append(registration(
            artifact(
                "ontology:test",
                "https://focusa.dev/test#",
                1,
                SemanticArtifactState::Draft,
            ),
            "test",
        ))
        .unwrap();
    assert_eq!(registry.events.len(), 1);
    let conflict = registration(
        artifact(
            "ontology:foreign",
            "https://foreign.example/#",
            1,
            SemanticArtifactState::Draft,
        ),
        "test",
    );
    assert_eq!(
        registry.append(conflict),
        Err(SemanticRegistryError::NamespaceConflict)
    );
    assert_eq!(
        registry.events.len(),
        1,
        "rejected events never partially project"
    );
}

#[test]
fn activation_requires_registered_active_import_closure() {
    let mut registry = SemanticRegistry::default();
    let imported = artifact(
        "ontology:base",
        "https://focusa.dev/base#",
        1,
        SemanticArtifactState::Active,
    );
    registry.append(registration(imported, "base")).unwrap();
    let mut dependent = artifact(
        "ontology:dependent",
        "https://focusa.dev/dependent#",
        1,
        SemanticArtifactState::Draft,
    );
    dependent.import_iris = vec!["https://focusa.dev/base#".into()];
    registry
        .append(registration(dependent, "dependent"))
        .unwrap();
    registry
        .append(SemanticRegistryEvent::ArtifactActivated {
            artifact_id: "ontology:dependent".into(),
            version: 1,
            receipt_ref: "receipt:activate".into(),
        })
        .unwrap();
    assert_eq!(
        registry.artifacts["ontology:dependent"][&1].state,
        SemanticArtifactState::Active
    );
}

#[test]
fn reproducible_build_sorts_sources_and_stabilizes_digest() {
    let request = SemanticBuildRequest {
        build_id: "build:test".into(),
        compiler_version: "focusa-semantic-compiler@1".into(),
        artifact: artifact(
            "ontology:test",
            "https://focusa.dev/test#",
            1,
            SemanticArtifactState::Draft,
        ),
        sources: vec![
            SemanticBuildSource {
                source_ref: "z.ttl".into(),
                content_sha256: "sha256:z".into(),
                evidence_refs: vec!["source:z".into()],
            },
            SemanticBuildSource {
                source_ref: "a.ttl".into(),
                content_sha256: "sha256:a".into(),
                evidence_refs: vec!["source:a".into()],
            },
        ],
        ordered_transform_refs: vec![
            "transform:expand-jsonld".into(),
            "transform:canonicalize".into(),
        ],
        signature_ref: "sig:build".into(),
    };
    let left = build_semantic_artifact(&request).unwrap();
    let mut reordered = request;
    reordered.sources.reverse();
    let right = build_semantic_artifact(&reordered).unwrap();
    assert_eq!(left, right);
    assert_eq!(
        semantic_build_manifest_digest(&left.manifest),
        semantic_build_manifest_digest(&right.manifest)
    );
}
