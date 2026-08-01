use super::{LifecycleScope, LifecycleState, PreservationDeclaration};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

const GENESIS_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleOperation {
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
        closure_allowed: final_state == LifecycleState::Accepted,
    })
}

fn validate_request(request: &LifecycleOperationRequest) -> Result<(), LifecycleOrchestratorError> {
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
    Ok(())
}

fn digest(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("lifecycle journal entry is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "orchestrator_tests.rs"]
mod tests;
