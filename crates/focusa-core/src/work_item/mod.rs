//! Provider-Neutral Work Item Closure Authority (Spec 116).
//!
//! Focusa validates closure truth; providers store and display the closure.
//! bd is adapter #1. Asana, Linear, GitHub Issues, GitLab, Jira, and
//! future providers plug in via the [`ProviderAdapter`] trait.
//!
//! ## Module map
//!
// - [`types`]   — Provider, WorkItemRef, EvidenceCitation, ClosureClaim,
//                 ClaimStatus, ClosureKind, ClosureProfile, closure envelopes.
// - [`adapter`] — `ProviderAdapter` trait + `ProviderRegistry` + capability flags.
// - [`evidence`] — Real evidence verifiers (code, spec, test, endpoint,
//                  artifact, workpoint, ci, deploy).
// - [`lifecycle`] — Prepare -> Validate -> Authorize -> Submit -> Reconcile.
// - [`policy`] — ClosurePolicy + TOML loader + pre-built profiles.
// - [`audit`] — closure-audit.jsonl append-only audit.
// - [`storage`] — durable claim storage at `~/.focusa/state/closure-claims/<id>.json`.
// - [`adapters`] — concrete adapters (bd, linear, asana, github, gitlab, jira, none).
//!
// All blocks and failures use the typed envelope [`types::ClosureBlock`].

#![forbid(unsafe_code)]

pub mod adapter;
pub mod adapters;
pub mod audit;
pub mod evidence;
pub mod lifecycle;
pub mod policy;
pub mod scheduler;
pub mod storage;
pub mod sweeper;
pub mod temporal_policy;
pub mod types;

pub use adapter::{ProviderAdapter, ProviderRegistry, RegistryError, RegistryResult};
pub use adapters::{BdAdapter, NoneAdapter};
pub use audit::{ClosureAuditEvent, ClosureAuditLog};
pub use evidence::{
    CodeVerifier, EndpointVerifier, SpecVerifier, TestVerifier, VerifyResult, WorkpointVerifier,
};
pub use lifecycle::{
    AuthorizeResult, Lifecycle, PrepareResult, ReconcileResult, SubmitResult, ValidateResult,
};
pub use policy::{ACTIVE_PROFILE_RELEASE_PROOF, ClosurePolicy, ClosureProfile, ProfileRule};
pub use scheduler::{BlockedWorkItem, WorkItemReadiness, evaluate_readiness, select_next_ready};
pub use storage::{ClaimStorage, ClaimStorageError, ClaimStorageResult};
pub use sweeper::{ProviderSweepReport, ProviderSweeper, SweepIncident};
pub use types::{
    ClaimStatus, ClosureAuthorityContext, ClosureBlock, ClosureClaim, ClosureClaimBuilder,
    ClosureError, ClosureKind, EvidenceCitation, EvidenceKind, LifecycleStage,
    ProviderCapabilities, RECLAIMED_BY_OPERATOR, WorkItem, WorkItemProvider, WorkItemQuery,
    WorkItemRef, WorkItemStatus,
};
