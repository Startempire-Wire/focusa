//! Spec137 forecast evaluation, calibration authority, and invalidation fingerprints.

use serde::{Deserialize, Serialize};

use crate::temporal_forecast::{ForecastError, ForecastRange, calibrate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastEvaluation {
    pub evaluation_id: String,
    pub exact_target_event_ref: String,
    pub cohort: String,
    pub sample_count: usize,
    pub censored_sample_count: usize,
    pub correlated_cluster_count: usize,
    pub reliability: f64,
    pub bias_ms: i64,
    pub empirical_coverage: f64,
    pub sharpness_ms: u64,
    pub proper_score: f64,
    pub baseline_score: f64,
    pub skill_score: f64,
    pub decision_value: f64,
    pub sample_error_lower: f64,
    pub sample_error_upper: f64,
    pub cohort_drift: f64,
    pub policy_quantiles: Vec<f64>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastValidityFingerprint {
    pub scope_revision: String,
    pub target_revision: String,
    pub dependency_digest: String,
    pub deadline_revision: String,
    pub environment_digest: String,
}

pub fn forecast_remains_valid(
    issued: &ForecastValidityFingerprint,
    current: &ForecastValidityFingerprint,
) -> bool {
    issued == current
}

pub fn evaluate_forecast(
    range: &ForecastRange,
    actual_ms: u64,
    exact_target_event_ref: impl Into<String>,
    baseline_score: f64,
    censored_sample_count: usize,
    correlated_cluster_count: usize,
    cohort_drift: f64,
    decision_value: f64,
    evidence_refs: Vec<String>,
) -> Result<ForecastEvaluation, ForecastError> {
    let authority = range
        .authority
        .as_ref()
        .ok_or(ForecastError::NoObservedHistory)?;
    let target_ref = exact_target_event_ref.into();
    if evidence_refs.is_empty() || target_ref.trim().is_empty() {
        return Err(ForecastError::NoObservedHistory);
    }
    let calibration = calibrate(range, actual_ms);
    let coverage = if calibration.within_p95 { 1.0 } else { 0.0 };
    let bias_ms = actual_ms as i128 - range.p50_ms as i128;
    let proper_score = calibration.score;
    let skill_score = baseline_score - proper_score;
    let standard_error = (proper_score.max(0.0) / range.sample_count.max(1) as f64).sqrt();
    Ok(ForecastEvaluation {
        evaluation_id: format!("forecast-evaluation:{target_ref}"),
        exact_target_event_ref: target_ref,
        cohort: authority.cohort.clone(),
        sample_count: range.sample_count,
        censored_sample_count,
        correlated_cluster_count,
        reliability: 1.0 - proper_score.clamp(0.0, 1.0),
        bias_ms: bias_ms.clamp(i64::MIN as i128, i64::MAX as i128) as i64,
        empirical_coverage: coverage,
        sharpness_ms: range.p95_ms.saturating_sub(range.minimum_ms),
        proper_score,
        baseline_score,
        skill_score,
        decision_value,
        sample_error_lower: (proper_score - 1.96 * standard_error).max(0.0),
        sample_error_upper: proper_score + 1.96 * standard_error,
        cohort_drift,
        policy_quantiles: vec![0.5, 0.8, 0.95],
        evidence_refs,
    })
}
