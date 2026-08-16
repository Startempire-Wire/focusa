//! Core types for the Closure Authority (Spec 116 §7).
//!
// Every datum here is also serialized to / from the durable closure
//! claim file at `~/.focusa/state/closure-claims/<claim_id>.json`
// and the append-only audit log at `~/.focusa/state/closure-audit.jsonl`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Sentinel string written into `actor_id` when an operator breaks the
/// policy via `--override`. Reconciliation surfaces this string in the
/// audit log so reviewers can grep for overrides deterministically.
pub const RECLAIMED_BY_OPERATOR: &str = "operator.override";

/// Provider kind. Provider-neutral: bd is one of many.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemProvider {
    /// Beads (`bd`). Adapter #1, not the sole authority.
    Bd,
    /// Linear (linear.app). REST + OAuth.
    Linear,
    /// Asana. REST + Personal Access Token.
    Asana,
    /// GitHub Issues. REST + GraphQL.
    Github,
    /// GitLab Issues. REST.
    Gitlab,
    /// Jira Cloud (Atlassian). REST + basic auth.
    Jira,
    /// No provider configured for this work item. Used for repos that
    /// do not host a tracker. The `none` adapter is a no-op submit
    /// that records the closure locally without mutating any backend.
    #[default]
    None,
}

impl fmt::Display for WorkItemProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Bd => "bd",
            Self::Linear => "linear",
            Self::Asana => "asana",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Jira => "jira",
            Self::None => "none",
        };
        f.write_str(s)
    }
}

/// The kind of work that the closure claim covers. The closure_kind
/// determines which pre-built evidence profile is selected by default
/// (see [`crate::policy::ClosureProfile`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureKind {
    /// Code change (tests + endpoint proof recommended).
    Code,
    /// Documentation change (spec citation required).
    Docs,
    /// Deployment / infra change (deploy health endpoint required).
    Deploy,
    /// Investigation / research (no code; spec + spec.md required).
    Investigation,
    /// Administrative / process change (no code; acceptable with admin
    /// override only).
    NoCode,
    /// Anything that does not fit the above. Closure must include a
    /// justification string in `closure_summary`.
    Admin,
}

impl fmt::Display for ClosureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Code => "code",
            Self::Docs => "docs",
            Self::Deploy => "deploy",
            Self::Investigation => "investigation",
            Self::NoCode => "no_code",
            Self::Admin => "admin",
        };
        f.write_str(s)
    }
}

/// Lifecycle status of a closure claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    /// Drafted by `prepare`; not yet validated.
    Draft,
    /// Every required evidence citation verified; ready to authorize.
    Valid,
    /// Operator / authorized actor signed off; ready to submit.
    Authorized,
    /// Provider mutation was attempted.
    Submitted,
    /// Provider reported the work item is in the expected end state.
    Reconciled,
    /// At least one citation failed or the policy rejected the claim.
    Blocked,
    /// Claim aged out (`expires_at < now`); re-prepare required.
    Expired,
}

impl fmt::Display for ClaimStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Draft => "draft",
            Self::Valid => "valid",
            Self::Authorized => "authorized",
            Self::Submitted => "submitted",
            Self::Reconciled => "reconciled",
            Self::Blocked => "blocked",
            Self::Expired => "expired",
        };
        f.write_str(s)
    }
}

/// Reference to a work item in a specific provider.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkItemRef {
    /// Which provider stores this work item.
    pub provider: WorkItemProvider,
    /// Provider-local identifier (e.g. `focusa-glny` for bd,
    /// `ISS-123` for Jira, `LIN-456` for Linear).
    pub provider_item_id: String,
    /// Repository / workspace root the work item belongs to.
    pub project_root: PathBuf,
    /// Optional external URL (e.g. https://linear.app/...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
}

/// Provider-side work item snapshot returned by
/// `ProviderAdapter::resolve`. Used by `reconcile`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItem {
    pub provider: WorkItemProvider,
    pub provider_item_id: String,
    /// Canonical project scope of this provider snapshot. Scheduler identity is
    /// `(provider, project_root, provider_item_id)`, never the provider ID alone.
    #[serde(default)]
    pub project_root: PathBuf,
    pub provider_status: WorkItemStatus,
    pub title: String,
    /// Lower numbers are scheduled first. Provider-specific priorities are
    /// normalized by the adapter; zero is the highest default priority.
    #[serde(default)]
    pub priority: i32,
    /// Optional parent in the provider-neutral execution graph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<WorkItemRef>,
    /// WorkItems that must be Done/Closed before this item is ready.
    #[serde(default)]
    pub dependencies: Vec<WorkItemRef>,
    /// Spec-derived acceptance criteria. Providers only persist projections.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    /// Normative specification references governing this item.
    #[serde(default)]
    pub spec_refs: Vec<String>,
    /// Typed provider-reported blocker, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    /// SHA / revision / version returned by the provider when meaningful.
    #[serde(default)]
    pub revision: Option<String>,
}

