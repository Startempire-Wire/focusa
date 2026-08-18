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

impl OperationClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::ValueMutation => "value_mutation",
            Self::Recovery => "recovery",
            Self::InternalMaintenance => "internal_maintenance",
            Self::Unknown => "unknown",
        }
    }
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
    pub const fn label(self) -> &'static str {
        match self {
            Self::AccountRecovery => "account_recovery",
            Self::ReadProjection => "read_projection",
            Self::BaseFocusa => "base_focusa",
            Self::Automation => "automation",
            Self::TeamRemote => "team_remote",
            Self::ReleaseProof => "release_proof",
            Self::PremiumUpdates => "premium_updates",
            Self::CustomerDataExport => "customer_data_export",
            Self::InternalMaintenance => "internal_maintenance",
        }
    }
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

impl CommercialTreatment {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AlwaysAvailable => "always_available",
            Self::ReadAllowance => "read_allowance",
            Self::BaseEntitlement => "base_entitlement",
            Self::OptionalPremium => "optional_premium",
            Self::AlwaysAvailableBasicWithOptionalPremiumPackaging => {
                "always_available_basic_with_optional_premium_packaging"
            }
            Self::InheritInitiatingOperation => "inherit_initiating_operation",
        }
    }
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

/// Future-granularity dimensions are carried as observable claims, but only a
/// registered, authority-backed policy may make one runtime-relevant.  The
/// existing capability-family resolver remains the capability authority; this
/// model must never be used as a second paywall or as a source of role grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FutureGranularityDimension {
    Operation,
    SubCapability,
    Role,
    Origin,
    Channel,
    Plan,
    Limit,
    Time,
}

impl FutureGranularityDimension {
    pub const ALL: [Self; 8] = [
        Self::Operation,
        Self::SubCapability,
        Self::Role,
        Self::Origin,
        Self::Channel,
        Self::Plan,
        Self::Limit,
        Self::Time,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::SubCapability => "sub_capability",
            Self::Role => "role",
            Self::Origin => "origin",
            Self::Channel => "channel",
            Self::Plan => "plan",
            Self::Limit => "limit",
            Self::Time => "time",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Operation => 0,
            Self::SubCapability => 1,
            Self::Role => 2,
            Self::Origin => 3,
            Self::Channel => 4,
            Self::Plan => 5,
            Self::Limit => 6,
            Self::Time => 7,
        }
    }
}

/// One authority-lease dimension claim.  `activation` is deliberately
/// observable even when `claim` is absent: a dormant claim is data for
/// projection/audit only, never an implicit denial or grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FutureDimensionClaim {
    activation: PolicyActivation,
    claim: Option<String>,
}

impl FutureDimensionClaim {
    pub const fn dormant() -> Self {
        Self {
            activation: PolicyActivation::Dormant,
            claim: None,
        }
    }

    /// Preserve an observed dormant value without making it commercially
    /// effective.  This is useful for redacted projections and audit records.
    pub fn observed_dormant(claim: Option<String>) -> Self {
        Self {
            activation: PolicyActivation::Dormant,
            claim,
        }
    }

    /// Construct a claim as if it came from an authority projection.  This
    /// does not activate a dimension by itself; `GranularityActivationGuard`
    /// still requires the matching embedded registered policy.
    pub fn active(claim: impl Into<String>) -> Self {
        Self {
            activation: PolicyActivation::Active,
            claim: Some(claim.into()),
        }
    }

    pub const fn activation(&self) -> PolicyActivation {
        self.activation
    }

    pub fn claim(&self) -> Option<&str> {
        self.claim.as_deref()
    }

    fn has_authority_claim(&self) -> bool {
        self.activation.permits_runtime_commercial_decision()
            && self.claim.as_deref().is_some_and(|claim| !claim.is_empty())
    }
}

/// All future-granularity claims are present in one bounded projection.  The
/// fields intentionally remain private so callers use the typed projection
/// rather than manufacturing a different shape for one presenter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FutureGranularityClaims {
    operation: FutureDimensionClaim,
    sub_capability: FutureDimensionClaim,
    role: FutureDimensionClaim,
    origin: FutureDimensionClaim,
    channel: FutureDimensionClaim,
    plan: FutureDimensionClaim,
    limit: FutureDimensionClaim,
    time: FutureDimensionClaim,
}

