//! Spec 144 §24 complete verification settlement evaluation.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementEvidence {
    pub evidence_ref: String,
    pub requirement_ids: BTreeSet<String>,
    pub fresh: bool,
    pub validation_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementInput {
    pub contract_revision: u64,
    pub verified_contract_revision: u64,
    pub workpoint_revision: u64,
    pub verified_workpoint_revision: u64,
    pub final_snapshot_hash: String,
    pub verified_snapshot_hash: String,
    pub mandatory_requirement_ids: BTreeSet<String>,
    pub passed_requirement_ids: BTreeSet<String>,
    pub evidence: Vec<SettlementEvidence>,
    pub verifier_calibration_valid: bool,
    pub verifier_eligible: bool,
    pub verifier_independent: bool,
    pub temporal_authority_valid: bool,
    pub epistemic_authority_valid: bool,
    pub pack_conflicts: BTreeSet<String>,
    pub required_approval_ids: BTreeSet<String>,
    pub received_approval_ids: BTreeSet<String>,
    pub migration_verified: bool,
    pub client_parity_verified: bool,
    pub receipt_ready: bool,
    pub runtime_variance_ids: BTreeSet<String>,
    pub partial_settlement_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementStatus {
    SettledFull,
    SettledPartial,
    Blocked,
    OperatorRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementEvaluation {
    pub status: SettlementStatus,
    pub settled_requirement_ids: BTreeSet<String>,
    pub unsettled_requirement_ids: BTreeSet<String>,
    pub blocker_codes: Vec<String>,
    pub evidence_by_requirement: BTreeMap<String, Vec<String>>,
    pub closure_allowed: bool,
}

pub fn evaluate_settlement(input: &SettlementInput) -> SettlementEvaluation {
    let mut blockers = Vec::new();
    if input.contract_revision != input.verified_contract_revision {
        blockers.push("contract_revision_changed".into());
    }
    if input.workpoint_revision != input.verified_workpoint_revision {
        blockers.push("workpoint_revision_changed".into());
    }
    if input.final_snapshot_hash != input.verified_snapshot_hash {
        blockers.push("snapshot_changed".into());
    }
    for (valid, code) in [
        (input.verifier_calibration_valid, "calibration_invalid"),
        (input.verifier_eligible, "verifier_ineligible"),
        (input.verifier_independent, "verifier_not_independent"),
        (input.temporal_authority_valid, "temporal_authority_invalid"),
        (
            input.epistemic_authority_valid,
            "epistemic_authority_invalid",
        ),
        (input.migration_verified, "migration_unverified"),
        (input.client_parity_verified, "client_parity_unverified"),
        (input.receipt_ready, "receipt_not_ready"),
    ] {
        if !valid {
            blockers.push(code.into());
        }
    }
    if !input.pack_conflicts.is_empty() {
        blockers.push("pack_conflict".into());
    }
    if !input.runtime_variance_ids.is_empty() {
        blockers.push("runtime_variance".into());
    }
    let missing_approvals: BTreeSet<_> = input
        .required_approval_ids
        .difference(&input.received_approval_ids)
        .cloned()
        .collect();
    if !missing_approvals.is_empty() {
        blockers.push("operator_approval_required".into());
    }
    let mut evidence_by_requirement: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for evidence in &input.evidence {
        if !evidence.fresh || evidence.validation_receipt_ref.is_none() {
            blockers.push("evidence_unfresh_or_unvalidated".into());
            continue;
        }
        for requirement_id in &evidence.requirement_ids {
            evidence_by_requirement
                .entry(requirement_id.clone())
                .or_default()
                .push(evidence.evidence_ref.clone());
        }
    }
    let evidence_covered: BTreeSet<_> = evidence_by_requirement.keys().cloned().collect();
    let settled_requirement_ids: BTreeSet<_> = input
        .passed_requirement_ids
        .intersection(&evidence_covered)
        .cloned()
        .collect();
    let unsettled_requirement_ids: BTreeSet<_> = input
        .mandatory_requirement_ids
        .difference(&settled_requirement_ids)
        .cloned()
        .collect();
    blockers.sort();
    blockers.dedup();
    let approvals_only = blockers == vec!["operator_approval_required"];
    let status = if !blockers.is_empty() {
        if approvals_only {
            SettlementStatus::OperatorRequired
        } else {
            SettlementStatus::Blocked
        }
    } else if unsettled_requirement_ids.is_empty() {
        SettlementStatus::SettledFull
    } else if input.partial_settlement_allowed && !settled_requirement_ids.is_empty() {
        SettlementStatus::SettledPartial
    } else {
        blockers.push("mandatory_requirement_unsettled".into());
        SettlementStatus::Blocked
    };
    let closure_allowed = status == SettlementStatus::SettledFull;
    SettlementEvaluation {
        status,
        settled_requirement_ids,
        unsettled_requirement_ids,
        blocker_codes: blockers,
        evidence_by_requirement,
        closure_allowed,
    }
}

#[cfg(test)]
#[path = "semantic_settlement_tests.rs"]
mod tests;
