//! Workstream-scoped state store (#125 slice 3, migration foundation).
//!
//! The singleton-elimination migration: `FocusaState` partitions per
//! workstream, keyed by the canonical scope key from
//! `focusa_core::workstream_root::workstream_scope_key`. The global state
//! remains canonical until route migration completes; this store is the
//! additive integration point routes opt into.

use focusa_core::types::FocusaState;
use focusa_core::workstream_root::workstream_scope_key;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct WorkstreamStateStore {
    states: RwLock<HashMap<String, Arc<RwLock<FocusaState>>>>,
}

impl WorkstreamStateStore {
    /// Resolve (or create) the partitioned state for a project root +
    /// continuity pair. Every mutation must name the workstream.
    pub async fn get_or_create(
        &self,
        project_root: &str,
        continuity_id: &str,
    ) -> Arc<RwLock<FocusaState>> {
        let key = workstream_scope_key(project_root, continuity_id);
        let mut states = self.states.write().await;
        states
            .entry(key)
            .or_insert_with(|| Arc::new(RwLock::new(FocusaState::default())))
            .clone()
    }

    /// Inspect how many partitioned states exist (observability).
    pub async fn len(&self) -> usize {
        self.states.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.states.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn partitions_by_workstream_scope() {
        let store = WorkstreamStateStore::default();
        let a = store.get_or_create("/root/ws1", "cont-1").await;
        let b = store.get_or_create("/root/ws1", "cont-1").await;
        let c = store.get_or_create("/root/ws2", "cont-1").await;
        assert!(Arc::ptr_eq(&a, &b), "same scope must resolve to the same state");
        assert!(!Arc::ptr_eq(&a, &c), "different workstreams must partition");
        assert_eq!(store.len().await, 2);
    }

    #[tokio::test]
    async fn scoped_writes_never_cross_partitions() {
        let store = WorkstreamStateStore::default();
        let a = store.get_or_create("/root/ws1", "cont-1").await;
        let b = store.get_or_create("/root/ws2", "cont-1").await;
        {
            let mut state = a.write().await;
            state.workpoint.active_workpoint_id = Some(uuid::Uuid::now_v7());
        }
        {
            let state = b.read().await;
            assert!(state.workpoint.active_workpoint_id.is_none());
        }
    }
}
