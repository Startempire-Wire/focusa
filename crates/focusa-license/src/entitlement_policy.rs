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

#[cfg(test)]
#[path = "entitlement_policy_tests.rs"]
mod tests;
