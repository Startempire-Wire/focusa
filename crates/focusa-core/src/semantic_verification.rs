//! Spec 144 §§12-18: deterministic, independent semantic verification runtime.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRequirement {
    pub requirement_id: String,
    pub criterion_refs: Vec<String>,
    pub risk_classes: Vec<String>,
    pub mandatory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationObligation {
    pub obligation_id: String,
    pub requirement_id: String,
    pub criterion_ref: String,
    pub risk_class: String,
    pub mandatory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSuggestedObligation {
    pub suggestion_id: String,
    pub requirement_id: String,
    pub criterion_ref: String,
    pub risk_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObligationCompilation {
    pub obligations: Vec<VerificationObligation>,
    pub requirement_coverage: BTreeMap<String, Vec<String>>,
    pub suggestions_added: Vec<String>,
}

pub fn compile_obligations(
    requirements: &[VerificationRequirement],
    suggestions: &[ModelSuggestedObligation],
) -> ObligationCompilation {
    let mut obligations = Vec::new();
    for requirement in requirements {
        for criterion in &requirement.criterion_refs {
            for risk in &requirement.risk_classes {
                obligations.push(VerificationObligation {
                    obligation_id: stable_id(&requirement.requirement_id, criterion, risk),
                    requirement_id: requirement.requirement_id.clone(),
                    criterion_ref: criterion.clone(),
                    risk_class: risk.clone(),
                    mandatory: requirement.mandatory,
                });
            }
        }
    }
    let known: BTreeSet<_> = requirements
        .iter()
        .map(|item| item.requirement_id.as_str())
        .collect();
    let mut suggestions_added = Vec::new();
    for suggestion in suggestions {
        if known.contains(suggestion.requirement_id.as_str()) {
            obligations.push(VerificationObligation {
                obligation_id: stable_id(
                    &suggestion.requirement_id,
                    &suggestion.criterion_ref,
                    &suggestion.risk_class,
                ),
                requirement_id: suggestion.requirement_id.clone(),
                criterion_ref: suggestion.criterion_ref.clone(),
                risk_class: suggestion.risk_class.clone(),
                mandatory: false,
            });
            suggestions_added.push(suggestion.suggestion_id.clone());
        }
    }
    obligations.sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
    obligations.dedup_by(|a, b| a.obligation_id == b.obligation_id);
    suggestions_added.sort();
    let mut requirement_coverage: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for obligation in &obligations {
        requirement_coverage
            .entry(obligation.requirement_id.clone())
            .or_default()
            .push(obligation.obligation_id.clone());
    }
    ObligationCompilation {
        obligations,
        requirement_coverage,
        suggestions_added,
    }
}

fn stable_id(requirement: &str, criterion: &str, risk: &str) -> String {
    let digest = Sha256::digest(format!("{requirement}\0{criterion}\0{risk}").as_bytes());
    format!("obl-{:x}", digest)[..20].to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierCapabilityProfile {
    pub verifier_id: String,
    pub provider_class: String,
    pub risk_classes: BTreeSet<String>,
    pub live_provider: bool,
    pub valid_tool_path: bool,
    pub calibrated: bool,
    pub conformance_proven: bool,
    pub deprecated: bool,
    pub approved_tool_ids: Vec<String>,
}

impl VerifierCapabilityProfile {
    pub fn eligible(&self) -> bool {
        self.live_provider
            && self.valid_tool_path
            && self.calibrated
            && self.conformance_proven
            && !self.deprecated
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationSnapshot {
    pub snapshot_id: String,
    pub source_hashes: BTreeMap<String, String>,
    pub criteria_hash: String,
    pub registry_version: String,
    pub frozen: bool,
    pub content_hash: String,
}

impl VerificationSnapshot {
    pub fn freeze(
        snapshot_id: impl Into<String>,
        source_hashes: BTreeMap<String, String>,
        criteria_hash: impl Into<String>,
        registry_version: impl Into<String>,
    ) -> Self {
        let snapshot_id = snapshot_id.into();
        let criteria_hash = criteria_hash.into();
        let registry_version = registry_version.into();
        let bytes = serde_json::to_vec(&(
            &snapshot_id,
            &source_hashes,
            &criteria_hash,
            &registry_version,
        ))
        .expect("verification snapshot is serializable");
        let content_hash = format!("sha256:{:x}", Sha256::digest(bytes));
        Self {
            snapshot_id,
            source_hashes,
            criteria_hash,
            registry_version,
            frozen: true,
            content_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationAssignment {
    pub assignment_id: String,
    pub builder_identity: String,
    pub verifier_identity: String,
    pub provider_class: String,
    pub obligation_ids: Vec<String>,
    pub snapshot_hash: String,
    pub approved_tool_ids: Vec<String>,
    pub writer_lease: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub snapshot_hash: String,
    pub assignments: Vec<VerificationAssignment>,
    pub uncovered_mandatory: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerificationError {
    #[error("verification snapshot is not frozen")]
    SnapshotNotFrozen,
    #[error("mandatory obligation has no eligible independent verifier: {0}")]
    UncoveredMandatory(String),
    #[error("builder and verifier identities are not independent")]
    IdentityConflict,
    #[error("verification assignment may not hold a writer lease")]
    WriterLeaseConflict,
    #[error("verification snapshot hash changed")]
    SnapshotChanged,
    #[error("blocking finding remains unresolved: {0}")]
    BlockingFinding(String),
    #[error("mandatory obligation did not pass: {0}")]
    ObligationNotPassed(String),
    #[error("response references unknown assignment: {0}")]
    UnknownAssignment(String),
    #[error("response verdict is outside its assignment: {0}")]
    OutOfScopeVerdict(String),
    #[error("finding has no evidence: {0}")]
    MissingFindingEvidence(String),
}

pub fn route_verification(
    builder_identity: &str,
    snapshot: &VerificationSnapshot,
    obligations: &[VerificationObligation],
    profiles: &[VerifierCapabilityProfile],
) -> Result<VerificationPlan, VerificationError> {
    if !snapshot.frozen {
        return Err(VerificationError::SnapshotNotFrozen);
    }
    let mut assignments: BTreeMap<String, VerificationAssignment> = BTreeMap::new();
    let mut uncovered = Vec::new();
    for obligation in obligations {
        let selected = profiles
            .iter()
            .filter(|profile| {
                profile.eligible()
                    && profile.verifier_id != builder_identity
                    && profile.risk_classes.contains(&obligation.risk_class)
            })
            .min_by(|a, b| {
                (a.provider_class.as_str(), a.verifier_id.as_str())
                    .cmp(&(b.provider_class.as_str(), b.verifier_id.as_str()))
            });
        let Some(profile) = selected else {
            if obligation.mandatory {
                uncovered.push(obligation.obligation_id.clone());
            }
            continue;
        };
        assignments
            .entry(profile.verifier_id.clone())
            .or_insert_with(|| VerificationAssignment {
                assignment_id: format!("verify-{}", profile.verifier_id),
                builder_identity: builder_identity.to_string(),
                verifier_identity: profile.verifier_id.clone(),
                provider_class: profile.provider_class.clone(),
                obligation_ids: Vec::new(),
                snapshot_hash: snapshot.content_hash.clone(),
                approved_tool_ids: profile.approved_tool_ids.clone(),
                writer_lease: false,
            })
            .obligation_ids
            .push(obligation.obligation_id.clone());
    }
    if let Some(id) = uncovered.first() {
        return Err(VerificationError::UncoveredMandatory(id.clone()));
    }
    Ok(VerificationPlan {
        snapshot_hash: snapshot.content_hash.clone(),
        assignments: assignments.into_values().collect(),
        uncovered_mandatory: uncovered,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingDisposition {
    Open,
    Accepted,
    Rejected,
    Remediated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationFinding {
    pub finding_id: String,
    pub obligation_id: String,
    pub blocking: bool,
    pub evidence_refs: Vec<String>,
    pub disposition: FindingDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObligationVerdict {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub assignment_id: String,
    pub snapshot_hash: String,
    pub verdicts: BTreeMap<String, ObligationVerdict>,
    pub findings: Vec<VerificationFinding>,
}

pub fn settle_verification(
    plan: &VerificationPlan,
    responses: &[VerificationResponse],
    mandatory_ids: &BTreeSet<String>,
) -> Result<(), VerificationError> {
    for assignment in &plan.assignments {
        if assignment.builder_identity == assignment.verifier_identity {
            return Err(VerificationError::IdentityConflict);
        }
        if assignment.writer_lease {
            return Err(VerificationError::WriterLeaseConflict);
        }
        if assignment.snapshot_hash != plan.snapshot_hash {
            return Err(VerificationError::SnapshotChanged);
        }
    }
    let assignment_obligations: BTreeMap<_, BTreeSet<_>> = plan
        .assignments
        .iter()
        .map(|assignment| {
            (
                assignment.assignment_id.as_str(),
                assignment
                    .obligation_ids
                    .iter()
                    .map(String::as_str)
                    .collect(),
            )
        })
        .collect();
    let mut verdicts = BTreeMap::new();
    for response in responses {
        if response.snapshot_hash != plan.snapshot_hash {
            return Err(VerificationError::SnapshotChanged);
        }
        let Some(assigned) = assignment_obligations.get(response.assignment_id.as_str()) else {
            return Err(VerificationError::UnknownAssignment(
                response.assignment_id.clone(),
            ));
        };
        for obligation_id in response.verdicts.keys() {
            if !assigned.contains(obligation_id.as_str()) {
                return Err(VerificationError::OutOfScopeVerdict(obligation_id.clone()));
            }
        }
        for finding in &response.findings {
            if finding.evidence_refs.is_empty() {
                return Err(VerificationError::MissingFindingEvidence(
                    finding.finding_id.clone(),
                ));
            }
            if finding.blocking && finding.disposition == FindingDisposition::Open {
                return Err(VerificationError::BlockingFinding(
                    finding.finding_id.clone(),
                ));
            }
        }
        verdicts.extend(response.verdicts.clone());
    }
    for id in mandatory_ids {
        if verdicts.get(id) != Some(&ObligationVerdict::Pass) {
            return Err(VerificationError::ObligationNotPassed(id.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "semantic_verification_tests.rs"]
mod tests;
