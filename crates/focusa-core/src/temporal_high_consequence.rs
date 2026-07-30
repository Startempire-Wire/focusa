use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    temporal::{TemporalClockDomain, TemporalScope},
    temporal_clock::{ClockSamplePair, TemporalVersionLineage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalPrecisionProfile {
    pub profile_id: String,
    pub integer_unit: String,
    pub displayed_precision: u32,
    pub measured_resolution_ns: u128,
    pub calibrated_accuracy_ns: u128,
    pub maximum_uncertainty_ns: u128,
    pub maximum_latency_ns: u128,
    pub ordering_method: String,
    pub deployed_path_evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighConsequenceTimestamp {
    pub timestamp_id: String,
    pub scope: TemporalScope,
    pub stable_capture_point: String,
    pub sample_pair: ClockSamplePair,
    pub source_ids: Vec<String>,
    pub sources_authenticated: bool,
    pub diversity_count: u32,
    pub clock_domain: TemporalClockDomain,
    pub synchronization_posture: String,
    pub holdover_age_ms: Option<u64>,
    pub integer_value: i128,
    pub integer_unit: String,
    pub uncertainty_method: String,
    pub coverage_probability: f64,
    pub lineage: TemporalVersionLineage,
    pub causal_predecessor_refs: Vec<String>,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchAgePolicy {
    pub maximum_clock_uncertainty_ns: u128,
    pub maximum_market_data_age_ms: u64,
    pub maximum_decision_age_ms: u64,
    pub maximum_dispatch_age_ms: u64,
    pub risk_limit_policy_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchAgeObservation {
    pub clock_uncertainty_ns: u128,
    pub market_data_age_ms: u64,
    pub decision_age_ms: u64,
    pub dispatch_age_ms: u64,
    pub in_scope: bool,
    pub within_risk_limits: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighConsequenceError {
    PrecisionMasqueradesAsAccuracy,
    CalibrationEvidenceMissing,
    UncertaintyExceeded,
    StaleMarketData,
    StaleDecision,
    StaleDispatch,
    OutOfScope,
    RiskLimitExceeded,
    CausalLineageMissing,
    LlmInDeterministicLoop,
    ActivationFirewallOpen,
    SecurityPolicyIncomplete,
    LedgerIntegrityMissing,
}

pub fn validate_precision_profile(
    profile: &TemporalPrecisionProfile,
) -> Result<(), HighConsequenceError> {
    if profile.deployed_path_evidence_refs.is_empty() {
        return Err(HighConsequenceError::CalibrationEvidenceMissing);
    }
    if profile.calibrated_accuracy_ns < profile.measured_resolution_ns
        && profile.displayed_precision > 0
    {
        return Err(HighConsequenceError::PrecisionMasqueradesAsAccuracy);
    }
    Ok(())
}

pub fn authorize_dispatch(
    policy: &DispatchAgePolicy,
    observation: &DispatchAgeObservation,
) -> Result<(), HighConsequenceError> {
    if observation.clock_uncertainty_ns > policy.maximum_clock_uncertainty_ns {
        return Err(HighConsequenceError::UncertaintyExceeded);
    }
    if observation.market_data_age_ms > policy.maximum_market_data_age_ms {
        return Err(HighConsequenceError::StaleMarketData);
    }
    if observation.decision_age_ms > policy.maximum_decision_age_ms {
        return Err(HighConsequenceError::StaleDecision);
    }
    if observation.dispatch_age_ms > policy.maximum_dispatch_age_ms {
        return Err(HighConsequenceError::StaleDispatch);
    }
    if !observation.in_scope {
        return Err(HighConsequenceError::OutOfScope);
    }
    if !observation.within_risk_limits {
        return Err(HighConsequenceError::RiskLimitExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketTemporalTrace {
    pub intent_id: String,
    pub idempotency_key: String,
    pub event_ref: String,
    pub ingestion_ref: String,
    pub decision_ref: String,
    pub authority_ref: String,
    pub risk_check_ref: String,
    pub dispatch_ref: Option<String>,
    pub acknowledgement_ref: Option<String>,
    pub fill_refs: Vec<String>,
    pub cancellation_ref: Option<String>,
    pub reconciliation_ref: String,
    pub causal_sequence_refs: Vec<String>,
    pub latency_distribution_ref: String,
    pub unknown_outcome: bool,
    pub partial_fill: bool,
    pub cancellation_race: bool,
    pub kill_switch_checked: bool,
}

pub fn validate_market_trace(trace: &MarketTemporalTrace) -> Result<(), HighConsequenceError> {
    let required = [
        trace.event_ref.as_str(),
        trace.ingestion_ref.as_str(),
        trace.decision_ref.as_str(),
        trace.authority_ref.as_str(),
        trace.risk_check_ref.as_str(),
        trace.reconciliation_ref.as_str(),
        trace.latency_distribution_ref.as_str(),
    ];
    if required.iter().any(|value| value.is_empty()) || trace.causal_sequence_refs.is_empty() {
        return Err(HighConsequenceError::CausalLineageMissing);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationLevel {
    Simulation,
    Paper,
    Shadow,
    Canary,
    Live,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationFirewall {
    pub current_level: ActivationLevel,
    pub requested_level: ActivationLevel,
    pub requirement_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub approval_receipt_refs: Vec<String>,
    pub deterministic_loop_has_llm: bool,
}

pub fn authorize_activation(firewall: &ActivationFirewall) -> Result<(), HighConsequenceError> {
    if firewall.deterministic_loop_has_llm {
        return Err(HighConsequenceError::LlmInDeterministicLoop);
    }
    if firewall.requested_level == ActivationLevel::Live
        && (firewall.requirement_refs.is_empty()
            || firewall.evidence_refs.len() < firewall.requirement_refs.len()
            || firewall.approval_receipt_refs.is_empty())
    {
        return Err(HighConsequenceError::ActivationFirewallOpen);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighConsequenceDomainPack {
    pub domain_id: String,
    pub precision_profile_ref: String,
    pub control_owner: String,
    pub independent_reviewer: String,
    pub jurisdiction: String,
    pub rule_version: String,
    pub annual_review_receipt_ref: String,
    pub certification_ref: String,
    pub resilience_evidence_refs: Vec<String>,
    pub capacity_evidence_refs: Vec<String>,
    pub integrity_evidence_refs: Vec<String>,
    pub availability_evidence_refs: Vec<String>,
    pub security_evidence_refs: Vec<String>,
    pub bcdr_ref: String,
    pub rto_ms: u64,
    pub rpo_ms: u64,
    pub retention_policy_ref: String,
    pub deterministic_boundary_ref: String,
    pub utc_traceability_review_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalDataPolicy {
    pub classification: String,
    pub least_privilege_policy_ref: String,
    pub encryption_policy_ref: String,
    pub coarsening_policy_ref: String,
    pub redaction_policy_ref: String,
    pub retention_policy_ref: String,
    pub deletion_policy_ref: String,
    pub legal_hold_policy_ref: String,
    pub export_policy_ref: String,
    pub aggregation_policy_ref: String,
    pub audit_access_policy_ref: String,
    pub side_channel_policy_ref: String,
}

pub fn validate_data_policy(policy: &TemporalDataPolicy) -> Result<(), HighConsequenceError> {
    let refs = [
        &policy.classification,
        &policy.least_privilege_policy_ref,
        &policy.encryption_policy_ref,
        &policy.coarsening_policy_ref,
        &policy.redaction_policy_ref,
        &policy.retention_policy_ref,
        &policy.deletion_policy_ref,
        &policy.legal_hold_policy_ref,
        &policy.export_policy_ref,
        &policy.aggregation_policy_ref,
        &policy.audit_access_policy_ref,
        &policy.side_channel_policy_ref,
    ];
    if refs.iter().any(|value| value.trim().is_empty()) {
        return Err(HighConsequenceError::SecurityPolicyIncomplete);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedTemporalLedgerControl {
    pub signed_event_kinds: BTreeSet<String>,
    pub hash_chain_verified: bool,
    pub source_authentication_test_refs: Vec<String>,
    pub ledger_integrity_test_refs: Vec<String>,
}

pub fn validate_ledger_controls(
    controls: &SignedTemporalLedgerControl,
) -> Result<(), HighConsequenceError> {
    let required = [
        "clock_sample",
        "correction",
        "deadline",
        "guard",
        "cancellation",
        "closure",
        "receipt",
    ];
    if !controls.hash_chain_verified
        || required
            .iter()
            .any(|kind| !controls.signed_event_kinds.contains(*kind))
        || controls.source_authentication_test_refs.is_empty()
        || controls.ledger_integrity_test_refs.is_empty()
    {
        return Err(HighConsequenceError::LedgerIntegrityMissing);
    }
    Ok(())
}
