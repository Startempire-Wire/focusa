use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::temporal::{TemporalConfidence, TemporalScope};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredTemporalScope {
    pub host: bool,
    pub operator: bool,
    pub project: bool,
    pub continuity: bool,
    pub workpoint: bool,
    pub item: bool,
    pub task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalScopeError {
    MissingProject,
    MissingContinuity,
    MissingHost,
    MissingOperator,
    MissingWorkpoint,
    MissingItem,
    MissingTask,
    ScopeMismatch,
}

pub fn validate_exact_scope(
    scope: &TemporalScope,
    required: &RequiredTemporalScope,
) -> Result<(), TemporalScopeError> {
    if required.project && scope.project_root.trim().is_empty() {
        return Err(TemporalScopeError::MissingProject);
    }
    if required.continuity && scope.continuity_id.trim().is_empty() {
        return Err(TemporalScopeError::MissingContinuity);
    }
    for (required, value, error) in [
        (
            required.host,
            &scope.host_id,
            TemporalScopeError::MissingHost,
        ),
        (
            required.operator,
            &scope.operator_id,
            TemporalScopeError::MissingOperator,
        ),
        (
            required.workpoint,
            &scope.workpoint_id,
            TemporalScopeError::MissingWorkpoint,
        ),
        (
            required.item,
            &scope.item_id,
            TemporalScopeError::MissingItem,
        ),
        (
            required.task,
            &scope.task_id,
            TemporalScopeError::MissingTask,
        ),
    ] {
        if required && value.as_deref().is_none_or(str::is_empty) {
            return Err(error);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicatedTemporalRecord {
    pub record_id: String,
    pub scope: TemporalScope,
    pub clock_epoch: String,
    pub elapsed_lower_ms: u64,
    pub elapsed_upper_ms: Option<u64>,
    pub revision: u64,
    pub predecessor_digest: Option<String>,
    pub digest: String,
    pub reducer_receipt_ref: String,
    pub fencing_token: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalReconciliationError {
    CrossScopeMerge,
    CrossEpochExactMerge,
    MissingReducerReceipt,
    DuplicateConflict,
}

pub fn reconcile_temporal_records(
    records: impl IntoIterator<Item = ReplicatedTemporalRecord>,
) -> Result<Vec<ReplicatedTemporalRecord>, TemporalReconciliationError> {
    let mut by_id = BTreeMap::<String, ReplicatedTemporalRecord>::new();
    for candidate in records {
        if candidate.reducer_receipt_ref.trim().is_empty() {
            return Err(TemporalReconciliationError::MissingReducerReceipt);
        }
        if let Some(current) = by_id.get(&candidate.record_id) {
            if current.scope != candidate.scope {
                return Err(TemporalReconciliationError::CrossScopeMerge);
            }
            if current.clock_epoch != candidate.clock_epoch
                && (current.elapsed_upper_ms.is_none() || candidate.elapsed_upper_ms.is_none())
            {
                return Err(TemporalReconciliationError::CrossEpochExactMerge);
            }
            if current.revision == candidate.revision && current.digest != candidate.digest {
                return Err(TemporalReconciliationError::DuplicateConflict);
            }
            if (candidate.revision, candidate.fencing_token)
                > (current.revision, current.fencing_token)
            {
                by_id.insert(candidate.record_id.clone(), candidate);
            }
        } else {
            by_id.insert(candidate.record_id.clone(), candidate);
        }
    }
    Ok(by_id.into_values().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalAuthorityAvailability {
    Verified,
    DegradedBounded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalArithmeticPermission {
    Exact,
    BoundedOnly,
    Blocked,
}

pub fn temporal_arithmetic_permission(
    availability: TemporalAuthorityAvailability,
    confidence: TemporalConfidence,
) -> TemporalArithmeticPermission {
    match (availability, confidence) {
        (TemporalAuthorityAvailability::Verified, TemporalConfidence::Verified) => {
            TemporalArithmeticPermission::Exact
        }
        (TemporalAuthorityAvailability::Unavailable, _) | (_, TemporalConfidence::Unavailable) => {
            TemporalArithmeticPermission::Blocked
        }
        _ => TemporalArithmeticPermission::BoundedOnly,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointTimingCategory {
    ActiveExecution,
    HarnessWait,
    OperatorWait,
    ExternalWait,
    Blocked,
    Suspended,
    OfflineUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointTimingInterval {
    pub interval_id: String,
    pub category: WorkpointTimingCategory,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub clock_epoch: String,
    pub elapsed_lower_ms: u64,
    pub elapsed_upper_ms: Option<u64>,
    pub overlaps_with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointTemporalProjection {
    pub workpoint_id: String,
    pub item_record_refs: Vec<String>,
    pub active_item_ref: Option<String>,
    pub blocked_item_refs: Vec<String>,
    pub next_item_ref: Option<String>,
    pub closure_status: String,
    pub intervals: Vec<WorkpointTimingInterval>,
    pub deadline_claim_refs: Vec<String>,
    pub progress_claim_refs: Vec<String>,
    pub incident_refs: Vec<String>,
    pub estimate_refs: Vec<String>,
    pub token_count: u64,
    pub tool_call_count: u64,
    pub attention_evidence_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkpointTemporalProjectionError {
    DuplicateItemRecord,
    ForeignActiveItem,
    ForeignBlockedItem,
    ForeignNextItem,
    NegativeInterval,
    UnboundedCrossEpochInterval,
    UsagePresentedAsProgress,
}

pub fn validate_workpoint_temporal_projection(
    projection: &WorkpointTemporalProjection,
) -> Result<(), WorkpointTemporalProjectionError> {
    let items = projection
        .item_record_refs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if items.len() != projection.item_record_refs.len() {
        return Err(WorkpointTemporalProjectionError::DuplicateItemRecord);
    }
    if projection
        .active_item_ref
        .as_deref()
        .is_some_and(|item| !items.contains(item))
    {
        return Err(WorkpointTemporalProjectionError::ForeignActiveItem);
    }
    if projection
        .blocked_item_refs
        .iter()
        .any(|item| !items.contains(item.as_str()))
    {
        return Err(WorkpointTemporalProjectionError::ForeignBlockedItem);
    }
    if projection
        .next_item_ref
        .as_deref()
        .is_some_and(|item| !items.contains(item))
    {
        return Err(WorkpointTemporalProjectionError::ForeignNextItem);
    }
    for interval in &projection.intervals {
        if interval
            .ended_at_ms
            .is_some_and(|end| end < interval.started_at_ms)
        {
            return Err(WorkpointTemporalProjectionError::NegativeInterval);
        }
        if interval.category == WorkpointTimingCategory::OfflineUnknown
            && interval.elapsed_upper_ms.is_none()
        {
            return Err(WorkpointTemporalProjectionError::UnboundedCrossEpochInterval);
        }
    }
    if !projection.progress_claim_refs.is_empty()
        && projection.evidence_refs.is_empty()
        && (projection.token_count > 0 || projection.tool_call_count > 0)
    {
        return Err(WorkpointTemporalProjectionError::UsagePresentedAsProgress);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementApplicability {
    Mandatory,
    ActivatedConditional,
    Optional,
    NotApplicable,
    Variance,
    OperatorReserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementConformance {
    pub requirement_id: String,
    pub applicability: RequirementApplicability,
    pub implementation_refs: Vec<String>,
    pub positive_test_refs: Vec<String>,
    pub negative_test_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub parity_refs: Vec<String>,
    pub applicability_evidence_refs: Vec<String>,
    pub variance_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementConformanceError {
    MissingImplementation,
    MissingPositiveTest,
    MissingNegativeTest,
    MissingEvidence,
    MissingReceipt,
    MissingParity,
    MissingApplicabilityEvidence,
    MissingVariance,
    UnsupportedOperatorReservedClosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec137AClosureReceipt {
    pub schema: String,
    pub source_sha256: String,
    pub requirement_count: usize,
    pub rows: Vec<RequirementConformance>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spec137AClosureError {
    EmptySourceHash,
    RequirementCountMismatch,
    DuplicateRequirement,
    Row(String, RequirementConformanceError),
    MissingReceiptEvidence,
}

pub fn validate_spec137a_closure(
    receipt: &Spec137AClosureReceipt,
    expected_requirement_count: usize,
) -> Result<(), Spec137AClosureError> {
    if receipt.source_sha256.trim().is_empty() {
        return Err(Spec137AClosureError::EmptySourceHash);
    }
    if receipt.requirement_count != expected_requirement_count
        || receipt.rows.len() != expected_requirement_count
    {
        return Err(Spec137AClosureError::RequirementCountMismatch);
    }
    let mut ids = std::collections::BTreeSet::new();
    for row in &receipt.rows {
        if !ids.insert(row.requirement_id.clone()) {
            return Err(Spec137AClosureError::DuplicateRequirement);
        }
        validate_requirement_conformance(row)
            .map_err(|error| Spec137AClosureError::Row(row.requirement_id.clone(), error))?;
    }
    if receipt.evidence_refs.is_empty() || receipt.receipt_refs.is_empty() {
        return Err(Spec137AClosureError::MissingReceiptEvidence);
    }
    Ok(())
}

pub fn validate_requirement_conformance(
    row: &RequirementConformance,
) -> Result<(), RequirementConformanceError> {
    match row.applicability {
        RequirementApplicability::Mandatory | RequirementApplicability::ActivatedConditional => {
            if row.implementation_refs.is_empty() {
                return Err(RequirementConformanceError::MissingImplementation);
            }
            if row.positive_test_refs.is_empty() {
                return Err(RequirementConformanceError::MissingPositiveTest);
            }
            if row.negative_test_refs.is_empty() {
                return Err(RequirementConformanceError::MissingNegativeTest);
            }
            if row.evidence_refs.is_empty() {
                return Err(RequirementConformanceError::MissingEvidence);
            }
            if row.receipt_refs.is_empty() {
                return Err(RequirementConformanceError::MissingReceipt);
            }
            if row.parity_refs.is_empty() {
                return Err(RequirementConformanceError::MissingParity);
            }
        }
        RequirementApplicability::Optional | RequirementApplicability::NotApplicable => {
            if row.applicability_evidence_refs.is_empty() {
                return Err(RequirementConformanceError::MissingApplicabilityEvidence);
            }
        }
        RequirementApplicability::Variance => {
            if row.variance_ref.is_none() || row.applicability_evidence_refs.is_empty() {
                return Err(RequirementConformanceError::MissingVariance);
            }
        }
        RequirementApplicability::OperatorReserved => {
            return Err(RequirementConformanceError::UnsupportedOperatorReservedClosure);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec137a_closure_receipt_rejects_schema_only_rows() {
        let receipt = Spec137AClosureReceipt {
            schema: "focusa.spec137a.closure.v1".into(),
            source_sha256: "sha256:test".into(),
            requirement_count: 1,
            rows: vec![RequirementConformance {
                requirement_id: "S137A-1".into(),
                applicability: RequirementApplicability::Mandatory,
                implementation_refs: vec![],
                positive_test_refs: vec![],
                negative_test_refs: vec![],
                evidence_refs: vec![],
                receipt_refs: vec![],
                parity_refs: vec![],
                applicability_evidence_refs: vec!["applicability:test".into()],
                variance_ref: None,
            }],
            evidence_refs: vec!["evidence:test".into()],
            receipt_refs: vec!["receipt:test".into()],
        };
        assert!(matches!(
            validate_spec137a_closure(&receipt, 1),
            Err(Spec137AClosureError::Row(
                _,
                RequirementConformanceError::MissingImplementation
            ))
        ));
    }

    #[test]
    fn exact_scope_and_reconciliation_fail_closed() {
        let scope = TemporalScope::project("/project", "main");
        let required = RequiredTemporalScope {
            host: true,
            operator: false,
            project: true,
            continuity: true,
            workpoint: false,
            item: false,
            task: false,
        };
        assert_eq!(
            validate_exact_scope(&scope, &required),
            Err(TemporalScopeError::MissingHost)
        );
    }

    #[test]
    fn authority_outage_never_grants_exact_arithmetic() {
        assert_eq!(
            temporal_arithmetic_permission(
                TemporalAuthorityAvailability::Unavailable,
                TemporalConfidence::Verified,
            ),
            TemporalArithmeticPermission::Blocked
        );
    }

    #[test]
    fn usage_without_evidence_is_not_progress() {
        let projection = WorkpointTemporalProjection {
            workpoint_id: "wp".into(),
            item_record_refs: vec!["item:1".into()],
            active_item_ref: Some("item:1".into()),
            blocked_item_refs: vec![],
            next_item_ref: None,
            closure_status: "active".into(),
            intervals: vec![],
            deadline_claim_refs: vec![],
            progress_claim_refs: vec!["claim:progress".into()],
            incident_refs: vec![],
            estimate_refs: vec![],
            token_count: 10,
            tool_call_count: 1,
            attention_evidence_refs: vec![],
            evidence_refs: vec![],
        };
        assert_eq!(
            validate_workpoint_temporal_projection(&projection),
            Err(WorkpointTemporalProjectionError::UsagePresentedAsProgress)
        );
    }

    #[test]
    fn mandatory_conformance_cannot_close_without_all_proof_classes() {
        let row = RequirementConformance {
            requirement_id: "S137-REQ-001".into(),
            applicability: RequirementApplicability::Mandatory,
            implementation_refs: vec!["src:clock".into()],
            positive_test_refs: vec!["test:positive".into()],
            negative_test_refs: vec![],
            evidence_refs: vec![],
            receipt_refs: vec![],
            parity_refs: vec![],
            applicability_evidence_refs: vec![],
            variance_ref: None,
        };
        assert_eq!(
            validate_requirement_conformance(&row),
            Err(RequirementConformanceError::MissingNegativeTest)
        );
    }
}
