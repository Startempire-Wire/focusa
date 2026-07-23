//! Daemon-backed operator projections, watch, notifications, and wizard contracts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentDashboardCard {
    pub session_id: String,
    pub run_id: String,
    pub generation: u64,
    pub lifecycle: String,
    pub health: String,
    pub project_ref: String,
    pub work_item_ref: String,
    pub model_ref: String,
    pub started_at: String,
    pub last_activity_at: String,
    pub output_cursor: String,
    pub resource_summary: String,
    pub checkpoint_ref: String,
    pub blocker: Option<String>,
    pub evidence_ref: String,
    pub completion_posture: String,
    pub available_controls: Vec<String>,
    pub daemon_projection: bool,
    pub scoped_authorization: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WatchView {
    Summary,
    Text,
    Tools,
    Stdout,
    Stderr,
    Events,
    Raw,
    Evidence,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorWatch {
    pub session_id: String,
    pub run_id: String,
    pub generation: u64,
    pub after_cursor: Option<String>,
    pub view: WatchView,
    pub bounded_limit: u32,
}
impl CursorWatch {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.generation > 0 && self.bounded_limit > 0 && self.bounded_limit <= 1000,
            "watch requires exact run guard and bounded page"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorControl {
    SendText,
    SendFollowUp,
    SendSteering,
    SendSpecialKey,
    SoftPause,
    HardPause,
    Resume,
    Interrupt,
    ControlledStop,
    ForceCancel,
    Restart,
    Adopt,
    Handoff,
    OpenWorktree,
    OpenEvidence,
    OpenReceipt,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTrigger {
    WaitingInput,
    JudgmentBlocker,
    ModelMismatch,
    AuthEntitlementFailure,
    RepeatedProviderFailure,
    ResourcePressure,
    CheckpointFailure,
    ProcessFailure,
    OrphanedRun,
    CompletionEvidenceMissing,
    VerifiedCompletion,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorNotification {
    pub trigger: NotificationTrigger,
    pub dedupe_key: String,
    pub why: String,
    pub exact_action: String,
    pub channels: Vec<String>,
}
impl OperatorNotification {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.dedupe_key.is_empty(),
            "notification dedupe key required"
        );
        anyhow::ensure!(
            !self.why.is_empty() && !self.exact_action.is_empty(),
            "notification must expose why and exact action"
        );
        anyhow::ensure!(
            !self.channels.is_empty(),
            "background work cannot be invisible"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WizardStep {
    ProjectIdentity,
    ContinuityWorkpoint,
    WorkItemMission,
    WorkspaceStrategy,
    HarnessProfile,
    ExactProviderModelThinking,
    AuthenticationEntitlement,
    PolicyPreset,
    ResourceCostBudgets,
    ContextAuthorityWriterLease,
    EffectiveConfigurationPreview,
    ApproveLaunch,
    OpenLiveWatch,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreationWizardState {
    pub completed_steps: Vec<WizardStep>,
    pub provider_visible: bool,
    pub model_visible: bool,
    pub mutation_started: bool,
}
impl CreationWizardState {
    pub fn authorize_launch(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.completed_steps.len() == 13,
            "all 13 creation steps are required"
        );
        anyhow::ensure!(
            self.provider_visible && self.model_visible,
            "provider and model must be visible before mutation"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientProjection {
    pub surface: String,
    pub daemon_api_ref: String,
    pub bounded_rehydrate_ref: String,
    pub authority_minted: bool,
    pub foreground_pi_required: bool,
}
impl ClientProjection {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.daemon_api_ref.is_empty() && !self.bounded_rehydrate_ref.is_empty(),
            "daemon API and bounded rehydrate refs required"
        );
        anyhow::ensure!(
            !self.authority_minted,
            "client projection cannot mint authority"
        );
        anyhow::ensure!(
            !self.foreground_pi_required,
            "projection cannot depend on foreground Pi"
        );
        Ok(())
    }
}
