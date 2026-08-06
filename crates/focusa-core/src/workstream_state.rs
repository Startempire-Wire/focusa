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
use crate::workstream_identity::{WorkstreamId, WorkstreamKey};
use serde::{Deserialize, Serialize};
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
}

impl<W> ProjectState<W> {
    pub fn new() -> Self {
        Self {
            workstreams: BTreeMap::new(),
        }
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