impl Default for FutureGranularityClaims {
    fn default() -> Self {
        Self {
            operation: FutureDimensionClaim::dormant(),
            sub_capability: FutureDimensionClaim::dormant(),
            role: FutureDimensionClaim::dormant(),
            origin: FutureDimensionClaim::dormant(),
            channel: FutureDimensionClaim::dormant(),
            plan: FutureDimensionClaim::dormant(),
            limit: FutureDimensionClaim::dormant(),
            time: FutureDimensionClaim::dormant(),
        }
    }
}

impl FutureGranularityClaims {
    pub fn with(self, dimension: FutureGranularityDimension, claim: FutureDimensionClaim) -> Self {
        match dimension {
            FutureGranularityDimension::Operation => Self {
                operation: claim,
                ..self
            },
            FutureGranularityDimension::SubCapability => Self {
                sub_capability: claim,
                ..self
            },
            FutureGranularityDimension::Role => Self {
                role: claim,
                ..self
            },
            FutureGranularityDimension::Origin => Self {
                origin: claim,
                ..self
            },
            FutureGranularityDimension::Channel => Self {
                channel: claim,
                ..self
            },
            FutureGranularityDimension::Plan => Self {
                plan: claim,
                ..self
            },
            FutureGranularityDimension::Limit => Self {
                limit: claim,
                ..self
            },
            FutureGranularityDimension::Time => Self {
                time: claim,
                ..self
            },
        }
    }

    pub fn claim(&self, dimension: FutureGranularityDimension) -> &FutureDimensionClaim {
        match dimension {
            FutureGranularityDimension::Operation => &self.operation,
            FutureGranularityDimension::SubCapability => &self.sub_capability,
            FutureGranularityDimension::Role => &self.role,
            FutureGranularityDimension::Origin => &self.origin,
            FutureGranularityDimension::Channel => &self.channel,
            FutureGranularityDimension::Plan => &self.plan,
            FutureGranularityDimension::Limit => &self.limit,
            FutureGranularityDimension::Time => &self.time,
        }
    }

