//! Exact requested/effective/observed model safety for Spec133.

use std::collections::BTreeSet;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExactModelBinding {
    pub provider: String,
    pub model: String,
    pub thinking_level: String,
    pub auth_profile_ref: String,
}

impl ExactModelBinding {
    pub fn validate(&self) -> anyhow::Result<()> {
        for value in [
            &self.provider,
            &self.model,
            &self.thinking_level,
            &self.auth_profile_ref,
        ] {
            anyhow::ensure!(!value.trim().is_empty(), "model binding field is empty");
            anyhow::ensure!(
                value.len() <= 512,
                "model binding field exceeds bounded length"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelPreflightEvidence {
    pub binding: ExactModelBinding,
    pub entitlement_verified: bool,
    pub catalog_verified: bool,
    pub context_window_verified: bool,
    pub rate_limit_verified: bool,
    pub budget_verified: bool,
    pub auth_verified: bool,
    pub observed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

impl ModelPreflightEvidence {
    pub fn verify(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        self.binding.validate()?;
        anyhow::ensure!(
            now >= self.observed_at && now < self.expires_at,
            "model preflight evidence is stale"
        );
        anyhow::ensure!(
            self.entitlement_verified,
            "model entitlement preflight failed"
        );
        anyhow::ensure!(self.catalog_verified, "model catalog preflight failed");
        anyhow::ensure!(
            self.context_window_verified,
            "model context-window preflight failed"
        );
        anyhow::ensure!(
            self.rate_limit_verified,
            "model rate-limit preflight failed"
        );
        anyhow::ensure!(self.budget_verified, "model budget preflight failed");
        anyhow::ensure!(self.auth_verified, "model auth-profile preflight failed");
        anyhow::ensure!(
            !self.evidence_refs.is_empty(),
            "model preflight requires evidence"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelRuntimeConfirmation {
    pub requested: ExactModelBinding,
    pub effective: ExactModelBinding,
    pub observed: ExactModelBinding,
    pub confirmed_at: DateTime<Utc>,
    pub evidence_ref: String,
}

impl ModelRuntimeConfirmation {
    pub fn verify_exact(&self) -> anyhow::Result<()> {
        self.requested.validate()?;
        self.effective.validate()?;
        self.observed.validate()?;
        anyhow::ensure!(
            self.requested == self.effective,
            "requested model differs from effective model"
        );
        anyhow::ensure!(
            self.effective == self.observed,
            "effective model differs from observed runtime model"
        );
        anyhow::ensure!(
            !self.evidence_ref.trim().is_empty(),
            "runtime model confirmation requires evidence"
        );
        Ok(())
    }

    pub fn authorize_project_mutation(&self) -> anyhow::Result<()> {
        self.verify_exact()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSwitchCheckpoint {
    pub prior_binding: ExactModelBinding,
    pub requested_binding: ExactModelBinding,
    pub runtime_checkpoint_ref: String,
    pub workpoint_checkpoint_ref: String,
    pub bootstrap_packet_ref: String,
    pub created_at: DateTime<Utc>,
}

impl ModelSwitchCheckpoint {
    pub fn verify(&self) -> anyhow::Result<()> {
        self.prior_binding.validate()?;
        self.requested_binding.validate()?;
        for reference in [
            &self.runtime_checkpoint_ref,
            &self.workpoint_checkpoint_ref,
            &self.bootstrap_packet_ref,
        ] {
            anyhow::ensure!(
                !reference.trim().is_empty(),
                "model switch checkpoint reference is empty"
            );
        }
        anyhow::ensure!(
            self.prior_binding != self.requested_binding,
            "model switch does not change the exact binding"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedModelFallbackPolicy {
    pub enabled: bool,
    pub allowed_bindings: BTreeSet<ExactModelBinding>,
    pub allowed_trigger_classes: BTreeSet<String>,
    pub max_fallbacks: u32,
}

impl GovernedModelFallbackPolicy {
    pub fn authorize(
        &self,
        target: &ExactModelBinding,
        trigger_class: &str,
        prior_fallbacks: u32,
        target_preflight: &ModelPreflightEvidence,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(self.enabled, "model fallback is disabled");
        anyhow::ensure!(
            prior_fallbacks < self.max_fallbacks,
            "model fallback budget exhausted"
        );
        anyhow::ensure!(
            self.allowed_bindings.contains(target),
            "fallback model is not explicitly allowlisted"
        );
        anyhow::ensure!(
            self.allowed_trigger_classes.contains(trigger_class),
            "fallback trigger class is not explicitly allowlisted"
        );
        target_preflight.verify(now)?;
        anyhow::ensure!(
            &target_preflight.binding == target,
            "fallback preflight does not match the allowlisted target"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedModelState {
    pub requested: ExactModelBinding,
    pub effective: Option<ExactModelBinding>,
    pub observed: Option<ExactModelBinding>,
    pub mismatch_abort_required: bool,
}

impl GovernedModelState {
    pub fn apply_preflight(
        &mut self,
        evidence: &ModelPreflightEvidence,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        evidence.verify(now)?;
        anyhow::ensure!(
            evidence.binding == self.requested,
            "preflight binding differs from requested model"
        );
        self.effective = Some(evidence.binding.clone());
        Ok(())
    }

    pub fn observe_runtime(
        &mut self,
        observed: ExactModelBinding,
        evidence_ref: impl Into<String>,
    ) -> anyhow::Result<ModelRuntimeConfirmation> {
        let effective = self
            .effective
            .clone()
            .ok_or_else(|| anyhow::anyhow!("runtime model observed before preflight"))?;
        let confirmation = ModelRuntimeConfirmation {
            requested: self.requested.clone(),
            effective,
            observed: observed.clone(),
            confirmed_at: Utc::now(),
            evidence_ref: evidence_ref.into(),
        };
        self.observed = Some(observed);
        if let Err(error) = confirmation.verify_exact() {
            self.mismatch_abort_required = true;
            return Err(error.context("abort exact run before project mutation"));
        }
        self.mismatch_abort_required = false;
        Ok(confirmation)
    }
}
