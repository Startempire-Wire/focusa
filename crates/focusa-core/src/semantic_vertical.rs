//! Spec 144 §§19-22: bounded verification lifecycle, calibration, learning, and Verticals.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Building,
    Built,
    VerificationPlanned,
    Verifying,
    VerificationBlocked,
    VerificationPassed,
    ReworkRequired,
    Settled,
    OscillationDetected,
    SnapshotInvalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationLifecycle {
    pub state: LifecycleState,
    pub snapshot_hash: String,
    pub obligation_ids: BTreeSet<String>,
    pub preserved_finding_ids: BTreeSet<String>,
    pub reroute_count: u32,
    pub max_reroutes: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerticalGovernanceError {
    #[error("verification lifecycle transition is invalid")]
    InvalidTransition,
    #[error("verification reroute budget exhausted")]
    RerouteBudgetExhausted,
    #[error("verifier cohort has no eligible champion")]
    NoEligibleChampion,
    #[error("challenger is not independent from champion")]
    ChallengerNotIndependent,
    #[error("learning proposal lacks causal evidence")]
    MissingCausalEvidence,
    #[error("cross-domain learning lacks transfer evidence")]
    MissingTransferEvidence,
    #[error("learning proposal may not mutate policy automatically")]
    AutomaticMutationForbidden,
    #[error("learning promotion requires operator approval")]
    OperatorApprovalRequired,
    #[error("vertical bundle signature is missing")]
    MissingSignature,
    #[error("vertical bundle signature is not verified")]
    SignatureUnverified,
    #[error("vertical bundle is missing required semantic components")]
    IncompleteBundle,
    #[error("vertical bundle digest does not match content")]
    DigestMismatch,
    #[error("vertical bundle is outside its activation window")]
    OutsideActivationWindow,
    #[error("critical contradiction remains unresolved: {0}")]
    UnresolvedContradiction(String),
}

impl VerificationLifecycle {
    pub fn transition(&mut self, next: LifecycleState) -> Result<(), VerticalGovernanceError> {
        use LifecycleState::*;
        let allowed = matches!(
            (&self.state, &next),
            (Building, Built)
                | (Built, VerificationPlanned)
                | (VerificationPlanned, Verifying)
                | (Verifying, VerificationPassed)
                | (Verifying, VerificationBlocked)
                | (Verifying, ReworkRequired)
                | (VerificationBlocked, VerificationPlanned)
                | (ReworkRequired, Building)
                | (VerificationPassed, Settled)
        );
        if !allowed {
            return Err(VerticalGovernanceError::InvalidTransition);
        }
        self.state = next;
        Ok(())
    }

    pub fn reroute(
        &mut self,
        new_snapshot_hash: impl Into<String>,
        new_obligations: impl IntoIterator<Item = String>,
        findings: impl IntoIterator<Item = String>,
    ) -> Result<(), VerticalGovernanceError> {
        if self.reroute_count >= self.max_reroutes {
            self.state = LifecycleState::OscillationDetected;
            return Err(VerticalGovernanceError::RerouteBudgetExhausted);
        }
        self.reroute_count += 1;
        self.snapshot_hash = new_snapshot_hash.into();
        self.obligation_ids.extend(new_obligations);
        self.preserved_finding_ids.extend(findings);
        self.state = LifecycleState::VerificationPlanned;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifierCalibration {
    pub verifier_id: String,
    pub provider_class: String,
    pub domain: String,
    pub sample_count: u32,
    pub precision: f64,
    pub recall: f64,
    pub brier_score: f64,
    pub evidence_refs: Vec<String>,
    pub valid_until: DateTime<Utc>,
}

impl VerifierCalibration {
    pub fn eligible_at(&self, now: DateTime<Utc>) -> bool {
        self.sample_count >= 20
            && self.precision >= 0.80
            && self.recall >= 0.80
            && self.brier_score <= 0.20
            && !self.evidence_refs.is_empty()
            && now <= self.valid_until
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifierCohort {
    pub champion: VerifierCalibration,
    pub challengers: Vec<VerifierCalibration>,
}

impl VerifierCohort {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), VerticalGovernanceError> {
        if !self.champion.eligible_at(now) {
            return Err(VerticalGovernanceError::NoEligibleChampion);
        }
        if self.challengers.iter().any(|item| {
            item.verifier_id == self.champion.verifier_id
                || item.provider_class == self.champion.provider_class
        }) {
            return Err(VerticalGovernanceError::ChallengerNotIndependent);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningProposal {
    pub proposal_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub causal_evidence_refs: Vec<String>,
    pub transfer_evidence_refs: Vec<String>,
    pub operator_approved: bool,
    pub automatic_policy_mutation: bool,
}

impl LearningProposal {
    pub fn validate(&self) -> Result<(), VerticalGovernanceError> {
        if self.causal_evidence_refs.is_empty() {
            return Err(VerticalGovernanceError::MissingCausalEvidence);
        }
        if self.source_domain != self.target_domain && self.transfer_evidence_refs.is_empty() {
            return Err(VerticalGovernanceError::MissingTransferEvidence);
        }
        if self.automatic_policy_mutation {
            return Err(VerticalGovernanceError::AutomaticMutationForbidden);
        }
        Ok(())
    }

    pub fn authorize_promotion(&self) -> Result<(), VerticalGovernanceError> {
        self.validate()?;
        if !self.operator_approved {
            return Err(VerticalGovernanceError::OperatorApprovalRequired);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    Advisory,
    Blocking,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainContradiction {
    pub contradiction_id: String,
    pub left_claim_ref: String,
    pub right_claim_ref: String,
    pub severity: ContradictionSeverity,
    pub resolution_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerticalBundleContent {
    pub bundle_id: String,
    pub domain: String,
    pub version: String,
    pub ontology_refs: BTreeSet<String>,
    pub shape_refs: BTreeSet<String>,
    pub domain_graph: BTreeMap<String, BTreeSet<String>>,
    pub evidence_index: BTreeMap<String, Vec<String>>,
    pub calibration_profile_refs: BTreeSet<String>,
    pub contradictions: Vec<DomainContradiction>,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedVerticalBundle {
    pub content: VerticalBundleContent,
    pub content_hash: String,
    pub signer_id: String,
    pub signature: String,
    pub signature_verified: bool,
}

pub fn vertical_content_hash(content: &VerticalBundleContent) -> String {
    let bytes = serde_json::to_vec(content).expect("vertical content is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn validate_vertical_activation(
    bundle: &SignedVerticalBundle,
    now: DateTime<Utc>,
) -> Result<(), VerticalGovernanceError> {
    if bundle.signer_id.is_empty() || bundle.signature.is_empty() {
        return Err(VerticalGovernanceError::MissingSignature);
    }
    if !bundle.signature_verified {
        return Err(VerticalGovernanceError::SignatureUnverified);
    }
    if bundle.content.ontology_refs.is_empty()
        || bundle.content.shape_refs.is_empty()
        || bundle.content.domain_graph.is_empty()
        || bundle.content.evidence_index.is_empty()
        || bundle.content.calibration_profile_refs.is_empty()
    {
        return Err(VerticalGovernanceError::IncompleteBundle);
    }
    if bundle.content_hash != vertical_content_hash(&bundle.content) {
        return Err(VerticalGovernanceError::DigestMismatch);
    }
    if now < bundle.content.valid_from || now > bundle.content.expires_at {
        return Err(VerticalGovernanceError::OutsideActivationWindow);
    }
    if let Some(item) = bundle.content.contradictions.iter().find(|item| {
        item.severity == ContradictionSeverity::Critical && item.resolution_ref.is_none()
    }) {
        return Err(VerticalGovernanceError::UnresolvedContradiction(
            item.contradiction_id.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "semantic_vertical_tests.rs"]
mod tests;
