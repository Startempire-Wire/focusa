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
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::work_item::adapter::{ProviderAdapter, ProviderRegistry, RegistryError, RegistryResult};
use crate::work_item::adapters::{BdAdapter, NoneAdapter};
use crate::work_item::audit::{ClosureAuditEvent, ClosureAuditLog};
use crate::work_item::evidence::{
    ArtifactStub, CiVerifier, CodeVerifier, DeployVerifier, EndpointVerifier, EvidenceVerifier,
    SpecVerifier, TestVerifier, VerifyResult, WorkpointVerifier,
};
use crate::work_item::policy::{ClosurePolicy, ClosureProfile, default_profile_for};
use crate::work_item::storage::ClaimStorage;
use crate::work_item::types::{
    ClaimStatus, ClosureAuthorityContext, ClosureBlock, ClosureClaim, ClosureClaimBuilder,
    ClosureError, ClosureKind, EvidenceCitation, EvidenceKind, LifecycleStage,
    RECLAIMED_BY_OPERATOR, WorkItem, WorkItemRef,
};

fn closure_idempotency_key(
    work_item: &WorkItemRef,
    closure_summary: &str,
    authority: Option<&ClosureAuthorityContext>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(work_item.provider.to_string());
    hasher.update(work_item.project_root.to_string_lossy().as_bytes());
    hasher.update(work_item.provider_item_id.as_bytes());
    hasher.update(closure_summary.as_bytes());
    if let Some(scope) = authority {
        hasher.update(scope.continuity_id.as_bytes());
        hasher.update(scope.workpoint_id.as_bytes());
    }
    format!("closure:{:x}", hasher.finalize())
}

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
        Self::open_for_kind(ClosureKind::Code)
    }

    /// Open the durable lifecycle with built-in adapters and the profile
    /// appropriate for the requested closure kind.
    pub fn open_for_kind(closure_kind: ClosureKind) -> Self {
        let mut registry = ProviderRegistry::empty();
        registry.register(Arc::new(BdAdapter::new()));
        registry.register(Arc::new(NoneAdapter::new()));
        let mut policy = ClosurePolicy::load();
        policy.active_profile = default_profile_for(closure_kind).to_string();
        Self::new(
            ClaimStorage::open_default(),
            ClosureAuditLog::open_default(),
            policy,
            ClosureProfile::load_all(&crate::work_item::policy::default_profiles_dir()),
            registry,
        )
    }

    /// Run all five stages in order. Returns the final claim on
    /// success, or the first `ClosureBlock` produced.
    #[allow(clippy::result_large_err)]
    pub async fn run(
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
            .await
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
            .await
            .map_err(|e| e.into_block())?;
        if let Some(b) = submitted.block {
            return Err(b);
        }
        let reconciled = self
            .reconcile(submitted.claim.claim_id.clone())
            .await
            .map_err(|e| e.into_block())?;
        if let Some(b) = reconciled.block {
            return Err(b);
        }
        Ok(reconciled.claim)
    }

    /// Run all lifecycle stages with canonical Work Loop authority scope.
    #[allow(clippy::result_large_err)]
    pub async fn run_scoped(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
        authority: ClosureAuthorityContext,
    ) -> Result<ClosureClaim, ClosureBlock> {
        let idempotency_key =
            closure_idempotency_key(&work_item, closure_summary, Some(&authority));
        if let Some(existing) = self
            .storage
            .find_by_idempotency_key(&idempotency_key)
            .map_err(|error| {
                ClosureBlock::new(
                    "storage_failure",
                    "closure_claim_lookup_failed",
                    error.to_string(),
                    "repair closure claim storage and retry",
                    LifecycleStage::Prepare,
                )
            })?
        {
            return self.advance_claim(actor, existing).await;
        }
        let prepared = self
            .prepare_scoped(
                actor,
                work_item,
                closure_summary,
                closure_kind,
                citations,
                authority,
            )
            .map_err(|error| error.into_block())?;
        if let Some(block) = prepared.block {
            return Err(block);
        }
        self.advance_claim(actor, prepared.claim).await
    }

    #[allow(clippy::result_large_err)]
    async fn advance_claim(
        &self,
        actor: &str,
        mut claim: ClosureClaim,
    ) -> Result<ClosureClaim, ClosureBlock> {
        loop {
            claim = match claim.status {
                ClaimStatus::Draft => {
                    let result = self
                        .validate(claim.claim_id.clone())
                        .await
                        .map_err(|error| error.into_block())?;
                    if let Some(block) = result.block {
                        return Err(block);
                    }
                    result.claim
                }
                ClaimStatus::Valid => {
                    let result = self
                        .authorize(actor, claim.claim_id.clone())
                        .map_err(|error| error.into_block())?;
                    if let Some(block) = result.block {
                        return Err(block);
                    }
                    result.claim
                }
                ClaimStatus::Authorized => {
                    let result = self
                        .submit(claim.claim_id.clone())
                        .await
                        .map_err(|error| error.into_block())?;
                    if let Some(block) = result.block {
                        return Err(block);
                    }
                    result.claim
                }
                ClaimStatus::Submitted => {
                    let result = self
                        .reconcile(claim.claim_id.clone())
                        .await
                        .map_err(|error| error.into_block())?;
                    if let Some(block) = result.block {
                        return Err(block);
                    }
                    result.claim
                }
                ClaimStatus::Reconciled => return Ok(claim),
                ClaimStatus::Blocked | ClaimStatus::Expired => {
                    let mut block = ClosureBlock::new(
                        "closure_not_resumable",
                        "closure_claim_terminal_block",
                        format!("closure claim {} is {}", claim.claim_id, claim.status),
                        "supply fresh evidence and prepare a new closure summary",
                        LifecycleStage::Prepare,
                    );
                    block.claim_id = Some(claim.claim_id);
                    return Err(block);
                }
            };
        }
    }

    /// Stage 1: prepare for legacy/manual callers without typed Work Loop scope.
    pub fn prepare(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
    ) -> Result<PrepareResult, ClosureError> {
        self.prepare_with_authority(
            actor,
            work_item,
            closure_summary,
            closure_kind,
            citations,
            None,
        )
    }

    /// Stage 1 with explicit project/workstream/Workpoint authority.
    pub fn prepare_scoped(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
        authority: ClosureAuthorityContext,
    ) -> Result<PrepareResult, ClosureError> {
        if authority.continuity_id.trim().is_empty() || authority.workpoint_id.trim().is_empty() {
            return Err(ClosureError {
                stage: LifecycleStage::Prepare,
                failure_class: "authority_scope_invalid".into(),
                code: "closure_scope_required".into(),
                why: "scoped closure requires non-empty continuity_id and workpoint_id".into(),
                recovery_hint: "bind a canonical Workpoint and typed WorkstreamKey before closure"
                    .into(),
            });
        }
        self.prepare_with_authority(
            actor,
            work_item,
            closure_summary,
            closure_kind,
            citations,
            Some(authority),
        )
    }

    fn prepare_with_authority(
        &self,
        actor: &str,
        work_item: WorkItemRef,
        closure_summary: &str,
        closure_kind: ClosureKind,
        citations: Vec<EvidenceCitation>,
        authority: Option<ClosureAuthorityContext>,
    ) -> Result<PrepareResult, ClosureError> {
        let claim_id = format!(
            "claim_{}_{}",
            work_item.provider,
            &Uuid::now_v7().to_string().replace('-', "")[..16]
        );
        let idempotency_key =
            closure_idempotency_key(&work_item, closure_summary, authority.as_ref());
        let profile_name = self.policy.active_profile.clone();
        let now = Utc::now();
        let claim = ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: claim_id.clone(),
            idempotency_key,
            work_item: work_item.clone(),
            project_root: work_item.project_root.clone(),
            continuity_id: authority
                .as_ref()
                .map(|scope| scope.continuity_id.clone())
                .unwrap_or_else(|| format!("focusa-cont-{}", &Uuid::now_v7().to_string()[..8])),
            workpoint_id: authority.as_ref().map(|scope| scope.workpoint_id.clone()),
            actor_id: actor.to_string(),
            agent_session_id: authority
                .as_ref()
                .and_then(|scope| scope.agent_session_id.clone()),
            closure_summary: closure_summary.to_string(),
            closure_kind,
            code_refs: citations
                .iter()
                .filter(|c| c.kind == EvidenceKind::Code)
                .cloned()
                .collect(),
            spec_refs: citations
                .iter()
                .filter(|c| c.kind == EvidenceKind::Spec)
                .cloned()
                .collect(),
            proof_refs: citations
                .iter()
                .filter(|c| {
                    matches!(
                        c.kind,
                        EvidenceKind::Test
                            | EvidenceKind::Endpoint
                            | EvidenceKind::Workpoint
                            | EvidenceKind::Ci
                    )
                })
                .cloned()
                .collect(),
            deploy_refs: citations
                .iter()
                .filter(|c| c.kind == EvidenceKind::Deploy)
                .cloned()
                .collect(),
            artifact_refs: citations
                .iter()
                .filter(|c| c.kind == EvidenceKind::Artifact)
                .cloned()
                .collect(),
            policy: profile_name,
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            status: ClaimStatus::Draft,
            override_reason: None,
            machine_id: None,
        };
        self.storage.save(&claim).map_err(|e| ClosureError {
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
                    format!(
                        "prepared claim {claim_id} ({} citations)",
                        claim.evidence_count()
                    ),
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
    pub async fn validate(&self, claim_id: String) -> Result<ValidateResult, ClosureError> {
        let mut claim = self.load_claim(&claim_id, LifecycleStage::Validate)?;
        let mut verify_results = Vec::new();
        let project_root = claim.project_root.clone();
        // Run verifier on every citation across every field.
        for citation in claim
            .code_refs
            .iter_mut()
            .chain(claim.spec_refs.iter_mut())
            .chain(claim.proof_refs.iter_mut())
            .chain(claim.deploy_refs.iter_mut())
            .chain(claim.artifact_refs.iter_mut())
        {
            let res = run_verifier_for_in_project(citation, &project_root).await;
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
        self.storage.save(&claim).map_err(|e| ClosureError {
            stage: LifecycleStage::Validate,
            failure_class: "persistence_failed".into(),
            code: "save_failed".into(),
            why: format!("claim save failed: {e}"),
            recovery_hint: "check the data_dir and disk space".into(),
        })?;
        let detail = if let Some(b) = &block {
            format!(
                "validate FAILED: {} ({} citations)",
                b.code,
                verify_results.len()
            )
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
    pub fn authorize(
        &self,
        actor: &str,
        claim_id: String,
    ) -> Result<AuthorizeResult, ClosureError> {
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
        self.storage.save(&claim).map_err(|e| ClosureError {
            stage: LifecycleStage::Authorize,
            failure_class: "persistence_failed".into(),
            code: "save_failed".into(),
            why: format!("claim save failed: {e}"),
            recovery_hint: "check the data_dir and disk space".into(),
        })?;
        self.audit
            .append(
                ClosureAuditEvent::new(LifecycleStage::Authorize, actor, "authorized")
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
    pub async fn submit(&self, claim_id: String) -> Result<SubmitResult, ClosureError> {
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
        let claim_for_audit = claim.clone();
        let work_item_for_call = claim.work_item.clone();
        let res = adapter.submit(&work_item_for_call).await;
        let work_item = match res {
            Ok(w) => w,
            Err(RegistryError::ProviderError {
                provider,
                stage: _,
                why,
            }) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Submit,
                    failure_class: "provider_failed".into(),
                    code: "submit_failed".into(),
                    why: format!("provider {provider} submit failed: {why}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry the claim"
                        .into(),
                });
            }
            Err(e) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Submit,
                    failure_class: "provider_error".into(),
                    code: "submit_error".into(),
                    why: format!("provider error: {e}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry the claim"
                        .into(),
                });
            }
        };
        let mut updated_claim = claim;
        updated_claim.status = ClaimStatus::Submitted;
        self.storage
            .save(&updated_claim)
            .map_err(|e| ClosureError {
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
    pub async fn reconcile(&self, claim_id: String) -> Result<ReconcileResult, ClosureError> {
        let claim = self.load_claim(&claim_id, LifecycleStage::Reconcile)?;
        let adapter = self
            .registry
            .get(claim.work_item.provider)
            .ok_or_else(|| ClosureError {
                stage: LifecycleStage::Reconcile,
                failure_class: "provider_unavailable".into(),
                code: "no_adapter".into(),
                why: format!(
                    "no adapter registered for provider {}",
                    claim.work_item.provider
                ),
                recovery_hint: "register the provider or accept the local reconciliation result"
                    .into(),
            })?;
        let work_item_for_call = claim.work_item.clone();
        let res = adapter.reconcile(&work_item_for_call).await;
        let work_item = match res {
            Ok(w) => w,
            Err(RegistryError::ProviderError {
                provider,
                stage: _,
                why,
            }) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Reconcile,
                    failure_class: "provider_failed".into(),
                    code: "reconcile_failed".into(),
                    why: format!("provider {provider} reconcile failed: {why}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry reconcile"
                        .into(),
                });
            }
            Err(e) => {
                return Err(ClosureError {
                    stage: LifecycleStage::Reconcile,
                    failure_class: "provider_error".into(),
                    code: "reconcile_error".into(),
                    why: format!("provider error: {e}"),
                    recovery_hint: "inspect the provider's CLI / API output; retry reconcile"
                        .into(),
                });
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
                String::from(
                    "rerun `focusa work-item closure submit <claim_id>` and inspect the provider",
                ),
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

    fn load_claim(
        &self,
        claim_id: &str,
        stage: LifecycleStage,
    ) -> Result<ClosureClaim, ClosureError> {
        self.storage.load(claim_id).map_err(|e| ClosureError {
            stage,
            failure_class: "persistence_failed".into(),
            code: "load_failed".into(),
            why: format!("claim load failed: {e}"),
            recovery_hint: "rerun `focusa work-item closure prepare <id>` to recreate the claim"
                .into(),
        })
    }

    /// Read-only access to the registry (used by the CLI to render
    /// `focusa work-item providers list`).
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }
}

async fn run_verifier_for_in_project(
    citation: &mut EvidenceCitation,
    project_root: &std::path::Path,
) -> VerifyResult {
    if matches!(
        citation.kind,
        EvidenceKind::Code | EvidenceKind::Spec | EvidenceKind::Test | EvidenceKind::Artifact
    ) {
        let split_at = citation
            .ref_
            .find(" sha256:")
            .or_else(|| citation.ref_.find('#'))
            .unwrap_or(citation.ref_.len());
        let (path_part, suffix) = citation.ref_.split_at(split_at);
        if !path_part.trim().is_empty() && std::path::Path::new(path_part).is_relative() {
            citation.ref_ = format!(
                "{}{}",
                project_root.join(path_part).to_string_lossy(),
                suffix
            );
        }
    }
    run_verifier_for(citation).await
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
                format!(
                    "add at least {} more `{}` citation(s) to the claim",
                    min - count,
                    kind
                ),
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
        for citation in all_citations
            .iter()
            .filter(|c| c.kind == EvidenceKind::Endpoint)
        {
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
    use crate::work_item::types::{
        ClaimStatus, ClosureClaim, ClosureKind, EvidenceCitation, EvidenceKind, WorkItemProvider,
        WorkItemRef,
    };
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
    fn apply_override_persists_reason_and_audit_row() {
        let root = std::env::temp_dir().join(format!("focusa-override-{}", Uuid::now_v7()));
        let storage = ClaimStorage::new(root.join("claims"));
        let audit_path = root.join("closure-audit.jsonl");
        let audit = ClosureAuditLog::open(&audit_path).expect("open audit");
        let lifecycle = Lifecycle::new(
            storage.clone(),
            audit,
            ClosurePolicy::default_policy(),
            ClosureProfile::all_builtins(),
            ProviderRegistry::empty(),
        );
        let claim = tmp_claim();
        storage.save(&claim).expect("save claim");

        let overridden = lifecycle
            .apply_override(
                "operator@test",
                &claim.claim_id,
                "emergency operator approval",
            )
            .expect("apply override");
        assert_eq!(overridden.status, ClaimStatus::Authorized);
        assert_eq!(
            overridden.override_reason.as_deref(),
            Some("emergency operator approval")
        );
        let events = ClosureAuditLog::replay(&audit_path).expect("replay audit");
        assert!(
            events
                .iter()
                .any(|event| event.detail.contains("OVERRIDE:"))
        );
        let _ = std::fs::remove_dir_all(root);
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
            .map(|c| VerifyResult {
                verified: c.verified,
                result: c.result.clone().unwrap_or_default(),
                evidence_url: None,
            })
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
        assert_eq!(
            parse_http_code("GET /v1/health -> http OK 200 body=`...`"),
            Some(200)
        );
        assert_eq!(parse_http_code("http FAIL 500"), Some(500));
        assert_eq!(parse_http_code("no http here"), None);
    }

    #[test]
    fn scoped_prepare_preserves_authority_and_stable_idempotency() {
        let root = std::env::temp_dir().join(format!("focusa-scoped-{}", Uuid::now_v7()));
        let lifecycle = Lifecycle::new(
            ClaimStorage::new(root.join("claims")),
            ClosureAuditLog::open(root.join("audit.jsonl")).unwrap(),
            ClosurePolicy::default_policy(),
            ClosureProfile::all_builtins(),
            ProviderRegistry::empty(),
        );
        let work_item = WorkItemRef {
            provider: WorkItemProvider::None,
            provider_item_id: "local-1".into(),
            project_root: root.clone(),
            external_url: None,
        };
        let authority = ClosureAuthorityContext {
            continuity_id: "continuity-1".into(),
            workpoint_id: Uuid::now_v7().to_string(),
            agent_session_id: Some("session-1".into()),
        };
        let first = lifecycle
            .prepare_scoped(
                "agent",
                work_item.clone(),
                "verified",
                ClosureKind::Code,
                vec![],
                authority.clone(),
            )
            .unwrap()
            .claim;
        let second = lifecycle
            .prepare_scoped(
                "agent",
                work_item,
                "verified",
                ClosureKind::Code,
                vec![],
                authority,
            )
            .unwrap()
            .claim;
        assert_eq!(first.continuity_id, "continuity-1");
        assert_eq!(first.workpoint_id, second.workpoint_id);
        assert_eq!(first.agent_session_id.as_deref(), Some("session-1"));
        assert_eq!(first.idempotency_key, second.idempotency_key);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn scoped_run_replays_reconciled_claim_without_double_submit() {
        let root = std::env::temp_dir().join(format!("focusa-idempotent-{}", Uuid::now_v7()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "fn proven() {}\n").unwrap();
        std::fs::write(root.join("tests/proof.rs"), "// proof\n").unwrap();
        let mut registry = ProviderRegistry::empty();
        registry.register(Arc::new(NoneAdapter::new()));
        let mut policy = ClosurePolicy::default_policy();
        policy.active_profile = "code_with_test".into();
        let lifecycle = Lifecycle::new(
            ClaimStorage::new(root.join("claims")),
            ClosureAuditLog::open(root.join("audit.jsonl")).unwrap(),
            policy,
            ClosureProfile::all_builtins(),
            registry,
        );
        let work_item = WorkItemRef {
            provider: WorkItemProvider::None,
            provider_item_id: "local-1".into(),
            project_root: root.clone(),
            external_url: None,
        };
        let authority = ClosureAuthorityContext {
            continuity_id: "continuity-1".into(),
            workpoint_id: Uuid::now_v7().to_string(),
            agent_session_id: Some("session-1".into()),
        };
        let citations = vec![
            sample_cite(EvidenceKind::Code, "src/lib.rs"),
            sample_cite(EvidenceKind::Test, "tests/proof.rs"),
        ];
        let first = lifecycle
            .run_scoped(
                "agent",
                work_item.clone(),
                "verified",
                ClosureKind::Code,
                citations.clone(),
                authority.clone(),
            )
            .await
            .unwrap();
        let snapshots_after_first = lifecycle.storage.list().unwrap().len();
        let second = lifecycle
            .run_scoped(
                "agent",
                work_item,
                "verified",
                ClosureKind::Code,
                citations,
                authority,
            )
            .await
            .unwrap();
        assert_eq!(first.claim_id, second.claim_id);
        assert_eq!(second.status, ClaimStatus::Reconciled);
        assert_eq!(
            lifecycle.storage.list().unwrap().len(),
            snapshots_after_first
        );
        assert_eq!(
            lifecycle
                .storage
                .list()
                .unwrap()
                .iter()
                .filter(|claim_id| !claim_id.contains('.'))
                .count(),
            1
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn spec79_canary_advances_ordered_bd_graph_without_manual_reprompt() {
        use crate::work_item::{BdAdapter, ProviderAdapter, WorkItemQuery, select_next_ready};
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!("focusa-spec79-canary-{}", Uuid::now_v7()));
        std::fs::create_dir_all(root.join(".beads")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(root.join(".beads/issues.jsonl"), "").unwrap();
        std::fs::write(root.join("src/lib.rs"), "fn canary() {}\n").unwrap();
        std::fs::write(root.join("tests/proof.rs"), "// passing canary proof\n").unwrap();
        std::fs::write(
            root.join("state.json"),
            serde_json::to_vec(&serde_json::json!([
                {"id":"root","title":"root","status":"closed","priority":0,
                 "dependents":[{"id":"a","dependency_type":"parent-child"},
                               {"id":"b","dependency_type":"parent-child"}]},
                {"id":"a","title":"first","status":"open","priority":0,
                 "dependencies":[{"depends_on_id":"root","type":"parent-child"}]},
                {"id":"b","title":"second","status":"open","priority":1,
                 "dependencies":[{"depends_on_id":"root","type":"parent-child"},
                                 {"depends_on_id":"a","type":"blocks"}]}
            ]))
            .unwrap(),
        )
        .unwrap();
        let script = root.join("fake-bd");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
[ "${1:-}" = "--no-db" ] && shift
case "${1:-}" in
  --version) echo 'bd fake 1';;
  list) cat "$DIR/state.json";;
  show) python3 - "$DIR/state.json" "$@" <<'PY'
import json,sys
xs=json.load(open(sys.argv[1])); ids=set(sys.argv[2:])
print(json.dumps([x for x in xs if x['id'] in ids]))
PY
  ;;
  close) python3 - "$DIR/state.json" "$2" <<'PY'
import json,sys
p=sys.argv[1]; xs=json.load(open(p))
for x in xs:
 if x['id']==sys.argv[2]: x['status']='closed'
open(p,'w').write(json.dumps(xs))
PY
  ;;
  *) echo 'unsupported fake bd command' >&2; exit 2;;
esac
"#,
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();

        let adapter = Arc::new(BdAdapter::with_bd_path(script.to_string_lossy()));
        let mut registry = ProviderRegistry::empty();
        registry.register(adapter.clone());
        let mut policy = ClosurePolicy::default_policy();
        policy.active_profile = "code_with_test".into();
        let lifecycle = Lifecycle::new(
            ClaimStorage::new(root.join("claims")),
            ClosureAuditLog::open(root.join("audit.jsonl")).unwrap(),
            policy,
            ClosureProfile::all_builtins(),
            registry,
        );
        let query = WorkItemQuery {
            project_root: root.clone(),
            parent: Some(WorkItemRef {
                provider: WorkItemProvider::Bd,
                provider_item_id: "root".into(),
                project_root: root.clone(),
                external_url: None,
            }),
            limit: 100,
        };
        let authority = ClosureAuthorityContext {
            continuity_id: "canary-continuity".into(),
            workpoint_id: Uuid::now_v7().to_string(),
            agent_session_id: Some("pi-rpc-canary".into()),
        };
        let citations = vec![
            sample_cite(EvidenceKind::Code, "src/lib.rs"),
            sample_cite(EvidenceKind::Test, "tests/proof.rs"),
        ];
        let mut order = Vec::new();
        while let Some(item) = select_next_ready(&adapter.list(&query).await.unwrap(), &query) {
            order.push(item.provider_item_id.clone());
            lifecycle
                .run_scoped(
                    "focusa-work-loop-canary",
                    item.reference(),
                    &format!("verified {}", item.provider_item_id),
                    ClosureKind::Code,
                    citations.clone(),
                    authority.clone(),
                )
                .await
                .unwrap();
        }
        assert_eq!(order, vec!["a", "b"]);
        assert!(
            adapter
                .list(&query)
                .await
                .unwrap()
                .iter()
                .all(WorkItem::is_terminal)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn scoped_prepare_rejects_missing_workpoint_authority() {
        let lifecycle = Lifecycle::open_for_kind(ClosureKind::Code);
        let error = lifecycle
            .prepare_scoped(
                "agent",
                WorkItemRef {
                    provider: WorkItemProvider::None,
                    provider_item_id: "local-1".into(),
                    project_root: PathBuf::from("/tmp/project"),
                    external_url: None,
                },
                "verified",
                ClosureKind::Code,
                vec![],
                ClosureAuthorityContext {
                    continuity_id: "continuity-1".into(),
                    workpoint_id: String::new(),
                    agent_session_id: None,
                },
            )
            .unwrap_err();
        assert_eq!(error.code, "closure_scope_required");
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
