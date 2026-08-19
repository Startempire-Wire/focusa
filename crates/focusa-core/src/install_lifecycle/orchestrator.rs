use super::{
    LifecycleEntitlementDecision, LifecycleEntitlementReceiptClass, LifecycleScope, LifecycleState,
    PreservationDeclaration,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

fn is_home_dev_bypass() -> bool {
    if std::env::var("FOCUSA_ACTIVATION_BYPASS_DISABLE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return false;
    }
    if std::env::var("FOCUSA_DEV_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("FOCUSA_TEST_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("FOCUSA_HOME_SERVER")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    false
}

const GENESIS_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleOperation {
    Inspect,
    Install,
    Repair,
    Rerun,
    Update,
    Rollback,
    Uninstall,
    Purge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleOperationRequest {
    pub transaction_id: String,
    pub operation: LifecycleOperation,
    pub scope: LifecycleScope,
    pub idempotency_key: String,
    pub selected_version: String,
    pub artifact_signature_verified: bool,
    pub preservation: PreservationDeclaration,
    pub purge_confirmed_separately: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub recovery_safe: bool,
    #[serde(default)]
    pub unattended: bool,
    #[serde(default)]
    pub selected_product: String,
    #[serde(default)]
    pub selected_channel: String,
    #[serde(default)]
    pub required_features: BTreeSet<String>,
    #[serde(default)]
    pub entitlement: Option<LifecycleEntitlementDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleJournalEntry {
    pub sequence: u64,
    pub transaction_id: String,
    pub operation: LifecycleOperation,
    pub prior_state: LifecycleState,
    pub new_state: LifecycleState,
    pub action: String,
    pub evidence_refs: Vec<String>,
    pub previous_hash: String,
    pub entry_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteVersionSet {
    pub cli_version: String,
    pub daemon_version: String,
    pub api_version: String,
    pub pi_extension_version: String,
    pub schema_version: String,
}

impl CompleteVersionSet {
    pub fn coherent(&self) -> bool {
        !self.cli_version.is_empty()
            && self.cli_version == self.daemon_version
            && self.cli_version == self.api_version
            && self.cli_version == self.pi_extension_version
            && !self.schema_version.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleAcceptanceReceipt {
    pub transaction_id: String,
    pub operation: LifecycleOperation,
    pub scope: LifecycleScope,
    pub final_state: LifecycleState,
    pub journal_head_hash: String,
    pub version_set: CompleteVersionSet,
    pub daemon_service_healthy: bool,
    pub project_verified: bool,
    pub bootstrap_committed: bool,
    pub genesis_committed: bool,
    pub first_workpoint_id: Option<String>,
    pub preserved_data_classes: BTreeSet<String>,
    pub entitlement_receipt_class: LifecycleEntitlementReceiptClass,
    pub entitlement_binding: Option<super::LifecycleEntitlementBinding>,
    pub entitlement_evidence_refs: Vec<String>,
    pub closure_allowed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LifecycleOrchestratorError {
    #[error("transaction id and idempotency key are required")]
    MissingAuthority,
    #[error("install or update artifact signature is not verified")]
    ArtifactTrustRequired,
    #[error("purge requires separate explicit confirmation")]
    PurgeConfirmationRequired,
    #[error("lifecycle preservation declaration is invalid")]
    InvalidPreservation,
    #[error("lifecycle journal sequence or hash chain is invalid")]
    JournalIntegrityFailure,
    #[error("idempotency key conflicts with a different operation")]
    IdempotencyConflict,
    #[error("complete installed version set is incoherent")]
    VersionSetIncoherent,
    #[error("first Workpoint acceptance evidence is incomplete")]
    FirstWorkpointNotAccepted,
    #[error("canonical entitlement decision is required before product mutation")]
    EntitlementRequired,
    #[error("signed entitlement does not grant the selected product")]
    ProductGrantRequired,
    #[error("signed entitlement does not grant every required lifecycle feature")]
    FeatureGrantRequired,
    #[error("entitlement state or signed time boundary blocks product execution")]
    EntitlementBlocked,
}

pub fn append_lifecycle_transition(
    journal: &mut Vec<LifecycleJournalEntry>,
    request: &LifecycleOperationRequest,
    prior_state: LifecycleState,
    new_state: LifecycleState,
    action: impl Into<String>,
    evidence_refs: Vec<String>,
) -> Result<(), LifecycleOrchestratorError> {
    validate_request(request)?;
    verify_journal(journal)?;
    let previous_hash = journal
        .last()
        .map(|entry| entry.entry_hash.clone())
        .unwrap_or_else(|| GENESIS_HASH.into());
    let sequence = journal.len() as u64 + 1;
    let action = action.into();
    let entry_hash = digest(&(
        sequence,
        &request.transaction_id,
        request.operation,
        prior_state,
        new_state,
        &action,
        &evidence_refs,
        &previous_hash,
    ));
    journal.push(LifecycleJournalEntry {
        sequence,
        transaction_id: request.transaction_id.clone(),
        operation: request.operation,
        prior_state,
        new_state,
        action,
        evidence_refs,
        previous_hash,
        entry_hash,
    });
    Ok(())
}

pub fn verify_journal(journal: &[LifecycleJournalEntry]) -> Result<(), LifecycleOrchestratorError> {
    let mut previous = GENESIS_HASH.to_string();
    for (index, entry) in journal.iter().enumerate() {
        let expected = digest(&(
            entry.sequence,
            &entry.transaction_id,
            entry.operation,
            entry.prior_state,
            entry.new_state,
            &entry.action,
            &entry.evidence_refs,
            &entry.previous_hash,
        ));
        if entry.sequence != index as u64 + 1
            || entry.previous_hash != previous
            || entry.entry_hash != expected
        {
            return Err(LifecycleOrchestratorError::JournalIntegrityFailure);
        }
        previous.clone_from(&entry.entry_hash);
    }
    Ok(())
}

pub fn resume_state(
    request: &LifecycleOperationRequest,
    journal: &[LifecycleJournalEntry],
) -> Result<LifecycleState, LifecycleOrchestratorError> {
    validate_request(request)?;
    verify_journal(journal)?;
    if let Some(entry) = journal.iter().find(|entry| {
        entry.transaction_id == request.transaction_id && entry.operation != request.operation
    }) {
        let _ = entry;
        return Err(LifecycleOrchestratorError::IdempotencyConflict);
    }
    Ok(journal
        .iter()
        .rev()
        .find(|entry| entry.transaction_id == request.transaction_id)
        .map(|entry| entry.new_state)
        .unwrap_or(LifecycleState::Uninspected))
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_lifecycle(
    request: &LifecycleOperationRequest,
    journal: &[LifecycleJournalEntry],
    version_set: CompleteVersionSet,
    daemon_service_healthy: bool,
    project_verified: bool,
    bootstrap_committed: bool,
    genesis_committed: bool,
    first_workpoint_id: Option<String>,
) -> Result<LifecycleAcceptanceReceipt, LifecycleOrchestratorError> {
    validate_request(request)?;
    verify_journal(journal)?;
    if !version_set.coherent() {
        return Err(LifecycleOrchestratorError::VersionSetIncoherent);
    }
    let operation_requires_project = matches!(
        request.operation,
        LifecycleOperation::Install | LifecycleOperation::Repair | LifecycleOperation::Rerun
    );
    if operation_requires_project
        && (!daemon_service_healthy
            || !project_verified
            || !bootstrap_committed
            || !genesis_committed
            || first_workpoint_id.as_deref().is_none_or(str::is_empty))
    {
        return Err(LifecycleOrchestratorError::FirstWorkpointNotAccepted);
    }
    let final_state = journal
        .last()
        .map(|entry| entry.new_state)
        .unwrap_or(LifecycleState::Uninspected);
    let preserved_data_classes = request
        .preservation
        .items
        .iter()
        .filter(|item| format!("{:?}", item.disposition) == "Preserve")
        .map(|item| format!("{:?}", item.data_class))
        .collect();
    let entitlement_receipt_class = request
        .entitlement
        .as_ref()
        .map(|decision| decision.binding.receipt_class())
        .unwrap_or(LifecycleEntitlementReceiptClass::RecoveryReady);
    let entitlement_binding = request
        .entitlement
        .as_ref()
        .map(|decision| decision.binding.clone());
    let entitlement_evidence_refs = request
        .entitlement
        .as_ref()
        .map(|decision| decision.evidence_refs.clone())
        .unwrap_or_default();
    Ok(LifecycleAcceptanceReceipt {
        transaction_id: request.transaction_id.clone(),
        operation: request.operation,
        scope: request.scope.clone(),
        final_state,
        journal_head_hash: journal
            .last()
            .map(|entry| entry.entry_hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.into()),
        version_set,
        daemon_service_healthy,
        project_verified,
        bootstrap_committed,
        genesis_committed,
        first_workpoint_id,
        preserved_data_classes,
        entitlement_receipt_class,
        entitlement_binding,
        entitlement_evidence_refs,
        closure_allowed: final_state == LifecycleState::Accepted,
    })
}

fn validate_request(request: &LifecycleOperationRequest) -> Result<(), LifecycleOrchestratorError> {
    validate_request_at(request, Utc::now())
}

pub fn validate_request_at(
    request: &LifecycleOperationRequest,
    now: DateTime<Utc>,
) -> Result<(), LifecycleOrchestratorError> {
    if is_home_dev_bypass() {
        return Ok(());
    }
    if request.transaction_id.is_empty() || request.idempotency_key.is_empty() {
        return Err(LifecycleOrchestratorError::MissingAuthority);
    }
    if matches!(
        request.operation,
        LifecycleOperation::Install | LifecycleOperation::Update
    ) && !request.artifact_signature_verified
    {
        return Err(LifecycleOrchestratorError::ArtifactTrustRequired);
    }
    if request.operation == LifecycleOperation::Purge && !request.purge_confirmed_separately {
        return Err(LifecycleOrchestratorError::PurgeConfirmationRequired);
    }
    request
        .preservation
        .validate()
        .map_err(|_| LifecycleOrchestratorError::InvalidPreservation)?;

    let requires_entitlement = match request.operation {
        LifecycleOperation::Inspect | LifecycleOperation::Uninstall | LifecycleOperation::Purge => {
            false
        }
        LifecycleOperation::Rollback => false,
        LifecycleOperation::Repair | LifecycleOperation::Rerun => !request.recovery_safe,
        LifecycleOperation::Install | LifecycleOperation::Update => !request.dry_run,
    };
    if !requires_entitlement {
        return Ok(());
    }
    if request.selected_product.trim().is_empty() || request.selected_channel.trim().is_empty() {
        return Err(LifecycleOrchestratorError::ProductGrantRequired);
    }
    let decision = request
        .entitlement
        .as_ref()
        .ok_or(LifecycleOrchestratorError::EntitlementRequired)?;
    decision
        .binding
        .validate()
        .map_err(|_| LifecycleOrchestratorError::EntitlementBlocked)?;
    if !decision.binding.allows_product_execution_at(now) {
        return Err(LifecycleOrchestratorError::EntitlementBlocked);
    }
    if !decision
        .granted_products
        .contains(&request.selected_product)
    {
        return Err(LifecycleOrchestratorError::ProductGrantRequired);
    }
    let mut required_features = request.required_features.clone();
    match request.operation {
        LifecycleOperation::Install => {
            required_features.insert(format!(
                "focusa.install.channel.{}",
                request.selected_channel
            ));
        }
        LifecycleOperation::Update => {
            required_features.insert("focusa.update.apply".into());
            if request.unattended {
                required_features.insert("focusa.update.unattended".into());
            }
        }
        LifecycleOperation::Repair | LifecycleOperation::Rerun => {
            required_features.insert("focusa.repair.execute".into());
        }
        LifecycleOperation::Inspect
        | LifecycleOperation::Rollback
        | LifecycleOperation::Uninstall
        | LifecycleOperation::Purge => {}
    }
    if !required_features.is_subset(&decision.granted_features) || decision.evidence_refs.is_empty()
    {
        return Err(LifecycleOrchestratorError::FeatureGrantRequired);
    }
    Ok(())
}

fn digest(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("lifecycle journal entry is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "orchestrator_tests.rs"]
mod tests;