    pub fn operation(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Operation)
    }

    pub fn sub_capability(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::SubCapability)
    }

    pub fn role(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Role)
    }

    pub fn origin(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Origin)
    }

    pub fn channel(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Channel)
    }

    pub fn plan(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Plan)
    }

    pub fn limit(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Limit)
    }

    pub fn time(&self) -> &FutureDimensionClaim {
        self.claim(FutureGranularityDimension::Time)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FutureDimensionAuthority {
    AuditOnly,
    FutureOperatorApprovedPolicy,
    SecurityAuthorizationOnly,
    RoutingSecurityPolicyOnly,
    PolicyRegistryAndSignedLease,
    EddAndSignedLease,
    ServerOwnedRegistryAndSignedLease,
    SignedLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisteredFutureDimensionPolicy {
    dimension: FutureGranularityDimension,
    activation: PolicyActivation,
    authority: FutureDimensionAuthority,
}

impl RegisteredFutureDimensionPolicy {
    pub const fn dimension(self) -> FutureGranularityDimension {
        self.dimension
    }

    pub const fn activation(self) -> PolicyActivation {
        self.activation
    }

    pub const fn authority(self) -> FutureDimensionAuthority {
        self.authority
    }
}

/// The only runtime activation registry.  It is closed and embedded: callers
/// cannot register a new policy or turn a dormant dimension on at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FutureGranularityPolicyRegistry {
    policies: [RegisteredFutureDimensionPolicy; 8],
}

impl FutureGranularityPolicyRegistry {
    pub const fn canonical() -> Self {
        Self {
            policies: [
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Operation,
                    activation: PolicyActivation::Dormant,
                    authority: FutureDimensionAuthority::AuditOnly,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::SubCapability,
                    activation: PolicyActivation::Dormant,
                    authority: FutureDimensionAuthority::FutureOperatorApprovedPolicy,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Role,
                    activation: PolicyActivation::DormantForCommerce,
                    authority: FutureDimensionAuthority::SecurityAuthorizationOnly,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Origin,
                    activation: PolicyActivation::DormantForCommerce,
                    authority: FutureDimensionAuthority::RoutingSecurityPolicyOnly,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Channel,
                    activation: PolicyActivation::ActiveForPreviewNightlyAndUnattended,
                    authority: FutureDimensionAuthority::PolicyRegistryAndSignedLease,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Plan,
                    activation: PolicyActivation::Active,
                    authority: FutureDimensionAuthority::EddAndSignedLease,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Limit,
                    activation: PolicyActivation::ActiveOnlyWhenDeclared,
                    authority: FutureDimensionAuthority::ServerOwnedRegistryAndSignedLease,
                },
                RegisteredFutureDimensionPolicy {
                    dimension: FutureGranularityDimension::Time,
                    activation: PolicyActivation::Active,
                    authority: FutureDimensionAuthority::SignedLease,
                },
            ],
        }
    }

    pub fn registered_policy(
        &self,
        dimension: FutureGranularityDimension,
    ) -> Option<RegisteredFutureDimensionPolicy> {
        self.policies
            .iter()
            .copied()
            .find(|policy| policy.dimension == dimension)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FutureGranularityDecision {
    /// The field remains observable, but has no authorization effect.
    IgnoredDormant,
    /// The registered policy and authority claim were both present.  This is
    /// still only a dimension check; it is not a capability or role grant.
    AuthorityBacked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FutureGranularityAuthorization {
    decisions: [FutureGranularityDecision; 8],
}

impl FutureGranularityAuthorization {
    pub fn decision(&self, dimension: FutureGranularityDimension) -> FutureGranularityDecision {
        self.decisions[dimension.index()]
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum FutureGranularityError {
    #[error("future dimension has no registered policy: {0:?}")]
    UnregisteredPolicy(FutureGranularityDimension),
    #[error("dormant future dimension cannot be activated at runtime: {0:?}")]
    UnapprovedRuntimeActivation(FutureGranularityDimension),
    #[error("active future dimension requires an authority claim: {0:?}")]
    MissingAuthorityClaim(FutureGranularityDimension),
    #[error("licensing cannot grant a role or operator authority")]
    LicensingRoleGrantForbidden,
}

/// Guard future dimensions before they can participate in runtime policy.
/// Dormant claims, including observed values, are intentionally ignored.
/// Active dimensions require the closed registered policy and a non-empty
/// authority claim; role permission is never a licensing grant.
pub struct GranularityActivationGuard;

impl GranularityActivationGuard {
    fn evaluate_one(
        registry: &FutureGranularityPolicyRegistry,
        dimension: FutureGranularityDimension,
        claim: &FutureDimensionClaim,
    ) -> Result<FutureGranularityDecision, FutureGranularityError> {
        let policy = registry
            .registered_policy(dimension)
            .ok_or(FutureGranularityError::UnregisteredPolicy(dimension))?;
        if dimension == FutureGranularityDimension::Role
            && claim.activation().permits_runtime_commercial_decision()
        {
            return Err(FutureGranularityError::LicensingRoleGrantForbidden);
        }
        if !policy.activation().permits_runtime_commercial_decision() {
            if claim.activation().permits_runtime_commercial_decision() {
                return Err(FutureGranularityError::UnapprovedRuntimeActivation(
                    dimension,
                ));
            }
            return Ok(FutureGranularityDecision::IgnoredDormant);
        }
        if !claim.has_authority_claim() {
            return Err(FutureGranularityError::MissingAuthorityClaim(dimension));
        }
        Ok(FutureGranularityDecision::AuthorityBacked)
    }

    pub fn evaluate(
        claims: &FutureGranularityClaims,
    ) -> Result<FutureGranularityAuthorization, FutureGranularityError> {
        let registry = FutureGranularityPolicyRegistry::canonical();
        let mut decisions = [FutureGranularityDecision::IgnoredDormant; 8];
        for dimension in FutureGranularityDimension::ALL {
            decisions[dimension.index()] =
                Self::evaluate_one(&registry, dimension, claims.claim(dimension))?;
        }
        Ok(FutureGranularityAuthorization { decisions })
    }

    pub fn evaluate_dimension(
        dimension: FutureGranularityDimension,
        claim: &FutureDimensionClaim,
    ) -> Result<FutureGranularityDecision, FutureGranularityError> {
        let registry = FutureGranularityPolicyRegistry::canonical();
        Self::evaluate_one(&registry, dimension, claim)
    }
}

/// Validate the contract metadata in the embedded policy as well as the typed
/// closed registry above.  This keeps the YAML projection from silently losing
/// explicit active/dormant status while the runtime guard remains fail closed.
fn validate_future_granularity_contract(document: &Value) -> Result<(), String> {
    let policy = document
        .get("entitlement_policy")
        .and_then(Value::as_object)
        .ok_or("missing entitlement_policy registry")?;
    let dimensions = policy
        .get("future_dimensions")
        .and_then(Value::as_array)
        .ok_or("future_dimensions must be an array")?;
    let expected = [
        ("capability_family", "active"),
        ("sub_capability", "dormant"),
        ("operation", "dormant"),
        ("limit_bucket", "active"),
        ("product_tier", "active"),
        ("role_permission", "dormant"),
        ("node_device", "active"),
        ("channel", "active"),
        ("time_window", "active"),
        ("origin_facade", "dormant"),
    ];
    for (id, status) in expected {
        let row = dimensions
            .iter()
            .find(|row| row.get("id").and_then(Value::as_str) == Some(id))
            .ok_or_else(|| format!("missing future dimension: {id}"))?;
        if row.get("status").and_then(Value::as_str) != Some(status) {
            return Err(format!("future dimension {id} has invalid explicit status"));
        }
    }
    if dimensions.len() != expected.len() {
        return Err("future dimension registry has an unexpected dimension count".into());
    }
    Ok(())
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

impl PolicyEntitlementState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PendingUnverified => "pending_unverified",
            Self::VerifiedNoLicense => "verified_no_license",
            Self::ActivePaid => "active_paid",
            Self::OfflineGrace => "offline_grace",
            Self::Expired => "expired",
            Self::RefundedOrRevoked => "refunded_or_revoked",
            Self::MissingOrCorrupt => "missing_or_corrupt",
        }
    }
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

impl DecisionReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowVerifiedLimited => "allow_verified_limited",
            Self::Read => "read",
            Self::ReadLocalOnly => "read_local_only",
            Self::AllowExistingLocalOnly => "allow_existing_local_only",
            Self::AllowOfflineOnly => "allow_offline_only",
            Self::RequireBase => "require_base",
            Self::RequireFeature => "require_feature",
            Self::RequireCachedFeature => "require_cached_feature",
            Self::RequireCachedFeatureWhenSafe => "require_cached_feature_when_safe",
            Self::Inherit => "inherit",
            Self::MissingInitiatingPolicy => "missing_initiating_policy",
            Self::Deny => "deny",
        }
    }

    pub const fn recovery_action(self) -> &'static str {
        match self {
            Self::RequireBase => "activate_evaluation_purchase_or_manage_entitlement",
            Self::RequireFeature
            | Self::RequireCachedFeature
            | Self::RequireCachedFeatureWhenSafe => "review_offer_or_manage_entitlement",
            Self::Deny | Self::MissingInitiatingPolicy => {
                "activate_evaluation_purchase_or_manage_entitlement"
            }
            _ => "license_status",
        }
    }
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

impl EntitlementPolicyPosture {
    pub const fn status(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Read => "read",
            Self::Base => "base",
            Self::Feature => "feature",
            Self::Deny => "deny",
        }
    }
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
pub fn embedded_entitlement_policy_registry()
-> Result<&'static EmbeddedEntitlementPolicyRegistry, EntitlementPolicyRegistryError> {
    static REGISTRY: OnceLock<
        Result<EmbeddedEntitlementPolicyRegistry, EntitlementPolicyRegistryError>,
    > = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let document: Value = serde_json::from_str(EMBEDDED_POLICY_REGISTRY_JSON)
                .map_err(|error| registry_error(error.to_string()))?;
            registry_validation::validate_registry_bundle(&document).map_err(registry_error)?;
            validate_future_granularity_contract(&document).map_err(registry_error)?;
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
    // Product codes are authority-owned canonical identifiers. Do not normalize
    // caller-provided values: whitespace, casing, aliases, and prefixed names
    // must never become a product grant at this boundary.
    if product != "focusa" {
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

/// Deterministic Spec 172 verified-no-license family classifier.
///
/// The posture is explicit allowlist-driven and fail-closed:
/// - blocked families are denied for their product.
/// - unknown families are denied.
/// - focusa manual_project mutations are allowed only while the mutable project
///   count remains at most one.
pub fn is_focusa_verified_no_license_family_allowed(
    product: &str,
    family: &str,
    mutable_project_count: usize,
) -> bool {
    match product {
        "focusa" => {
            if SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES.contains(&family) {
                return false;
            }
            if family == "manual_project" {
                return mutable_project_count <= 1;
            }
            SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.contains(&family)
        }
        "uiai_engine" => {
            SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.contains(&family)
                && !SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES.contains(&family)
        }
        _ => false,
    }
}

pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES: [&str; 6] = [
    "manual_project",
    "manual_mission",
    "manual_focus_state",
    "manual_workpoint",
    "manual_trajectory",
    "manual_basic_evidence",
];

pub const SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES: [&str; 4] = [
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
];

pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES: [&str; 6] = [
    "public_search",
    "source_to_markdown",
    "public_page_read",
    "accessibility_snapshot",
    "screenshot",
    "basic_diagnostics",
];

pub const SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES: [&str; 6] = [
    "browser_action",
    "browser_persistence",
    "authenticated_private_targets",
    "unattended_browser_automation",
    "scheduled_batch_qa",
    "premium_hosted_resources",
];

/// Canonical Focusa Operator v1 capability families.
///
/// All families included in the Focusa Operator Lifetime v1 License Type.
/// New operations in these families inherit when the five Spec 172 Section 8.2
/// conditions are all met; materially new families are excluded by default.
pub const SPEC172_FOCUSA_OPERATOR_V1_FAMILIES: [&str; 10] = [
    "manual_project",
    "manual_mission",
    "manual_focus_state",
    "manual_workpoint",
    "manual_trajectory",
    "manual_basic_evidence",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
];

/// Deterministic Spec 172 Operator family inheritance decision.
///
/// Implements Section 8.2 (existing-family inheritance) and 8.3 (materially
/// new capability). A new operation inherits an existing Operator family only
/// when all five conditions are met; otherwise it is excluded pending explicit
/// assignment, or denied for unknown owner/product/side-effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorFamilyInheritanceDecision {
    /// Operation inherits an existing Operator family (all five 8.2 conditions met).
    Inherit,
    /// Materially new family: excluded pending explicit versioned assignment.
    ExcludedPendingAssignment,
    /// Unknown product: denies all classification.
    DeniedUnknownProduct,
    /// Unknown owner: denies all classification.
    DeniedUnknownOwner,
    /// Unknown side effect class: denies all classification.
    DeniedUnknownSideEffect,
    /// Future (unregistered) product excluded pending operator-approved registration.
    DeniedFutureProduct,
    /// Materially new hosted cost excluded.
    DeniedMateriallyNewHostedCost,
}

impl OperatorFamilyInheritanceDecision {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::ExcludedPendingAssignment => "excluded_pending_assignment",
            Self::DeniedUnknownProduct => "denied_unknown_product",
            Self::DeniedUnknownOwner => "denied_unknown_owner",
            Self::DeniedUnknownSideEffect => "denied_unknown_side_effect",
            Self::DeniedFutureProduct => "denied_future_product",
            Self::DeniedMateriallyNewHostedCost => "denied_materially_new_hosted_cost",
        }
    }

    pub const fn is_inherited(&self) -> bool {
        matches!(self, Self::Inherit)
    }

    pub const fn is_denied(&self) -> bool {
        matches!(
            self,
            Self::DeniedUnknownProduct
                | Self::DeniedUnknownOwner
                | Self::DeniedUnknownSideEffect
                | Self::DeniedFutureProduct
                | Self::DeniedMateriallyNewHostedCost
        )
    }
}