impl WorkItem {
    pub fn reference(&self) -> WorkItemRef {
        WorkItemRef {
            provider: self.provider,
            provider_item_id: self.provider_item_id.clone(),
            project_root: self.project_root.clone(),
            external_url: self.url.clone(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.provider_status,
            WorkItemStatus::Done | WorkItemStatus::Closed | WorkItemStatus::Cancelled
        )
    }
}

/// Provider-neutral graph query used by Work Loop. No provider command or
/// identifier is permitted to become scheduler authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItemQuery {
    pub project_root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<WorkItemRef>,
    #[serde(default = "default_work_item_query_limit")]
    pub limit: usize,
}

fn default_work_item_query_limit() -> usize {
    100
}

/// Work item status returned by the provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemStatus {
    Open,
    InProgress,
    Blocked,
    Done,
    Closed,
    Cancelled,
    Unknown,
}

impl fmt::Display for WorkItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Closed => "closed",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        };
        f.write_str(s)
    }
}

/// The seven evidence kinds defined by Spec 116 §7.3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Source code change. Path relative to project root.
    Code,
    /// Spec / design doc. Path to the spec file.
    Spec,
    /// Test file. Path; the verifier may optionally run the test.
    Test,
    /// Live HTTP endpoint (`/v1/health`, etc.).
    Endpoint,
    /// Build artifact (binary, tarball, signature).
    Artifact,
    /// Workpoint evidence reference (already tracked by the daemon).
    Workpoint,
    /// CI run reference (GitHub Actions, GitLab CI, etc.).
    Ci,
    /// Deployment reference (live daemon, deployed commit, etc.).
    Deploy,
}

impl fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Code => "code",
            Self::Spec => "spec",
            Self::Test => "test",
            Self::Endpoint => "endpoint",
            Self::Artifact => "artifact",
            Self::Workpoint => "workpoint",
            Self::Ci => "ci",
            Self::Deploy => "deploy",
        };
        f.write_str(s)
    }
}

/// A single evidence citation. The lifecycle validator runs the
/// matching `EvidenceVerifier` and records the result in `VerifyResult`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceCitation {
    /// Kind of evidence.
    pub kind: EvidenceKind,
    /// Reference payload. Examples:
    /// - code:    `crates/focusa-core/src/work_item/types.rs:42`
    /// - spec:    `docs/116-provider-neutral-work-item-closure-authority-spec.md#7.3`
    /// - test:    `tests/spec_closure_authority_e2e_static_test.sh`
    /// - endpoint: `GET http://127.0.0.1:8787/v1/health -> 200 OK`
    /// - artifact: `target/release/focusa-daemon sha256:abc...`
    /// - workpoint: `019f3b0f-7068-7a11-aabc-26969ee39dde`
    /// - ci:      `gh run 28845429408 pass`
    /// - deploy:   `version 0.9.74-dev uptime_ms 1494 ok=true`
    #[serde(rename = "ref", alias = "ref_")]
    pub ref_: String,
    /// Optional 1-based line range for code/spec artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Optional end line for code/spec artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    /// Required citation. If false, the claim still validates when
    /// this one fails (informational only).
    #[serde(default = "default_required")]
    pub required: bool,
    /// Result recorded by the verifier. Populated by `validate` so the
    /// claim JSON is self-describing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// `true` iff the verifier passed during `validate`.
    #[serde(default)]
    pub verified: bool,
}

fn default_required() -> bool {
    true
}

/// Typed envelope used by every blocked path in the closure lifecycle
/// (matches Spec 116 envelope shape; see `focusa.closure_block.v1`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClosureBlock {
    pub status: String,        // "blocked"
    pub canonical: bool,       // false
    pub degraded: bool,        // true
    pub failure_class: String, // "validation_rejected" | "policy_denied" | ...
    pub code: String,
    pub why: String,
    pub recovery_hint: String,
    pub next_tools: Vec<String>,
    pub claim_id: Option<String>,
    pub stage: LifecycleStage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_index: Option<usize>,
}

impl ClosureBlock {
    /// Construct a typed block with the canonical envelope shape.
    pub fn new(
        failure_class: impl Into<String>,
        code: impl Into<String>,
        why: impl Into<String>,
        recovery_hint: impl Into<String>,
        stage: LifecycleStage,
    ) -> Self {
        Self {
            status: "blocked".into(),
            canonical: false,
            degraded: true,
            failure_class: failure_class.into(),
            code: code.into(),
            why: why.into(),
            recovery_hint: recovery_hint.into(),
            next_tools: vec![
                "focusa work-item closure prepare".into(),
                "focusa work-item closure validate".into(),
                "focusa doctor closure".into(),
            ],
            claim_id: None,
            stage,
            citation_index: None,
        }
    }
}

