//! Spec137A fail-closed omission, disposition, and variance firewall.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureStatus {
    ProofPending,
    VerifiedComplete,
    VerifiedNotApplicable,
    VerifiedOptionalUnimplemented,
    VarianceVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShouldVarianceRecord {
    pub variance_id: String,
    pub exact_clause_ref: String,
    pub reason: String,
    pub risk: String,
    pub scope_refs: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub replacement_behavior_ref: String,
    pub test_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub closure_consequence: String,
    pub rollback_ref: String,
    pub operator_approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionFirewallRow {
    pub requirement_id: String,
    pub status: ClosureStatus,
    pub disposition: String,
    pub active: bool,
    pub implementation_refs: Vec<String>,
    pub test_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub variance: Option<ShouldVarianceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionFirewallInput {
    pub claim: String,
    pub full_conformance_claimed: bool,
    pub expected_requirement_count: usize,
    pub source_coverage_complete: bool,
    pub root_delivery_dag_complete: bool,
    pub rows: Vec<OmissionFirewallRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmissionFirewallError {
    RequirementCountMismatch,
    DuplicateRequirement,
    SourceCoverageOpen,
    DeliveryDagOpen,
    ForbiddenDisposition(String),
    BroadCompletionQualifier,
    ActiveRowNotComplete(String),
    MissingProof(String),
    InvalidVariance(String),
    ExpiredVariance(String),
}

pub fn validate_omission_firewall(
    input: &OmissionFirewallInput,
    now: DateTime<Utc>,
) -> Result<(), OmissionFirewallError> {
    if input.rows.len() != input.expected_requirement_count {
        return Err(OmissionFirewallError::RequirementCountMismatch);
    }
    if !input.source_coverage_complete {
        return Err(OmissionFirewallError::SourceCoverageOpen);
    }
    if !input.root_delivery_dag_complete {
        return Err(OmissionFirewallError::DeliveryDagOpen);
    }
    let claim = input.claim.to_ascii_lowercase();
    if input.full_conformance_claimed
        && [
            "mostly complete",
            "core complete",
            "schema complete",
            "substantially complete",
        ]
        .iter()
        .any(|term| claim.contains(term))
    {
        return Err(OmissionFirewallError::BroadCompletionQualifier);
    }
    let mut ids = BTreeSet::new();
    for row in &input.rows {
        if !ids.insert(row.requirement_id.clone()) {
            return Err(OmissionFirewallError::DuplicateRequirement);
        }
        let disposition = row.disposition.to_ascii_lowercase();
        if [
            "later",
            "eventually",
            "post-mvp",
            "deferred",
            "out of scope",
            "schema only",
        ]
        .iter()
        .any(|term| disposition.contains(term))
        {
            return Err(OmissionFirewallError::ForbiddenDisposition(
                row.requirement_id.clone(),
            ));
        }
        if row.active && matches!(row.status, ClosureStatus::VerifiedComplete) {
            if row.implementation_refs.is_empty()
                || row.test_refs.is_empty()
                || row.evidence_refs.is_empty()
                || row.receipt_refs.is_empty()
            {
                return Err(OmissionFirewallError::MissingProof(
                    row.requirement_id.clone(),
                ));
            }
        }
        if input.full_conformance_claimed
            && row.active
            && !matches!(row.status, ClosureStatus::VerifiedComplete)
        {
            return Err(OmissionFirewallError::ActiveRowNotComplete(
                row.requirement_id.clone(),
            ));
        }
        if matches!(row.status, ClosureStatus::VarianceVerified) {
            let variance = row.variance.as_ref().ok_or_else(|| {
                OmissionFirewallError::InvalidVariance(row.requirement_id.clone())
            })?;
            let complete = variance.operator_approved
                && !variance.exact_clause_ref.trim().is_empty()
                && !variance.reason.trim().is_empty()
                && !variance.risk.trim().is_empty()
                && !variance.scope_refs.is_empty()
                && !variance.replacement_behavior_ref.trim().is_empty()
                && !variance.test_refs.is_empty()
                && !variance.evidence_refs.is_empty()
                && !variance.receipt_refs.is_empty()
                && !variance.closure_consequence.trim().is_empty()
                && !variance.rollback_ref.trim().is_empty();
            if !complete {
                return Err(OmissionFirewallError::InvalidVariance(
                    row.requirement_id.clone(),
                ));
            }
            if variance.expires_at <= now {
                return Err(OmissionFirewallError::ExpiredVariance(
                    row.requirement_id.clone(),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceConformanceState {
    Implemented,
    Degraded,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceParityRecord {
    pub surface: String,
    pub state: SurfaceConformanceState,
    pub behavior_ref: String,
    pub evidence_refs: Vec<String>,
    pub recovery_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceParityError {
    MissingSurface(String),
    DuplicateSurface(String),
    UnknownSurface(String),
    MissingBehavior(String),
    MissingEvidence(String),
    MissingRecovery(String),
}

pub fn validate_surface_parity(
    records: &[SurfaceParityRecord],
    required_surfaces: &[&str],
) -> Result<(), SurfaceParityError> {
    let mut seen = BTreeSet::new();
    for record in records {
        if !seen.insert(record.surface.clone()) {
            return Err(SurfaceParityError::DuplicateSurface(record.surface.clone()));
        }
        if matches!(record.state, SurfaceConformanceState::Unknown) {
            return Err(SurfaceParityError::UnknownSurface(record.surface.clone()));
        }
        if record.behavior_ref.trim().is_empty() {
            return Err(SurfaceParityError::MissingBehavior(record.surface.clone()));
        }
        if record.evidence_refs.is_empty() {
            return Err(SurfaceParityError::MissingEvidence(record.surface.clone()));
        }
        if matches!(
            record.state,
            SurfaceConformanceState::Degraded | SurfaceConformanceState::Unsupported
        ) && record.recovery_ref.as_deref().is_none_or(str::is_empty)
        {
            return Err(SurfaceParityError::MissingRecovery(record.surface.clone()));
        }
    }
    for required in required_surfaces {
        if !seen.contains(*required) {
            return Err(SurfaceParityError::MissingSurface((*required).into()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(status: ClosureStatus) -> OmissionFirewallRow {
        OmissionFirewallRow {
            requirement_id: "S137A-1".into(),
            status,
            disposition: "implemented and proven".into(),
            active: true,
            implementation_refs: vec!["code".into()],
            test_refs: vec!["test".into()],
            evidence_refs: vec!["evidence".into()],
            receipt_refs: vec!["receipt".into()],
            variance: None,
        }
    }

    #[test]
    fn pending_row_blocks_full_conformance() {
        let input = OmissionFirewallInput {
            claim: "full conformance".into(),
            full_conformance_claimed: true,
            expected_requirement_count: 1,
            source_coverage_complete: true,
            root_delivery_dag_complete: true,
            rows: vec![row(ClosureStatus::ProofPending)],
        };
        assert!(matches!(
            validate_omission_firewall(&input, Utc::now()),
            Err(OmissionFirewallError::ActiveRowNotComplete(_))
        ));
    }

    #[test]
    fn hidden_deferral_and_broad_qualifier_fail_closed() {
        let mut deferred = row(ClosureStatus::ProofPending);
        deferred.disposition = "implement later".into();
        let input = OmissionFirewallInput {
            claim: "mostly complete".into(),
            full_conformance_claimed: false,
            expected_requirement_count: 1,
            source_coverage_complete: true,
            root_delivery_dag_complete: true,
            rows: vec![deferred],
        };
        assert!(matches!(
            validate_omission_firewall(&input, Utc::now()),
            Err(OmissionFirewallError::ForbiddenDisposition(_))
        ));
    }

    #[test]
    fn broad_completion_qualifier_is_never_a_full_claim() {
        let input = OmissionFirewallInput {
            claim: "core complete".into(),
            full_conformance_claimed: true,
            expected_requirement_count: 1,
            source_coverage_complete: true,
            root_delivery_dag_complete: true,
            rows: vec![row(ClosureStatus::VerifiedComplete)],
        };
        assert_eq!(
            validate_omission_firewall(&input, Utc::now()),
            Err(OmissionFirewallError::BroadCompletionQualifier)
        );
    }

    #[test]
    fn should_variance_requires_complete_unexpired_operator_approval() {
        let mut variance_row = row(ClosureStatus::VarianceVerified);
        variance_row.active = false;
        assert!(matches!(
            validate_omission_firewall(
                &OmissionFirewallInput {
                    claim: "verified variance".into(),
                    full_conformance_claimed: false,
                    expected_requirement_count: 1,
                    source_coverage_complete: true,
                    root_delivery_dag_complete: true,
                    rows: vec![variance_row.clone()],
                },
                Utc::now(),
            ),
            Err(OmissionFirewallError::InvalidVariance(_))
        ));
        variance_row.variance = Some(ShouldVarianceRecord {
            variance_id: "variance-1".into(),
            exact_clause_ref: "S137A:59".into(),
            reason: "bounded platform constraint".into(),
            risk: "reduced precision".into(),
            scope_refs: vec!["platform:test".into()],
            expires_at: Utc::now() - chrono::Duration::seconds(1),
            replacement_behavior_ref: "fallback:test".into(),
            test_refs: vec!["test:variance".into()],
            evidence_refs: vec!["evidence:variance".into()],
            receipt_refs: vec!["receipt:variance".into()],
            closure_consequence: "full conformance remains blocked".into(),
            rollback_ref: "rollback:variance".into(),
            operator_approved: true,
        });
        assert!(matches!(
            validate_omission_firewall(
                &OmissionFirewallInput {
                    claim: "verified variance".into(),
                    full_conformance_claimed: false,
                    expected_requirement_count: 1,
                    source_coverage_complete: true,
                    root_delivery_dag_complete: true,
                    rows: vec![variance_row],
                },
                Utc::now(),
            ),
            Err(OmissionFirewallError::ExpiredVariance(_))
        ));
    }

    #[test]
    fn surface_parity_rejects_unknown_and_requires_degraded_recovery() {
        let unknown = SurfaceParityRecord {
            surface: "api".into(),
            state: SurfaceConformanceState::Unknown,
            behavior_ref: "route".into(),
            evidence_refs: vec!["test".into()],
            recovery_ref: None,
        };
        assert_eq!(
            validate_surface_parity(&[unknown], &["api"]),
            Err(SurfaceParityError::UnknownSurface("api".into()))
        );
        let degraded = SurfaceParityRecord {
            surface: "platform".into(),
            state: SurfaceConformanceState::Degraded,
            behavior_ref: "unsupported remains open".into(),
            evidence_refs: vec!["test".into()],
            recovery_ref: None,
        };
        assert_eq!(
            validate_surface_parity(&[degraded], &["platform"]),
            Err(SurfaceParityError::MissingRecovery("platform".into()))
        );
    }

    #[test]
    fn proven_active_row_can_close() {
        let input = OmissionFirewallInput {
            claim: "full conformance".into(),
            full_conformance_claimed: true,
            expected_requirement_count: 1,
            source_coverage_complete: true,
            root_delivery_dag_complete: true,
            rows: vec![row(ClosureStatus::VerifiedComplete)],
        };
        assert!(validate_omission_firewall(&input, Utc::now()).is_ok());
    }
}
