//! Typed append-only events and deterministic replay for `SemanticPair`.

use crate::semantic_pair::{
    Assignment, BuilderAttempt, BuilderContext, Disposition, Finding, ImmutableSnapshot,
    Obligation, Plan, Reroute, Response, SemanticPair, SemanticPairError, SemanticReceipt,
    Settlement, Validation, SEMANTIC_PAIR_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const GENESIS_HASH: &str = "semantic-pair-genesis-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SemanticPairEvent {
    PairCreated {
        builder_attempt: BuilderAttempt,
        builder_context: BuilderContext,
        snapshot: ImmutableSnapshot,
    },
    ObligationAdded(Obligation),
    PlanAdded(Plan),
    AssignmentAdded(Assignment),
    FindingAdded(Finding),
    ResponseAdded(Response),
    DispositionAdded(Disposition),
    ValidationAdded(Validation),
    RerouteAdded(Reroute),
    SettlementAdded(Settlement),
    ReceiptAdded(SemanticReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticEventEnvelope {
    pub event_id: String,
    pub pair_id: String,
    pub sequence: u64,
    pub schema_version: u32,
    pub occurred_at: String,
    pub previous_hash: String,
    pub event: SemanticPairEvent,
    pub hash: String,
}

impl SemanticEventEnvelope {
    pub fn new(
        event_id: impl Into<String>,
        pair_id: impl Into<String>,
        sequence: u64,
        occurred_at: impl Into<String>,
        previous_hash: impl Into<String>,
        event: SemanticPairEvent,
    ) -> Result<Self, ReplayError> {
        let mut envelope = Self {
            event_id: event_id.into(),
            pair_id: pair_id.into(),
            sequence,
            schema_version: SEMANTIC_PAIR_SCHEMA_VERSION,
            occurred_at: occurred_at.into(),
            previous_hash: previous_hash.into(),
            event,
            hash: String::new(),
        };
        envelope.hash = envelope.computed_hash()?;
        Ok(envelope)
    }

    pub fn computed_hash(&self) -> Result<String, ReplayError> {
        #[derive(Serialize)]
        struct HashInput<'a> {
            event_id: &'a str,
            pair_id: &'a str,
            sequence: u64,
            schema_version: u32,
            occurred_at: &'a str,
            previous_hash: &'a str,
            event: &'a SemanticPairEvent,
        }
        let input = HashInput {
            event_id: &self.event_id,
            pair_id: &self.pair_id,
            sequence: self.sequence,
            schema_version: self.schema_version,
            occurred_at: &self.occurred_at,
            previous_hash: &self.previous_hash,
            event: &self.event,
        };
        let bytes = serde_json::to_vec(&input).map_err(ReplayError::Serialization)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayResult {
    pub aggregate: SemanticPair,
    pub event_count: usize,
    pub head_hash: String,
}

/// Pure reducer. It performs no I/O and produces identical output for identical
/// ordered input.
pub fn replay(events: &[SemanticEventEnvelope]) -> Result<ReplayResult, ReplayError> {
    let mut aggregate: Option<SemanticPair> = None;
    let mut expected_hash = GENESIS_HASH.to_string();
    let mut ids = HashSet::new();
    let mut pair_id: Option<&str> = None;

    for (index, envelope) in events.iter().enumerate() {
        let expected_sequence = index as u64;
        if envelope.sequence != expected_sequence {
            return Err(ReplayError::OutOfOrder {
                expected: expected_sequence,
                actual: envelope.sequence,
            });
        }
        if !ids.insert(envelope.event_id.as_str()) {
            return Err(ReplayError::DuplicateEvent(envelope.event_id.clone()));
        }
        if let Some(expected_pair) = pair_id {
            if envelope.pair_id != expected_pair {
                return Err(ReplayError::PairMismatch);
            }
        } else {
            pair_id = Some(&envelope.pair_id);
        }
        if envelope.schema_version > SEMANTIC_PAIR_SCHEMA_VERSION {
            return Err(ReplayError::FutureVersion(envelope.schema_version));
        }
        if envelope.previous_hash != expected_hash {
            return Err(ReplayError::BrokenChain {
                sequence: envelope.sequence,
            });
        }
        if envelope.computed_hash()? != envelope.hash {
            return Err(ReplayError::HashMismatch {
                sequence: envelope.sequence,
            });
        }
        reduce_one(&mut aggregate, envelope)?;
        expected_hash.clone_from(&envelope.hash);
    }

    let aggregate = aggregate.ok_or(ReplayError::MissingCreation)?;
    aggregate
        .validate()
        .map_err(ReplayError::InvalidAggregate)?;
    Ok(ReplayResult {
        aggregate,
        event_count: events.len(),
        head_hash: expected_hash,
    })
}

fn reduce_one(
    aggregate: &mut Option<SemanticPair>,
    envelope: &SemanticEventEnvelope,
) -> Result<(), ReplayError> {
    match &envelope.event {
        SemanticPairEvent::PairCreated {
            builder_attempt,
            builder_context,
            snapshot,
        } => {
            if aggregate.is_some() || envelope.sequence != 0 {
                return Err(ReplayError::DuplicateCreation);
            }
            *aggregate = Some(SemanticPair::empty(
                envelope.pair_id.clone(),
                builder_attempt.clone(),
                builder_context.clone(),
                snapshot.clone(),
            ));
            return Ok(());
        }
        _ if aggregate.is_none() => return Err(ReplayError::MissingCreation),
        _ => {}
    }

    let pair = aggregate.as_mut().ok_or(ReplayError::MissingCreation)?;
    match &envelope.event {
        SemanticPairEvent::PairCreated { .. } => unreachable!(),
        SemanticPairEvent::ObligationAdded(value) => push_unique(&mut pair.obligations, value)?,
        SemanticPairEvent::PlanAdded(value) => push_unique(&mut pair.plans, value)?,
        SemanticPairEvent::AssignmentAdded(value) => push_unique(&mut pair.assignments, value)?,
        SemanticPairEvent::FindingAdded(value) => push_unique(&mut pair.findings, value)?,
        SemanticPairEvent::ResponseAdded(value) => push_unique(&mut pair.responses, value)?,
        SemanticPairEvent::DispositionAdded(value) => push_unique(&mut pair.dispositions, value)?,
        SemanticPairEvent::ValidationAdded(value) => push_unique(&mut pair.validations, value)?,
        SemanticPairEvent::RerouteAdded(value) => push_unique(&mut pair.reroutes, value)?,
        SemanticPairEvent::SettlementAdded(value) => push_unique(&mut pair.settlements, value)?,
        SemanticPairEvent::ReceiptAdded(value) => {
            value.validate().map_err(ReplayError::InvalidAggregate)?;
            if pair
                .receipts
                .iter()
                .any(|v| v.receipt_id == value.receipt_id)
            {
                return Err(ReplayError::DuplicateEntity(value.receipt_id.clone()));
            }
            pair.receipts.push(value.clone());
        }
    }
    Ok(())
}

fn push_unique<T>(values: &mut Vec<T>, value: &T) -> Result<(), ReplayError>
where
    T: Clone + AsRefSemanticItem,
{
    value
        .semantic_item()
        .validate()
        .map_err(ReplayError::InvalidAggregate)?;
    let id = &value.semantic_item().id;
    if values.iter().any(|v| v.semantic_item().id == *id) {
        return Err(ReplayError::DuplicateEntity(id.clone()));
    }
    values.push(value.clone());
    Ok(())
}

trait AsRefSemanticItem {
    fn semantic_item(&self) -> &crate::semantic_pair::SemanticItem;
}

impl AsRefSemanticItem for crate::semantic_pair::SemanticItem {
    fn semantic_item(&self) -> &crate::semantic_pair::SemanticItem {
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("semantic event serialization failed: {0}")]
    Serialization(serde_json::Error),
    #[error("event sequence is out of order: expected {expected}, got {actual}")]
    OutOfOrder { expected: u64, actual: u64 },
    #[error("duplicate event id {0}")]
    DuplicateEvent(String),
    #[error("duplicate aggregate entity id {0}")]
    DuplicateEntity(String),
    #[error("events from different semantic pairs cannot share one stream")]
    PairMismatch,
    #[error("semantic event hash chain is broken at sequence {sequence}")]
    BrokenChain { sequence: u64 },
    #[error("semantic event hash does not match its payload at sequence {sequence}")]
    HashMismatch { sequence: u64 },
    #[error("semantic pair stream has no creation event")]
    MissingCreation,
    #[error("semantic pair has more than one creation event")]
    DuplicateCreation,
    #[error("semantic event version {0} is newer than this runtime")]
    FutureVersion(u32),
    #[error("invalid semantic aggregate: {0}")]
    InvalidAggregate(SemanticPairError),
}