/// Classify whether a new operation inherits an existing Operator family.
///
/// Implements Spec 172 Sections 8.2 and 8.3. The five conditions of 8.2 must
/// ALL be true for inheritance; otherwise the classifier fails closed:
///
/// 1. Same registered product
/// 2. Same customer-understandable outcome as an included family
/// 3. Security, side-effect, privacy, and resource profile fits the family
/// 4. No separately named product
/// 5. No materially new hosted cost
///
/// Materially new families, future products, unknown owners, and unknown
/// side effects are denied for all pending classification.
///
/// Callers supply the factual classification inputs; the classifier determines
/// the inheritance decision. Callers NEVER supply product, price, family,
/// feature, limit, node, or commercial right.
pub fn classify_operator_family_inheritance(
    product_owner: &str,
    capability_family: &str,
    is_known_registered_product: bool,
    is_known_operator_family: bool,
    is_known_owner: bool,
    is_known_side_effect: bool,
    has_materially_new_hosted_cost: bool,
) -> OperatorFamilyInheritanceDecision {
    // Gate 1: unknown product → deny (fail closed)
    if !is_known_registered_product {
        return OperatorFamilyInheritanceDecision::DeniedUnknownProduct;
    }
    // Gate 2: future (unregistered) product → deny
    if product_owner != "focusa" && product_owner != "uiai_engine" {
        return OperatorFamilyInheritanceDecision::DeniedFutureProduct;
    }
    // Gate 3: unknown owner → deny
    if !is_known_owner {
        return OperatorFamilyInheritanceDecision::DeniedUnknownOwner;
    }
    // Gate 4: unknown side effect → deny
    if !is_known_side_effect {
        return OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect;
    }
    // Gate 5: materially new family (not in known Operator families) → excluded
    if !is_known_operator_family {
        return OperatorFamilyInheritanceDecision::ExcludedPendingAssignment;
    }
    // Gate 6: materially new hosted cost → denied
    if has_materially_new_hosted_cost {
        return OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost;
    }
    // All five Spec 8.2 conditions met → inherit
    OperatorFamilyInheritanceDecision::Inherit
}

