//! Spec 158 project-to-Workstream state partition.
//!
//! The complete ProjectState fields migrate in bounded slices. This module owns the
//! canonical partition now: cognitive state is addressable only by durable
//! `WorkstreamId`, never by continuity, session, attachment, or recency.

use crate::workstream_identity::WorkstreamId;
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

/// Project-owned cognitive state partition keyed by stable Workstream identity.
///
/// `W` is the bounded Workstream state payload being migrated. Infrastructure and
/// attachment registries are intentionally not accepted by this container.
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

impl<W> Default for ProjectState<W> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(project
            .workstream(&WorkstreamId::parse("session-a").unwrap())
            .is_err());
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
}
