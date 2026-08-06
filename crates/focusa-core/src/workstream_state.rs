//! Spec 158 project-to-Workstream state partition.
//!
//! The complete ProjectState fields migrate in bounded slices.  This module owns
//! the canonical partition now: cognition is addressable only by durable
//! [`WorkstreamId`] and an exact [`WorkstreamKey`], never by continuity, session,
//! attachment, UI selection, or recency.
//!
//! `WorkstreamState::cognitive_state` is an explicit, fully typed migration seam
//! around the existing reducer owner (`FocusaState`).  It is deliberately not a
//! `serde_json::Value` bag or a second inferred state root.  Later slices can move
//! the concrete fields behind the typed accessors without changing the ProjectState
//! routing contract.

use crate::types::{
    ContextClaimRecord, ContextSourceRecord, ExplicitMemory, FocusStackState, FocusState,
    FocusaEvent, FocusaState, OntologyState, ReactiveContextProjection, ReferenceIndex,
    TrajectoryState, WorkLoopState, WorkpointState,
};
use crate::workstream_context::{
    WorkstreamContext, WorkstreamContextError, WorkstreamRequestEnvelope,
};
use crate::workstream_identity::{ScopeRef, WorkstreamId, WorkstreamKey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkstreamStateError {
    #[error("workstream is already registered: {0}")]
    AlreadyRegistered(WorkstreamId),
    #[error("workstream is not registered: {0}")]
    NotFound(WorkstreamId),
}

/// The durable reducer/event head for one Workstream partition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EventHead {
    /// The successful reducer revision represented by this Workstream.
    pub sequence: u64,
}

/// Version of the typed Workstream projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(transparent)]
pub struct ProjectionVersion(pub u64);

pub const ACTIVE_WORKSTREAM_COMMAND_ID: &str = "focusa.workstream.active.select";
pub const ACTIVE_WORKSTREAM_EVENT_SCHEMA_V1: &str = "focusa.workstream.active_selected.v1";
pub const ACTIVE_WORKSTREAM_RECEIPT_SCHEMA_V1: &str =
    "focusa.workstream.active_selected_receipt.v1";
const MAX_ACTIVE_WORKSTREAM_IDEMPOTENCY_RECORDS: usize = 128;

/// The canonical project-owned selection cursor.  It is deliberately separate
/// from every Workstream's cognitive payload: selecting one Workstream is a
/// mutation of the owning project partition, not a daemon-wide pointer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ActiveWorkstreamState {
    /// The only canonical active selection.  The field is private so callers
    /// cannot grant themselves authority by assigning a UI value directly.
    #[serde(default)]
    active_workstream: Option<WorkstreamKey>,
    /// Monotonic reducer revision for the selection partition.  It is private
    /// so a client cannot advance authority without a reducer transition.
    revision: u64,
    /// Monotonic fencing cursor for delayed or concurrent selection requests.
    fencing_token: u64,
    /// Bounded canonical idempotency receipts for safe replay.
    #[serde(default)]
    idempotency_records: Vec<ActiveWorkstreamIdempotencyRecord>,
}

impl ActiveWorkstreamState {
    pub fn active_workstream(&self) -> Option<&WorkstreamKey> {
        self.active_workstream.as_ref()
    }

    pub fn active_workstream_id(&self) -> Option<&WorkstreamId> {
        self.active_workstream
            .as_ref()
            .map(|key| &key.workstream_id)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn fencing_token(&self) -> u64 {
        self.fencing_token
    }

    pub fn idempotency_records(&self) -> &[ActiveWorkstreamIdempotencyRecord] {
        &self.idempotency_records
    }

    fn validate_shape(&self) -> Result<(), ActiveWorkstreamError> {
        match self.active_workstream.as_ref() {
            Some(_) if self.revision == 0 || self.fencing_token == 0 => {
                Err(ActiveWorkstreamError::InvalidPartition(
                    "active selection has a zero revision or fencing token".to_string(),
                ))
            }
            Some(_) => Ok(()),
            None if self.revision != 0
                || self.fencing_token != 0
                || !self.idempotency_records.is_empty() =>
            {
                Err(ActiveWorkstreamError::InvalidPartition(
                    "selection cursor has history but no active Workstream".to_string(),
                ))
            }
            None => Ok(()),
        }
    }

    fn find_idempotency_record(&self, key: &str) -> Option<&ActiveWorkstreamIdempotencyRecord> {
        self.idempotency_records
            .iter()
            .find(|record| record.idempotency_key == key)
    }

    fn apply(
        &mut self,
        event: &ActiveWorkstreamEvent,
        receipt: &ActiveWorkstreamReceipt,
        command_fingerprint: String,
    ) -> Result<(), ActiveWorkstreamError> {
        if self.revision != event.before_revision {
            return Err(ActiveWorkstreamError::StaleRevision {
                expected: event.before_revision,
                actual: self.revision,
            });
        }
        if self.fencing_token.checked_add(1) != Some(event.fencing_token) {
            return Err(ActiveWorkstreamError::StaleFencingToken {
                expected: self.fencing_token,
                actual: event.fencing_token,
            });
        }
        if self.active_workstream != event.previous_workstream {
            return Err(ActiveWorkstreamError::InvalidPartition(
                "active selection predecessor does not match reducer state".to_string(),
            ));
        }
        self.active_workstream = Some(event.active_workstream.clone());
        self.revision = event.after_revision;
        self.fencing_token = event.fencing_token;
        self.idempotency_records
            .push(ActiveWorkstreamIdempotencyRecord {
                idempotency_key: event.idempotency_key.clone(),
                workstream: event.workstream.clone(),
                command_fingerprint,
                event: event.clone(),
                receipt: receipt.clone(),
            });
        if self.idempotency_records.len() > MAX_ACTIVE_WORKSTREAM_IDEMPOTENCY_RECORDS {
            self.idempotency_records.remove(0);
        }
        Ok(())
    }
}

/// A Desktop, Pi, CLI, or agent selection is only a request until the
/// canonical reducer accepts it.  It carries the exact WorkstreamKey and both
/// optimistic-concurrency cursors; it contains no presentation-derived owner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveWorkstreamCommand {
    pub workstream: WorkstreamKey,
    pub context: WorkstreamContext,
    pub command_id: String,
    pub idempotency_key: String,
    pub expected_revision: u64,
    pub expected_fencing_token: u64,
}

