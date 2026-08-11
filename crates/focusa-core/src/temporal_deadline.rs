use chrono::{DateTime, LocalResult, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::temporal::{TemporalScope, TemporalUncertainty};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineContractKind {
    ReadinessTarget,
    ExternalDeadline,
    SafetyMargin,
    CompletionTarget,
    SettlementRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineBoundaryPolicy {
    Inclusive,
    Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineComparison {
    OnTime,
    LateWindow,
    PossiblyCrossed,
    Breached,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadlineContract {
    pub contract_id: String,
    pub scope: TemporalScope,
    pub kind: DeadlineContractKind,
    pub readiness_at: Option<DateTime<Utc>>,
    pub deadline_at: DateTime<Utc>,
    pub boundary_policy: DeadlineBoundaryPolicy,
    pub source_authority: String,
    pub immutable_external_boundary: bool,
    pub inheritance_source_ref: Option<String>,
    pub working_window_ref: Option<String>,
    pub conflict_refs: Vec<String>,
    pub uncertainty: Option<TemporalUncertainty>,
    pub revision: u64,
    pub reducer_receipt_ref: String,
    pub cas_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivilTimeIntent {
    pub intent_id: String,
    pub original_expression: String,
    pub timezone: String,
    pub tzdb_version: String,
    pub calendar: String,
    pub calendar_version: String,
    pub jurisdiction: Option<String>,
    pub jurisdiction_rule_version: Option<String>,
    pub fold_policy: String,
    pub gap_policy: String,
    pub recurrence_rule: Option<String>,
    pub floating: bool,
    pub resolved_instants: Vec<DateTime<Utc>>,
    pub resolution_receipt_refs: Vec<String>,
    pub supersedes_resolution_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalBreach {
    pub breach_id: String,
    pub contract_id: String,
    pub definitely_crossed_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub overdue_opportunity_assessment_ref: String,
    pub settlement_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpportunityRisk {
    pub risk_id: String,
    pub contract_id: String,
    pub comparison: DeadlineComparison,
    pub reconciliation_action: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadlineDispatchPolicy {
    pub pinned_delivery_path_ref: Option<String>,
    pub expired_dispatch_blocked: bool,
    pub harmful_dispatch_blocked: bool,
    pub preserve_reconciliation: bool,
    pub preserve_compensation: bool,
    pub preserve_cleanup: bool,
    pub preserve_evidence: bool,
    pub preserve_settlement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadlineError {
    InvalidWindow,
    MissingReducerAuthority,
    ImmutableBoundaryMutation,
    MissingCivilTimeVersion,
    UncertaintyIsNotFinite,
    BreachEvidenceMissing,
    InvalidTimezone,
    AmbiguousCivilTime,
    NonexistentCivilTime,
}

pub fn compare_deadline(
    contract: &DeadlineContract,
    observed_earliest: Option<DateTime<Utc>>,
    observed_latest: Option<DateTime<Utc>>,
) -> DeadlineComparison {
    let (Some(earliest), Some(latest)) = (observed_earliest, observed_latest) else {
        return DeadlineComparison::Indeterminate;
    };
    if latest < earliest {
        return DeadlineComparison::Indeterminate;
    }
    let before_deadline = match contract.boundary_policy {
        DeadlineBoundaryPolicy::Inclusive => latest <= contract.deadline_at,
        DeadlineBoundaryPolicy::Exclusive => latest < contract.deadline_at,
    };
    if before_deadline {
        return match contract.readiness_at {
            Some(readiness) if latest <= readiness => DeadlineComparison::OnTime,
            Some(_) => DeadlineComparison::LateWindow,
            None => DeadlineComparison::OnTime,
        };
    }
    let crossed = match contract.boundary_policy {
        DeadlineBoundaryPolicy::Inclusive => earliest > contract.deadline_at,
        DeadlineBoundaryPolicy::Exclusive => earliest >= contract.deadline_at,
    };
    if crossed {
        DeadlineComparison::Breached
    } else {
        DeadlineComparison::PossiblyCrossed
    }
}

pub fn validate_deadline_contract(contract: &DeadlineContract) -> Result<(), DeadlineError> {
    if contract
        .readiness_at
        .is_some_and(|readiness| readiness > contract.deadline_at)
    {
        return Err(DeadlineError::InvalidWindow);
    }
    if contract.reducer_receipt_ref.trim().is_empty() || contract.cas_token.trim().is_empty() {
        return Err(DeadlineError::MissingReducerAuthority);
    }
    if contract.immutable_external_boundary
        && contract.kind != DeadlineContractKind::ExternalDeadline
    {
        return Err(DeadlineError::ImmutableBoundaryMutation);
    }
    if contract.uncertainty.as_ref().is_some_and(|uncertainty| {
        uncertainty
            .coverage_probability
            .is_some_and(|coverage| !coverage.is_finite() || !(0.0..=1.0).contains(&coverage))
    }) {
        return Err(DeadlineError::UncertaintyIsNotFinite);
    }
    Ok(())
}

pub fn resolve_civil_time(
    intent: &CivilTimeIntent,
    local: NaiveDateTime,
) -> Result<Vec<DateTime<Utc>>, DeadlineError> {
    validate_civil_time_intent(intent)?;
    let timezone: chrono_tz::Tz = intent
        .timezone
        .parse()
        .map_err(|_| DeadlineError::InvalidTimezone)?;
    match timezone.from_local_datetime(&local) {
        LocalResult::Single(value) => Ok(vec![value.with_timezone(&Utc)]),
        LocalResult::Ambiguous(first, second) => match intent.fold_policy.as_str() {
            "first" => Ok(vec![first.with_timezone(&Utc)]),
            "second" => Ok(vec![second.with_timezone(&Utc)]),
            "both" => Ok(vec![first.with_timezone(&Utc), second.with_timezone(&Utc)]),
            _ => Err(DeadlineError::AmbiguousCivilTime),
        },
        LocalResult::None if intent.gap_policy == "shift_forward" => {
            for minutes in 1..=180 {
                let shifted = local + chrono::Duration::minutes(minutes);
                if let LocalResult::Single(value) = timezone.from_local_datetime(&shifted) {
                    return Ok(vec![value.with_timezone(&Utc)]);
                }
            }
            Err(DeadlineError::NonexistentCivilTime)
        }
        LocalResult::None => Err(DeadlineError::NonexistentCivilTime),
    }
}

pub fn validate_civil_time_intent(intent: &CivilTimeIntent) -> Result<(), DeadlineError> {
    if intent.original_expression.trim().is_empty()
        || intent.tzdb_version.trim().is_empty()
        || intent.calendar_version.trim().is_empty()
        || intent.resolution_receipt_refs.len() != intent.resolved_instants.len()
    {
        return Err(DeadlineError::MissingCivilTimeVersion);
    }
    Ok(())
}

pub fn breach_or_risk(
    contract: &DeadlineContract,
    comparison: DeadlineComparison,
    observed_at: DateTime<Utc>,
    evidence_refs: Vec<String>,
) -> Result<(Option<TemporalBreach>, Option<OpportunityRisk>), DeadlineError> {
    match comparison {
        DeadlineComparison::Breached => {
            if evidence_refs.is_empty() {
                return Err(DeadlineError::BreachEvidenceMissing);
            }
            Ok((
                Some(TemporalBreach {
                    breach_id: format!("breach:{}", contract.contract_id),
                    contract_id: contract.contract_id.clone(),
                    definitely_crossed_at: observed_at,
                    evidence_refs,
                    overdue_opportunity_assessment_ref: format!(
                        "overdue-opportunity:{}",
                        contract.contract_id
                    ),
                    settlement_required: true,
                }),
                None,
            ))
        }
        DeadlineComparison::PossiblyCrossed | DeadlineComparison::Indeterminate => Ok((
            None,
            Some(OpportunityRisk {
                risk_id: format!("opportunity-risk:{}", contract.contract_id),
                contract_id: contract.contract_id.clone(),
                comparison,
                reconciliation_action: "refresh trusted time and reconcile boundary evidence"
                    .into(),
                evidence_refs,
            }),
        )),
        DeadlineComparison::OnTime | DeadlineComparison::LateWindow => Ok((None, None)),
    }
}

pub fn dispatch_policy_for(
    comparison: DeadlineComparison,
    smallest_valid_path_ref: Option<String>,
) -> DeadlineDispatchPolicy {
    let breached = comparison == DeadlineComparison::Breached;
    DeadlineDispatchPolicy {
        pinned_delivery_path_ref: breached.then_some(smallest_valid_path_ref).flatten(),
        expired_dispatch_blocked: breached,
        harmful_dispatch_blocked: breached,
        preserve_reconciliation: true,
        preserve_compensation: true,
        preserve_cleanup: true,
        preserve_evidence: true,
        preserve_settlement: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn test_contract(deadline_at: DateTime<Utc>) -> DeadlineContract {
        DeadlineContract {
            contract_id: "test-contract".into(),
            scope: crate::temporal::TemporalScope {
                project_root: "/tmp/test".into(),
                continuity_id: "test".into(),
                host_id: None, operator_id: None, workpoint_id: None, item_id: None, task_id: None,
            },
            kind: DeadlineContractKind::ReadinessTarget,
            deadline_at,
            readiness_at: None,
            boundary_policy: DeadlineBoundaryPolicy::Inclusive,
            source_authority: "operator".into(),
            immutable_external_boundary: false,
            inheritance_source_ref: None,
            working_window_ref: None,
            conflict_refs: vec![],
            uncertainty: None,
            revision: 1,
            reducer_receipt_ref: "receipt/1".into(),
            cas_token: "cas-1".into(),
        }
    }

    #[test]
    fn compare_deadline_on_time_before_readiness() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            deadline_at: now + Duration::hours(2),
            readiness_at: Some(now + Duration::hours(1)),
            ..test_contract(now + Duration::hours(2))
        };
        let result = compare_deadline(&contract, Some(now), Some(now));
        assert_eq!(result, DeadlineComparison::OnTime);
    }

    #[test]
    fn compare_deadline_breached_after_deadline() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            deadline_at: now - Duration::hours(1),
            boundary_policy: DeadlineBoundaryPolicy::Exclusive,
            ..test_contract(now - Duration::hours(1))
        };
        let result = compare_deadline(&contract, Some(now), Some(now));
        assert_eq!(result, DeadlineComparison::Breached);
    }

    #[test]
    fn compare_deadline_indeterminate_without_observations() {
        let now = chrono::Utc::now();
        let contract = test_contract(now + Duration::hours(1));
        let result = compare_deadline(&contract, None, Some(now));
        assert_eq!(result, DeadlineComparison::Indeterminate);
    }

    #[test]
    fn validate_contract_rejects_readiness_after_deadline() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            readiness_at: Some(now + Duration::hours(2)),
            deadline_at: now + Duration::hours(1),
            ..test_contract(now + Duration::hours(1))
        };
        assert!(validate_deadline_contract(&contract).is_err());
    }

    #[test]
    fn validate_contract_rejects_empty_reducer() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            reducer_receipt_ref: "  ".into(),
            ..test_contract(now + Duration::hours(1))
        };
        assert!(validate_deadline_contract(&contract).is_err());
    }

    #[test]
    fn validate_immutable_external_without_external_kind() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            immutable_external_boundary: true,
            kind: DeadlineContractKind::ReadinessTarget,
            ..test_contract(now + Duration::hours(1))
        };
        assert!(validate_deadline_contract(&contract).is_err());
    }

    #[test]
    fn validate_civil_time_rejects_empty_expression() {
        let intent = CivilTimeIntent {
            intent_id: "test-intent".into(),
            original_expression: "  ".into(),
            timezone: "America/Los_Angeles".into(),
            tzdb_version: "2024a".into(),
            calendar: "gregorian".into(),
            calendar_version: "v1".into(),
            jurisdiction: None,
            jurisdiction_rule_version: None,
            fold_policy: "first".into(),
            gap_policy: "shift_forward".into(),
            recurrence_rule: None,
            floating: false,
            resolution_receipt_refs: vec![],
            resolved_instants: vec![],
            supersedes_resolution_ref: None,
        };
        assert!(validate_civil_time_intent(&intent).is_err());
    }

    #[test]
    fn dispatch_blocks_on_breach() {
        let policy = dispatch_policy_for(DeadlineComparison::Breached, Some("safe/path".into()));
        assert!(policy.expired_dispatch_blocked);
        assert!(policy.preserve_evidence);
        assert_eq!(policy.pinned_delivery_path_ref, Some("safe/path".into()));
    }

    #[test]
    fn dispatch_allows_on_time() {
        let policy = dispatch_policy_for(DeadlineComparison::OnTime, None);
        assert!(!policy.expired_dispatch_blocked);
        assert!(!policy.harmful_dispatch_blocked);
    }

    #[test]
    fn compare_deadline_possibly_crossed_when_earliest_before_but_latest_after() {
        let now = chrono::Utc::now();
        let contract = DeadlineContract {
            deadline_at: now + Duration::hours(1),
            boundary_policy: DeadlineBoundaryPolicy::Exclusive,
            ..test_contract(now + Duration::hours(1))
        };
        let result = compare_deadline(&contract, Some(now), Some(now + Duration::hours(2)));
        assert_eq!(result, DeadlineComparison::PossiblyCrossed);
    }
}
