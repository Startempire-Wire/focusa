//! Spec138 metacognitive claim, promotion, outcome, and rollback authority.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalStatus {
    Descriptive,
    Correlational,
    Hypothesis,
    CausalSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectionClaim {
    pub claim_id: String,
    pub hypothesis: String,
    pub causal_status: CausalStatus,
    pub applicability_refs: Vec<String>,
    pub alternative_explanations: Vec<String>,
    pub disconfirming_evidence_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub review_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningPromotionPolicy {
    pub policy_id: String,
    pub version: u64,
    pub minimum_sample_count: usize,
    pub minimum_independent_sources: usize,
    pub minimum_holdout_improvement: f64,
    pub maximum_regression: f64,
    pub minimum_confidence: f64,
    pub allow_single_event_exception: bool,
    pub high_consequence_operator_approval_required: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningOutcomeEvaluation {
    pub evaluation_id: String,
    pub candidate_id: String,
    pub sample_count: usize,
    pub independent_source_count: usize,
    pub baseline_score: f64,
    pub candidate_score: f64,
    pub holdout_improvement: f64,
    pub worst_regression: f64,
    pub confidence: f64,
    pub negative_transfer_detected: bool,
    pub disconfirming_evidence_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionAuthority {
    pub authority_id: String,
    pub actor_ref: String,
    pub policy_ref: String,
    pub high_consequence: bool,
    pub operator_approved: bool,
    pub single_event_exception_approved: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionVerdict {
    Promote,
    Inhibit,
    Defer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionAssessment {
    pub assessment_id: String,
    pub candidate_id: String,
    pub verdict: PromotionVerdict,
    pub reason_codes: Vec<String>,
    pub policy_ref: String,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedLearning {
    pub application_id: String,
    pub learning_id: String,
    pub baseline_metric: f64,
    pub expected_improvement: f64,
    pub rollback_threshold: f64,
    pub applied_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostApplicationOutcome {
    pub outcome_id: String,
    pub application_id: String,
    pub observed_metric: f64,
    pub negative_transfer_detected: bool,
    pub observed_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningDisposition {
    Keep,
    Rollback,
    Supersede,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningSettlement {
    pub settlement_id: String,
    pub application_id: String,
    pub disposition: LearningDisposition,
    pub reason_codes: Vec<String>,
    pub rollback_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub settled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningAuthorityError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    InvalidReviewWindow,
    UnsupportedCausalClaim,
    InvalidPolicy,
    AuthorityMismatch,
    HighConsequenceApprovalRequired,
    SingleEventApprovalRequired,
    OutcomeMismatch,
}

pub fn validate_reflection_claim(claim: &ReflectionClaim) -> Result<(), LearningAuthorityError> {
    if claim.claim_id.trim().is_empty() || claim.hypothesis.trim().is_empty() {
        return Err(LearningAuthorityError::MissingIdentity);
    }
    if claim.evidence_refs.is_empty() || claim.applicability_refs.is_empty() {
        return Err(LearningAuthorityError::MissingEvidence);
    }
    if claim.receipt_ref.trim().is_empty() {
        return Err(LearningAuthorityError::MissingReceipt);
    }
    if claim.review_at <= claim.created_at || claim.expires_at <= claim.review_at {
        return Err(LearningAuthorityError::InvalidReviewWindow);
    }
    if matches!(claim.causal_status, CausalStatus::CausalSupported)
        && (claim.alternative_explanations.is_empty()
            || claim.disconfirming_evidence_refs.is_empty())
    {
        return Err(LearningAuthorityError::UnsupportedCausalClaim);
    }
    Ok(())
}

pub fn assess_learning_promotion(
    assessment_id: impl Into<String>,
    claim: &ReflectionClaim,
    evaluation: &LearningOutcomeEvaluation,
    policy: &LearningPromotionPolicy,
    authority: &PromotionAuthority,
    now: DateTime<Utc>,
) -> Result<PromotionAssessment, LearningAuthorityError> {
    validate_reflection_claim(claim)?;
    if policy.version == 0
        || policy.policy_id.trim().is_empty()
        || policy.evidence_refs.is_empty()
        || !(0.0..=1.0).contains(&policy.minimum_confidence)
    {
        return Err(LearningAuthorityError::InvalidPolicy);
    }
    if evaluation.candidate_id.trim().is_empty()
        || evaluation.evidence_refs.is_empty()
        || evaluation.receipt_ref.trim().is_empty()
    {
        return Err(LearningAuthorityError::MissingEvidence);
    }
    if authority.policy_ref != policy.policy_id
        || authority.evidence_refs.is_empty()
        || authority.receipt_ref.trim().is_empty()
    {
        return Err(LearningAuthorityError::AuthorityMismatch);
    }
    if authority.high_consequence
        && policy.high_consequence_operator_approval_required
        && !authority.operator_approved
    {
        return Err(LearningAuthorityError::HighConsequenceApprovalRequired);
    }
    if evaluation.sample_count == 1
        && (!policy.allow_single_event_exception
            || !authority.single_event_exception_approved
            || !authority.operator_approved)
    {
        return Err(LearningAuthorityError::SingleEventApprovalRequired);
    }
    let mut reasons = Vec::new();
    if evaluation.negative_transfer_detected {
        reasons.push("negative_transfer".into());
    }
    if evaluation.worst_regression > policy.maximum_regression {
        reasons.push("non_regression_gate_failed".into());
    }
    let inhibited = !reasons.is_empty();
    if evaluation.sample_count < policy.minimum_sample_count {
        reasons.push("sample_count_below_policy".into());
    }
    if evaluation.independent_source_count < policy.minimum_independent_sources {
        reasons.push("source_independence_below_policy".into());
    }
    if evaluation.holdout_improvement < policy.minimum_holdout_improvement {
        reasons.push("holdout_improvement_below_policy".into());
    }
    if evaluation.confidence < policy.minimum_confidence {
        reasons.push("confidence_below_policy".into());
    }
    if !evaluation.disconfirming_evidence_refs.is_empty() {
        reasons.push("unresolved_disconfirming_evidence".into());
    }
    let verdict = if inhibited {
        PromotionVerdict::Inhibit
    } else if reasons.is_empty() {
        PromotionVerdict::Promote
    } else {
        PromotionVerdict::Defer
    };
    let mut evidence_refs = claim.evidence_refs.clone();
    evidence_refs.extend(evaluation.evidence_refs.clone());
    evidence_refs.extend(authority.evidence_refs.clone());
    evidence_refs.sort();
    evidence_refs.dedup();
    Ok(PromotionAssessment {
        assessment_id: assessment_id.into(),
        candidate_id: evaluation.candidate_id.clone(),
        verdict,
        reason_codes: reasons,
        policy_ref: policy.policy_id.clone(),
        evidence_refs,
        receipt_ref: authority.receipt_ref.clone(),
        assessed_at: now,
    })
}

pub fn settle_learning_outcome(
    settlement_id: impl Into<String>,
    application: &AppliedLearning,
    outcome: &PostApplicationOutcome,
    now: DateTime<Utc>,
) -> Result<LearningSettlement, LearningAuthorityError> {
    if application.application_id != outcome.application_id {
        return Err(LearningAuthorityError::OutcomeMismatch);
    }
    if application.evidence_refs.is_empty()
        || outcome.evidence_refs.is_empty()
        || application.receipt_ref.trim().is_empty()
        || outcome.receipt_ref.trim().is_empty()
    {
        return Err(LearningAuthorityError::MissingEvidence);
    }
    let delta = outcome.observed_metric - application.baseline_metric;
    let rollback = outcome.negative_transfer_detected || delta < application.rollback_threshold;
    let expired = now >= application.expires_at;
    let disposition = if rollback {
        LearningDisposition::Rollback
    } else if expired {
        LearningDisposition::Supersede
    } else {
        LearningDisposition::Keep
    };
    let mut reasons = Vec::new();
    if outcome.negative_transfer_detected {
        reasons.push("negative_transfer".into());
    }
    if delta < application.rollback_threshold {
        reasons.push("rollback_threshold_crossed".into());
    }
    if expired && !rollback {
        reasons.push("learning_expired".into());
    }
    if reasons.is_empty() {
        reasons.push("observed_outcome_within_policy".into());
    }
    let mut evidence_refs = application.evidence_refs.clone();
    evidence_refs.extend(outcome.evidence_refs.clone());
    Ok(LearningSettlement {
        settlement_id: settlement_id.into(),
        application_id: application.application_id.clone(),
        disposition,
        reason_codes: reasons,
        rollback_ref: rollback.then(|| format!("rollback:{}", application.application_id)),
        evidence_refs,
        receipt_ref: outcome.receipt_ref.clone(),
        settled_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(now: DateTime<Utc>) -> ReflectionClaim {
        ReflectionClaim {
            claim_id: "claim".into(),
            hypothesis: "source diversity improves calibration".into(),
            causal_status: CausalStatus::Hypothesis,
            applicability_refs: vec!["cohort:release".into()],
            alternative_explanations: vec!["regime change".into()],
            disconfirming_evidence_refs: vec![],
            evidence_refs: vec!["evidence:reflection".into()],
            created_at: now,
            review_at: now + chrono::Duration::days(1),
            expires_at: now + chrono::Duration::days(30),
            receipt_ref: "receipt:reflection".into(),
        }
    }
    fn policy() -> LearningPromotionPolicy {
        LearningPromotionPolicy {
            policy_id: "policy:v1".into(),
            version: 1,
            minimum_sample_count: 10,
            minimum_independent_sources: 2,
            minimum_holdout_improvement: 0.05,
            maximum_regression: 0.01,
            minimum_confidence: 0.8,
            allow_single_event_exception: false,
            high_consequence_operator_approval_required: true,
            evidence_refs: vec!["evidence:policy".into()],
        }
    }
    fn evaluation() -> LearningOutcomeEvaluation {
        LearningOutcomeEvaluation {
            evaluation_id: "evaluation".into(),
            candidate_id: "candidate".into(),
            sample_count: 20,
            independent_source_count: 3,
            baseline_score: 0.3,
            candidate_score: 0.2,
            holdout_improvement: 0.1,
            worst_regression: 0.0,
            confidence: 0.9,
            negative_transfer_detected: false,
            disconfirming_evidence_refs: vec![],
            evidence_refs: vec!["evidence:evaluation".into()],
            evaluated_at: Utc::now(),
            receipt_ref: "receipt:evaluation".into(),
        }
    }
    fn authority() -> PromotionAuthority {
        PromotionAuthority {
            authority_id: "authority".into(),
            actor_ref: "operator".into(),
            policy_ref: "policy:v1".into(),
            high_consequence: true,
            operator_approved: true,
            single_event_exception_approved: false,
            evidence_refs: vec!["evidence:authority".into()],
            receipt_ref: "receipt:promotion".into(),
        }
    }

    #[test]
    fn promotion_requires_holdout_independence_and_authority() {
        let now = Utc::now();
        let assessment = assess_learning_promotion(
            "assessment",
            &claim(now),
            &evaluation(),
            &policy(),
            &authority(),
            now,
        )
        .unwrap();
        assert_eq!(assessment.verdict, PromotionVerdict::Promote);
        let mut weak = evaluation();
        weak.independent_source_count = 1;
        assert_eq!(
            assess_learning_promotion(
                "assessment",
                &claim(now),
                &weak,
                &policy(),
                &authority(),
                now
            )
            .unwrap()
            .verdict,
            PromotionVerdict::Defer
        );
    }

    #[test]
    fn single_event_and_high_consequence_promotion_fail_closed() {
        let now = Utc::now();
        let mut single = evaluation();
        single.sample_count = 1;
        let mut auth = authority();
        auth.operator_approved = false;
        assert_eq!(
            assess_learning_promotion("assessment", &claim(now), &single, &policy(), &auth, now),
            Err(LearningAuthorityError::HighConsequenceApprovalRequired)
        );
        auth.high_consequence = false;
        assert_eq!(
            assess_learning_promotion("assessment", &claim(now), &single, &policy(), &auth, now),
            Err(LearningAuthorityError::SingleEventApprovalRequired)
        );
    }

    #[test]
    fn unsupported_causal_reflection_and_negative_outcome_rollback() {
        let now = Utc::now();
        let mut causal = claim(now);
        causal.causal_status = CausalStatus::CausalSupported;
        causal.disconfirming_evidence_refs.clear();
        assert_eq!(
            validate_reflection_claim(&causal),
            Err(LearningAuthorityError::UnsupportedCausalClaim)
        );
        let application = AppliedLearning {
            application_id: "app".into(),
            learning_id: "learning".into(),
            baseline_metric: 0.5,
            expected_improvement: 0.1,
            rollback_threshold: 0.0,
            applied_at: now,
            expires_at: now + chrono::Duration::days(1),
            evidence_refs: vec!["evidence:application".into()],
            receipt_ref: "receipt:application".into(),
        };
        let outcome = PostApplicationOutcome {
            outcome_id: "outcome".into(),
            application_id: "app".into(),
            observed_metric: 0.4,
            negative_transfer_detected: true,
            observed_at: now,
            evidence_refs: vec!["evidence:outcome".into()],
            receipt_ref: "receipt:outcome".into(),
        };
        let settlement =
            settle_learning_outcome("settlement", &application, &outcome, now).unwrap();
        assert_eq!(settlement.disposition, LearningDisposition::Rollback);
        assert!(settlement.rollback_ref.is_some());
    }
}
