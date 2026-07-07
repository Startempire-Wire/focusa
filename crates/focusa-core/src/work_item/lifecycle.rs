//! Closure lifecycle: Prepare -> Validate -> Authorize -> Submit -> Reconcile
//! (Spec 116 §9).
//!
//! The lifecycle is the only place that mutates the closure claim. Every
//! transition writes a copy of the claim to durable storage and appends
//! an audit row. The five stages are:
//!
//! 1. `prepare`  — collect citations from a Workpoint + project state
//! 2. `validate` — run the matching verifier for every citation; flip
//!    claim.status to "valid" only when every required citation passes
//! 3. `authorize`— check the closure policy, actor, and machine_id
//! 4. `submit`   — call the provider adapter; mutate the task manager
//! 5. `reconcile`— verify the provider's post-submit state and write
//!    the final audit row

use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::work_item::audit::{ClosureAuditEvent, ClosureAuditLog};
use crate::work_item::evidence::{
    ArtifactStub, CiVerifier, CodeVerifier, DeployVerifier, EndpointVerifier, EvidenceVerifier,
    SpecVerifier, TestVerifier, VerifyResult, WorkpointVerifier,
};
use crate::work_item::policy::{ClosurePolicy, ClosureProfile};
use crate::work_item::storage::ClaimStorage;
use crate::work_item::types::{
    ClaimStatus, ClosureBlock, ClosureClaim, ClosureClaimBuilder, ClosureError, ClosureKind,
    EvidenceCitation, EvidenceKind, LifecycleStage, RECLAIMED_BY_OPERATOR, WorkItem, WorkItemRef,
};
use crate::work_item::adapter::{ProviderAdapter, ProviderRegistry, RegistryError, RegistryResult};

/// Result of `prepare`.
#[derive(Clone, Debug)]
pub struct PrepareResult {
    pub claim: ClosureClaim,
    pub block: Option<ClosureBlock>,
}

/// Result of `validate`.
#[derive(Clone, Debug)]
pub struct ValidateResult {
    pub claim: ClosureClaim,
    pub verify_results: Vec<VerifyResult>,
    pub block: Option<ClosureBlock>,
}

/// Result of `authorize`.
#[derive(Clone, Debug)]
pub struct AuthorizeResult {
    pub claim: ClosureClaim,
    pub block: Option<ClosureBlock>,
}

/// Result of `submit`.
#[derive(Clone, Debug)]
pub struct SubmitResult {
    pub claim: ClosureClaim,
    pub work_item: WorkItem,
    pub block: Option<ClosureBlock>,
}

/// Result of `reconcile`.
#[derive(Clone, Debug)]
pub struct ReconcileResult {
    pub claim: ClosureClaim,
    pub block: Option<ClosureBlock>,
}

/// The lifecycle is a long-lived service. Construct one with
/// `Lifecycle::new(...)` and run each stage in order.
#[derive(Clone)]
pub struct Lifecycle {
    storage: ClaimStorage,
    audit: ClosureAuditLog,
    policy: ClosurePolicy,
    profiles: Vec<ClosureProfile>,
    registry: ProviderRegistry,
}

impl Lifecycle {
    pub fn new(
        storage: ClaimStorage,
        audit: ClosureAuditLog,
        policy: ClosurePolicy,
        profiles: Vec<ClosureProfile>,
        registry: ProviderRegistry,
    ) -> Self {
        Self {
            storage,
            audit,
            policy,
            profiles,
            registry,
        }
    }

    /// Open the lifecycle with default storage + audit paths and the
    /// default policy. The provider registry is empty; callers must
    /// register adapters (or use `Lifecycle::with_default_adapters`).
    pub fn open_default() -> Self {
        Self::new(
            ClaimStorage::open_default(),
            ClosureAuditLog::open_default(),
            ClosurePolicy::load(),
            ClosureProfile::load_all(&crate::work_item::policy::default_profiles_dir()),
            ProviderRegistry::empty(),
        )
    }

