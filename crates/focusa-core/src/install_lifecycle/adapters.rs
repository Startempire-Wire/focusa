use super::LifecycleScope;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleAdapterKind {
    Pi,
    Uiai,
    ProviderAuth,
    MacMenubar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterCapabilityState {
    PresentCompatible,
    Absent,
    Incompatible,
    Busy,
    Compacting,
    Healthy,
    Saturated,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterSelection {
    Required,
    OptionalEnabled,
    OptedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSelectionRecord {
    pub adapter: LifecycleAdapterKind,
    pub selection: AdapterSelection,
    pub capability: AdapterCapabilityState,
    pub operator_confirmed: bool,
    pub capability_evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAuthHandoff {
    pub provider_id: String,
    pub handoff_url: String,
    pub state_ref: String,
    pub credential_payload: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleAdapterRequest {
    pub transaction_id: String,
    pub transaction_receipt_id: String,
    pub scope: LifecycleScope,
    pub selections: Vec<AdapterSelectionRecord>,
    pub provider_handoff: Option<ProviderAuthHandoff>,
    pub prior_attempt: u32,
    pub evidence_messages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterOutcomeState {
    Active,
    Degraded,
    Blocked,
    OptedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterOutcome {
    pub adapter: LifecycleAdapterKind,
    pub state: AdapterOutcomeState,
    pub reason_code: String,
    pub retryable: bool,
    pub resume_action: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleAdapterReceipt {
    pub transaction_id: String,
    pub transaction_receipt_id: String,
    pub scope: LifecycleScope,
    pub attempt: u32,
    pub outcomes: BTreeMap<LifecycleAdapterKind, AdapterOutcome>,
    pub redacted_evidence: Vec<String>,
    pub all_required_active: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LifecycleAdapterError {
    #[error("adapter selection requires explicit operator confirmation")]
    SelectionNotConfirmed,
    #[error("duplicate adapter selection")]
    DuplicateSelection,
    #[error("provider-neutral handoff may not ingest credentials")]
    CredentialIngestionForbidden,
    #[error("provider handoff must use an absolute HTTPS URL and state reference")]
    InvalidProviderHandoff,
    #[error("transaction and receipt identifiers are required")]
    MissingTransactionBinding,
}

pub fn evaluate_lifecycle_adapters(
    request: &LifecycleAdapterRequest,
) -> Result<LifecycleAdapterReceipt, LifecycleAdapterError> {
    if request.transaction_id.is_empty() || request.transaction_receipt_id.is_empty() {
        return Err(LifecycleAdapterError::MissingTransactionBinding);
    }
    if let Some(handoff) = &request.provider_handoff {
        if handoff.credential_payload.is_some() {
            return Err(LifecycleAdapterError::CredentialIngestionForbidden);
        }
        if !handoff.handoff_url.starts_with("https://") || handoff.state_ref.is_empty() {
            return Err(LifecycleAdapterError::InvalidProviderHandoff);
        }
    }
    let mut outcomes = BTreeMap::new();
    for record in &request.selections {
        if !record.operator_confirmed {
            return Err(LifecycleAdapterError::SelectionNotConfirmed);
        }
        if outcomes.contains_key(&record.adapter) {
            return Err(LifecycleAdapterError::DuplicateSelection);
        }
        let outcome = evaluate_one(record);
        outcomes.insert(record.adapter, outcome);
    }
    let all_required_active = request.selections.iter().all(|record| {
        record.selection != AdapterSelection::Required
            || outcomes
                .get(&record.adapter)
                .is_some_and(|outcome| outcome.state == AdapterOutcomeState::Active)
    });
    Ok(LifecycleAdapterReceipt {
        transaction_id: request.transaction_id.clone(),
        transaction_receipt_id: request.transaction_receipt_id.clone(),
        scope: request.scope.clone(),
        attempt: request.prior_attempt.saturating_add(1),
        outcomes,
        redacted_evidence: request
            .evidence_messages
            .iter()
            .map(|message| redact_evidence(message))
            .collect(),
        all_required_active,
    })
}

fn evaluate_one(record: &AdapterSelectionRecord) -> AdapterOutcome {
    if record.selection == AdapterSelection::OptedOut {
        return outcome(
            record,
            AdapterOutcomeState::OptedOut,
            "operator_opted_out",
            false,
            None,
        );
    }
    use AdapterCapabilityState::*;
    let (state, reason, retryable, resume) = match record.capability {
        PresentCompatible | Healthy => {
            (AdapterOutcomeState::Active, "capability_ready", false, None)
        }
        Busy => (
            AdapterOutcomeState::Degraded,
            "pi_busy",
            true,
            Some("retry_when_idle"),
        ),
        Compacting => (
            AdapterOutcomeState::Degraded,
            "pi_compacting",
            true,
            Some("resume_after_compaction"),
        ),
        Saturated => (
            AdapterOutcomeState::Degraded,
            "uiai_saturated",
            true,
            Some("retry_bounded_session"),
        ),
        Absent | Incompatible | Unsupported if record.selection == AdapterSelection::Required => (
            AdapterOutcomeState::Blocked,
            "required_capability_unavailable",
            false,
            Some("operator_select_recovery"),
        ),
        Absent => (
            AdapterOutcomeState::Degraded,
            "optional_capability_absent",
            false,
            None,
        ),
        Incompatible => (
            AdapterOutcomeState::Degraded,
            "optional_capability_incompatible",
            false,
            None,
        ),
        Unsupported => (
            AdapterOutcomeState::Degraded,
            "platform_unsupported",
            false,
            None,
        ),
    };
    outcome(record, state, reason, retryable, resume)
}

fn outcome(
    record: &AdapterSelectionRecord,
    state: AdapterOutcomeState,
    reason: &str,
    retryable: bool,
    resume: Option<&str>,
) -> AdapterOutcome {
    AdapterOutcome {
        adapter: record.adapter,
        state,
        reason_code: reason.into(),
        retryable,
        resume_action: resume.map(str::to_owned),
        evidence_refs: vec![record.capability_evidence_ref.clone()],
    }
}

fn redact_evidence(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if [
        "token=",
        "api_key=",
        "password=",
        "secret=",
        "authorization:",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        "[REDACTED]".into()
    } else {
        message.into()
    }
}

#[cfg(test)]
#[path = "adapters_tests.rs"]
mod tests;
