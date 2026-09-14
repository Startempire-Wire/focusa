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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precision_rejects_missing_calibration_evidence() {
        let profile = TemporalPrecisionProfile {
            profile_id: "p-1".into(),
            integer_unit: "nanosecond".into(),
            deployed_path_evidence_refs: vec![],
            calibrated_accuracy_ns: 100,
            measured_resolution_ns: 50,
            displayed_precision: 5,
            maximum_uncertainty_ns: 500,
            maximum_latency_ns: 10000,
            ordering_method: "monotonic".into(),
        };
        assert!(validate_precision_profile(&profile).is_err());
    }

    #[test]
    fn precision_rejects_masquerading_accuracy_when_below_resolution() {
        let profile = TemporalPrecisionProfile {
            profile_id: "p-2".into(),
            integer_unit: "nanosecond".into(),
            deployed_path_evidence_refs: vec!["evidence/1".into()],
            calibrated_accuracy_ns: 30,
            measured_resolution_ns: 50,
            displayed_precision: 3,
            maximum_uncertainty_ns: 500,
            maximum_latency_ns: 10000,
            ordering_method: "monotonic".into(),
        };
        assert!(validate_precision_profile(&profile).is_err());
    }

    #[test]
    fn dispatch_rejects_stale_market_data() {
        let policy = DispatchAgePolicy {
            maximum_clock_uncertainty_ns: 1000,
            maximum_market_data_age_ms: 500,
            maximum_decision_age_ms: 1000,
            maximum_dispatch_age_ms: 2000,
            risk_limit_policy_ref: "risk-1".into(),
        };
        let observation = DispatchAgeObservation {
            clock_uncertainty_ns: 100,
            market_data_age_ms: 600,
            decision_age_ms: 200,
            dispatch_age_ms: 300,
            in_scope: true,
            within_risk_limits: true,
        };
        assert!(matches!(
            authorize_dispatch(&policy, &observation),
            Err(HighConsequenceError::StaleMarketData)
        ));
    }

    #[test]
    fn dispatch_rejects_out_of_scope() {
        let policy = DispatchAgePolicy {
            maximum_clock_uncertainty_ns: 1000,
            maximum_market_data_age_ms: 500,
            maximum_decision_age_ms: 1000,
            maximum_dispatch_age_ms: 2000,
            risk_limit_policy_ref: "risk-1".into(),
        };
        let observation = DispatchAgeObservation {
            clock_uncertainty_ns: 100,
            market_data_age_ms: 200,
            decision_age_ms: 200,
            dispatch_age_ms: 300,
            in_scope: false,
            within_risk_limits: true,
        };
        assert!(matches!(
            authorize_dispatch(&policy, &observation),
            Err(HighConsequenceError::OutOfScope)
        ));
    }

    #[test]
    fn dispatch_passes_valid_observation() {
        let policy = DispatchAgePolicy {
            maximum_clock_uncertainty_ns: 1000,
            maximum_market_data_age_ms: 500,
            maximum_decision_age_ms: 1000,
            maximum_dispatch_age_ms: 2000,
            risk_limit_policy_ref: "risk-1".into(),
        };
        let observation = DispatchAgeObservation {
            clock_uncertainty_ns: 100,
            market_data_age_ms: 200,
            decision_age_ms: 200,
            dispatch_age_ms: 300,
            in_scope: true,
            within_risk_limits: true,
        };
        assert!(authorize_dispatch(&policy, &observation).is_ok());
    }

    #[test]
    fn market_trace_rejects_empty_causal_lineage() {
        let trace = MarketTemporalTrace {
            intent_id: "i-1".into(),
            idempotency_key: "k-1".into(),
            event_ref: "ev".into(),
            ingestion_ref: "ing".into(),
            decision_ref: "dec".into(),
            authority_ref: "auth".into(),
            risk_check_ref: "risk".into(),
            dispatch_ref: None,
            acknowledgement_ref: None,
            fill_refs: vec![],
            cancellation_ref: None,
            reconciliation_ref: "rec".into(),
            causal_sequence_refs: vec![],
            latency_distribution_ref: "lat".into(),
            unknown_outcome: false,
            partial_fill: false,
            cancellation_race: false,
            kill_switch_checked: true,
        };
        assert!(validate_market_trace(&trace).is_err());
    }

    #[test]
    fn market_trace_passes_valid() {
        let trace = MarketTemporalTrace {
            intent_id: "i-1".into(),
            idempotency_key: "k-1".into(),
            event_ref: "ev".into(),
            ingestion_ref: "ing".into(),
            decision_ref: "dec".into(),
            authority_ref: "auth".into(),
            risk_check_ref: "risk".into(),
            dispatch_ref: None,
            acknowledgement_ref: None,
            fill_refs: vec![],
            cancellation_ref: None,
            reconciliation_ref: "rec".into(),
            causal_sequence_refs: vec!["c1".into()],
            latency_distribution_ref: "lat".into(),
            unknown_outcome: false,
            partial_fill: false,
            cancellation_race: false,
            kill_switch_checked: true,
        };
        assert!(validate_market_trace(&trace).is_ok());
    }

    #[test]
    fn activation_rejects_llm_in_deterministic_loop() {
        let firewall = ActivationFirewall {
            current_level: ActivationLevel::Shadow,
            requested_level: ActivationLevel::Canary,
            requirement_refs: vec!["r1".into()],
            evidence_refs: vec!["e1".into()],
            approval_receipt_refs: vec!["a1".into()],
            deterministic_loop_has_llm: true,
        };
        assert!(matches!(
            authorize_activation(&firewall),
            Err(HighConsequenceError::LlmInDeterministicLoop)
        ));
    }

    #[test]
    fn data_policy_rejects_empty_references() {
        let policy = TemporalDataPolicy {
            classification: "restricted".into(),
            least_privilege_policy_ref: "".into(),
            encryption_policy_ref: "enc".into(),
            coarsening_policy_ref: "c".into(),
            redaction_policy_ref: "r".into(),
            retention_policy_ref: "ret".into(),
            deletion_policy_ref: "del".into(),
            legal_hold_policy_ref: "lh".into(),
            export_policy_ref: "exp".into(),
            aggregation_policy_ref: "agg".into(),
            audit_access_policy_ref: "aud".into(),
            side_channel_policy_ref: "sc".into(),
        };
        assert!(validate_data_policy(&policy).is_err());
    }

    #[test]
    fn ledger_controls_require_hash_chain_verified() {
        let mut kinds = BTreeSet::new();
        for k in &[
            "clock_sample",
            "correction",
            "deadline",
            "guard",
            "cancellation",
            "closure",
            "receipt",
        ] {
            kinds.insert(k.to_string());
        }
        let controls = SignedTemporalLedgerControl {
            signed_event_kinds: kinds,
            hash_chain_verified: false,
            source_authentication_test_refs: vec!["t1".into()],
            ledger_integrity_test_refs: vec!["t2".into()],
        };
        assert!(validate_ledger_controls(&controls).is_err());
    }
}
