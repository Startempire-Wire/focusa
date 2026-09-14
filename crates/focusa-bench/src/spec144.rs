//! Deterministic Spec 144 §28 promotion metrics.
//!
//! Fixture execution lives in `../spec144_evaluation.py` so the release gate can
//! run without compiling the workspace. These types keep the benchmark contract
//! available to Rust callers.

use serde::{Deserialize, Serialize};

pub const COMPARISON_COHORTS: [&str; 6] = [
    "builder_only",
    "same_model_self_review",
    "same_model_separate_context",
    "cross_family_verification",
    "deterministic_model_verification",
    "multi_aspect_portfolio",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationMetrics {
    pub precision: f64,
    pub recall: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub coverage: f64,
    pub calibration_ece: f64,
    pub p95_latency_ms: u64,
    pub resource_units: u64,
    pub replay_equivalence: bool,
    pub golden_pass_rate: f64,
    pub blocking_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromotionThresholds {
    pub precision_min: f64,
    pub recall_min: f64,
    pub false_positive_rate_max: f64,
    pub false_negative_rate_max: f64,
    pub coverage_min: f64,
    pub calibration_ece_max: f64,
    pub p95_latency_ms_max: u64,
    pub resource_units_max: u64,
    pub replay_equivalence_required: bool,
    pub golden_pass_rate_min: f64,
    pub blocking_failures_max: u64,
}

impl PromotionThresholds {
    /// Every threshold is conjunctive. Unknown or blocking results cannot be
    /// averaged away by stronger scores in another metric.
    pub fn failures(&self, value: &EvaluationMetrics) -> Vec<&'static str> {
        let checks = [
            (value.precision >= self.precision_min, "precision_min"),
            (value.recall >= self.recall_min, "recall_min"),
            (
                value.false_positive_rate <= self.false_positive_rate_max,
                "false_positive_rate_max",
            ),
            (
                value.false_negative_rate <= self.false_negative_rate_max,
                "false_negative_rate_max",
            ),
            (value.coverage >= self.coverage_min, "coverage_min"),
            (
                value.calibration_ece <= self.calibration_ece_max,
                "calibration_ece_max",
            ),
            (
                value.p95_latency_ms <= self.p95_latency_ms_max,
                "p95_latency_ms_max",
            ),
            (
                value.resource_units <= self.resource_units_max,
                "resource_units_max",
            ),
            (
                value.replay_equivalence == self.replay_equivalence_required,
                "replay_equivalence_required",
            ),
            (
                value.golden_pass_rate >= self.golden_pass_rate_min,
                "golden_pass_rate_min",
            ),
            (
                value.blocking_failures <= self.blocking_failures_max,
                "blocking_failures_max",
            ),
        ];
        checks
            .into_iter()
            .filter_map(|(pass, name)| (!pass).then_some(name))
            .collect()
    }

    pub fn eligible(&self, value: &EvaluationMetrics) -> bool {
        self.failures(value).is_empty()
    }
}
