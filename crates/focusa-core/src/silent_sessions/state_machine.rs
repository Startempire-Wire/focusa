use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::{
    CompletionDecision, CompletionEvaluation, SemanticActivity, SilentSessionHealth,
    SilentSessionLifecycle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationProvenance {
    HarnessExplicit,
    RuntimeObserved,
    AdapterHeuristic,
    ModelInferred,
    TerminalInferred,
    VerificationConfirmed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ConfidenceBasisPoints(u16);

impl ConfidenceBasisPoints {
    pub const VERIFIED: Self = Self(10_000);
    pub const HIGH_CONFIDENCE: Self = Self(9_000);

    pub fn new(value: u16) -> Result<Self, StateReducerError> {
        if value > 10_000 {
            return Err(StateReducerError::ConfidenceOutOfRange(value));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateObservation<T> {
    pub value: T,
    pub source: String,
    pub provenance: ObservationProvenance,
    pub confidence: ConfidenceBasisPoints,
    pub observed_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
}

impl<T> StateObservation<T> {
    pub fn is_fresh_at(&self, at: DateTime<Utc>) -> bool {
        self.observed_at <= at && at <= self.fresh_until
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedBlocker {
    pub blocker_type: String,
    pub reason: String,
    pub recovery_tools: Vec<String>,
}

#[derive(Debug, Default)]
pub struct TransitionEvidence<'a> {
    pub waiting_input: Option<&'a StateObservation<bool>>,
    pub blocker: Option<&'a TypedBlocker>,
    pub completion: Option<&'a CompletionEvaluation>,
    pub evaluated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleTransition {
    pub from: SilentSessionLifecycle,
    pub to: SilentSessionLifecycle,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateReducerError {
    #[error("invalid lifecycle transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: SilentSessionLifecycle,
        to: SilentSessionLifecycle,
    },
    #[error(
        "waiting_input requires fresh explicit harness evidence or a high-confidence adapter observation"
    )]
    WaitingInputUnproven,
    #[error("blocked requires a typed blocker with a reason")]
    BlockerUnproven,
    #[error("completed requires a complete evaluation with receipt readiness")]
    CompletionUnproven,
    #[error("confidence basis points out of range: {0}")]
    ConfidenceOutOfRange(u16),
}

pub fn reduce_lifecycle(
    current: SilentSessionLifecycle,
    target: SilentSessionLifecycle,
    evidence: &TransitionEvidence<'_>,
) -> Result<LifecycleTransition, StateReducerError> {
    if !transition_allowed(current, target) {
        return Err(StateReducerError::InvalidTransition {
            from: current,
            to: target,
        });
    }

    match target {
        SilentSessionLifecycle::WaitingInput if !waiting_input_is_proven(evidence) => {
            return Err(StateReducerError::WaitingInputUnproven);
        }
        SilentSessionLifecycle::Blocked if !blocker_is_proven(evidence) => {
            return Err(StateReducerError::BlockerUnproven);
        }
        SilentSessionLifecycle::Completed if !completion_is_proven(evidence) => {
            return Err(StateReducerError::CompletionUnproven);
        }
        _ => {}
    }

    Ok(LifecycleTransition {
        from: current,
        to: target,
    })
}

pub fn reduce_health(
    current: SilentSessionHealth,
    observation: &StateObservation<SilentSessionHealth>,
    evaluated_at: DateTime<Utc>,
) -> SilentSessionHealth {
    if observation.is_fresh_at(evaluated_at) {
        observation.value
    } else {
        current
    }
}

pub fn reduce_activity(
    current: SemanticActivity,
    observation: &StateObservation<SemanticActivity>,
    evaluated_at: DateTime<Utc>,
) -> SemanticActivity {
    if observation.is_fresh_at(evaluated_at) {
        observation.value
    } else {
        current
    }
}

fn waiting_input_is_proven(evidence: &TransitionEvidence<'_>) -> bool {
    let Some(observation) = evidence.waiting_input else {
        return false;
    };
    let Some(evaluated_at) = evidence.evaluated_at else {
        return false;
    };
    if !observation.value || !observation.is_fresh_at(evaluated_at) {
        return false;
    }
    match observation.provenance {
        ObservationProvenance::HarnessExplicit | ObservationProvenance::VerificationConfirmed => {
            true
        }
        ObservationProvenance::AdapterHeuristic => {
            observation.confidence >= ConfidenceBasisPoints::HIGH_CONFIDENCE
        }
        ObservationProvenance::RuntimeObserved
        | ObservationProvenance::ModelInferred
        | ObservationProvenance::TerminalInferred => false,
    }
}

fn blocker_is_proven(evidence: &TransitionEvidence<'_>) -> bool {
    evidence.blocker.is_some_and(|blocker| {
        !blocker.blocker_type.trim().is_empty() && !blocker.reason.trim().is_empty()
    })
}

fn completion_is_proven(evidence: &TransitionEvidence<'_>) -> bool {
    evidence.completion.is_some_and(|evaluation| {
        evaluation.decision == CompletionDecision::Complete && evaluation.receipt_ready
    })
}

fn transition_allowed(from: SilentSessionLifecycle, to: SilentSessionLifecycle) -> bool {
    use SilentSessionLifecycle as State;
    matches!(
        (from, to),
        (State::Draft, State::Validating)
            | (State::Validating, State::Queued)
            | (State::Queued, State::Launching)
            | (State::Launching, State::Initializing)
            | (State::Initializing, State::Running)
            | (State::Running, State::WaitingInput)
            | (State::Running, State::Blocked)
            | (State::Running, State::Pausing)
            | (State::Running, State::Completing)
            | (State::Running, State::Recovering)
            | (State::Running, State::Orphaned)
            | (State::Running, State::Cancelling)
            | (State::WaitingInput, State::Running)
            | (State::WaitingInput, State::Paused)
            | (State::WaitingInput, State::Cancelling)
            | (State::Blocked, State::Running)
            | (State::Blocked, State::Paused)
            | (State::Blocked, State::Completing)
            | (State::Blocked, State::Cancelling)
            | (State::Pausing, State::Paused)
            | (State::Paused, State::Resuming)
            | (State::Paused, State::Cancelling)
            | (State::Resuming, State::Running)
            | (State::Resuming, State::Blocked)
            | (State::Recovering, State::Running)
            | (State::Recovering, State::WaitingInput)
            | (State::Recovering, State::Blocked)
            | (State::Recovering, State::Orphaned)
            | (State::Recovering, State::Failed)
            | (State::Orphaned, State::Recovering)
            | (State::Orphaned, State::Cancelled)
            | (State::Orphaned, State::Failed)
            | (State::Cancelling, State::Cancelled)
            | (State::Completing, State::Completed)
            | (State::Completing, State::Blocked)
            | (State::Completing, State::Failed)
    )
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;
    use crate::silent_sessions::{
        ActorInstanceId, CompletionEvaluationId, SilentSessionId, SilentSessionRunId,
    };

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 17, 13, 0, 0).unwrap()
    }

    fn waiting_observation(
        provenance: ObservationProvenance,
        confidence: u16,
    ) -> StateObservation<bool> {
        StateObservation {
            value: true,
            source: "adapter:proof".into(),
            provenance,
            confidence: ConfidenceBasisPoints::new(confidence).unwrap(),
            observed_at: now(),
            fresh_until: now() + Duration::seconds(30),
        }
    }

    fn completion(receipt_ready: bool) -> CompletionEvaluation {
        CompletionEvaluation {
            schema_version: 1,
            id: CompletionEvaluationId::new(),
            silent_session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            decision: CompletionDecision::Complete,
            reason: "all evidence verified".into(),
            required_evidence_refs: vec!["test:proof".into()],
            verified_evidence_refs: vec!["test:proof".into()],
            receipt_ready,
            evaluated_by: ActorInstanceId::new(),
            evaluated_at: now(),
        }
    }

    #[test]
    fn exact_happy_path_transitions_are_allowed() {
        use SilentSessionLifecycle as State;
        let path = [
            State::Draft,
            State::Validating,
            State::Queued,
            State::Launching,
            State::Initializing,
            State::Running,
        ];
        for pair in path.windows(2) {
            assert!(reduce_lifecycle(pair[0], pair[1], &TransitionEvidence::default()).is_ok());
        }
    }

    #[test]
    fn invalid_shortcuts_are_rejected() {
        assert!(matches!(
            reduce_lifecycle(
                SilentSessionLifecycle::Draft,
                SilentSessionLifecycle::Running,
                &TransitionEvidence::default()
            ),
            Err(StateReducerError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn silence_or_model_inference_cannot_prove_waiting_input() {
        let observation = waiting_observation(ObservationProvenance::ModelInferred, 10_000);
        let evidence = TransitionEvidence {
            waiting_input: Some(&observation),
            evaluated_at: Some(now()),
            ..TransitionEvidence::default()
        };
        assert_eq!(
            reduce_lifecycle(
                SilentSessionLifecycle::Running,
                SilentSessionLifecycle::WaitingInput,
                &evidence
            ),
            Err(StateReducerError::WaitingInputUnproven)
        );
    }

    #[test]
    fn fresh_explicit_or_high_confidence_adapter_can_prove_waiting_input() {
        for observation in [
            waiting_observation(ObservationProvenance::HarnessExplicit, 5_000),
            waiting_observation(ObservationProvenance::AdapterHeuristic, 9_000),
        ] {
            let evidence = TransitionEvidence {
                waiting_input: Some(&observation),
                evaluated_at: Some(now()),
                ..TransitionEvidence::default()
            };
            assert!(
                reduce_lifecycle(
                    SilentSessionLifecycle::Running,
                    SilentSessionLifecycle::WaitingInput,
                    &evidence
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn blocked_requires_typed_reason() {
        let blocker = TypedBlocker {
            blocker_type: "dependency".into(),
            reason: "database unavailable".into(),
            recovery_tools: vec!["focusa doctor".into()],
        };
        let evidence = TransitionEvidence {
            blocker: Some(&blocker),
            ..TransitionEvidence::default()
        };
        assert!(
            reduce_lifecycle(
                SilentSessionLifecycle::Running,
                SilentSessionLifecycle::Blocked,
                &evidence
            )
            .is_ok()
        );
    }

    #[test]
    fn completion_requires_evaluation_and_receipt_readiness() {
        let not_ready = completion(false);
        let blocked = TransitionEvidence {
            completion: Some(&not_ready),
            ..TransitionEvidence::default()
        };
        assert_eq!(
            reduce_lifecycle(
                SilentSessionLifecycle::Completing,
                SilentSessionLifecycle::Completed,
                &blocked
            ),
            Err(StateReducerError::CompletionUnproven)
        );
        let ready = completion(true);
        let proven = TransitionEvidence {
            completion: Some(&ready),
            ..TransitionEvidence::default()
        };
        assert!(
            reduce_lifecycle(
                SilentSessionLifecycle::Completing,
                SilentSessionLifecycle::Completed,
                &proven
            )
            .is_ok()
        );
    }

    #[test]
    fn process_exit_health_does_not_change_lifecycle() {
        let observation = StateObservation {
            value: SilentSessionHealth::ProcessExited,
            source: "runner".into(),
            provenance: ObservationProvenance::RuntimeObserved,
            confidence: ConfidenceBasisPoints::VERIFIED,
            observed_at: now(),
            fresh_until: now() + Duration::seconds(30),
        };
        assert_eq!(
            reduce_health(SilentSessionHealth::Healthy, &observation, now()),
            SilentSessionHealth::ProcessExited
        );
        assert!(
            reduce_lifecycle(
                SilentSessionLifecycle::Running,
                SilentSessionLifecycle::Completed,
                &TransitionEvidence::default()
            )
            .is_err()
        );
    }

    #[test]
    fn stale_activity_observation_is_ignored() {
        let observation = StateObservation {
            value: SemanticActivity::WaitingForOperator,
            source: "adapter".into(),
            provenance: ObservationProvenance::AdapterHeuristic,
            confidence: ConfidenceBasisPoints::HIGH_CONFIDENCE,
            observed_at: now() - Duration::seconds(60),
            fresh_until: now() - Duration::seconds(30),
        };
        assert_eq!(
            reduce_activity(SemanticActivity::Working, &observation, now()),
            SemanticActivity::Working
        );
    }
}
