//! Provider/model preflight and runtime mutation-barrier policy for Spec 133 §16.
//!
//! Adapters collect evidence; this module evaluates it deterministically. It
//! never discovers credentials, selects an ambient provider default, or treats
//! a physical runner launch as permission to mutate a project.

use crate::contract::{HarnessCapabilities, PreflightStatus};
use chrono::{DateTime, Utc};
use focusa_core::silent_session::{
    ModelBinding, ModelFallbackPolicy, ModelSelectionPolicy, SilentSessionModelConfig,
    SilentSessionRun,
};
use focusa_core::silent_session_protocol::{CapabilityRequirement, CapabilitySupport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const MODEL_PREFLIGHT_EVIDENCE_SCHEMA: &str = "focusa.model_preflight_evidence.v1";
pub const MODEL_PREFLIGHT_VERDICT_SCHEMA: &str = "focusa.model_preflight_verdict.v1";
pub const MODEL_RUNTIME_CONFIRMATION_SCHEMA: &str = "focusa.model_runtime_confirmation.v1";
pub const MODEL_SWITCH_PROOF_SCHEMA: &str = "focusa.model_switch_proof.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCheckKind {
    ProviderConfigured,
    AuthenticationAvailable,
    AuthenticationType,
    SubscriptionOrApiEntitlement,
    ExactModelAvailable,
    ThinkingLevelSupported,
    ContextWindowCompatible,
    RateLimitPosture,
    BillingOrUsageBudget,
    ModelCatalogFreshness,
}

