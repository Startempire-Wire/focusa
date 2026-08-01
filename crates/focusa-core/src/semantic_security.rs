//! Spec 144 §§26-27 semantic security, identity, privacy, and resource laws.
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticResourceBudget {
    pub max_nodes: u64,
    pub max_edges: u64,
    pub max_depth: u32,
    pub max_reasoning_steps: u64,
    pub max_memory_bytes: u64,
    pub max_result_bytes: u64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSecurityPolicy {
    pub project_root: String,
    pub continuity_id: String,
    pub trusted_origins: BTreeSet<String>,
    pub trusted_keys: BTreeMap<String, String>,
    pub allowed_evidence_classes: BTreeSet<String>,
    pub budget: SemanticResourceBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSecurityEnvelope {
    pub project_root: String,
    pub continuity_id: String,
    pub origin: String,
    pub artifact_digest: String,
    pub signing_key_id: String,
    pub signature_hex: String,
    pub import_origins: BTreeSet<String>,
    pub hot_import_requested: bool,
    pub shacl_sparql_present: bool,
    pub recursive_shape_depth: u32,
    pub node_count: u64,
    pub edge_count: u64,
    pub reasoning_steps: u64,
    pub estimated_memory_bytes: u64,
    pub estimated_result_bytes: u64,
    pub requested_timeout_ms: u64,
    pub predicates: BTreeSet<String>,
    pub textual_payloads: Vec<String>,
    pub evidence_data_classes: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSecurityReceipt {
    pub envelope_digest: String,
    pub policy_digest: String,
    pub previous_receipt_digest: Option<String>,
    pub receipt_digest: String,
    pub signature_verified: bool,
    pub scope_verified: bool,
    pub budget_verified: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticSecurityError {
    #[error("semantic project or continuity scope mismatch")]
    ScopeMismatch,
    #[error("semantic origin or import origin is not trusted: {0}")]
    UntrustedOrigin(String),
    #[error("hot imports are prohibited")]
    HotImportProhibited,
    #[error("SHACL-SPARQL execution is prohibited")]
    ShaclSparqlProhibited,
    #[error("semantic resource budget exceeded: {0}")]
    BudgetExceeded(&'static str),
    #[error("recursive semantic shape exceeds bounded depth")]
    RecursiveShapeDenied,
    #[error("canonical owl:sameAs identity merge is prohibited")]
    CanonicalSameAsProhibited,
    #[error("semantic payload contains secret-bearing material")]
    SecretMaterialProhibited,
    #[error("evidence data class is not eligible: {0}")]
    EvidenceClassDenied(String),
    #[error("semantic signing key is unknown or invalid")]
    InvalidSigningKey,
    #[error("semantic artifact signature is invalid")]
    InvalidSignature,
    #[error("semantic receipt chain predecessor is malformed")]
    InvalidReceiptPredecessor,
}

pub fn validate_semantic_security(
    policy: &SemanticSecurityPolicy,
    envelope: &SemanticSecurityEnvelope,
    previous_receipt_digest: Option<&str>,
) -> Result<SemanticSecurityReceipt, SemanticSecurityError> {
    if envelope.project_root != policy.project_root
        || envelope.continuity_id != policy.continuity_id
    {
        return Err(SemanticSecurityError::ScopeMismatch);
    }
    for origin in std::iter::once(&envelope.origin).chain(envelope.import_origins.iter()) {
        if !policy.trusted_origins.contains(origin) {
            return Err(SemanticSecurityError::UntrustedOrigin(origin.clone()));
        }
    }
    if envelope.hot_import_requested {
        return Err(SemanticSecurityError::HotImportProhibited);
    }
    if envelope.shacl_sparql_present {
        return Err(SemanticSecurityError::ShaclSparqlProhibited);
    }
    validate_budget(&policy.budget, envelope)?;
    if envelope.recursive_shape_depth > policy.budget.max_depth {
        return Err(SemanticSecurityError::RecursiveShapeDenied);
    }
    if envelope
        .predicates
        .iter()
        .any(|item| item == "http://www.w3.org/2002/07/owl#sameAs")
    {
        return Err(SemanticSecurityError::CanonicalSameAsProhibited);
    }
    if envelope
        .textual_payloads
        .iter()
        .any(|text| contains_secret(text))
    {
        return Err(SemanticSecurityError::SecretMaterialProhibited);
    }
    if let Some(class) = envelope
        .evidence_data_classes
        .iter()
        .find(|item| !policy.allowed_evidence_classes.contains(*item))
    {
        return Err(SemanticSecurityError::EvidenceClassDenied(class.clone()));
    }
    verify_artifact_signature(policy, envelope)?;
    if let Some(previous) = previous_receipt_digest {
        if !is_sha256(previous) {
            return Err(SemanticSecurityError::InvalidReceiptPredecessor);
        }
    }
    let envelope_digest = digest_json(envelope);
    let policy_digest = digest_json(policy);
    let previous_receipt_digest = previous_receipt_digest.map(str::to_owned);
    let receipt_digest = digest_json(&(
        &envelope_digest,
        &policy_digest,
        &previous_receipt_digest,
        true,
        true,
        true,
    ));
    Ok(SemanticSecurityReceipt {
        envelope_digest,
        policy_digest,
        previous_receipt_digest,
        receipt_digest,
        signature_verified: true,
        scope_verified: true,
        budget_verified: true,
    })
}

fn validate_budget(
    budget: &SemanticResourceBudget,
    envelope: &SemanticSecurityEnvelope,
) -> Result<(), SemanticSecurityError> {
    for (exceeded, name) in [
        (envelope.node_count > budget.max_nodes, "nodes"),
        (envelope.edge_count > budget.max_edges, "edges"),
        (
            envelope.reasoning_steps > budget.max_reasoning_steps,
            "reasoning_steps",
        ),
        (
            envelope.estimated_memory_bytes > budget.max_memory_bytes,
            "memory",
        ),
        (
            envelope.estimated_result_bytes > budget.max_result_bytes,
            "result",
        ),
        (envelope.requested_timeout_ms > budget.timeout_ms, "timeout"),
    ] {
        if exceeded {
            return Err(SemanticSecurityError::BudgetExceeded(name));
        }
    }
    Ok(())
}

fn verify_artifact_signature(
    policy: &SemanticSecurityPolicy,
    envelope: &SemanticSecurityEnvelope,
) -> Result<(), SemanticSecurityError> {
    verify_ed25519_digest(
        &policy.trusted_keys,
        &envelope.signing_key_id,
        &envelope.artifact_digest,
        &envelope.signature_hex,
    )
}

pub fn verify_ed25519_digest(
    trusted_keys: &BTreeMap<String, String>,
    signing_key_id: &str,
    digest: &str,
    signature_hex: &str,
) -> Result<(), SemanticSecurityError> {
    let key_hex = trusted_keys
        .get(signing_key_id)
        .ok_or(SemanticSecurityError::InvalidSigningKey)?;
    let key_bytes: [u8; 32] = hex::decode(key_hex)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(SemanticSecurityError::InvalidSigningKey)?;
    let key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| SemanticSecurityError::InvalidSigningKey)?;
    let signature = Signature::from_slice(
        &hex::decode(signature_hex).map_err(|_| SemanticSecurityError::InvalidSignature)?,
    )
    .map_err(|_| SemanticSecurityError::InvalidSignature)?;
    key.verify(digest.as_bytes(), &signature)
        .map_err(|_| SemanticSecurityError::InvalidSignature)
}

fn contains_secret(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "authorization:",
        "api_key=",
        "apikey=",
        "password=",
        "secret=",
        "bearer ",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn digest_json(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("semantic security value is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "semantic_security_tests.rs"]
mod tests;
