//! `ProviderAdapter` trait + `ProviderRegistry` (Spec 116 §7.1, §7.6).
//!
// Every concrete provider (bd, linear, asana, github, gitlab, jira,
// none) implements this trait. The lifecycle depends only on the trait
//! — no provider-specific code lives outside `adapters/`.

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::work_item::types::{
    ProviderCapabilities, WorkItem, WorkItemProvider, WorkItemQuery, WorkItemRef,
};

/// Behaviors shared by all provider adapters.
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Alias for `WorkItemProvider` so downstream code can refer to it by
/// the spec 116 name.
pub type ProviderKind = WorkItemProvider;

/// Errors produced by the registry or by adapter construction.
#[derive(Clone, Debug)]
pub enum RegistryError {
    /// The named provider is not installed (e.g. `gh` CLI is missing
    /// when the GitHub adapter was selected).
    ProviderNotInstalled {
        provider: WorkItemProvider,
        missing: Vec<String>,
    },
    /// The provider is installed but the configured credentials are
    /// missing or invalid.
    CredentialsInvalid {
        provider: WorkItemProvider,
        why: String,
    },
    /// A required capability is not supported by the selected adapter.
    CapabilityUnsupported {
        provider: WorkItemProvider,
        capability: &'static str,
    },
    /// The provider returned an error during a stage. The `why` field
    /// carries the provider's own error string.
    ProviderError {
        provider: WorkItemProvider,
        stage: &'static str,
        why: String,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderNotInstalled { provider, missing } => write!(
                f,
                "provider {provider} not installed; missing: {}",
                missing.join(", ")
            ),
            Self::CredentialsInvalid { provider, why } => {
                write!(f, "provider {provider} credentials invalid: {why}")
            }
            Self::CapabilityUnsupported {
                provider,
                capability,
            } => write!(
                f,
                "provider {provider} does not support capability: {capability}"
            ),
            Self::ProviderError {
                provider,
                stage,
                why,
            } => {
                write!(f, "provider {provider} error in {stage}: {why}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// The trait every concrete provider implements. The lifecycle calls
/// these methods in a fixed order: detect -> resolve -> prepare ->
/// submit -> reconcile. `WorkItemProvider` is included in every
/// method's signature so adapters that wrap multiple providers
/// (e.g. a future multi-tenant linear-cli) can dispatch.
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Which provider this adapter implements.
    fn provider(&self) -> WorkItemProvider;

    /// What this adapter can do. The lifecycle refuses to call
    /// unsupported methods (e.g. `submit` on a read-only adapter).
    fn capabilities(&self) -> ProviderCapabilities;

    /// Cheap detection: is the underlying CLI / API reachable? Returns
    /// `true` only when the adapter is ready to operate.
    async fn detect(&self) -> bool;

    /// Look up a work item in the provider. Returns the current status
    /// so `reconcile` can verify the post-submit state.
    async fn resolve(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem>;

    /// List provider snapshots for core dependency evaluation. Providers do
    /// not decide readiness or ordering; Work Loop calls the core scheduler.
    async fn list(&self, _query: &WorkItemQuery) -> RegistryResult<Vec<WorkItem>> {
        Err(RegistryError::CapabilityUnsupported {
            provider: self.provider(),
            capability: "list_work_items",
        })
    }

    /// Validate that a `provider_item_id` exists and is in a state that
    /// can be closed (e.g. not already closed, not archived).
    async fn validate_ref(&self, work_item: &WorkItemRef) -> RegistryResult<()>;

    /// Prepare a closure claim for submission. Returns the human-
    /// readable summary the provider will write into the closed item.
    async fn prepare(
        &self,
        work_item: &WorkItemRef,
        closure_summary: &str,
    ) -> RegistryResult<String>;

    /// Submit the closure. Returns the post-submit `WorkItem` snapshot
    /// so the lifecycle can run `reconcile` against it.
    async fn submit(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem>;

    /// Optional post-submit reconciliation. The default implementation
    /// is `Ok(self.resolve(work_item).await?)` and is sufficient for
    /// providers whose `submit` is already idempotent.
    async fn reconcile(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        self.resolve(work_item).await
    }
}

/// Thread-safe registry mapping `WorkItemProvider` to its adapter.
#[derive(Clone, Default)]
pub struct ProviderRegistry {
    by_kind: BTreeMap<WorkItemProvider, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    /// Construct an empty registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Register a concrete adapter. The last registration for a given
    /// `WorkItemProvider` wins, which makes it easy to override the
    /// default adapter in tests.
    pub fn register(&mut self, adapter: Arc<dyn ProviderAdapter>) -> &mut Self {
        self.by_kind.insert(adapter.provider(), adapter);
        self
    }

    /// Get the adapter for a provider kind, if registered.
    pub fn get(&self, kind: WorkItemProvider) -> Option<Arc<dyn ProviderAdapter>> {
        self.by_kind.get(&kind).cloned()
    }

    /// Iterate the registered adapters in provider-name order.
    pub fn iter(&self) -> impl Iterator<Item = (WorkItemProvider, Arc<dyn ProviderAdapter>)> + '_ {
        self.by_kind.iter().map(|(k, a)| (*k, a.clone()))
    }

    /// Number of registered adapters.
    pub fn len(&self) -> usize {
        self.by_kind.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_kind.is_empty()
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.by_kind.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubAdapter {
        kind: ProviderKind,
        caps: ProviderCapabilities,
        detect_result: bool,
    }

    #[async_trait]
    impl ProviderAdapter for StubAdapter {
        fn provider(&self) -> ProviderKind {
            self.kind
        }
        fn capabilities(&self) -> ProviderCapabilities {
            self.caps
        }
        async fn detect(&self) -> bool {
            self.detect_result
        }
        async fn resolve(&self, _w: &WorkItemRef) -> RegistryResult<WorkItem> {
            Ok(WorkItem {
                provider: self.kind,
                provider_item_id: "id".into(),
                project_root: std::path::PathBuf::from("/tmp/project"),
                provider_status: crate::work_item::types::WorkItemStatus::Open,
                title: "stub".into(),
                priority: 0,
                parent: None,
                dependencies: vec![],
                acceptance_criteria: vec![],
                spec_refs: vec![],
                blocked_reason: None,
                url: None,
                revision: None,
            })
        }
        async fn validate_ref(&self, _w: &WorkItemRef) -> RegistryResult<()> {
            Ok(())
        }
        async fn prepare(&self, _w: &WorkItemRef, _s: &str) -> RegistryResult<String> {
            Ok("prepared".into())
        }
        async fn submit(&self, w: &WorkItemRef) -> RegistryResult<WorkItem> {
            self.resolve(w).await
        }
    }

    fn ref_for(kind: ProviderKind) -> WorkItemRef {
        WorkItemRef {
            provider: kind,
            provider_item_id: "x".into(),
            project_root: "/tmp".into(),
            external_url: None,
        }
    }

    #[tokio::test]
    async fn registry_register_and_get() {
        let mut r = ProviderRegistry::empty();
        assert!(r.is_empty());
        r.register(Arc::new(StubAdapter {
            kind: ProviderKind::Bd,
            caps: ProviderCapabilities::full(),
            detect_result: true,
        }));
        r.register(Arc::new(StubAdapter {
            kind: ProviderKind::None,
            caps: ProviderCapabilities::none(),
            detect_result: false,
        }));
        assert_eq!(r.len(), 2);
        let bd = r.get(ProviderKind::Bd).unwrap();
        assert_eq!(bd.provider(), ProviderKind::Bd);
        assert!(bd.detect().await);
        let n = r.get(ProviderKind::None).unwrap();
        assert!(!n.detect().await);
    }

    #[tokio::test]
    async fn default_reconcile_delegates_to_resolve() {
        let r = ProviderRegistry::empty();
        let a = r.get(ProviderKind::Bd); // not registered
        assert!(a.is_none());
        let mut r2 = ProviderRegistry::empty();
        r2.register(Arc::new(StubAdapter {
            kind: ProviderKind::Bd,
            caps: ProviderCapabilities::full(),
            detect_result: true,
        }));
        let a = r2.get(ProviderKind::Bd).unwrap();
        let wi = a.reconcile(&ref_for(ProviderKind::Bd)).await.unwrap();
        assert_eq!(wi.provider, ProviderKind::Bd);
    }
}
