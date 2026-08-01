//! Spec137A tranche ordering, settlement, merge, and publication truth gates.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrancheState {
    ProofPending,
    VerifiedSlice,
    Settled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalSettlement {
    pub factual_completion_proven: bool,
    pub temporal_outcome_ref: String,
    pub target_missed: bool,
    pub missed_target_receipt_ref: Option<String>,
    pub disposition_ref: String,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrancheSettlement {
    pub tranche_id: String,
    pub depends_on: Vec<String>,
    pub state: TrancheState,
    pub open_requirement_refs: Vec<String>,
    pub unsupported_requirement_refs: Vec<String>,
    pub settlement: Option<TemporalSettlement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseConformanceRequest {
    pub parent_complete_claimed: bool,
    pub publication_requested: bool,
    pub full_conformance_receipt_ref: Option<String>,
    pub tranches: Vec<TrancheSettlement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseGateError {
    DuplicateTranche,
    UnknownDependency(String),
    DependencyUnsettled(String),
    OpenRequirements(String),
    UnsupportedRequirements(String),
    MissingSettlement(String),
    MissingSettlementProof(String),
    MissingMissedTargetReceipt(String),
    PartialClaim,
    PublicationBlocked,
}

pub fn validate_release_conformance(
    request: &ReleaseConformanceRequest,
) -> Result<(), ReleaseGateError> {
    let mut by_id = BTreeMap::new();
    for tranche in &request.tranches {
        if by_id.insert(tranche.tranche_id.clone(), tranche).is_some() {
            return Err(ReleaseGateError::DuplicateTranche);
        }
    }
    for tranche in &request.tranches {
        for dependency in &tranche.depends_on {
            let dependency = by_id
                .get(dependency)
                .ok_or_else(|| ReleaseGateError::UnknownDependency(dependency.clone()))?;
            if matches!(tranche.state, TrancheState::Settled)
                && !matches!(dependency.state, TrancheState::Settled)
            {
                return Err(ReleaseGateError::DependencyUnsettled(
                    tranche.tranche_id.clone(),
                ));
            }
        }
        if !tranche.open_requirement_refs.is_empty() {
            return Err(ReleaseGateError::OpenRequirements(
                tranche.tranche_id.clone(),
            ));
        }
        if !tranche.unsupported_requirement_refs.is_empty() {
            return Err(ReleaseGateError::UnsupportedRequirements(
                tranche.tranche_id.clone(),
            ));
        }
        if matches!(tranche.state, TrancheState::Settled) {
            let settlement = tranche
                .settlement
                .as_ref()
                .ok_or_else(|| ReleaseGateError::MissingSettlement(tranche.tranche_id.clone()))?;
            if !settlement.factual_completion_proven
                || settlement.temporal_outcome_ref.trim().is_empty()
                || settlement.disposition_ref.trim().is_empty()
                || settlement.evidence_refs.is_empty()
                || settlement.receipt_refs.is_empty()
            {
                return Err(ReleaseGateError::MissingSettlementProof(
                    tranche.tranche_id.clone(),
                ));
            }
            if settlement.target_missed && settlement.missed_target_receipt_ref.is_none() {
                return Err(ReleaseGateError::MissingMissedTargetReceipt(
                    tranche.tranche_id.clone(),
                ));
            }
        }
    }
    let all_settled = !request.tranches.is_empty()
        && request
            .tranches
            .iter()
            .all(|tranche| matches!(tranche.state, TrancheState::Settled));
    if request.parent_complete_claimed && !all_settled {
        return Err(ReleaseGateError::PartialClaim);
    }
    if request.publication_requested
        && (!all_settled || request.full_conformance_receipt_ref.is_none())
    {
        return Err(ReleaseGateError::PublicationBlocked);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settlement(missed: bool) -> TemporalSettlement {
        TemporalSettlement {
            factual_completion_proven: true,
            temporal_outcome_ref: "temporal:outcome".into(),
            target_missed: missed,
            missed_target_receipt_ref: missed.then(|| "receipt:missed".into()),
            disposition_ref: "disposition:accepted".into(),
            evidence_refs: vec!["evidence".into()],
            receipt_refs: vec!["receipt".into()],
        }
    }

    #[test]
    fn pending_or_unsupported_work_blocks_completion_and_publication() {
        let request = ReleaseConformanceRequest {
            parent_complete_claimed: true,
            publication_requested: true,
            full_conformance_receipt_ref: None,
            tranches: vec![TrancheSettlement {
                tranche_id: "runtime".into(),
                depends_on: vec![],
                state: TrancheState::ProofPending,
                open_requirement_refs: vec!["S137A-1".into()],
                unsupported_requirement_refs: vec![],
                settlement: None,
            }],
        };
        assert!(matches!(
            validate_release_conformance(&request),
            Err(ReleaseGateError::OpenRequirements(_))
        ));
    }

    #[test]
    fn settled_tranche_requires_settled_dependencies() {
        let request = ReleaseConformanceRequest {
            parent_complete_claimed: false,
            publication_requested: false,
            full_conformance_receipt_ref: None,
            tranches: vec![
                TrancheSettlement {
                    tranche_id: "a".into(),
                    depends_on: vec![],
                    state: TrancheState::VerifiedSlice,
                    open_requirement_refs: vec![],
                    unsupported_requirement_refs: vec![],
                    settlement: None,
                },
                TrancheSettlement {
                    tranche_id: "b".into(),
                    depends_on: vec!["a".into()],
                    state: TrancheState::Settled,
                    open_requirement_refs: vec![],
                    unsupported_requirement_refs: vec![],
                    settlement: Some(settlement(false)),
                },
            ],
        };
        assert_eq!(
            validate_release_conformance(&request),
            Err(ReleaseGateError::DependencyUnsettled("b".into()))
        );
    }

    #[test]
    fn complete_settled_chain_can_publish_with_full_receipt() {
        let request = ReleaseConformanceRequest {
            parent_complete_claimed: true,
            publication_requested: true,
            full_conformance_receipt_ref: Some("receipt:full".into()),
            tranches: vec![TrancheSettlement {
                tranche_id: "all".into(),
                depends_on: vec![],
                state: TrancheState::Settled,
                open_requirement_refs: vec![],
                unsupported_requirement_refs: vec![],
                settlement: Some(settlement(true)),
            }],
        };
        assert!(validate_release_conformance(&request).is_ok());
    }
}
