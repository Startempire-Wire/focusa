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
            .entry(key.clone())
            .or_insert_with(|| {
                // Slice-2 durability: a persisted partition (state.sqlite
                // via partition_paths) rehydrates on first access — a
                // daemon restart resumes the exact workstream root.
                let mut initial = FocusaState::default();
                let data_dir = std::env::var("FOCUSA_DATA_DIR")
                    .unwrap_or_else(|_| "data/.focusa".to_string());
                let partitions = focusa_core::workstream_root::partition_paths(
                    std::path::Path::new(&data_dir),
                    &key,
                );
                if let Ok(raw) = std::fs::read_to_string(&partitions.state_ref) {
                    if let Ok(state) = serde_json::from_str::<FocusaState>(&raw) {
                        initial = state;
                    }
                }
                Arc::new(RwLock::new(initial))
            })
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

/// Shared migration helper (#125): resolve the partitioned workstream state
/// for a scope, falling back to the global state for unmigrated scopes.
/// Owned guards keep lifetimes simple across both branches.
pub async fn scoped_focusa_read(
    state: Arc<crate::server::AppState>,
    scope: &crate::scope::ScopeContext,
) -> tokio::sync::OwnedRwLockReadGuard<FocusaState> {
    match (&scope.project_root, &scope.continuity_id) {
        (Some(root), Some(continuity)) => {
            let partition = state.workstream_states.get_or_create(root, continuity).await;
            partition.read_owned().await
        }
        _ => state.focusa.clone().read_owned().await,
    }
}

/// WorkstreamKey-aware variant: work-loop handlers carry a typed key.
pub async fn scoped_focusa_read_workstream(
    state: Arc<crate::server::AppState>,
    key: &focusa_core::scoped_state::WorkstreamKey,
) -> tokio::sync::OwnedRwLockReadGuard<FocusaState> {
    let partition = state
        .workstream_states
        .get_or_create(
            key.root_scope.root_path.to_string_lossy().as_ref(),
            &key.continuity_id,
        )
        .await;
    partition.read_owned().await
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

/// Slice-2 write side (docs/164 invariant 1): a scoped mutation must
/// name its root AND land durably in the workstream partition. The
/// global state remains the compatibility projection (unmigrated
/// consumers), but the partition is the durable per-workstream truth.
pub async fn scoped_write_through(
    state: Arc<crate::server::AppState>,
    project_root: &str,
    continuity_id: &str,
    new_state: FocusaState,
) {
    let key = workstream_scope_key(project_root, continuity_id);
    // 1) In-memory partition update.
    {
        let partition = state.workstream_states.get_or_create(project_root, continuity_id).await;
        *partition.write().await = new_state.clone();
    }
    // 2) Durable per-workstream persistence (partition_paths state.sqlite).
    let data_dir = state.config.data_dir.clone();
    let _ = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let partitions = focusa_core::workstream_root::partition_paths(
            std::path::Path::new(&data_dir),
            &key,
        );
        if let Some(parent) = std::path::Path::new(&partitions.state_ref).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&new_state)?;
        std::fs::write(&partitions.state_ref, json)?;
        Ok(())
    })
    .await;
}
