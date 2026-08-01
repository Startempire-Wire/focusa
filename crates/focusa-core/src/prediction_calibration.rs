//! Spec138 calibration cohorts, scorer authority, and evidence-backed reports.

use crate::prediction_scoring::{ScoreInput, ScorerDescriptor, ScorerId, ScoringError, score};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationDimension {
    Target,
    Horizon,
    Entity,
    Cohort,
    Sources,
    Features,
    Model,
    Prompt,
    Policy,
    Scorer,
    Forecaster,
    ProbabilityBucket,
    Regime,
    Scenario,
    Trajectory,
    Environment,
    TimePeriod,
    TransferContext,
    VerifierCapability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationObservation {
    pub observation_id: String,
    pub commitment_id: String,
    pub probability: f64,
    pub outcome: bool,
    pub weight: f64,
    pub dimensions: BTreeMap<CalibrationDimension, String>,
    pub scorer_id: ScorerId,
    pub scorer_version: u64,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBucket {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub mean_probability: f64,
    pub observed_frequency: f64,
    pub absolute_gap: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub report_id: String,
    pub dimensions: BTreeMap<CalibrationDimension, String>,
    pub scorer_id: ScorerId,
    pub scorer_version: u64,
    pub sample_count: usize,
    pub effective_sample_weight: f64,
    pub buckets: Vec<CalibrationBucket>,
    pub expected_calibration_error: f64,
    pub maximum_calibration_error: f64,
    pub adaptive_calibration_error: f64,
    pub brier_score: f64,
    pub bias: f64,
    pub sharpness: f64,
    pub coverage_probability: f64,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationAuthority {
    pub authority_id: String,
    pub commitment_id: String,
    pub information_set_ref: String,
    pub outcome_resolution_ref: String,
    pub resolution_authority_ref: String,
    pub scorer: ScorerDescriptor,
    pub scoring_policy_ref: String,
    pub policy_locked_at: DateTime<Utc>,
    pub commitment_at: DateTime<Utc>,
    pub outcome_resolved_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalibrationError {
    EmptyCohort,
    InsufficientSample,
    InvalidProbability,
    InvalidWeight,
    CohortMismatch,
    ScorerMismatch,
    MissingEvidence,
    MissingReceipt,
    PolicyLockedAfterCommitment,
    OutcomeBeforeCommitment,
    MissingAuthority,
    Scoring(ScoringError),
}

pub fn validate_evaluation_authority(
    authority: &EvaluationAuthority,
) -> Result<(), CalibrationError> {
    if authority.commitment_id.trim().is_empty()
        || authority.information_set_ref.trim().is_empty()
        || authority.outcome_resolution_ref.trim().is_empty()
        || authority.resolution_authority_ref.trim().is_empty()
        || authority.scoring_policy_ref.trim().is_empty()
    {
        return Err(CalibrationError::MissingAuthority);
    }
    if authority.policy_locked_at > authority.commitment_at {
        return Err(CalibrationError::PolicyLockedAfterCommitment);
    }
    if authority.outcome_resolved_at < authority.commitment_at {
        return Err(CalibrationError::OutcomeBeforeCommitment);
    }
    if authority.evidence_refs.is_empty() {
        return Err(CalibrationError::MissingEvidence);
    }
    if authority.receipt_ref.trim().is_empty() {
        return Err(CalibrationError::MissingReceipt);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build_calibration_report(
    report_id: impl Into<String>,
    observations: &[CalibrationObservation],
    dimensions: BTreeMap<CalibrationDimension, String>,
    scorer_id: ScorerId,
    scorer_version: u64,
    bucket_count: usize,
    minimum_sample_count: usize,
    receipt_ref: impl Into<String>,
    generated_at: DateTime<Utc>,
) -> Result<CalibrationReport, CalibrationError> {
    if observations.is_empty() {
        return Err(CalibrationError::EmptyCohort);
    }
    if observations.len() < minimum_sample_count || bucket_count == 0 {
        return Err(CalibrationError::InsufficientSample);
    }
    let receipt_ref = receipt_ref.into();
    if receipt_ref.trim().is_empty() {
        return Err(CalibrationError::MissingReceipt);
    }
    let mut probabilities = Vec::with_capacity(observations.len());
    let mut outcomes = Vec::with_capacity(observations.len());
    let mut evidence = Vec::new();
    let mut total_weight = 0.0;
    for observation in observations {
        if !(0.0..=1.0).contains(&observation.probability) {
            return Err(CalibrationError::InvalidProbability);
        }
        if !observation.weight.is_finite() || observation.weight <= 0.0 {
            return Err(CalibrationError::InvalidWeight);
        }
        if observation.scorer_id != scorer_id || observation.scorer_version != scorer_version {
            return Err(CalibrationError::ScorerMismatch);
        }
        if dimensions
            .iter()
            .any(|(key, value)| observation.dimensions.get(key) != Some(value))
        {
            return Err(CalibrationError::CohortMismatch);
        }
        if observation.evidence_refs.is_empty() {
            return Err(CalibrationError::MissingEvidence);
        }
        probabilities.push(observation.probability);
        outcomes.push(observation.outcome);
        total_weight += observation.weight;
        evidence.extend(observation.evidence_refs.clone());
    }
    evidence.sort();
    evidence.dedup();
    let calibration_input = ScoreInput::Calibration {
        probabilities: probabilities.clone(),
        outcomes: outcomes.clone(),
        bucket_count,
    };
    let expected_calibration_error = score(ScorerId::ExpectedCalibrationError, &calibration_input)
        .map_err(CalibrationError::Scoring)?;
    let maximum_calibration_error = score(ScorerId::MaximumCalibrationError, &calibration_input)
        .map_err(CalibrationError::Scoring)?;
    let adaptive_calibration_error = score(ScorerId::AdaptiveCalibrationError, &calibration_input)
        .map_err(CalibrationError::Scoring)?;
    let mut bins = vec![Vec::<usize>::new(); bucket_count];
    for (index, probability) in probabilities.iter().enumerate() {
        let bucket = ((probability * bucket_count as f64) as usize).min(bucket_count - 1);
        bins[bucket].push(index);
    }
    let buckets = bins
        .into_iter()
        .enumerate()
        .filter(|(_, indexes)| !indexes.is_empty())
        .map(|(index, indexes)| {
            let mean_probability =
                indexes.iter().map(|i| probabilities[*i]).sum::<f64>() / indexes.len() as f64;
            let observed_frequency =
                indexes.iter().map(|i| f64::from(outcomes[*i])).sum::<f64>() / indexes.len() as f64;
            CalibrationBucket {
                lower: index as f64 / bucket_count as f64,
                upper: (index + 1) as f64 / bucket_count as f64,
                count: indexes.len(),
                mean_probability,
                observed_frequency,
                absolute_gap: (mean_probability - observed_frequency).abs(),
            }
        })
        .collect();
    let brier_score = probabilities
        .iter()
        .zip(&outcomes)
        .map(|(p, outcome)| (p - f64::from(*outcome)).powi(2))
        .sum::<f64>()
        / observations.len() as f64;
    let mean_probability = probabilities.iter().sum::<f64>() / observations.len() as f64;
    let outcome_rate =
        outcomes.iter().map(|value| f64::from(*value)).sum::<f64>() / observations.len() as f64;
    let sharpness = probabilities
        .iter()
        .map(|p| (p - 0.5).abs() * 2.0)
        .sum::<f64>()
        / observations.len() as f64;
    Ok(CalibrationReport {
        report_id: report_id.into(),
        dimensions,
        scorer_id,
        scorer_version,
        sample_count: observations.len(),
        effective_sample_weight: total_weight,
        buckets,
        expected_calibration_error,
        maximum_calibration_error,
        adaptive_calibration_error,
        brier_score,
        bias: mean_probability - outcome_rate,
        sharpness,
        coverage_probability: observations.len() as f64
            / minimum_sample_count.max(observations.len()) as f64,
        evidence_refs: evidence,
        receipt_ref,
        generated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction_scoring::required_scorer_registry;

    fn observations() -> Vec<CalibrationObservation> {
        [
            ("a", 0.1, false),
            ("b", 0.2, false),
            ("c", 0.8, true),
            ("d", 0.9, true),
        ]
        .into_iter()
        .map(|(id, probability, outcome)| CalibrationObservation {
            observation_id: id.into(),
            commitment_id: format!("commitment-{id}"),
            probability,
            outcome,
            weight: 1.0,
            dimensions: BTreeMap::from([(CalibrationDimension::Target, "release".into())]),
            scorer_id: ScorerId::BrierScore,
            scorer_version: 1,
            evidence_refs: vec![format!("evidence:{id}")],
        })
        .collect()
    }

    #[test]
    fn calibration_report_is_cohort_and_scorer_bound() {
        let report = build_calibration_report(
            "report-1",
            &observations(),
            BTreeMap::from([(CalibrationDimension::Target, "release".into())]),
            ScorerId::BrierScore,
            1,
            2,
            4,
            "receipt:report",
            Utc::now(),
        )
        .unwrap();
        assert_eq!(report.sample_count, 4);
        assert!(report.expected_calibration_error <= 0.2);
        assert_eq!(report.evidence_refs.len(), 4);
    }

    #[test]
    fn calibration_rejects_small_or_mismatched_cohorts() {
        assert_eq!(
            build_calibration_report(
                "report",
                &observations()[..1],
                BTreeMap::new(),
                ScorerId::BrierScore,
                1,
                2,
                4,
                "receipt",
                Utc::now(),
            ),
            Err(CalibrationError::InsufficientSample)
        );
        let mut wrong = observations();
        wrong[0].scorer_version = 2;
        assert_eq!(
            build_calibration_report(
                "report",
                &wrong,
                BTreeMap::new(),
                ScorerId::BrierScore,
                1,
                2,
                4,
                "receipt",
                Utc::now(),
            ),
            Err(CalibrationError::ScorerMismatch)
        );
    }

    #[test]
    fn evaluation_authority_blocks_post_outcome_policy_choice() {
        let scorer = required_scorer_registry()
            .into_iter()
            .find(|value| value.id == ScorerId::BrierScore)
            .unwrap();
        let now = Utc::now();
        let authority = EvaluationAuthority {
            authority_id: "authority".into(),
            commitment_id: "commitment".into(),
            information_set_ref: "information-set".into(),
            outcome_resolution_ref: "resolution".into(),
            resolution_authority_ref: "resolver".into(),
            scorer,
            scoring_policy_ref: "policy:v1".into(),
            policy_locked_at: now + chrono::Duration::seconds(1),
            commitment_at: now,
            outcome_resolved_at: now + chrono::Duration::seconds(2),
            evidence_refs: vec!["evidence".into()],
            receipt_ref: "receipt".into(),
        };
        assert_eq!(
            validate_evaluation_authority(&authority),
            Err(CalibrationError::PolicyLockedAfterCommitment)
        );
    }
}
