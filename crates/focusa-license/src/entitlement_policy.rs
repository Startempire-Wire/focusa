use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use thiserror::Error;

#[path = "entitlement_policy_registry_validation.rs"]
mod registry_validation;

include!(concat!(env!("OUT_DIR"), "/entitlement_policy_registry.rs"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationClass {
    Read,
    ValueMutation,
    Recovery,
    InternalMaintenance,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityFamily {
    AccountRecovery,
    ReadProjection,
    BaseFocusa,
    Automation,
    TeamRemote,
    ReleaseProof,
    PremiumUpdates,
    CustomerDataExport,
    InternalMaintenance,
}

impl CapabilityFamily {
    pub fn commercial_treatment(self) -> CommercialTreatment {
        match self {
            Self::AccountRecovery => CommercialTreatment::AlwaysAvailable,
            Self::ReadProjection => CommercialTreatment::ReadAllowance,
            Self::BaseFocusa => CommercialTreatment::BaseEntitlement,
            Self::Automation | Self::TeamRemote | Self::ReleaseProof | Self::PremiumUpdates => {
                CommercialTreatment::OptionalPremium
            }
            Self::CustomerDataExport => {
                CommercialTreatment::AlwaysAvailableBasicWithOptionalPremiumPackaging
            }
            Self::InternalMaintenance => CommercialTreatment::InheritInitiatingOperation,
        }
    }

    pub fn is_optional_premium(self) -> bool {
        matches!(
            self,
            Self::Automation | Self::TeamRemote | Self::ReleaseProof | Self::PremiumUpdates
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialTreatment {
    AlwaysAvailable,
    ReadAllowance,
    BaseEntitlement,
    OptionalPremium,
    AlwaysAvailableBasicWithOptionalPremiumPackaging,
    InheritInitiatingOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyActivation {
    Active,
    Dormant,
    ActiveOnlyWhenDeclared,
    DormantForCommerce,
    ActiveForPreviewNightlyAndUnattended,
}

impl PolicyActivation {
    pub fn permits_runtime_commercial_decision(self) -> bool {
        matches!(
            self,
            Self::Active
                | Self::ActiveOnlyWhenDeclared
                | Self::ActiveForPreviewNightlyAndUnattended
        )
    }
}

/// Closed Spec 172 product registry. Deserialization is intentionally closed so
/// callers cannot manufacture commercial products.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductCode {
    Focusa,
    UiaiEngine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTypeCode {
    FocusaOperatorLifetimeV1,
    UiaiOperatorLifetimeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTypeVersion {
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleStatus {
    ApprovedNotYetEnabled,
}

/// Runtime postures are not legacy tiers. In particular, Evaluation has no
/// active-policy spelling and therefore fails serde parsing as an unknown value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessPosture {
    Unverified,
    VerifiedNoLicense,
    ActivePaidOperator,
    OfflineGrace,
    RecoveryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceRight {
    LocalIncluded,
    HostedExcluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorSeats {
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedNodeLimit {
    OperatorSharedV1Three,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseTypeGrant {
    pub product: ProductCode,
    pub license_type: LicenseTypeCode,
    pub version: LicenseTypeVersion,
    pub sale_status: SaleStatus,
    pub operator_seats: OperatorSeats,
    pub node_limit: SharedNodeLimit,
    pub local_resource: ResourceRight,
    pub hosted_resource: ResourceRight,
}

impl LicenseTypeGrant {
    pub const fn focusa_operator_v1() -> Self {
        Self::operator(
            ProductCode::Focusa,
            LicenseTypeCode::FocusaOperatorLifetimeV1,
        )
    }

    pub const fn uiai_operator_v1() -> Self {
        Self::operator(
            ProductCode::UiaiEngine,
            LicenseTypeCode::UiaiOperatorLifetimeV1,
        )
    }

    const fn operator(product: ProductCode, license_type: LicenseTypeCode) -> Self {
        Self {
            product,
            license_type,
            version: LicenseTypeVersion::V1,
            sale_status: SaleStatus::ApprovedNotYetEnabled,
            operator_seats: OperatorSeats::One,
            node_limit: SharedNodeLimit::OperatorSharedV1Three,
            local_resource: ResourceRight::LocalIncluded,
            hosted_resource: ResourceRight::HostedExcluded,
        }
    }

    pub fn validate(&self) -> Result<(), EntitlementPolicyTypeError> {
        let expected = match self.license_type {
            LicenseTypeCode::FocusaOperatorLifetimeV1 => Self::focusa_operator_v1(),
            LicenseTypeCode::UiaiOperatorLifetimeV1 => Self::uiai_operator_v1(),
        };
        if *self == expected {
            Ok(())
        } else {
            Err(EntitlementPolicyTypeError::InvalidLicenseTypeGrant)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeGrant {
    grants: [LicenseTypeGrant; 2],
}

impl CompositeGrant {
    pub fn operator_bundle_v1(
        grants: [LicenseTypeGrant; 2],
    ) -> Result<Self, EntitlementPolicyTypeError> {
        let expected = [
            LicenseTypeGrant::focusa_operator_v1(),
            LicenseTypeGrant::uiai_operator_v1(),
        ];
        if grants != expected {
            return Err(EntitlementPolicyTypeError::MalformedBundleUnion);
        }
        Ok(Self { grants })
    }

    pub fn grants(&self) -> &[LicenseTypeGrant; 2] {
        &self.grants
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEntitlementState {
    PendingUnverified,
    VerifiedNoLicense,
    ActivePaid,
    OfflineGrace,
    Expired,
    RefundedOrRevoked,
    MissingOrCorrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RequiredFeature(String);

impl RequiredFeature {
    pub fn new(value: impl Into<String>) -> Result<Self, EntitlementPolicyTypeError> {
        let value = value.into();
        if is_qualified_identifier(&value, "focusa.") {
            Ok(Self(value))
        } else {
            Err(EntitlementPolicyTypeError::InvalidRequiredFeature)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RequiredFeature {
    type Error = EntitlementPolicyTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RequiredFeature> for String {
    fn from(value: RequiredFeature) -> Self {
        value.0
    }
}

impl AsRef<str> for RequiredFeature {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LimitBucket(String);

impl LimitBucket {
    pub fn new(value: impl Into<String>) -> Result<Self, EntitlementPolicyTypeError> {
        let value = value.into();
        if is_identifier(&value) {
            Ok(Self(value))
        } else {
            Err(EntitlementPolicyTypeError::InvalidLimitBucket)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LimitBucket {
    type Error = EntitlementPolicyTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<LimitBucket> for String {
    fn from(value: LimitBucket) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityPrerequisite {
    IdentityVerification,
    RolePermission,
    ScopeBinding,
    DeviceBinding,
    OperatorConfirmation,
    PlatformPermission,
    ArtifactSignature,
    TrustMetadata,
    PrivacyRedaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAllowance {
    None,
    AccountRecovery,
    ReadProjection,
    CustomerDataExport,
    StableSecurityUpdate,
    RepairRollback,
    Uninstall,
}

const ACCOUNT_RECOVERY_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::RolePermission,
    SecurityPrerequisite::ScopeBinding,
    SecurityPrerequisite::DeviceBinding,
    SecurityPrerequisite::OperatorConfirmation,
];

const READ_PROJECTION_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::ScopeBinding,
];

const CUSTOMER_DATA_EXPORT_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::ScopeBinding,
    SecurityPrerequisite::PrivacyRedaction,
];

const STABLE_SECURITY_UPDATE_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::RolePermission,
    SecurityPrerequisite::ScopeBinding,
    SecurityPrerequisite::DeviceBinding,
    SecurityPrerequisite::OperatorConfirmation,
    SecurityPrerequisite::PlatformPermission,
    SecurityPrerequisite::ArtifactSignature,
    SecurityPrerequisite::TrustMetadata,
];

const REPAIR_ROLLBACK_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::RolePermission,
    SecurityPrerequisite::ScopeBinding,
    SecurityPrerequisite::DeviceBinding,
    SecurityPrerequisite::OperatorConfirmation,
    SecurityPrerequisite::PlatformPermission,
    SecurityPrerequisite::ArtifactSignature,
    SecurityPrerequisite::TrustMetadata,
];

const UNINSTALL_SECURITY_PREREQUISITES: &[SecurityPrerequisite] = &[
    SecurityPrerequisite::IdentityVerification,
    SecurityPrerequisite::RolePermission,
    SecurityPrerequisite::ScopeBinding,
    SecurityPrerequisite::OperatorConfirmation,
];

impl RecoveryAllowance {
    pub const fn security_prerequisites(self) -> &'static [SecurityPrerequisite] {
        match self {
            Self::None => &[],
            Self::AccountRecovery => ACCOUNT_RECOVERY_SECURITY_PREREQUISITES,
            Self::ReadProjection => READ_PROJECTION_SECURITY_PREREQUISITES,
            Self::CustomerDataExport => CUSTOMER_DATA_EXPORT_SECURITY_PREREQUISITES,
            Self::StableSecurityUpdate => STABLE_SECURITY_UPDATE_SECURITY_PREREQUISITES,
            Self::RepairRollback => REPAIR_ROLLBACK_SECURITY_PREREQUISITES,
            Self::Uninstall => UNINSTALL_SECURITY_PREREQUISITES,
        }
    }

    /// Recovery allowance variants resolve to a capability family; no caller input is
    /// consulted here, and `None` is explicitly non-operational.
    pub const fn implied_family(self) -> Option<CapabilityFamily> {
        match self {
            Self::None => None,
            Self::AccountRecovery
            | Self::StableSecurityUpdate
            | Self::RepairRollback
            | Self::Uninstall => Some(CapabilityFamily::AccountRecovery),
            Self::ReadProjection => Some(CapabilityFamily::ReadProjection),
            Self::CustomerDataExport => Some(CapabilityFamily::CustomerDataExport),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReason {
    Allow,
    AllowVerifiedLimited,
    Read,
    ReadLocalOnly,
    AllowExistingLocalOnly,
    AllowOfflineOnly,
    RequireBase,
    RequireFeature,
    RequireCachedFeature,
    RequireCachedFeatureWhenSafe,
    Inherit,
    MissingInitiatingPolicy,
    Deny,
}

/// Commercial posture produced by the state-grid reducer. Security, identity,
/// role, scope, node, sequence, and confirmation gates remain independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementPolicyPosture {
    Allow,
    Read,
    Base,
    Feature,
    Deny,
}

/// Pure, bounded result for one Spec 172-overlaid Spec 152F grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementStateDecision {
    posture: EntitlementPolicyPosture,
    reason: DecisionReason,
}

impl EntitlementStateDecision {
    const fn new(posture: EntitlementPolicyPosture, reason: DecisionReason) -> Self {
        Self { posture, reason }
    }

    pub const fn posture(self) -> EntitlementPolicyPosture {
        self.posture
    }

    pub const fn reason(self) -> DecisionReason {
        self.reason
    }
}

/// Reduce authority state and canonical capability family to commercial
/// posture. `initiating_posture` is considered only for internal maintenance,
/// which may never broaden the operation that caused it.
///
/// This reducer intentionally accepts no product, price, lease, key, grant, or
/// role input. Those values are authority/security concerns resolved before or
/// after this bounded policy step.
pub const fn reduce_entitlement_state(
    state: PolicyEntitlementState,
    family: CapabilityFamily,
    initiating_posture: Option<EntitlementPolicyPosture>,
) -> EntitlementStateDecision {
    use CapabilityFamily as Family;
    use DecisionReason as Reason;
    use EntitlementPolicyPosture as Posture;
    use PolicyEntitlementState as State;

    if matches!(family, Family::InternalMaintenance) {
        return match initiating_posture {
            Some(posture) => EntitlementStateDecision::new(posture, Reason::Inherit),
            None => EntitlementStateDecision::new(Posture::Deny, Reason::MissingInitiatingPolicy),
        };
    }

    match (state, family) {
        (State::PendingUnverified, Family::AccountRecovery) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::Allow)
        }
        (State::PendingUnverified, Family::CustomerDataExport) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::AllowExistingLocalOnly)
        }
        (State::PendingUnverified, _) => EntitlementStateDecision::new(Posture::Deny, Reason::Deny),

        (State::VerifiedNoLicense, Family::AccountRecovery | Family::CustomerDataExport) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::Allow)
        }
        (State::VerifiedNoLicense, Family::ReadProjection) => {
            EntitlementStateDecision::new(Posture::Read, Reason::Read)
        }
        (State::VerifiedNoLicense, Family::BaseFocusa) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::AllowVerifiedLimited)
        }
        (State::VerifiedNoLicense, _) => EntitlementStateDecision::new(Posture::Deny, Reason::Deny),

        (State::ActivePaid, Family::AccountRecovery | Family::CustomerDataExport) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::Allow)
        }
        (State::ActivePaid, Family::ReadProjection) => {
            EntitlementStateDecision::new(Posture::Read, Reason::Read)
        }
        (State::ActivePaid, Family::BaseFocusa) => {
            EntitlementStateDecision::new(Posture::Base, Reason::RequireBase)
        }
        (State::ActivePaid, _) => {
            EntitlementStateDecision::new(Posture::Feature, Reason::RequireFeature)
        }

        (State::OfflineGrace, Family::AccountRecovery) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::AllowOfflineOnly)
        }
        (State::OfflineGrace, Family::CustomerDataExport) => {
            EntitlementStateDecision::new(Posture::Allow, Reason::Allow)
        }
        (State::OfflineGrace, Family::ReadProjection) => {
            EntitlementStateDecision::new(Posture::Read, Reason::Read)
        }
        (State::OfflineGrace, Family::BaseFocusa) => {
            EntitlementStateDecision::new(Posture::Base, Reason::RequireBase)
        }
        (State::OfflineGrace, Family::PremiumUpdates) => {
            EntitlementStateDecision::new(Posture::Feature, Reason::RequireCachedFeatureWhenSafe)
        }
        (State::OfflineGrace, _) => {
            EntitlementStateDecision::new(Posture::Feature, Reason::RequireCachedFeature)
        }

        (
            State::Expired | State::RefundedOrRevoked | State::MissingOrCorrupt,
            Family::AccountRecovery | Family::CustomerDataExport,
        ) => EntitlementStateDecision::new(Posture::Allow, Reason::Allow),
        (State::MissingOrCorrupt, Family::ReadProjection) => {
            EntitlementStateDecision::new(Posture::Read, Reason::ReadLocalOnly)
        }
        (State::Expired | State::RefundedOrRevoked, Family::ReadProjection) => {
            EntitlementStateDecision::new(Posture::Read, Reason::Read)
        }
        (State::Expired | State::RefundedOrRevoked | State::MissingOrCorrupt, _) => {
            EntitlementStateDecision::new(Posture::Deny, Reason::Deny)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedEntitlementPolicy {
    operation_class: OperationClass,
    capability_family: CapabilityFamily,
    commercial_treatment: CommercialTreatment,
    policy_activation: PolicyActivation,
    entitlement_state: PolicyEntitlementState,
    required_feature: Option<RequiredFeature>,
    limit_bucket: Option<LimitBucket>,
    recovery_allowance: RecoveryAllowance,
    decision_reason: DecisionReason,
}

impl ResolvedEntitlementPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        operation_class: OperationClass,
        capability_family: CapabilityFamily,
        commercial_treatment: CommercialTreatment,
        policy_activation: PolicyActivation,
        entitlement_state: PolicyEntitlementState,
        required_feature: Option<RequiredFeature>,
        limit_bucket: Option<LimitBucket>,
        recovery_allowance: RecoveryAllowance,
        decision_reason: DecisionReason,
    ) -> Result<Self, EntitlementPolicyTypeError> {
        if operation_class == OperationClass::Unknown {
            return Err(EntitlementPolicyTypeError::UnknownOperationClass);
        }
        if !policy_activation.permits_runtime_commercial_decision() {
            return Err(EntitlementPolicyTypeError::DormantPolicyActivation);
        }
        if commercial_treatment != capability_family.commercial_treatment() {
            return Err(EntitlementPolicyTypeError::FamilyTreatmentMismatch);
        }
        validate_operation_family(operation_class, capability_family)?;
        validate_feature_and_reason(
            capability_family,
            required_feature.as_ref(),
            decision_reason,
        )?;
        validate_recovery(capability_family, recovery_allowance)?;
        if limit_bucket.is_some()
            && matches!(
                decision_reason,
                DecisionReason::Allow
                    | DecisionReason::AllowVerifiedLimited
                    | DecisionReason::Read
                    | DecisionReason::ReadLocalOnly
                    | DecisionReason::AllowExistingLocalOnly
                    | DecisionReason::AllowOfflineOnly
                    | DecisionReason::Inherit
                    | DecisionReason::Deny
            )
        {
            return Err(EntitlementPolicyTypeError::InactiveLimitBucket);
        }
        Ok(Self {
            operation_class,
            capability_family,
            commercial_treatment,
            policy_activation,
            entitlement_state,
            required_feature,
            limit_bucket,
            recovery_allowance,
            decision_reason,
        })
    }

    pub fn operation_class(&self) -> OperationClass {
        self.operation_class
    }
    pub fn capability_family(&self) -> CapabilityFamily {
        self.capability_family
    }
    pub fn commercial_treatment(&self) -> CommercialTreatment {
        self.commercial_treatment
    }
    pub fn policy_activation(&self) -> PolicyActivation {
        self.policy_activation
    }
    pub fn entitlement_state(&self) -> PolicyEntitlementState {
        self.entitlement_state
    }
    pub fn required_feature(&self) -> Option<&RequiredFeature> {
        self.required_feature.as_ref()
    }
    pub fn limit_bucket(&self) -> Option<&LimitBucket> {
        self.limit_bucket.as_ref()
    }
    pub fn recovery_allowance(&self) -> RecoveryAllowance {
        self.recovery_allowance
    }
    pub fn decision_reason(&self) -> DecisionReason {
        self.decision_reason
    }
}

/// Deterministically compiled operation-policy registry. Its only constructor
/// consumes build-embedded canonical JSON; production has no path-based loader.
#[derive(Debug)]
pub struct EmbeddedEntitlementPolicyRegistry {
    document: Value,
}

impl EmbeddedEntitlementPolicyRegistry {
    pub fn digest(&self) -> &'static str {
        EMBEDDED_POLICY_REGISTRY_DIGEST
    }

    pub fn canonical_json(&self) -> &'static str {
        EMBEDDED_POLICY_REGISTRY_JSON
    }

    pub fn family_count(&self) -> usize {
        self.document["entitlement_policy"]["families"]
            .as_array()
            .map_or(0, Vec::len)
    }

    pub fn license_type_count(&self) -> usize {
        self.document["license_types"]["license_types"]
            .as_array()
            .map_or(0, Vec::len)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("embedded entitlement policy registry is invalid: {message}")]
pub struct EntitlementPolicyRegistryError {
    message: String,
}

/// Load the validated, digest-bound production registry. The registry was also
/// validated by build.rs, and is revalidated once after decoding to guard the
/// generated/runtime boundary.
pub fn embedded_entitlement_policy_registry(
) -> Result<&'static EmbeddedEntitlementPolicyRegistry, EntitlementPolicyRegistryError> {
    static REGISTRY: OnceLock<
        Result<EmbeddedEntitlementPolicyRegistry, EntitlementPolicyRegistryError>,
    > = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let document: Value = serde_json::from_str(EMBEDDED_POLICY_REGISTRY_JSON)
                .map_err(|error| registry_error(error.to_string()))?;
            registry_validation::validate_registry_bundle(&document).map_err(registry_error)?;
            let actual = registry_validation::semantic_digest(&document);
            if actual != EMBEDDED_POLICY_REGISTRY_DIGEST {
                return Err(registry_error("embedded registry digest mismatch"));
            }
            Ok(EmbeddedEntitlementPolicyRegistry { document })
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn registry_error(message: impl Into<String>) -> EntitlementPolicyRegistryError {
    EntitlementPolicyRegistryError {
        message: message.into(),
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum EntitlementPolicyTypeError {
    #[error("active operation policy cannot use an unknown operation class")]
    UnknownOperationClass,
    #[error("dormant policy dimensions cannot make runtime commercial decisions")]
    DormantPolicyActivation,
    #[error("capability family and commercial treatment do not match the registry")]
    FamilyTreatmentMismatch,
    #[error("operation class and capability family are incompatible")]
    OperationFamilyMismatch,
    #[error("required feature is not a qualified Focusa feature identifier")]
    InvalidRequiredFeature,
    #[error("limit bucket is not a stable snake-case identifier")]
    InvalidLimitBucket,
    #[error("required feature and decision reason are incompatible with the capability family")]
    FeatureReasonMismatch,
    #[error("recovery allowance is incompatible with the capability family")]
    RecoveryAllowanceMismatch,
    #[error("limit bucket cannot affect a non-entitled decision")]
    InactiveLimitBucket,
    #[error("license type grant does not exactly match the frozen Spec 172 registry")]
    InvalidLicenseTypeGrant,
    #[error("Bundle must be the ordered exact union of Focusa and UIAI Operator v1 grants")]
    MalformedBundleUnion,
}

fn validate_operation_family(
    operation_class: OperationClass,
    family: CapabilityFamily,
) -> Result<(), EntitlementPolicyTypeError> {
    let valid = match family {
        CapabilityFamily::AccountRecovery => operation_class == OperationClass::Recovery,
        CapabilityFamily::ReadProjection => operation_class == OperationClass::Read,
        CapabilityFamily::InternalMaintenance => {
            operation_class == OperationClass::InternalMaintenance
        }
        CapabilityFamily::CustomerDataExport => {
            matches!(
                operation_class,
                OperationClass::Read | OperationClass::Recovery | OperationClass::ValueMutation
            )
        }
        _ => operation_class == OperationClass::ValueMutation,
    };
    if valid {
        Ok(())
    } else {
        Err(EntitlementPolicyTypeError::OperationFamilyMismatch)
    }
}

fn validate_feature_and_reason(
    family: CapabilityFamily,
    feature: Option<&RequiredFeature>,
    reason: DecisionReason,
) -> Result<(), EntitlementPolicyTypeError> {
    let premium_reason = matches!(
        reason,
        DecisionReason::RequireFeature
            | DecisionReason::RequireCachedFeature
            | DecisionReason::RequireCachedFeatureWhenSafe
    );
    let valid = if family.is_optional_premium() {
        feature.is_some() == premium_reason && !matches!(reason, DecisionReason::RequireBase)
    } else if family == CapabilityFamily::CustomerDataExport {
        feature.is_some() == premium_reason
    } else {
        feature.is_none() && !premium_reason
    };
    if valid {
        Ok(())
    } else {
        Err(EntitlementPolicyTypeError::FeatureReasonMismatch)
    }
}

fn validate_recovery(
    family: CapabilityFamily,
    allowance: RecoveryAllowance,
) -> Result<(), EntitlementPolicyTypeError> {
    let valid = match family {
        CapabilityFamily::AccountRecovery => matches!(
            allowance,
            RecoveryAllowance::AccountRecovery
                | RecoveryAllowance::StableSecurityUpdate
                | RecoveryAllowance::RepairRollback
                | RecoveryAllowance::Uninstall
        ),
        CapabilityFamily::ReadProjection => allowance == RecoveryAllowance::ReadProjection,
        CapabilityFamily::CustomerDataExport => allowance == RecoveryAllowance::CustomerDataExport,
        _ => allowance == RecoveryAllowance::None,
    };
    if valid {
        Ok(())
    } else {
        Err(EntitlementPolicyTypeError::RecoveryAllowanceMismatch)
    }
}

fn is_qualified_identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix) && value.len() > prefix.len() && value.split('.').all(is_identifier)
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && value.as_bytes()[0].is_ascii_lowercase()
}

/// Canonical base Focusa product gate (Spec 152F P3 / P2).
///
/// One usable signed product entitlement for product `focusa` grants the base
/// product. Value-producing core mutations (projects, missions, Focus State,
/// Workpoints, Trajectories, Work Loops, evidence, cognition) inherit this single
/// decision; the legacy `focusa.core.mission` / `focusa.core.workpoint` /
/// `focusa.core.evidence` identifiers resolve as parts of the base product and are
/// never separately purchased features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseProductDecision {
    /// Usable signed product entitlement (Active paid lease or valid Offline Grace).
    Entitled,
    /// Verified identity but no usable product entitlement: only the explicit
    /// manual one-project Focusa subset is permitted.
    Limited,
    /// No base product entitlement; value-producing mutations are denied.
    Denied,
}

impl BaseProductDecision {
    /// Base gate satisfied: value-producing core mutations are permitted.
    pub const fn permits_base_mutations(self) -> bool {
        matches!(self, Self::Entitled)
    }

    /// Stable snake_case label for projections and telemetry.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Entitled => "entitled",
            Self::Limited => "limited",
            Self::Denied => "denied",
        }
    }
}

/// Legacy core identifiers that resolve as parts of the base Focusa product.
/// They may remain in leases, telemetry, and compatibility projections, but they
/// SHALL resolve as base-product claims rather than separate purchases.
pub const BASE_PRODUCT_CORE_COMPATIBILITY_IDS: [&str; 3] = [
    "focusa.core.mission",
    "focusa.core.workpoint",
    "focusa.core.evidence",
];

/// Resolve the canonical base Focusa product gate from a Spec 172 policy state.
///
/// Only `product == "focusa"` counts. Active paid and valid Offline Grace are
/// usable; verified-but-license-less resolves to the explicit manual one-project
/// subset; every other state denies value-producing mutations. No caller-supplied
/// product, price, grant, or feature input is accepted here — those remain
/// authority-owned concerns resolved before this bounded policy step.
pub fn resolve_base_focusa_product(
    product: &str,
    state: PolicyEntitlementState,
) -> BaseProductDecision {
    if !product.trim().eq_ignore_ascii_case("focusa") {
        return BaseProductDecision::Denied;
    }
    match state {
        PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace => {
            BaseProductDecision::Entitled
        }
        PolicyEntitlementState::VerifiedNoLicense => BaseProductDecision::Limited,
        _ => BaseProductDecision::Denied,
    }
}

/// Compatibility projection for the base product (Spec 152F P3).
///
/// `focusa.core.mission`, `focusa.core.workpoint`, and `focusa.core.evidence`
/// remain visible for telemetry and compatibility, but their projected values
/// resolve from the base product gate — stored lease values are non-authoritative
/// projection claims, never separately purchased features.
pub fn base_product_compatibility_projection(
    decision: BaseProductDecision,
    stored_features: &std::collections::BTreeMap<String, bool>,
) -> std::collections::BTreeMap<String, bool> {
    let entitled = decision.permits_base_mutations();
    let mut projected = stored_features.clone();
    for id in BASE_PRODUCT_CORE_COMPATIBILITY_IDS {
        projected.insert(id.to_string(), entitled);
    }
    projected
}

/// Authority-owned feature identifiers for each of the four optional premium
/// families. The operation policy supplies the family and its canonical feature
/// identifier; callers never supply a grant or expand these sets.
pub const AUTOMATION_PREMIUM_FEATURE_IDS: &[&str] = &[
    "focusa.agent.parallelism",
    "focusa.agent.silent_sessions",
];
pub const TEAM_REMOTE_PREMIUM_FEATURE_IDS: &[&str] = &[
    "focusa.remote.stream",
    "focusa.team.multi_operator",
];
pub const RELEASE_PROOF_PREMIUM_FEATURE_IDS: &[&str] = &["focusa.release.proof"];
pub const PREMIUM_UPDATES_PREMIUM_FEATURE_IDS: &[&str] = &[
    "focusa.install.channel.nightly",
    "focusa.install.channel.preview",
    "focusa.update.unattended",
];

/// Return the exact registered feature identifiers for one optional family.
/// Non-premium families intentionally return an empty set and cannot be
/// promoted by a caller into an optional feature decision.
pub const fn premium_family_feature_ids(family: CapabilityFamily) -> &'static [&'static str] {
    match family {
        CapabilityFamily::Automation => AUTOMATION_PREMIUM_FEATURE_IDS,
        CapabilityFamily::TeamRemote => TEAM_REMOTE_PREMIUM_FEATURE_IDS,
        CapabilityFamily::ReleaseProof => RELEASE_PROOF_PREMIUM_FEATURE_IDS,
        CapabilityFamily::PremiumUpdates => PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
        _ => &[],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PremiumFamilyDenial {
    /// Base Focusa is always checked before an optional family.
    BaseProductRequired { decision: BaseProductDecision },
    /// The authority snapshot has no usable lease sequence to bind the result.
    MissingLeaseSequence,
    /// The authority snapshot is missing the immutable lease identity/digest
    /// needed to bind a feature claim to its authority record.
    MissingLeaseBinding,
    /// A feature identifier was not a qualified Focusa identifier.
    InvalidRequiredFeature { feature: String },
    /// The requested operation feature is not registered under this family.
    FeatureNotRegistered {
        family: CapabilityFamily,
        feature: RequiredFeature,
    },
    /// The signed authority feature allowlist does not grant this operation.
    MissingFeature {
        family: CapabilityFamily,
        feature: RequiredFeature,
    },
    /// An Offline Grace snapshot did not carry a bounded cached-grant window.
    MissingCachedGrantExpiry,
    /// The cached authority grant is outside its signed Offline Grace window.
    CachedGrantExpired,
    /// A directly supplied snapshot is stale even though it says Active.
    ActiveLeaseExpired,
    /// Recovery, read, base, or maintenance policy is not an optional family.
    NotPremiumFamily { family: CapabilityFamily },
}

/// A typed result for one operation-bound premium feature decision.
///
/// The `Feature` variant is only produced after the base product gate, a
/// non-zero authority lease sequence, the family-to-feature registry mapping,
/// and the authority feature claim all pass. `offline_cached` is descriptive;
/// it never broadens the authority feature set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PremiumFamilyDecision {
    Feature {
        family: CapabilityFamily,
        required_feature: RequiredFeature,
        lease_sequence: u64,
        offline_cached: bool,
    },
    Denied(PremiumFamilyDenial),
}

impl PremiumFamilyDecision {
    pub const fn posture(&self) -> EntitlementPolicyPosture {
        match self {
            Self::Feature { .. } => EntitlementPolicyPosture::Feature,
            Self::Denied(_) => EntitlementPolicyPosture::Deny,
        }
    }

    pub const fn is_feature(&self) -> bool {
        matches!(self, Self::Feature { .. })
    }

    pub const fn lease_sequence(&self) -> Option<u64> {
        match self {
            Self::Feature { lease_sequence, .. } => Some(*lease_sequence),
            Self::Denied(_) => None,
        }
    }

    pub fn required_feature(&self) -> Option<&RequiredFeature> {
        match self {
            Self::Feature {
                required_feature, ..
            } => Some(required_feature),
            Self::Denied(_) => None,
        }
    }

    pub const fn denial(&self) -> Option<&PremiumFamilyDenial> {
        match self {
            Self::Feature { .. } => None,
            Self::Denied(denial) => Some(denial),
        }
    }
}

fn authority_policy_state(
    snapshot: &crate::authority::EntitlementSnapshot,
) -> PolicyEntitlementState {
    match snapshot.state {
        crate::authority::EntitlementState::Active => PolicyEntitlementState::ActivePaid,
        crate::authority::EntitlementState::OfflineGrace => PolicyEntitlementState::OfflineGrace,
        crate::authority::EntitlementState::Unactivated => PolicyEntitlementState::PendingUnverified,
        crate::authority::EntitlementState::RecoveryOnly => PolicyEntitlementState::RefundedOrRevoked,
    }
}

/// Resolve an operation-bound premium feature from one verified authority
/// snapshot. The feature key is operation metadata, not a grant request: it
/// must be one of the exact identifiers registered for `family`, and the
/// snapshot's authority-owned feature map is the only source of permission.
///
/// `now` is explicit so Offline Grace cannot be extended by a caller or by a
/// cached local flag. Authority verification normally establishes these bounds;
/// this function repeats the expiry check at the policy boundary before emitting
/// a sequence-bound FEATURE decision.
pub fn resolve_premium_family<F>(
    snapshot: &crate::authority::EntitlementSnapshot,
    family: CapabilityFamily,
    required_feature: F,
    now: DateTime<Utc>,
) -> PremiumFamilyDecision
where
    F: AsRef<str>,
{
    if premium_family_feature_ids(family).is_empty() {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::NotPremiumFamily { family });
    }

    let base = resolve_base_focusa_product(&snapshot.product, authority_policy_state(snapshot));
    if !base.permits_base_mutations() {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::BaseProductRequired {
            decision: base,
        });
    }

    let Some(lease_sequence) = snapshot.sequence.filter(|sequence| *sequence > 0) else {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingLeaseSequence);
    };
    if snapshot.lease_id.as_deref().is_none_or(str::is_empty)
        || snapshot.lease_digest.as_deref().is_none_or(str::is_empty)
    {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingLeaseBinding);
    }

    let feature_name = required_feature.as_ref().to_string();
    let Ok(feature) = RequiredFeature::new(feature_name.clone()) else {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::InvalidRequiredFeature {
            feature: feature_name,
        });
    };
    if !premium_family_feature_ids(family)
        .iter()
        .any(|registered| *registered == feature.as_str())
    {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::FeatureNotRegistered {
            family,
            feature,
        });
    }

    let offline_cached = snapshot.state == crate::authority::EntitlementState::OfflineGrace;
    if offline_cached {
        let Some(grace_until) = snapshot.offline_grace_until else {
            return PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingCachedGrantExpiry);
        };
        if now > grace_until {
            return PremiumFamilyDecision::Denied(PremiumFamilyDenial::CachedGrantExpired);
        }
    } else if snapshot
        .expires_at
        .is_some_and(|expires_at| now > expires_at)
    {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::ActiveLeaseExpired);
    }

    if !snapshot
        .features
        .get(feature.as_str())
        .copied()
        .unwrap_or(false)
    {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingFeature {
            family,
            feature,
        });
    }

    PremiumFamilyDecision::Feature {
        family,
        required_feature: feature,
        lease_sequence,
        offline_cached,
    }
}

