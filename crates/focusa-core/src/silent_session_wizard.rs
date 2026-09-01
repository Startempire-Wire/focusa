//! Exact creation wizard and bounded operator context projection.

use crate::silent_session::{ModelBinding, WorkpointBinding, WorkspaceStrategy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub const SILENT_SESSION_WIZARD_SCHEMA: &str = "focusa.silent_session_wizard.v1";
pub const OPERATOR_CONTEXT_SUMMARY_SCHEMA: &str = "focusa.operator_context_summary.v1";
pub const MAX_SUMMARY_ITEMS: usize = 20;
pub const MAX_SUMMARY_TEXT_BYTES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WizardStep {
    ProjectIdentity,
    ContinuityAndWorkpoint,
    WorkItemAndMission,
    WorkspaceStrategy,
    HarnessProfile,
    ExactModel,
    AuthenticationAndEntitlement,
    PolicyPreset,
    ResourceAndCostBudgets,
    ContextAuthorityAndWriterLease,
    EffectiveConfigurationPreview,
    ApproveAndLaunch,
    OpenLiveWatch,
    Complete,
}

impl WizardStep {
    const fn ordinal(self) -> u8 {
        match self {
            Self::ProjectIdentity => 1,
            Self::ContinuityAndWorkpoint => 2,
            Self::WorkItemAndMission => 3,
            Self::WorkspaceStrategy => 4,
            Self::HarnessProfile => 5,
            Self::ExactModel => 6,
            Self::AuthenticationAndEntitlement => 7,
            Self::PolicyPreset => 8,
            Self::ResourceAndCostBudgets => 9,
            Self::ContextAuthorityAndWriterLease => 10,
            Self::EffectiveConfigurationPreview => 11,
            Self::ApproveAndLaunch => 12,
            Self::OpenLiveWatch => 13,
            Self::Complete => 14,
        }
    }

    const fn next(self) -> Self {
        match self {
            Self::ProjectIdentity => Self::ContinuityAndWorkpoint,
            Self::ContinuityAndWorkpoint => Self::WorkItemAndMission,
            Self::WorkItemAndMission => Self::WorkspaceStrategy,
            Self::WorkspaceStrategy => Self::HarnessProfile,
            Self::HarnessProfile => Self::ExactModel,
            Self::ExactModel => Self::AuthenticationAndEntitlement,
            Self::AuthenticationAndEntitlement => Self::PolicyPreset,
            Self::PolicyPreset => Self::ResourceAndCostBudgets,
            Self::ResourceAndCostBudgets => Self::ContextAuthorityAndWriterLease,
            Self::ContextAuthorityAndWriterLease => Self::EffectiveConfigurationPreview,
            Self::EffectiveConfigurationPreview => Self::ApproveAndLaunch,
            Self::ApproveAndLaunch => Self::OpenLiveWatch,
            Self::OpenLiveWatch | Self::Complete => Self::Complete,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCostBudgets {
    pub wall_time_seconds: u64,
    pub token_budget: u64,
    pub cost_microunits: u64,
    pub memory_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectiveConfigurationPreview {
    pub config_ref: String,
    pub config_sha256: String,
    pub provider: String,
    pub model: String,
    pub thinking: Option<String>,
    pub workspace_root: PathBuf,
    pub mutation_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionWizardDraft {
    pub schema: String,
    pub wizard_id: Uuid,
    pub current_step: WizardStep,
    pub completed_steps: BTreeSet<WizardStep>,
    pub project_root: Option<PathBuf>,
    pub project_identity_ref: Option<String>,
    pub continuity_id: Option<String>,
    pub workpoint_ref: Option<WorkpointBinding>,
    pub work_item_ref: Option<String>,
    pub mission: Option<String>,
    pub workspace_strategy: Option<WorkspaceStrategy>,
    pub workspace_root: Option<PathBuf>,
    pub harness_profile_ref: Option<String>,
    pub model: Option<ModelBinding>,
    pub authentication_ref: Option<String>,
    pub entitlement_ref: Option<String>,
    pub policy_preset_ref: Option<String>,
    pub budgets: Option<ResourceCostBudgets>,
    pub context_authority_ref: Option<String>,
    pub writer_lease_ref: Option<String>,
    pub effective_preview: Option<EffectiveConfigurationPreview>,
    pub approval_ref: Option<String>,
    pub launch_action_digest: Option<String>,
    pub live_watch_ref: Option<String>,
    pub mutation_allowed: bool,
}

impl SilentSessionWizardDraft {
    pub fn new() -> Self {
        Self {
            schema: SILENT_SESSION_WIZARD_SCHEMA.into(),
            wizard_id: Uuid::now_v7(),
            current_step: WizardStep::ProjectIdentity,
            completed_steps: BTreeSet::new(),
            project_root: None,
            project_identity_ref: None,
            continuity_id: None,
            workpoint_ref: None,
            work_item_ref: None,
            mission: None,
            workspace_strategy: None,
            workspace_root: None,
            harness_profile_ref: None,
            model: None,
            authentication_ref: None,
            entitlement_ref: None,
            policy_preset_ref: None,
            budgets: None,
            context_authority_ref: None,
            writer_lease_ref: None,
            effective_preview: None,
            approval_ref: None,
            launch_action_digest: None,
            live_watch_ref: None,
            mutation_allowed: false,
        }
    }
}

impl Default for SilentSessionWizardDraft {
    fn default() -> Self {
        Self::new()
    }
}

pub enum WizardInput {
    ProjectIdentity {
        project_root: PathBuf,
        project_identity_ref: String,
        verified: bool,
    },
    ContinuityAndWorkpoint {
        continuity_id: String,
        workpoint_ref: WorkpointBinding,
    },
    WorkItemAndMission {
        work_item_ref: String,
        mission: String,
    },
    WorkspaceStrategy {
        strategy: WorkspaceStrategy,
        workspace_root: PathBuf,
    },
    HarnessProfile {
        harness_profile_ref: String,
    },
    ExactModel {
        model: ModelBinding,
    },
    AuthenticationAndEntitlement {
        authentication_ref: String,
        entitlement_ref: String,
        verified: bool,
    },
    PolicyPreset {
        policy_preset_ref: String,
    },
    ResourceAndCostBudgets {
        budgets: ResourceCostBudgets,
    },
    ContextAuthorityAndWriterLease {
        context_authority_ref: String,
        writer_lease_ref: String,
    },
    EffectiveConfigurationPreview {
        preview: EffectiveConfigurationPreview,
    },
    ApproveAndLaunch {
        approval_ref: String,
        launch_action_digest: String,
        approved: bool,
    },
    OpenLiveWatch {
        live_watch_ref: String,
    },
}

pub fn advance_wizard(
    mut draft: SilentSessionWizardDraft,
    input: WizardInput,
) -> Result<SilentSessionWizardDraft, WizardError> {
    if draft.schema != SILENT_SESSION_WIZARD_SCHEMA
        || draft.wizard_id.get_version() != Some(uuid::Version::SortRand)
        || draft.current_step == WizardStep::Complete
    {
        return Err(WizardError::InvalidWizard);
    }
    let input_step = input_step(&input);
    if input_step != draft.current_step
        || input_step.ordinal() != draft.completed_steps.len() as u8 + 1
    {
        return Err(WizardError::StepSkippedOrRepeated);
    }
    match input {
        WizardInput::ProjectIdentity {
            project_root,
            project_identity_ref,
            verified,
        } => {
            if !verified || !project_root.is_absolute() || empty(&project_identity_ref) {
                return Err(WizardError::ProjectIdentityUnverified);
            }
            draft.project_root = Some(project_root);
            draft.project_identity_ref = Some(project_identity_ref);
        }
        WizardInput::ContinuityAndWorkpoint {
            continuity_id,
            workpoint_ref,
        } => {
            if empty(&continuity_id) || empty(&workpoint_ref.workpoint_id) {
                return Err(WizardError::InvalidStepInput);
            }
            draft.continuity_id = Some(continuity_id);
            draft.workpoint_ref = Some(workpoint_ref);
        }
        WizardInput::WorkItemAndMission {
            work_item_ref,
            mission,
        } => {
            if empty(&work_item_ref) || empty(&mission) {
                return Err(WizardError::InvalidStepInput);
            }
            draft.work_item_ref = Some(work_item_ref);
            draft.mission = Some(mission);
        }
        WizardInput::WorkspaceStrategy {
            strategy,
            workspace_root,
        } => {
            if !workspace_root.is_absolute() {
                return Err(WizardError::InvalidStepInput);
            }
            draft.workspace_strategy = Some(strategy);
            draft.workspace_root = Some(workspace_root);
        }
        WizardInput::HarnessProfile {
            harness_profile_ref,
        } => {
            require(&harness_profile_ref)?;
            draft.harness_profile_ref = Some(harness_profile_ref);
        }
        WizardInput::ExactModel { model } => {
            if empty(&model.provider) || empty(&model.model) {
                return Err(WizardError::ExactModelRequired);
            }
            draft.model = Some(model);
        }
        WizardInput::AuthenticationAndEntitlement {
            authentication_ref,
            entitlement_ref,
            verified,
        } => {
            if !verified || empty(&authentication_ref) || empty(&entitlement_ref) {
                return Err(WizardError::ProviderAccessUnverified);
            }
            draft.authentication_ref = Some(authentication_ref);
            draft.entitlement_ref = Some(entitlement_ref);
        }
        WizardInput::PolicyPreset { policy_preset_ref } => {
            require(&policy_preset_ref)?;
            draft.policy_preset_ref = Some(policy_preset_ref);
        }
        WizardInput::ResourceAndCostBudgets { budgets } => {
            if budgets.wall_time_seconds == 0
                || budgets.token_budget == 0
                || budgets.cost_microunits == 0
                || budgets.memory_bytes == 0
                || budgets.output_bytes == 0
            {
                return Err(WizardError::BudgetRequired);
            }
            draft.budgets = Some(budgets);
        }
        WizardInput::ContextAuthorityAndWriterLease {
            context_authority_ref,
            writer_lease_ref,
        } => {
            require(&context_authority_ref)?;
            require(&writer_lease_ref)?;
            draft.context_authority_ref = Some(context_authority_ref);
            draft.writer_lease_ref = Some(writer_lease_ref);
        }
        WizardInput::EffectiveConfigurationPreview { preview } => {
            let selected_model = draft
                .model
                .as_ref()
                .ok_or(WizardError::ExactModelRequired)?;
            if empty(&preview.config_ref)
                || !valid_sha256(&preview.config_sha256)
                || preview.provider != selected_model.provider
                || preview.model != selected_model.model
                || preview.thinking != selected_model.thinking
                || draft.workspace_root.as_ref() != Some(&preview.workspace_root)
                || preview.mutation_allowed
            {
                return Err(WizardError::EffectivePreviewMismatch);
            }
            draft.effective_preview = Some(preview);
            draft.mutation_allowed = false;
        }
        WizardInput::ApproveAndLaunch {
            approval_ref,
            launch_action_digest,
            approved,
        } => {
            if !approved
                || empty(&approval_ref)
                || !valid_sha256(&launch_action_digest)
                || draft.effective_preview.is_none()
            {
                return Err(WizardError::LaunchNotApproved);
            }
            draft.approval_ref = Some(approval_ref);
            draft.launch_action_digest = Some(launch_action_digest);
            draft.mutation_allowed = true;
        }
        WizardInput::OpenLiveWatch { live_watch_ref } => {
            require(&live_watch_ref)?;
            if !draft.mutation_allowed {
                return Err(WizardError::LaunchNotApproved);
            }
            draft.live_watch_ref = Some(live_watch_ref);
        }
    }
    draft.completed_steps.insert(input_step);
    draft.current_step = input_step.next();
    Ok(draft)
}

fn input_step(input: &WizardInput) -> WizardStep {
    match input {
        WizardInput::ProjectIdentity { .. } => WizardStep::ProjectIdentity,
        WizardInput::ContinuityAndWorkpoint { .. } => WizardStep::ContinuityAndWorkpoint,
        WizardInput::WorkItemAndMission { .. } => WizardStep::WorkItemAndMission,
        WizardInput::WorkspaceStrategy { .. } => WizardStep::WorkspaceStrategy,
        WizardInput::HarnessProfile { .. } => WizardStep::HarnessProfile,
        WizardInput::ExactModel { .. } => WizardStep::ExactModel,
        WizardInput::AuthenticationAndEntitlement { .. } => {
            WizardStep::AuthenticationAndEntitlement
        }
        WizardInput::PolicyPreset { .. } => WizardStep::PolicyPreset,
        WizardInput::ResourceAndCostBudgets { .. } => WizardStep::ResourceAndCostBudgets,
        WizardInput::ContextAuthorityAndWriterLease { .. } => {
            WizardStep::ContextAuthorityAndWriterLease
        }
        WizardInput::EffectiveConfigurationPreview { .. } => {
            WizardStep::EffectiveConfigurationPreview
        }
        WizardInput::ApproveAndLaunch { .. } => WizardStep::ApproveAndLaunch,
        WizardInput::OpenLiveWatch { .. } => WizardStep::OpenLiveWatch,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorContextSummary {
    pub schema: String,
    pub meaningful_deltas: Vec<String>,
    pub current_action: String,
    pub errors: Vec<String>,
    pub blockers: Vec<String>,
    pub tool_boundaries: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub checkpoint_refs: Vec<String>,
    pub full_output_cursor: String,
    pub full_output_artifact_ref: String,
    pub full_output_inlined: bool,
}

impl OperatorContextSummary {
    pub fn validate(&self) -> Result<(), WizardError> {
        if self.schema != OPERATOR_CONTEXT_SUMMARY_SCHEMA
            || empty(&self.current_action)
            || empty(&self.full_output_cursor)
            || empty(&self.full_output_artifact_ref)
            || self.full_output_inlined
        {
            return Err(WizardError::ContextFloodRisk);
        }
        for values in [
            &self.meaningful_deltas,
            &self.errors,
            &self.blockers,
            &self.tool_boundaries,
            &self.evidence_refs,
            &self.checkpoint_refs,
        ] {
            if values.len() > MAX_SUMMARY_ITEMS
                || values
                    .iter()
                    .any(|value| empty(value) || value.len() > MAX_SUMMARY_TEXT_BYTES)
            {
                return Err(WizardError::ContextFloodRisk);
            }
        }
        Ok(())
    }
}

fn empty(value: &str) -> bool {
    value.trim().is_empty()
}

fn require(value: &str) -> Result<(), WizardError> {
    if empty(value) {
        Err(WizardError::InvalidStepInput)
    } else {
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WizardError {
    #[error("wizard identity or terminal state is invalid")]
    InvalidWizard,
    #[error("wizard steps cannot be skipped, repeated, or reordered")]
    StepSkippedOrRepeated,
    #[error("ProjectIdentity must be verified")]
    ProjectIdentityUnverified,
    #[error("wizard step input is missing or invalid")]
    InvalidStepInput,
    #[error("exact provider and model are required")]
    ExactModelRequired,
    #[error("provider authentication and entitlement must be verified")]
    ProviderAccessUnverified,
    #[error("non-zero resource and cost budgets are required")]
    BudgetRequired,
    #[error("effective preview differs from selected model/workspace or enables mutation")]
    EffectivePreviewMismatch,
    #[error("launch requires explicit approval and exact action digest")]
    LaunchNotApproved,
    #[error("operator summary exceeds bounds or inlines full output")]
    ContextFloodRisk,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wp() -> WorkpointBinding {
        WorkpointBinding {
            workpoint_id: "workpoint:test".into(),
            revision: Some(1),
        }
    }

    #[test]
    fn exact_thirteen_steps_keep_mutation_disabled_until_approval_and_open_watch_last() {
        let mut draft = SilentSessionWizardDraft::new();
        let inputs = vec![
            WizardInput::ProjectIdentity {
                project_root: crate::test_support::absolute_path("silent-wizard-project"),
                project_identity_ref: "project:focusa".into(),
                verified: true,
            },
            WizardInput::ContinuityAndWorkpoint {
                continuity_id: "continuity:test".into(),
                workpoint_ref: wp(),
            },
            WizardInput::WorkItemAndMission {
                work_item_ref: "focusa-a6yq6.8.4".into(),
                mission: "implement wizard".into(),
            },
            WizardInput::WorkspaceStrategy {
                strategy: WorkspaceStrategy::IsolatedWorktree,
                workspace_root: crate::test_support::absolute_path("silent-wizard-worktree"),
            },
            WizardInput::HarnessProfile {
                harness_profile_ref: "harness:pi".into(),
            },
            WizardInput::ExactModel {
                model: ModelBinding {
                    provider: "anthropic".into(),
                    model: "claude-test".into(),
                    thinking: Some("high".into()),
                },
            },
            WizardInput::AuthenticationAndEntitlement {
                authentication_ref: "auth:verified".into(),
                entitlement_ref: "entitlement:verified".into(),
                verified: true,
            },
            WizardInput::PolicyPreset {
                policy_preset_ref: "preset:balanced".into(),
            },
            WizardInput::ResourceAndCostBudgets {
                budgets: ResourceCostBudgets {
                    wall_time_seconds: 3600,
                    token_budget: 100_000,
                    cost_microunits: 10_000,
                    memory_bytes: 1_000_000_000,
                    output_bytes: 10_000_000,
                },
            },
            WizardInput::ContextAuthorityAndWriterLease {
                context_authority_ref: "context-authority:1".into(),
                writer_lease_ref: "writer-lease:1".into(),
            },
            WizardInput::EffectiveConfigurationPreview {
                preview: EffectiveConfigurationPreview {
                    config_ref: "config:effective".into(),
                    config_sha256: "a".repeat(64),
                    provider: "anthropic".into(),
                    model: "claude-test".into(),
                    thinking: Some("high".into()),
                    workspace_root: crate::test_support::absolute_path("silent-wizard-worktree"),
                    mutation_allowed: false,
                },
            },
            WizardInput::ApproveAndLaunch {
                approval_ref: "approval:launch".into(),
                launch_action_digest: "b".repeat(64),
                approved: true,
            },
            WizardInput::OpenLiveWatch {
                live_watch_ref: "watch:session".into(),
            },
        ];
        for (index, input) in inputs.into_iter().enumerate() {
            draft = advance_wizard(draft, input).unwrap();
            if index < 11 {
                assert!(!draft.mutation_allowed);
            }
        }
        assert_eq!(draft.completed_steps.len(), 13);
        assert_eq!(draft.current_step, WizardStep::Complete);
        assert!(draft.mutation_allowed);
        assert_eq!(draft.model.unwrap().provider, "anthropic");
        assert_eq!(draft.live_watch_ref.as_deref(), Some("watch:session"));
    }

    #[test]
    fn steps_cannot_skip_and_auth_or_preview_mismatch_blocks_launch() {
        let draft = SilentSessionWizardDraft::new();
        assert_eq!(
            advance_wizard(
                draft.clone(),
                WizardInput::ExactModel {
                    model: ModelBinding {
                        provider: "provider".into(),
                        model: "model".into(),
                        thinking: None,
                    },
                },
            ),
            Err(WizardError::StepSkippedOrRepeated)
        );
        assert_eq!(
            advance_wizard(
                draft,
                WizardInput::ProjectIdentity {
                    project_root: crate::test_support::absolute_path("silent-wizard-project"),
                    project_identity_ref: "project:focusa".into(),
                    verified: false,
                },
            ),
            Err(WizardError::ProjectIdentityUnverified)
        );
    }

    #[test]
    fn operator_summary_rejects_full_output_and_context_floods() {
        let mut summary = OperatorContextSummary {
            schema: OPERATOR_CONTEXT_SUMMARY_SCHEMA.into(),
            meaningful_deltas: vec!["implemented exact wizard".into()],
            current_action: "review effective configuration".into(),
            errors: vec![],
            blockers: vec![],
            tool_boundaries: vec!["server owns builds".into()],
            evidence_refs: vec!["evidence:wizard".into()],
            checkpoint_refs: vec!["checkpoint:workpoint".into()],
            full_output_cursor: "cursor:full".into(),
            full_output_artifact_ref: "artifact:full-output".into(),
            full_output_inlined: false,
        };
        summary.validate().unwrap();
        summary.full_output_inlined = true;
        assert_eq!(summary.validate(), Err(WizardError::ContextFloodRisk));
        summary.full_output_inlined = false;
        summary.errors = (0..=MAX_SUMMARY_ITEMS)
            .map(|n| format!("error:{n}"))
            .collect();
        assert_eq!(summary.validate(), Err(WizardError::ContextFloodRisk));
    }
}