/// Lifecycle stage at which a block was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Prepare,
    Validate,
    Authorize,
    Submit,
    Reconcile,
}

impl fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Prepare => "prepare",
            Self::Validate => "validate",
            Self::Authorize => "authorize",
            Self::Submit => "submit",
            Self::Reconcile => "reconcile",
        };
        f.write_str(s)
    }
}

/// Capability flags exposed by a provider adapter. The lifecycle uses
/// these to decide which stages are possible (e.g. a `None` adapter
/// reports `Mutable = false` so submit is a local-only operation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub can_resolve: bool,
    pub can_validate_ref: bool,
    pub can_prepare: bool,
    pub can_submit: bool,
    pub can_reconcile: bool,
    pub supports_override: bool,
    /// When true, submit mutates state in an external tracker.
    pub mutable: bool,
}

/// Per-adapter capabilities. The default is "all capabilities" for an
/// adapter that wants to declare everything at once. Most real adapters
/// will return a narrower set.
impl ProviderCapabilities {
    pub fn full() -> Self {
        Self {
            can_resolve: true,
            can_validate_ref: true,
            can_prepare: true,
            can_submit: true,
            can_reconcile: true,
            supports_override: true,
            mutable: true,
        }
    }
    pub fn readonly() -> Self {
        Self {
            can_resolve: true,
            can_validate_ref: true,
            can_prepare: false,
            can_submit: false,
            can_reconcile: true,
            supports_override: false,
            mutable: false,
        }
    }
    pub fn none() -> Self {
        Self::default()
    }
}

/// Typed authority scope required when autonomous execution requests closure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureAuthorityContext {
    pub continuity_id: String,
    pub workpoint_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_id: Option<String>,
}

/// Top-level typed closure claim (Spec 116 §7.4). Serialized verbatim
/// to `~/.focusa/state/closure-claims/<claim_id>.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClosureClaim {
    pub schema: String, // "focusa.closure_claim.v1"
    pub claim_id: String,
    pub idempotency_key: String,

    pub work_item: WorkItemRef,
    pub project_root: PathBuf,
    pub continuity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workpoint_id: Option<String>,

    pub actor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_id: Option<String>,

    pub closure_summary: String,
    pub closure_kind: ClosureKind,

    pub code_refs: Vec<EvidenceCitation>,
    pub spec_refs: Vec<EvidenceCitation>,
    pub proof_refs: Vec<EvidenceCitation>,
    pub deploy_refs: Vec<EvidenceCitation>,
    pub artifact_refs: Vec<EvidenceCitation>,

    pub policy: String, // active profile name, e.g. "release_proof"
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ClaimStatus,
    /// Optional override reason. Populated when the claim is closed
    /// via `--override --reason=...`. The audit log records the actor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,
    /// Machine fingerprint for seat enforcement (V3 from the install
    /// gap audit). Populated by the closure lifecycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
}

impl ClosureClaim {
    /// Total number of evidence citations across every kind, used by
    /// profiles and progress displays.
    pub fn evidence_count(&self) -> usize {
        self.code_refs.len()
            + self.spec_refs.len()
            + self.proof_refs.len()
            + self.deploy_refs.len()
            + self.artifact_refs.len()
    }

    /// Whether the claim has expired and must be re-prepared.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }

    /// Whether the actor issued an override (i.e. broke policy rather
    /// than producing real evidence).
    pub fn is_override(&self) -> bool {
        self.override_reason.is_some() || self.actor_id == RECLAIMED_BY_OPERATOR
    }
}

/// Builder for `ClosureClaim`. The lifecycle creates one in `prepare`,
/// mutates fields in `validate` / `authorize`, and persists on every
/// stage transition.
#[derive(Default)]
pub struct ClosureClaimBuilder {
    inner: Option<ClosureClaim>,
}

impl ClosureClaimBuilder {
    pub fn new(claim: ClosureClaim) -> Self {
        Self { inner: Some(claim) }
    }
    pub fn status(mut self, status: ClaimStatus) -> Self {
        if let Some(c) = self.inner.as_mut() {
            c.status = status;
        }
        self
    }
    pub fn override_reason(mut self, reason: impl Into<String>) -> Self {
        if let Some(c) = self.inner.as_mut() {
            c.override_reason = Some(reason.into());
            c.actor_id = RECLAIMED_BY_OPERATOR.to_string();
        }
        self
    }
    pub fn build(self) -> Option<ClosureClaim> {
        self.inner
    }
}