pub const ALL_PROVIDER_CHECKS: [ProviderCheckKind; 10] = [
    ProviderCheckKind::ProviderConfigured,
    ProviderCheckKind::AuthenticationAvailable,
    ProviderCheckKind::AuthenticationType,
    ProviderCheckKind::SubscriptionOrApiEntitlement,
    ProviderCheckKind::ExactModelAvailable,
    ProviderCheckKind::ThinkingLevelSupported,
    ProviderCheckKind::ContextWindowCompatible,
    ProviderCheckKind::RateLimitPosture,
    ProviderCheckKind::BillingOrUsageBudget,
    ProviderCheckKind::ModelCatalogFreshness,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCheckEvidence {
    pub kind: ProviderCheckKind,
    pub status: PreflightStatus,
    pub source_ref: String,
    pub observed_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPreflightEvidence {
    pub schema: String,
    pub candidate: ModelBinding,
    pub auth_profile_ref: String,
    pub checks: Vec<ProviderCheckEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFallbackTrigger {
    ProviderUnavailable,
    ModelUnavailable,
    ContextIncompatible,
    RateLimited,
    BudgetPressure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelFallbackAttempt {
    pub trigger: ModelFallbackTrigger,
    pub trigger_evidence_ref: String,
    pub candidate: ModelBinding,
    pub operator_notification_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPreflightVerdict {
    pub schema: String,
    pub status: PreflightStatus,
    pub requested: ModelBinding,
    pub selected: Option<ModelBinding>,
    pub fallback: Option<ModelFallbackAttempt>,
    pub blocking_checks: Vec<ProviderCheckKind>,
    pub degraded_checks: Vec<ProviderCheckKind>,
    pub event_kind: String,
    pub launch_allowed: bool,
    pub mutation_allowed: bool,
    pub operator_notification_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelSafetyError {
    #[error("model preflight evidence schema is unsupported")]
    UnsupportedEvidenceSchema,
    #[error("model preflight evidence does not match the selected model or auth profile")]
    EvidenceScopeMismatch,
    #[error("model preflight evidence must contain every check exactly once")]
    IncompleteEvidence,
    #[error("model preflight evidence is stale, future-dated, or missing provenance")]
    InvalidEvidenceFreshness,
    #[error("model fallback is disabled")]
    FallbackDisabled,
    #[error("model fallback candidate is not in the explicit allowlist")]
    FallbackNotAllowlisted,
    #[error("model fallback requires explicit trigger evidence and operator notification")]
    IncompleteFallbackAuthority,
    #[error("exact model selection cannot use fallback")]
    ExactSelectionForbidsFallback,
}

/// Evaluate all provider checks for the requested binding or one explicitly
/// allowlisted fallback. Optional unknown checks degrade truthfully; blocked
/// checks and unknown strict requirements fail closed.
pub fn evaluate_model_preflight(
    config: &SilentSessionModelConfig,
    capabilities: &HarnessCapabilities,
    evidence: &ModelPreflightEvidence,
    fallback: Option<&ModelFallbackAttempt>,
    now: DateTime<Utc>,
) -> Result<ModelPreflightVerdict, ModelSafetyError> {
    validate_fallback(config, fallback)?;
    let selected = fallback
        .map(|attempt| &attempt.candidate)
        .unwrap_or(&config.requested);
    validate_evidence(config, evidence, selected, now)?;

    let mut blocking_checks = Vec::new();
    let mut degraded_checks = Vec::new();
    for check in &evidence.checks {
        let required = check_required(config, capabilities, selected, check.kind);
        match check.status {
            PreflightStatus::Passed => {}
            PreflightStatus::Blocked => blocking_checks.push(check.kind),
            PreflightStatus::Unknown if required => blocking_checks.push(check.kind),
            PreflightStatus::Degraded if required => blocking_checks.push(check.kind),
            PreflightStatus::Unknown | PreflightStatus::Degraded => {
                degraded_checks.push(check.kind)
            }
        }
    }

    let status = if !blocking_checks.is_empty() {
        PreflightStatus::Blocked
    } else if !degraded_checks.is_empty() {
        PreflightStatus::Degraded
    } else {
        PreflightStatus::Passed
    };
    let launch_allowed = status != PreflightStatus::Blocked;
    let event_kind = match status {
        PreflightStatus::Passed => "model.preflight_passed",
        PreflightStatus::Blocked => "model.preflight_blocked",
        PreflightStatus::Degraded => "model.preflight_degraded",
        PreflightStatus::Unknown => "model.preflight_unknown",
    };

    Ok(ModelPreflightVerdict {
        schema: MODEL_PREFLIGHT_VERDICT_SCHEMA.into(),
        status,
        requested: config.requested.clone(),
        selected: launch_allowed.then(|| selected.clone()),
        fallback: fallback.cloned(),
        blocking_checks,
        degraded_checks,
        event_kind: event_kind.into(),
        launch_allowed,
        // Runtime model observation, bootstrap, lease, and Context Authority
        // are separate mandatory barriers.
        mutation_allowed: false,
        operator_notification_required: fallback.is_some()
            || matches!(status, PreflightStatus::Blocked | PreflightStatus::Degraded),
    })
}

fn validate_fallback(
    config: &SilentSessionModelConfig,
    fallback: Option<&ModelFallbackAttempt>,
) -> Result<(), ModelSafetyError> {
    let Some(fallback) = fallback else {
        return Ok(());
    };
    if config.selection_policy == ModelSelectionPolicy::Exact {
        return Err(ModelSafetyError::ExactSelectionForbidsFallback);
    }
    if config.fallback_policy != ModelFallbackPolicy::ExplicitAllowList {
        return Err(ModelSafetyError::FallbackDisabled);
    }
    if !config.allowed_fallbacks.contains(&fallback.candidate)
        || fallback.candidate == config.requested
    {
        return Err(ModelSafetyError::FallbackNotAllowlisted);
    }
    if fallback.trigger_evidence_ref.trim().is_empty()
        || fallback.operator_notification_ref.trim().is_empty()
    {
        return Err(ModelSafetyError::IncompleteFallbackAuthority);
    }
    Ok(())
}

fn validate_evidence(
    config: &SilentSessionModelConfig,
    evidence: &ModelPreflightEvidence,
    selected: &ModelBinding,
    now: DateTime<Utc>,
) -> Result<(), ModelSafetyError> {
    if evidence.schema != MODEL_PREFLIGHT_EVIDENCE_SCHEMA {
        return Err(ModelSafetyError::UnsupportedEvidenceSchema);
    }
    if &evidence.candidate != selected || evidence.auth_profile_ref != config.auth_profile_ref {
        return Err(ModelSafetyError::EvidenceScopeMismatch);
    }
    let observed: BTreeSet<_> = evidence.checks.iter().map(|check| check.kind).collect();
    let required: BTreeSet<_> = ALL_PROVIDER_CHECKS.into_iter().collect();
    if observed != required || evidence.checks.len() != ALL_PROVIDER_CHECKS.len() {
        return Err(ModelSafetyError::IncompleteEvidence);
    }
    if evidence.checks.iter().any(|check| {
        check.source_ref.trim().is_empty()
            || check.detail.trim().is_empty()
            || check.observed_at > now
            || check.fresh_until <= now
            || check.fresh_until < check.observed_at
    }) {
        return Err(ModelSafetyError::InvalidEvidenceFreshness);
    }
    Ok(())
}

fn check_required(
    config: &SilentSessionModelConfig,
    capabilities: &HarnessCapabilities,
    selected: &ModelBinding,
    kind: ProviderCheckKind,
) -> bool {
    match kind {
        ProviderCheckKind::ProviderConfigured
        | ProviderCheckKind::AuthenticationAvailable
        | ProviderCheckKind::AuthenticationType
        | ProviderCheckKind::ExactModelAvailable => true,
        ProviderCheckKind::SubscriptionOrApiEntitlement => config.require_entitlement_preflight,
        ProviderCheckKind::ThinkingLevelSupported => selected.thinking.is_some(),
        ProviderCheckKind::ContextWindowCompatible => capabilities
            .model_preflight
            .satisfies(CapabilityRequirement::Deterministic),
        ProviderCheckKind::RateLimitPosture
        | ProviderCheckKind::BillingOrUsageBudget
        | ProviderCheckKind::ModelCatalogFreshness => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelRunTransitionError {
    #[error("model transition does not match the run's requested binding")]
    RequestedBindingMismatch,
    #[error("model preflight verdict is malformed or not launchable")]
    PreflightNotLaunchable,
    #[error("runtime model confirmation does not match the run's effective binding")]
    EffectiveBindingMismatch,
    #[error("runtime model confirmation schema is unsupported")]
    UnsupportedConfirmationSchema,
}

/// Persist the selected binding before launch. Observation remains absent until
/// the live harness reports it; preflight alone can never synthesize runtime
/// truth.
pub fn apply_model_preflight_to_run(
    run: &mut SilentSessionRun,
    verdict: &ModelPreflightVerdict,
) -> Result<(), ModelRunTransitionError> {
    if run.requested_model_binding != verdict.requested {
        return Err(ModelRunTransitionError::RequestedBindingMismatch);
    }
    if !verdict.launch_allowed || verdict.selected.is_none() {
        run.effective_model_binding = None;
        run.observed_model_binding = None;
        return Err(ModelRunTransitionError::PreflightNotLaunchable);
    }
    run.effective_model_binding = verdict.selected.clone();
    run.observed_model_binding = None;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeModelStatus {
    Confirmed,
    Mismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeModelConfirmation {
    pub schema: String,
    pub status: RuntimeModelStatus,
    pub requested: ModelBinding,
    pub effective: Option<ModelBinding>,
    pub observed: Option<ModelBinding>,
    pub event_kind: String,
    pub mutation_allowed: bool,
    pub controlled_abort_required: bool,
    pub blocked_state_required: bool,
    pub operator_notification_required: bool,
    pub reasons: Vec<String>,
}

pub struct RuntimeModelConfirmationRequest<'a> {
    pub config: &'a SilentSessionModelConfig,
    pub preflight: &'a ModelPreflightVerdict,
    pub effective: Option<&'a ModelBinding>,
    pub observed: Option<&'a ModelBinding>,
    pub harness_connected: bool,
    pub bootstrap_verified: bool,
    pub writer_lease_valid: bool,
    pub context_authority_fresh: bool,
}

/// Confirm model truth at runtime. Any mismatch returns a complete controlled
/// abort/block/notify directive rather than a permissive boolean.
pub fn confirm_runtime_model(
    request: &RuntimeModelConfirmationRequest<'_>,
) -> RuntimeModelConfirmation {
    let mut reasons = Vec::new();
    if !request.preflight.launch_allowed {
        reasons.push("provider_preflight_not_launchable".into());
    }
    if !request.harness_connected {
        reasons.push("harness_not_connected".into());
    }
    if !request.bootstrap_verified {
        reasons.push("bootstrap_not_verified".into());
    }
    if !request.writer_lease_valid {
        reasons.push("writer_lease_invalid".into());
    }
    if !request.context_authority_fresh {
        reasons.push("context_authority_stale".into());
    }
    let expected = request.preflight.selected.as_ref();
    if request.effective != expected {
        reasons.push("effective_model_mismatch".into());
    }
    if request.observed != expected {
        reasons.push("observed_model_mismatch".into());
    }
    if request.config.selection_policy == ModelSelectionPolicy::Exact
        && (request.effective != Some(&request.config.requested)
            || request.observed != Some(&request.config.requested)
            || request.preflight.fallback.is_some())
    {
        reasons.push("exact_model_policy_mismatch".into());
    }
    if request.config.require_runtime_model_confirmation && request.observed.is_none() {
        reasons.push("runtime_model_unobserved".into());
    }

    let confirmed = reasons.is_empty();
    RuntimeModelConfirmation {
        schema: MODEL_RUNTIME_CONFIRMATION_SCHEMA.into(),
        status: if confirmed {
            RuntimeModelStatus::Confirmed
        } else {
            RuntimeModelStatus::Mismatch
        },
        requested: request.config.requested.clone(),
        effective: request.effective.cloned(),
        observed: request.observed.cloned(),
        event_kind: if confirmed {
            "model.confirmed".into()
        } else {
            "model.mismatch".into()
        },
        mutation_allowed: confirmed,
        controlled_abort_required: !confirmed,
        blocked_state_required: !confirmed,
        operator_notification_required: !confirmed,
        reasons,
    }
}

/// Persist the observed binding even on mismatch so recovery and audit cannot
/// erase why the mutation barrier aborted the run.
pub fn apply_runtime_confirmation_to_run(
    run: &mut SilentSessionRun,
    confirmation: &RuntimeModelConfirmation,
) -> Result<(), ModelRunTransitionError> {
    if confirmation.schema != MODEL_RUNTIME_CONFIRMATION_SCHEMA {
        return Err(ModelRunTransitionError::UnsupportedConfirmationSchema);
    }
    if run.requested_model_binding != confirmation.requested {
        return Err(ModelRunTransitionError::RequestedBindingMismatch);
    }
    if run.effective_model_binding != confirmation.effective {
        return Err(ModelRunTransitionError::EffectiveBindingMismatch);
    }
    run.observed_model_binding = confirmation.observed.clone();
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSwitchProof {
    pub schema: String,
    pub checkpoint_ref: String,
    pub checkpoint_reason: String,
    pub config_revision_ref: String,
    pub prior_generation: u64,
    pub next_generation: u64,
    pub safe_in_place_switch_proof_ref: Option<String>,
    pub preflight_ref: String,
    pub refreshed_bootstrap_ref: String,
    pub runtime_confirmation_ref: String,
    pub event_ref: String,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelSwitchProofError {
    #[error("model switch proof schema is unsupported")]
    UnsupportedSchema,
    #[error("model switch requires a model_switch checkpoint and complete linkage")]
    MissingLinkage,
    #[error("model switch requires a new run generation unless safe in-place switching is proven")]
    GenerationNotAdvanced,
}

pub fn validate_model_switch_proof(proof: &ModelSwitchProof) -> Result<(), ModelSwitchProofError> {
    if proof.schema != MODEL_SWITCH_PROOF_SCHEMA {
        return Err(ModelSwitchProofError::UnsupportedSchema);
    }
    if proof.checkpoint_reason != "model_switch"
        || [
            &proof.checkpoint_ref,
            &proof.config_revision_ref,
            &proof.preflight_ref,
            &proof.refreshed_bootstrap_ref,
            &proof.runtime_confirmation_ref,
            &proof.event_ref,
            &proof.receipt_ref,
        ]
        .into_iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(ModelSwitchProofError::MissingLinkage);
    }
    if proof.next_generation <= proof.prior_generation
        && proof
            .safe_in_place_switch_proof_ref
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return Err(ModelSwitchProofError::GenerationNotAdvanced);
    }
    Ok(())
}

pub fn entitlement_capability_is_sufficient(
    config: &SilentSessionModelConfig,
    support: CapabilitySupport,
) -> bool {
    !config.require_entitlement_preflight || support.satisfies(CapabilityRequirement::Deterministic)
}
