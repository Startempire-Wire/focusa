//! Transcript-free reconstruction from durable Silent Session refs.

use crate::silent_session::{
    ModelBinding, OperatorAskBinding, SilentSessionId, SilentSessionRunId, WorkpointBinding,
    WorkspaceBinding,
};
use crate::silent_session_receipts::{
    ExecutionMode, ReceiptType, SilentSessionReceiptProjection, WorkSessionOutcome,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub const TRANSCRIPT_FREE_RECONSTRUCTION_SCHEMA: &str = "focusa.silent_session_reconstruction.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointHistoryEntry {
    pub workpoint_ref: WorkpointBinding,
    pub checkpoint_ref: String,
    pub meaning_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructionChanges {
    pub starting_git_status_ref: String,
    pub ending_git_status_ref: String,
    pub bounded_diff_summary: String,
    pub full_diff_artifact_ref: String,
    pub full_diff_sha256: String,
    pub files_changed: BTreeSet<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptFreeReconstructionBundle {
    pub schema: String,
    pub reconstruction_id: Uuid,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub original_ask: OperatorAskBinding,
    pub effective_config_revision_ref: String,
    pub effective_config_sha256: String,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workspace: WorkspaceBinding,
    pub requested_model: ModelBinding,
    pub effective_model: ModelBinding,
    pub observed_model: ModelBinding,
    pub workpoint_history: Vec<WorkpointHistoryEntry>,
    pub output_cursor: String,
    pub event_refs: Vec<String>,
    pub changes: ReconstructionChanges,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub session_transfer_refs: Vec<String>,
    pub final_completion_evaluation_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructedSilentSessionView {
    pub reconstruction_id: Uuid,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub original_ask_ref: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub latest_workpoint_ref: WorkpointBinding,
    pub output_cursor: String,
    pub outcome: WorkSessionOutcome,
    pub source_refs: BTreeSet<String>,
    pub transcript_required: bool,
    pub reconstruction_complete: bool,
}

pub fn reconstruct_without_transcript(
    bundle: &TranscriptFreeReconstructionBundle,
    work_session_receipt: &SilentSessionReceiptProjection,
) -> Result<ReconstructedSilentSessionView, ReconstructionError> {
    validate_bundle(bundle)?;
    work_session_receipt
        .validate()
        .map_err(|_| ReconstructionError::InvalidWorkSessionReceipt)?;
    let latest_workpoint = bundle
        .workpoint_history
        .last()
        .ok_or(ReconstructionError::MissingDurableRefs)?;
    if work_session_receipt.receipt_type != ReceiptType::WorkSession
        || work_session_receipt.execution_mode != ExecutionMode::SilentSession
        || work_session_receipt.session_id != bundle.session_id
        || work_session_receipt.run_id != bundle.run_id
        || work_session_receipt.project_identity_ref != bundle.project_identity_ref
        || work_session_receipt.continuity_id != bundle.continuity_id
        || work_session_receipt.workpoint_ref != latest_workpoint.workpoint_ref
        || work_session_receipt.claim_ref != bundle.final_completion_evaluation_ref
    {
        return Err(ReconstructionError::ScopeMismatch);
    }
    let outcome: WorkSessionOutcome = serde_json::from_value(
        work_session_receipt
            .payload
            .get("outcome")
            .cloned()
            .ok_or(ReconstructionError::InvalidWorkSessionReceipt)?,
    )
    .map_err(|_| ReconstructionError::InvalidWorkSessionReceipt)?;
    let mut source_refs = BTreeSet::new();
    source_refs.insert(bundle.effective_config_revision_ref.clone());
    source_refs.insert(bundle.changes.starting_git_status_ref.clone());
    source_refs.insert(bundle.changes.ending_git_status_ref.clone());
    source_refs.insert(bundle.changes.full_diff_artifact_ref.clone());
    source_refs.insert(bundle.final_completion_evaluation_ref.clone());
    source_refs.extend(bundle.event_refs.iter().cloned());
    source_refs.extend(bundle.evidence_refs.iter().cloned());
    source_refs.extend(bundle.receipt_refs.iter().cloned());
    source_refs.extend(bundle.session_transfer_refs.iter().cloned());
    source_refs.extend(
        bundle
            .workpoint_history
            .iter()
            .map(|entry| entry.checkpoint_ref.clone()),
    );
    Ok(ReconstructedSilentSessionView {
        reconstruction_id: bundle.reconstruction_id,
        session_id: bundle.session_id,
        run_id: bundle.run_id,
        generation: bundle.generation,
        original_ask_ref: bundle.original_ask.ask_ref.clone(),
        project_identity_ref: bundle.project_identity_ref.clone(),
        continuity_id: bundle.continuity_id.clone(),
        latest_workpoint_ref: latest_workpoint.workpoint_ref.clone(),
        output_cursor: bundle.output_cursor.clone(),
        outcome,
        source_refs,
        transcript_required: false,
        reconstruction_complete: true,
    })
}

fn validate_bundle(bundle: &TranscriptFreeReconstructionBundle) -> Result<(), ReconstructionError> {
    bundle
        .original_ask
        .validate()
        .map_err(|_| ReconstructionError::InvalidAsk)?;
    if bundle.schema != TRANSCRIPT_FREE_RECONSTRUCTION_SCHEMA
        || bundle.reconstruction_id.get_version() != Some(uuid::Version::SortRand)
        || !bundle.session_id.is_uuid_v7()
        || !bundle.run_id.is_uuid_v7()
        || bundle.generation == 0
        || bundle.effective_config_revision_ref.trim().is_empty()
        || !valid_sha256(&bundle.effective_config_sha256)
        || !bundle.project_root.is_absolute()
        || bundle.project_identity_ref.trim().is_empty()
        || bundle.continuity_id.trim().is_empty()
        || !bundle.workspace.root.is_absolute()
        || bundle.workspace.workspace_id.trim().is_empty()
        || model_missing(&bundle.requested_model)
        || model_missing(&bundle.effective_model)
        || model_missing(&bundle.observed_model)
        || bundle.output_cursor.trim().is_empty()
        || bundle.final_completion_evaluation_ref.trim().is_empty()
        || bundle.changes.starting_git_status_ref.trim().is_empty()
        || bundle.changes.ending_git_status_ref.trim().is_empty()
        || bundle.changes.bounded_diff_summary.trim().is_empty()
        || bundle.changes.full_diff_artifact_ref.trim().is_empty()
        || !valid_sha256(&bundle.changes.full_diff_sha256)
        || bundle.changes.files_changed.is_empty()
    {
        return Err(ReconstructionError::InvalidBundle);
    }
    if bundle.workpoint_history.is_empty()
        || bundle.event_refs.is_empty()
        || bundle.evidence_refs.is_empty()
        || bundle.receipt_refs.is_empty()
    {
        return Err(ReconstructionError::MissingDurableRefs);
    }
    for entry in &bundle.workpoint_history {
        if entry.workpoint_ref.workpoint_id.trim().is_empty()
            || entry.checkpoint_ref.trim().is_empty()
            || !valid_sha256(&entry.meaning_sha256)
        {
            return Err(ReconstructionError::MissingDurableRefs);
        }
    }
    for values in [
        &bundle.event_refs,
        &bundle.evidence_refs,
        &bundle.receipt_refs,
        &bundle.session_transfer_refs,
    ] {
        if values.iter().any(|value| value.trim().is_empty()) {
            return Err(ReconstructionError::MissingDurableRefs);
        }
    }
    Ok(())
}

fn model_missing(model: &ModelBinding) -> bool {
    model.provider.trim().is_empty() || model.model.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReconstructionError {
    #[error("original Ask binding is invalid")]
    InvalidAsk,
    #[error("reconstruction bundle is invalid")]
    InvalidBundle,
    #[error("durable reconstruction refs are missing")]
    MissingDurableRefs,
    #[error("work-session receipt is invalid")]
    InvalidWorkSessionReceipt,
    #[error("work-session receipt and reconstruction scope differ")]
    ScopeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::{WorkpointBinding, WorkspaceStrategy};
    use crate::silent_session_receipts::project_receipt;
    use chrono::Utc;
    use serde_json::json;

    fn workpoint() -> WorkpointBinding {
        WorkpointBinding {
            workpoint_id: "workpoint:final".into(),
            revision: Some(8),
        }
    }

    fn bundle() -> TranscriptFreeReconstructionBundle {
        let now = Utc::now();
        TranscriptFreeReconstructionBundle {
            schema: TRANSCRIPT_FREE_RECONSTRUCTION_SCHEMA.into(),
            reconstruction_id: Uuid::now_v7(),
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 3,
            original_ask: OperatorAskBinding::capture(
                "ask:original",
                "finish the governed work item",
                1,
                now,
            ),
            effective_config_revision_ref: "config-revision:3".into(),
            effective_config_sha256: "a".repeat(64),
            project_root: crate::test_support::absolute_path("silent-reconstruction-project"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "continuity:test".into(),
            workspace: WorkspaceBinding {
                workspace_id: "workspace:test".into(),
                root: crate::test_support::absolute_path("silent-reconstruction-worktree"),
                strategy: WorkspaceStrategy::IsolatedWorktree,
                branch_ref: Some("branch:test".into()),
            },
            requested_model: ModelBinding {
                provider: "provider".into(),
                model: "requested".into(),
                thinking: None,
            },
            effective_model: ModelBinding {
                provider: "provider".into(),
                model: "effective".into(),
                thinking: None,
            },
            observed_model: ModelBinding {
                provider: "provider".into(),
                model: "effective".into(),
                thinking: None,
            },
            workpoint_history: vec![WorkpointHistoryEntry {
                workpoint_ref: workpoint(),
                checkpoint_ref: "workpoint-checkpoint:final".into(),
                meaning_sha256: "b".repeat(64),
            }],
            output_cursor: "cursor:final".into(),
            event_refs: vec!["event:1".into(), "event:2".into()],
            changes: ReconstructionChanges {
                starting_git_status_ref: "git-status:start".into(),
                ending_git_status_ref: "git-status:end".into(),
                bounded_diff_summary: "one file changed".into(),
                full_diff_artifact_ref: "artifact:diff".into(),
                full_diff_sha256: "c".repeat(64),
                files_changed: BTreeSet::from([PathBuf::from("src/lib.rs")]),
            },
            evidence_refs: vec!["evidence:test".into()],
            receipt_refs: vec!["receipt:work-session".into()],
            session_transfer_refs: vec!["transfer:handoff".into()],
            final_completion_evaluation_ref: "completion-evaluation:final".into(),
        }
    }

    fn receipt(bundle: &TranscriptFreeReconstructionBundle) -> SilentSessionReceiptProjection {
        project_receipt(
            ReceiptType::WorkSession,
            bundle.session_id,
            bundle.run_id,
            bundle.project_identity_ref.clone(),
            bundle.continuity_id.clone(),
            workpoint(),
            Some("focusa-a6yq6.7.7".into()),
            bundle.final_completion_evaluation_ref.clone(),
            vec!["evidence:test".into()],
            "cursor:final",
            json!({
                "outcome": "completed",
                "process_exit_is_completion": false,
            }),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn reconstructs_original_ask_config_scope_model_workpoint_events_changes_and_outcome() {
        let bundle = bundle();
        let view = reconstruct_without_transcript(&bundle, &receipt(&bundle)).unwrap();
        assert!(!view.transcript_required);
        assert!(view.reconstruction_complete);
        assert_eq!(view.original_ask_ref, "ask:original");
        assert_eq!(view.latest_workpoint_ref, workpoint());
        assert_eq!(view.outcome, WorkSessionOutcome::Completed);
        assert!(view.source_refs.contains("artifact:diff"));
        assert!(view.source_refs.contains("event:2"));
        assert!(view.source_refs.contains("transfer:handoff"));
    }

    #[test]
    fn missing_durable_refs_or_mismatched_receipt_fail_closed() {
        let mut missing = bundle();
        missing.event_refs.clear();
        assert_eq!(
            reconstruct_without_transcript(&missing, &receipt(&missing)),
            Err(ReconstructionError::MissingDurableRefs)
        );

        let bundle = bundle();
        let mut wrong_receipt = receipt(&bundle);
        wrong_receipt.continuity_id = "continuity:other".into();
        assert_eq!(
            reconstruct_without_transcript(&bundle, &wrong_receipt),
            Err(ReconstructionError::ScopeMismatch)
        );
    }
}
