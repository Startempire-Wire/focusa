//! Spec 138 generic prediction, outcome, learning, and transfer authority.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicScope {
    pub project_root: String,
    pub continuity_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceDimensions {
    pub forecast_probability: f64,
    pub evidence_confidence: f64,
    pub source_reliability: f64,
    pub model_confidence: f64,
    pub resolution_confidence: Option<f64>,
}

impl ConfidenceDimensions {
    pub fn validate(&self) -> Result<(), String> {
        let values = [
            self.forecast_probability,
            self.evidence_confidence,
            self.source_reliability,
            self.model_confidence,
        ];
        if values.into_iter().all(|value| (0.0..=1.0).contains(&value))
            && self
                .resolution_confidence
                .is_none_or(|value| (0.0..=1.0).contains(&value))
        {
            Ok(())
        } else {
            Err("confidence dimensions must be distinct values in [0,1]".into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionQuestion {
    pub question_id: String,
    pub subject_ref: String,
    pub outcome_space: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub horizon_claim_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InformationSetRef {
    pub information_set_id: String,
    pub version: u64,
    pub as_of_claim_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionCommitment {
    pub commitment_id: String,
    pub question_id: String,
    pub predicted_outcome: String,
    pub confidence: ConfidenceDimensions,
    pub information_set: InformationSetRef,
    pub resolver_policy_ref: String,
    pub scoring_policy_ref: String,
    pub committed_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeClaim {
    pub claim_id: String,
    pub commitment_id: String,
    pub claimed_outcome: String,
    pub claimed_at: DateTime<Utc>,
    pub source_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeResolution {
    pub resolution_id: String,
    pub claim_id: String,
    pub resolved_outcome: String,
    pub resolver_policy_ref: String,
    pub resolved_at: DateTime<Utc>,
    pub resolution_confidence: f64,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreDirection {
    LowerIsBetter,
    HigherIsBetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorerKind {
    BrierBinary,
    LogBinary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoringPolicy {
    pub policy_id: String,
    pub version: u64,
    pub scorer: ScorerKind,
    pub direction: ScoreDirection,
    pub range_min: f64,
    pub range_max: f64,
    pub assumptions: Vec<String>,
    pub frozen_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

impl ScoringPolicy {
    pub fn policy_ref(&self) -> String {
        format!("{}@{}", self.policy_id, self.version)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScorerRegistry {
    policies: BTreeMap<String, ScoringPolicy>,
}

impl ScorerRegistry {
    pub fn register(&mut self, policy: ScoringPolicy) -> Result<String, String> {
        let key = policy.policy_ref();
        if self.policies.contains_key(&key) {
            return Err(format!("frozen scoring policy already exists: {key}"));
        }
        if policy.range_min >= policy.range_max {
            return Err("scoring policy range is invalid".into());
        }
        self.policies.insert(key.clone(), policy);
        Ok(key)
    }

    pub fn get(&self, policy_ref: &str) -> Option<&ScoringPolicy> {
        self.policies.get(policy_ref)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionEvaluation {
    pub evaluation_id: String,
    pub commitment_id: String,
    pub resolution_id: String,
    pub scoring_policy_ref: String,
    pub canonical_score: f64,
    pub correct: bool,
    pub evaluated_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

pub fn evaluate_binary(
    commitment: &PredictionCommitment,
    resolution: &OutcomeResolution,
    policy: &ScoringPolicy,
    evaluation_id: impl Into<String>,
    evaluated_at: DateTime<Utc>,
    receipt_ref: impl Into<String>,
) -> Result<PredictionEvaluation, String> {
    commitment.confidence.validate()?;
    if commitment.scoring_policy_ref != policy.policy_ref() {
        return Err("commitment scoring policy is not the frozen scorer version".into());
    }
    if commitment.resolver_policy_ref != resolution.resolver_policy_ref {
        return Err("resolution used a different resolver policy".into());
    }
    let correct = commitment.predicted_outcome == resolution.resolved_outcome;
    let p = commitment
        .confidence
        .forecast_probability
        .clamp(1e-12, 1.0 - 1e-12);
    let actual = if correct { 1.0 } else { 0.0 };
    let score = match policy.scorer {
        ScorerKind::BrierBinary => (p - actual).powi(2),
        ScorerKind::LogBinary => -(actual * p.ln() + (1.0 - actual) * (1.0 - p).ln()),
    };
    if !(policy.range_min..=policy.range_max).contains(&score) {
        return Err("canonical score falls outside frozen policy range".into());
    }
    Ok(PredictionEvaluation {
        evaluation_id: evaluation_id.into(),
        commitment_id: commitment.commitment_id.clone(),
        resolution_id: resolution.resolution_id.clone(),
        scoring_policy_ref: policy.policy_ref(),
        canonical_score: score,
        correct,
        evaluated_at,
        evidence_refs: resolution.evidence_refs.clone(),
        receipt_ref: receipt_ref.into(),
    })
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationProjection {
    pub scorer_policy_ref: String,
    pub sample_size: u64,
    pub predicted_probability_sum: f64,
    pub observed_successes: u64,
    pub score_sum: f64,
}

impl CalibrationProjection {
    pub fn observe(&mut self, probability: f64, evaluation: &PredictionEvaluation) {
        self.sample_size += 1;
        self.predicted_probability_sum += probability;
        self.observed_successes += u64::from(evaluation.correct);
        self.score_sum += evaluation.canonical_score;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applicability {
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningCandidate {
    pub candidate_id: String,
    pub evaluation_id: String,
    pub reason_code: String,
    pub hypothesis: String,
    pub applicability: Applicability,
    pub created_at: DateTime<Utc>,
    pub review_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

pub fn high_confidence_miss_candidate(
    commitment: &PredictionCommitment,
    evaluation: &PredictionEvaluation,
    candidate_id: impl Into<String>,
    now: DateTime<Utc>,
    review_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Option<LearningCandidate> {
    (!evaluation.correct && commitment.confidence.forecast_probability >= 0.8).then(|| {
        LearningCandidate {
            candidate_id: candidate_id.into(),
            evaluation_id: evaluation.evaluation_id.clone(),
            reason_code: "high_confidence_miss".into(),
            hypothesis: "Inspect information-set, source-reliability, and model assumptions".into(),
            applicability: Applicability {
                includes: vec![commitment.question_id.clone()],
                excludes: Vec::new(),
            },
            created_at: now,
            review_at,
            expires_at,
            evidence_refs: evaluation.evidence_refs.clone(),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedMetric {
    pub metric_id: String,
    pub value: f64,
    pub baseline: f64,
    pub sample_size: u64,
    pub higher_is_better: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionDecisionKind {
    Promoted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub decision_id: String,
    pub candidate_id: String,
    pub decision: PromotionDecisionKind,
    pub reason_codes: Vec<String>,
    pub decided_at: DateTime<Utc>,
    pub receipt_ref: String,
}

pub fn decide_promotion(
    candidate: &LearningCandidate,
    typed_metrics: &[TypedMetric],
    prose_metrics: &[String],
    negative_effect: bool,
    decision_id: impl Into<String>,
    now: DateTime<Utc>,
    receipt_ref: impl Into<String>,
) -> PromotionDecision {
    let mut reasons = Vec::new();
    if typed_metrics.is_empty() {
        reasons.push(if prose_metrics.is_empty() {
            "typed_metrics_missing"
        } else {
            "prose_only_metrics"
        });
    }
    if typed_metrics.iter().any(|metric| metric.sample_size == 0) {
        reasons.push("sample_size_missing");
    }
    if negative_effect {
        reasons.push("negative_effect_detected");
    }
    if candidate.applicability.includes.is_empty() || candidate.expires_at <= now {
        reasons.push("applicability_or_expiry_invalid");
    }
    let improved = typed_metrics.iter().all(|metric| {
        if metric.higher_is_better {
            metric.value > metric.baseline
        } else {
            metric.value < metric.baseline
        }
    });
    if !typed_metrics.is_empty() && !improved {
        reasons.push("baseline_not_beaten");
    }
    PromotionDecision {
        decision_id: decision_id.into(),
        candidate_id: candidate.candidate_id.clone(),
        decision: if reasons.is_empty() {
            PromotionDecisionKind::Promoted
        } else {
            PromotionDecisionKind::Rejected
        },
        reason_codes: reasons.into_iter().map(str::to_string).collect(),
        decided_at: now,
        receipt_ref: receipt_ref.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningStatus {
    Active,
    Superseded,
    Revoked,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningRecord {
    pub learning_id: String,
    pub candidate_id: String,
    pub decision_id: String,
    pub content: String,
    pub applicability: Applicability,
    pub status: LearningStatus,
    pub review_at_claim_ref: String,
    pub expiry_claim_ref: String,
    pub supersedes: Option<String>,
    pub rollback_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferPrediction {
    pub transfer_id: String,
    pub learning_id: String,
    pub source_context_ref: String,
    pub target_context_ref: String,
    pub expected_metric_delta: f64,
    pub window_claim_ref: String,
    pub exclusions: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferResult {
    Success,
    NoEffect,
    NegativeTransfer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferOutcome {
    pub outcome_id: String,
    pub transfer_id: String,
    pub result: TransferResult,
    pub observed_metric_delta: f64,
    pub resolved_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum PredictionAuthorityEvent {
    Question(PredictionQuestion),
    Commitment(PredictionCommitment),
    OutcomeClaim(OutcomeClaim),
    OutcomeResolution(OutcomeResolution),
    ScoringPolicy(ScoringPolicy),
    Evaluation(PredictionEvaluation),
    LearningCandidate(LearningCandidate),
    PromotionDecision(PromotionDecision),
    LearningRecord(LearningRecord),
    TransferPrediction(TransferPrediction),
    TransferOutcome(TransferOutcome),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedAuthorityEvent {
    pub event_id: String,
    pub sequence: u64,
    pub scope: EpistemicScope,
    pub recorded_at: DateTime<Utc>,
    pub event: PredictionAuthorityEvent,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[cfg(test)]
#[path = "prediction_authority_tests.rs"]
mod tests;
