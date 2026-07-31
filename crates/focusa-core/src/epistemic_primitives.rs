//! Canonical Spec138 epistemic primitive registry and governed records.

use crate::prediction_authority::EpistemicScope;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SPEC138_PRIMITIVE_REGISTRY_SHA256: &str =
    "692fae69f17762d65eafb7f1ed30f1567ce0a3c6b1737bccc6b1b8afa69f743f";
const REGISTRY: &str = include_str!("epistemic_primitives.txt");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicPrimitiveDescriptor {
    pub family_section: u8,
    pub family: String,
    pub primitive: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicPrimitiveStatus {
    Proposed,
    Canonical,
    Superseded,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicProvenance {
    pub source_refs: Vec<String>,
    pub information_set_ref: String,
    pub observed_at: DateTime<Utc>,
    pub producer_ref: String,
    pub derivation_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpistemicPrimitiveRecord {
    pub primitive_id: String,
    pub descriptor: EpistemicPrimitiveDescriptor,
    pub scope: EpistemicScope,
    pub status: EpistemicPrimitiveStatus,
    pub value: serde_json::Value,
    pub provenance: EpistemicProvenance,
    pub version: u64,
    pub supersedes_version: Option<u64>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpistemicPrimitiveError {
    UnknownPrimitive,
    InvalidScope,
    MissingIdentity,
    MissingProvenance,
    MissingEvidence,
    MissingReceipt,
    InvalidVersion,
    SupersessionRequired,
}

pub fn canonical_primitive_registry() -> Vec<EpistemicPrimitiveDescriptor> {
    REGISTRY
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '|');
            Some(EpistemicPrimitiveDescriptor {
                family_section: parts.next()?.parse().ok()?,
                family: parts.next()?.to_string(),
                primitive: parts.next()?.to_string(),
            })
        })
        .collect()
}

pub fn resolve_canonical_primitive(
    family_section: u8,
    primitive: &str,
) -> Option<EpistemicPrimitiveDescriptor> {
    canonical_primitive_registry()
        .into_iter()
        .find(|entry| entry.family_section == family_section && entry.primitive == primitive)
}

pub fn validate_epistemic_primitive(
    record: &EpistemicPrimitiveRecord,
) -> Result<(), EpistemicPrimitiveError> {
    let Some(canonical) = resolve_canonical_primitive(
        record.descriptor.family_section,
        &record.descriptor.primitive,
    ) else {
        return Err(EpistemicPrimitiveError::UnknownPrimitive);
    };
    if canonical.family != record.descriptor.family {
        return Err(EpistemicPrimitiveError::UnknownPrimitive);
    }
    if record.scope.validate().is_err() {
        return Err(EpistemicPrimitiveError::InvalidScope);
    }
    if record.primitive_id.trim().is_empty() {
        return Err(EpistemicPrimitiveError::MissingIdentity);
    }
    if record.provenance.source_refs.is_empty()
        || record.provenance.information_set_ref.trim().is_empty()
        || record.provenance.producer_ref.trim().is_empty()
    {
        return Err(EpistemicPrimitiveError::MissingProvenance);
    }
    if record.evidence_refs.is_empty() {
        return Err(EpistemicPrimitiveError::MissingEvidence);
    }
    if record.receipt_ref.trim().is_empty() {
        return Err(EpistemicPrimitiveError::MissingReceipt);
    }
    if record.version == 0 {
        return Err(EpistemicPrimitiveError::InvalidVersion);
    }
    if record.version > 1 && record.supersedes_version != Some(record.version - 1) {
        return Err(EpistemicPrimitiveError::SupersessionRequired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> EpistemicPrimitiveRecord {
        EpistemicPrimitiveRecord {
            primitive_id: "source-1".into(),
            descriptor: resolve_canonical_primitive(2, "SourceIdentity").unwrap(),
            scope: crate::scoped_state::WorkstreamKey::new(
                crate::scoped_state::ScopeRef::project(
                    "project:primitive-test",
                    "/project",
                    "primitive-test",
                    "fingerprint:primitive-test",
                )
                .unwrap(),
                "main",
            )
            .unwrap(),
            status: EpistemicPrimitiveStatus::Canonical,
            value: serde_json::json!({"source":"operator"}),
            provenance: EpistemicProvenance {
                source_refs: vec!["source:operator".into()],
                information_set_ref: "information-set:1".into(),
                observed_at: Utc::now(),
                producer_ref: "focusa".into(),
                derivation_refs: vec![],
            },
            version: 1,
            supersedes_version: None,
            evidence_refs: vec!["evidence:source".into()],
            receipt_ref: "receipt:source".into(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn generated_registry_is_complete_and_family_qualified() {
        let registry = canonical_primitive_registry();
        assert_eq!(registry.len(), 629);
        assert_eq!(
            registry
                .iter()
                .map(|entry| (entry.family_section, entry.primitive.as_str()))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            629
        );
        assert!(resolve_canonical_primitive(2, "SourceIdentity").is_some());
        assert!(resolve_canonical_primitive(15, "ScorerIdentity").is_some());
    }

    #[test]
    fn primitive_validation_rejects_unknown_and_unproven_records() {
        let mut value = record();
        assert!(validate_epistemic_primitive(&value).is_ok());
        value.descriptor.primitive = "InventedPrimitive".into();
        assert_eq!(
            validate_epistemic_primitive(&value),
            Err(EpistemicPrimitiveError::UnknownPrimitive)
        );
        value = record();
        value.evidence_refs.clear();
        assert_eq!(
            validate_epistemic_primitive(&value),
            Err(EpistemicPrimitiveError::MissingEvidence)
        );
    }
}
