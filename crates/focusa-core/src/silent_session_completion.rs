//! Fail-closed completion evidence and decision protocol for Silent Sessions.

use crate::silent_session::{
    CompletionDecision, SILENT_SESSION_COMPLETION_SCHEMA, SilentSession,
    SilentSessionCompletionEvaluation, SilentSessionCompletionEvaluationId,
    SilentSessionLifecycleState, SilentSessionRun,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatusEvidence {
    pub head_ref: String,
    pub status_artifact_ref: String,
    pub status_sha256: String,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffEvidence {
    pub bounded_summary: String,
    pub full_diff_artifact_ref: String,
    pub full_diff_sha256: String,
    pub files_changed: BTreeSet<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRunEvidence {
    pub verification_class: String,
    pub command_ref: String,
    pub output_artifact_ref: String,
    pub output_sha256: String,
    pub exit_code: i32,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelUsageEvidence {
    pub requested_model_ref: String,
    pub effective_model_ref: String,
    pub observed_model_ref: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_microunits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionEvidenceBundle {
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub workspace_root: PathBuf,
    pub workspace_ref: String,
    pub starting_git_status: GitStatusEvidence,
    pub ending_git_status: GitStatusEvidence,
    pub diff: DiffEvidence,
    pub verification_runs: Vec<VerificationRunEvidence>,
    pub commit_refs: Vec<String>,
    pub final_workpoint_checkpoint_ref: String,
    pub unresolved_blockers: Vec<String>,
    pub context_authority_refs: Vec<String>,
    pub model_usage: ModelUsageEvidence,
    pub resource_usage: BTreeMap<String, u64>,
    pub stream_manifest_ref: String,
    pub stream_manifest_sha256: String,
    pub completion_verifier_ref: String,
    pub completion_verifier_passed: bool,
    pub receipt_preview_ref: String,
    pub receipt_preview_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionPolicy {
    pub code_changing: bool,
    pub commit_required: bool,
    pub adversarial_verifier_required: bool,
    pub required_verification_classes: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceEvaluation {
    pub criteria: BTreeMap<String, bool>,
    pub adversarial_verifier_verdict: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingCompletionEvidence {
    ExactProject,
    ExactWorkspace,
    StartingGitStatus,
    EndingGitStatus,
    BoundedDiffSummary,
    FullDiffArtifact,
    FilesChanged,
    RequiredVerification(String),
    CommitRef,
    FinalWorkpointCheckpoint,
    ContextAuthority,
    ModelUsage,
    ResourceUsage,
    StreamManifest,
    CompletionVerifier,
    ReceiptPreview,
    ReceiptCommit,
    RefreshedProjectIdentity,
    RefreshedWorkpoint,
    AcceptanceCriteria,
    AdversarialVerifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionEvaluationOutcome {
    pub evaluation: SilentSessionCompletionEvaluation,
    pub next_lifecycle_state: SilentSessionLifecycleState,
    pub reason: Option<String>,
    pub missing_evidence: BTreeSet<MissingCompletionEvidence>,
    pub receipt_commit_ref: Option<String>,
}

pub struct CompletionEvaluationRequest<'a> {
    pub session: &'a SilentSession,
    pub run: &'a SilentSessionRun,
    pub bundle: &'a CompletionEvidenceBundle,
    pub policy: &'a CompletionPolicy,
    pub acceptance: &'a AcceptanceEvaluation,
    pub project_identity_refreshed: bool,
    pub workpoint_refreshed: bool,
    pub receipt_commit_ref: Option<&'a str>,
    pub evaluated_at: DateTime<Utc>,
}

pub fn evaluate_completion(
    request: &CompletionEvaluationRequest<'_>,
) -> Result<CompletionEvaluationOutcome, CompletionProtocolError> {
    request
        .session
        .validate()
        .map_err(|_| CompletionProtocolError::InvalidSession)?;
    request
        .run
        .validate(request.session)
        .map_err(|_| CompletionProtocolError::InvalidRun)?;
    if request.session.lifecycle_state != SilentSessionLifecycleState::Completing
        || request.run.ended_at.is_none()
        || request.run.exit_status.is_none()
    {
        return Err(CompletionProtocolError::ProcessNotCompleting);
    }

    let mut missing = missing_evidence(request);
    let verification_failed = request.run.exit_status != Some(0)
        || !request.bundle.completion_verifier_passed
        || request
            .bundle
            .verification_runs
            .iter()
            .any(|run| run.required && run.exit_code != 0);
    let acceptance_failed = request
        .acceptance
        .criteria
        .values()
        .any(|accepted| !accepted);
    let adversarial_failed = request.policy.adversarial_verifier_required
        && request.acceptance.adversarial_verifier_verdict.as_deref() != Some("passed");

    let (decision, next_state, reason, receipt_ready) = if !missing.is_empty() {
        (
            CompletionDecision::Blocked,
            SilentSessionLifecycleState::Blocked,
            Some("completion_evidence_missing".to_owned()),
            false,
        )
    } else if !request.bundle.unresolved_blockers.is_empty() {
        (
            CompletionDecision::Blocked,
            SilentSessionLifecycleState::Blocked,
            Some("unresolved_blockers".to_owned()),
            false,
        )
    } else if verification_failed || acceptance_failed || adversarial_failed {
        (
            CompletionDecision::Failed,
            SilentSessionLifecycleState::Failed,
            Some("verification_failed".to_owned()),
            false,
        )
    } else if request.receipt_commit_ref.is_none() {
        missing.insert(MissingCompletionEvidence::ReceiptCommit);
        (
            CompletionDecision::Incomplete,
            SilentSessionLifecycleState::Completing,
            Some("receipt_not_committed".to_owned()),
            true,
        )
    } else {
        (
            CompletionDecision::Completed,
            SilentSessionLifecycleState::Completed,
            None,
            true,
        )
    };

    let evidence_classes = completion_evidence_classes(request.bundle);
    let test_results = request
        .bundle
        .verification_runs
        .iter()
        .map(|run| serde_json::to_value(run).expect("verification evidence serializes"))
        .collect();
    let evaluation = SilentSessionCompletionEvaluation {
        schema: SILENT_SESSION_COMPLETION_SCHEMA.into(),
        evaluation_id: SilentSessionCompletionEvaluationId::new(),
        session_id: request.session.session_id,
        run_id: request.run.run_id,
        evaluated_at: request.evaluated_at,
        process_result: json!({
            "exit_status": request.run.exit_status,
            "ended_at": request.run.ended_at,
        }),
        workpoint_status: if request.workpoint_refreshed {
            "refreshed"
        } else {
            "stale"
        }
        .into(),
        work_item_acceptance: request.acceptance.criteria.clone(),
        evidence_classes,
        test_results,
        diff_refs: vec![request.bundle.diff.full_diff_artifact_ref.clone()],
        commit_refs: request.bundle.commit_refs.clone(),
        unresolved_blockers: request.bundle.unresolved_blockers.clone(),
        adversarial_verifier_verdict: request.acceptance.adversarial_verifier_verdict.clone(),
        receipt_ready,
        decision,
    };
    Ok(CompletionEvaluationOutcome {
        evaluation,
        next_lifecycle_state: next_state,
        reason,
        missing_evidence: missing,
        receipt_commit_ref: request.receipt_commit_ref.map(ToOwned::to_owned),
    })
}

fn missing_evidence(
    request: &CompletionEvaluationRequest<'_>,
) -> BTreeSet<MissingCompletionEvidence> {
    let bundle = request.bundle;
    let mut missing = BTreeSet::new();
    if !bundle.project_root.is_absolute()
        || bundle.project_identity_ref.trim().is_empty()
        || bundle.project_root != request.session.project_root
        || bundle.project_identity_ref != request.session.project_identity_ref
    {
        missing.insert(MissingCompletionEvidence::ExactProject);
    }
    if !bundle.workspace_root.is_absolute()
        || bundle.workspace_ref.trim().is_empty()
        || bundle.workspace_root != request.run.workspace_binding.root
    {
        missing.insert(MissingCompletionEvidence::ExactWorkspace);
    }
    validate_git_status(
        &bundle.starting_git_status,
        MissingCompletionEvidence::StartingGitStatus,
        &mut missing,
    );
    validate_git_status(
        &bundle.ending_git_status,
        MissingCompletionEvidence::EndingGitStatus,
        &mut missing,
    );
    if bundle.diff.bounded_summary.trim().is_empty()
        || bundle.diff.bounded_summary.len() > 16 * 1024
    {
        missing.insert(MissingCompletionEvidence::BoundedDiffSummary);
    }
    if bundle.diff.full_diff_artifact_ref.trim().is_empty()
        || !valid_sha256(&bundle.diff.full_diff_sha256)
    {
        missing.insert(MissingCompletionEvidence::FullDiffArtifact);
    }
    if request.policy.code_changing && bundle.diff.files_changed.is_empty() {
        missing.insert(MissingCompletionEvidence::FilesChanged);
    }
    for class in &request.policy.required_verification_classes {
        if !bundle.verification_runs.iter().any(|run| {
            run.required
                && &run.verification_class == class
                && !run.command_ref.trim().is_empty()
                && !run.output_artifact_ref.trim().is_empty()
                && valid_sha256(&run.output_sha256)
        }) {
            missing.insert(MissingCompletionEvidence::RequiredVerification(
                class.clone(),
            ));
        }
    }
    if request.policy.commit_required
        && (bundle.commit_refs.is_empty()
            || bundle
                .commit_refs
                .iter()
                .any(|value| value.trim().is_empty()))
    {
        missing.insert(MissingCompletionEvidence::CommitRef);
    }
    if bundle.final_workpoint_checkpoint_ref.trim().is_empty() {
        missing.insert(MissingCompletionEvidence::FinalWorkpointCheckpoint);
    }
    if request.policy.code_changing
        && (bundle.context_authority_refs.is_empty()
            || bundle
                .context_authority_refs
                .iter()
                .any(|value| value.trim().is_empty()))
    {
        missing.insert(MissingCompletionEvidence::ContextAuthority);
    }
    if bundle.model_usage.requested_model_ref.trim().is_empty()
        || bundle.model_usage.effective_model_ref.trim().is_empty()
        || bundle.model_usage.observed_model_ref.trim().is_empty()
    {
        missing.insert(MissingCompletionEvidence::ModelUsage);
    }
    if bundle.resource_usage.is_empty() {
        missing.insert(MissingCompletionEvidence::ResourceUsage);
    }
    if bundle.stream_manifest_ref.trim().is_empty() || !valid_sha256(&bundle.stream_manifest_sha256)
    {
        missing.insert(MissingCompletionEvidence::StreamManifest);
    }
    if bundle.completion_verifier_ref.trim().is_empty() {
        missing.insert(MissingCompletionEvidence::CompletionVerifier);
    }
    if bundle.receipt_preview_ref.trim().is_empty() || !valid_sha256(&bundle.receipt_preview_sha256)
    {
        missing.insert(MissingCompletionEvidence::ReceiptPreview);
    }
    if !request.project_identity_refreshed {
        missing.insert(MissingCompletionEvidence::RefreshedProjectIdentity);
    }
    if !request.workpoint_refreshed {
        missing.insert(MissingCompletionEvidence::RefreshedWorkpoint);
    }
    if request.acceptance.criteria.is_empty() {
        missing.insert(MissingCompletionEvidence::AcceptanceCriteria);
    }
    if request.policy.adversarial_verifier_required
        && request.acceptance.adversarial_verifier_verdict.is_none()
    {
        missing.insert(MissingCompletionEvidence::AdversarialVerifier);
    }
    missing
}

fn validate_git_status(
    status: &GitStatusEvidence,
    missing_class: MissingCompletionEvidence,
    missing: &mut BTreeSet<MissingCompletionEvidence>,
) {
    if status.head_ref.trim().is_empty()
        || status.status_artifact_ref.trim().is_empty()
        || !valid_sha256(&status.status_sha256)
    {
        missing.insert(missing_class);
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn completion_evidence_classes(bundle: &CompletionEvidenceBundle) -> Vec<String> {
    let mut classes = BTreeSet::from([
        "project_workspace".to_owned(),
        "git_status".to_owned(),
        "diff".to_owned(),
        "workpoint".to_owned(),
        "model_usage".to_owned(),
        "resource_usage".to_owned(),
        "stream_manifest".to_owned(),
        "receipt_preview".to_owned(),
    ]);
    classes.extend(
        bundle
            .verification_runs
            .iter()
            .map(|run| run.verification_class.clone()),
    );
    classes.into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CompletionProtocolError {
    #[error("completion requires a valid Silent Session")]
    InvalidSession,
    #[error("completion requires a valid Silent Session run")]
    InvalidRun,
    #[error("completion evaluation requires an exited run in completing state")]
    ProcessNotCompleting,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::{
        ModelBinding, OperatorAskBinding, SilentSessionConfigRevisionId, SilentSessionHealth,
        SilentSessionId, SilentSessionRunId, SilentSessionVersions, WorkpointBinding,
        WorkspaceBinding, WorkspaceStrategy,
    };
    use chrono::Duration;

    fn session_and_run() -> (SilentSession, SilentSessionRun) {
        let now = Utc::now();
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let session = SilentSession {
            schema: crate::silent_session::SILENT_SESSION_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            session_id,
            display_name: "completion".into(),
            created_at: now - Duration::minutes(5),
            created_by_actor_ref: "actor:test".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "runner".into(),
            project_root: crate::test_support::absolute_path("silent-completion-project"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "continuity:test".into(),
            trajectory_ref: Some("trajectory:test".into()),
            workpoint_ref: Some(WorkpointBinding {
                workpoint_id: "workpoint:test".into(),
                revision: Some(9),
            }),
            work_item_ref: Some("focusa-a6yq6.7.4".into()),
            operator_ask: OperatorAskBinding::capture("ask:completion", "prove completion", 1, now),
            mission: "prove completion".into(),
            lifecycle_state: SilentSessionLifecycleState::Completing,
            health: SilentSessionHealth::ProcessExited,
            semantic_observation: None,
            active_run_id: Some(run_id),
            config_revision_id: SilentSessionConfigRevisionId::new(),
            writer_lease_ref: None,
            retention_policy_ref: "retention:test".into(),
            receipt_refs: vec![],
        };
        let run = SilentSessionRun {
            schema: crate::silent_session::SILENT_SESSION_RUN_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            run_id,
            session_id,
            generation: 1,
            runner_id: "runner:test".into(),
            adapter_id: "adapter:test".into(),
            process_backend_id: "process:test".into(),
            requested_model_binding: ModelBinding {
                provider: "provider".into(),
                model: "model".into(),
                thinking: None,
            },
            effective_model_binding: None,
            observed_model_binding: None,
            workspace_binding: WorkspaceBinding {
                workspace_id: "workspace:test".into(),
                root: crate::test_support::absolute_path("silent-completion-worktree"),
                strategy: WorkspaceStrategy::IsolatedWorktree,
                branch_ref: Some("branch:test".into()),
            },
            process_identity: None,
            harness_native_session_ref: None,
            started_at: Some(now - Duration::minutes(4)),
            ended_at: Some(now),
            exit_status: Some(0),
            current_event_seq: 10,
            output_stream_refs: vec!["stream:test".into()],
            runtime_checkpoint_refs: vec![],
            workpoint_checkpoint_refs: vec!["workpoint-checkpoint:final".into()],
        };
        (session, run)
    }

    fn bundle() -> CompletionEvidenceBundle {
        let status = GitStatusEvidence {
            head_ref: "aaaaaaaa".into(),
            status_artifact_ref: "artifact:git-status".into(),
            status_sha256: "a".repeat(64),
            dirty: false,
        };
        CompletionEvidenceBundle {
            project_root: crate::test_support::absolute_path("silent-completion-project"),
            project_identity_ref: "project:focusa".into(),
            workspace_root: crate::test_support::absolute_path("silent-completion-worktree"),
            workspace_ref: "workspace:test".into(),
            starting_git_status: status.clone(),
            ending_git_status: status,
            diff: DiffEvidence {
                bounded_summary: "one file changed".into(),
                full_diff_artifact_ref: "artifact:full-diff".into(),
                full_diff_sha256: "b".repeat(64),
                files_changed: BTreeSet::from([PathBuf::from("src/lib.rs")]),
            },
            verification_runs: vec![VerificationRunEvidence {
                verification_class: "tests".into(),
                command_ref: "command:tests".into(),
                output_artifact_ref: "artifact:tests".into(),
                output_sha256: "c".repeat(64),
                exit_code: 0,
                required: true,
            }],
            commit_refs: vec!["bbbbbbbb".into()],
            final_workpoint_checkpoint_ref: "workpoint-checkpoint:final".into(),
            unresolved_blockers: vec![],
            context_authority_refs: vec!["context-authority:mutation".into()],
            model_usage: ModelUsageEvidence {
                requested_model_ref: "model:requested".into(),
                effective_model_ref: "model:effective".into(),
                observed_model_ref: "model:observed".into(),
                input_tokens: 10,
                output_tokens: 5,
                cost_microunits: 3,
            },
            resource_usage: BTreeMap::from([("peak_rss_bytes".into(), 100)]),
            stream_manifest_ref: "stream-manifest:1".into(),
            stream_manifest_sha256: "d".repeat(64),
            completion_verifier_ref: "verifier:completion".into(),
            completion_verifier_passed: true,
            receipt_preview_ref: "receipt-preview:1".into(),
            receipt_preview_sha256: "e".repeat(64),
        }
    }

    fn policy() -> CompletionPolicy {
        CompletionPolicy {
            code_changing: true,
            commit_required: true,
            adversarial_verifier_required: true,
            required_verification_classes: BTreeSet::from(["tests".into()]),
        }
    }

    fn acceptance() -> AcceptanceEvaluation {
        AcceptanceEvaluation {
            criteria: BTreeMap::from([("spec133-6.4".into(), true)]),
            adversarial_verifier_verdict: Some("passed".into()),
        }
    }

    #[test]
    fn complete_evidence_and_committed_receipt_are_required_for_completed() {
        let (session, run) = session_and_run();
        let bundle = bundle();
        let policy = policy();
        let acceptance = acceptance();
        let outcome = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: true,
            workpoint_refreshed: true,
            receipt_commit_ref: Some("receipt:committed"),
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(outcome.evaluation.decision, CompletionDecision::Completed);
        assert_eq!(
            outcome.next_lifecycle_state,
            SilentSessionLifecycleState::Completed
        );
        assert!(outcome.missing_evidence.is_empty());
    }

    #[test]
    fn missing_evidence_blocks_and_never_completes() {
        let (session, run) = session_and_run();
        let mut bundle = bundle();
        bundle.diff.files_changed.clear();
        bundle.final_workpoint_checkpoint_ref.clear();
        bundle.verification_runs.clear();
        let policy = policy();
        let acceptance = acceptance();
        let outcome = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: false,
            workpoint_refreshed: false,
            receipt_commit_ref: None,
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(outcome.evaluation.decision, CompletionDecision::Blocked);
        assert_eq!(
            outcome.next_lifecycle_state,
            SilentSessionLifecycleState::Blocked
        );
        assert!(
            outcome
                .missing_evidence
                .contains(&MissingCompletionEvidence::FilesChanged)
        );
        assert!(
            outcome
                .missing_evidence
                .contains(&MissingCompletionEvidence::FinalWorkpointCheckpoint)
        );
    }

    #[test]
    fn failed_required_verification_fails_instead_of_completing() {
        let (session, run) = session_and_run();
        let mut bundle = bundle();
        bundle.verification_runs[0].exit_code = 1;
        let policy = policy();
        let acceptance = acceptance();
        let outcome = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: true,
            workpoint_refreshed: true,
            receipt_commit_ref: Some("receipt:committed"),
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(outcome.evaluation.decision, CompletionDecision::Failed);
        assert_eq!(outcome.reason.as_deref(), Some("verification_failed"));
    }

    #[test]
    fn unresolved_blockers_block_and_failed_verifier_fails() {
        let (session, run) = session_and_run();
        let mut blocked_bundle = bundle();
        blocked_bundle
            .unresolved_blockers
            .push("server proof pending".into());
        let policy = policy();
        let acceptance = acceptance();
        let blocked = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &blocked_bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: true,
            workpoint_refreshed: true,
            receipt_commit_ref: Some("receipt:committed"),
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(blocked.evaluation.decision, CompletionDecision::Blocked);
        assert_eq!(blocked.reason.as_deref(), Some("unresolved_blockers"));

        let mut failed_bundle = bundle();
        failed_bundle.completion_verifier_passed = false;
        let failed = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &failed_bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: true,
            workpoint_refreshed: true,
            receipt_commit_ref: Some("receipt:committed"),
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(failed.evaluation.decision, CompletionDecision::Failed);
    }

    #[test]
    fn receipt_preview_without_commit_remains_completing() {
        let (session, run) = session_and_run();
        let bundle = bundle();
        let policy = policy();
        let acceptance = acceptance();
        let outcome = evaluate_completion(&CompletionEvaluationRequest {
            session: &session,
            run: &run,
            bundle: &bundle,
            policy: &policy,
            acceptance: &acceptance,
            project_identity_refreshed: true,
            workpoint_refreshed: true,
            receipt_commit_ref: None,
            evaluated_at: Utc::now(),
        })
        .unwrap();
        assert_eq!(outcome.evaluation.decision, CompletionDecision::Incomplete);
        assert_eq!(
            outcome.next_lifecycle_state,
            SilentSessionLifecycleState::Completing
        );
        assert!(
            outcome
                .missing_evidence
                .contains(&MissingCompletionEvidence::ReceiptCommit)
        );
    }
}
