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
