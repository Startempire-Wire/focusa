//! Spec138 append-only outcome resolution authority and correction reducer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionOutcomeObservation {
    pub observation_id: String,
    pub action_id: String,
    pub commitment_id: String,
    pub predicted_outcome: String,
    pub actual_outcome: String,
    pub outcome_match_score: f64,
    pub completed_temporal: crate::temporal_clock::TemporalActionEnvelope,
    pub timing_trace: crate::temporal_progress::ActionTimingTrace,
    pub expected_duration_ns: u128,
    pub actual_duration_ns: u128,
    pub duration_delta_ns: i128,
    pub outcome_claim: crate::prediction_authority::OutcomeClaim,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcomeObservationError {
    MissingIdentity,
    CrossActionSettlement,
    CommitmentMismatch,
    OutcomePredictionMismatch,
    CompletionBeforeAction,
    UnavailableCompletionTime,
    InvalidTimingTrace,
    DurationMismatch,
    ExpectedDurationMismatch,
    InvalidDurationDelta,
    InvalidOutcomeScore,
    ClaimTimestampMismatch,
    MissingEvidence,
}

pub fn validate_action_outcome_observation(
    prediction: &crate::prediction_authority::ActionPredictionCommitment,
    observation: &ActionOutcomeObservation,
) -> Result<(), ActionOutcomeObservationError> {
    if observation.observation_id.trim().is_empty() || observation.action_id.trim().is_empty() {
        return Err(ActionOutcomeObservationError::MissingIdentity);
    }
    if observation.action_id != prediction.action_id
        || observation.timing_trace.action_id != prediction.action_id
    {
        return Err(ActionOutcomeObservationError::CrossActionSettlement);
    }
    if observation.commitment_id != prediction.commitment.commitment_id
        || observation.outcome_claim.commitment_id != prediction.commitment.commitment_id
        || observation.timing_trace.prediction_id != prediction.commitment.commitment_id
    {
        return Err(ActionOutcomeObservationError::CommitmentMismatch);
    }
    if observation.predicted_outcome != prediction.commitment.predicted_outcome {
        return Err(ActionOutcomeObservationError::OutcomePredictionMismatch);
    }
    if observation.completed_temporal.confidence == crate::temporal::TemporalConfidence::Unavailable
        || observation.completed_temporal.capture_failure.is_some()
    {
        return Err(ActionOutcomeObservationError::UnavailableCompletionTime);
    }
    if observation.completed_temporal.monotonic_ns < prediction.action_start_temporal.monotonic_ns
        || observation.completed_temporal.captured_at_utc
            < prediction.action_start_temporal.captured_at_utc
    {
        return Err(ActionOutcomeObservationError::CompletionBeforeAction);
    }
    crate::temporal_progress::validate_action_timing_trace(&observation.timing_trace)
        .map_err(|_| ActionOutcomeObservationError::InvalidTimingTrace)?;
    if observation.actual_duration_ns != observation.timing_trace.total_elapsed_ns {
        return Err(ActionOutcomeObservationError::DurationMismatch);
    }
    if observation.expected_duration_ns != prediction.duration_baseline.estimate_ns {
        return Err(ActionOutcomeObservationError::ExpectedDurationMismatch);
    }
    if observation.duration_delta_ns
        != observation.actual_duration_ns as i128 - observation.expected_duration_ns as i128
    {
        return Err(ActionOutcomeObservationError::InvalidDurationDelta);
    }
    if !(0.0..=1.0).contains(&observation.outcome_match_score) {
        return Err(ActionOutcomeObservationError::InvalidOutcomeScore);
    }
    if observation.outcome_claim.claimed_at != observation.completed_temporal.captured_at_utc {
        return Err(ActionOutcomeObservationError::ClaimTimestampMismatch);
    }
    if observation.evidence_refs.is_empty() || observation.outcome_claim.evidence_refs.is_empty() {
        return Err(ActionOutcomeObservationError::MissingEvidence);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeState {
    Claimed,
    Disputed,
    PendingAuthority,
    Resolved,
    Corrected,
    Void,
    Censored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAuthorityKind {
    RegisteredResolver,
    Operator,
    ExternalAuthority,
    Reducer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum OutcomeAuthorityAction {
    Claim {
        claimed_outcome: String,
    },
    Dispute {
        reason: String,
    },
    Escalate,
    Resolve {
        resolved_outcome: String,
    },
    Correct {
        resolved_outcome: String,
        supersedes_event_ref: String,
    },
    Void {
        reason: String,
    },
    Censor {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeAuthorityEvent {
    pub event_id: String,
    pub commitment_id: String,
    pub action: OutcomeAuthorityAction,
    pub authority: ResolutionAuthorityKind,
    pub authority_ref: String,
    pub resolution_policy_ref: String,
    pub resolution_policy_version: u64,
    pub policy_locked_at: DateTime<Utc>,
    pub commitment_at: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
    pub caller_score_advisory: Option<f64>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeAuthorityProjection {
    pub commitment_id: String,
    pub state: OutcomeState,
    pub canonical_outcome: Option<String>,
    pub canonical_resolution_event_ref: Option<String>,
    pub policy_ref: String,
    pub policy_version: u64,
    pub history: Vec<OutcomeAuthorityEvent>,
    pub advisory_caller_scores: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutcomeAuthorityError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    PolicyLockedAfterCommitment,
    PolicyChanged,
    CommitmentMismatch,
    InvalidTransition,
    MissingSupersession,
    UnauthorizedResolution,
    NotResolvedForScoring,
}

#[derive(Debug, Default)]
pub struct OutcomeAuthorityLedger {
    projection: Option<OutcomeAuthorityProjection>,
}

impl OutcomeAuthorityLedger {
    pub fn append(&mut self, event: OutcomeAuthorityEvent) -> Result<(), OutcomeAuthorityError> {
        validate_outcome_authority_event(&event)?;
        if !matches!(
            event.authority,
            ResolutionAuthorityKind::RegisteredResolver
                | ResolutionAuthorityKind::Operator
                | ResolutionAuthorityKind::ExternalAuthority
                | ResolutionAuthorityKind::Reducer
        ) {
            return Err(OutcomeAuthorityError::UnauthorizedResolution);
        }
        let next_state = action_state(&event.action);
        match &mut self.projection {
            None => {
                if !matches!(event.action, OutcomeAuthorityAction::Claim { .. }) {
                    return Err(OutcomeAuthorityError::InvalidTransition);
                }
                self.projection = Some(OutcomeAuthorityProjection {
                    commitment_id: event.commitment_id.clone(),
                    state: next_state,
                    canonical_outcome: action_outcome(&event.action),
                    canonical_resolution_event_ref: None,
                    policy_ref: event.resolution_policy_ref.clone(),
                    policy_version: event.resolution_policy_version,
                    history: vec![event.clone()],
                    advisory_caller_scores: event.caller_score_advisory.into_iter().collect(),
                });
            }
            Some(projection) => {
                if projection.commitment_id != event.commitment_id {
                    return Err(OutcomeAuthorityError::CommitmentMismatch);
                }
                if projection.policy_ref != event.resolution_policy_ref
                    || projection.policy_version != event.resolution_policy_version
                {
                    return Err(OutcomeAuthorityError::PolicyChanged);
                }
                validate_transition(projection, &event)?;
                projection.state = next_state;
                if let Some(outcome) = action_outcome(&event.action) {
                    projection.canonical_outcome = Some(outcome);
                }
                if matches!(
                    event.action,
                    OutcomeAuthorityAction::Resolve { .. } | OutcomeAuthorityAction::Correct { .. }
                ) {
                    projection.canonical_resolution_event_ref = Some(event.event_id.clone());
                }
                if let Some(score) = event.caller_score_advisory {
                    projection.advisory_caller_scores.push(score);
                }
                projection.history.push(event.clone());
            }
        }
        Ok(())
    }

    pub fn projection(&self) -> Option<&OutcomeAuthorityProjection> {
        self.projection.as_ref()
    }

    pub fn scoring_resolution_ref(&self) -> Result<&str, OutcomeAuthorityError> {
        let projection = self
            .projection
            .as_ref()
            .ok_or(OutcomeAuthorityError::NotResolvedForScoring)?;
        if !matches!(
            projection.state,
            OutcomeState::Resolved | OutcomeState::Corrected
        ) {
            return Err(OutcomeAuthorityError::NotResolvedForScoring);
        }
        projection
            .canonical_resolution_event_ref
            .as_deref()
            .ok_or(OutcomeAuthorityError::NotResolvedForScoring)
    }
}

pub fn validate_outcome_authority_event(
    event: &OutcomeAuthorityEvent,
) -> Result<(), OutcomeAuthorityError> {
    if event.event_id.trim().is_empty()
        || event.commitment_id.trim().is_empty()
        || event.authority_ref.trim().is_empty()
        || event.resolution_policy_ref.trim().is_empty()
        || event.resolution_policy_version == 0
    {
        return Err(OutcomeAuthorityError::MissingIdentity);
    }
    if event.evidence_refs.is_empty() {
        return Err(OutcomeAuthorityError::MissingEvidence);
    }
    if event.receipt_ref.trim().is_empty() {
        return Err(OutcomeAuthorityError::MissingReceipt);
    }
    if event.policy_locked_at > event.commitment_at {
        return Err(OutcomeAuthorityError::PolicyLockedAfterCommitment);
    }
    Ok(())
}

fn validate_transition(
    projection: &OutcomeAuthorityProjection,
    event: &OutcomeAuthorityEvent,
) -> Result<(), OutcomeAuthorityError> {
    let allowed = match (&projection.state, &event.action) {
        (
            OutcomeState::Claimed,
            OutcomeAuthorityAction::Dispute { .. }
            | OutcomeAuthorityAction::Escalate
            | OutcomeAuthorityAction::Resolve { .. }
            | OutcomeAuthorityAction::Void { .. }
            | OutcomeAuthorityAction::Censor { .. },
        ) => true,
        (
            OutcomeState::Disputed,
            OutcomeAuthorityAction::Escalate
            | OutcomeAuthorityAction::Resolve { .. }
            | OutcomeAuthorityAction::Void { .. }
            | OutcomeAuthorityAction::Censor { .. },
        ) => true,
        (
            OutcomeState::PendingAuthority,
            OutcomeAuthorityAction::Resolve { .. }
            | OutcomeAuthorityAction::Void { .. }
            | OutcomeAuthorityAction::Censor { .. },
        ) => true,
        (
            OutcomeState::Resolved | OutcomeState::Corrected,
            OutcomeAuthorityAction::Correct {
                supersedes_event_ref,
                ..
            },
        ) => projection.canonical_resolution_event_ref.as_deref() == Some(supersedes_event_ref),
        _ => false,
    };
    if !allowed {
        if matches!(event.action, OutcomeAuthorityAction::Correct { .. }) {
            return Err(OutcomeAuthorityError::MissingSupersession);
        }
        return Err(OutcomeAuthorityError::InvalidTransition);
    }
    Ok(())
}

fn action_state(action: &OutcomeAuthorityAction) -> OutcomeState {
    match action {
        OutcomeAuthorityAction::Claim { .. } => OutcomeState::Claimed,
        OutcomeAuthorityAction::Dispute { .. } => OutcomeState::Disputed,
        OutcomeAuthorityAction::Escalate => OutcomeState::PendingAuthority,
        OutcomeAuthorityAction::Resolve { .. } => OutcomeState::Resolved,
        OutcomeAuthorityAction::Correct { .. } => OutcomeState::Corrected,
        OutcomeAuthorityAction::Void { .. } => OutcomeState::Void,
        OutcomeAuthorityAction::Censor { .. } => OutcomeState::Censored,
    }
}

fn action_outcome(action: &OutcomeAuthorityAction) -> Option<String> {
    match action {
        OutcomeAuthorityAction::Claim { claimed_outcome } => Some(claimed_outcome.clone()),
        OutcomeAuthorityAction::Resolve { resolved_outcome }
        | OutcomeAuthorityAction::Correct {
            resolved_outcome, ..
        } => Some(resolved_outcome.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, action: OutcomeAuthorityAction) -> OutcomeAuthorityEvent {
        let now = Utc::now();
        OutcomeAuthorityEvent {
            event_id: id.into(),
            commitment_id: "commitment".into(),
            action,
            authority: ResolutionAuthorityKind::RegisteredResolver,
            authority_ref: "resolver:v1".into(),
            resolution_policy_ref: "policy:v1".into(),
            resolution_policy_version: 1,
            policy_locked_at: now - chrono::Duration::seconds(1),
            commitment_at: now,
            occurred_at: now,
            caller_score_advisory: Some(0.0),
            evidence_refs: vec![format!("evidence:{id}")],
            receipt_ref: format!("receipt:{id}"),
        }
    }

    #[test]
    fn claim_dispute_resolution_and_correction_are_append_only() {
        let mut ledger = OutcomeAuthorityLedger::default();
        ledger
            .append(event(
                "claim",
                OutcomeAuthorityAction::Claim {
                    claimed_outcome: "yes".into(),
                },
            ))
            .unwrap();
        ledger
            .append(event(
                "dispute",
                OutcomeAuthorityAction::Dispute {
                    reason: "conflict".into(),
                },
            ))
            .unwrap();
        ledger
            .append(event(
                "resolve",
                OutcomeAuthorityAction::Resolve {
                    resolved_outcome: "no".into(),
                },
            ))
            .unwrap();
        ledger
            .append(event(
                "correct",
                OutcomeAuthorityAction::Correct {
                    resolved_outcome: "yes".into(),
                    supersedes_event_ref: "resolve".into(),
                },
            ))
            .unwrap();
        let p = ledger.projection().unwrap();
        assert_eq!(p.state, OutcomeState::Corrected);
        assert_eq!(p.canonical_outcome.as_deref(), Some("yes"));
        assert_eq!(p.history.len(), 4);
        assert_eq!(ledger.scoring_resolution_ref().unwrap(), "correct");
    }

    #[test]
    fn policy_changes_bad_corrections_and_unresolved_scoring_fail_closed() {
        let mut ledger = OutcomeAuthorityLedger::default();
        ledger
            .append(event(
                "claim",
                OutcomeAuthorityAction::Claim {
                    claimed_outcome: "yes".into(),
                },
            ))
            .unwrap();
        assert_eq!(
            ledger.scoring_resolution_ref(),
            Err(OutcomeAuthorityError::NotResolvedForScoring)
        );
        let mut changed = event(
            "resolve",
            OutcomeAuthorityAction::Resolve {
                resolved_outcome: "yes".into(),
            },
        );
        changed.resolution_policy_version = 2;
        assert_eq!(
            ledger.append(changed),
            Err(OutcomeAuthorityError::PolicyChanged)
        );
        ledger
            .append(event(
                "resolve",
                OutcomeAuthorityAction::Resolve {
                    resolved_outcome: "yes".into(),
                },
            ))
            .unwrap();
        assert_eq!(
            ledger.append(event(
                "bad",
                OutcomeAuthorityAction::Correct {
                    resolved_outcome: "no".into(),
                    supersedes_event_ref: "wrong".into()
                }
            )),
            Err(OutcomeAuthorityError::MissingSupersession)
        );
    }
}
