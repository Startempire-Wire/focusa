//! No-op adapter for repositories that do not host a work-item tracker.
//!
//! `validate_ref` and `submit` succeed locally; the closure is recorded
//! in the focusa audit log and the durable claim storage. The
//! provider is recorded as `none` so reviewers know the close was
//! local-only and did not mutate an external tracker.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::work_item::adapter::{ProviderAdapter, RegistryResult};
use crate::work_item::types::{
    ProviderCapabilities, WorkItem, WorkItemProvider, WorkItemRef, WorkItemStatus,
};

pub struct NoneAdapter {
    /// In-memory map of ref id -> cached status. Default state is
    /// `Closed` so a `submit` on this adapter reports the post-submit
    /// status correctly.
    states: Arc<RwLock<std::collections::HashMap<String, WorkItem>>>,
}

impl NoneAdapter {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn arc() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for NoneAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderAdapter for NoneAdapter {
    fn provider(&self) -> WorkItemProvider {
        WorkItemProvider::None
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            can_resolve: true,
            can_validate_ref: true,
            can_prepare: true,
            can_submit: true,
            can_reconcile: true,
            supports_override: true,
            // No external mutation, but local record still mutates.
            mutable: false,
        }
    }

    async fn detect(&self) -> bool {
        // The none adapter is always available — there is nothing to
        // detect. The lifecycle treats this as "provider already
        // configured".
        true
    }

    async fn resolve(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        let map = self.states.read().await;
        Ok(map
            .get(&work_item.provider_item_id)
            .cloned()
            .unwrap_or_else(|| WorkItem {
                provider: self.provider(),
                provider_item_id: work_item.provider_item_id.clone(),
                provider_status: WorkItemStatus::Open,
                title: format!("local-only: {}", work_item.provider_item_id),
                url: work_item.external_url.clone(),
                revision: None,
            }))
    }

    async fn validate_ref(&self, work_item: &WorkItemRef) -> RegistryResult<()> {
        if work_item.provider_item_id.trim().is_empty() {
            return Err(crate::work_item::adapter::RegistryError::ProviderError {
                provider: self.provider(),
                stage: "validate_ref",
                why: "none adapter requires a non-empty provider_item_id".into(),
            });
        }
        Ok(())
    }

    async fn prepare(&self, work_item: &WorkItemRef, summary: &str) -> RegistryResult<String> {
        Ok(format!(
            "{}\n[local-only closure via focusa none adapter; no external mutation]",
            summary
        ))
    }

    async fn submit(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        let mut map = self.states.write().await;
        let updated = WorkItem {
            provider: self.provider(),
            provider_item_id: work_item.provider_item_id.clone(),
            provider_status: WorkItemStatus::Closed,
            title: format!("closed (local-only): {}", work_item.provider_item_id),
            url: work_item.external_url.clone(),
            revision: Some("local".into()),
        };
        map.insert(work_item.provider_item_id.clone(), updated.clone());
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ref_id(s: &str) -> WorkItemRef {
        WorkItemRef {
            provider: WorkItemProvider::None,
            provider_item_id: s.into(),
            project_root: PathBuf::from("/tmp"),
            external_url: None,
        }
    }

    #[tokio::test]
    async fn none_adapter_local_only_lifecycle() {
        let a = NoneAdapter::new();
        assert!(a.detect().await);
        a.validate_ref(&ref_id("abcd-1234")).await.unwrap();
        let summary = a.prepare(&ref_id("abcd-1234"), "shipped").await.unwrap();
        assert!(summary.contains("shipped"));
        let after = a.submit(&ref_id("abcd-1234")).await.unwrap();
        assert_eq!(after.provider_status, WorkItemStatus::Closed);
        let resolved = a.resolve(&ref_id("abcd-1234")).await.unwrap();
        assert_eq!(resolved.provider_status, WorkItemStatus::Closed);
    }

    #[tokio::test]
    async fn none_adapter_rejects_empty_ref() {
        let a = NoneAdapter::new();
        let err = a.validate_ref(&ref_id("")).await.unwrap_err();
        let s = format!("{err}");
        assert!(s.contains("non-empty"), "{s}");
    }
}
