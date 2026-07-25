//! Existing Session Transfer, prediction, and metacog bindings for Silent Sessions.

use crate::prediction::PredictionValue;
use crate::silent_session::{SilentSessionId, SilentSessionRunId, WorkpointBinding};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const SILENT_SESSION_TRANSFER_PROJECTION_SCHEMA: &str =
    "focusa.silent_session_transfer_projection.v1";
pub const SILENT_SESSION_PREDICTION_BINDING_SCHEMA: &str =
    "focusa.silent_session_prediction_binding.v1";
pub const SILENT_SESSION_LEARNING_CANDIDATE_SCHEMA: &str =
    "focusa.silent_session_learning_candidate.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionTransferReason {
    Pause,
    OrphanRecovery,
    Handoff,
    ModelSwitch,
    ForegroundTakeover,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionTransferProjection {
    pub schema: String,
    pub projection_id: Uuid,
    pub existing_session_transfer_ref: String,
    pub reason: SessionTransferReason,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: WorkpointBinding,
    pub runtime_checkpoint_ref: String,
    pub event_cursor: String,
    pub writer_handoff_ref: Option<String>,
    pub operator_authorization_ref: Option<String>,
    pub source_model_ref: Option<String>,
    pub target_model_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub persist_via_existing_session_transfer: bool,
}

