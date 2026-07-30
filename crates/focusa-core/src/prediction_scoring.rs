//! Spec138 versioned scorer registry and deterministic scoring fixtures.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorerId {
    BinaryAccuracy,
    MulticlassAccuracy,
    BrierScore,
    MulticlassBrierScore,
    LogLoss,
    MulticlassLogLoss,
    SphericalScore,
    ContinuousRankedProbabilityScore,
    MeanAbsoluteError,
    MeanSquaredError,
    RootMeanSquaredError,
    MeanAbsolutePercentageError,
    SymmetricMape,
    QuantilePinballLoss,
    IntervalCoverage,
    IntervalWidth,
    WinklerIntervalScore,
    RankCorrelation,
    InformationCoefficient,
    TopKPrecision,
    TopKRecall,
    Ndcg,
    ConcordanceIndex,
    SurvivalBrierScore,
    ExpectedCalibrationError,
    MaximumCalibrationError,
    AdaptiveCalibrationError,
    SkillScore,
    ExpectedUtility,
    RealizedRegret,
    CustomRegistered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreDirection {
    LowerIsBetter,
    HigherIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreShape {
    Binary,
    Categorical,
    NumericSeries,
    Samples,
    Quantile,
    Interval,
    Ranking,
    Survival,
    Calibration,
    Baseline,
    Utility,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScorerDescriptor {
    pub id: ScorerId,
    pub version: u64,
    pub direction: ScoreDirection,
    pub shape: ScoreShape,
    pub range_min: Option<f64>,
    pub range_max: Option<f64>,
    pub assumptions: Vec<String>,
    pub fixture_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum ScoreInput {
    Binary {
        probability: f64,
        outcome: bool,
    },
    Categorical {
        probabilities: Vec<f64>,
        outcome_index: usize,
    },
    NumericSeries {
        forecasts: Vec<f64>,
        actuals: Vec<f64>,
    },
    Samples {
        forecast_samples: Vec<f64>,
        actual: f64,
    },
    Quantile {
        quantile: f64,
        forecast: f64,
        actual: f64,
    },
    Interval {
        lower: f64,
        upper: f64,
        alpha: f64,
        actual: f64,
    },
    Ranking {
        scores: Vec<f64>,
        relevance: Vec<f64>,
        k: usize,
    },
    Survival {
        probabilities: Vec<f64>,
        outcomes: Vec<bool>,
    },
    Calibration {
        probabilities: Vec<f64>,
        outcomes: Vec<bool>,
        bucket_count: usize,
    },
    Baseline {
        score: f64,
        baseline_score: f64,
        lower_is_better: bool,
    },
    Utility {
        expected: f64,
        realized: f64,
        best_available: f64,
    },
    Custom {
        values: Vec<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoringError {
    ShapeMismatch,
    InvalidProbability,
    InvalidDistribution,
    InvalidOutcome,
    EmptyInput,
    LengthMismatch,
    InvalidParameter,
    DivisionByZero,
    CustomScorerRequiresRegistration,
}

pub fn required_scorer_registry() -> Vec<ScorerDescriptor> {
    use ScoreDirection::{HigherIsBetter as High, LowerIsBetter as Low};
    use ScoreShape::*;
    use ScorerId::*;
    let rows = [
        (BinaryAccuracy, High, Binary),
        (MulticlassAccuracy, High, Categorical),
        (BrierScore, Low, Binary),
        (MulticlassBrierScore, Low, Categorical),
        (LogLoss, Low, Binary),
        (MulticlassLogLoss, Low, Categorical),
        (SphericalScore, High, Categorical),
        (ContinuousRankedProbabilityScore, Low, Samples),
        (MeanAbsoluteError, Low, NumericSeries),
        (MeanSquaredError, Low, NumericSeries),
        (RootMeanSquaredError, Low, NumericSeries),
        (MeanAbsolutePercentageError, Low, NumericSeries),
        (SymmetricMape, Low, NumericSeries),
        (QuantilePinballLoss, Low, Quantile),
        (IntervalCoverage, High, Interval),
        (IntervalWidth, Low, Interval),
        (WinklerIntervalScore, Low, Interval),
        (RankCorrelation, High, Ranking),
        (InformationCoefficient, High, Ranking),
        (TopKPrecision, High, Ranking),
        (TopKRecall, High, Ranking),
        (Ndcg, High, Ranking),
        (ConcordanceIndex, High, Ranking),
        (SurvivalBrierScore, Low, Survival),
        (ExpectedCalibrationError, Low, Calibration),
        (MaximumCalibrationError, Low, Calibration),
        (AdaptiveCalibrationError, Low, Calibration),
        (SkillScore, High, Baseline),
        (ExpectedUtility, High, Utility),
        (RealizedRegret, Low, Utility),
        (CustomRegistered, Low, Custom),
    ];
    rows.into_iter()
        .map(|(id, direction, shape)| ScorerDescriptor {
            id,
            version: 1,
            direction,
            shape,
            range_min: None,
            range_max: None,
            assumptions: vec!["input shape validated before scoring".into()],
            fixture_refs: vec!["crates/focusa-core/src/prediction_scoring.rs#tests".into()],
            evidence_refs: vec![
                "docs/contracts/spec138-scorer-and-calibration-matrix.v1.yaml".into(),
            ],
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomScorerRegistration {
    pub scorer_id: String,
    pub version: u64,
    pub direction: ScoreDirection,
    pub range_min: f64,
    pub range_max: f64,
    pub assumptions: Vec<String>,
    pub fixture_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub implementation_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScorerRegistryError {
    ReservedIdentity,
    InvalidVersion,
    InvalidRange,
    MissingAssumptions,
    MissingFixtures,
    MissingEvidence,
    MissingImplementation,
}

#[derive(Debug, Clone)]
pub struct VersionedScorerRegistry {
    required: BTreeMap<ScorerId, ScorerDescriptor>,
    custom: BTreeMap<(String, u64), CustomScorerRegistration>,
}

impl Default for VersionedScorerRegistry {
    fn default() -> Self {
        Self {
            required: required_scorer_registry()
                .into_iter()
                .map(|descriptor| (descriptor.id, descriptor))
                .collect(),
            custom: BTreeMap::new(),
        }
    }
}

impl VersionedScorerRegistry {
    pub fn required(&self, id: ScorerId) -> Option<&ScorerDescriptor> {
        self.required.get(&id)
    }

    pub fn custom(&self, scorer_id: &str, version: u64) -> Option<&CustomScorerRegistration> {
        self.custom.get(&(scorer_id.to_string(), version))
    }

    pub fn register_custom(
        &mut self,
        registration: CustomScorerRegistration,
    ) -> Result<(), ScorerRegistryError> {
        let reserved = required_scorer_registry().iter().any(|descriptor| {
            serde_json::to_value(descriptor.id)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .as_deref()
                == Some(registration.scorer_id.as_str())
        });
        if registration.scorer_id.trim().is_empty() || reserved {
            return Err(ScorerRegistryError::ReservedIdentity);
        }
        let latest = self
            .custom
            .keys()
            .filter(|(id, _)| id == &registration.scorer_id)
            .map(|(_, version)| *version)
            .max()
            .unwrap_or(0);
        if registration.version != latest + 1 {
            return Err(ScorerRegistryError::InvalidVersion);
        }
        if !registration.range_min.is_finite()
            || !registration.range_max.is_finite()
            || registration.range_min >= registration.range_max
        {
            return Err(ScorerRegistryError::InvalidRange);
        }
        if registration.assumptions.is_empty() {
            return Err(ScorerRegistryError::MissingAssumptions);
        }
        if registration.fixture_refs.is_empty() {
            return Err(ScorerRegistryError::MissingFixtures);
        }
        if registration.evidence_refs.is_empty() {
            return Err(ScorerRegistryError::MissingEvidence);
        }
        if registration.implementation_ref.trim().is_empty() {
            return Err(ScorerRegistryError::MissingImplementation);
        }
        self.custom.insert(
            (registration.scorer_id.clone(), registration.version),
            registration,
        );
        Ok(())
    }
}

pub use crate::prediction_scoring_algorithms::score;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_contains_all_required_scorers() {
        let r = required_scorer_registry();
        assert_eq!(r.len(), 31);
        assert_eq!(
            r.iter()
                .map(|v| v.id)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            31
        );
        assert!(
            r.iter().all(|v| v.version > 0
                && !v.fixture_refs.is_empty()
                && !v.evidence_refs.is_empty())
        );
    }
    #[test]
    fn custom_scorer_registration_is_versioned_and_proven() {
        let mut registry = VersionedScorerRegistry::default();
        let registration = CustomScorerRegistration {
            scorer_id: "domain_cost_score".into(),
            version: 1,
            direction: ScoreDirection::LowerIsBetter,
            range_min: 0.0,
            range_max: 100.0,
            assumptions: vec!["bounded domain cost".into()],
            fixture_refs: vec!["fixture:domain-cost".into()],
            evidence_refs: vec!["evidence:domain-cost".into()],
            implementation_ref: "adapter:domain-cost".into(),
        };
        registry.register_custom(registration.clone()).unwrap();
        assert!(registry.custom("domain_cost_score", 1).is_some());
        assert_eq!(
            registry.register_custom(registration),
            Err(ScorerRegistryError::InvalidVersion)
        );
    }

    #[test]
    fn proper_binary_and_multiclass_scores_match_fixtures() {
        assert!(
            (score(
                ScorerId::BrierScore,
                &ScoreInput::Binary {
                    probability: 0.8,
                    outcome: true
                }
            )
            .unwrap()
                - 0.04)
                .abs()
                < 1e-12
        );
        assert!(
            (score(
                ScorerId::LogLoss,
                &ScoreInput::Binary {
                    probability: 0.8,
                    outcome: true
                }
            )
            .unwrap()
                + 0.8f64.ln())
            .abs()
                < 1e-12
        );
        assert!(
            (score(
                ScorerId::MulticlassBrierScore,
                &ScoreInput::Categorical {
                    probabilities: vec![0.7, 0.2, 0.1],
                    outcome_index: 0
                }
            )
            .unwrap()
                - 0.14)
                .abs()
                < 1e-12
        );
    }
    #[test]
    fn malformed_shapes_probabilities_and_custom_scorers_fail_closed() {
        assert_eq!(
            score(
                ScorerId::BrierScore,
                &ScoreInput::Binary {
                    probability: 1.2,
                    outcome: true
                }
            ),
            Err(ScoringError::InvalidProbability)
        );
        assert_eq!(
            score(
                ScorerId::CustomRegistered,
                &ScoreInput::Custom { values: vec![1.0] }
            ),
            Err(ScoringError::CustomScorerRequiresRegistration)
        );
        assert_eq!(
            score(
                ScorerId::LogLoss,
                &ScoreInput::NumericSeries {
                    forecasts: vec![1.0],
                    actuals: vec![1.0]
                }
            ),
            Err(ScoringError::ShapeMismatch)
        );
    }
    #[test]
    fn interval_ranking_calibration_and_utility_fixtures_are_bounded() {
        assert_eq!(
            score(
                ScorerId::IntervalCoverage,
                &ScoreInput::Interval {
                    lower: 1.0,
                    upper: 3.0,
                    alpha: 0.1,
                    actual: 2.0
                }
            )
            .unwrap(),
            1.0
        );
        let ece = score(
            ScorerId::ExpectedCalibrationError,
            &ScoreInput::Calibration {
                probabilities: vec![0.1, 0.9],
                outcomes: vec![false, true],
                bucket_count: 2,
            },
        )
        .unwrap();
        assert!((0.0..=1.0).contains(&ece));
        assert_eq!(
            score(
                ScorerId::RealizedRegret,
                &ScoreInput::Utility {
                    expected: 5.0,
                    realized: 3.0,
                    best_available: 7.0
                }
            )
            .unwrap(),
            4.0
        );
    }
}
