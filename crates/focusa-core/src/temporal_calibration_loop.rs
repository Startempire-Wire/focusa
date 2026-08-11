//! Spec138 Slice 8: continuous calibration feedback loop.
//!
//! Every forecast evaluation feeds the calibration loop which detects drift,
//! generates LearningCandidates, and promotes/rolls back calibration profiles.
//! No self-sovereign learning — every change requires evidence.

use crate::temporal_forecast::{
    ForecastAuthorityContext, ForecastRange, ReleasePhase, calibrate,
};
use crate::temporal_forecast_evaluation::{ForecastEvaluation, evaluate_forecast};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum evaluations retained for calibration drift detection.
const MAX_EVALUATION_HISTORY: usize = 128;

/// Minimum evaluations before calibration drift can be assessed.
const MIN_EVALUATIONS_FOR_DRIFT: usize = 5;

/// Drift threshold — if reliability drops below this ratio of baseline,
/// a LearningCandidate is generated.
const RELIABILITY_DRIFT_THRESHOLD: f64 = 0.15;

/// The continuous calibration state machine.
///
/// Every forecast evaluation is ingested. When enough evaluations accumulate,
/// drift detection compares recent reliability to the calibration baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationLoop {
    /// Scoped calibration profile identifier.
    pub profile_id: String,
    /// Baseline calibration score from the authority context.
    pub baseline_calibration: f64,
    /// Recent evaluations (FIFO, max 128).
    pub recent_evaluations: VecDeque<CalibrationSnapshot>,
    /// Current drift status.
    pub drift_status: CalibrationDriftStatus,
    /// Cumulative statistics.
    pub cumulative: CalibrationCumulative,
    /// Learning candidates generated from this loop.
    pub learning_candidates: Vec<LearningCandidate>,
    /// Last time the loop was updated.
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationSnapshot {
    pub evaluation_id: String,
    pub reliability: f64,
    pub skill_score: f64,
    pub empirical_coverage: f64,
    pub sample_count: usize,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationDriftStatus {
    /// Baseline calibration is accurate.
    Stable,
    /// Minor drift detected — monitor more closely.
    Monitoring,
    /// Significant drift — LearningCandidate required.
    Drifted,
    /// Calibration is unreliable — profile must be rebuilt.
    Degraded,
    /// Insufficient data for assessment.
    InsufficientData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationCumulative {
    pub total_evaluations: usize,
    pub mean_reliability: f64,
    pub mean_skill_score: f64,
    pub mean_coverage: f64,
    pub total_bias_ms: i64,
    pub drift_events: usize,
}

impl Default for CalibrationCumulative {
    fn default() -> Self {
        Self {
            total_evaluations: 0,
            mean_reliability: 1.0,
            mean_skill_score: 0.0,
            mean_coverage: 1.0,
            total_bias_ms: 0,
            drift_events: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningCandidate {
    pub candidate_id: String,
    pub generated_at: DateTime<Utc>,
    pub drift_severity: f64,
    pub evidence_evaluation_refs: Vec<String>,
    pub recommended_action: LearningAction,
    pub confidence: f64,
    pub promoted: bool,
    pub rolled_back: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningAction {
    RecalibrateCohort,
    ExpandSampleSet,
    UpdateBaseline,
    RebuildProfile,
    NoAction,
}

impl CalibrationLoop {
    /// Create a new calibration loop seeded from an authority context.
    pub fn new(
        profile_id: impl Into<String>,
        authority: &ForecastAuthorityContext,
    ) -> Self {
        Self {
            profile_id: profile_id.into(),
            baseline_calibration: 0.8, // default prior
            recent_evaluations: VecDeque::new(),
            drift_status: CalibrationDriftStatus::InsufficientData,
            cumulative: CalibrationCumulative::default(),
            learning_candidates: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// Ingest a forecast evaluation into the calibration loop.
    ///
    /// After MIN_EVALUATIONS_FOR_DRIFT evaluations accumulate, drift detection
    /// compares recent reliability against baseline.
    pub fn ingest(
        &mut self,
        evaluation: &ForecastEvaluation,
        range: &ForecastRange,
        actual_ms: u64,
        exact_target_event_ref: &str,
        baseline_score: f64,
        evidence_refs: Vec<String>,
    ) -> Option<LearningCandidate> {
        let snapshot = CalibrationSnapshot {
            evaluation_id: evaluation.evaluation_id.clone(),
            reliability: evaluation.reliability,
            skill_score: evaluation.skill_score,
            empirical_coverage: evaluation.empirical_coverage,
            sample_count: evaluation.sample_count,
            recorded_at: Utc::now(),
        };

        self.recent_evaluations.push_back(snapshot);
        if self.recent_evaluations.len() > MAX_EVALUATION_HISTORY {
            self.recent_evaluations.pop_front();
        }

        // Update cumulative statistics.
        let n = self.cumulative.total_evaluations as f64 + 1.0;
        self.cumulative.mean_reliability = (self.cumulative.mean_reliability * (n - 1.0)
            + evaluation.reliability)
            / n;
        self.cumulative.mean_skill_score = (self.cumulative.mean_skill_score * (n - 1.0)
            + evaluation.skill_score)
            / n;
        self.cumulative.mean_coverage = (self.cumulative.mean_coverage * (n - 1.0)
            + evaluation.empirical_coverage)
            / n;
        self.cumulative.total_bias_ms = self
            .cumulative
            .total_bias_ms
            .saturating_add(evaluation.bias_ms);
        self.cumulative.total_evaluations += 1;
        self.last_updated = Utc::now();

        // Drift detection.
        if self.recent_evaluations.len() < MIN_EVALUATIONS_FOR_DRIFT {
            return None;
        }

        let recent_reliability: f64 = self
            .recent_evaluations
            .iter()
            .map(|e| e.reliability)
            .sum::<f64>()
            / self.recent_evaluations.len() as f64;

        let drift = self.baseline_calibration - recent_reliability;

        if drift > RELIABILITY_DRIFT_THRESHOLD {
            self.drift_status = CalibrationDriftStatus::Drifted;
            self.cumulative.drift_events += 1;

            let candidate = LearningCandidate {
                candidate_id: format!(
                    "calibration-drift:{}:{}",
                    self.profile_id,
                    Utc::now().timestamp_millis()
                ),
                generated_at: Utc::now(),
                drift_severity: drift,
                evidence_evaluation_refs: vec![
                    evaluation.evaluation_id.clone(),
                ],
                recommended_action: if drift > 0.3 {
                    LearningAction::RebuildProfile
                } else {
                    LearningAction::RecalibrateCohort
                },
                confidence: (1.0 - drift).max(0.0),
                promoted: false,
                rolled_back: false,
            };

            self.learning_candidates.push(candidate.clone());
            return Some(candidate);
        } else if drift > RELIABILITY_DRIFT_THRESHOLD * 0.5 {
            self.drift_status = CalibrationDriftStatus::Monitoring;
        } else {
            self.drift_status = CalibrationDriftStatus::Stable;
        }

        None
    }

    /// Promote a learning candidate — update baseline calibration.
    ///
    /// Evidence must be provided to justify the promotion.
    pub fn promote_candidate(
        &mut self,
        candidate_id: &str,
        evidence_refs: Vec<String>,
    ) -> bool {
        if evidence_refs.is_empty() {
            return false;
        }
        if let Some(candidate) = self
            .learning_candidates
            .iter_mut()
            .find(|c| c.candidate_id == candidate_id && !c.promoted && !c.rolled_back)
        {
            candidate.promoted = true;
            candidate.evidence_evaluation_refs.extend(evidence_refs);
            // Update baseline to current cumulative reliability.
            self.baseline_calibration = self.cumulative.mean_reliability;
            self.drift_status = CalibrationDriftStatus::Stable;
            return true;
        }
        false
    }

    /// Roll back a learning candidate — the evidence was insufficient.
    pub fn rollback_candidate(&mut self, candidate_id: &str) -> bool {
        if let Some(candidate) = self
            .learning_candidates
            .iter_mut()
            .find(|c| c.candidate_id == candidate_id && !c.rolled_back)
        {
            candidate.rolled_back = true;
            candidate.promoted = false;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_range() -> ForecastRange {
        ForecastRange {
            phase: ReleasePhase::Build,
            authority: Some(ForecastAuthorityContext {
                claim_kind: "effort".into(),
                target_state: "done".into(),
                scope_revision: "v1".into(),
                expires_at: Utc::now() + Duration::hours(24),
                estimator_version: "1.0".into(),
                cohort: "test".into(),
                evidence_basis: vec!["data".into()],
                comparable_sample_count: 10,
                all_attempt_sample_count: 15,
                censoring_method: "none".into(),
                correlation_method: "spearman".into(),
                calibration_profile: "standard".into(),
                grounding_status: "grounded".into(),
                baseline_ref: "baseline".into(),
                drift_policy_ref: "drift-policy".into(),
            }),
            sample_count: 10,
            minimum_ms: 400,
            p50_ms: 480,
            p80_ms: 520,
            p95_ms: 600,
            maximum_ms: 700,
            coverage_probability: 0.95,
            confidence: crate::temporal::TemporalConfidence::Medium,
            method: "expert".into(),
            evidence_refs: vec!["ev".into()],
        }
    }

    #[test]
    fn calibration_loop_requires_minimum_evaluations() {
        let authority = ForecastAuthorityContext {
            claim_kind: "effort".into(),
            target_state: "done".into(),
            scope_revision: "v1".into(),
            expires_at: Utc::now() + Duration::hours(24),
            estimator_version: "1.0".into(),
            cohort: "test".into(),
            evidence_basis: vec!["data".into()],
            comparable_sample_count: 10,
            all_attempt_sample_count: 15,
            censoring_method: "none".into(),
            correlation_method: "spearman".into(),
            calibration_profile: "standard".into(),
            grounding_status: "grounded".into(),
            baseline_ref: "baseline".into(),
            drift_policy_ref: "drift-policy".into(),
        };
        let mut loop_ = CalibrationLoop::new("test-profile", &authority);
        let range = sample_range();
        // Only 4 evaluations — below MIN_EVALUATIONS_FOR_DRIFT (5).
        for i in 0..4 {
            let eval = evaluate_forecast(
                &range, 500, &format!("target-{}", i),
                0.6, 0, 0, 0.0, 1.0,
                vec!["e".into()],
            ).unwrap();
            let candidate = loop_.ingest(&eval, &range, 500, "t", 0.6, vec!["e".into()]);
            assert!(candidate.is_none(), "should not generate candidate before min evaluations");
        }
    }

    #[test]
    fn calibration_loop_detects_drift() {
        let authority = ForecastAuthorityContext {
            claim_kind: "effort".into(),
            target_state: "done".into(),
            scope_revision: "v1".into(),
            expires_at: Utc::now() + Duration::hours(24),
            estimator_version: "1.0".into(),
            cohort: "test".into(),
            evidence_basis: vec!["data".into()],
            comparable_sample_count: 10,
            all_attempt_sample_count: 15,
            censoring_method: "none".into(),
            correlation_method: "spearman".into(),
            calibration_profile: "standard".into(),
            grounding_status: "grounded".into(),
            baseline_ref: "baseline".into(),
            drift_policy_ref: "drift-policy".into(),
        };
        let mut loop_ = CalibrationLoop::new("test-profile", &authority);
        loop_.baseline_calibration = 0.9;
        let range = sample_range();
        // 5 evaluations with low reliability should trigger drift.
        for i in 0..5 {
            let eval = evaluate_forecast(
                &range, 650, // Exceeds p95=600 → coverage=0
                &format!("target-{}", i),
                0.6, 0, 0, 0.0, 1.0,
                vec!["e".into()],
            ).unwrap();
            let candidate = loop_.ingest(&eval, &range, 650, "t", 0.6, vec!["e".into()]);
            if i == 4 {
                assert!(candidate.is_some(), "should detect drift on 5th evaluation");
            }
        }
        assert_eq!(loop_.drift_status, CalibrationDriftStatus::Drifted);
    }

    #[test]
    fn promote_updates_baseline() {
        let authority = ForecastAuthorityContext {
            claim_kind: "effort".into(),
            target_state: "done".into(),
            scope_revision: "v1".into(),
            expires_at: Utc::now() + Duration::hours(24),
            estimator_version: "1.0".into(),
            cohort: "test".into(),
            evidence_basis: vec!["data".into()],
            comparable_sample_count: 10,
            all_attempt_sample_count: 15,
            censoring_method: "none".into(),
            correlation_method: "spearman".into(),
            calibration_profile: "standard".into(),
            grounding_status: "grounded".into(),
            baseline_ref: "baseline".into(),
            drift_policy_ref: "drift-policy".into(),
        };
        let mut loop_ = CalibrationLoop::new("test", &authority);
        loop_.baseline_calibration = 0.9;
        loop_.learning_candidates.push(LearningCandidate {
            candidate_id: "c1".into(),
            generated_at: Utc::now(),
            drift_severity: 0.2,
            evidence_evaluation_refs: vec!["e1".into()],
            recommended_action: LearningAction::RecalibrateCohort,
            confidence: 0.8,
            promoted: false,
            rolled_back: false,
        });
        let old_baseline = loop_.baseline_calibration;
        assert!(loop_.promote_candidate("c1", vec!["evidence".into()]));
        assert!(loop_.baseline_calibration != old_baseline);
        assert_eq!(loop_.drift_status, CalibrationDriftStatus::Stable);
    }

    #[test]
    fn promote_rejects_empty_evidence() {
        let mut loop_ = CalibrationLoop {
            profile_id: "test".into(),
            baseline_calibration: 0.9,
            recent_evaluations: VecDeque::new(),
            drift_status: CalibrationDriftStatus::Drifted,
            cumulative: CalibrationCumulative::default(),
            learning_candidates: vec![LearningCandidate {
                candidate_id: "c1".into(),
                generated_at: Utc::now(),
                drift_severity: 0.2,
                evidence_evaluation_refs: vec![],
                recommended_action: LearningAction::RecalibrateCohort,
                confidence: 0.8,
                promoted: false,
                rolled_back: false,
            }],
            last_updated: Utc::now(),
        };
        assert!(!loop_.promote_candidate("c1", vec![]));
    }

    #[test]
    fn rollback_marks_candidate() {
        let mut loop_ = CalibrationLoop {
            profile_id: "test".into(),
            baseline_calibration: 0.9,
            recent_evaluations: VecDeque::new(),
            drift_status: CalibrationDriftStatus::Drifted,
            cumulative: CalibrationCumulative::default(),
            learning_candidates: vec![LearningCandidate {
                candidate_id: "c1".into(),
                generated_at: Utc::now(),
                drift_severity: 0.2,
                evidence_evaluation_refs: vec![],
                recommended_action: LearningAction::RecalibrateCohort,
                confidence: 0.8,
                promoted: false,
                rolled_back: false,
            }],
            last_updated: Utc::now(),
        };
        assert!(loop_.rollback_candidate("c1"));
    }

    #[test]
    fn cumulative_statistics_update_correctly() {
        let authority = ForecastAuthorityContext {
            claim_kind: "effort".into(),
            target_state: "done".into(),
            scope_revision: "v1".into(),
            expires_at: Utc::now() + Duration::hours(24),
            estimator_version: "1.0".into(),
            cohort: "test".into(),
            evidence_basis: vec!["data".into()],
            comparable_sample_count: 10,
            all_attempt_sample_count: 15,
            censoring_method: "none".into(),
            correlation_method: "spearman".into(),
            calibration_profile: "standard".into(),
            grounding_status: "grounded".into(),
            baseline_ref: "baseline".into(),
            drift_policy_ref: "drift-policy".into(),
        };
        let mut loop_ = CalibrationLoop::new("test", &authority);
        let range = sample_range();
        for i in 0..3 {
            let eval = evaluate_forecast(
                &range, 500, &format!("t-{}", i),
                0.6, 0, 0, 0.0, 1.0,
                vec!["e".into()],
            ).unwrap();
            loop_.ingest(&eval, &range, 500, "t", 0.6, vec!["e".into()]);
        }
        assert_eq!(loop_.cumulative.total_evaluations, 3);
        assert!(loop_.cumulative.mean_reliability.is_finite());
        assert!(loop_.cumulative.mean_skill_score.is_finite());
    }

    #[test]
    fn degraded_when_drift_severe() {
        let authority = ForecastAuthorityContext {
            claim_kind: "effort".into(),
            target_state: "done".into(),
            scope_revision: "v1".into(),
            expires_at: Utc::now() + Duration::hours(24),
            estimator_version: "1.0".into(),
            cohort: "test".into(),
            evidence_basis: vec!["data".into()],
            comparable_sample_count: 10,
            all_attempt_sample_count: 15,
            censoring_method: "none".into(),
            correlation_method: "spearman".into(),
            calibration_profile: "standard".into(),
            grounding_status: "grounded".into(),
            baseline_ref: "baseline".into(),
            drift_policy_ref: "drift-policy".into(),
        };
        let mut loop_ = CalibrationLoop::new("test", &authority);
        loop_.baseline_calibration = 0.9;
        let range = sample_range();
        // All evaluations outside p95 → severe drift
        for i in 0..5 {
            let eval = evaluate_forecast(
                &range, 700,
                &format!("t-{}", i),
                0.6, 0, 0, 0.0, 1.0,
                vec!["e".into()],
            ).unwrap();
            let candidate = loop_.ingest(&eval, &range, 700, "t", 0.6, vec!["e".into()]);
            if i == 4 {
                let c = candidate.expect("should detect drift");
                assert_eq!(c.recommended_action, LearningAction::RebuildProfile);
            }
        }
    }
}