#[cfg(test)]
mod premium_family_resolution_tests {
    use super::*;
    use crate::authority::{EntitlementSnapshot, EntitlementState};

    fn snapshot(state: EntitlementState) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-001");
        snapshot.state = state;
        snapshot.lease_id = Some("lease-001".to_string());
        snapshot.sequence = Some(7);
        snapshot.lease_digest = Some("sha256:lease".to_string());
        snapshot.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        snapshot
    }

    #[test]
    fn premium_family_resolution_maps_exact_feature_ids_and_is_base_first() {
        let mut active = snapshot(EntitlementState::Active);
        for (family, feature_ids) in [
            (CapabilityFamily::Automation, AUTOMATION_PREMIUM_FEATURE_IDS),
            (CapabilityFamily::TeamRemote, TEAM_REMOTE_PREMIUM_FEATURE_IDS),
            (
                CapabilityFamily::ReleaseProof,
                RELEASE_PROOF_PREMIUM_FEATURE_IDS,
            ),
            (
                CapabilityFamily::PremiumUpdates,
                PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
            ),
        ] {
            for feature in feature_ids {
                active.features.insert((*feature).to_string(), true);
                let decision = resolve_premium_family(&active, family, *feature, Utc::now());
                assert!(decision.is_feature(), "{family:?}/{feature}");
                assert_eq!(decision.lease_sequence(), Some(7));
                assert_eq!(decision.required_feature().unwrap().as_str(), *feature);
            }
        }

        active.product = "uiai-engine".to_string();
        assert_eq!(
            resolve_premium_family(
                &active,
                CapabilityFamily::Automation,
                "focusa.agent.parallelism",
                Utc::now()
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::BaseProductRequired {
                decision: BaseProductDecision::Denied,
            })
        );
    }

    #[test]
    fn premium_family_resolution_fails_closed_with_exact_missing_feature_reasons() {
        let active = snapshot(EntitlementState::Active);
        assert_eq!(
            resolve_premium_family(
                &active,
                CapabilityFamily::ReleaseProof,
                "focusa.release.proof",
                Utc::now()
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingFeature {
                family: CapabilityFamily::ReleaseProof,
                feature: RequiredFeature::new("focusa.release.proof").unwrap(),
            })
        );
        assert_eq!(
            resolve_premium_family(
                &active,
                CapabilityFamily::Automation,
                "focusa.release.proof",
                Utc::now()
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::FeatureNotRegistered {
                family: CapabilityFamily::Automation,
                feature: RequiredFeature::new("focusa.release.proof").unwrap(),
            })
        );
        assert_eq!(
            resolve_premium_family(
                &active,
                CapabilityFamily::Automation,
                "focusa.agent.parallelism",
                Utc::now()
            )
            .posture(),
            EntitlementPolicyPosture::Deny
        );
    }

    #[test]
    fn premium_family_resolution_requires_sequence_and_bounded_cached_grants() {
        let mut active = snapshot(EntitlementState::Active);
        active.features.insert("focusa.release.proof".to_string(), true);
        active.sequence = None;
        assert_eq!(
            resolve_premium_family(
                &active,
                CapabilityFamily::ReleaseProof,
                "focusa.release.proof",
                Utc::now()
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingLeaseSequence)
        );

        let mut offline = snapshot(EntitlementState::OfflineGrace);
        offline.features.insert("focusa.release.proof".to_string(), true);
        offline.offline_grace_until = Some(Utc::now() + chrono::Duration::minutes(5));
        let decision = resolve_premium_family(
            &offline,
            CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
            Utc::now(),
        );
        assert_eq!(decision.lease_sequence(), Some(7));
        assert!(matches!(
            decision,
            PremiumFamilyDecision::Feature {
                offline_cached: true,
                ..
            }
        ));

        assert_eq!(
            resolve_premium_family(
                &offline,
                CapabilityFamily::ReleaseProof,
                "focusa.release.proof",
                Utc::now() + chrono::Duration::minutes(6)
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::CachedGrantExpired)
        );
        assert_eq!(
            resolve_premium_family(
                &offline,
                CapabilityFamily::PremiumUpdates,
                "focusa.install.channel.preview",
                Utc::now()
            ),
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingFeature {
                family: CapabilityFamily::PremiumUpdates,
                feature: RequiredFeature::new("focusa.install.channel.preview").unwrap(),
            })
        );
    }
}