    /// Run all five stages in order. Returns the final claim on
    /// success, or the first `ClosureBlock` produced.
    #[allow(clippy::result_large_err)]
    pub fn run(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
    ) -> Result<ClosureClaim, ClosureBlock> {
        let prepared = self
            .prepare(actor, work_item, closure_summary, closure_kind, citations)
            .map_err(|e| e.into_block())?;
        if let Some(b) = prepared.block {
            return Err(b);
        }
        let validated = self
            .validate(prepared.claim.claim_id.clone())
            .map_err(|e| e.into_block())?;
        if let Some(b) = validated.block {
            return Err(b);
        }
        let authorized = self
            .authorize(actor, validated.claim.claim_id.clone())
            .map_err(|e| e.into_block())?;
        if let Some(b) = authorized.block {
            return Err(b);
        }
        let submitted = self
            .submit(authorized.claim.claim_id.clone())
            .map_err(|e| e.into_block())?;
        if let Some(b) = submitted.block {
            return Err(b);
        }
        let reconciled = self
            .reconcile(submitted.claim.claim_id.clone())
            .map_err(|e| e.into_block())?;
        if let Some(b) = reconciled.block {
            return Err(b);
        }
        Ok(reconciled.claim)
    }

    /// Stage 1: prepare.
    pub fn prepare(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
    ) -> Result<PrepareResult, ClosureError> {
        let claim_id = format!(
            "claim_{}_{}",
            work_item.provider,
            &Uuid::now_v7().to_string().replace('-', "")[..16]
        );
        let idempotency_key = format!(
            "{}:{}:{}",
            work_item.provider,
            work_item.provider_item_id,
            closure_summary.len()
        );
        let profile_name = self
            .policy
            .active_profile
            .clone();
        let now = Utc::now();
        let claim = ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: claim_id.clone(),
            idempotency_key,
            work_item: work_item.clone(),
            project_root: work_item.project_root.clone(),
            continuity_id: format!("focusa-cont-{}", &Uuid::now_v7().to_string()[..8]),
            workpoint_id: None,
            actor_id: actor.to_string(),
            agent_session_id: None,
            closure_summary: closure_summary.to_string(),
            closure_kind,
            code_refs: citations.iter().filter(|c| c.kind == EvidenceKind::Code).cloned().collect(),
            spec_refs: citations.iter().filter(|c| c.kind == EvidenceKind::Spec).cloned().collect(),
            proof_refs: citations.iter().filter(|c| matches!(c.kind, EvidenceKind::Test | EvidenceKind::Endpoint | EvidenceKind::Workpoint | EvidenceKind::Ci)).cloned().collect(),
            deploy_refs: citations.iter().filter(|c| c.kind == EvidenceKind::Deploy).cloned().collect(),
            artifact_refs: citations.iter().filter(|c| c.kind == EvidenceKind::Artifact).cloned().collect(),
            policy: profile_name,
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            status: ClaimStatus::Draft,
            override_reason: None,
            machine_id: None,
        };
        self.storage
            .save(&claim)
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Prepare,
                failure_class: "persistence_failed".into(),
                code: "save_failed".into(),
                why: format!("claim save failed: {e}"),
                recovery_hint: "check the data_dir and disk space".into(),
            })?;
        self.audit
            .append(
                ClosureAuditEvent::new(
                    LifecycleStage::Prepare,
                    actor,
                    format!("prepared claim {claim_id} ({} citations)", claim.evidence_count()),
                )
                .with_claim(&claim),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Prepare,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(PrepareResult { claim, block: None })
    }

    /// Stage 2: validate. Loads the claim from storage, runs every
    /// verifier, persists, audits.
    pub fn validate(&self, claim_id: String) -> Result<ValidateResult, ClosureError> {
        let mut claim = self.load_claim(&claim_id, LifecycleStage::Validate)?;
        let mut verify_results = Vec::new();
        // The verifier dispatch is async; we run a single-threaded
        // tokio runtime inside the sync `validate()` so the rest of
        // the lifecycle stays sync (the CLI doctor and the audit
        // replay path are easier to reason about that way).
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(r) => r,
            Err(e) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Validate,
                    failure_class: "internal_error".into(),
                    code: "runtime_build_failed".into(),
                    why: format!("tokio runtime build failed: {e}"),
                    recovery_hint: "check the host's libssl/libc".into(),
                });
            }
        };
        // Run verifier on every citation across every field.
        for citation in claim.code_refs.iter_mut()
            .chain(claim.spec_refs.iter_mut())
            .chain(claim.proof_refs.iter_mut())
            .chain(claim.deploy_refs.iter_mut())
            .chain(claim.artifact_refs.iter_mut())
        {
            let res = rt.block_on(run_verifier_for(citation));
            citation.result = Some(res.result.clone());
            citation.verified = res.verified;
            verify_results.push(res.clone());
        }
        let profile = self
            .profiles
            .iter()
            .find(|p| p.name == claim.policy)
            .or_else(|| self.profiles.first())
            .cloned();
        let block = if let Some(p) = profile {
            evaluate_profile(&p, &claim, &verify_results)
        } else {
            None
        };
        if block.is_none() {
            claim.status = ClaimStatus::Valid;
        } else {
            claim.status = ClaimStatus::Blocked;
        }
        self.storage
            .save(&claim)
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Validate,
                failure_class: "persistence_failed".into(),
                code: "save_failed".into(),
                why: format!("claim save failed: {e}"),
                recovery_hint: "check the data_dir and disk space".into(),
            })?;
        let detail = if let Some(b) = &block {
            format!("validate FAILED: {} ({} citations)", b.code, verify_results.len())
        } else {
            format!(
                "validate OK: {}/{} citations verified",
                verify_results.iter().filter(|r| r.verified).count(),
                verify_results.len()
            )
        };
        self.audit
            .append(
                ClosureAuditEvent::new(LifecycleStage::Validate, &claim.actor_id, detail)
                    .with_claim(&claim),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Validate,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(ValidateResult {
            claim,
            verify_results,
            block,
        })
    }

    /// Stage 3: authorize. Checks the policy.
    pub fn authorize(&self, actor: &str, claim_id: String) -> Result<AuthorizeResult, ClosureError> {
        let mut claim = self.load_claim(&claim_id, LifecycleStage::Authorize)?;
        if claim.status != ClaimStatus::Valid && !claim.is_override() {
            return Err(ClosureError {
                stage: LifecycleStage::Authorize,
                failure_class: "policy_denied".into(),
                code: "not_valid".into(),
                why: format!(
                    "claim status is {} (must be valid or override)",
                    claim.status
                ),
                recovery_hint: "run `focusa work-item closure validate <claim_id>` first".into(),
            });
        }
        if self.policy.block_list.contains(&claim.actor_id) {
            return Err(ClosureError {
                stage: LifecycleStage::Authorize,
                failure_class: "policy_denied".into(),
                code: "actor_blocked".into(),
                why: format!("actor {} is in policy block list", claim.actor_id),
                recovery_hint: "use a different actor or update the policy".into(),
            });
        }
        if claim.is_override() {
            let allowed = self.policy.override_policy.agents_can_override
                || self
                    .policy
                    .override_allow_list
                    .iter()
                    .any(|a| a == actor || a == &claim.actor_id);
            if !allowed {
                return Err(ClosureError {
                    stage: LifecycleStage::Authorize,
                    failure_class: "policy_denied".into(),
                    code: "override_disallowed".into(),
                    why: "override is disabled for this actor; set policy.override.agents_can_override=true or add actor to override_allow_list".into(),
                    recovery_hint: "rerun with a real evidence claim or update the policy".into(),
                });
            }
        }
        claim.status = ClaimStatus::Authorized;
        self.storage
            .save(&claim)
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Authorize,
                failure_class: "persistence_failed".into(),
                code: "save_failed".into(),
                why: format!("claim save failed: {e}"),
                recovery_hint: "check the data_dir and disk space".into(),
            })?;
        self.audit
            .append(
                ClosureAuditEvent::new(
                    LifecycleStage::Authorize,
                    actor,
                    "authorized",
                )
                .with_claim(&claim),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Authorize,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(AuthorizeResult { claim, block: None })
    }

    /// Stage 4: submit. Calls the provider adapter.
    pub fn submit(&self, claim_id: String) -> Result<SubmitResult, ClosureError> {
        let claim = self.load_claim(&claim_id, LifecycleStage::Submit)?;
        let adapter = self.registry.get(claim.work_item.provider).ok_or_else(|| {
            ClosureError {
                stage: LifecycleStage::Submit,
                failure_class: "provider_unavailable".into(),
                code: "no_adapter".into(),
                why: format!(
                    "no adapter registered for provider {}",
                    claim.work_item.provider
                ),
                recovery_hint: "register the provider via `focusa work-item providers add <provider>` or run `focusa install closure-guard --auto`".into(),
            }
        })?;
        let caps = adapter.capabilities();
        if !caps.can_submit {
            return Err(ClosureError {
                stage: LifecycleStage::Submit,
                failure_class: "capability_unsupported".into(),
                code: "submit_unsupported".into(),
                why: format!(
                    "provider {} does not support submit; this is a read-only adapter",
                    claim.work_item.provider
                ),
                recovery_hint: "switch to a mutable provider or use the no-op adapter".into(),
            });
        }
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| {
            ClosureError {
                stage: LifecycleStage::Submit,
                failure_class: "internal_error".into(),
                code: "runtime_build_failed".into(),
                why: format!("tokio runtime build failed: {e}"),
                recovery_hint: "check the host's libssl/libc".into(),
            }
        })?;
        let claim_for_audit = claim.clone();
        let adapter_for_call = adapter.clone();
        let work_item_for_call = claim.work_item.clone();
        let res = rt.block_on(async move {
            adapter_for_call
                .submit(&work_item_for_call)
                .await
        });
        let work_item = match res {
            Ok(w) => w,
            Err(RegistryError::ProviderError { provider, stage: _, why }) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Submit,
                    failure_class: "provider_failed".into(),
                    code: "submit_failed".into(),
                    why: format!("provider {provider} submit failed: {why}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry the claim".into(),
                })
            }
            Err(e) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Submit,
                    failure_class: "provider_error".into(),
                    code: "submit_error".into(),
                    why: format!("provider error: {e}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry the claim".into(),
                })
            }
        };
        let mut updated_claim = claim;
        updated_claim.status = ClaimStatus::Submitted;
        self.storage.save(&updated_claim).map_err(|e| ClosureError {
            stage: LifecycleStage::Submit,
            failure_class: "persistence_failed".into(),
            code: "save_failed".into(),
            why: format!("claim save failed: {e}"),
            recovery_hint: "check the data_dir and disk space".into(),
        })?;
        self.audit
            .append(
                ClosureAuditEvent::new(
                    LifecycleStage::Submit,
                    &updated_claim.actor_id,
                    format!("submitted; provider status={}", work_item.provider_status),
                )
                .with_claim(&claim_for_audit),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Submit,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(SubmitResult {
            claim: updated_claim,
            work_item,
            block: None,
        })
    }

    /// Stage 5: reconcile. Re-resolves the work item from the
    /// provider and verifies the post-submit state.
    pub fn reconcile(&self, claim_id: String) -> Result<ReconcileResult, ClosureError> {
        let claim = self.load_claim(&claim_id, LifecycleStage::Reconcile)?;
        let adapter = self.registry.get(claim.work_item.provider).ok_or_else(|| {
            ClosureError {
                stage: LifecycleStage::Reconcile,
                failure_class: "provider_unavailable".into(),
                code: "no_adapter".into(),
                why: format!(
                    "no adapter registered for provider {}",
                    claim.work_item.provider
                ),
                recovery_hint: "register the provider or accept the local reconciliation result".into(),
            }
        })?;
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| ClosureError {
            stage: LifecycleStage::Reconcile,
            failure_class: "internal_error".into(),
            code: "runtime_build_failed".into(),
            why: format!("tokio runtime build failed: {e}"),
            recovery_hint: "check the host's libssl/libc".into(),
        })?;
        let adapter_for_call = adapter.clone();
        let work_item_for_call = claim.work_item.clone();
        let res = rt.block_on(async move {
            adapter_for_call.reconcile(&work_item_for_call).await
        });
        let work_item = match res {
            Ok(w) => w,
            Err(RegistryError::ProviderError { provider, stage: _, why }) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Reconcile,
                    failure_class: "provider_failed".into(),
                    code: "reconcile_failed".into(),
                    why: format!("provider {provider} reconcile failed: {why}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry reconcile".into(),
                })
            }
            Err(e) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Reconcile,
                    failure_class: "provider_error".into(),
                    code: "reconcile_error".into(),
                    why: format!("provider error: {e}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry reconcile".into(),
                })
            }
        };
        let expected_closed = matches!(
            work_item.provider_status,
            crate::work_item::types::WorkItemStatus::Closed
                | crate::work_item::types::WorkItemStatus::Done
        );
        let block = if !expected_closed {
            let mut b = ClosureBlock::new(
                "provider_unexpected_state",
                "reconcile_state_mismatch",
                format!(
                    "provider reports status `{}` after submit; expected Closed or Done",
                    work_item.provider_status
                ),
                String::from("rerun `focusa work-item closure submit <claim_id>` and inspect the provider"),
                LifecycleStage::Reconcile,
            );
            b.claim_id = Some(claim.claim_id.clone());
            Some(b)
        } else {
            None
        };
        let mut updated = claim;
        updated.status = if block.is_some() {
            ClaimStatus::Blocked
        } else {
            ClaimStatus::Reconciled
        };
        self.storage.save(&updated).map_err(|e| ClosureError {
            stage: LifecycleStage::Reconcile,
            failure_class: "persistence_failed".into(),
            code: "save_failed".into(),
            why: format!("claim save failed: {e}"),
            recovery_hint: "check the data_dir and disk space".into(),
        })?;
        self.audit
            .append(
                ClosureAuditEvent::new(
                    LifecycleStage::Reconcile,
                    &updated.actor_id,
                    if block.is_some() {
                        "reconcile FAILED".into()
                    } else {
                        format!("reconciled; provider status={}", work_item.provider_status)
                    },
                )
                .with_claim(&updated),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Reconcile,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(ReconcileResult {
            claim: updated,
            block,
        })
    }

    /// Apply `--override --reason`. Marks the claim as overridden
    /// and records the operator's reason. Use only as a last resort.
    pub fn apply_override(
        &self,
        actor: &str,
        claim_id: &str,
        reason: &str,
    ) -> Result<ClosureClaim, ClosureError> {
        let mut claim = self.load_claim(claim_id, LifecycleStage::Authorize)?;
        let mut b = ClosureClaimBuilder::new(claim).override_reason(reason);
        b = b.status(ClaimStatus::Authorized);
        claim = b.build().unwrap();
        claim.actor_id = actor.to_string();
        self.storage.save(&claim).map_err(|e| ClosureError {
            stage: LifecycleStage::Authorize,
            failure_class: "persistence_failed".into(),
            code: "save_failed".into(),
            why: format!("claim save failed: {e}"),
            recovery_hint: "check the data_dir and disk space".into(),
        })?;
        self.audit
            .append(
                ClosureAuditEvent::new(
                    LifecycleStage::Authorize,
                    actor,
                    format!("OVERRIDE: {reason}"),
                )
                .with_claim(&claim),
            )
            .map_err(|e| ClosureError {
                stage: LifecycleStage::Authorize,
                failure_class: "audit_append_failed".into(),
                code: "audit_failed".into(),
                why: format!("audit append failed: {e}"),
                recovery_hint: "check the audit log path and disk space".into(),
            })?;
        Ok(claim)
    }

    fn load_claim(&self, claim_id: &str, stage: LifecycleStage) -> Result<ClosureClaim, ClosureError> {
        self.storage.load(claim_id).map_err(|e| {
            ClosureError {
                stage,
                failure_class: "persistence_failed".into(),
                code: "load_failed".into(),
                why: format!("claim load failed: {e}"),
                recovery_hint: "rerun `focusa work-item closure prepare <id>` to recreate the claim".into(),
            }
        })
    }

    /// Read-only access to the registry (used by the CLI to render
    /// `focusa work-item providers list`).
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }
}

