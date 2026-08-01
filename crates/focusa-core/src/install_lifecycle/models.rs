use serde::{Deserialize, Serialize};

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
