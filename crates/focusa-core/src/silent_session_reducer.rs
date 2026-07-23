//! Pure Spec 133 Silent Session reducer.
//! This module accepts facts and returns state; it owns no I/O, process, timer, retry, or workspace behavior.

use crate::silent_session::{
    CompletionDecision, ObservationProvenance, SemanticObservation, SilentSession,
    SilentSessionCompletionEvaluationId, SilentSessionHealth, SilentSessionLifecycleState,
    SilentSessionSemanticActivity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedSessionBlocker {
    pub class: String,
    pub reason_code: String,
    pub summary: String,
    pub evidence_refs: Vec<String>,
}

impl TypedSessionBlocker {
    fn is_valid(&self) -> bool {
        !self.class.trim().is_empty()
            && !self.reason_code.trim().is_empty()
            && !self.summary.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "evidence_kind", rename_all = "snake_case")]
pub enum LifecycleTransitionEvidence {
    CanonicalFact {
        event_ref: String,
        reason_code: String,
    },
    ExplicitInputRequest {
        event_ref: String,
        source: String,
        confidence: f64,
        observed_at: DateTime<Utc>,
        fresh_until: DateTime<Utc>,
        provenance: ObservationProvenance,
    },
    TypedBlocker {
        blocker: TypedSessionBlocker,
    },
    CompletionEvaluation {
        evaluation_id: SilentSessionCompletionEvaluationId,
        decision: CompletionDecision,
        receipt_ready: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "fact_kind", rename_all = "snake_case")]
pub enum SilentSessionFact {
    LifecycleTransition {
        target: SilentSessionLifecycleState,
        evidence: LifecycleTransitionEvidence,
    },
    ControlIssued {
        control: String,
        event_ref: String,
        reason_code: String,
    },
    HealthObserved {
        health: SilentSessionHealth,
        source: String,
        observed_at: DateTime<Utc>,
        provenance: ObservationProvenance,
    },
    SemanticActivityObserved {
        observation: SemanticObservation,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionReducerState {
    pub session: SilentSession,
    pub active_blocker: Option<TypedSessionBlocker>,
    pub completion_evaluation_id: Option<SilentSessionCompletionEvaluationId>,
    pub last_health_source: Option<String>,
    pub last_health_observed_at: Option<DateTime<Utc>>,
    pub last_health_provenance: Option<ObservationProvenance>,
}

impl SilentSessionReducerState {
    pub fn new(session: SilentSession) -> Self {
        Self {
            session,
            active_blocker: None,
            completion_evaluation_id: None,
            last_health_source: None,
            last_health_observed_at: None,
            last_health_provenance: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SilentSessionReduceError {
    InvalidTransition {
        from: SilentSessionLifecycleState,
        to: SilentSessionLifecycleState,
    },
    MissingCanonicalEvidence,
    WaitingInputRequiresExplicitFreshObservation,
    BlockedRequiresTypedBlocker,
    CompletedRequiresReceiptReadyEvaluation,
    InvalidControlFact,
    InvalidHealthObservation,
    InvalidSemanticObservation,
}

pub fn lifecycle_transition_allowed(
    from: &SilentSessionLifecycleState,
    to: &SilentSessionLifecycleState,
) -> bool {
    use SilentSessionLifecycleState as S;
    matches!(
        (from, to),
        (S::Draft, S::Validating)
            | (S::Validating, S::Queued)
            | (S::Queued, S::Launching)
            | (S::Launching, S::Initializing)
            | (S::Initializing, S::Running)
            | (S::Running, S::WaitingInput)
            | (S::Running, S::Blocked)
            | (S::Running, S::Pausing)
            | (S::Running, S::Completing)
            | (S::Running, S::Recovering)
            | (S::Running, S::Orphaned)
            | (S::Running, S::Cancelling)
            | (S::WaitingInput, S::Running)
            | (S::WaitingInput, S::Paused)
            | (S::WaitingInput, S::Cancelling)
            | (S::Blocked, S::Running)
            | (S::Blocked, S::Paused)
            | (S::Blocked, S::Completing)
            | (S::Blocked, S::Cancelling)
            | (S::Pausing, S::Paused)
            | (S::Paused, S::Resuming)
            | (S::Paused, S::Cancelling)
            | (S::Resuming, S::Running)
            | (S::Resuming, S::Blocked)
            | (S::Recovering, S::Running)
            | (S::Recovering, S::WaitingInput)
            | (S::Recovering, S::Blocked)
            | (S::Recovering, S::Orphaned)
            | (S::Recovering, S::Failed)
            | (S::Orphaned, S::Recovering)
            | (S::Orphaned, S::Cancelled)
            | (S::Orphaned, S::Failed)
            | (S::Cancelling, S::Cancelled)
            | (S::Completing, S::Completed)
            | (S::Completing, S::Blocked)
            | (S::Completing, S::Failed)
    )
}

pub fn reduce_silent_session(
    state: &SilentSessionReducerState,
    fact: &SilentSessionFact,
) -> Result<SilentSessionReducerState, SilentSessionReduceError> {
    let mut next = state.clone();
    match fact {
        SilentSessionFact::LifecycleTransition { target, evidence } => {
            let from = &state.session.lifecycle_state;
            if !lifecycle_transition_allowed(from, target) {
                return Err(SilentSessionReduceError::InvalidTransition {
                    from: from.clone(),
                    to: target.clone(),
                });
            }
            validate_transition_evidence(target, evidence)?;
            next.session.lifecycle_state = target.clone();
            match evidence {
                LifecycleTransitionEvidence::TypedBlocker { blocker } => {
                    next.active_blocker = Some(blocker.clone());
                }
                LifecycleTransitionEvidence::CompletionEvaluation { evaluation_id, .. } => {
                    next.completion_evaluation_id = Some(*evaluation_id);
                    next.active_blocker = None;
                }
                _ if !matches!(target, SilentSessionLifecycleState::Blocked) => {
                    next.active_blocker = None;
                }
                _ => {}
            }
        }
        SilentSessionFact::ControlIssued {
            control,
            event_ref,
            reason_code,
        } => {
            if control.trim().is_empty()
                || event_ref.trim().is_empty()
                || reason_code.trim().is_empty()
            {
                return Err(SilentSessionReduceError::InvalidControlFact);
            }
        }
        SilentSessionFact::HealthObserved {
            health,
            source,
            observed_at,
            provenance,
        } => {
            if source.trim().is_empty() {
                return Err(SilentSessionReduceError::InvalidHealthObservation);
            }
            next.session.health = health.clone();
            next.last_health_source = Some(source.clone());
            next.last_health_observed_at = Some(*observed_at);
            next.last_health_provenance = Some(provenance.clone());
        }
        SilentSessionFact::SemanticActivityObserved { observation } => {
            if observation.source.trim().is_empty()
                || !(0.0..=1.0).contains(&observation.confidence)
                || observation.fresh_until < observation.observed_at
            {
                return Err(SilentSessionReduceError::InvalidSemanticObservation);
            }
            next.session.semantic_observation = Some(observation.clone());
        }
    }
    Ok(next)
}

fn validate_transition_evidence(
    target: &SilentSessionLifecycleState,
    evidence: &LifecycleTransitionEvidence,
) -> Result<(), SilentSessionReduceError> {
    match target {
        SilentSessionLifecycleState::WaitingInput => match evidence {
            LifecycleTransitionEvidence::ExplicitInputRequest {
                event_ref,
                source,
                confidence,
                observed_at,
                fresh_until,
                provenance,
            } if !event_ref.trim().is_empty()
                && !source.trim().is_empty()
                && (0.8..=1.0).contains(confidence)
                && fresh_until >= observed_at
                && matches!(
                    provenance,
                    ObservationProvenance::RuntimeObserved
                        | ObservationProvenance::VerificationConfirmed
                        | ObservationProvenance::ModelInferred
                ) =>
            {
                Ok(())
            }
            _ => Err(SilentSessionReduceError::WaitingInputRequiresExplicitFreshObservation),
        },
        SilentSessionLifecycleState::Blocked => match evidence {
            LifecycleTransitionEvidence::TypedBlocker { blocker } if blocker.is_valid() => Ok(()),
            _ => Err(SilentSessionReduceError::BlockedRequiresTypedBlocker),
        },
        SilentSessionLifecycleState::Completed => match evidence {
            LifecycleTransitionEvidence::CompletionEvaluation {
                decision: CompletionDecision::Completed,
                receipt_ready: true,
                ..
            } => Ok(()),
            _ => Err(SilentSessionReduceError::CompletedRequiresReceiptReadyEvaluation),
        },
        _ => match evidence {
            LifecycleTransitionEvidence::CanonicalFact {
                event_ref,
                reason_code,
            } if !event_ref.trim().is_empty() && !reason_code.trim().is_empty() => Ok(()),
            LifecycleTransitionEvidence::TypedBlocker { blocker } if blocker.is_valid() => Ok(()),
            LifecycleTransitionEvidence::CompletionEvaluation { .. } => Ok(()),
            _ => Err(SilentSessionReduceError::MissingCanonicalEvidence),
        },
    }
}

pub fn semantic_activity_is_fresh(
    observation: Option<&SemanticObservation>,
    now: DateTime<Utc>,
) -> Option<SilentSessionSemanticActivity> {
    observation
        .filter(|observation| observation.fresh_until >= now)
        .map(|observation| observation.activity.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::{
        SILENT_SESSION_SCHEMA, SilentSessionConfigRevisionId, SilentSessionId,
        SilentSessionVersions,
    };
    use std::path::PathBuf;

    fn state_at(lifecycle_state: SilentSessionLifecycleState) -> SilentSessionReducerState {
        SilentSessionReducerState::new(SilentSession {
            schema: SILENT_SESSION_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            session_id: SilentSessionId::new(),
            display_name: "worker".into(),
            created_at: Utc::now(),
            created_by_actor_ref: "actor:pi".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "test".into(),
            project_root: PathBuf::from("/tmp/focusa"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "workloop-completion".into(),
            trajectory_ref: None,
            workpoint_ref: None,
            work_item_ref: None,
            operator_ask: crate::silent_session::OperatorAskBinding::capture(
                "ask:reducer-test",
                "test reducer",
                1,
                Utc::now(),
            ),
            mission: "test reducer".into(),
            lifecycle_state,
            health: SilentSessionHealth::Unknown,
            semantic_observation: None,
            active_run_id: None,
            config_revision_id: SilentSessionConfigRevisionId::new(),
            writer_lease_ref: None,
            retention_policy_ref: "retention:test".into(),
            receipt_refs: vec![],
        })
    }

    fn canonical_fact(target: SilentSessionLifecycleState) -> SilentSessionFact {
        SilentSessionFact::LifecycleTransition {
            target,
            evidence: LifecycleTransitionEvidence::CanonicalFact {
                event_ref: "event:test".into(),
                reason_code: "verified_fact".into(),
            },
        }
    }

    #[test]
    fn exact_state_machine_accepts_every_normative_edge() {
        use SilentSessionLifecycleState as S;
        let edges = [
            (S::Draft, S::Validating),
            (S::Validating, S::Queued),
            (S::Queued, S::Launching),
            (S::Launching, S::Initializing),
            (S::Initializing, S::Running),
            (S::Running, S::Pausing),
            (S::Running, S::Completing),
            (S::Running, S::Recovering),
            (S::Running, S::Orphaned),
            (S::Running, S::Cancelling),
            (S::WaitingInput, S::Running),
            (S::WaitingInput, S::Paused),
            (S::WaitingInput, S::Cancelling),
            (S::Blocked, S::Running),
            (S::Blocked, S::Paused),
            (S::Blocked, S::Completing),
            (S::Blocked, S::Cancelling),
            (S::Pausing, S::Paused),
            (S::Paused, S::Resuming),
            (S::Paused, S::Cancelling),
            (S::Resuming, S::Running),
            (S::Recovering, S::Running),
            (S::Recovering, S::Orphaned),
            (S::Recovering, S::Failed),
            (S::Orphaned, S::Recovering),
            (S::Orphaned, S::Cancelled),
            (S::Orphaned, S::Failed),
            (S::Cancelling, S::Cancelled),
            (S::Completing, S::Failed),
        ];
        for (from, to) in edges {
            assert!(
                lifecycle_transition_allowed(&from, &to),
                "{from:?} -> {to:?}"
            );
            assert!(reduce_silent_session(&state_at(from), &canonical_fact(to)).is_ok());
        }
        for (from, to) in [
            (S::Running, S::WaitingInput),
            (S::Recovering, S::WaitingInput),
            (S::Running, S::Blocked),
            (S::Resuming, S::Blocked),
            (S::Recovering, S::Blocked),
            (S::Completing, S::Blocked),
            (S::Completing, S::Completed),
        ] {
            assert!(
                lifecycle_transition_allowed(&from, &to),
                "{from:?} -> {to:?}"
            );
        }
    }

    #[test]
    fn non_normative_edges_fail_closed() {
        use SilentSessionLifecycleState as S;
        for (from, to) in [
            (S::Draft, S::Running),
            (S::Running, S::Completed),
            (S::Completed, S::Running),
        ] {
            assert!(matches!(
                reduce_silent_session(&state_at(from.clone()), &canonical_fact(to.clone())),
                Err(SilentSessionReduceError::InvalidTransition { from: actual_from, to: actual_to })
                    if actual_from == from && actual_to == to
            ));
        }
    }

    #[test]
    fn waiting_input_requires_explicit_fresh_high_confidence_fact() {
        let state = state_at(SilentSessionLifecycleState::Running);
        assert_eq!(
            reduce_silent_session(
                &state,
                &canonical_fact(SilentSessionLifecycleState::WaitingInput)
            ),
            Err(SilentSessionReduceError::WaitingInputRequiresExplicitFreshObservation)
        );
        let now = Utc::now();
        let fact = SilentSessionFact::LifecycleTransition {
            target: SilentSessionLifecycleState::WaitingInput,
            evidence: LifecycleTransitionEvidence::ExplicitInputRequest {
                event_ref: "event:prompt".into(),
                source: "pi_rpc".into(),
                confidence: 0.99,
                observed_at: now,
                fresh_until: now + chrono::Duration::seconds(30),
                provenance: ObservationProvenance::RuntimeObserved,
            },
        };
        assert!(reduce_silent_session(&state, &fact).is_ok());
    }

    #[test]
    fn blocked_and_completed_require_typed_truth() {
        let running = state_at(SilentSessionLifecycleState::Running);
        assert_eq!(
            reduce_silent_session(
                &running,
                &canonical_fact(SilentSessionLifecycleState::Blocked)
            ),
            Err(SilentSessionReduceError::BlockedRequiresTypedBlocker)
        );
        let completing = state_at(SilentSessionLifecycleState::Completing);
        assert_eq!(
            reduce_silent_session(
                &completing,
                &canonical_fact(SilentSessionLifecycleState::Completed)
            ),
            Err(SilentSessionReduceError::CompletedRequiresReceiptReadyEvaluation)
        );
        let fact = SilentSessionFact::LifecycleTransition {
            target: SilentSessionLifecycleState::Completed,
            evidence: LifecycleTransitionEvidence::CompletionEvaluation {
                evaluation_id: SilentSessionCompletionEvaluationId::new(),
                decision: CompletionDecision::Completed,
                receipt_ready: true,
            },
        };
        assert!(reduce_silent_session(&completing, &fact).is_ok());
    }

    #[test]
    fn process_exit_health_does_not_imply_completion() {
        let state = state_at(SilentSessionLifecycleState::Running);
        let next = reduce_silent_session(
            &state,
            &SilentSessionFact::HealthObserved {
                health: SilentSessionHealth::ProcessExited,
                source: "native_backend".into(),
                observed_at: Utc::now(),
                provenance: ObservationProvenance::RuntimeObserved,
            },
        )
        .unwrap();
        assert_eq!(
            next.session.lifecycle_state,
            SilentSessionLifecycleState::Running
        );
        assert_eq!(next.session.health, SilentSessionHealth::ProcessExited);
    }

    #[test]
    fn stale_activity_is_not_returned_as_current_truth() {
        let now = Utc::now();
        let observation = SemanticObservation {
            activity: SilentSessionSemanticActivity::Thinking,
            source: "adapter".into(),
            confidence: 0.9,
            observed_at: now - chrono::Duration::seconds(60),
            fresh_until: now - chrono::Duration::seconds(1),
            provenance: ObservationProvenance::RuntimeObserved,
        };
        assert_eq!(semantic_activity_is_fresh(Some(&observation), now), None);
    }
}