/// Run the matching verifier for a citation. This is the dispatch
/// table referenced by Spec 116 §7.3.
pub async fn run_verifier_for(citation: &mut EvidenceCitation) -> VerifyResult {
    let verifier: Box<dyn EvidenceVerifier> = match citation.kind {
        EvidenceKind::Code => Box::new(CodeVerifier),
        EvidenceKind::Spec => Box::new(SpecVerifier),
        EvidenceKind::Test => Box::new(TestVerifier),
        EvidenceKind::Endpoint => Box::new(EndpointVerifier::default()),
        EvidenceKind::Artifact => Box::new(ArtifactStub),
        EvidenceKind::Workpoint => Box::new(WorkpointVerifier::default()),
        EvidenceKind::Ci => Box::new(CiVerifier::default()),
        EvidenceKind::Deploy => Box::new(DeployVerifier::default()),
    };
    verifier.verify(citation).await
}

/// Evaluate the profile against the verified citations. Returns a
/// `ClosureBlock` if the claim does not satisfy the profile, `None`
/// otherwise.
pub fn evaluate_profile(
    profile: &ClosureProfile,
    claim: &ClosureClaim,
    results: &[VerifyResult],
) -> Option<ClosureBlock> {
    let mut all_citations: Vec<&EvidenceCitation> = Vec::new();
    all_citations.extend(claim.code_refs.iter());
    all_citations.extend(claim.spec_refs.iter());
    all_citations.extend(claim.proof_refs.iter());
    all_citations.extend(claim.deploy_refs.iter());
    all_citations.extend(claim.artifact_refs.iter());

    // Min-count per kind.
    for (kind, min) in &profile.rule.min_required {
        let count = all_citations.iter().filter(|c| &c.kind == kind).count() as u32;
        if count < *min {
            return Some(ClosureBlock::new(
                "validation_rejected",
                "min_evidence_count",
                format!(
                    "profile `{}` requires at least {min} `{kind}` citation(s); found {count}",
                    profile.name
                ),
                format!("add at least {} more `{}` citation(s) to the claim", min - count, kind),
                LifecycleStage::Validate,
            ));
        }
    }

    // Required kinds present at all.
    for kind in &profile.rule.required_kinds {
        if !all_citations.iter().any(|c| &c.kind == kind) {
            return Some(ClosureBlock::new(
                "validation_rejected",
                "missing_required_kind",
                format!(
                    "profile `{}` requires a `{kind}` citation; none present",
                    profile.name
                ),
                format!("add a `{kind}` citation to the claim"),
                LifecycleStage::Validate,
            ));
        }
    }

    // Required citations verified.
    let failed: Vec<usize> = all_citations
        .iter()
        .enumerate()
        .filter(|(_, c)| c.required && !c.verified)
        .map(|(i, _)| i)
        .collect();
    if !failed.is_empty() {
        let idx = failed[0];
        let citation = &all_citations[idx];
        return Some(ClosureBlock::new(
            "validation_rejected",
            "citation_failed",
            format!(
                "required `{kind}` citation failed: {ref_}",
                kind = citation.kind,
                ref_ = citation.ref_
            ),
            format!(
                "re-run the matching verifier or replace the citation; current result: {:?}",
                citation.result
            ),
            LifecycleStage::Validate,
        ));
    }

    // Optional: endpoint status whitelist.
    if !profile.rule.endpoint_status_in.is_empty() {
        for citation in all_citations.iter().filter(|c| c.kind == EvidenceKind::Endpoint) {
            // The verifier's result string contains "http NNN" — pull
            // out the first number that is between 100 and 599.
            if let Some(code) = parse_http_code(citation.result.as_deref().unwrap_or("")) {
                if !profile.rule.endpoint_status_in.contains(&code) {
                    return Some(ClosureBlock::new(
                        "validation_rejected",
                        "endpoint_status_mismatch",
                        format!(
                            "endpoint {} returned http {}; profile `{}` requires one of {:?}",
                            citation.ref_, code, profile.name, profile.rule.endpoint_status_in
                        ),
                        "replace the citation with an endpoint that returns one of the allowed status codes".to_string(),
                        LifecycleStage::Validate,
                    ));
                }
            }
        }
    }

    None
}

