//! Spec 144 namespace/version registry and reproducible semantic build lifecycle.

use crate::semantic_integrity::{
    CanonicalSemanticArtifact, SemanticArtifact, SemanticArtifactState,
    SemanticCanonicalizationError, canonicalize_semantic_artifact,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedGraphKind {
    Registry,
    Shapes,
    Contract,
    Builder,
    Observations,
    Inference,
    Verifier,
    Response,
    Settlement,
    Quarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicClass {
    OperatorAsserted,
    UserAsserted,
    DeterministicAsserted,
    ToolObserved,
    RuntimeObserved,
    ReducerAsserted,
    ModelProposed,
    ModelInferred,
    ReasonerInferred,
    VerificationConfirmed,
    LegacyAssumed,
    Contradicted,
    Invalid,
    Quarantined,
    UnsupportedOpaque,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedGraphRegistration {
    pub graph_iri: String,
    pub kind: NamedGraphKind,
    pub owner_scope_ref: String,
    pub artifact_id: String,
    pub artifact_version: u64,
    pub epistemic_class: EpistemicClass,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceRegistration {
    pub prefix: String,
    pub namespace_iri: String,
    pub owner_ref: String,
    pub artifact_id: String,
    pub artifact_version: u64,
    pub evidence_refs: Vec<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SemanticRegistryEvent {
    ArtifactRegistered {
        artifact: SemanticArtifact,
        namespace: NamespaceRegistration,
        graph: NamedGraphRegistration,
    },
    ArtifactActivated {
        artifact_id: String,
        version: u64,
        receipt_ref: String,
    },
    ArtifactDeprecated {
        artifact_id: String,
        version: u64,
        superseded_by: String,
        receipt_ref: String,
    },
    ArtifactQuarantined {
        artifact_id: String,
        version: u64,
        report_ref: String,
        receipt_ref: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRegistry {
    pub events: Vec<SemanticRegistryEvent>,
    pub artifacts: BTreeMap<String, BTreeMap<u64, SemanticArtifact>>,
    pub namespaces: BTreeMap<String, NamespaceRegistration>,
    pub graphs: BTreeMap<String, NamedGraphRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticRegistryError {
    InvalidRegistration,
    NamespaceConflict,
    GraphConflict,
    VersionNotMonotonic,
    ArtifactNotFound,
    InvalidTransition,
    MissingReceipt,
    ImportNotRegistered,
    ImportNotActive,
    Canonicalization(SemanticCanonicalizationError),
}

impl SemanticRegistry {
    pub fn append(&mut self, event: SemanticRegistryEvent) -> Result<(), SemanticRegistryError> {
        let mut projected = self.clone();
        projected.apply(&event)?;
        projected.events.push(event);
        *self = projected;
        Ok(())
    }

    fn apply(&mut self, event: &SemanticRegistryEvent) -> Result<(), SemanticRegistryError> {
        match event {
            SemanticRegistryEvent::ArtifactRegistered {
                artifact,
                namespace,
                graph,
            } => {
                canonicalize_semantic_artifact(artifact)
                    .map_err(SemanticRegistryError::Canonicalization)?;
                if namespace.prefix.trim().is_empty()
                    || namespace.owner_ref.trim().is_empty()
                    || namespace.evidence_refs.is_empty()
                    || graph.owner_scope_ref.trim().is_empty()
                    || graph.evidence_refs.is_empty()
                    || namespace.namespace_iri != artifact.namespace_iri
                    || namespace.artifact_id != artifact.artifact_id
                    || namespace.artifact_version != artifact.version
                    || graph.graph_iri != artifact.graph_iri
                    || graph.artifact_id != artifact.artifact_id
                    || graph.artifact_version != artifact.version
                    || graph.owner_scope_ref != artifact.owner_scope_ref
                {
                    return Err(SemanticRegistryError::InvalidRegistration);
                }
                if self
                    .namespaces
                    .get(&namespace.prefix)
                    .is_some_and(|current| {
                        current.namespace_iri != namespace.namespace_iri
                            || current.owner_ref != namespace.owner_ref
                    })
                {
                    return Err(SemanticRegistryError::NamespaceConflict);
                }
                if self.graphs.get(&graph.graph_iri).is_some_and(|current| {
                    current.owner_scope_ref != graph.owner_scope_ref
                        || current.artifact_id != graph.artifact_id
                }) {
                    return Err(SemanticRegistryError::GraphConflict);
                }
                let versions = self
                    .artifacts
                    .entry(artifact.artifact_id.clone())
                    .or_default();
                if versions
                    .keys()
                    .next_back()
                    .is_some_and(|latest| artifact.version <= *latest)
                {
                    return Err(SemanticRegistryError::VersionNotMonotonic);
                }
                versions.insert(artifact.version, artifact.clone());
                self.namespaces
                    .insert(namespace.prefix.clone(), namespace.clone());
                self.graphs.insert(graph.graph_iri.clone(), graph.clone());
            }
            SemanticRegistryEvent::ArtifactActivated {
                artifact_id,
                version,
                receipt_ref,
            } => {
                if receipt_ref.trim().is_empty() {
                    return Err(SemanticRegistryError::MissingReceipt);
                }
                self.validate_import_closure(artifact_id, *version)?;
                let artifact = self.artifact_mut(artifact_id, *version)?;
                if artifact.state != SemanticArtifactState::Draft {
                    return Err(SemanticRegistryError::InvalidTransition);
                }
                artifact.state = SemanticArtifactState::Active;
            }
            SemanticRegistryEvent::ArtifactDeprecated {
                artifact_id,
                version,
                superseded_by,
                receipt_ref,
            } => {
                if receipt_ref.trim().is_empty() || superseded_by.trim().is_empty() {
                    return Err(SemanticRegistryError::MissingReceipt);
                }
                let artifact = self.artifact_mut(artifact_id, *version)?;
                if artifact.state != SemanticArtifactState::Active {
                    return Err(SemanticRegistryError::InvalidTransition);
                }
                artifact.state = SemanticArtifactState::Deprecated;
            }
            SemanticRegistryEvent::ArtifactQuarantined {
                artifact_id,
                version,
                report_ref,
                receipt_ref,
            } => {
                if report_ref.trim().is_empty() || receipt_ref.trim().is_empty() {
                    return Err(SemanticRegistryError::MissingReceipt);
                }
                let artifact = self.artifact_mut(artifact_id, *version)?;
                if artifact.state == SemanticArtifactState::Deprecated {
                    return Err(SemanticRegistryError::InvalidTransition);
                }
                artifact.state = SemanticArtifactState::Quarantined;
            }
        }
        Ok(())
    }

    fn artifact_mut(
        &mut self,
        artifact_id: &str,
        version: u64,
    ) -> Result<&mut SemanticArtifact, SemanticRegistryError> {
        self.artifacts
            .get_mut(artifact_id)
            .and_then(|versions| versions.get_mut(&version))
            .ok_or(SemanticRegistryError::ArtifactNotFound)
    }

    pub fn validate_import_closure(
        &self,
        artifact_id: &str,
        version: u64,
    ) -> Result<(), SemanticRegistryError> {
        let artifact = self
            .artifacts
            .get(artifact_id)
            .and_then(|versions| versions.get(&version))
            .ok_or(SemanticRegistryError::ArtifactNotFound)?;
        for import in &artifact.import_iris {
            let imported = self
                .namespaces
                .values()
                .find(|registration| &registration.namespace_iri == import)
                .ok_or(SemanticRegistryError::ImportNotRegistered)?;
            let active = self
                .artifacts
                .get(&imported.artifact_id)
                .and_then(|versions| versions.get(&imported.artifact_version))
                .is_some_and(|artifact| artifact.state == SemanticArtifactState::Active);
            if !active {
                return Err(SemanticRegistryError::ImportNotActive);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBuildSource {
    pub source_ref: String,
    pub content_sha256: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBuildRequest {
    pub build_id: String,
    pub compiler_version: String,
    pub artifact: SemanticArtifact,
    pub sources: Vec<SemanticBuildSource>,
    pub ordered_transform_refs: Vec<String>,
    pub signature_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBuildManifest {
    pub build_id: String,
    pub compiler_version: String,
    pub artifact_id: String,
    pub artifact_version: u64,
    pub source_digests: BTreeMap<String, String>,
    pub ordered_transform_refs: Vec<String>,
    pub canonicalization_algorithm: String,
    pub output_sha256: String,
    pub signature_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBuildOutput {
    pub canonical: CanonicalSemanticArtifact,
    pub manifest: SemanticBuildManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticBuildError {
    MissingBuildAuthority,
    DuplicateSource,
    InvalidSourceDigest,
    Canonicalization(SemanticCanonicalizationError),
}

pub fn build_semantic_artifact(
    request: &SemanticBuildRequest,
) -> Result<SemanticBuildOutput, SemanticBuildError> {
    if request.build_id.trim().is_empty()
        || request.compiler_version.trim().is_empty()
        || request.signature_ref.trim().is_empty()
        || request.sources.is_empty()
        || request.ordered_transform_refs.is_empty()
    {
        return Err(SemanticBuildError::MissingBuildAuthority);
    }
    let mut source_digests = BTreeMap::new();
    for source in &request.sources {
        if source.source_ref.trim().is_empty()
            || !source.content_sha256.starts_with("sha256:")
            || source.evidence_refs.is_empty()
        {
            return Err(SemanticBuildError::InvalidSourceDigest);
        }
        if source_digests
            .insert(source.source_ref.clone(), source.content_sha256.clone())
            .is_some()
        {
            return Err(SemanticBuildError::DuplicateSource);
        }
    }
    let canonical = canonicalize_semantic_artifact(&request.artifact)
        .map_err(SemanticBuildError::Canonicalization)?;
    Ok(SemanticBuildOutput {
        manifest: SemanticBuildManifest {
            build_id: request.build_id.clone(),
            compiler_version: request.compiler_version.clone(),
            artifact_id: request.artifact.artifact_id.clone(),
            artifact_version: request.artifact.version,
            source_digests,
            ordered_transform_refs: request.ordered_transform_refs.clone(),
            canonicalization_algorithm: canonical.canonicalization_algorithm.clone(),
            output_sha256: canonical.sha256.clone(),
            signature_ref: request.signature_ref.clone(),
        },
        canonical,
    })
}

pub fn semantic_build_manifest_digest(manifest: &SemanticBuildManifest) -> String {
    let bytes = serde_json::to_vec(manifest).expect("semantic build manifest is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "semantic_registry_tests.rs"]
mod tests;