impl SilentSessionTransferProjection {
    pub fn validate(&self) -> Result<(), SilentSessionContinuationError> {
        if self.schema != SILENT_SESSION_TRANSFER_PROJECTION_SCHEMA
            || self.projection_id.get_version() != Some(uuid::Version::SortRand)
            || self.existing_session_transfer_ref.trim().is_empty()
            || !self.session_id.is_uuid_v7()
            || !self.run_id.is_uuid_v7()
            || self.generation == 0
            || self.project_identity_ref.trim().is_empty()
            || self.continuity_id.trim().is_empty()
            || self.workpoint_ref.workpoint_id.trim().is_empty()
            || self.runtime_checkpoint_ref.trim().is_empty()
            || self.event_cursor.trim().is_empty()
            || !self.persist_via_existing_session_transfer
        {
            return Err(SilentSessionContinuationError::InvalidTransfer);
        }
        match self.reason {
            SessionTransferReason::Handoff | SessionTransferReason::ForegroundTakeover => {
                if empty(&self.writer_handoff_ref) || empty(&self.operator_authorization_ref) {
                    return Err(SilentSessionContinuationError::WriterHandoffRequired);
                }
            }
            SessionTransferReason::ModelSwitch => {
                if empty(&self.source_model_ref)
                    || empty(&self.target_model_ref)
                    || self.source_model_ref == self.target_model_ref
                {
                    return Err(SilentSessionContinuationError::ModelSwitchBindingRequired);
                }
            }
            SessionTransferReason::Pause | SessionTransferReason::OrphanRecovery => {}
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn project_session_transfer(
    existing_session_transfer_ref: impl Into<String>,
    reason: SessionTransferReason,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: u64,
    project_identity_ref: impl Into<String>,
    continuity_id: impl Into<String>,
    workpoint_ref: WorkpointBinding,
    runtime_checkpoint_ref: impl Into<String>,
    event_cursor: impl Into<String>,
    writer_handoff_ref: Option<String>,
    operator_authorization_ref: Option<String>,
    source_model_ref: Option<String>,
    target_model_ref: Option<String>,
    created_at: DateTime<Utc>,
) -> Result<SilentSessionTransferProjection, SilentSessionContinuationError> {
    let projection = SilentSessionTransferProjection {
        schema: SILENT_SESSION_TRANSFER_PROJECTION_SCHEMA.into(),
        projection_id: Uuid::now_v7(),
        existing_session_transfer_ref: existing_session_transfer_ref.into(),
        reason,
        session_id,
        run_id,
        generation,
        project_identity_ref: project_identity_ref.into(),
        continuity_id: continuity_id.into(),
        workpoint_ref,
        runtime_checkpoint_ref: runtime_checkpoint_ref.into(),
        event_cursor: event_cursor.into(),
        writer_handoff_ref,
        operator_authorization_ref,
        source_model_ref,
        target_model_ref,
        created_at,
        persist_via_existing_session_transfer: true,
    };
    projection.validate()?;
    Ok(projection)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertainActionClass {
    ModelFallback,
    BroadRefactor,
    FlakyTestRepair,
    DependencyUpgrade,
    RiskyIntegration,
    RecoveryStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilentSessionPredictionBinding {
    pub schema: String,
    pub binding_id: Uuid,
    pub existing_prediction_ref: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub action_class: UncertainActionClass,
    pub prediction_recorded_event_seq: u64,
    pub planned_action_event_seq: u64,
    pub prediction: PredictionValue,
}

pub fn bind_prediction_before_action(
    existing_prediction_ref: impl Into<String>,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    action_class: UncertainActionClass,
    prediction_recorded_event_seq: u64,
    planned_action_event_seq: u64,
    prediction: PredictionValue,
) -> Result<SilentSessionPredictionBinding, SilentSessionContinuationError> {
    let binding = SilentSessionPredictionBinding {
        schema: SILENT_SESSION_PREDICTION_BINDING_SCHEMA.into(),
        binding_id: Uuid::now_v7(),
        existing_prediction_ref: existing_prediction_ref.into(),
        session_id,
        run_id,
        action_class,
        prediction_recorded_event_seq,
        planned_action_event_seq,
        prediction,
    };
    if binding.existing_prediction_ref.trim().is_empty()
        || !binding.session_id.is_uuid_v7()
        || !binding.run_id.is_uuid_v7()
        || binding.prediction_recorded_event_seq == 0
        || binding.planned_action_event_seq <= binding.prediction_recorded_event_seq
        || binding.prediction.prediction_type.trim().is_empty()
        || binding.prediction.predicted_outcome.trim().is_empty()
        || binding.prediction.recommended_action.trim().is_empty()
        || binding.prediction.why.trim().is_empty()
        || !(0.0..=1.0).contains(&binding.prediction.confidence)
        || binding.prediction.actual_outcome.is_some()
        || binding.prediction.evaluated_at.is_some()
    {
        return Err(SilentSessionContinuationError::PredictionNotBeforeAction);
    }
    Ok(binding)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionEvaluationBinding {
    pub existing_prediction_ref: String,
    pub action_event_seq: u64,
    pub evaluation_event_seq: u64,
    pub actual_outcome: String,
    pub score: f64,
    pub evaluation_evidence_refs: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
}

pub fn bind_prediction_evaluation(
    prediction: &SilentSessionPredictionBinding,
    action_event_seq: u64,
    evaluation_event_seq: u64,
    actual_outcome: impl Into<String>,
    score: f64,
    evaluation_evidence_refs: Vec<String>,
    evaluated_at: DateTime<Utc>,
) -> Result<PredictionEvaluationBinding, SilentSessionContinuationError> {
    let evaluation = PredictionEvaluationBinding {
        existing_prediction_ref: prediction.existing_prediction_ref.clone(),
        action_event_seq,
        evaluation_event_seq,
        actual_outcome: actual_outcome.into(),
        score,
        evaluation_evidence_refs,
        evaluated_at,
    };
    if action_event_seq != prediction.planned_action_event_seq
        || evaluation_event_seq <= action_event_seq
        || evaluation.actual_outcome.trim().is_empty()
        || !(0.0..=1.0).contains(&evaluation.score)
        || evaluation.evaluation_evidence_refs.is_empty()
        || evaluation
            .evaluation_evidence_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(SilentSessionContinuationError::InvalidPredictionEvaluation);
    }
    Ok(evaluation)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningOutcomeClass {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionLearningCandidate {
    pub schema: String,
    pub candidate_id: Uuid,
    pub existing_prediction_ref: String,
    pub outcome_class: LearningOutcomeClass,
    pub content: String,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub prediction_score: f64,
    pub captured_at: DateTime<Utc>,
    pub capture_via_existing_metacog: bool,
    pub advisory_only: bool,
    pub governance_authority: bool,
}

pub fn prepare_learning_candidate(
    evaluation: &PredictionEvaluationBinding,
    outcome_class: LearningOutcomeClass,
    content: impl Into<String>,
    rationale: impl Into<String>,
    additional_evidence_refs: Vec<String>,
    captured_at: DateTime<Utc>,
) -> Result<SilentSessionLearningCandidate, SilentSessionContinuationError> {
    let mut evidence_refs = evaluation.evaluation_evidence_refs.clone();
    evidence_refs.extend(additional_evidence_refs);
    evidence_refs.sort();
    evidence_refs.dedup();
    let candidate = SilentSessionLearningCandidate {
        schema: SILENT_SESSION_LEARNING_CANDIDATE_SCHEMA.into(),
        candidate_id: Uuid::now_v7(),
        existing_prediction_ref: evaluation.existing_prediction_ref.clone(),
        outcome_class,
        content: content.into(),
        rationale: rationale.into(),
        evidence_refs,
        prediction_score: evaluation.score,
        captured_at,
        capture_via_existing_metacog: true,
        advisory_only: true,
        governance_authority: false,
    };
    if candidate.content.trim().is_empty()
        || candidate.rationale.trim().is_empty()
        || candidate.evidence_refs.is_empty()
        || candidate
            .evidence_refs
            .iter()
            .any(|value| value.trim().is_empty())
        || !candidate.capture_via_existing_metacog
        || !candidate.advisory_only
        || candidate.governance_authority
    {
        return Err(SilentSessionContinuationError::InvalidLearningCandidate);
    }
    Ok(candidate)
}

fn empty(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|value| value.trim().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SilentSessionContinuationError {
    #[error("Session Transfer projection is invalid")]
    InvalidTransfer,
    #[error("handoff/takeover requires explicit writer handoff and operator authority")]
    WriterHandoffRequired,
    #[error("model switch requires distinct source and target model bindings")]
    ModelSwitchBindingRequired,
    #[error("prediction must be recorded before the uncertain action")]
    PredictionNotBeforeAction,
    #[error("prediction evaluation must follow the action and carry evidence")]
    InvalidPredictionEvaluation,
    #[error("metacog learning candidate must be evidence-backed and advisory")]
    InvalidLearningCandidate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction::PredictionOntologyContext;

    fn workpoint() -> WorkpointBinding {
        WorkpointBinding {
            workpoint_id: "workpoint:test".into(),
            revision: Some(5),
        }
    }

    #[test]
    fn foreground_takeover_requires_existing_transfer_writer_handoff_and_operator_authority() {
        let result = project_session_transfer(
            "transfer:existing",
            SessionTransferReason::ForegroundTakeover,
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            2,
            "project:focusa",
            "continuity:test",
            workpoint(),
            "runtime-checkpoint:1",
            "cursor:20",
            None,
            None,
            None,
            None,
            Utc::now(),
        );
        assert_eq!(
            result,
            Err(SilentSessionContinuationError::WriterHandoffRequired)
        );
    }

    #[test]
    fn model_switch_binds_distinct_models_and_existing_transfer() {
        let transfer = project_session_transfer(
            "transfer:existing",
            SessionTransferReason::ModelSwitch,
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            2,
            "project:focusa",
            "continuity:test",
            workpoint(),
            "runtime-checkpoint:1",
            "cursor:20",
            None,
            None,
            Some("model:one".into()),
            Some("model:two".into()),
            Utc::now(),
        )
        .unwrap();
        assert!(transfer.persist_via_existing_session_transfer);
    }

    fn prediction() -> PredictionValue {
        PredictionValue {
            prediction_type: "risky_integration".into(),
            context_refs: vec!["workpoint:test".into()],
            ontology_context: PredictionOntologyContext::default(),
            predicted_outcome: "integration will preserve operator work".into(),
            confidence: 0.7,
            recommended_action: "run governed integration preflight".into(),
            why: "isolated worktree and writer lease are present".into(),
            trajectory: None,
            actual_outcome: None,
            evaluated_at: None,
            score: None,
            learning_signal_ref: None,
            outcome_capture: None,
        }
    }

    #[test]
    fn prediction_precedes_action_evaluation_follows_and_learning_is_advisory() {
        let binding = bind_prediction_before_action(
            "prediction:existing",
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            UncertainActionClass::RiskyIntegration,
            10,
            11,
            prediction(),
        )
        .unwrap();
        let evaluation = bind_prediction_evaluation(
            &binding,
            11,
            12,
            "integration preserved operator work",
            1.0,
            vec!["evidence:integration".into()],
            Utc::now(),
        )
        .unwrap();
        let candidate = prepare_learning_candidate(
            &evaluation,
            LearningOutcomeClass::Completed,
            "isolated integration preserves dirty primary state",
            "verified by governed integration receipt",
            vec!["receipt:integration".into()],
            Utc::now(),
        )
        .unwrap();
        assert!(candidate.capture_via_existing_metacog);
        assert!(candidate.advisory_only);
        assert!(!candidate.governance_authority);
        assert_eq!(candidate.evidence_refs.len(), 2);
    }

    #[test]
    fn prediction_cannot_be_recorded_after_action_or_evaluated_without_evidence() {
        let late = bind_prediction_before_action(
            "prediction:late",
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            UncertainActionClass::RecoveryStrategy,
            12,
            11,
            prediction(),
        );
        assert_eq!(
            late.err(),
            Some(SilentSessionContinuationError::PredictionNotBeforeAction)
        );

        let binding = bind_prediction_before_action(
            "prediction:existing",
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            UncertainActionClass::RecoveryStrategy,
            10,
            11,
            prediction(),
        )
        .unwrap();
        assert_eq!(
            bind_prediction_evaluation(&binding, 11, 12, "outcome", 1.0, vec![], Utc::now()),
            Err(SilentSessionContinuationError::InvalidPredictionEvaluation)
        );
    }
}