/// Convenience error type returned by every closure operation. The
/// lifecycle wraps these into `ClosureBlock` envelopes.
#[derive(Clone, Debug)]
pub struct ClosureError {
    pub stage: LifecycleStage,
    pub failure_class: String,
    pub code: String,
    pub why: String,
    pub recovery_hint: String,
}

impl ClosureError {
    pub fn into_block(self) -> ClosureBlock {
        ClosureBlock::new(
            self.failure_class,
            self.code,
            self.why,
            self.recovery_hint,
            self.stage,
        )
    }
}

impl std::fmt::Display for ClosureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "closure error at {} ({}): {} — {}",
            self.stage, self.failure_class, self.code, self.why
        )
    }
}

impl std::error::Error for ClosureError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref() -> WorkItemRef {
        WorkItemRef {
            provider: WorkItemProvider::Bd,
            provider_item_id: "focusa-glny".into(),
            project_root: PathBuf::from("/home/example/project"),
            external_url: None,
        }
    }

    fn sample_citation(kind: EvidenceKind, ref_: &str) -> EvidenceCitation {
        EvidenceCitation {
            kind,
            ref_: ref_.into(),
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        }
    }

    #[test]
    fn provider_display_matches_serde_name() {
        for p in [
            WorkItemProvider::Bd,
            WorkItemProvider::Linear,
            WorkItemProvider::Asana,
            WorkItemProvider::Github,
            WorkItemProvider::Gitlab,
            WorkItemProvider::Jira,
            WorkItemProvider::None,
        ] {
            let s = p.to_string();
            assert!(matches!(
                s.as_str(),
                "bd" | "linear" | "asana" | "github" | "gitlab" | "jira" | "none"
            ));
        }
    }

    #[test]
    fn closure_claim_roundtrip_json() {
        let claim = ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: "claim_test_1".into(),
            idempotency_key: "idemp_test_1".into(),
            work_item: sample_ref(),
            project_root: PathBuf::from("/tmp/p"),
            continuity_id: "focusa-cont-test".into(),
            workpoint_id: Some("wp_test".into()),
            actor_id: "verious.smith@philoveracity.com".into(),
            agent_session_id: Some("sess_test".into()),
            closure_summary: "test".into(),
            closure_kind: ClosureKind::Code,
            code_refs: vec![sample_citation(EvidenceKind::Code, "crates/foo.rs:42")],
            spec_refs: vec![sample_citation(EvidenceKind::Spec, "docs/116-...md#7.4")],
            proof_refs: vec![sample_citation(EvidenceKind::Test, "tests/spec_...sh")],
            deploy_refs: vec![sample_citation(
                EvidenceKind::Endpoint,
                "GET /v1/health -> 200",
            )],
            artifact_refs: vec![],
            policy: "release_proof".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            status: ClaimStatus::Draft,
            override_reason: None,
            machine_id: None,
        };
        let s = serde_json::to_string(&claim).unwrap();
        let back: ClosureClaim = serde_json::from_str(&s).unwrap();
        assert_eq!(back.claim_id, claim.claim_id);
        assert_eq!(back.work_item.provider, WorkItemProvider::Bd);
        assert_eq!(back.evidence_count(), 4);
        assert!(!back.is_expired(Utc::now() - chrono::Duration::seconds(1)));
        assert!(back.is_expired(Utc::now() + chrono::Duration::hours(2)));
        assert!(!back.is_override());
    }

    #[test]
    fn override_claim_records_reclaimed_marker() {
        let mut claim = ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: "claim_test_2".into(),
            idempotency_key: "idemp_test_2".into(),
            work_item: sample_ref(),
            project_root: PathBuf::from("/tmp/p"),
            continuity_id: "focusa-cont-test".into(),
            workpoint_id: None,
            actor_id: "agent@local".into(),
            agent_session_id: None,
            closure_summary: "docs only".into(),
            closure_kind: ClosureKind::Docs,
            code_refs: vec![],
            spec_refs: vec![sample_citation(EvidenceKind::Spec, "docs/116-...md#7")],
            proof_refs: vec![],
            deploy_refs: vec![],
            artifact_refs: vec![],
            policy: "default".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            status: ClaimStatus::Authorized,
            override_reason: None,
            machine_id: None,
        };
        claim.override_reason = Some("docs only, no code change".into());
        claim.actor_id = RECLAIMED_BY_OPERATOR.to_string();
        assert!(claim.is_override());
    }

    #[test]
    fn closure_block_envelope_shape() {
        let b = ClosureBlock::new(
            "validation_rejected",
            "test_failed",
            "tests/foo.sh exited with 1",
            "rerun `bash tests/foo.sh` and inspect stderr",
            LifecycleStage::Validate,
        );
        assert_eq!(b.status, "blocked");
        assert_eq!(b.failure_class, "validation_rejected");
        assert_eq!(b.stage, LifecycleStage::Validate);
        assert!(!b.next_tools.is_empty());
    }
}
