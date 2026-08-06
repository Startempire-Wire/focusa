use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        Self::operator(ProductCode::Focusa, LicenseTypeCode::FocusaOperatorLifetimeV1)
    }

    pub const fn uiai_operator_v1() -> Self {
        Self::operator(ProductCode::UiaiEngine, LicenseTypeCode::UiaiOperatorLifetimeV1)
    }

    const fn operator(product: ProductCode, license_type: LicenseTypeCode) -> Self {
        Self {
            product, license_type, version: LicenseTypeVersion::V1,
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
        if *self == expected { Ok(()) } else { Err(EntitlementPolicyTypeError::InvalidLicenseTypeGrant) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeGrant {
    grants: [LicenseTypeGrant; 2],
}

impl CompositeGrant {
    pub fn operator_bundle_v1(grants: [LicenseTypeGrant; 2]) -> Result<Self, EntitlementPolicyTypeError> {
        let expected = [LicenseTypeGrant::focusa_operator_v1(), LicenseTypeGrant::uiai_operator_v1()];
        if grants != expected { return Err(EntitlementPolicyTypeError::MalformedBundleUnion); }
        Ok(Self { grants })
    }

    pub fn grants(&self) -> &[LicenseTypeGrant; 2] { &self.grants }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEntitlementState {
    PendingUnverified,
    VerifiedNoGrant,
    Evaluation,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReason {
    Allow,
    Read,
    ReadLocalOnly,
    AllowExistingLocalOnly,
    AllowOfflineOnly,
    RequireBase,
    RequireFeature,
    RequireCachedFeature,
    RequireCachedFeatureWhenSafe,
    Inherit,
    Deny,
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

#[cfg(test)]
#[path = "entitlement_policy_tests.rs"]
mod tests;
