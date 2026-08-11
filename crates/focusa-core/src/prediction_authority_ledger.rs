//! Append-only scoped Spec 138 authority ledger and projection.

use crate::{
    epistemic_fusion::FusionResult,
    epistemic_memory_lifecycle::MemoryLifecycleEvent,
    epistemic_primitives::EpistemicPrimitiveRecord,
    epistemic_security::SourceSecurityDecision,
    metacognitive_learning::{
        ActionDeltaPattern, LearningSettlement, PromotionAssessment, ReflectionClaim,
    },
    outcome_resolution::{ActionOutcomeObservation, OutcomeAuthorityEvent},
    prediction_advanced::{ScenarioProjection, SelfModelEstimate, TransferEvaluation},
    prediction_authority::*,
    prediction_migration::LegacyMigrationRecord,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PredictionAuthorityProjection {
    pub sequence: u64,
    pub primitives: BTreeMap<String, EpistemicPrimitiveRecord>,
    pub reflection_claims: BTreeMap<String, ReflectionClaim>,
    pub promotion_assessments: BTreeMap<String, PromotionAssessment>,
    pub learning_settlements: BTreeMap<String, LearningSettlement>,
    pub outcome_authority_events: BTreeMap<String, OutcomeAuthorityEvent>,
    pub fusion_results: BTreeMap<String, FusionResult>,
    pub scenarios: BTreeMap<String, ScenarioProjection>,
    pub transfer_evaluations: BTreeMap<String, TransferEvaluation>,
    pub self_model: BTreeMap<String, SelfModelEstimate>,
    pub memory_lifecycle: BTreeMap<String, MemoryLifecycleEvent>,
    pub source_security_decisions: BTreeMap<String, SourceSecurityDecision>,
    pub legacy_migrations: BTreeMap<String, LegacyMigrationRecord>,
    pub questions: BTreeMap<String, PredictionQuestion>,
    pub commitments: BTreeMap<String, PredictionCommitment>,
    pub action_commitments: BTreeMap<String, ActionPredictionCommitment>,
    pub action_outcomes: BTreeMap<String, ActionOutcomeObservation>,
    pub action_patterns: BTreeMap<String, ActionDeltaPattern>,
    pub resolutions: BTreeMap<String, OutcomeResolution>,
    pub outcome_claims: BTreeMap<String, OutcomeClaim>,
    pub scoring_policies: BTreeMap<String, ScoringPolicy>,
    pub evaluations: BTreeMap<String, PredictionEvaluation>,
    pub learning_candidates: BTreeMap<String, LearningCandidate>,
    pub promotion_decisions: BTreeMap<String, PromotionDecision>,
    pub learning: BTreeMap<String, LearningRecord>,
    pub transfer_predictions: BTreeMap<String, TransferPrediction>,
    pub transfers: BTreeMap<String, TransferOutcome>,
}

#[derive(Debug, Clone, Default)]
pub struct PredictionAuthorityLedger {
    events: Vec<ScopedAuthorityEvent>,
    event_ids: BTreeSet<String>,
}

impl PredictionAuthorityLedger {
    pub fn append(&mut self, event: ScopedAuthorityEvent) -> Result<(), String> {
        crate::prediction_authority_validation::validate_scoped_authority_event(&event)?;
        if self.event_ids.contains(&event.event_id) {
            return Err("append-only event id already exists".into());
        }
        let expected = self.events.last().map_or(1, |prior| prior.sequence + 1);
        if event.sequence != expected {
            return Err(format!("event sequence gap: expected {expected}"));
        }
        self.event_ids.insert(event.event_id.clone());
        self.events.push(event);
        Ok(())
    }

    pub fn events(&self) -> &[ScopedAuthorityEvent] {
        &self.events
    }

    pub fn project(&self, scope: &EpistemicScope) -> PredictionAuthorityProjection {
        let mut projection = PredictionAuthorityProjection::default();
        for envelope in self.events.iter().filter(|event| &event.scope == scope) {
            projection.sequence = envelope.sequence;
            match &envelope.event {
                PredictionAuthorityEvent::EpistemicPrimitive(value) => {
                    projection
                        .primitives
                        .insert(value.primitive_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ReflectionClaim(value) => {
                    projection
                        .reflection_claims
                        .insert(value.claim_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::PromotionAssessment(value) => {
                    projection
                        .promotion_assessments
                        .insert(value.assessment_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::LearningSettlement(value) => {
                    projection
                        .learning_settlements
                        .insert(value.settlement_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::OutcomeAuthority(value) => {
                    projection
                        .outcome_authority_events
                        .insert(value.event_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::FusionResult(value) => {
                    projection
                        .fusion_results
                        .insert(value.fusion_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ScenarioProjection(value) => {
                    projection
                        .scenarios
                        .insert(value.scenario_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::TransferEvaluation(value) => {
                    projection
                        .transfer_evaluations
                        .insert(value.evaluation_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::SelfModelEstimate(value) => {
                    projection
                        .self_model
                        .insert(value.estimate_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::MemoryLifecycle(value) => {
                    projection
                        .memory_lifecycle
                        .insert(value.event_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::SourceSecurityDecision(value) => {
                    projection
                        .source_security_decisions
                        .insert(value.decision_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::LegacyMigration(value) => {
                    projection
                        .legacy_migrations
                        .insert(value.migration_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::Question(value) => {
                    projection
                        .questions
                        .insert(value.question_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::Commitment(value) => {
                    projection
                        .commitments
                        .insert(value.commitment_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ActionCommitment(value) => {
                    projection
                        .action_commitments
                        .insert(value.action_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ActionOutcome(value) => {
                    projection
                        .action_outcomes
                        .insert(value.action_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ActionPattern(value) => {
                    projection
                        .action_patterns
                        .insert(value.pattern_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::OutcomeClaim(value) => {
                    projection
                        .outcome_claims
                        .insert(value.claim_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::OutcomeResolution(value) => {
                    projection
                        .resolutions
                        .insert(value.resolution_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::ScoringPolicy(value) => {
                    projection
                        .scoring_policies
                        .insert(value.policy_ref(), value.clone());
                }
                PredictionAuthorityEvent::Evaluation(value) => {
                    projection
                        .evaluations
                        .insert(value.evaluation_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::LearningCandidate(value) => {
                    projection
                        .learning_candidates
                        .insert(value.candidate_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::PromotionDecision(value) => {
                    projection
                        .promotion_decisions
                        .insert(value.decision_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::LearningRecord(value) => {
                    projection
                        .learning
                        .insert(value.learning_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::TransferPrediction(value) => {
                    projection
                        .transfer_predictions
                        .insert(value.transfer_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::TransferOutcome(value) => {
                    projection
                        .transfers
                        .insert(value.outcome_id.clone(), value.clone());
                }
            }
        }
        projection
    }

    pub fn recover(jsonl: &str) -> Result<Self, String> {
        let mut ledger = Self::default();
        for line in jsonl.lines().filter(|line| !line.trim().is_empty()) {
            let event = serde_json::from_str(line).map_err(|error| error.to_string())?;
            ledger.append(event)?;
        }
        Ok(ledger)
    }
}