#[cfg(test)]
mod recovery_allowance_tests {
    use super::{
        reduce_entitlement_state, CapabilityFamily as Family, DecisionReason as Reason,
        EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
        RecoveryAllowance, SecurityPrerequisite,
    };

    #[test]
    fn recovery_allowances_require_typed_security_prerequisites() {
        assert!(RecoveryAllowance::AccountRecovery
            .security_prerequisites()
            .contains(&SecurityPrerequisite::IdentityVerification));
        assert!(RecoveryAllowance::AccountRecovery
            .security_prerequisites()
            .contains(&SecurityPrerequisite::RolePermission));
        assert!(RecoveryAllowance::StableSecurityUpdate
            .security_prerequisites()
            .contains(&SecurityPrerequisite::ArtifactSignature));
        assert!(RecoveryAllowance::CustomerDataExport
            .security_prerequisites()
            .contains(&SecurityPrerequisite::PrivacyRedaction));
        assert!(RecoveryAllowance::Uninstall
            .security_prerequisites()
            .contains(&SecurityPrerequisite::OperatorConfirmation));
        assert!(RecoveryAllowance::None.security_prerequisites().is_empty());
    }

    #[test]
    fn recovery_allowances_return_non_commercial_postures() {
        let states = [
            State::PendingUnverified,
            State::VerifiedNoLicense,
            State::ActivePaid,
            State::OfflineGrace,
            State::Expired,
            State::RefundedOrRevoked,
            State::MissingOrCorrupt,
        ];

        for state in states {
            let recovery = reduce_entitlement_state(state, Family::AccountRecovery, None);
            let export = reduce_entitlement_state(state, Family::CustomerDataExport, None);

            assert_ne!(recovery.posture(), Posture::Deny, "{state:?}/account_recovery");
            assert_ne!(export.posture(), Posture::Deny, "{state:?}/customer_data_export");
            if state != State::PendingUnverified {
                let read = reduce_entitlement_state(state, Family::ReadProjection, None);
                assert_ne!(read.posture(), Posture::Deny, "{state:?}/read_projection");
            }

            assert_ne!(recovery.reason(), Reason::RequireFeature);
            assert_ne!(recovery.reason(), Reason::RequireCachedFeature);
        }
    }

    #[test]
    fn recovery_allowances_resolve_to_expected_families() {
        assert_eq!(
            RecoveryAllowance::AccountRecovery.implied_family(),
            Some(Family::AccountRecovery)
        );
        assert_eq!(
            RecoveryAllowance::StableSecurityUpdate.implied_family(),
            Some(Family::AccountRecovery)
        );
        assert_eq!(
            RecoveryAllowance::RepairRollback.implied_family(),
            Some(Family::AccountRecovery)
        );
        assert_eq!(
            RecoveryAllowance::CustomerDataExport.implied_family(),
            Some(Family::CustomerDataExport)
        );
        assert_eq!(
            RecoveryAllowance::Uninstall.implied_family(),
            Some(Family::AccountRecovery)
        );
        assert_eq!(
            RecoveryAllowance::ReadProjection.implied_family(),
            Some(Family::ReadProjection)
        );
        assert_eq!(RecoveryAllowance::None.implied_family(), None);
    }
}

#[cfg(test)]
#[path = "entitlement_policy_tests.rs"]
mod tests;
