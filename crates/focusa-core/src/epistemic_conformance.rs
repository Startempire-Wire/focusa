//! Spec138A combined-source zero-deferral and omission-firewall authority.

use crate::prediction_profiles::{PredictionProfile, ProfileStatus};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormativeSource {
    ParentSpec138,
    AddendumSpec138A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormativeDisposition {
    Implemented,
    NotApplicable,
    Deferred,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombinedConformanceRow {
    pub requirement_id: String,
    pub source_atom_ref: String,
    pub source: NormativeSource,
    pub source_line: usize,
    pub disposition: NormativeDisposition,
    pub applicability_decision_ref: Option<String>,
    pub implementation_refs: Vec<String>,
    pub test_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryDagNode {
    pub node_id: String,
    pub depends_on: Vec<String>,
    pub verified_complete: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombinedConformanceRequest {
    pub source_sha256: BTreeMap<String, String>,
    pub combined_source_sha256: String,
    pub rows: Vec<CombinedConformanceRow>,
    pub profile_statuses: BTreeMap<PredictionProfile, ProfileStatus>,
    pub delivery_dag: Vec<DeliveryDagNode>,
    pub scorer_registry_count: usize,
    pub durable_append_only_history: bool,
    pub migration_verified: bool,
    pub operation_client_parity_verified: bool,
    pub source_independence_verified: bool,
    pub security_verified: bool,
    pub exact_sha_integrated_proof: bool,
    pub forbidden_placeholder_hits: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombinedConformanceError {
    SourceHashSetIncomplete,
    InvalidSourceHash,
    InvalidCombinedHash,
    ParentCountMismatch,
    AddendumCountMismatch,
    DuplicateRequirement,
    DuplicateSourceAtom,
    DeferredRequirement,
    UnknownRequirement,
    MissingApplicabilityDecision,
    MissingRowProof,
    MissingProfile(PredictionProfile),
    ProfileNotVerified(PredictionProfile),
    ScorerRegistryIncomplete,
    AppendOnlyHistoryRequired,
    MigrationRequired,
    OperationParityRequired,
    SourceIndependenceRequired,
    SecurityRequired,
    IntegratedProofRequired,
    ForbiddenPlaceholder,
    DeliveryDagEmpty,
    DeliveryDagDuplicate,
    DeliveryDependencyMissing,
    DeliveryDependencyUnsettled,
    DeliveryNodeUnverified,
    MissingEvidence,
    MissingReceipt,
}

pub fn validate_combined_conformance(
    request: &CombinedConformanceRequest,
) -> Result<(), CombinedConformanceError> {
    if request.source_sha256.len() != 2 {
        return Err(CombinedConformanceError::SourceHashSetIncomplete);
    }
    if request.source_sha256.values().any(|hash| !valid_sha(hash)) {
        return Err(CombinedConformanceError::InvalidSourceHash);
    }
    if !valid_sha(&request.combined_source_sha256) {
        return Err(CombinedConformanceError::InvalidCombinedHash);
    }
    let parent = request
        .rows
        .iter()
        .filter(|row| row.source == NormativeSource::ParentSpec138)
        .count();
    let addendum = request
        .rows
        .iter()
        .filter(|row| row.source == NormativeSource::AddendumSpec138A)
        .count();
    if parent != 273 {
        return Err(CombinedConformanceError::ParentCountMismatch);
    }
    if addendum != 269 {
        return Err(CombinedConformanceError::AddendumCountMismatch);
    }
    let mut requirements = BTreeSet::new();
    let mut atoms = BTreeSet::new();
    for row in &request.rows {
        if !requirements.insert(&row.requirement_id) {
            return Err(CombinedConformanceError::DuplicateRequirement);
        }
        if !atoms.insert(&row.source_atom_ref) {
            return Err(CombinedConformanceError::DuplicateSourceAtom);
        }
        match row.disposition {
            NormativeDisposition::Deferred => {
                return Err(CombinedConformanceError::DeferredRequirement);
            }
            NormativeDisposition::Unknown => {
                return Err(CombinedConformanceError::UnknownRequirement);
            }
            NormativeDisposition::NotApplicable
                if row
                    .applicability_decision_ref
                    .as_deref()
                    .is_none_or(str::is_empty) =>
            {
                return Err(CombinedConformanceError::MissingApplicabilityDecision);
            }
            NormativeDisposition::Implemented | NormativeDisposition::NotApplicable => {}
        }
        if row.requirement_id.is_empty()
            || row.source_atom_ref.is_empty()
            || row.source_line == 0
            || row.implementation_refs.is_empty()
            || row.test_refs.is_empty()
            || row.evidence_refs.is_empty()
            || row.receipt_refs.is_empty()
        {
            return Err(CombinedConformanceError::MissingRowProof);
        }
    }
    for profile in [
        PredictionProfile::A,
        PredictionProfile::B,
        PredictionProfile::C,
        PredictionProfile::D,
        PredictionProfile::E,
        PredictionProfile::F,
        PredictionProfile::G,
        PredictionProfile::H,
    ] {
        let status = request
            .profile_statuses
            .get(&profile)
            .ok_or(CombinedConformanceError::MissingProfile(profile))?;
        if *status != ProfileStatus::VerifiedComplete {
            return Err(CombinedConformanceError::ProfileNotVerified(profile));
        }
    }
    if request.scorer_registry_count != 31 {
        return Err(CombinedConformanceError::ScorerRegistryIncomplete);
    }
    if !request.durable_append_only_history {
        return Err(CombinedConformanceError::AppendOnlyHistoryRequired);
    }
    if !request.migration_verified {
        return Err(CombinedConformanceError::MigrationRequired);
    }
    if !request.operation_client_parity_verified {
        return Err(CombinedConformanceError::OperationParityRequired);
    }
    if !request.source_independence_verified {
        return Err(CombinedConformanceError::SourceIndependenceRequired);
    }
    if !request.security_verified {
        return Err(CombinedConformanceError::SecurityRequired);
    }
    if !request.exact_sha_integrated_proof {
        return Err(CombinedConformanceError::IntegratedProofRequired);
    }
    if !request.forbidden_placeholder_hits.is_empty() {
        return Err(CombinedConformanceError::ForbiddenPlaceholder);
    }
    validate_delivery_dag(&request.delivery_dag)?;
    if request.evidence_refs.is_empty() {
        return Err(CombinedConformanceError::MissingEvidence);
    }
    if request.receipt_ref.trim().is_empty() {
        return Err(CombinedConformanceError::MissingReceipt);
    }
    Ok(())
}

pub fn validate_delivery_dag(nodes: &[DeliveryDagNode]) -> Result<(), CombinedConformanceError> {
    if nodes.is_empty() {
        return Err(CombinedConformanceError::DeliveryDagEmpty);
    }
    let mut settled = BTreeSet::new();
    let ids: BTreeSet<_> = nodes.iter().map(|node| node.node_id.as_str()).collect();
    if ids.len() != nodes.len() {
        return Err(CombinedConformanceError::DeliveryDagDuplicate);
    }
    for node in nodes {
        if !node.verified_complete
            || node.evidence_refs.is_empty()
            || node.receipt_ref.trim().is_empty()
        {
            return Err(CombinedConformanceError::DeliveryNodeUnverified);
        }
        for dependency in &node.depends_on {
            if !ids.contains(dependency.as_str()) {
                return Err(CombinedConformanceError::DeliveryDependencyMissing);
            }
            if !settled.contains(dependency) {
                return Err(CombinedConformanceError::DeliveryDependencyUnsettled);
            }
        }
        settled.insert(node.node_id.clone());
    }
    Ok(())
}

pub fn validate_requirement_removal(
    old_source_atom_ref: &str,
    current_source_atoms: &BTreeSet<String>,
    replacement_combined_hash: &str,
    review_receipt_ref: &str,
) -> Result<(), CombinedConformanceError> {
    if current_source_atoms.contains(old_source_atom_ref) || !valid_sha(replacement_combined_hash) {
        return Err(CombinedConformanceError::InvalidCombinedHash);
    }
    if review_receipt_ref.trim().is_empty() {
        return Err(CombinedConformanceError::MissingReceipt);
    }
    Ok(())
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(source: NormativeSource, index: usize) -> CombinedConformanceRow {
        CombinedConformanceRow {
            requirement_id: format!("requirement-{source:?}-{index}"),
            source_atom_ref: format!("atom-{source:?}-{index}"),
            source,
            source_line: index + 1,
            disposition: NormativeDisposition::Implemented,
            applicability_decision_ref: None,
            implementation_refs: vec!["runtime".into()],
            test_refs: vec!["test".into()],
            evidence_refs: vec!["evidence".into()],
            receipt_refs: vec!["receipt".into()],
        }
    }
    fn request() -> CombinedConformanceRequest {
        let mut rows = Vec::new();
        for i in 0..273 {
            rows.push(row(NormativeSource::ParentSpec138, i));
        }
        for i in 0..269 {
            rows.push(row(NormativeSource::AddendumSpec138A, i));
        }
        CombinedConformanceRequest {
            source_sha256: BTreeMap::from([
                ("parent".into(), "a".repeat(64)),
                ("addendum".into(), "b".repeat(64)),
            ]),
            combined_source_sha256: "c".repeat(64),
            rows,
            profile_statuses: [
                PredictionProfile::A,
                PredictionProfile::B,
                PredictionProfile::C,
                PredictionProfile::D,
                PredictionProfile::E,
                PredictionProfile::F,
                PredictionProfile::G,
                PredictionProfile::H,
            ]
            .into_iter()
            .map(|profile| (profile, ProfileStatus::VerifiedComplete))
            .collect(),
            delivery_dag: vec![
                DeliveryDagNode {
                    node_id: "order_0".into(),
                    depends_on: vec![],
                    verified_complete: true,
                    evidence_refs: vec!["evidence".into()],
                    receipt_ref: "receipt".into(),
                },
                DeliveryDagNode {
                    node_id: "order_1".into(),
                    depends_on: vec!["order_0".into()],
                    verified_complete: true,
                    evidence_refs: vec!["evidence".into()],
                    receipt_ref: "receipt".into(),
                },
            ],
            scorer_registry_count: 31,
            durable_append_only_history: true,
            migration_verified: true,
            operation_client_parity_verified: true,
            source_independence_verified: true,
            security_verified: true,
            exact_sha_integrated_proof: true,
            forbidden_placeholder_hits: vec![],
            evidence_refs: vec!["evidence:combined".into()],
            receipt_ref: "receipt:combined".into(),
        }
    }
    #[test]
    fn full_combined_source_conformance_accepts_exact_complete_proof() {
        assert!(validate_combined_conformance(&request()).is_ok());
    }
    #[test]
    fn zero_deferral_and_profile_firewalls_fail_closed() {
        let mut value = request();
        value.rows[0].disposition = NormativeDisposition::Deferred;
        assert_eq!(
            validate_combined_conformance(&value),
            Err(CombinedConformanceError::DeferredRequirement)
        );
        let mut value = request();
        value.profile_statuses.remove(&PredictionProfile::H);
        assert_eq!(
            validate_combined_conformance(&value),
            Err(CombinedConformanceError::MissingProfile(
                PredictionProfile::H
            ))
        );
    }
    #[test]
    fn non_applicability_and_removal_require_explicit_authority() {
        let mut value = request();
        value.rows[0].disposition = NormativeDisposition::NotApplicable;
        assert_eq!(
            validate_combined_conformance(&value),
            Err(CombinedConformanceError::MissingApplicabilityDecision)
        );
        let current = BTreeSet::from(["still-present".into()]);
        assert!(
            validate_requirement_removal("removed", &current, &"d".repeat(64), "receipt:review")
                .is_ok()
        );
        assert!(
            validate_requirement_removal(
                "still-present",
                &current,
                &"d".repeat(64),
                "receipt:review"
            )
            .is_err()
        );
    }
}
