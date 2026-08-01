use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{LifecycleScope, LifecycleState, LifecycleTransactionKind, MaintenanceAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallLifecycleValidationError {
    EmptyTransactionId,
    EmptyHostId,
    TargetSelectionRequired,
    ExplicitProjectPathRequired,
    PlatformEvidenceRequired,
    ProjectInspectionWithoutExactScope,
    UnsupportedTargetMustBlock,
    IncompletePreflightFinding,
    RecoveryRequiresGuidance,
    UnknownCompletionRequiresInspection,
    RollbackBoundaryIncomplete,
    PreservationDeclarationIncomplete,
    DestructiveActionNotAuthorized,
    UninstallMustPreserveUserData,
    IdempotencyKeyRequired,
    IdempotencyConflict,
    CompletedTransactionRequiresReceipt,
    TransactionKindMismatch,
    ProjectScopeForbiddenForHostInstall,
    ProjectSelectionForbiddenForHostInstall,
    ProjectScopeRequiredForOnboarding,
    ScopeMismatch,
    BootstrapPreviewRequired,
    MutationConfirmationRequired,
    MaintenanceActionMismatch,
    ArtifactSelectionMismatch,
    ArtifactTrustIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionSelection {
    Interactive,
    Headless,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationSelection {
    Evaluation,
    Commercial,
    AuthorizedDevelopment,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelSelection {
    Stable,
    Preview,
    Nightly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencySelection {
    ApprovedInstall,
    VerifyOnly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceSelection {
    SupportedUserService,
    NoService,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitSelection {
    Preserve,
    ExplicitInitialize,
    Skip,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskProviderSelection {
    Preserve,
    ExplicitSupportedProvider,
    Skip,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionSelection {
    Preserve,
    GovernedPreviewGenerate,
    Skip,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasSelection {
    Guided,
    Full,
    Off,
    LeaveUnchanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "selection", content = "value")]
pub enum ProjectSelection {
    Skip,
    ExistingPath(String),
    NewPath(String),
}

impl ProjectSelection {
    pub fn explicit_path(&self) -> Option<&str> {
        match self {
            Self::Skip => None,
            Self::ExistingPath(path) | Self::NewPath(path) => Some(path),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "integration", content = "identifier")]
pub enum IntegrationSelection {
    Pi,
    Uiai,
    Menubar,
    DeclaredHarness(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleSelections {
    pub interaction: InteractionSelection,
    pub authorization: AuthorizationSelection,
    pub channel: ChannelSelection,
    pub target: String,
    pub dependencies: DependencySelection,
    pub service: ServiceSelection,
    #[serde(default)]
    pub integrations: Vec<IntegrationSelection>,
    pub project: ProjectSelection,
    pub git: GitSelection,
    pub task_provider: TaskProviderSelection,
    pub instructions: InstructionSelection,
    pub canvas: CanvasSelection,
}

impl LifecycleSelections {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.target.trim().is_empty() {
            return Err(InstallLifecycleValidationError::TargetSelectionRequired);
        }
        if self
            .project
            .explicit_path()
            .is_some_and(|path| path.trim().is_empty())
        {
            return Err(InstallLifecycleValidationError::ExplicitProjectPathRequired);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightFindingDisposition {
    Required,
    Optional,
    AlreadySatisfied,
    OperatorChoice,
    Unsupported,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightSubject {
    HostTarget,
    UserHomeBoundary,
    Binary,
    Daemon,
    Service,
    Extension,
    Skill,
    FocusaState,
    Dependency,
    Network,
    ArtifactMetadata,
    License,
    PiCapability,
    UiaiCapability,
    MenubarCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicensePosture {
    Evaluation,
    Commercial,
    AuthorizedDevelopment,
    ActivationRequired,
    Blocked,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightFinding {
    pub finding_id: String,
    pub subject: PreflightSubject,
    pub disposition: PreflightFindingDisposition,
    pub summary: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub host_id: String,
    pub os: String,
    pub architecture: String,
    pub user_home_boundary: String,
    pub shell: String,
    pub tty_present: bool,
    pub supported_target: Option<String>,
    pub existing_version_set: Vec<ComponentVersion>,
    pub writable_user_targets: Vec<String>,
    pub network_available: bool,
    pub offline_allowed: bool,
    pub artifact_metadata_reachable: bool,
    pub license_posture: LicensePosture,
    pub explicit_project_path: Option<String>,
    pub inspected_project_path: Option<String>,
    #[serde(default)]
    pub findings: Vec<PreflightFinding>,
    pub inspected_at: DateTime<Utc>,
}

impl PreflightReport {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.host_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyHostId);
        }
        if self.os.trim().is_empty() || self.architecture.trim().is_empty() {
            return Err(InstallLifecycleValidationError::PlatformEvidenceRequired);
        }
        if let Some(inspected) = self.inspected_project_path.as_deref() {
            if self.explicit_project_path.as_deref() != Some(inspected) {
                return Err(InstallLifecycleValidationError::ProjectInspectionWithoutExactScope);
            }
        }
        if self.supported_target.is_none()
            && !self.findings.iter().any(|finding| {
                finding.disposition == PreflightFindingDisposition::Unsupported
                    || finding.disposition == PreflightFindingDisposition::Blocked
            })
        {
            return Err(InstallLifecycleValidationError::UnsupportedTargetMustBlock);
        }
        if self.findings.iter().any(|finding| {
            finding.finding_id.trim().is_empty() || finding.summary.trim().is_empty()
        }) {
            return Err(InstallLifecycleValidationError::IncompletePreflightFinding);
        }
        Ok(())
    }

    pub fn mutation_ready(&self) -> bool {
        self.validate().is_ok()
            && self.supported_target.is_some()
            && !self.findings.iter().any(|finding| {
                matches!(
                    finding.disposition,
                    PreflightFindingDisposition::Unsupported | PreflightFindingDisposition::Blocked
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub component: String,
    pub version: String,
    pub compatible: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTrustEvidence {
    pub declared_version: String,
    pub declared_channel: ChannelSelection,
    pub target: String,
    pub metadata_complete: bool,
    #[serde(default)]
    pub checksum_refs: Vec<String>,
    #[serde(default)]
    pub signature_refs: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<String>,
    pub staged_extraction_verified: bool,
}

impl ArtifactTrustEvidence {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.is_complete() {
            Ok(())
        } else {
            Err(InstallLifecycleValidationError::ArtifactTrustIncomplete)
        }
    }

    fn is_complete(&self) -> bool {
        !self.declared_version.trim().is_empty()
            && !self.target.trim().is_empty()
            && self.metadata_complete
            && !self.checksum_refs.is_empty()
            && !self.signature_refs.is_empty()
            && !self.provenance_refs.is_empty()
            && self.staged_extraction_verified
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryClass {
    UnsupportedHost,
    ArtifactIncomplete,
    TrustFailure,
    PermissionBoundary,
    DaemonDegraded,
    IntegrationIncompatible,
    ScopeMismatch,
    ConfirmationMissing,
    ProviderUnavailable,
    ProjectConflict,
    UpdatePartial,
    UninstallAmbiguous,
    UnknownCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryInstructions {
    pub primary_class: RecoveryClass,
    pub summary: String,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    pub resume_from_state: LifecycleState,
    pub inspect_before_retry: bool,
    pub requires_confirmation: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl RecoveryInstructions {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.summary.trim().is_empty() || self.operator_actions.is_empty() {
            return Err(InstallLifecycleValidationError::RecoveryRequiresGuidance);
        }
        if self.primary_class == RecoveryClass::UnknownCompletion && !self.inspect_before_retry {
            return Err(InstallLifecycleValidationError::UnknownCompletionRequiresInspection);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackBoundary {
    pub replacement_planned: bool,
    #[serde(default)]
    pub prior_version_set: Vec<ComponentVersion>,
    #[serde(default)]
    pub rollback_artifact_refs: Vec<String>,
    #[serde(default)]
    pub rollback_trust_refs: Vec<String>,
    pub atomic_activation: bool,
    pub preserves_user_data: bool,
    pub preserves_project_data: bool,
}

impl RollbackBoundary {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.replacement_planned
            && (self.prior_version_set.is_empty()
                || self.rollback_artifact_refs.is_empty()
                || self.rollback_trust_refs.is_empty()
                || !self.preserves_user_data
                || !self.preserves_project_data)
        {
            return Err(InstallLifecycleValidationError::RollbackBoundaryIncomplete);
        }
        Ok(())
    }

    pub fn rollback_available(&self) -> bool {
        !self.replacement_planned || self.validate().is_ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreservationDisposition {
    Preserve,
    RemoveManagedArtifact,
    NotTouched,
    PurgeConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleDataClass {
    ManagedBinaries,
    Services,
    Integrations,
    FocusaState,
    LogsCaches,
    LicenseState,
    ProviderHarnessState,
    ProjectFiles,
    ProjectTaskData,
    OperatorAuthoredInstructions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservationItem {
    pub data_class: LifecycleDataClass,
    pub disposition: PreservationDisposition,
    pub owner_authorized: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservationDeclaration {
    pub action: MaintenanceAction,
    pub items: Vec<PreservationItem>,
    pub destructive_purge_confirmed: bool,
}

impl PreservationDeclaration {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        const ALL_CLASSES: [LifecycleDataClass; 10] = [
            LifecycleDataClass::ManagedBinaries,
            LifecycleDataClass::Services,
            LifecycleDataClass::Integrations,
            LifecycleDataClass::FocusaState,
            LifecycleDataClass::LogsCaches,
            LifecycleDataClass::LicenseState,
            LifecycleDataClass::ProviderHarnessState,
            LifecycleDataClass::ProjectFiles,
            LifecycleDataClass::ProjectTaskData,
            LifecycleDataClass::OperatorAuthoredInstructions,
        ];
        if ALL_CLASSES.iter().any(|class| {
            self.items
                .iter()
                .filter(|item| item.data_class == *class)
                .count()
                != 1
        }) {
            return Err(InstallLifecycleValidationError::PreservationDeclarationIncomplete);
        }
        for item in &self.items {
            if item.disposition == PreservationDisposition::PurgeConfirmed
                && (!self.destructive_purge_confirmed || !item.owner_authorized)
            {
                return Err(InstallLifecycleValidationError::DestructiveActionNotAuthorized);
            }
            if matches!(
                item.data_class,
                LifecycleDataClass::ProviderHarnessState
                    | LifecycleDataClass::ProjectFiles
                    | LifecycleDataClass::ProjectTaskData
                    | LifecycleDataClass::OperatorAuthoredInstructions
            ) && matches!(
                item.disposition,
                PreservationDisposition::RemoveManagedArtifact
                    | PreservationDisposition::PurgeConfirmed
            ) && !item.owner_authorized
            {
                return Err(InstallLifecycleValidationError::DestructiveActionNotAuthorized);
            }
        }
        if self.action == MaintenanceAction::Uninstall
            && self.items.iter().any(|item| {
                matches!(
                    item.data_class,
                    LifecycleDataClass::FocusaState
                        | LifecycleDataClass::LicenseState
                        | LifecycleDataClass::ProviderHarnessState
                        | LifecycleDataClass::ProjectFiles
                        | LifecycleDataClass::ProjectTaskData
                        | LifecycleDataClass::OperatorAuthoredInstructions
                ) && item.disposition != PreservationDisposition::Preserve
            })
        {
            return Err(InstallLifecycleValidationError::UninstallMustPreserveUserData);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionProgress {
    Pending,
    InProgress,
    Blocked,
    Partial,
    Complete,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub key: String,
    pub intent_digest: String,
    pub replay_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDisposition {
    Start,
    Resume,
    ReturnStoredReceipt,
    InspectBeforeResume,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLifecycleState {
    pub transaction_id: String,
    pub transaction_kind: LifecycleTransactionKind,
    pub scope: LifecycleScope,
    pub idempotency: IdempotencyRecord,
    pub current_state: LifecycleState,
    pub progress: TransactionProgress,
    pub last_completed_action: Option<String>,
    #[serde(default)]
    pub transition_refs: Vec<String>,
    pub stored_receipt_ref: Option<String>,
    pub completion_known: bool,
    pub recovery: Option<RecoveryInstructions>,
    pub rollback: RollbackBoundary,
    pub updated_at: DateTime<Utc>,
}

impl PersistedLifecycleState {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.transaction_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyTransactionId);
        }
        if self.scope.host_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyHostId);
        }
        if self.idempotency.key.trim().is_empty()
            || self.idempotency.intent_digest.trim().is_empty()
        {
            return Err(InstallLifecycleValidationError::IdempotencyKeyRequired);
        }
        self.rollback.validate()?;
        if self.current_state.is_recovery() {
            self.recovery
                .as_ref()
                .ok_or(InstallLifecycleValidationError::RecoveryRequiresGuidance)?
                .validate()?;
        }
        if self.progress == TransactionProgress::Complete
            && (!self.completion_known || self.stored_receipt_ref.is_none())
        {
            return Err(InstallLifecycleValidationError::CompletedTransactionRequiresReceipt);
        }
        Ok(())
    }

    pub fn replay(
        &self,
        key: &str,
        intent_digest: &str,
    ) -> Result<ReplayDisposition, InstallLifecycleValidationError> {
        if key != self.idempotency.key || intent_digest != self.idempotency.intent_digest {
            return Err(InstallLifecycleValidationError::IdempotencyConflict);
        }
        if !self.completion_known {
            return Ok(ReplayDisposition::InspectBeforeResume);
        }
        if self.progress == TransactionProgress::Complete {
            return if self.stored_receipt_ref.is_some() {
                Ok(ReplayDisposition::ReturnStoredReceipt)
            } else {
                Err(InstallLifecycleValidationError::CompletedTransactionRequiresReceipt)
            };
        }
        Ok(ReplayDisposition::Resume)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallIntent {
    pub selections: LifecycleSelections,
    pub preflight: PreflightReport,
    pub artifact: ArtifactTrustEvidence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingIntent {
    pub selections: LifecycleSelections,
    pub exact_scope: LifecycleScope,
    pub bootstrap_preview_ref: String,
    pub mutation_confirmation_ref: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceIntent {
    pub action: MaintenanceAction,
    pub selections: LifecycleSelections,
    pub preservation: PreservationDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallTransaction {
    pub intent: HostInstallIntent,
    pub persisted: PersistedLifecycleState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingTransaction {
    pub intent: ProjectOnboardingIntent,
    pub persisted: PersistedLifecycleState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceTransaction {
    pub intent: LifecycleMaintenanceIntent,
    pub persisted: PersistedLifecycleState,
}

impl HostInstallTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::HostInstall {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        if self.persisted.scope.project_root.is_some()
            || self.persisted.scope.continuity_id.is_some()
        {
            return Err(InstallLifecycleValidationError::ProjectScopeForbiddenForHostInstall);
        }
        if self.intent.selections.project != ProjectSelection::Skip {
            return Err(InstallLifecycleValidationError::ProjectSelectionForbiddenForHostInstall);
        }
        self.intent.selections.validate()?;
        self.intent.preflight.validate()?;
        if self.intent.preflight.host_id != self.persisted.scope.host_id {
            return Err(InstallLifecycleValidationError::ScopeMismatch);
        }
        if self.intent.artifact.target != self.intent.selections.target
            || self.intent.artifact.declared_channel != self.intent.selections.channel
            || self.intent.preflight.supported_target.as_deref()
                != Some(self.intent.selections.target.as_str())
        {
            return Err(InstallLifecycleValidationError::ArtifactSelectionMismatch);
        }
        self.intent.artifact.validate()?;
        Ok(())
    }
}
impl ProjectOnboardingTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::ProjectOnboarding {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        self.intent.selections.validate()?;
        if self.persisted.scope != self.intent.exact_scope
            || self.intent.exact_scope.project_root.is_none()
            || self.intent.exact_scope.continuity_id.is_none()
        {
            return Err(InstallLifecycleValidationError::ProjectScopeRequiredForOnboarding);
        }
        let selected_path = self.intent.selections.project.explicit_path();
        if selected_path != self.intent.exact_scope.project_root.as_deref() {
            return Err(InstallLifecycleValidationError::ScopeMismatch);
        }
        if self.intent.bootstrap_preview_ref.trim().is_empty() {
            return Err(InstallLifecycleValidationError::BootstrapPreviewRequired);
        }
        if self.persisted.current_state == LifecycleState::ProjectBootstrapped
            && self.intent.mutation_confirmation_ref.is_none()
        {
            return Err(InstallLifecycleValidationError::MutationConfirmationRequired);
        }
        Ok(())
    }
}
impl LifecycleMaintenanceTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::LifecycleMaintenance {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        self.intent.selections.validate()?;
        self.intent.preservation.validate()?;
        if self.intent.action != self.intent.preservation.action {
            return Err(InstallLifecycleValidationError::MaintenanceActionMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealthEvidence {
    pub required: bool,
    pub healthy: bool,
    pub posture: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformEvidence {
    pub target: String,
    pub mechanism: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationOutcome {
    pub integration: IntegrationSelection,
    pub status: String,
    pub required: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionProof {
    pub artifact_trust: ArtifactTrustEvidence,
    pub version_set: Vec<ComponentVersion>,
    pub daemon_healthy: bool,
    #[serde(default)]
    pub daemon_health_refs: Vec<String>,
    pub service: ServiceHealthEvidence,
    pub expected_scope: LifecycleScope,
    pub observed_scope: LifecycleScope,
    #[serde(default)]
    pub scope_evidence_refs: Vec<String>,
    pub mutation_required_confirmation: bool,
    pub mutation_confirmation_ref: Option<String>,
    pub workpoint_required: bool,
    pub workpoint_ref: Option<String>,
    pub secret_values_detected: bool,
    #[serde(default)]
    pub secret_handling_refs: Vec<String>,
    pub rollback: RollbackBoundary,
    pub preservation: PreservationDeclaration,
    #[serde(default)]
    pub platform_evidence: Vec<PlatformEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalseCompletionReason {
    ArtifactTrustAbsent,
    VersionSetIncompatible,
    DaemonUnhealthy,
    RequiredServiceUnhealthy,
    ExactScopeUnproven,
    MutationConfirmationMissing,
    WorkpointMissing,
    SecretLeakDetected,
    PreservationAmbiguous,
    RollbackUnavailable,
    PlatformEvidenceMissing,
    ReceiptPayloadIncomplete,
}

impl CompletionProof {
    pub fn false_completion_reasons(&self) -> Vec<FalseCompletionReason> {
        let mut reasons = Vec::new();
        if !self.artifact_trust.is_complete() {
            reasons.push(FalseCompletionReason::ArtifactTrustAbsent);
        }
        if self.version_set.is_empty()
            || self
                .version_set
                .iter()
                .any(|item| !item.compatible || item.evidence_refs.is_empty())
        {
            reasons.push(FalseCompletionReason::VersionSetIncompatible);
        }
        if !self.daemon_healthy || self.daemon_health_refs.is_empty() {
            reasons.push(FalseCompletionReason::DaemonUnhealthy);
        }
        if self.service.required && (!self.service.healthy || self.service.evidence_refs.is_empty())
        {
            reasons.push(FalseCompletionReason::RequiredServiceUnhealthy);
        }
        if self.expected_scope != self.observed_scope || self.scope_evidence_refs.is_empty() {
            reasons.push(FalseCompletionReason::ExactScopeUnproven);
        }
        if self.mutation_required_confirmation
            && self
                .mutation_confirmation_ref
                .as_deref()
                .is_none_or(str::is_empty)
        {
            reasons.push(FalseCompletionReason::MutationConfirmationMissing);
        }
        if self.workpoint_required && self.workpoint_ref.as_deref().is_none_or(str::is_empty) {
            reasons.push(FalseCompletionReason::WorkpointMissing);
        }
        if self.secret_values_detected || self.secret_handling_refs.is_empty() {
            reasons.push(FalseCompletionReason::SecretLeakDetected);
        }
        if self.preservation.validate().is_err() {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if !self.rollback.rollback_available() {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        if self.platform_evidence.is_empty()
            || self.platform_evidence.iter().any(|item| {
                item.target.trim().is_empty()
                    || item.mechanism.trim().is_empty()
                    || item.evidence_refs.is_empty()
            })
        {
            reasons.push(FalseCompletionReason::PlatformEvidenceMissing);
        }
        reasons
    }

    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let reasons = self.false_completion_reasons();
        if reasons.is_empty() {
            Ok(())
        } else {
            Err(reasons)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallReceiptPayload {
    pub target: String,
    pub version_set: Vec<ComponentVersion>,
    pub artifact_trust_refs: Vec<String>,
    pub daemon_health_refs: Vec<String>,
    pub service_posture: String,
    pub integration_outcomes: Vec<IntegrationOutcome>,
    pub preservation: PreservationDeclaration,
    pub recovery: Option<RecoveryInstructions>,
    pub rollback: RollbackBoundary,
    pub update_action: Option<MaintenanceAction>,
    pub uninstall_action: Option<MaintenanceAction>,
    pub completion: CompletionProof,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingReceiptPayload {
    pub exact_scope: LifecycleScope,
    pub bootstrap_refs: Vec<String>,
    pub git: GitSelection,
    pub task_provider: TaskProviderSelection,
    pub instructions: InstructionSelection,
    pub genesis_status: String,
    pub hlt_status: String,
    pub workpoint_ref: String,
    pub canvas: CanvasSelection,
    pub deferred_optional_work: Vec<String>,
    pub completion: CompletionProof,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceReceiptPayload {
    pub action: MaintenanceAction,
    pub preservation: PreservationDeclaration,
    pub rollback: RollbackBoundary,
    pub recovery: Option<RecoveryInstructions>,
    pub completion: CompletionProof,
}

fn finish_receipt_validation(
    mut reasons: Vec<FalseCompletionReason>,
) -> Result<(), Vec<FalseCompletionReason>> {
    reasons.sort_by_key(|reason| *reason as u8);
    reasons.dedup();
    if reasons.is_empty() {
        Ok(())
    } else {
        Err(reasons)
    }
}

impl HostInstallReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.target.trim().is_empty()
            || self.target != self.completion.artifact_trust.target
            || self.artifact_trust_refs.is_empty()
        {
            reasons.push(FalseCompletionReason::ArtifactTrustAbsent);
        }
        if self.version_set.is_empty() || self.version_set != self.completion.version_set {
            reasons.push(FalseCompletionReason::VersionSetIncompatible);
        }
        if self.daemon_health_refs.is_empty()
            || self.daemon_health_refs != self.completion.daemon_health_refs
        {
            reasons.push(FalseCompletionReason::DaemonUnhealthy);
        }
        if self.service_posture.trim().is_empty() {
            reasons.push(FalseCompletionReason::RequiredServiceUnhealthy);
        }
        if self.preservation != self.completion.preservation {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if self.rollback != self.completion.rollback {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        finish_receipt_validation(reasons)
    }
}
impl ProjectOnboardingReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.exact_scope != self.completion.expected_scope
            || self.exact_scope.project_root.is_none()
            || self.exact_scope.continuity_id.is_none()
        {
            reasons.push(FalseCompletionReason::ExactScopeUnproven);
        }
        if self.workpoint_ref.trim().is_empty()
            || self.completion.workpoint_ref.as_deref() != Some(self.workpoint_ref.as_str())
        {
            reasons.push(FalseCompletionReason::WorkpointMissing);
        }
        if self.bootstrap_refs.is_empty()
            || self.genesis_status.trim().is_empty()
            || self.hlt_status.trim().is_empty()
        {
            reasons.push(FalseCompletionReason::ReceiptPayloadIncomplete);
        }
        finish_receipt_validation(reasons)
    }
}
impl LifecycleMaintenanceReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.preservation.validate().is_err()
            || self.preservation != self.completion.preservation
        {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if !self.rollback.rollback_available() || self.rollback != self.completion.rollback {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        if self.action != self.preservation.action {
            reasons.push(FalseCompletionReason::ReceiptPayloadIncomplete);
        }
        finish_receipt_validation(reasons)
    }
}
