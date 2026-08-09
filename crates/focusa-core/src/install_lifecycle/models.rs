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
    LimitedAccess,
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

fn entitlement_snapshot_ready(
    snapshot: &focusa_license::authority::EntitlementSnapshot,
    now: DateTime<Utc>,
) -> bool {
    match snapshot.state {
        focusa_license::authority::EntitlementState::Active => {
            snapshot.expires_at.is_some_and(|expiry| expiry > now)
        }
        focusa_license::authority::EntitlementState::OfflineGrace => snapshot
            .offline_grace_until
            .is_some_and(|expiry| expiry > now),
        focusa_license::authority::EntitlementState::Unactivated
        | focusa_license::authority::EntitlementState::RecoveryOnly => false,
    }
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
    ActiveVerifiedLimited,
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
    LimitedAccessReady,
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
            LifecycleEntitlementState::ActiveVerifiedLimited => {
                LifecycleEntitlementReceiptClass::LimitedAccessReady
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
            LifecycleEntitlementState::ActiveVerifiedLimited
            | LifecycleEntitlementState::ActivePaid => self
                .expires_at
                .is_some_and(|expires_at| now < expires_at),
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
    #[serde(default)]
    pub parent_lease_digest: String,
    #[serde(default)]
    pub child_token_id: String,
    pub child_token_audience: Option<String>,
    pub child_token_expires_at: Option<DateTime<Utc>>,
    pub entitlement_digest: String,
    /// Single verified EDD account the UIAI activation is bound to (Spec 152E
    /// §7/§15). Present only on same-account UIAI postures; the installer
    /// never creates a second customer identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edd_customer_id: Option<u64>,
}

impl AdapterEntitlementPosture {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.schema_version.trim().is_empty()
            || self.product.trim().is_empty()
            || self.lease_id.trim().is_empty()
            || self.lease_sequence == 0
            || !self.parent_lease_digest.starts_with("sha256:")
            || self.parent_lease_digest.len() != 71
            || self.child_token_id.trim().is_empty()
            || !self.entitlement_digest.starts_with("sha256:")
            || self.entitlement_digest.len() != 71
            || self
                .child_token_audience
                .as_ref()
                .is_some_and(|audience| audience.trim().is_empty())
            || (self.is_entitled()
                && (self.child_token_audience.is_none() || self.child_token_expires_at.is_none()))
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        Ok(())
    }

    pub fn is_entitled(&self) -> bool {
        self.product_granted && self.required_features_granted
    }

    pub fn from_independent_uiai_authority(
        focusa_parent: &focusa_license::authority::EntitlementSnapshot,
        uiai_grant: &focusa_license::authority::EntitlementSnapshot,
        request: &focusa_license::uiai_child_token::UiaiChildTokenRequest,
        receipt: &focusa_license::uiai_child_token::UiaiChildTokenReceipt,
        now: DateTime<Utc>,
    ) -> Result<Self, InstallLifecycleValidationError> {
        let focusa_ready = entitlement_snapshot_ready(focusa_parent, now)
            && focusa_parent.product == "focusa"
            && focusa_parent.node_id == request.node_id
            && focusa_parent.lease_id.as_deref() == Some(request.parent_lease_id.as_str())
            && focusa_parent.sequence == Some(request.parent_lease_sequence)
            && focusa_parent.lease_digest.as_deref() == Some(request.parent_lease_digest.as_str());
        let uiai_ready = entitlement_snapshot_ready(uiai_grant, now)
            && uiai_grant.product == "uiai-engine"
            && uiai_grant.node_id == request.node_id
            && uiai_grant.lease_id.as_deref() == Some(request.uiai_grant_lease_id.as_str())
            && uiai_grant.sequence == Some(request.uiai_grant_sequence)
            && request
                .requested_features
                .iter()
                .all(|feature| uiai_grant.features.get(feature).copied().unwrap_or(false));
        let child_ready = receipt.request_id == request.request_id
            && receipt.parent_lease_sequence == request.parent_lease_sequence
            && receipt.uiai_grant_sequence == request.uiai_grant_sequence
            && receipt.feature_count == request.requested_features.len()
            && receipt.limit_count == request.requested_limits.len()
            && receipt.expires_at > now;
        if !focusa_ready || !uiai_ready || !child_ready {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        let posture = Self {
            schema_version: adapter_entitlement_schema_v1(),
            product: "uiai-engine".into(),
            lease_id: request.uiai_grant_lease_id.clone(),
            lease_sequence: request.uiai_grant_sequence,
            product_granted: true,
            required_features_granted: true,
            parent_lease_digest: request.parent_lease_digest.clone(),
            child_token_id: receipt.token_id.clone(),
            child_token_audience: Some(receipt.audience.clone()),
            child_token_expires_at: Some(receipt.expires_at),
            entitlement_digest: uiai_grant.lease_digest.clone().unwrap_or_default(),
            account_id: None,
            edd_customer_id: None,
        };
        posture.validate()?;
        Ok(posture)
    }

    /// Same-EDD-account UIAI activation (Spec 152E §7, §8, §15, §21, §23
    /// "UIAI purchase"): the UIAI adapter posture is built only when the
    /// Focusa parent and the independent UIAI grant are issued to the SAME
    /// verified EDD account (no duplicate customer identity), the requested
    /// scope is an exact subset of the independent `uiai-engine` grant, and
    /// the authority child-token receipt settles the same registration. The
    /// posture carries the single account identity; product isolation is
    /// preserved because grants come only from the UIAI projection.
    #[allow(clippy::too_many_arguments)]
    pub fn from_same_edd_account_uiai_authority(
        account: &focusa_license::uiai_activation::UiaiAccountIdentity,
        projection: &focusa_license::uiai_activation::UiaiGrantProjection,
        focusa_parent: &focusa_license::authority::EntitlementSnapshot,
        uiai_grant: &focusa_license::authority::EntitlementSnapshot,
        request: &focusa_license::uiai_child_token::UiaiChildTokenRequest,
        receipt: &focusa_license::uiai_child_token::UiaiChildTokenReceipt,
        now: DateTime<Utc>,
    ) -> Result<Self, InstallLifecycleValidationError> {
        // One verified EDD account, exactly: no duplicate customer identity.
        if !account.valid()
            || !focusa_license::uiai_activation::same_account_binding(
                account,
                focusa_parent,
                uiai_grant,
            )
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        // Independent UIAI grant: exact grants only from the UIAI projection.
        if projection.product != "uiai-engine"
            || projection.account != *account
            || projection.node_id != uiai_grant.node_id
            || projection.grant_lease_id != request.uiai_grant_lease_id
            || projection.grant_sequence != request.uiai_grant_sequence
            || !entitlement_snapshot_ready(uiai_grant, now)
            || uiai_grant.product != "uiai-engine"
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        // The same registration settled the child token for the UIAI lease.
        let child_ready = receipt.request_id == request.request_id
            && receipt.parent_lease_sequence == request.parent_lease_sequence
            && receipt.uiai_grant_sequence == request.uiai_grant_sequence
            && receipt.feature_count == projection.features.len()
            && receipt.limit_count == projection.limits.len()
            && receipt.expires_at > now;
        if !child_ready {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        let posture = Self {
            schema_version: adapter_entitlement_schema_v1(),
            product: projection.product.clone(),
            lease_id: projection.grant_lease_id.clone(),
            lease_sequence: projection.grant_sequence,
            product_granted: true,
            required_features_granted: true,
            parent_lease_digest: request.parent_lease_digest.clone(),
            child_token_id: receipt.token_id.clone(),
            child_token_audience: Some(receipt.audience.clone()),
            child_token_expires_at: Some(receipt.expires_at),
            entitlement_digest: uiai_grant.lease_digest.clone().unwrap_or_default(),
            account_id: Some(account.account_id.clone()),
            edd_customer_id: Some(account.edd_customer_id),
        };
        posture.validate()?;
        Ok(posture)
    }
}

fn bundle_adapter_schema_v1() -> String {
    "focusa.bundle_adapter_posture.v1".into()
}

/// Bundle adapter posture (Spec 172 §9.2; Spec 152E §23 "Bundle purchase"):
/// one verified account, one EDD order, one canonical human key. The bundle
/// projection carries explicit Focusa and UIAI product grants on the SAME
/// shared operator node identities (three shared nodes — never six unrelated
/// activations). The posture exists only when both exact grants are active
/// and bound to the one verified EDD account; the bundle decision returns a
/// typed recoverable partial state instead of a silent half-grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleAdapterPosture {
    #[serde(default = "bundle_adapter_schema_v1")]
    pub schema_version: String,
    pub public_code: String,
    pub account_id: String,
    pub edd_customer_id: u64,
    pub order_handle: String,
    pub node_id: String,
    pub shared_node_identities: Vec<String>,
    pub focusa_lease_id: String,
    pub focusa_lease_sequence: u64,
    pub focusa_lease_digest: String,
    pub uiai_lease_id: String,
    pub uiai_lease_sequence: u64,
    pub uiai_lease_digest: String,
    pub both_grants_active: bool,
}

impl BundleAdapterPosture {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        let digests = [&self.focusa_lease_digest, &self.uiai_lease_digest];
        if self.schema_version.trim().is_empty()
            || self.public_code.trim().is_empty()
            || self.account_id.trim().is_empty()
            || self.edd_customer_id == 0
            || self.order_handle.trim().is_empty()
            || self.node_id.trim().is_empty()
            || self.shared_node_identities.is_empty()
            || !self
                .shared_node_identities
                .iter()
                .all(|identity| identity == &self.node_id)
            || self.focusa_lease_id.trim().is_empty()
            || self.focusa_lease_sequence == 0
            || self.uiai_lease_id.trim().is_empty()
            || self.uiai_lease_sequence == 0
            || digests
                .iter()
                .any(|value| !value.starts_with("sha256:") || value.len() != 71)
            || !self.both_grants_active
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        Ok(())
    }

    /// Build the bundle adapter posture from the bundle authority projection
    /// (Spec 172 §9.2): both exact grants are active and bound to the SAME
    /// verified EDD account and the SAME shared operator node; the signed
    /// lease pair carries explicit Focusa and UIAI product grants; the
    /// projection is the only source of grants (no third feature list).
    pub fn from_bundle_authority(
        account: &focusa_license::uiai_activation::UiaiAccountIdentity,
        projection: &focusa_license::bundle_activation::BundleActivationProjection,
        focusa_grant: &focusa_license::authority::EntitlementSnapshot,
        uiai_grant: &focusa_license::authority::EntitlementSnapshot,
        now: DateTime<Utc>,
    ) -> Result<Self, InstallLifecycleValidationError> {
        // One verified EDD account, exactly: no duplicate customer identity.
        if !account.valid()
            || !focusa_license::uiai_activation::same_account_binding(
                account,
                focusa_grant,
                uiai_grant,
            )
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        // Exact two-product grants from the bundle projection; the shared
        // operator node identity binds both products (never six unrelated
        // activations).
        if projection.account != *account
            || projection.focusa.product != "focusa"
            || projection.uiai_engine.product != "uiai-engine"
            || projection.focusa.grant_lease_id
                != focusa_grant.lease_id.as_deref().unwrap_or_default()
            || projection.uiai_engine.grant_lease_id
                != uiai_grant.lease_id.as_deref().unwrap_or_default()
            || projection.focusa.node_id != projection.node_id
            || projection.uiai_engine.node_id != projection.node_id
            || !entitlement_snapshot_ready(focusa_grant, now)
            || !entitlement_snapshot_ready(uiai_grant, now)
        {
            return Err(InstallLifecycleValidationError::AdapterEntitlementPostureIncomplete);
        }
        let posture = Self {
            schema_version: bundle_adapter_schema_v1(),
            public_code: projection.public_code.clone(),
            account_id: account.account_id.clone(),
            edd_customer_id: account.edd_customer_id,
            order_handle: projection.order_handle.clone(),
            node_id: projection.node_id.clone(),
            shared_node_identities: projection.shared_node_identities.clone(),
            focusa_lease_id: projection.focusa.grant_lease_id.clone(),
            focusa_lease_sequence: projection.focusa.grant_sequence,
            focusa_lease_digest: focusa_grant.lease_digest.clone().unwrap_or_default(),
            uiai_lease_id: projection.uiai_engine.grant_lease_id.clone(),
            uiai_lease_sequence: projection.uiai_engine.grant_sequence,
            uiai_lease_digest: uiai_grant.lease_digest.clone().unwrap_or_default(),
            both_grants_active: true,
        };
        posture.validate()?;
        Ok(posture)
    }
}