/// Authority-owned feature identifiers for each of the four optional premium
/// families. The operation policy supplies the family and its canonical feature
/// identifier; callers never supply a grant or expand these sets.
pub const AUTOMATION_PREMIUM_FEATURE_IDS: &[&str] =
    &["focusa.agent.parallelism", "focusa.agent.silent_sessions"];
pub const TEAM_REMOTE_PREMIUM_FEATURE_IDS: &[&str] =
    &["focusa.remote.stream", "focusa.team.multi_operator"];
pub const RELEASE_PROOF_PREMIUM_FEATURE_IDS: &[&str] = &["focusa.release.proof"];
pub const PREMIUM_UPDATES_PREMIUM_FEATURE_IDS: &[&str] = &[
    "focusa.install.channel.nightly",
    "focusa.install.channel.preview",
    "focusa.update.unattended",
];
pub const CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS: &[&str] = &["focusa.export.packaged"];

/// Return the exact registered feature identifiers for one optional family.
/// Non-premium families (except CustomerDataExport which carries an optional
/// premium packaging feature) intentionally return an empty set and cannot be
/// promoted by a caller into an optional feature decision.
pub const fn premium_family_feature_ids(family: CapabilityFamily) -> &'static [&'static str] {
    match family {
        CapabilityFamily::Automation => AUTOMATION_PREMIUM_FEATURE_IDS,
        CapabilityFamily::TeamRemote => TEAM_REMOTE_PREMIUM_FEATURE_IDS,
        CapabilityFamily::ReleaseProof => RELEASE_PROOF_PREMIUM_FEATURE_IDS,
        CapabilityFamily::PremiumUpdates => PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
        CapabilityFamily::CustomerDataExport => CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS,
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
    /// The snapshot's entitlement state cannot carry a usable premium feature
    /// (unactivated/pending, refunded/revoked, or corrupt). Only Active paid and
    /// valid Offline Grace can resolve a premium feature; a stored claim or
    /// client metadata can never widen a non-usable state (Spec 152F §4 grid).
    EntitlementStateNotUsable { state: PolicyEntitlementState },
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

pub fn authority_policy_state(
    snapshot: &crate::authority::EntitlementSnapshot,
) -> PolicyEntitlementState {
    match snapshot.state {
        crate::authority::EntitlementState::Active => PolicyEntitlementState::ActivePaid,
        crate::authority::EntitlementState::OfflineGrace => PolicyEntitlementState::OfflineGrace,
        crate::authority::EntitlementState::Unactivated => {
            PolicyEntitlementState::PendingUnverified
        }
        crate::authority::EntitlementState::RecoveryOnly => {
            PolicyEntitlementState::RefundedOrRevoked
        }
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

/// Resolve an export-packaged premium feature from one verified authority
/// snapshot. Basic customer-data export is always available through the
/// `CustomerDataExport` recovery allowance; this function gates only the
/// optional value-added hosted packaging/transformation/report formats
/// behind `focusa.export.packaged`.
///
/// `now` is explicit so Offline Grace cannot be extended by a caller or by a
/// cached local flag. The function reuses the same premium-family resolution
/// path, but the base product gate is intentionally not required here: basic
/// export is always available, and this function only gates the additive
/// premium packaging feature.
pub fn resolve_export_packaged<F>(
    snapshot: &crate::authority::EntitlementSnapshot,
    required_feature: F,
    now: DateTime<Utc>,
) -> PremiumFamilyDecision
where
    F: AsRef<str>,
{
    let family = CapabilityFamily::CustomerDataExport;

    // Fail closed on entitlement state: only an Active paid lease or a valid
    // Offline Grace can carry the additive premium packaging feature. A
    // pending/unactivated, refunded/revoked, or corrupt snapshot can never be
    // widened by stored feature claims or client metadata (Spec 152F §4 grid:
    // premium features are DENY outside Active/Offline Grace). Basic export is
    // untouched: it remains always available through the CustomerDataExport
    // recovery allowance.
    let state = authority_policy_state(snapshot);
    if !matches!(
        state,
        PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace
    ) {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::EntitlementStateNotUsable {
            state,
        });
    }

    let feature_name = required_feature.as_ref().to_string();
    let Ok(feature) = RequiredFeature::new(feature_name.clone()) else {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::InvalidRequiredFeature {
            feature: feature_name,
        });
    };
    if !CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS
        .iter()
        .any(|registered| *registered == feature.as_str())
    {
        return PremiumFamilyDecision::Denied(PremiumFamilyDenial::FeatureNotRegistered {
            family,
            feature,
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
            (
                CapabilityFamily::TeamRemote,
                TEAM_REMOTE_PREMIUM_FEATURE_IDS,
            ),
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
        active
            .features
            .insert("focusa.release.proof".to_string(), true);
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
        offline
            .features
            .insert("focusa.release.proof".to_string(), true);
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
        CapabilityFamily as Family, DecisionReason as Reason, EntitlementPolicyPosture as Posture,
        PolicyEntitlementState as State, RecoveryAllowance, SecurityPrerequisite,
        reduce_entitlement_state,
    };

    #[test]
    fn recovery_allowances_require_typed_security_prerequisites() {
        assert!(
            RecoveryAllowance::AccountRecovery
                .security_prerequisites()
                .contains(&SecurityPrerequisite::IdentityVerification)
        );
        assert!(
            RecoveryAllowance::AccountRecovery
                .security_prerequisites()
                .contains(&SecurityPrerequisite::RolePermission)
        );
        assert!(
            RecoveryAllowance::StableSecurityUpdate
                .security_prerequisites()
                .contains(&SecurityPrerequisite::ArtifactSignature)
        );
        assert!(
            RecoveryAllowance::CustomerDataExport
                .security_prerequisites()
                .contains(&SecurityPrerequisite::PrivacyRedaction)
        );
        assert!(
            RecoveryAllowance::Uninstall
                .security_prerequisites()
                .contains(&SecurityPrerequisite::OperatorConfirmation)
        );
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

            assert_ne!(
                recovery.posture(),
                Posture::Deny,
                "{state:?}/account_recovery"
            );
            assert_ne!(
                export.posture(),
                Posture::Deny,
                "{state:?}/customer_data_export"
            );
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
mod dormant_granularity_tests {
    use super::{
        FutureDimensionClaim, FutureGranularityAuthorization, FutureGranularityClaims,
        FutureGranularityDecision, FutureGranularityDimension, FutureGranularityError,
        GranularityActivationGuard, embedded_entitlement_policy_registry,
    };

    fn active_claims() -> FutureGranularityClaims {
        FutureGranularityClaims::default()
            .with(
                FutureGranularityDimension::Channel,
                FutureDimensionClaim::active("preview"),
            )
            .with(
                FutureGranularityDimension::Plan,
                FutureDimensionClaim::active("focusa"),
            )
            .with(
                FutureGranularityDimension::Limit,
                FutureDimensionClaim::active("concurrent_agents"),
            )
            .with(
                FutureGranularityDimension::Time,
                FutureDimensionClaim::active("bounded_window"),
            )
    }

    #[test]
    fn dormant_granularity_fields_are_observable_but_ignored() {
        for dimension in [
            FutureGranularityDimension::Operation,
            FutureGranularityDimension::SubCapability,
            FutureGranularityDimension::Role,
            FutureGranularityDimension::Origin,
        ] {
            let claim = FutureDimensionClaim::observed_dormant(Some("observed-only".into()));
            assert_eq!(
                GranularityActivationGuard::evaluate_dimension(dimension, &claim),
                Ok(FutureGranularityDecision::IgnoredDormant),
                "dormant dimension must not deny or grant capability: {dimension:?}"
            );
            assert_eq!(claim.activation(), super::PolicyActivation::Dormant);
            assert_eq!(claim.claim(), Some("observed-only"));
        }

        let claims = FutureGranularityClaims::default();
        assert_eq!(claims.operation().claim(), None);
        assert_eq!(
            claims.sub_capability().activation(),
            super::PolicyActivation::Dormant
        );
        assert_eq!(claims.role().activation(), super::PolicyActivation::Dormant);
        assert_eq!(
            claims.origin().activation(),
            super::PolicyActivation::Dormant
        );
    }

    #[test]
    fn active_granularity_requires_registered_authority_claims() {
        for dimension in [
            FutureGranularityDimension::Channel,
            FutureGranularityDimension::Plan,
            FutureGranularityDimension::Limit,
            FutureGranularityDimension::Time,
        ] {
            assert_eq!(
                GranularityActivationGuard::evaluate_dimension(
                    dimension,
                    &FutureDimensionClaim::dormant()
                ),
                Err(FutureGranularityError::MissingAuthorityClaim(dimension))
            );
        }
    }

    #[test]
    fn registered_authority_claims_activate_only_active_dimensions() {
        let authorization: FutureGranularityAuthorization =
            GranularityActivationGuard::evaluate(&active_claims()).expect("valid claims");
        assert_eq!(
            authorization.decision(FutureGranularityDimension::Operation),
            FutureGranularityDecision::IgnoredDormant
        );
        assert_eq!(
            authorization.decision(FutureGranularityDimension::SubCapability),
            FutureGranularityDecision::IgnoredDormant
        );
        assert_eq!(
            authorization.decision(FutureGranularityDimension::Role),
            FutureGranularityDecision::IgnoredDormant
        );
        assert_eq!(
            authorization.decision(FutureGranularityDimension::Origin),
            FutureGranularityDecision::IgnoredDormant
        );
        for dimension in [
            FutureGranularityDimension::Channel,
            FutureGranularityDimension::Plan,
            FutureGranularityDimension::Limit,
            FutureGranularityDimension::Time,
        ] {
            assert_eq!(
                authorization.decision(dimension),
                FutureGranularityDecision::AuthorityBacked
            );
        }
    }

    #[test]
    fn unapproved_runtime_activation_is_rejected() {
        let claim = FutureDimensionClaim::active("caller-requested-operation");
        assert_eq!(
            GranularityActivationGuard::evaluate_dimension(
                FutureGranularityDimension::Operation,
                &claim
            ),
            Err(FutureGranularityError::UnapprovedRuntimeActivation(
                FutureGranularityDimension::Operation
            ))
        );
    }

    #[test]
    fn licensing_cannot_grant_roles_or_operator_authority() {
        let claim = FutureDimensionClaim::active("operator");
        assert_eq!(
            GranularityActivationGuard::evaluate_dimension(
                FutureGranularityDimension::Role,
                &claim
            ),
            Err(FutureGranularityError::LicensingRoleGrantForbidden)
        );
    }

    #[test]
    fn policy_contract_embeds_explicit_dimension_statuses() {
        let registry = embedded_entitlement_policy_registry().expect("embedded registry");
        assert!(registry.canonical_json().contains("\"status\":\"dormant\""));
        assert!(registry.canonical_json().contains("\"dimension\":\"plan\""));
        assert!(
            registry
                .canonical_json()
                .contains("\"licensing_role_grant_forbidden\":true")
        );
    }
}

#[cfg(test)]
#[path = "entitlement_policy_tests.rs"]
mod tests;
