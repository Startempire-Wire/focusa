//! Spec140A foundational instruction integrity, amendment, and headless enforcement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionAdaptabilityClass {
    Invariant,
    TemporallyAdaptive,
    OperatorSelectable,
    ImplementationDiscretion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalAdaptationField {
    Scheduling,
    CheckpointFrequency,
    RetryBackoff,
    PollingCadence,
    NotificationCadence,
    MaintenanceWindow,
    EvidenceRefreshCadence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalInstructionAdaptation {
    pub adaptation_id: String,
    pub field: TemporalAdaptationField,
    pub prior_value: String,
    pub adapted_value: String,
    pub temporal_claim_ref: String,
    pub effective_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedTargetProfile {
    pub profile_id: String,
    pub revision: u64,
    pub environment_ref: String,
    pub required_capability_refs: Vec<String>,
    pub contingency_profile_ref: Option<String>,
    pub approved_by_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepDisposition {
    Changed,
    ReviewedNoChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentationSweepEntry {
    pub source_ref: String,
    pub before_sha256: String,
    pub after_sha256: String,
    pub disposition: SweepDisposition,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialDocumentationSweepManifest {
    pub manifest_id: String,
    pub affected_source_refs: Vec<String>,
    pub entries: Vec<DocumentationSweepEntry>,
    pub completed_at: DateTime<Utc>,
    pub reviewer_ref: String,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmendmentStatus {
    Proposed,
    Activated,
    Superseded,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalInstructionAmendment {
    pub amendment_id: String,
    pub scope_ref: String,
    pub prior_revision: u64,
    pub next_revision: u64,
    pub changed_instruction_refs: Vec<String>,
    pub proposed_by_operator_ref: String,
    pub proposal_receipt_ref: String,
    pub approved_by_operator_ref: Option<String>,
    pub approval_receipt_ref: Option<String>,
    pub sweep_manifest: Option<OfficialDocumentationSweepManifest>,
    pub effective_at: Option<DateTime<Utc>>,
    pub status: AmendmentStatus,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessCapabilityParity {
    pub effective_instruction_read: bool,
    pub source_inspection: bool,
    pub conflict_inspection: bool,
    pub integrity_guard_evaluation: bool,
    pub amendment_proposal: bool,
    pub amendment_activation: bool,
    pub drift_detection: bool,
    pub rollback: bool,
    pub evidence_receipts: bool,
    pub mission_canvas_independent: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionIntegrityRequest {
    pub scope_ref: String,
    pub expected_invariant_hashes: BTreeMap<String, String>,
    pub effective_invariant_hashes: BTreeMap<String, String>,
    pub dynamic_authority_available: bool,
    pub durable_or_consequential_action: bool,
    pub mission_canvas_available: bool,
    pub approved_target_profiles: Vec<ApprovedTargetProfile>,
    pub selected_target_profile_ref: String,
    pub temporal_adaptations: Vec<TemporalInstructionAdaptation>,
    pub active_constitution_revision: u64,
    pub observed_constitution_revision: u64,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionIntegrityDecision {
    Authorized,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionIntegrityResult {
    pub scope_ref: String,
    pub decision: InstructionIntegrityDecision,
    pub reason_codes: Vec<String>,
    pub mission_canvas_authoritative: bool,
    pub active_constitution_revision: u64,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionIntegrityError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    InvalidHash,
    AdaptationExpired,
    InvalidAdaptationWindow,
    TargetProfileInvalid,
    AmendmentRevisionInvalid,
    AmendmentProposalAuthorityMissing,
    AmendmentApprovalMissing,
    AmendmentSweepMissing,
    AmendmentSweepIncomplete,
    AmendmentEffectivityMissing,
    HeadlessParityIncomplete,
    ReplicaEpochStale,
}

pub fn validate_temporal_adaptation(
    adaptation: &TemporalInstructionAdaptation,
    now: DateTime<Utc>,
) -> Result<(), InstructionIntegrityError> {
    if adaptation.adaptation_id.trim().is_empty() || adaptation.temporal_claim_ref.trim().is_empty()
    {
        return Err(InstructionIntegrityError::MissingIdentity);
    }
    if adaptation.effective_at >= adaptation.expires_at {
        return Err(InstructionIntegrityError::InvalidAdaptationWindow);
    }
    if adaptation.expires_at <= now {
        return Err(InstructionIntegrityError::AdaptationExpired);
    }
    require_proof(&adaptation.evidence_refs, &adaptation.receipt_ref)
}

pub fn validate_target_profile(
    profile: &ApprovedTargetProfile,
) -> Result<(), InstructionIntegrityError> {
    if profile.profile_id.trim().is_empty()
        || profile.revision == 0
        || profile.environment_ref.trim().is_empty()
        || profile.approved_by_ref.trim().is_empty()
        || profile.required_capability_refs.is_empty()
        || profile.evidence_refs.is_empty()
    {
        return Err(InstructionIntegrityError::TargetProfileInvalid);
    }
    Ok(())
}

pub fn validate_amendment_proposal(
    amendment: &CanonicalInstructionAmendment,
) -> Result<(), InstructionIntegrityError> {
    if amendment.amendment_id.trim().is_empty()
        || amendment.scope_ref.trim().is_empty()
        || amendment.changed_instruction_refs.is_empty()
    {
        return Err(InstructionIntegrityError::MissingIdentity);
    }
    if amendment.next_revision != amendment.prior_revision.saturating_add(1) {
        return Err(InstructionIntegrityError::AmendmentRevisionInvalid);
    }
    if amendment.proposed_by_operator_ref.trim().is_empty()
        || amendment.proposal_receipt_ref.trim().is_empty()
    {
        return Err(InstructionIntegrityError::AmendmentProposalAuthorityMissing);
    }
    if amendment.evidence_refs.is_empty() {
        return Err(InstructionIntegrityError::MissingEvidence);
    }
    Ok(())
}

pub fn validate_amendment_activation(
    amendment: &CanonicalInstructionAmendment,
) -> Result<(), InstructionIntegrityError> {
    validate_amendment_proposal(amendment)?;
    if amendment.status != AmendmentStatus::Activated
        || amendment
            .approved_by_operator_ref
            .as_deref()
            .is_none_or(str::is_empty)
        || amendment
            .approval_receipt_ref
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return Err(InstructionIntegrityError::AmendmentApprovalMissing);
    }
    if amendment.effective_at.is_none() {
        return Err(InstructionIntegrityError::AmendmentEffectivityMissing);
    }
    let manifest = amendment
        .sweep_manifest
        .as_ref()
        .ok_or(InstructionIntegrityError::AmendmentSweepMissing)?;
    let affected: BTreeSet<_> = manifest.affected_source_refs.iter().collect();
    let entries: BTreeSet<_> = manifest
        .entries
        .iter()
        .map(|entry| &entry.source_ref)
        .collect();
    if affected.is_empty()
        || affected != entries
        || manifest.reviewer_ref.trim().is_empty()
        || manifest.receipt_ref.trim().is_empty()
    {
        return Err(InstructionIntegrityError::AmendmentSweepIncomplete);
    }
    for entry in &manifest.entries {
        if !valid_sha(&entry.before_sha256)
            || !valid_sha(&entry.after_sha256)
            || entry.evidence_refs.is_empty()
        {
            return Err(InstructionIntegrityError::AmendmentSweepIncomplete);
        }
        if entry.disposition == SweepDisposition::ReviewedNoChange
            && entry.before_sha256 != entry.after_sha256
        {
            return Err(InstructionIntegrityError::AmendmentSweepIncomplete);
        }
        if entry.disposition == SweepDisposition::Changed
            && entry.before_sha256 == entry.after_sha256
        {
            return Err(InstructionIntegrityError::AmendmentSweepIncomplete);
        }
    }
    Ok(())
}

pub fn evaluate_instruction_integrity(
    request: &InstructionIntegrityRequest,
    now: DateTime<Utc>,
) -> InstructionIntegrityResult {
    let mut reasons = Vec::new();
    if request.scope_ref.trim().is_empty()
        || request.evidence_refs.is_empty()
        || request.receipt_ref.trim().is_empty()
    {
        reasons.push("integrity_request_proof_missing".into());
    }
    if request.expected_invariant_hashes.is_empty()
        || request.expected_invariant_hashes != request.effective_invariant_hashes
        || request
            .expected_invariant_hashes
            .values()
            .any(|hash| !valid_sha(hash))
    {
        reasons.push("foundational_instruction_drift".into());
    }
    if request.durable_or_consequential_action && !request.dynamic_authority_available {
        reasons.push("dynamic_authority_unavailable_fail_closed".into());
    }
    if request.observed_constitution_revision != request.active_constitution_revision {
        reasons.push("stale_constitution_revision".into());
    }
    let selected = request
        .approved_target_profiles
        .iter()
        .find(|profile| profile.profile_id == request.selected_target_profile_ref);
    if selected.is_none()
        || selected.is_some_and(|profile| validate_target_profile(profile).is_err())
    {
        reasons.push("target_profile_not_approved".into());
    }
    for adaptation in &request.temporal_adaptations {
        if validate_temporal_adaptation(adaptation, now).is_err() {
            reasons.push("temporal_adaptation_invalid".into());
        }
    }
    InstructionIntegrityResult {
        scope_ref: request.scope_ref.clone(),
        decision: if reasons.is_empty() {
            InstructionIntegrityDecision::Authorized
        } else {
            InstructionIntegrityDecision::Blocked
        },
        reason_codes: reasons,
        mission_canvas_authoritative: false,
        active_constitution_revision: request.active_constitution_revision,
        evidence_refs: request.evidence_refs.clone(),
        receipt_ref: request.receipt_ref.clone(),
    }
}

pub fn validate_headless_parity(
    parity: &HeadlessCapabilityParity,
) -> Result<(), InstructionIntegrityError> {
    if ![
        parity.effective_instruction_read,
        parity.source_inspection,
        parity.conflict_inspection,
        parity.integrity_guard_evaluation,
        parity.amendment_proposal,
        parity.amendment_activation,
        parity.drift_detection,
        parity.rollback,
        parity.evidence_receipts,
        parity.mission_canvas_independent,
    ]
    .into_iter()
    .all(|value| value)
    {
        return Err(InstructionIntegrityError::HeadlessParityIncomplete);
    }
    require_proof(&parity.evidence_refs, &parity.receipt_ref)
}

pub fn validate_replica_activation(
    active_revision: u64,
    candidate_revision: u64,
    evidence_refs: &[String],
    receipt_ref: &str,
) -> Result<(), InstructionIntegrityError> {
    if candidate_revision != active_revision.saturating_add(1) {
        return Err(InstructionIntegrityError::ReplicaEpochStale);
    }
    require_proof(evidence_refs, receipt_ref)
}

fn require_proof(
    evidence_refs: &[String],
    receipt_ref: &str,
) -> Result<(), InstructionIntegrityError> {
    if evidence_refs.is_empty() {
        Err(InstructionIntegrityError::MissingEvidence)
    } else if receipt_ref.trim().is_empty() {
        Err(InstructionIntegrityError::MissingReceipt)
    } else {
        Ok(())
    }
}
fn valid_sha(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> ApprovedTargetProfile {
        ApprovedTargetProfile {
            profile_id: "production".into(),
            revision: 1,
            environment_ref: "env:prod".into(),
            required_capability_refs: vec!["capability:ssh".into()],
            contingency_profile_ref: Some("recovery".into()),
            approved_by_ref: "operator:approval".into(),
            evidence_refs: vec!["evidence:profile".into()],
        }
    }
    fn request() -> InstructionIntegrityRequest {
        InstructionIntegrityRequest {
            scope_ref: "project:focusa".into(),
            expected_invariant_hashes: BTreeMap::from([("foundation".into(), "a".repeat(64))]),
            effective_invariant_hashes: BTreeMap::from([("foundation".into(), "a".repeat(64))]),
            dynamic_authority_available: true,
            durable_or_consequential_action: true,
            mission_canvas_available: false,
            approved_target_profiles: vec![profile()],
            selected_target_profile_ref: "production".into(),
            temporal_adaptations: vec![],
            active_constitution_revision: 2,
            observed_constitution_revision: 2,
            evidence_refs: vec!["evidence:guard".into()],
            receipt_ref: "receipt:guard".into(),
        }
    }
    #[test]
    fn headless_guard_authorizes_without_mission_canvas() {
        let result = evaluate_instruction_integrity(&request(), Utc::now());
        assert_eq!(result.decision, InstructionIntegrityDecision::Authorized);
        assert!(!result.mission_canvas_authoritative);
    }
    #[test]
    fn guard_blocks_drift_outage_stale_revision_and_target_invention() {
        let mut value = request();
        value.dynamic_authority_available = false;
        value.observed_constitution_revision = 1;
        value.selected_target_profile_ref = "invented".into();
        value
            .effective_invariant_hashes
            .insert("foundation".into(), "b".repeat(64));
        let result = evaluate_instruction_integrity(&value, Utc::now());
        assert_eq!(result.decision, InstructionIntegrityDecision::Blocked);
        assert_eq!(result.reason_codes.len(), 4);
    }
    #[test]
    fn amendment_requires_second_approval_and_complete_documentation_sweep() {
        let mut amendment = CanonicalInstructionAmendment {
            amendment_id: "amendment:1".into(),
            scope_ref: "project:focusa".into(),
            prior_revision: 1,
            next_revision: 2,
            changed_instruction_refs: vec!["instruction:one".into()],
            proposed_by_operator_ref: "operator:proposal".into(),
            proposal_receipt_ref: "receipt:proposal".into(),
            approved_by_operator_ref: None,
            approval_receipt_ref: None,
            sweep_manifest: None,
            effective_at: None,
            status: AmendmentStatus::Proposed,
            evidence_refs: vec!["evidence:amendment".into()],
        };
        assert!(validate_amendment_proposal(&amendment).is_ok());
        assert_eq!(
            validate_amendment_activation(&amendment),
            Err(InstructionIntegrityError::AmendmentApprovalMissing)
        );
        amendment.status = AmendmentStatus::Activated;
        amendment.approved_by_operator_ref = Some("operator:approval".into());
        amendment.approval_receipt_ref = Some("receipt:approval".into());
        amendment.effective_at = Some(Utc::now());
        amendment.sweep_manifest = Some(OfficialDocumentationSweepManifest {
            manifest_id: "sweep:1".into(),
            affected_source_refs: vec!["AGENTS.md".into()],
            entries: vec![DocumentationSweepEntry {
                source_ref: "AGENTS.md".into(),
                before_sha256: "a".repeat(64),
                after_sha256: "b".repeat(64),
                disposition: SweepDisposition::Changed,
                evidence_refs: vec!["diff:agents".into()],
            }],
            completed_at: Utc::now(),
            reviewer_ref: "operator:review".into(),
            receipt_ref: "receipt:sweep".into(),
        });
        assert!(validate_amendment_activation(&amendment).is_ok());
    }
    #[test]
    fn headless_parity_and_replica_activation_fail_closed() {
        let parity = HeadlessCapabilityParity {
            effective_instruction_read: true,
            source_inspection: true,
            conflict_inspection: true,
            integrity_guard_evaluation: true,
            amendment_proposal: true,
            amendment_activation: true,
            drift_detection: true,
            rollback: true,
            evidence_receipts: true,
            mission_canvas_independent: true,
            evidence_refs: vec!["evidence:headless".into()],
            receipt_ref: "receipt:headless".into(),
        };
        assert!(validate_headless_parity(&parity).is_ok());
        assert_eq!(
            validate_replica_activation(3, 3, &["evidence".into()], "receipt"),
            Err(InstructionIntegrityError::ReplicaEpochStale)
        );
    }
}
