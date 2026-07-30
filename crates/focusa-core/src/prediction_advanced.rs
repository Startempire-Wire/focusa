//! Spec138 scenario, transfer, and calibrated self-model authority.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioCausalStatus {
    Descriptive,
    Conditional,
    CounterfactualHypothesis,
    CausalSupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioBranch {
    pub branch_id: String,
    pub condition: String,
    pub probability: f64,
    pub forecast_value: f64,
    pub intervention_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioDefinition {
    pub scenario_id: String,
    pub baseline_ref: String,
    pub causal_status: ScenarioCausalStatus,
    pub assumptions: Vec<String>,
    pub branches: Vec<ScenarioBranch>,
    pub alternative_scenario_refs: Vec<String>,
    pub disconfirming_evidence_refs: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioProjection {
    pub scenario_id: String,
    pub expected_value: f64,
    pub branch_contributions: Vec<(String, f64)>,
    pub causal_status: ScenarioCausalStatus,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferAssessment {
    pub transfer_id: String,
    pub learning_ref: String,
    pub source_context_ref: String,
    pub target_context_ref: String,
    pub similarity_score: f64,
    pub difference_refs: Vec<String>,
    pub expected_effect: f64,
    pub expected_benefit: f64,
    pub risk: f64,
    pub confidence: f64,
    pub uncertainty: f64,
    pub evaluation_plan_ref: String,
    pub exclusion_refs: Vec<String>,
    pub failure_mode_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferEvaluation {
    pub evaluation_id: String,
    pub transfer_id: String,
    pub baseline_metric: f64,
    pub observed_metric: f64,
    pub observed_effect: f64,
    pub negative_transfer: bool,
    pub confidence: f64,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferDisposition {
    Apply,
    BoundedExperiment,
    Reject,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfModelEstimate {
    pub estimate_id: String,
    pub task_ref: String,
    pub domain_ref: String,
    pub regime_ref: String,
    pub capability_score: f64,
    pub calibration_error: f64,
    pub confidence: f64,
    pub uncertainty: f64,
    pub abstention_threshold: f64,
    pub sample_count: usize,
    pub version: u64,
    pub supersedes_version: Option<u64>,
    pub limitation_refs: Vec<String>,
    pub error_mode_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub observed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancedPredictionError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    InvalidProbability,
    InvalidScenarioMass,
    UnsupportedCausalClaim,
    InvalidTransfer,
    TransferMismatch,
    InvalidSelfModel,
    GlobalSelfModelProhibited,
    Expired,
}

pub fn project_scenario(
    scenario: &ScenarioDefinition,
    now: DateTime<Utc>,
) -> Result<ScenarioProjection, AdvancedPredictionError> {
    if scenario.scenario_id.trim().is_empty() || scenario.baseline_ref.trim().is_empty() {
        return Err(AdvancedPredictionError::MissingIdentity);
    }
    if scenario.assumptions.is_empty() || scenario.evidence_refs.is_empty() {
        return Err(AdvancedPredictionError::MissingEvidence);
    }
    if scenario.receipt_ref.trim().is_empty() {
        return Err(AdvancedPredictionError::MissingReceipt);
    }
    if scenario.expires_at <= now {
        return Err(AdvancedPredictionError::Expired);
    }
    if scenario.branches.is_empty() {
        return Err(AdvancedPredictionError::InvalidScenarioMass);
    }
    let mut total = 0.0;
    let mut evidence_refs = scenario.evidence_refs.clone();
    let mut contributions = Vec::new();
    for branch in &scenario.branches {
        if branch.branch_id.trim().is_empty()
            || branch.condition.trim().is_empty()
            || !(0.0..=1.0).contains(&branch.probability)
        {
            return Err(AdvancedPredictionError::InvalidProbability);
        }
        if branch.evidence_refs.is_empty() {
            return Err(AdvancedPredictionError::MissingEvidence);
        }
        total += branch.probability;
        contributions.push((
            branch.branch_id.clone(),
            branch.probability * branch.forecast_value,
        ));
        evidence_refs.extend(branch.evidence_refs.clone());
    }
    if (total - 1.0).abs() > 1e-9 {
        return Err(AdvancedPredictionError::InvalidScenarioMass);
    }
    if matches!(
        scenario.causal_status,
        ScenarioCausalStatus::CausalSupported
    ) && (scenario.alternative_scenario_refs.is_empty()
        || scenario.disconfirming_evidence_refs.is_empty()
        || scenario
            .branches
            .iter()
            .all(|branch| branch.intervention_refs.is_empty()))
    {
        return Err(AdvancedPredictionError::UnsupportedCausalClaim);
    }
    let expected_value = contributions.iter().map(|(_, value)| *value).sum();
    evidence_refs.sort();
    evidence_refs.dedup();
    Ok(ScenarioProjection {
        scenario_id: scenario.scenario_id.clone(),
        expected_value,
        branch_contributions: contributions,
        causal_status: scenario.causal_status,
        evidence_refs,
        receipt_ref: scenario.receipt_ref.clone(),
    })
}

pub fn evaluate_transfer(
    assessment: &TransferAssessment,
    evaluation_id: impl Into<String>,
    baseline_metric: f64,
    observed_metric: f64,
    confidence: f64,
    evidence_refs: Vec<String>,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<(TransferEvaluation, TransferDisposition), AdvancedPredictionError> {
    if assessment.transfer_id.trim().is_empty()
        || assessment.learning_ref.trim().is_empty()
        || assessment.source_context_ref.trim().is_empty()
        || assessment.target_context_ref.trim().is_empty()
    {
        return Err(AdvancedPredictionError::MissingIdentity);
    }
    if !(0.0..=1.0).contains(&assessment.similarity_score)
        || !(0.0..=1.0).contains(&assessment.risk)
        || !(0.0..=1.0).contains(&assessment.confidence)
        || !(0.0..=1.0).contains(&assessment.uncertainty)
        || !(0.0..=1.0).contains(&confidence)
        || assessment.difference_refs.is_empty()
        || assessment.evaluation_plan_ref.trim().is_empty()
    {
        return Err(AdvancedPredictionError::InvalidTransfer);
    }
    if assessment.evidence_refs.is_empty() || evidence_refs.is_empty() {
        return Err(AdvancedPredictionError::MissingEvidence);
    }
    let receipt_ref = receipt_ref.into();
    if assessment.receipt_ref.trim().is_empty() || receipt_ref.trim().is_empty() {
        return Err(AdvancedPredictionError::MissingReceipt);
    }
    let effect = observed_metric - baseline_metric;
    let negative_transfer = effect < 0.0;
    let disposition = if negative_transfer {
        TransferDisposition::Rollback
    } else if assessment.similarity_score >= 0.8
        && assessment.uncertainty <= 0.2
        && confidence >= 0.8
    {
        TransferDisposition::Apply
    } else if confidence >= 0.5 {
        TransferDisposition::BoundedExperiment
    } else {
        TransferDisposition::Reject
    };
    Ok((
        TransferEvaluation {
            evaluation_id: evaluation_id.into(),
            transfer_id: assessment.transfer_id.clone(),
            baseline_metric,
            observed_metric,
            observed_effect: effect,
            negative_transfer,
            confidence,
            evidence_refs,
            receipt_ref,
            evaluated_at: now,
        },
        disposition,
    ))
}

pub fn validate_self_model(
    estimate: &SelfModelEstimate,
    now: DateTime<Utc>,
) -> Result<(), AdvancedPredictionError> {
    if estimate.estimate_id.trim().is_empty() {
        return Err(AdvancedPredictionError::MissingIdentity);
    }
    if estimate.task_ref.trim().is_empty()
        || estimate.domain_ref.trim().is_empty()
        || estimate.regime_ref.trim().is_empty()
    {
        return Err(AdvancedPredictionError::GlobalSelfModelProhibited);
    }
    if !(0.0..=1.0).contains(&estimate.capability_score)
        || !(0.0..=1.0).contains(&estimate.calibration_error)
        || !(0.0..=1.0).contains(&estimate.confidence)
        || !(0.0..=1.0).contains(&estimate.uncertainty)
        || !(0.0..=1.0).contains(&estimate.abstention_threshold)
        || estimate.sample_count == 0
        || estimate.version == 0
        || (estimate.version > 1 && estimate.supersedes_version != Some(estimate.version - 1))
    {
        return Err(AdvancedPredictionError::InvalidSelfModel);
    }
    if estimate.limitation_refs.is_empty()
        || estimate.error_mode_refs.is_empty()
        || estimate.evidence_refs.is_empty()
    {
        return Err(AdvancedPredictionError::MissingEvidence);
    }
    if estimate.receipt_ref.trim().is_empty() {
        return Err(AdvancedPredictionError::MissingReceipt);
    }
    if estimate.expires_at <= now || estimate.observed_at > now {
        return Err(AdvancedPredictionError::Expired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scenario_mass_and_causal_evidence_are_enforced() {
        let now = Utc::now();
        let mut scenario = ScenarioDefinition {
            scenario_id: "scenario".into(),
            baseline_ref: "baseline".into(),
            causal_status: ScenarioCausalStatus::Conditional,
            assumptions: vec!["stable regime".into()],
            branches: vec![
                ScenarioBranch {
                    branch_id: "a".into(),
                    condition: "x".into(),
                    probability: 0.4,
                    forecast_value: 1.0,
                    intervention_refs: vec![],
                    evidence_refs: vec!["evidence:a".into()],
                },
                ScenarioBranch {
                    branch_id: "b".into(),
                    condition: "not x".into(),
                    probability: 0.6,
                    forecast_value: 3.0,
                    intervention_refs: vec![],
                    evidence_refs: vec!["evidence:b".into()],
                },
            ],
            alternative_scenario_refs: vec![],
            disconfirming_evidence_refs: vec![],
            expires_at: now + chrono::Duration::days(1),
            evidence_refs: vec!["evidence:scenario".into()],
            receipt_ref: "receipt:scenario".into(),
        };
        assert!((project_scenario(&scenario, now).unwrap().expected_value - 2.2).abs() < 1e-12);
        scenario.branches[0].probability = 0.5;
        assert_eq!(
            project_scenario(&scenario, now),
            Err(AdvancedPredictionError::InvalidScenarioMass)
        );
    }
    #[test]
    fn negative_transfer_rolls_back_and_uncertain_transfer_stays_bounded() {
        let assessment = TransferAssessment {
            transfer_id: "transfer".into(),
            learning_ref: "learning".into(),
            source_context_ref: "source".into(),
            target_context_ref: "target".into(),
            similarity_score: 0.9,
            difference_refs: vec!["difference:regime".into()],
            expected_effect: 0.1,
            expected_benefit: 0.1,
            risk: 0.1,
            confidence: 0.9,
            uncertainty: 0.1,
            evaluation_plan_ref: "evaluation-plan:transfer".into(),
            exclusion_refs: vec![],
            failure_mode_refs: vec!["regime mismatch".into()],
            evidence_refs: vec!["evidence:transfer".into()],
            receipt_ref: "receipt:transfer".into(),
        };
        let (_, disposition) = evaluate_transfer(
            &assessment,
            "evaluation",
            1.0,
            0.8,
            0.9,
            vec!["evidence:outcome".into()],
            "receipt:evaluation",
            Utc::now(),
        )
        .unwrap();
        assert_eq!(disposition, TransferDisposition::Rollback);
    }
    #[test]
    fn global_or_unproven_self_model_is_prohibited() {
        let now = Utc::now();
        let estimate = SelfModelEstimate {
            estimate_id: "estimate".into(),
            task_ref: "".into(),
            domain_ref: "".into(),
            regime_ref: "".into(),
            capability_score: 0.8,
            calibration_error: 0.1,
            confidence: 0.9,
            uncertainty: 0.1,
            abstention_threshold: 0.6,
            sample_count: 10,
            version: 1,
            supersedes_version: None,
            limitation_refs: vec!["limit".into()],
            error_mode_refs: vec!["error-mode:unknown-domain".into()],
            evidence_refs: vec!["evidence".into()],
            receipt_ref: "receipt".into(),
            observed_at: now,
            expires_at: now + chrono::Duration::days(1),
        };
        assert_eq!(
            validate_self_model(&estimate, now),
            Err(AdvancedPredictionError::GlobalSelfModelProhibited)
        );
    }
}