impl ActiveWorkstreamCommand {
    pub fn new(
        context: WorkstreamContext,
        idempotency_key: impl Into<String>,
        expected_revision: u64,
        expected_fencing_token: u64,
    ) -> Self {
        Self::with_command_id(
            context,
            ACTIVE_WORKSTREAM_COMMAND_ID,
            idempotency_key,
            expected_revision,
            expected_fencing_token,
        )
    }

    pub fn with_command_id(
        context: WorkstreamContext,
        command_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        expected_revision: u64,
        expected_fencing_token: u64,
    ) -> Self {
        Self {
            workstream: context.workstream.clone(),
            context,
            command_id: command_id.into(),
            idempotency_key: idempotency_key.into(),
            expected_revision,
            expected_fencing_token,
        }
    }

    /// Convert the shared request envelope into the active-selection command.
    /// The request must already carry the exact WorkstreamKey, idempotency key,
    /// revision cursor and fencing cursor.
    pub fn from_request(
        mut request: WorkstreamRequestEnvelope,
    ) -> Result<Self, ActiveWorkstreamError> {
        let idempotency_key = request
            .idempotency_key
            .take()
            .filter(|value| !value.trim().is_empty())
            .ok_or(ActiveWorkstreamError::MissingIdempotencyKey)?;
        let expected_revision = request
            .expected_revision
            .ok_or(ActiveWorkstreamError::MissingExpectedRevision)?;
        let expected_fencing_token = request
            .expected_fencing_token
            .ok_or(ActiveWorkstreamError::MissingExpectedFencingToken)?;
        let command_id = request.command_id.clone();
        if command_id.trim().is_empty() {
            return Err(ActiveWorkstreamError::MissingCommandId);
        }
        let context = WorkstreamContext::extract(request)?;
        let workstream = context.workstream.clone();
        Ok(Self {
            workstream,
            context,
            command_id,
            idempotency_key,
            expected_revision,
            expected_fencing_token,
        })
    }

    pub fn operation_fingerprint(&self) -> String {
        let bytes = serde_json::to_vec((
            &self.workstream,
            &self.context.actor,
            &self.context.authority.authority_ref,
            &self.command_id,
        ))
        .unwrap_or_default();
        format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
    }

    /// Validate every authority-bearing part of the request before the
    /// reducer can inspect or mutate the active selection cursor.
    pub fn validate(&self, state: &CanonicalProjectState) -> Result<(), ActiveWorkstreamError> {
        self.validate_registered_target(state)?;
        if self.idempotency_key.trim().is_empty() {
            return Err(ActiveWorkstreamError::MissingIdempotencyKey);
        }
        if self.command_id.trim().is_empty() {
            return Err(ActiveWorkstreamError::MissingCommandId);
        }

        let cursor = state.active_workstream_state();
        if let Some(record) = cursor.find_idempotency_record(&self.idempotency_key) {
            if record.workstream != self.workstream
                || record.command_fingerprint != self.operation_fingerprint()
            {
                return Err(ActiveWorkstreamError::IdempotencyConflict);
            }
            // An exact replay may carry the original (now stale) cursors.  It
            // returns the original canonical result without another mutation.
            return Ok(());
        }

        if self.expected_revision != cursor.revision {
            return Err(ActiveWorkstreamError::StaleRevision {
                expected: self.expected_revision,
                actual: cursor.revision,
            });
        }
        if self.expected_fencing_token != cursor.fencing_token {
            return Err(ActiveWorkstreamError::StaleFencingToken {
                expected: self.expected_fencing_token,
                actual: cursor.fencing_token,
            });
        }
        Ok(())
    }

