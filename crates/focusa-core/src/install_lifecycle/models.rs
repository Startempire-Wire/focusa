use chrono::{DateTime, Utc};
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
    FirstMissionEntitlementRequired,
    FirstMissionLimitReservationInvalid,
    ScopeMismatch,
    BootstrapPreviewRequired,
    MutationConfirmationRequired,
    MaintenanceActionMismatch,
    ArtifactSelectionMismatch,
    ArtifactTrustIncomplete,
    EntitlementBindingIncomplete,
    EntitlementReceiptClassMismatch,
    AdapterEntitlementPostureIncomplete,
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

fn lifecycle_entitlement_schema_v1() -> String {
    "focusa.lifecycle_entitlement_binding.v1".into()
}

fn adapter_entitlement_schema_v1() -> String {
    "focusa.adapter_entitlement_posture.v1".into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEntitlementState {
    Unactivated,
    PendingIdentity,
    PendingDeviceCode,
    ActiveEvaluation,
    ActivePaid,
    OfflineGrace,
    Expired,
    Revoked,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEntitlementReceiptClass {
    RecoveryReady,
    EvaluationReady,
    PaidReady,
    DevelopmentReady,
    BlockedEntitlement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleEntitlementBinding {
    #[serde(default = "lifecycle_entitlement_schema_v1")]
    pub schema_version: String,
    pub state: LifecycleEntitlementState,
    pub lease_id: String,
    pub lease_sequence: u64,
    pub lease_payload_digest: String,
    pub product_grants_digest: String,
    pub feature_grants_digest: String,
    pub node_id: String,
    pub license_class: String,
    pub refresh_after: DateTime<Utc>,
    pub offline_valid_until: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authority_key_id: String,
    pub signature_verified: bool,
}

impl LifecycleEntitlementBinding {
    pub fn receipt_class(&self) -> LifecycleEntitlementReceiptClass {
        match self.state {
            LifecycleEntitlementState::ActiveEvaluation => {
                LifecycleEntitlementReceiptClass::EvaluationReady
            }
            LifecycleEntitlementState::ActivePaid => LifecycleEntitlementReceiptClass::PaidReady,
            LifecycleEntitlementState::OfflineGrace => {
                if self.license_class == "authorized_development" {
                    LifecycleEntitlementReceiptClass::DevelopmentReady
                } else {
                    LifecycleEntitlementReceiptClass::RecoveryReady
                }
            }
            LifecycleEntitlementState::Unactivated
            | LifecycleEntitlementState::PendingIdentity
            | LifecycleEntitlementState::PendingDeviceCode
            | LifecycleEntitlementState::Expired
            | LifecycleEntitlementState::Revoked
            | LifecycleEntitlementState::Invalid => {
                LifecycleEntitlementReceiptClass::BlockedEntitlement
            }
        }
    }

    pub fn allows_product_execution_at(&self, now: DateTime<Utc>) -> bool {
        if self.validate().is_err() {
            return false;
        }
        match self.state {
            LifecycleEntitlementState::ActiveEvaluation | LifecycleEntitlementState::ActivePaid => {
                self.expires_at.is_some_and(|expires_at| now < expires_at)
            }
            LifecycleEntitlementState::OfflineGrace => now < self.offline_valid_until,
            LifecycleEntitlementState::Unactivated
            | LifecycleEntitlementState::PendingIdentity
            | LifecycleEntitlementState::PendingDeviceCode
            | LifecycleEntitlementState::Expired
            | LifecycleEntitlementState::Revoked
            | LifecycleEntitlementState::Invalid => false,
        }
    }

    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        let required_text = [
            &self.schema_version,
            &self.lease_id,
            &self.lease_payload_digest,
            &self.product_grants_digest,
            &self.feature_grants_digest,
            &self.node_id,
            &self.license_class,
            &self.authority_key_id,
        ];
        let digests = [
            &self.lease_payload_digest,
            &self.product_grants_digest,
            &self.feature_grants_digest,
        ];
        if required_text.iter().any(|value| value.trim().is_empty())
            || digests
                .iter()
                .any(|value| !value.starts_with("sha256:") || value.len() != 71)
            || self.lease_sequence == 0
            || !self.signature_verified
        {
            return Err(InstallLifecycleValidationError::EntitlementBindingIncomplete);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleEntitlementDecision {
    pub binding: LifecycleEntitlementBinding,
    #[serde(default)]
    pub granted_products: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub granted_features: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub remaining_limits: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl LifecycleEntitlementDecision {
    pub fn grants(
        &self,
        product: &str,
        required_features: &std::collections::BTreeSet<String>,
        now: DateTime<Utc>,
    ) -> bool {
        self.binding.allows_product_execution_at(now)
            && self.granted_products.contains(product)
            && required_features.is_subset(&self.granted_features)
            && !self.evidence_refs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterEntitlementPosture {
    #[serde(default = "adapter_entitlement_schema_v1")]
    pub schema_version: String,
    pub product: String,
    pub lease_id: String,
    pub lease_sequence: u64,
    pub product_granted: bool,
    pub required_features_granted: bool,
    pub child_token_audience: Option<String>,
    pub child_token_expires_at: Option<DateTime<Utc>>,
    pub entitlement_digest: String,
}

impl AdapterEntitlementPosture {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.schema_version.trim().is_empty()
            || self.product.trim().is_empty()
            || self.lease_id.trim().is_empty()
            || self.lease_sequence == 0
            || !self.entitlement_digest.starts_with("sha256:")
            || self.entitlement_digest.len() != 71
            || self
                .child_token_audience
                .as_ref()
                .is_some_and(|audience| audience.trim().is_empty())
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        Ok(())
    }

    pub fn is_entitled(&self) -> bool {
        self.product_granted && self.required_features_granted
    }
}
