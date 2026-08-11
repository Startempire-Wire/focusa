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

#[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal_forecast::{ForecastRange, ForecastAuthorityContext, ReleasePhase};
    use crate::temporal::TemporalConfidence;

    fn sample_forecast_range() -> ForecastRange {
        ForecastRange {
            phase: ReleasePhase::Build,
            authority: Some(ForecastAuthorityContext {
                claim_kind: "effort-estimate".into(),
                target_state: "spec137-implemented".into(),
                scope_revision: "v1".into(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
                estimator_version: "1.0".into(),
                cohort: "temporal-auth-v1".into(),
                evidence_basis: vec!["previous-sprint-data".into()],
                comparable_sample_count: 10,
                all_attempt_sample_count: 15,
                censoring_method: "kaplan-meier".into(),
                correlation_method: "spearman".into(),
                calibration_profile: "standard-v1".into(),
                grounding_status: "grounded".into(),
                baseline_ref: "baseline-v1".into(),
                drift_policy_ref: "drift-v1".into(),
            }),
            sample_count: 10,
            minimum_ms: 400,
            p50_ms: 480,
            p80_ms: 520,
            p95_ms: 600,
            maximum_ms: 700,
            coverage_probability: 0.95,
            confidence: TemporalConfidence::Medium,
            method: "expert-judgment".into(),
            evidence_refs: vec!["data/sprint-velocity.json".into()],
        }
    }

    #[test]
    fn validity_fingerprint_equality_detects_no_change() {
        let fp = ForecastValidityFingerprint {
            scope_revision: "v1".into(), target_revision: "v1".into(),
            dependency_digest: "abc".into(), deadline_revision: "v1".into(),
            environment_digest: "env1".into(),
        };
        assert!(forecast_remains_valid(&fp, &fp));
    }

    #[test]
    fn validity_fingerprint_detects_scope_change() {
        let fp = ForecastValidityFingerprint {
            scope_revision: "v1".into(), target_revision: "v1".into(),
            dependency_digest: "abc".into(), deadline_revision: "v1".into(),
            environment_digest: "env1".into(),
        };
        let changed = ForecastValidityFingerprint { scope_revision: "v2".into(), ..fp.clone() };
        assert!(!forecast_remains_valid(&fp, &changed));
    }

    #[test]
    fn validity_fingerprint_detects_deadline_change() {
        let fp = ForecastValidityFingerprint {
            scope_revision: "v1".into(), target_revision: "v1".into(),
            dependency_digest: "abc".into(), deadline_revision: "v1".into(),
            environment_digest: "env1".into(),
        };
        let changed = ForecastValidityFingerprint { deadline_revision: "v2".into(), ..fp.clone() };
        assert!(!forecast_remains_valid(&fp, &changed));
    }

    #[test]
    fn evaluate_forecast_produces_valid_evaluation() {
        let range = sample_forecast_range();
        let result = evaluate_forecast(
            &range, 490, "verify-impl",
            0.6, 2, 1, 0.05, 1.0,
            vec!["evidence/1.txt".into()],
        ).expect("valid forecast should evaluate");
        assert_eq!(result.exact_target_event_ref, "verify-impl");
        assert_eq!(result.sample_count, 10);
        assert!(result.skill_score.is_finite());
        assert!(!result.evidence_refs.is_empty());
    }

    #[test]
    fn evaluate_forecast_rejects_empty_evidence() {
        let range = sample_forecast_range();
        let result = evaluate_forecast(&range, 500, "verify-impl", 0.6, 0, 0, 0.0, 1.0, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn evaluate_forecast_rejects_empty_target() {
        let range = sample_forecast_range();
        let result = evaluate_forecast(&range, 500, "  ", 0.6, 0, 0, 0.0, 1.0, vec!["e".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn evaluate_forecast_rejects_missing_authority() {
        let mut range = sample_forecast_range();
        range.authority = None;
        let result = evaluate_forecast(&range, 500, "verify-impl", 0.6, 0, 0, 0.0, 1.0, vec!["e".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn evaluate_forecast_coverage_is_one_when_actual_within_p95() {
        let range = sample_forecast_range();
        let eval = evaluate_forecast(&range, 590, "verify-impl", 0.6, 0, 0, 0.0, 1.0, vec!["e".into()]).unwrap();
        assert_eq!(eval.empirical_coverage, 1.0);
    }

    #[test]
    fn evaluate_forecast_coverage_is_zero_when_actual_exceeds_p95() {
        let range = sample_forecast_range();
        let eval = evaluate_forecast(&range, 650, "verify-impl", 0.6, 0, 0, 0.0, 1.0, vec!["e".into()]).unwrap();
        assert_eq!(eval.empirical_coverage, 0.0);
    }
}
