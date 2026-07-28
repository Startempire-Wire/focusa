//! Append-only scoped Spec 138 authority ledger and projection.

use crate::prediction_authority::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PredictionAuthorityProjection {
    pub sequence: u64,
    pub questions: BTreeMap<String, PredictionQuestion>,
    pub commitments: BTreeMap<String, PredictionCommitment>,
    pub resolutions: BTreeMap<String, OutcomeResolution>,
    pub evaluations: BTreeMap<String, PredictionEvaluation>,
    pub learning: BTreeMap<String, LearningRecord>,
    pub transfers: BTreeMap<String, TransferOutcome>,
}

#[derive(Debug, Clone, Default)]
pub struct PredictionAuthorityLedger {
    events: Vec<ScopedAuthorityEvent>,
    event_ids: BTreeSet<String>,
}

impl PredictionAuthorityLedger {
    pub fn append(&mut self, event: ScopedAuthorityEvent) -> Result<(), String> {
        if event.scope.project_root.trim().is_empty() || event.scope.continuity_id.trim().is_empty()
        {
            return Err("typed project/workstream scope required".into());
        }
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
                PredictionAuthorityEvent::OutcomeResolution(value) => {
                    projection
                        .resolutions
                        .insert(value.resolution_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::Evaluation(value) => {
                    projection
                        .evaluations
                        .insert(value.evaluation_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::LearningRecord(value) => {
                    projection
                        .learning
                        .insert(value.learning_id.clone(), value.clone());
                }
                PredictionAuthorityEvent::TransferOutcome(value) => {
                    projection
                        .transfers
                        .insert(value.outcome_id.clone(), value.clone());
                }
                _ => {}
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
