//! Cognitive authority, checkpoints, evidence, receipts, transfer, and learning.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionBinding {
    pub current_ask: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub trajectory_ref: String,
    pub workpoint_ref: String,
    pub waypoints: Vec<String>,
    pub gap: String,
    pub action: String,
    pub object_refs: Vec<String>,
    pub hooks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_action: String,
    pub do_not_drift: Vec<String>,
    pub steering_revision: u64,
    pub project_verified: bool,
    pub generic_trajectory: bool,
}
impl MissionBinding {
    pub fn authorize(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.project_verified && !self.generic_trajectory,
            "project mismatch or generic trajectory blocks mutation"
        );
        anyhow::ensure!(
            !self.current_ask.is_empty() && !self.workpoint_ref.is_empty(),
            "ask and Workpoint binding required"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CognitiveBootstrap {
    pub context_packet_ref: String,
    pub context_bounded: bool,
    pub context_advisory_only: bool,
    pub action_authority_ref: String,
    pub authority_fresh: bool,
    pub action_risks: Vec<String>,
    pub ontology_refs: Vec<String>,
    pub agent_bootstrap_verified: bool,
}
impl CognitiveBootstrap {
    pub fn authorize_mutation(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.context_bounded && self.context_advisory_only,
            "context packet boundary invalid"
        );
        anyhow::ensure!(
            self.authority_fresh && !self.action_authority_ref.is_empty(),
            "fresh action-specific authority required"
        );
        anyhow::ensure!(
            self.agent_bootstrap_verified && !self.ontology_refs.is_empty(),
            "verified bootstrap and ontology refs required"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCheckpoint {
    pub stream_cursor: String,
    pub resource_usage_ref: String,
    pub retry_ledger_ref: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeaningfulCheckpointTrigger {
    MissionChanged,
    SteeringAccepted,
    ActionChanged,
    BlockerChanged,
    EvidenceChanged,
    BeforeRiskyMutation,
    BeforeTransfer,
    BeforeModelSwitch,
    BeforeCompletion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionEvidenceBundle {
    pub workspace_ref: String,
    pub git_status_ref: String,
    pub diff_ref: String,
    pub files_ref: String,
    pub tests_ref: String,
    pub lint_ref: String,
    pub commits_ref: String,
    pub checkpoint_ref: String,
    pub blockers_ref: String,
    pub authority_ref: String,
    pub model_ref: String,
    pub resources_ref: String,
    pub streams_ref: String,
    pub acceptance_verified: bool,
    pub adversarial_verified: bool,
}
impl CompletionEvidenceBundle {
    pub fn authorize_completion(&self) -> anyhow::Result<()> {
        let refs = [
            &self.workspace_ref,
            &self.git_status_ref,
            &self.diff_ref,
            &self.files_ref,
            &self.tests_ref,
            &self.lint_ref,
            &self.commits_ref,
            &self.checkpoint_ref,
            &self.blockers_ref,
            &self.authority_ref,
            &self.model_ref,
            &self.resources_ref,
            &self.streams_ref,
        ];
        anyhow::ensure!(
            refs.iter().all(|r| !r.is_empty()),
            "completion evidence missing"
        );
        anyhow::ensure!(
            self.acceptance_verified && self.adversarial_verified,
            "completion verification failed"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptKind {
    WorkSession,
    RiskyMutation,
    BlockedClaim,
    Handoff,
    Bootstrap,
    Closure,
    Final,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptStage {
    Prepare,
    Validate,
    Authorize,
    Provider,
    Reconcile,
    Audit,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClosureProposal {
    pub execution_mode: String,
    pub receipt_kind: ReceiptKind,
    pub hash_chain_ref: String,
    pub stage: ReceiptStage,
    pub closure_authority_ref: Option<String>,
}
impl ClosureProposal {
    pub fn authorize_close(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.execution_mode == "silent_session",
            "silent receipt execution mode required"
        );
        anyhow::ensure!(
            !self.hash_chain_ref.is_empty(),
            "existing receipt hash chain required"
        );
        anyhow::ensure!(
            self.stage == ReceiptStage::Audit && self.closure_authority_ref.is_some(),
            "session may propose but cannot self-close"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferReason {
    ForegroundTakeover,
    Handoff,
    ModelSwitch,
    RuntimeLoss,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningPacket {
    pub transfer_reason: TransferReason,
    pub session_transfer_ref: String,
    pub reconstruction_checkpoint_ref: String,
    pub prediction_ref: String,
    pub prediction_evaluated: bool,
    pub lesson_ref: String,
    pub evidence_refs: Vec<String>,
    pub outcome_score: f64,
    pub governance_override: bool,
}
impl LearningPacket {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.session_transfer_ref.is_empty() && !self.reconstruction_checkpoint_ref.is_empty(),
            "transfer and reconstruction evidence required"
        );
        anyhow::ensure!(
            !self.prediction_ref.is_empty() && self.prediction_evaluated,
            "prediction must be recorded and evaluated"
        );
        anyhow::ensure!(
            !self.lesson_ref.is_empty() && !self.evidence_refs.is_empty(),
            "evidence-backed lesson required"
        );
        anyhow::ensure!(
            !self.governance_override,
            "learning cannot override governance"
        );
        Ok(())
    }
}
