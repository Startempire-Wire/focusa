use serde::{Deserialize, Serialize};

use crate::temporal::{TemporalClockSample, TemporalConfidence};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalClockCapabilities {
    pub realtime: bool,
    pub suspend_excluding_monotonic: bool,
    pub suspend_aware_monotonic: bool,
    pub process_cpu: bool,
    pub thread_cpu: bool,
    pub tai: bool,
    pub fallback_behavior: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockUncertaintyBudget {
    pub method: String,
    pub standard_uncertainty_ns: f64,
    pub expanded_uncertainty_ns: f64,
    pub coverage_factor: f64,
    pub coverage_probability: f64,
    pub offset_ns: i128,
    pub delay_ns: u128,
    pub jitter_ns: u128,
    pub dispersion_ns: u128,
    pub root_distance_ns: u128,
    pub frequency_error_ppb: f64,
    pub sample_age_ms: u64,
    pub calibration_lineage: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalVersionLineage {
    pub schema_version: String,
    pub policy_version: String,
    pub adapter_version: String,
    pub calendar_version: Option<String>,
    pub tzdb_version: Option<String>,
    pub estimator_version: Option<String>,
    pub clock_profile_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockSamplePair {
    pub before: TemporalClockSample,
    pub after: TemporalClockSample,
    pub elapsed_lower_ns: u128,
    pub elapsed_upper_ns: Option<u128>,
    pub uncertainty: ClockUncertaintyBudget,
    pub crosses_boot_epoch: bool,
    pub crosses_suspend: bool,
    pub lineage: TemporalVersionLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockSamplePairError {
    ScopeEpochMismatch,
    NegativeElapsed,
    MissingCrossEpochBound,
    InvalidUncertainty,
    UntrustedSample,
}

pub fn validate_clock_sample_pair(pair: &ClockSamplePair) -> Result<(), ClockSamplePairError> {
    if pair
        .elapsed_upper_ns
        .is_some_and(|upper| upper < pair.elapsed_lower_ns)
    {
        return Err(ClockSamplePairError::NegativeElapsed);
    }
    if pair.crosses_boot_epoch && pair.elapsed_upper_ns.is_none() {
        return Err(ClockSamplePairError::MissingCrossEpochBound);
    }
    if pair.before.boot_id != pair.after.boot_id && !pair.crosses_boot_epoch {
        return Err(ClockSamplePairError::ScopeEpochMismatch);
    }
    if pair.before.confidence == TemporalConfidence::Unavailable
        || pair.after.confidence == TemporalConfidence::Unavailable
    {
        return Err(ClockSamplePairError::UntrustedSample);
    }
    if !pair.uncertainty.standard_uncertainty_ns.is_finite()
        || pair.uncertainty.standard_uncertainty_ns < 0.0
        || !pair.uncertainty.expanded_uncertainty_ns.is_finite()
        || pair.uncertainty.expanded_uncertainty_ns < pair.uncertainty.standard_uncertainty_ns
        || !(0.0..=1.0).contains(&pair.uncertainty.coverage_probability)
        || pair.uncertainty.coverage_factor < 1.0
        || pair.uncertainty.calibration_lineage.is_empty()
    {
        return Err(ClockSamplePairError::InvalidUncertainty);
    }
    Ok(())
}
