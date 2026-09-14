//! Spec 144 semantic artifact, canonicalization, validation, and work-contract authority.
//!
//! This module is core-only. Surfaces may project these records but may not
//! redefine semantic identity, canonical bytes, validation severity, or
//! quarantine authority.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticArtifactKind {
    Ontology,
    ShapeGraph,
    JsonLdContext,
    SemanticWorkContract,
    VerificationContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticArtifactState {
    Draft,
    Active,
    Deprecated,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticStatement {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub graph_iri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProvenance {
    pub source_ref: String,
    pub source_digest: String,
    pub generated_by: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticArtifact {
    pub artifact_id: String,
    pub kind: SemanticArtifactKind,
    pub namespace_iri: String,
    pub version: u64,
    pub graph_iri: String,
    pub owner_scope_ref: String,
    pub statements: Vec<SemanticStatement>,
    pub import_iris: Vec<String>,
    pub signature_ref: String,
    pub provenance: SemanticProvenance,
    pub state: SemanticArtifactState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSemanticArtifact {
    pub artifact_id: String,
    pub canonicalization_algorithm: String,
    pub canonical_bytes: Vec<u8>,
    pub sha256: String,
    pub statement_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticCanonicalizationError {
    MissingIdentity,
    InvalidIri,
    MissingProvenance,
    DuplicateStatement,
    CrossGraphStatement,
    UnsignedActiveArtifact,
}

fn is_absolute_iri(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("https://") || value.starts_with("urn:")
}

fn normalized_term(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn canonicalize_semantic_artifact(
    artifact: &SemanticArtifact,
) -> Result<CanonicalSemanticArtifact, SemanticCanonicalizationError> {
    if artifact.artifact_id.trim().is_empty()
        || artifact.owner_scope_ref.trim().is_empty()
        || artifact.version == 0
    {
        return Err(SemanticCanonicalizationError::MissingIdentity);
    }
    if !is_absolute_iri(&artifact.namespace_iri)
        || !is_absolute_iri(&artifact.graph_iri)
        || artifact
            .import_iris
            .iter()
            .any(|value| !is_absolute_iri(value))
    {
        return Err(SemanticCanonicalizationError::InvalidIri);
    }
    if artifact.provenance.source_ref.trim().is_empty()
        || artifact.provenance.source_digest.trim().is_empty()
        || artifact.provenance.generated_by.trim().is_empty()
        || artifact.provenance.evidence_refs.is_empty()
    {
        return Err(SemanticCanonicalizationError::MissingProvenance);
    }
    if artifact.state == SemanticArtifactState::Active && artifact.signature_ref.trim().is_empty() {
        return Err(SemanticCanonicalizationError::UnsignedActiveArtifact);
    }

    let mut lines = BTreeSet::new();
    for statement in &artifact.statements {
        if statement.graph_iri != artifact.graph_iri {
            return Err(SemanticCanonicalizationError::CrossGraphStatement);
        }
        if !is_absolute_iri(&statement.subject)
            || !is_absolute_iri(&statement.predicate)
            || statement.object.trim().is_empty()
        {
            return Err(SemanticCanonicalizationError::InvalidIri);
        }
        let line = format!(
            "<{}> <{}> {} <{}> .",
            normalized_term(&statement.subject),
            normalized_term(&statement.predicate),
            normalized_term(&statement.object),
            normalized_term(&statement.graph_iri)
        );
        if !lines.insert(line) {
            return Err(SemanticCanonicalizationError::DuplicateStatement);
        }
    }

    let mut header = BTreeMap::new();
    header.insert("artifact_id", artifact.artifact_id.clone());
    header.insert("graph_iri", artifact.graph_iri.clone());
    header.insert("namespace_iri", artifact.namespace_iri.clone());
    header.insert("owner_scope_ref", artifact.owner_scope_ref.clone());
    header.insert("version", artifact.version.to_string());
    let mut canonical = header
        .into_iter()
        .map(|(key, value)| format!("# {key}={value}"))
        .collect::<Vec<_>>();
    canonical.extend(lines);
    let canonical_bytes = format!("{}\n", canonical.join("\n")).into_bytes();
    let sha256 = format!("sha256:{:x}", Sha256::digest(&canonical_bytes));
    Ok(CanonicalSemanticArtifact {
        artifact_id: artifact.artifact_id.clone(),
        canonicalization_algorithm: "focusa-rdf-deterministic-v1".into(),
        statement_count: artifact.statements.len(),
        canonical_bytes,
        sha256,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationProfileFamily {
    Intake,
    Promotion,
    ActionPreflight,
    VerificationPlan,
    FindingVerdict,
    Settlement,
    DomainPack,
    MigrationReplay,
    VerticalBundle,
    OmissionFirewall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSeverity {
    Info,
    Warning,
    Violation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticShape {
    pub shape_id: String,
    pub target_class_iri: String,
    pub required_predicate_iris: Vec<String>,
    pub allowed_predicate_iris: Vec<String>,
    pub closed: bool,
    pub severity: SemanticSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationProfile {
    pub profile_id: String,
    pub family: ValidationProfileFamily,
    pub version: u64,
    pub shapes: Vec<SemanticShape>,
    pub import_allowlist: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticValidationFinding {
    pub shape_id: String,
    pub severity: SemanticSeverity,
    pub message: String,
    pub predicate_iri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticValidationReport {
    pub artifact_id: String,
    pub profile_ref: String,
    pub canonical_digest: Option<String>,
    pub conforms: bool,
    pub quarantine_required: bool,
    pub findings: Vec<SemanticValidationFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticValidationReceipt {
    pub validation_id: String,
    pub validation_purpose: ValidationProfileFamily,
    pub target_ref: String,
    pub semantic_pair_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub workpoint_ref: String,
    pub registry_version: u64,
    pub domain_pack_versions: Vec<String>,
    pub shape_bundle_id: String,
    pub shape_bundle_hash: String,
    pub data_graph_hash: String,
    pub inference_graph_hash: String,
    pub inference_profile: String,
    pub reasoner_implementation: String,
    pub reasoner_version: String,
    pub validator_implementation: String,
    pub validator_version: String,
    pub conforms: bool,
    pub severity_counts: BTreeMap<SemanticSeverity, u64>,
    pub results: Vec<SemanticValidationFinding>,
    pub policy_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticValidationError {
    InvalidProfile,
    Canonicalization(SemanticCanonicalizationError),
}

pub fn validate_semantic_artifact(
    artifact: &SemanticArtifact,
    profile: &ValidationProfile,
) -> Result<SemanticValidationReport, SemanticValidationError> {
    if profile.profile_id.trim().is_empty()
        || profile.version == 0
        || profile.shapes.is_empty()
        || profile.evidence_refs.is_empty()
    {
        return Err(SemanticValidationError::InvalidProfile);
    }
    let canonical = canonicalize_semantic_artifact(artifact)
        .map_err(SemanticValidationError::Canonicalization)?;
    let predicates = artifact
        .statements
        .iter()
        .map(|statement| statement.predicate.as_str())
        .collect::<BTreeSet<_>>();
    let mut findings = Vec::new();

    for import in &artifact.import_iris {
        if !profile.import_allowlist.contains(import) {
            findings.push(SemanticValidationFinding {
                shape_id: "focusa:ImportAllowlistShape".into(),
                severity: SemanticSeverity::Violation,
                message: format!("import is not allowlisted: {import}"),
                predicate_iri: None,
            });
        }
    }
    for shape in &profile.shapes {
        if shape.shape_id.trim().is_empty() || !is_absolute_iri(&shape.target_class_iri) {
            return Err(SemanticValidationError::InvalidProfile);
        }
        for required in &shape.required_predicate_iris {
            if !predicates.contains(required.as_str()) {
                findings.push(SemanticValidationFinding {
                    shape_id: shape.shape_id.clone(),
                    severity: shape.severity,
                    message: format!("required predicate is missing: {required}"),
                    predicate_iri: Some(required.clone()),
                });
            }
        }
        if shape.closed {
            let allowed = shape.allowed_predicate_iris.iter().collect::<BTreeSet<_>>();
            for predicate in &predicates {
                if !allowed.contains(&predicate.to_string()) {
                    findings.push(SemanticValidationFinding {
                        shape_id: shape.shape_id.clone(),
                        severity: shape.severity,
                        message: format!("predicate is not allowed by closed shape: {predicate}"),
                        predicate_iri: Some((*predicate).to_string()),
                    });
                }
            }
        }
    }
    findings.sort_by(|left, right| {
        (
            &left.severity,
            &left.shape_id,
            &left.predicate_iri,
            &left.message,
        )
            .cmp(&(
                &right.severity,
                &right.shape_id,
                &right.predicate_iri,
                &right.message,
            ))
    });
    let quarantine_required = findings
        .iter()
        .any(|finding| finding.severity == SemanticSeverity::Violation);
    Ok(SemanticValidationReport {
        artifact_id: artifact.artifact_id.clone(),
        profile_ref: format!("{}@{}", profile.profile_id, profile.version),
        canonical_digest: Some(canonical.sha256),
        conforms: !quarantine_required,
        quarantine_required,
        findings,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticExecutionPair {
    pub action_plan_ref: String,
    pub verification_plan_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticWorkContract {
    pub contract_id: String,
    pub work_item_ref: String,
    pub project_scope_ref: String,
    pub deliverable_refs: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub allowed_mutation_refs: Vec<String>,
    pub prohibited_mutation_refs: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub receipt_destinations: Vec<String>,
    pub execution_pair: SemanticExecutionPair,
    pub ontology_version_ref: String,
    pub validation_profile_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticWorkContractError {
    MissingIdentity,
    MissingAcceptance,
    MissingActionVerificationPair,
    MutationConflict,
    MissingProofAuthority,
}

pub fn validate_semantic_work_contract(
    contract: &SemanticWorkContract,
) -> Result<(), SemanticWorkContractError> {
    if contract.contract_id.trim().is_empty()
        || contract.work_item_ref.trim().is_empty()
        || contract.project_scope_ref.trim().is_empty()
        || contract.ontology_version_ref.trim().is_empty()
    {
        return Err(SemanticWorkContractError::MissingIdentity);
    }
    if contract.deliverable_refs.is_empty() || contract.acceptance_criteria.is_empty() {
        return Err(SemanticWorkContractError::MissingAcceptance);
    }
    if contract.execution_pair.action_plan_ref.trim().is_empty()
        || contract
            .execution_pair
            .verification_plan_ref
            .trim()
            .is_empty()
        || contract.execution_pair.action_plan_ref == contract.execution_pair.verification_plan_ref
    {
        return Err(SemanticWorkContractError::MissingActionVerificationPair);
    }
    let allowed = contract
        .allowed_mutation_refs
        .iter()
        .collect::<BTreeSet<_>>();
    if contract
        .prohibited_mutation_refs
        .iter()
        .any(|item| allowed.contains(item))
    {
        return Err(SemanticWorkContractError::MutationConflict);
    }
    if contract.evidence_requirements.is_empty()
        || contract.receipt_destinations.is_empty()
        || contract.validation_profile_refs.is_empty()
    {
        return Err(SemanticWorkContractError::MissingProofAuthority);
    }
    Ok(())
}

#[cfg(test)]
#[path = "semantic_integrity_tests.rs"]
mod tests;
