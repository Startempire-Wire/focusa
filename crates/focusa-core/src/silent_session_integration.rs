//! Evidence-gated governed integration protocol for Silent Sessions.

use crate::silent_session::{SilentSessionId, SilentSessionRunId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub const INTEGRATION_PREFLIGHT_SCHEMA: &str = "focusa.integration_preflight.v1";
pub const INTEGRATION_RECEIPT_SCHEMA: &str = "focusa.integration_receipt.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernedIntegrationMethod {
    VerifiedFastForward,
    GovernedMerge,
    GovernedRebase,
    GovernedCherryPick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationMissingGate {
    Tests,
    Verification,
    FinalWorkpoint,
    DiffEvidence,
    CommitEvidence,
    Preview,
    ContextAuthority,
    WriterLease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationPreflightRequest {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub source_workspace_ref: String,
    pub target_workspace_ref: String,
    pub source_head: String,
    pub target_head: String,
    pub method: GovernedIntegrationMethod,
    pub tests_passed: bool,
    pub verification_evidence_refs: Vec<String>,
    pub final_workpoint_checkpoint_ref: Option<String>,
    pub diff_evidence_ref: Option<String>,
    pub commit_refs: Vec<String>,
    pub integration_preview_ref: Option<String>,
    pub context_authority_ref: Option<String>,
    pub writer_lease_ref: Option<String>,
    pub writer_fencing_token: Option<u64>,
    pub unrelated_dirty_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationPreflight {
    pub schema: String,
    pub authorized: bool,
    pub missing_gates: Vec<IntegrationMissingGate>,
    pub action_digest: Option<String>,
    pub request: IntegrationPreflightRequest,
}

impl IntegrationPreflightRequest {
    pub fn evaluate(self) -> Result<IntegrationPreflight, IntegrationProtocolError> {
        if !self.session_id.is_uuid_v7()
            || !self.run_id.is_uuid_v7()
            || self.generation == 0
            || self.source_workspace_ref.trim().is_empty()
            || self.target_workspace_ref.trim().is_empty()
            || self.source_workspace_ref == self.target_workspace_ref
            || !valid_revision(&self.source_head)
            || !valid_revision(&self.target_head)
            || !safe_paths(&self.unrelated_dirty_paths)
        {
            return Err(IntegrationProtocolError::InvalidScope);
        }
        let mut missing = Vec::new();
        if !self.tests_passed {
            missing.push(IntegrationMissingGate::Tests);
        }
        if self.verification_evidence_refs.is_empty()
            || self
                .verification_evidence_refs
                .iter()
                .any(|value| value.trim().is_empty())
        {
            missing.push(IntegrationMissingGate::Verification);
        }
        if empty(&self.final_workpoint_checkpoint_ref) {
            missing.push(IntegrationMissingGate::FinalWorkpoint);
        }
        if empty(&self.diff_evidence_ref) {
            missing.push(IntegrationMissingGate::DiffEvidence);
        }
        if self.commit_refs.is_empty()
            || self.commit_refs.iter().any(|value| !valid_revision(value))
        {
            missing.push(IntegrationMissingGate::CommitEvidence);
        }
        if empty(&self.integration_preview_ref) {
            missing.push(IntegrationMissingGate::Preview);
        }
        if empty(&self.context_authority_ref) {
            missing.push(IntegrationMissingGate::ContextAuthority);
        }
        if empty(&self.writer_lease_ref) || self.writer_fencing_token.unwrap_or(0) == 0 {
            missing.push(IntegrationMissingGate::WriterLease);
        }
        missing.sort();
        missing.dedup();
        let action_digest = if missing.is_empty() {
            Some(action_digest(&self)?)
        } else {
            None
        };
        Ok(IntegrationPreflight {
            schema: INTEGRATION_PREFLIGHT_SCHEMA.into(),
            authorized: missing.is_empty(),
            missing_gates: missing,
            action_digest,
            request: self,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationExecutionObservation {
    pub action_digest: String,
    pub resulting_head: Option<String>,
    pub conflict_paths: Vec<PathBuf>,
    pub preserved_unrelated_dirty_paths: Vec<PathBuf>,
    pub destructive_cleanup_performed: bool,
    pub executed_command_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationOutcomeStatus {
    Integrated,
    BlockedConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationReceipt {
    pub schema: String,
    pub receipt_id: Uuid,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub method: GovernedIntegrationMethod,
    pub source_head: String,
    pub target_head: String,
    pub resulting_head: String,
    pub final_workpoint_checkpoint_ref: String,
    pub diff_evidence_ref: String,
    pub commit_refs: Vec<String>,
    pub integration_preview_ref: String,
    pub context_authority_ref: String,
    pub writer_lease_ref: String,
    pub writer_fencing_token: u64,
    pub executed_command_ref: String,
    pub preserved_unrelated_dirty_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationOutcome {
    pub status: IntegrationOutcomeStatus,
    pub lifecycle_blocked: bool,
    pub conflict_paths: Vec<PathBuf>,
    pub preserved_unrelated_dirty_paths: Vec<PathBuf>,
    pub receipt: Option<IntegrationReceipt>,
}

pub fn evaluate_integration_execution(
    preflight: &IntegrationPreflight,
    observation: IntegrationExecutionObservation,
) -> Result<IntegrationOutcome, IntegrationProtocolError> {
    if !preflight.authorized
        || preflight.action_digest.as_deref() != Some(observation.action_digest.as_str())
        || observation.executed_command_ref.trim().is_empty()
        || !safe_paths(&observation.conflict_paths)
        || !safe_paths(&observation.preserved_unrelated_dirty_paths)
    {
        return Err(IntegrationProtocolError::ExecutionNotAuthorized);
    }
    if observation.destructive_cleanup_performed {
        return Err(IntegrationProtocolError::DestructiveCleanupForbidden);
    }
    let expected_dirty = preflight
        .request
        .unrelated_dirty_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let preserved_dirty = observation
        .preserved_unrelated_dirty_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !expected_dirty.is_subset(&preserved_dirty) {
        return Err(IntegrationProtocolError::UnrelatedDirtyStateLost);
    }
    if !observation.conflict_paths.is_empty() {
        return Ok(IntegrationOutcome {
            status: IntegrationOutcomeStatus::BlockedConflict,
            lifecycle_blocked: true,
            conflict_paths: observation.conflict_paths,
            preserved_unrelated_dirty_paths: observation.preserved_unrelated_dirty_paths,
            receipt: None,
        });
    }
    let resulting_head = observation
        .resulting_head
        .filter(|value| valid_revision(value))
        .ok_or(IntegrationProtocolError::ResultingRevisionMissing)?;
    let request = &preflight.request;
    let receipt = IntegrationReceipt {
        schema: INTEGRATION_RECEIPT_SCHEMA.into(),
        receipt_id: Uuid::now_v7(),
        session_id: request.session_id,
        run_id: request.run_id,
        generation: request.generation,
        method: request.method,
        source_head: request.source_head.clone(),
        target_head: request.target_head.clone(),
        resulting_head,
        final_workpoint_checkpoint_ref: request
            .final_workpoint_checkpoint_ref
            .clone()
            .unwrap_or_default(),
        diff_evidence_ref: request.diff_evidence_ref.clone().unwrap_or_default(),
        commit_refs: request.commit_refs.clone(),
        integration_preview_ref: request.integration_preview_ref.clone().unwrap_or_default(),
        context_authority_ref: request.context_authority_ref.clone().unwrap_or_default(),
        writer_lease_ref: request.writer_lease_ref.clone().unwrap_or_default(),
        writer_fencing_token: request.writer_fencing_token.unwrap_or_default(),
        executed_command_ref: observation.executed_command_ref,
        preserved_unrelated_dirty_paths: observation.preserved_unrelated_dirty_paths.clone(),
    };
    Ok(IntegrationOutcome {
        status: IntegrationOutcomeStatus::Integrated,
        lifecycle_blocked: false,
        conflict_paths: vec![],
        preserved_unrelated_dirty_paths: observation.preserved_unrelated_dirty_paths,
        receipt: Some(receipt),
    })
}

fn action_digest(
    request: &IntegrationPreflightRequest,
) -> Result<String, IntegrationProtocolError> {
    let bytes = serde_json::to_vec(request).map_err(|_| IntegrationProtocolError::Serialization)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn empty(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|value| value.trim().is_empty())
}

fn valid_revision(value: &str) -> bool {
    let trimmed = value.trim();
    (7..=64).contains(&trimmed.len()) && trimmed.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_paths(paths: &[PathBuf]) -> bool {
    paths.iter().all(|path| !path.as_os_str().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IntegrationProtocolError {
    #[error("integration scope or revision is invalid")]
    InvalidScope,
    #[error("integration preflight serialization failed")]
    Serialization,
    #[error("integration execution does not match an authorized preflight")]
    ExecutionNotAuthorized,
    #[error("integration protocol forbids destructive cleanup")]
    DestructiveCleanupForbidden,
    #[error("integration execution lost unrelated dirty state")]
    UnrelatedDirtyStateLost,
    #[error("successful integration requires a resulting revision")]
    ResultingRevisionMissing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> IntegrationPreflightRequest {
        IntegrationPreflightRequest {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 2,
            source_workspace_ref: "workspace:isolated".into(),
            target_workspace_ref: "workspace:primary".into(),
            source_head: "aaaaaaaa".into(),
            target_head: "bbbbbbbb".into(),
            method: GovernedIntegrationMethod::GovernedCherryPick,
            tests_passed: true,
            verification_evidence_refs: vec!["test:all".into()],
            final_workpoint_checkpoint_ref: Some("workpoint:final".into()),
            diff_evidence_ref: Some("diff:preview".into()),
            commit_refs: vec!["cccccccc".into()],
            integration_preview_ref: Some("integration:preview".into()),
            context_authority_ref: Some("context:verified".into()),
            writer_lease_ref: Some("lease:target".into()),
            writer_fencing_token: Some(17),
            unrelated_dirty_paths: vec![PathBuf::from("docs/unrelated.md")],
        }
    }

    #[test]
    fn every_required_preflight_gate_blocks_execution_until_present() {
        let mut incomplete = request();
        incomplete.tests_passed = false;
        incomplete.verification_evidence_refs.clear();
        incomplete.final_workpoint_checkpoint_ref = None;
        incomplete.diff_evidence_ref = None;
        incomplete.commit_refs.clear();
        incomplete.integration_preview_ref = None;
        incomplete.context_authority_ref = None;
        incomplete.writer_lease_ref = None;
        incomplete.writer_fencing_token = None;
        let preflight = incomplete.evaluate().unwrap();
        assert!(!preflight.authorized);
        assert_eq!(preflight.missing_gates.len(), 8);
        assert!(preflight.action_digest.is_none());
    }

    #[test]
    fn conflict_blocks_without_cleanup_and_preserves_unrelated_dirty_state() {
        let preflight = request().evaluate().unwrap();
        let outcome = evaluate_integration_execution(
            &preflight,
            IntegrationExecutionObservation {
                action_digest: preflight.action_digest.clone().unwrap(),
                resulting_head: None,
                conflict_paths: vec![PathBuf::from("src/conflict.rs")],
                preserved_unrelated_dirty_paths: vec![PathBuf::from("docs/unrelated.md")],
                destructive_cleanup_performed: false,
                executed_command_ref: "git:cherry-pick:no-commit".into(),
            },
        )
        .unwrap();
        assert_eq!(outcome.status, IntegrationOutcomeStatus::BlockedConflict);
        assert!(outcome.lifecycle_blocked);
        assert!(outcome.receipt.is_none());

        assert_eq!(
            evaluate_integration_execution(
                &preflight,
                IntegrationExecutionObservation {
                    action_digest: preflight.action_digest.clone().unwrap(),
                    resulting_head: None,
                    conflict_paths: vec![PathBuf::from("src/conflict.rs")],
                    preserved_unrelated_dirty_paths: vec![],
                    destructive_cleanup_performed: false,
                    executed_command_ref: "git:cherry-pick:no-commit".into(),
                },
            ),
            Err(IntegrationProtocolError::UnrelatedDirtyStateLost)
        );
    }

    #[test]
    fn successful_integration_emits_complete_receipt_and_rejects_cleanup() {
        let preflight = request().evaluate().unwrap();
        let observation = IntegrationExecutionObservation {
            action_digest: preflight.action_digest.clone().unwrap(),
            resulting_head: Some("dddddddd".into()),
            conflict_paths: vec![],
            preserved_unrelated_dirty_paths: vec![PathBuf::from("docs/unrelated.md")],
            destructive_cleanup_performed: false,
            executed_command_ref: "git:cherry-pick:no-commit".into(),
        };
        let outcome = evaluate_integration_execution(&preflight, observation.clone()).unwrap();
        let receipt = outcome.receipt.unwrap();
        assert_eq!(outcome.status, IntegrationOutcomeStatus::Integrated);
        assert_eq!(receipt.schema, INTEGRATION_RECEIPT_SCHEMA);
        assert_eq!(receipt.resulting_head, "dddddddd");
        assert_eq!(receipt.writer_fencing_token, 17);
        assert_eq!(receipt.preserved_unrelated_dirty_paths.len(), 1);

        let mut destructive = observation;
        destructive.destructive_cleanup_performed = true;
        assert_eq!(
            evaluate_integration_execution(&preflight, destructive),
            Err(IntegrationProtocolError::DestructiveCleanupForbidden)
        );
    }
}