    fn validate_registered_target(
        &self,
        state: &CanonicalProjectState,
    ) -> Result<(), ActiveWorkstreamError> {
        self.context
            .validate_for_workstream(&self.workstream)
            .map_err(ActiveWorkstreamError::Context)?;
        if self.workstream.workstream_id.as_str().trim().is_empty()
            || self.workstream.legacy_scope().validate().is_err()
        {
            return Err(ActiveWorkstreamError::InvalidWorkstream);
        }

        let target = state
            .workstreams
            .get(&self.workstream.workstream_id)
            .ok_or_else(|| {
                ActiveWorkstreamError::UnknownWorkstream(self.workstream.workstream_id.clone())
            })?;
        if target.key != self.workstream {
            return Err(ActiveWorkstreamError::ForeignWorkstream);
        }

        let mut project_scope: Option<&ScopeRef> = None;
        for (map_id, candidate) in &state.workstreams {
            if map_id != &candidate.key.workstream_id {
                return Err(ActiveWorkstreamError::InvalidPartition(
                    "ProjectState map key does not match WorkstreamState identity".to_string(),
                ));
            }
            if let Some(existing_scope) = project_scope {
                if existing_scope != &candidate.key.scope {
                    return Err(ActiveWorkstreamError::CrossProjectWorkstream);
                }
            } else {
                project_scope = Some(&candidate.key.scope);
            }
        }
        if project_scope != Some(&self.workstream.scope) {
            return Err(ActiveWorkstreamError::ForeignWorkstream);
        }

        let cursor = state.active_workstream_state();
        cursor.validate_shape()?;
        if let Some(active) = cursor.active_workstream() {
            let active_state = state
                .workstreams
                .get(&active.workstream_id)
                .ok_or_else(|| {
                    ActiveWorkstreamError::InvalidPartition(
                        "active selection points to an unregistered Workstream".to_string(),
                    )
                })?;
            if active_state.key != *active {
                return Err(ActiveWorkstreamError::InvalidPartition(
                    "active selection does not exactly own registered state".to_string(),
                ));
            }
            if active.scope != self.workstream.scope {
                return Err(ActiveWorkstreamError::CrossProjectWorkstream);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActiveWorkstreamError {
    #[error("active Workstream command has no idempotency key")]
    MissingIdempotencyKey,
    #[error("active Workstream command has no command id")]
    MissingCommandId,
    #[error("active Workstream command has no expected revision cursor")]
    MissingExpectedRevision,
    #[error("active Workstream command has no expected fencing cursor")]
    MissingExpectedFencingToken,
    #[error("active Workstream key is invalid")]
    InvalidWorkstream,
    #[error("active Workstream is not registered: {0}")]
    UnknownWorkstream(WorkstreamId),
    #[error("active Workstream key does not exactly own registered state")]
    ForeignWorkstream,
    #[error("active Workstream state contains more than one project scope")]
    CrossProjectWorkstream,
    #[error("active Workstream partition is inconsistent: {0}")]
    InvalidPartition(String),
    #[error("stale active Workstream revision: expected {expected}, actual {actual}")]
    StaleRevision { expected: u64, actual: u64 },
    #[error("stale active Workstream fencing token: expected {expected}, actual {actual}")]
    StaleFencingToken { expected: u64, actual: u64 },
    #[error("active Workstream idempotency key was reused for another command")]
    IdempotencyConflict,
    #[error("active Workstream revision cursor overflowed")]
    RevisionOverflow,
    #[error("active Workstream event is invalid: {0}")]
    InvalidEvent(String),
    #[error("active Workstream receipt is invalid: {0}")]
    InvalidReceipt(String),
    #[error("active Workstream request context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
}

/// Canonical event emitted only after the reducer accepts an active-selection
/// command.  A client request is never itself an event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveWorkstreamEvent {
    pub schema: String,
    pub event_type: String,
    pub event_id: String,
    pub workstream: WorkstreamKey,
    pub previous_workstream: Option<WorkstreamKey>,
    pub active_workstream: WorkstreamKey,
    pub context: WorkstreamContext,
    pub command_id: String,
    pub idempotency_key: String,
    pub before_revision: u64,
    pub after_revision: u64,
    pub fencing_token: u64,
    pub recorded_at: DateTime<Utc>,
}

impl ActiveWorkstreamEvent {
    fn from_command(
        command: &ActiveWorkstreamCommand,
        previous_workstream: Option<WorkstreamKey>,
        before_revision: u64,
        after_revision: u64,
        fencing_token: u64,
    ) -> Self {
        let event_id = format!(
            "event:active-workstream:{}",
            stable_active_workstream_hash(&command.workstream, &command.idempotency_key)
        );
        Self {
            schema: ACTIVE_WORKSTREAM_EVENT_SCHEMA_V1.to_string(),
            event_type: "workstream.active_selected".to_string(),
            event_id,
            workstream: command.workstream.clone(),
            previous_workstream,
            active_workstream: command.workstream.clone(),
            context: command.context.clone(),
            command_id: command.command_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            before_revision,
            after_revision,
            fencing_token,
            recorded_at: Utc::now(),
        }
    }
}

/// Receipt proving that the canonical reducer, rather than a UI, granted the
/// active Workstream authority.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveWorkstreamReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub event_id: String,
    pub workstream: WorkstreamKey,
    pub previous_workstream: Option<WorkstreamKey>,
    pub active_workstream: WorkstreamKey,
    pub authority_ref: String,
    pub idempotency_key: String,
    pub before_revision: u64,
    pub after_revision: u64,
    pub fencing_token: u64,
    pub evidence_ref: String,
    pub canonical: bool,
    pub replayed: bool,
    pub emitted_at: DateTime<Utc>,
}

/// The shared receipt emission seam for the active Workstream operation.
/// `Receipt::emit` is intentionally called by the canonical reducer only.
pub struct Receipt;

impl Receipt {
    pub fn emit(
        event: &ActiveWorkstreamEvent,
    ) -> Result<ActiveWorkstreamReceipt, ActiveWorkstreamError> {
        event
            .context
            .validate_for_workstream(&event.workstream)
            .map_err(ActiveWorkstreamError::Context)?;
        if event.schema != ACTIVE_WORKSTREAM_EVENT_SCHEMA_V1
            || event.event_type != "workstream.active_selected"
            || event.event_id.trim().is_empty()
            || event.command_id.trim().is_empty()
            || event.idempotency_key.trim().is_empty()
        {
            return Err(ActiveWorkstreamError::InvalidEvent(
                "active Workstream event identity is incomplete".to_string(),
            ));
        }
        if event.active_workstream != event.workstream {
            return Err(ActiveWorkstreamError::InvalidEvent(
                "active Workstream event target is not its exact owner".to_string(),
            ));
        }
        if event.after_revision != event.before_revision.saturating_add(1)
            || event.fencing_token == 0
        {
            return Err(ActiveWorkstreamError::InvalidEvent(
                "active Workstream event cursor is not monotonic".to_string(),
            ));
        }
        let receipt_id = format!("receipt:active-workstream:{}", event.event_id);
        Ok(ActiveWorkstreamReceipt {
            schema: ACTIVE_WORKSTREAM_RECEIPT_SCHEMA_V1.to_string(),
            receipt_id: receipt_id.clone(),
            event_id: event.event_id.clone(),
            workstream: event.workstream.clone(),
            previous_workstream: event.previous_workstream.clone(),
            active_workstream: event.active_workstream.clone(),
            authority_ref: event.context.authority.authority_ref.clone(),
            idempotency_key: event.idempotency_key.clone(),
            before_revision: event.before_revision,
            after_revision: event.after_revision,
            fencing_token: event.fencing_token,
            evidence_ref: format!("evidence:{receipt_id}"),
            canonical: true,
            replayed: false,
            emitted_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveWorkstreamIdempotencyRecord {
    pub idempotency_key: String,
    pub workstream: WorkstreamKey,
    pub command_fingerprint: String,
    pub event: ActiveWorkstreamEvent,
    pub receipt: ActiveWorkstreamReceipt,
}

/// Result returned by the canonical active-selection reducer.
#[derive(Debug, Clone)]
pub struct ActiveWorkstreamReductionResult {
    pub new_state: CanonicalProjectState,
    pub event: ActiveWorkstreamEvent,
    pub emitted_events: Vec<ActiveWorkstreamEvent>,
    pub receipt: ActiveWorkstreamReceipt,
}

/// Canonical active Workstream reduction.  It resolves only an exact
/// registered WorkstreamKey and mutates only the project-owned cursor.
pub fn reduce_active_workstream(
    mut state: CanonicalProjectState,
    command: ActiveWorkstreamCommand,
) -> Result<ActiveWorkstreamReductionResult, ActiveWorkstreamError> {
    command.validate(&state)?;
    let fingerprint = command.operation_fingerprint();
    if let Some(record) = state
        .active_workstream_state()
        .find_idempotency_record(&command.idempotency_key)
    {
        let event = record.event.clone();
        let mut receipt = record.receipt.clone();
        receipt.replayed = true;
        return Ok(ActiveWorkstreamReductionResult {
            new_state: state,
            event: event.clone(),
            emitted_events: vec![event],
            receipt,
        });
    }

    let cursor = state.active_workstream_state();
    let before_revision = cursor.revision;
    let after_revision = before_revision
        .checked_add(1)
        .ok_or(ActiveWorkstreamError::RevisionOverflow)?;
    let fencing_token = cursor
        .fencing_token
        .checked_add(1)
        .ok_or(ActiveWorkstreamError::RevisionOverflow)?;
    let previous_workstream = cursor.active_workstream().cloned();
    let event = ActiveWorkstreamEvent::from_command(
        &command,
        previous_workstream,
        before_revision,
        after_revision,
        fencing_token,
    );
    let receipt = Receipt::emit(&event)?;
    state
        .active_workstream_state_mut()
        .apply(&event, &receipt, fingerprint)?;
    Ok(ActiveWorkstreamReductionResult {
        new_state: state,
        event: event.clone(),
        emitted_events: vec![event],
        receipt,
    })
}

fn stable_active_workstream_hash(workstream: &WorkstreamKey, idempotency_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(workstream.storage_key().as_bytes());
    hasher.update([0]);
    hasher.update(idempotency_key.as_bytes());
    hex::encode(hasher.finalize())
}

/// The named reducer owner used by the Spec 158 active-state operation.
impl WorkstreamState {
    pub fn reduce(
        state: CanonicalProjectState,
        command: ActiveWorkstreamCommand,
    ) -> Result<ActiveWorkstreamReductionResult, ActiveWorkstreamError> {
        reduce_active_workstream(state, command)
    }
}

/// The project partition exposes the same canonical reducer without giving
/// clients a mutable active-selection setter.
impl ProjectState<WorkstreamState> {
    pub fn reduce(
        self,
        command: ActiveWorkstreamCommand,
    ) -> Result<ActiveWorkstreamReductionResult, ActiveWorkstreamError> {
        WorkstreamState::reduce(self, command)
    }
}

/// Canonical cognitive state for exactly one Workstream.
///
/// The existing [`FocusaState`] remains the concrete reducer owner during this
/// bounded migration slice.  The Workstream key and reducer metadata live beside
/// it so the state can be partitioned without inventing untyped field copies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamState {
    /// Exact durable owner.  This must match the ProjectState map key before a
    /// Workstream event can be reduced.
    pub key: WorkstreamKey,
    /// Typed migration seam to the existing reducer/domain state owners.
    #[serde(rename = "cognitive_state")]
    pub(crate) cognitive_state: FocusaState,
    pub event_head: EventHead,
    pub projection_version: ProjectionVersion,
}

impl WorkstreamState {
    /// Create an empty canonical Workstream state for an exact owner key.
    pub fn new(key: WorkstreamKey) -> Self {
        Self::from_focusa_state(key, FocusaState::default())
    }

    /// Wrap an existing concrete reducer state without changing its contents.
    pub fn from_focusa_state(key: WorkstreamKey, cognitive_state: FocusaState) -> Self {
        let revision = cognitive_state.version;
        Self {
            key,
            cognitive_state,
            event_head: EventHead { sequence: revision },
            projection_version: ProjectionVersion(revision),
        }
    }

    /// Read the existing typed reducer owner during migration.
    pub fn cognitive_state(&self) -> &FocusaState {
        &self.cognitive_state
    }

    /// Read the canonical Focus Stack owner without exposing a second copy.
    pub fn focus_stack(&self) -> &FocusStackState {
        &self.cognitive_state.focus_stack
    }

    /// Read the active frame's typed Focus State, if this Workstream has one.
    pub fn focus_state(&self) -> Option<&FocusState> {
        let active_id = self.cognitive_state.focus_stack.active_id?;
        self.cognitive_state
            .focus_stack
            .frames
            .iter()
            .find(|frame| frame.id == active_id)
            .map(|frame| &frame.focus_state)
    }

    /// Read the existing typed Workpoint owner.
    pub fn workpoints(&self) -> &WorkpointState {
        &self.cognitive_state.workpoint
    }

    /// Read the existing typed Trajectory owner.
    pub fn trajectory(&self) -> &TrajectoryState {
        &self.cognitive_state.trajectory
    }

    /// Read the existing typed Work Loop owner.
    pub fn work_loop(&self) -> &WorkLoopState {
        &self.cognitive_state.work_loop
    }

    /// Read the existing typed memory owner.
    pub fn memory(&self) -> &ExplicitMemory {
        &self.cognitive_state.memory
    }

    /// Read the existing typed ontology owner.
    pub fn ontology(&self) -> &OntologyState {
        &self.cognitive_state.ontology
    }

    /// Read the typed Context source owner.
    pub fn context_sources(&self) -> &Vec<ContextSourceRecord> {
        &self.cognitive_state.context_sources
    }

    /// Read the typed Context claim owner.
    pub fn context_claims(&self) -> &Vec<ContextClaimRecord> {
        &self.cognitive_state.context_claims
    }

    /// Read the typed reactive Context projection owner.
    pub fn reactive_context(&self) -> &Vec<ReactiveContextProjection> {
        &self.cognitive_state.reactive_context
    }

    /// Read the typed artifact/reference owner.
    pub fn reference_index(&self) -> &ReferenceIndex {
        &self.cognitive_state.reference_index
    }

    /// Revision represented by the reducer payload.
    pub(crate) fn reducer_revision(&self) -> u64 {
        self.cognitive_state.version
    }

    /// Borrow the reducer payload for the one selected ProjectState entry.
    pub(crate) fn reducer_state(&self) -> &FocusaState {
        &self.cognitive_state
    }

    /// Replace the reducer payload after a successful reduction.
    pub(crate) fn replace_reducer_state(&mut self, state: FocusaState) {
        self.cognitive_state = state;
    }
}

/// An event envelope with exact Workstream authority and the context resolved
/// from the canonical request envelope.
///
/// The legacy `FocusaEvent` remains the typed domain event.  A reducer event can
/// only be constructed through [`WorkstreamEvent::from_request`], so actor and
/// authority are present before the Workstream partition is selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamEvent {
    pub workstream: WorkstreamKey,
    pub context: WorkstreamContext,
    pub event: FocusaEvent,
    /// Optional optimistic-concurrency cursor.  When present it must equal the
    /// selected Workstream reducer revision; a stale cursor fails closed.
    #[serde(default)]
    pub expected_revision: Option<u64>,
}

impl WorkstreamEvent {
    /// Resolve the canonical request envelope before constructing a reducer
    /// event. No request-local fallback can bypass this path.
    pub fn from_request(
        request: WorkstreamRequestEnvelope,
        event: FocusaEvent,
    ) -> Result<Self, WorkstreamContextError> {
        let expected_revision = request.expected_revision;
        let context = WorkstreamContext::extract(request)?;
        let workstream = context.workstream.clone();
        Ok(Self {
            workstream,
            context,
            event,
            expected_revision,
        })
    }

    pub fn new(
        request: WorkstreamRequestEnvelope,
        event: FocusaEvent,
    ) -> Result<Self, WorkstreamContextError> {
        Self::from_request(request, event)
    }

    pub fn at_revision(
        mut request: WorkstreamRequestEnvelope,
        expected_revision: u64,
        event: FocusaEvent,
    ) -> Result<Self, WorkstreamContextError> {
        request.expected_revision = Some(expected_revision);
        Self::from_request(request, event)
    }

    pub fn workstream_id(&self) -> &WorkstreamId {
        &self.workstream.workstream_id
    }
}

/// Project-owned cognitive state partitions keyed by stable Workstream identity.
///
/// Infrastructure, runtime attachments, and session registries are intentionally
/// not accepted by this container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectState<W> {
    pub workstreams: BTreeMap<WorkstreamId, W>,
    /// Canonical project-owned active Workstream selection cursor.  This is
    /// not a daemon-global pointer and is never inferred from presentation.
    #[serde(default)]
    active_workstream: ActiveWorkstreamState,
}

impl<W> ProjectState<W> {
    pub fn new() -> Self {
        Self {
            workstreams: BTreeMap::new(),
            active_workstream: ActiveWorkstreamState::default(),
        }
    }

    pub fn active_workstream_state(&self) -> &ActiveWorkstreamState {
        &self.active_workstream
    }

    pub fn active_workstream(&self) -> Option<&WorkstreamKey> {
        self.active_workstream.active_workstream()
    }

    pub fn active_workstream_id(&self) -> Option<&WorkstreamId> {
        self.active_workstream.active_workstream_id()
    }

    pub(crate) fn active_workstream_state_mut(&mut self) -> &mut ActiveWorkstreamState {
        &mut self.active_workstream
    }

    pub fn register_workstream(
        &mut self,
        workstream_id: WorkstreamId,
        state: W,
    ) -> Result<(), WorkstreamStateError> {
        if self.workstreams.contains_key(&workstream_id) {
            return Err(WorkstreamStateError::AlreadyRegistered(workstream_id));
        }
        self.workstreams.insert(workstream_id, state);
        Ok(())
    }

    pub fn workstream(&self, workstream_id: &WorkstreamId) -> Result<&W, WorkstreamStateError> {
        self.workstreams
            .get(workstream_id)
            .ok_or_else(|| WorkstreamStateError::NotFound(workstream_id.clone()))
    }

    pub fn workstream_mut(
        &mut self,
        workstream_id: &WorkstreamId,
    ) -> Result<&mut W, WorkstreamStateError> {
        self.workstreams
            .get_mut(workstream_id)
            .ok_or_else(|| WorkstreamStateError::NotFound(workstream_id.clone()))
    }
}

/// The canonical ProjectState instantiation for the current migration slice.
pub type CanonicalProjectState = ProjectState<WorkstreamState>;

impl<W> Default for ProjectState<W> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::ScopeRef;

    fn canonical_workstream(id: &str) -> WorkstreamKey {
        let legacy_scope = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamKey::new(
            ScopeRef::project(legacy_scope).unwrap(),
            WorkstreamId::parse(id).unwrap(),
        )
    }

    #[test]
    fn one_project_routes_two_workstreams_to_distinct_state() {
        let planning = WorkstreamId::parse("planning").unwrap();
        let delivery = WorkstreamId::parse("delivery").unwrap();
        let mut project = ProjectState::new();
        project
            .register_workstream(planning.clone(), vec!["plan"])
            .unwrap();
        project
            .register_workstream(delivery.clone(), vec!["ship"])
            .unwrap();

        assert_eq!(project.workstream(&planning).unwrap(), &vec!["plan"]);
        assert_eq!(project.workstream(&delivery).unwrap(), &vec!["ship"]);
    }

    #[test]
    fn continuity_or_session_cannot_address_project_state() {
        let mut project = ProjectState::new();
        let workstream = WorkstreamId::parse("delivery").unwrap();
        project.register_workstream(workstream.clone(), 7).unwrap();

        assert_eq!(*project.workstream(&workstream).unwrap(), 7);
        assert!(
            project
                .workstream(&WorkstreamId::parse("session-a").unwrap())
                .is_err()
        );
    }

    #[test]
    fn registration_cannot_silently_replace_existing_state() {
        let id = WorkstreamId::parse("delivery").unwrap();
        let mut project = ProjectState::new();
        project.register_workstream(id.clone(), 7).unwrap();
        assert_eq!(
            project.register_workstream(id.clone(), 9),
            Err(WorkstreamStateError::AlreadyRegistered(id.clone()))
        );
        assert_eq!(*project.workstream(&id).unwrap(), 7);
    }

    #[test]
    fn workstream_state_uses_existing_typed_reducer_owners() {
        let state = WorkstreamState::new(canonical_workstream("delivery"));
        assert_eq!(state.key.workstream_id.as_str(), "delivery");
        assert_eq!(state.reducer_revision(), 0);
        assert_eq!(state.focus_stack().version, 0);
        assert!(state.focus_state().is_none());
        assert!(state.workpoints().records.is_empty());
        assert!(state.context_sources().is_empty());
    }
}

#[cfg(test)]
mod active_workstream_state {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_context::{ActorRef, ActorType, AuthorityContext};
    use crate::workstream_identity::{ContinuityId, ScopeRef};

    fn workstream(id: &str, fingerprint: &str) -> WorkstreamKey {
        let legacy_scope =
            LegacyScopeRef::project("project:focusa", "/workspace/focusa", "Focusa", fingerprint)
                .expect("valid project scope");
        WorkstreamKey::new(
            ScopeRef::project(legacy_scope).expect("canonical project scope"),
            WorkstreamId::parse(id).expect("WorkstreamId"),
        )
    }

    fn context(key: WorkstreamKey) -> WorkstreamContext {
        WorkstreamContext::extract(WorkstreamRequestEnvelope::for_workstream(
            key,
            ActorRef::new(ActorType::Desktop, "desktop:test").expect("actor"),
            AuthorityContext::canonical("authority:test", "verified active selection test"),
        ))
        .expect("exact Workstream context")
    }

    fn project(keys: &[WorkstreamKey]) -> CanonicalProjectState {
        let mut state = CanonicalProjectState::new();
        for key in keys {
            state
                .register_workstream(key.workstream_id.clone(), WorkstreamState::new(key.clone()))
                .expect("unique Workstream registration");
        }
        state
    }

    #[test]
    fn active_workstream_state_emits_canonical_event_and_receipt() {
        let planning = workstream("planning", "host-a:worktree-main");
        let delivery = workstream("delivery", "host-a:worktree-main");
        let state = project(&[planning, delivery.clone()]);
        let command =
            ActiveWorkstreamCommand::new(context(delivery.clone()), "idem:delivery", 0, 0);

        assert!(state.active_workstream().is_none());
        let reduced = WorkstreamState::reduce(state, command).expect("selection reduces");

        assert_eq!(reduced.new_state.active_workstream(), Some(&delivery));
        assert_eq!(
            reduced
                .new_state
                .active_workstream_id()
                .map(WorkstreamId::as_str),
            Some("delivery")
        );
        assert_eq!(reduced.new_state.active_workstream_state().revision(), 1);
        assert_eq!(
            reduced.new_state.active_workstream_state().fencing_token(),
            1
        );
        assert_eq!(reduced.event.active_workstream, delivery);
        assert_eq!(reduced.event.before_revision, 0);
        assert_eq!(reduced.event.after_revision, 1);
        assert_eq!(reduced.event.fencing_token, 1);
        assert!(reduced.receipt.canonical);
        assert!(!reduced.receipt.replayed);
        assert_eq!(reduced.receipt.after_revision, 1);
        assert_eq!(reduced.receipt.fencing_token, 1);
        assert_eq!(reduced.emitted_events, vec![reduced.event.clone()]);
    }

    #[test]
    fn desktop_request_is_only_a_request_until_reducer_accepts_it() {
        let delivery = workstream("delivery", "host-a:worktree-main");
        let state = project(std::slice::from_ref(&delivery));
        let mut request = WorkstreamRequestEnvelope::for_workstream(
            delivery.clone(),
            ActorRef::new(ActorType::Desktop, "desktop:test").expect("actor"),
            AuthorityContext::canonical("authority:test", "desktop request is verified"),
        );
        request.command_id = ACTIVE_WORKSTREAM_COMMAND_ID.to_string();
        request.idempotency_key = Some("idem:desktop-request".to_string());
        request.expected_revision = Some(0);
        request.expected_fencing_token = Some(0);

        let command = ActiveWorkstreamCommand::from_request(request).expect("request converts");
        assert!(state.active_workstream().is_none());
        let reduced = state
            .reduce(command)
            .expect("canonical reducer grants authority");
        assert_eq!(reduced.new_state.active_workstream(), Some(&delivery));
        assert!(reduced.receipt.canonical);
    }

    #[test]
    fn unknown_workstream_selection_fails_closed_without_mutation() {
        let registered = workstream("registered", "host-a:worktree-main");
        let unknown = workstream("unknown", "host-a:worktree-main");
        let state = project(std::slice::from_ref(&registered));
        let command = ActiveWorkstreamCommand::new(context(unknown), "idem:unknown", 0, 0);

        assert!(matches!(
            WorkstreamState::reduce(state.clone(), command),
            Err(ActiveWorkstreamError::UnknownWorkstream(id)) if id.as_str() == "unknown"
        ));
        assert!(state.active_workstream().is_none());
        assert_eq!(state.active_workstream_state().revision(), 0);
    }

    #[test]
    fn foreign_workstream_key_selection_fails_closed() {
        let local = workstream("delivery", "host-a:worktree-main");
        let foreign = workstream("delivery", "host-b:worktree-main");
        let state = project(std::slice::from_ref(&local));
        let command = ActiveWorkstreamCommand::new(context(foreign), "idem:foreign", 0, 0);

        assert!(matches!(
            WorkstreamState::reduce(state.clone(), command),
            Err(ActiveWorkstreamError::ForeignWorkstream)
        ));
        assert!(state.active_workstream().is_none());
    }

    #[test]
    fn cross_project_partition_selection_fails_closed() {
        let local = workstream("local", "host-a:worktree-main");
        let foreign = workstream("foreign", "host-b:worktree-other");
        let state = project(&[local.clone(), foreign]);
        let command = ActiveWorkstreamCommand::new(context(local), "idem:cross-project", 0, 0);

        assert!(matches!(
            WorkstreamState::reduce(state.clone(), command),
            Err(ActiveWorkstreamError::CrossProjectWorkstream)
        ));
        assert!(state.active_workstream().is_none());
    }

    #[test]
    fn stale_revision_and_fencing_cursors_fail_closed() {
        let planning = workstream("planning", "host-a:worktree-main");
        let delivery = workstream("delivery", "host-a:worktree-main");
        let state = project(&[planning.clone(), delivery.clone()]);
        let first = WorkstreamState::reduce(
            state,
            ActiveWorkstreamCommand::new(context(planning), "idem:planning", 0, 0),
        )
        .expect("first selection")
        .new_state;

        let stale_revision = WorkstreamState::reduce(
            first.clone(),
            ActiveWorkstreamCommand::new(context(delivery.clone()), "idem:stale-revision", 0, 0),
        );
        assert!(matches!(
            stale_revision,
            Err(ActiveWorkstreamError::StaleRevision {
                expected: 0,
                actual: 1
            })
        ));

        let stale_fence = WorkstreamState::reduce(
            first.clone(),
            ActiveWorkstreamCommand::new(context(delivery), "idem:stale-fence", 1, 0),
        );
        assert!(matches!(
            stale_fence,
            Err(ActiveWorkstreamError::StaleFencingToken {
                expected: 0,
                actual: 1
            })
        ));
        assert_eq!(first.active_workstream_state().revision(), 1);
        assert_eq!(first.active_workstream_state().fencing_token(), 1);
    }

    #[test]
    fn idempotent_replay_returns_receipt_without_second_mutation() {
        let delivery = workstream("delivery", "host-a:worktree-main");
        let state = project(std::slice::from_ref(&delivery));
        let command = ActiveWorkstreamCommand::new(context(delivery.clone()), "idem:replay", 0, 0);
        let first = WorkstreamState::reduce(state, command.clone())
            .expect("first selection")
            .new_state;
        let replay = WorkstreamState::reduce(first.clone(), command).expect("idempotent replay");

        assert_eq!(replay.new_state.active_workstream_state().revision(), 1);
        assert_eq!(replay.new_state.active_workstream(), Some(&delivery));
        assert!(replay.receipt.canonical);
        assert!(replay.receipt.replayed);
        assert_eq!(replay.event.before_revision, 0);
    }

    #[test]
    fn continuity_only_request_cannot_select_a_workstream() {
        let actor = ActorRef::new(ActorType::Desktop, "desktop:test").expect("actor");
        let authority = AuthorityContext::canonical("authority:test", "continuity is subordinate");
        let mut request = WorkstreamRequestEnvelope::new(None, None, actor, authority);
        request.continuity_id = Some(ContinuityId::parse("continuity-only").expect("continuity"));
        request.command_id = ACTIVE_WORKSTREAM_COMMAND_ID.to_string();
        request.idempotency_key = Some("idem:continuity-only".to_string());
        request.expected_revision = Some(0);
        request.expected_fencing_token = Some(0);

        assert!(matches!(
            ActiveWorkstreamCommand::from_request(request),
            Err(ActiveWorkstreamError::Context(
                WorkstreamContextError::MissingWorkstream
            ))
        ));
    }
}
