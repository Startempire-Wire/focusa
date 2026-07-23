//! Typed retry, adoption, reconnect, and reboot-relaunch policy.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    Provider,
    Transport,
    Harness,
    Tool,
    Model,
    Runner,
    WorkItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryBudget {
    pub max_attempts: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryLedger {
    pub budgets: BTreeMap<RetryClass, RetryBudget>,
    pub attempts: BTreeMap<RetryClass, u32>,
}

impl RetryLedger {
    pub fn authorize(&mut self, class: RetryClass) -> anyhow::Result<u64> {
        let budget = self
            .budgets
            .get(&class)
            .ok_or_else(|| anyhow::anyhow!("retry class has no explicit budget"))?;
        let attempts = self.attempts.entry(class).or_default();
        anyhow::ensure!(
            *attempts < budget.max_attempts,
            "independent retry budget exhausted"
        );
        let delay = budget
            .base_backoff_ms
            .saturating_mul(2u64.saturating_pow(*attempts))
            .min(budget.max_backoff_ms);
        *attempts += 1;
        Ok(delay)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdoptionEvidence {
    pub process_identity_matches: bool,
    pub manifest_hash_matches: bool,
    pub user_matches: bool,
    pub workspace_matches: bool,
    pub heartbeat_authenticated: bool,
}

impl AdoptionEvidence {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.process_identity_matches
                && self.manifest_hash_matches
                && self.user_matches
                && self.workspace_matches
                && self.heartbeat_authenticated,
            "unknown process rejected; adoption identity barrier did not pass"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebootRelaunchPolicy {
    pub automatic_relaunch: bool,
    pub operator_approval_required: bool,
    pub allowed_failure_classes: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebootRecoveryEvidence {
    pub durable_state_restored: bool,
    pub prior_run_classified_orphaned: bool,
    pub runtime_checkpoint_ref: String,
    pub workpoint_checkpoint_ref: String,
    pub operator_approved: bool,
    pub failure_class: String,
    pub next_generation: u64,
}

impl RebootRelaunchPolicy {
    pub fn authorize(&self, evidence: &RebootRecoveryEvidence) -> anyhow::Result<()> {
        anyhow::ensure!(
            evidence.durable_state_restored && evidence.prior_run_classified_orphaned,
            "reboot state was not durably reconciled"
        );
        anyhow::ensure!(
            !evidence.runtime_checkpoint_ref.is_empty()
                && !evidence.workpoint_checkpoint_ref.is_empty(),
            "relaunch requires runtime and Workpoint checkpoints"
        );
        anyhow::ensure!(
            evidence.next_generation > 1,
            "relaunch must create a new run generation"
        );
        anyhow::ensure!(
            self.automatic_relaunch || evidence.operator_approved,
            "operator approval required for relaunch"
        );
        anyhow::ensure!(
            !self.operator_approval_required || evidence.operator_approved,
            "relaunch policy requires operator approval"
        );
        anyhow::ensure!(
            self.allowed_failure_classes
                .contains(&evidence.failure_class),
            "failure class is not relaunch-allowlisted"
        );
        Ok(())
    }
}