fn parse_http_code(s: &str) -> Option<u16> {
    // "GET ... -> http OK 200 body=`...`" — find the first 3-digit
    // number after "http".
    if let Some(idx) = s.find("http ") {
        let rest = &s[idx + 5..];
        for token in rest.split_whitespace() {
            if let Ok(n) = token.parse::<u16>() {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work_item::types::{ClaimStatus, ClosureClaim, ClosureKind, EvidenceCitation, EvidenceKind, WorkItemProvider, WorkItemRef};
    use chrono::Utc;
    use std::path::PathBuf;

    fn tmp_claim() -> ClosureClaim {
        ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: "claim_test_lc".into(),
            idempotency_key: "idem_test_lc".into(),
            work_item: WorkItemRef {
                provider: WorkItemProvider::Bd,
                provider_item_id: "focusa-test".into(),
                project_root: PathBuf::from("/tmp/p"),
                external_url: None,
            },
            project_root: PathBuf::from("/tmp/p"),
            continuity_id: "focusa-cont-test".into(),
            workpoint_id: None,
            actor_id: "verious.smith@philoveracity.com".into(),
            agent_session_id: None,
            closure_summary: "test".into(),
            closure_kind: ClosureKind::Code,
            code_refs: vec![],
            spec_refs: vec![],
            proof_refs: vec![],
            deploy_refs: vec![],
            artifact_refs: vec![],
            policy: "release_proof".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            status: ClaimStatus::Valid,
            override_reason: None,
            machine_id: None,
        }
    }

    #[test]
    fn evaluate_profile_rejects_when_min_count_missing() {
        let profile = ClosureProfile::release_proof();
        let claim = tmp_claim();
        // No citations at all -> min_required for Code=1 fails.
        let block = evaluate_profile(&profile, &claim, &[]);
        assert!(block.is_some());
        assert_eq!(block.unwrap().code, "min_evidence_count");
    }

    #[test]
    fn evaluate_profile_accepts_minimum_evidence_set() {
        let profile = ClosureProfile::release_proof();
        let mut claim = tmp_claim();
        // Provide the minimum evidence.
        for i in 0..1 {
            let mut code = sample_cite(EvidenceKind::Code, "crates/foo.rs");
            code.verified = true;
            code.result = Some("verified".into());
            claim.code_refs.push(code);
        }
        for i in 0..1 {
            let mut test = sample_cite(EvidenceKind::Test, "tests/foo.sh");
            test.verified = true;
            test.result = Some("verified".into());
            claim.proof_refs.push(test);
        }
        for i in 0..2 {
            let mut ep = sample_cite(EvidenceKind::Endpoint, "GET /v1/health -> 200");
            ep.verified = true;
            ep.result = Some("http OK 200".into());
            claim.deploy_refs.push(ep);
        }
        let results: Vec<VerifyResult> = claim
            .code_refs
            .iter()
            .chain(claim.proof_refs.iter())
            .chain(claim.deploy_refs.iter())
            .map(|c| VerifyResult { verified: c.verified, result: c.result.clone().unwrap_or_default(), evidence_url: None })
            .collect();
        let block = evaluate_profile(&profile, &claim, &results);
        assert!(block.is_none(), "block should be None, got {:?}", block);
    }

    #[test]
    fn evaluate_profile_rejects_failed_required_citation() {
        let profile = ClosureProfile::code_only();
        let mut claim = tmp_claim();
        let mut code = sample_cite(EvidenceKind::Code, "crates/foo.rs");
        code.verified = false; // failed
        code.result = Some("not verified".into());
        claim.code_refs.push(code);
        let results = vec![VerifyResult {
            verified: false,
            result: "not verified".into(),
            evidence_url: None,
        }];
        let block = evaluate_profile(&profile, &claim, &results);
        assert!(block.is_some());
        assert_eq!(block.unwrap().code, "citation_failed");
    }

    #[test]
    fn parse_http_code_extracts_status() {
        assert_eq!(parse_http_code("GET /v1/health -> http OK 200 body=`...`"), Some(200));
        assert_eq!(parse_http_code("http FAIL 500"), Some(500));
        assert_eq!(parse_http_code("no http here"), None);
    }

    fn sample_cite(kind: EvidenceKind, ref_: &str) -> EvidenceCitation {
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
}
