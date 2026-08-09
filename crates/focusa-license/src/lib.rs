//! Focusa LicenseGuard \u2014 tier evaluation + capability assertions + BSL boundary.
//!
//! Bead: focusa-nbai (MVP BLOCKER).
//!
//! Runtime capability authority comes only from a signed Spec 152 authority lease.
//! Legacy tier/file parsing is retained solely as non-authoritative migration input;
//! missing, edited, expired, revoked, or unverifiable state cannot grant capability.

pub mod activation_client;
pub mod activation_agent;
pub mod activation_facade;
pub mod activation_http;
pub mod activation_reducer;
pub mod authority;
pub mod authority_client;
pub mod authority_credentials;
pub mod authority_http;
pub mod authority_store;
pub mod capsule_manifest;
pub mod dynamic_operation_manifest;
mod entitlement_policy;
pub mod denial_ux;
pub mod facade_policy_presenter;
pub mod feature_decision;
pub mod license_migration;
pub mod observability;

pub use activation_client::{
    ActivationClientError, ActivationJourney, ActivationLedgerEvent, ActivationRegistration,
    ActivationSession, ActivationStartReply, ActivationAuthority, CheckoutOutcome, PollOutcome,
    PublicOffer, DEFAULT_MAX_POLLS, retry_policy_for_code,
};
pub use activation_agent::{
    AGENT_ENVELOPE_SCHEMA, AgentActivationEnvelope, AgentKeyReveal, human_action_for_state,
    human_action_required, mask_key_prefix, masked_email_or_none,
};
pub use activation_facade::{
    ActivationError, ActivationErrorCode, ActivationErrorSpec, ActivationRequestContext,
    FacadeOperation, mask_email,
};
pub use activation_http::{
    ActivationHttpClient, ActivationHttpError, ActivationHttpPolicy, LeaseDeliveryEnvelope,
    code_from_label,
};
pub use activation_reducer::{
    ActivationEnvelopeError, ActivationErrorEnvelope, ActivationOutputEnvelope,
    ActivationState, ActivationTransition, ActivationTransitionError, PollRetryPolicy,
    PresenterActivationState, RetryPosture, presenter_next_action, presenter_state,
    reduce_activation,
};
pub use entitlement_policy::{
    authority_policy_state, base_product_compatibility_projection,
    classify_operator_family_inheritance,
    embedded_entitlement_policy_registry, is_focusa_verified_no_license_family_allowed,
    premium_family_feature_ids, reduce_entitlement_state, resolve_base_focusa_product,
    resolve_export_packaged, resolve_premium_family,
    AccessPosture, BaseProductDecision, CapabilityFamily, CommercialTreatment,
    CompositeGrant, DecisionReason, EmbeddedEntitlementPolicyRegistry,
    EntitlementPolicyPosture, EntitlementPolicyRegistryError, EntitlementPolicyTypeError,
    EntitlementStateDecision, LicenseTypeCode, LicenseTypeGrant,
    LicenseTypeVersion, LimitBucket, OperationClass, OperatorFamilyInheritanceDecision,
    OperatorSeats, PolicyActivation,
    PolicyEntitlementState, PremiumFamilyDecision, PremiumFamilyDenial, ProductCode,
    RecoveryAllowance, RequiredFeature, ResolvedEntitlementPolicy, ResourceRight, SaleStatus,
    SecurityPrerequisite, SharedNodeLimit, AUTOMATION_PREMIUM_FEATURE_IDS,
    BASE_PRODUCT_CORE_COMPATIBILITY_IDS,
    CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS,
    PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
    RELEASE_PROOF_PREMIUM_FEATURE_IDS,
    SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    TEAM_REMOTE_PREMIUM_FEATURE_IDS,
};
pub use capsule_manifest::{
    canonical_capsule_manifest_bytes, capsule_manifest_sha256, verify_capsule_manifest,
    CapsuleDigest, CapsuleDigests, CapsuleManifest, CapsuleProvenance, CapsuleRevocation,
    CapsuleSignature, CapsuleVerificationDecision, CapsuleVerificationFacts, KeyEnvelopeRef,
    PublicShellContract, TrustedSignerKey, CAPSULE_MANIFEST_SCHEMA, CAPSULE_MANIFEST_VERSION,
    CAPSULE_SIGNATURE_ALGORITHM, KNOWN_LIMIT_POLICY_VERSION, NODE_KEY_ENVELOPE_SCHEMA,
    REGISTERED_CAPSULE_ARCHES, REGISTERED_CAPSULE_CHANNELS, REGISTERED_CAPSULE_PLATFORMS,
    REGISTERED_CAPSULE_RELEASE_STATUSES,
};
pub use dynamic_operation_manifest::{
    verify_dynamic_operation_manifest, verify_generated_ui_action,
    CanonicalManifestFacts, DynamicOperationManifest, ManifestQuarantineLedger,
    ManifestTrustDecision, QuarantinedManifestRecord,
    ENTITLEMENT_POLICY_UNKNOWN, FORBIDDEN_CLIENT_POLICY_FIELDS,
    REGISTERED_OPERATION_CLASSES, REGISTERED_PRODUCT_OWNERS,
    REGISTERED_SIDE_EFFECT_CLASSES,
};
pub use facade_policy_presenter::{
    facade_family, facade_next_action_for_posture, facade_next_action_for_status,
    safe_masked_status, FacadeFamily, FacadeMaskedStatus, FacadeNextAction,
    FacadePolicyDecision, FacadePresenterError, FACADE_ALWAYS_REACHABLE,
    FACADE_PRESENTER_FIELDS, FACADE_PRESENTER_FORBIDDEN_FIELDS,
    FACADE_STATUS_ALLOWLIST,
};
pub use denial_ux::{
    blocked_action_for_family, available_reason_for_family,
    denial_ux_action_label, denial_ux_link,
    denial_ux_message_for, denial_ux_message_for_code,
    embedded_denial_ux_catalog,
    ACTION_LABEL_MANAGE_CAPACITY, ACTION_LABEL_REFRESH_DIAGNOSTICS,
    ACTION_LABEL_RETRY_DIAGNOSTICS, ACTION_LABEL_RETRY_IDENTIFIER,
    ACTION_LABELS, DENIAL_UX_ACTIONS, DENIAL_UX_CATALOG_JSON,
    DENIAL_UX_LINK_IDS, DENIAL_UX_SCHEMA, DenialUxError, DenialUxErrorCode,
    DenialUxErrorSpec, DenialUxKind, DenialUxMessage, LINK_ACCOUNT,
    LINK_CHECKOUT, LINK_EVALUATION, LINK_RECOVERY, MSG_BASE_REQUIRED,
    MSG_FEATURE_REQUIRED, MSG_IDEMPOTENCY_REQUIRED, MSG_LIMIT_EXHAUSTED,
    MSG_POLICY_UNKNOWN, MSG_RECOVERY_ONLY, MSG_REQUIRED,
    MSG_RESERVATION_FAILED, MSG_ROUTE_UNCLASSIFIED, MSG_SNAPSHOT_MISSING,
    PUBLIC_MESSAGE_RULES, RETAINED_ACCESS,
};
pub mod bundle_activation;
pub mod uiai_activation;
pub mod uiai_child_token;
pub mod limit_reservation;

pub use limit_reservation::{
    declared_server_owned_limit_buckets, family_limit_buckets,
    AUTOMATION_LIMIT_BUCKETS, CUSTOMER_DATA_EXPORT_LIMIT_BUCKETS,
    DECLARED_SERVER_OWNED_LIMIT_BUCKETS, PREMIUM_UPDATES_LIMIT_BUCKETS,
    RELEASE_PROOF_LIMIT_BUCKETS, TEAM_REMOTE_LIMIT_BUCKETS,
    LimitReservationService, ReservationError, ReservationGrant, ReservationScope,
};

pub use observability::EntitlementDecisionCounters;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Canonical, client-safe projection of one immutable signed entitlement snapshot.
/// Core, REST, and CLI surfaces must serialize this type instead of rebuilding fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementProjection {
    pub schema: String,
    pub state: String,
    pub product: String,
    pub node_id: String,
    pub lease_id: Option<String>,
    pub lease_sequence: Option<u64>,
    pub lease_digest: Option<String>,
    pub expires_at: Option<String>,
    pub offline_grace_until: Option<String>,
    pub features: BTreeMap<String, bool>,
    pub limits: BTreeMap<String, u64>,
    pub recovery_reason: Option<String>,
}

/// Project a signed authority snapshot without consulting legacy files, environment
/// promotion, feature inference, or caller-specific defaults.
pub fn entitlement_projection(
    snapshot: Option<&authority::EntitlementSnapshot>,
) -> Result<EntitlementProjection, LicenseError> {
    let snapshot = snapshot.ok_or(LicenseError::EntitlementSnapshotMissing)?;
    Ok(EntitlementProjection {
        schema: "focusa.entitlement_projection.v1".to_string(),
        state: format!("{:?}", snapshot.state).to_ascii_lowercase(),
        product: snapshot.product.clone(),
        node_id: snapshot.node_id.clone(),
        lease_id: snapshot.lease_id.clone(),
        lease_sequence: snapshot.sequence,
        lease_digest: snapshot.lease_digest.clone(),
        expires_at: snapshot.expires_at.map(|value| value.to_rfc3339()),
        offline_grace_until: snapshot.offline_grace_until.map(|value| value.to_rfc3339()),
        features: snapshot.features.clone(),
        limits: snapshot.limits.clone(),
        recovery_reason: snapshot.recovery_reason.clone(),
    })
}

/// Canonical, presenter-neutral entitlement decision projection for status-style
/// presenters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementDecisionProjection {
    pub status: String,
    pub entitlement_state: String,
    pub operation_id: String,
    pub operation_class: String,
    pub capability_family: String,
    pub commercial_treatment: String,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
    pub reason_code: String,
    pub recovery_action: String,
    pub policy_digest: String,
    pub lease_sequence: u64,
}

pub const LICENSE_STATUS_OPERATION_ID: &str = "focusa.license.status";

/// Project one redacted entitlement decision for the canonical license status view.
///
/// This projection intentionally excludes raw tokens, lease digests, and identity
/// claims while preserving enough context for presenter rendering and recovery UX.
pub fn entitlement_decision_projection(
    snapshot: Option<&authority::EntitlementSnapshot>,
) -> Result<EntitlementDecisionProjection, LicenseError> {
    let snapshot = snapshot.ok_or(LicenseError::EntitlementSnapshotMissing)?;
    let policy_state = authority_policy_state(snapshot);
    let decision = reduce_entitlement_state(policy_state, CapabilityFamily::ReadProjection, None);

    Ok(EntitlementDecisionProjection {
        status: decision.posture().status().to_string(),
        entitlement_state: policy_state.label().to_string(),
        operation_id: LICENSE_STATUS_OPERATION_ID.to_string(),
        operation_class: OperationClass::Read.label().to_string(),
        capability_family: CapabilityFamily::ReadProjection.label().to_string(),
        commercial_treatment: CapabilityFamily::ReadProjection
            .commercial_treatment()
            .label()
            .to_string(),
        required_feature: None,
        limit_bucket: None,
        reason_code: decision.reason().label().to_string(),
        recovery_action: decision.reason().recovery_action().to_string(),
        policy_digest: embedded_entitlement_policy_registry()
            .expect("embedded entitlement policy registry")
            .digest()
            .to_string(),
        lease_sequence: snapshot.sequence.unwrap_or_default(),
    })
}

/// Canonical, client-safe base-product projection derived from one signed
/// entitlement snapshot (Spec 152F P3). REST, CLI, desktop, TUI, Pi, agents,
/// installers, UIAI, workers, and schedulers inherit this single decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseProductProjection {
    pub schema: String,
    pub product: String,
    pub decision: String,
    pub permits_base_mutations: bool,
    pub compatibility: BTreeMap<String, bool>,
}

/// Project the canonical base Focusa product gate from a signed authority
/// snapshot. Legacy core identifiers are reported through the compatibility
/// projection and resolve as base-product claims, never separate purchases.
pub fn base_product_projection(
    snapshot: Option<&authority::EntitlementSnapshot>,
) -> Result<BaseProductProjection, LicenseError> {
    let snapshot = snapshot.ok_or(LicenseError::EntitlementSnapshotMissing)?;
    let policy_state = authority_policy_state(snapshot);
    let decision = resolve_base_focusa_product(&snapshot.product, policy_state);
    let compatibility = base_product_compatibility_projection(decision, &snapshot.features);
    Ok(BaseProductProjection {
        schema: "focusa.base_product_projection.v1".to_string(),
        product: snapshot.product.clone(),
        decision: decision.label().to_string(),
        permits_base_mutations: decision.permits_base_mutations(),
        compatibility,
    })
}

/// License tiers supported by Focusa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Unactivated,
    RecoveryOnly,
    Entitled,
    OfflineGrace,
    Eval,
    Licensed,
    Open,
}

impl Tier {
    pub fn permits_commercial_use(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn permits_hosted_deployment(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn permits_local_eval(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn label(self) -> &'static str {
        match self {
            Tier::Unactivated => "unactivated",
            Tier::RecoveryOnly => "recovery_only",
            Tier::Entitled => "entitled",
            Tier::OfflineGrace => "offline_grace",
            Tier::Eval => "eval",
            Tier::Licensed => "licensed",
            Tier::Open => "open",
        }
    }
}

/// Capabilities that LicenseGuard gates. Each capability is a static enum member
/// so call sites can do `guard.require(Capability::HostedMode)` and get a typed
/// error rather than a stringly-typed allowlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Process / orchestrate commercial workloads.
    CommercialUse,
    /// Operate as a hosted multi-tenant daemon.
    HostedMode,
    /// Embed Focusa inside a commercial product.
    ProductEmbedding,
    /// Send telemetry/analytics events (Focusa is no-telemetry by default).
    TelemetrySend,
    /// Local-only single-user use, free for everyone.
    LocalEval,
}

impl Capability {
    pub fn label(self) -> &'static str {
        match self {
            Capability::CommercialUse => "commercial_use",
            Capability::HostedMode => "hosted_mode",
            Capability::ProductEmbedding => "product_embedding",
            Capability::TelemetrySend => "telemetry_send",
            Capability::LocalEval => "local_eval",
        }
    }
}

/// Outcome of a capability check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum CapabilityCheck {
    /// Capability is permitted under current tier.
    Permitted,
    /// Capability is permitted but with a soft warning (e.g., eval tier + commercial).
    PermittedWithWarning { warning: String },
    /// Capability is denied under current tier (hard fail).
    Denied { reason: String },
}

impl CapabilityCheck {
    pub fn is_permitted(&self) -> bool {
        !matches!(self, CapabilityCheck::Denied { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, CapabilityCheck::Denied { .. })
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            CapabilityCheck::Denied { reason } => Some(reason),
            CapabilityCheck::PermittedWithWarning { warning } => Some(warning),
            CapabilityCheck::Permitted => None,
        }
    }
}

/// LicenseGuard evaluates a Tier against Capabilities and decides permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseGuard {
    pub tier: Tier,
    pub key_hash: Option<String>,
    pub customer_email: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub bsl_change_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entitlement: Option<authority::EntitlementSnapshot>,
}

impl LicenseGuard {
    /// Construct an Eval guard (self-issued, no key, offline grace window).
    pub fn eval(duration_days: i64) -> Self {
        let now = Utc::now();
        Self {
            tier: Tier::Eval,
            key_hash: None,
            customer_email: None,
            issued_at: now,
            expires_at: Some(now + chrono::Duration::days(duration_days)),
            bsl_change_date: bsl_change_date(),
            entitlement: None,
        }
    }

    /// Construct a Licensed guard (key-required).
    pub fn licensed(key_hash: String, customer_email: String) -> Self {
        let now = Utc::now();
        Self {
            tier: Tier::Licensed,
            key_hash: Some(key_hash),
            customer_email: Some(customer_email),
            issued_at: now,
            expires_at: None,
            bsl_change_date: bsl_change_date(),
            entitlement: None,
        }
    }

    pub fn from_entitlement(entitlement: authority::EntitlementSnapshot) -> Self {
        let tier = match entitlement.state {
            authority::EntitlementState::Unactivated => Tier::Unactivated,
            authority::EntitlementState::RecoveryOnly => Tier::RecoveryOnly,
            authority::EntitlementState::Active => Tier::Entitled,
            authority::EntitlementState::OfflineGrace => Tier::OfflineGrace,
        };
        Self {
            tier,
            key_hash: entitlement.lease_digest.clone(),
            customer_email: None,
            issued_at: Utc::now(),
            expires_at: entitlement.expires_at,
            bsl_change_date: bsl_change_date(),
            entitlement: Some(entitlement),
        }
    }

    /// Returns true if the authority lease or legacy evaluation has expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(e) => Utc::now() > e,
            None => false,
        }
    }

    /// Check a capability only against the immutable signed entitlement snapshot.
    pub fn check(&self, capability: Capability) -> CapabilityCheck {
        let Some(entitlement) = &self.entitlement else {
            return CapabilityCheck::Denied {
                reason: "signed authority entitlement required; legacy tier is migration-only"
                    .into(),
            };
        };
        if entitlement.feature_enabled(capability.label()) {
            CapabilityCheck::Permitted
        } else {
            CapabilityCheck::Denied {
                reason: format!(
                    "authority entitlement state={} does not grant {}",
                    self.tier.label(),
                    capability.label()
                ),
            }
        }
    }

    /// Hard-require a capability. Returns Ok(()) when permitted (possibly with warning
    /// in caller-routed logs), Err(LicenseError::Denied{..}) when denied.
    pub fn require(&self, capability: Capability) -> Result<Option<String>, LicenseError> {
        match self.check(capability) {
            CapabilityCheck::Permitted => Ok(None),
            CapabilityCheck::PermittedWithWarning { warning } => Ok(Some(warning)),
            CapabilityCheck::Denied { reason } => Err(LicenseError::Denied {
                capability: capability.label().into(),
                tier: self.tier.label().into(),
                reason,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LicenseError {
    #[error("license denied: tier={tier} does not permit {capability}: {reason}")]
    Denied {
        capability: String,
        tier: String,
        reason: String,
    },
    #[error("ENTITLEMENT_SNAPSHOT_MISSING: signed authority entitlement snapshot required")]
    EntitlementSnapshotMissing,
}

/// BSL change date placeholder (4 years from typical release cadence).
/// Per operator rule 2026-07-08: Update when BSL change date is finalized.
fn bsl_change_date() -> DateTime<Utc> {
    // Hardcoded safe default. Real release uses release pipeline.
    chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Resolve a LicenseGuard only from signed, persisted authority state.
pub fn resolve_license_guard() -> LicenseGuard {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    resolve_license_guard_from(
        &home.join(".config/focusa"),
        authority_store::embedded_production_trust_roots(),
        Utc::now(),
    )
}

pub fn resolve_license_guard_from(
    config_dir: &Path,
    roots: Result<
        std::collections::BTreeMap<String, ed25519_dalek::VerifyingKey>,
        authority_store::AuthorityStoreError,
    >,
    now: DateTime<Utc>,
) -> LicenseGuard {
    let state_path = config_dir.join(authority_store::AUTHORITY_STATE_FILE);
    let expected_node_id = std::fs::read_to_string(config_dir.join("node-id"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            authority_credentials::load_or_create_node_identity(config_dir, "focusa")
                .ok()
                .map(|identity| identity.node_id)
        })
        .unwrap_or_else(|| "unbound".to_string());
    let context = authority::LeaseVerificationContext {
        expected_product: "focusa".to_string(),
        expected_node_id,
        now,
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    LicenseGuard::from_entitlement(authority_store::resolve_authority_state(
        &state_path,
        roots,
        &context,
    ))
}

/// Read ~/.config/focusa/license.json and construct a guard.
fn read_license_json() -> Option<LicenseGuard> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".config/focusa/license.json");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some(LicenseGuard {
        tier: parse_tier(json.get("tier")?.as_str()?)?,
        key_hash: json
            .get("key_hash")
            .and_then(|v| v.as_str())
            .map(String::from),
        customer_email: json
            .get("customer_email")
            .and_then(|v| v.as_str())
            .map(String::from),
        issued_at: parse_iso(json.get("issued_at")?.as_str()?)?,
        expires_at: json
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(parse_iso),
        bsl_change_date: parse_iso(json.get("bsl_change_date")?.as_str()?)
            .unwrap_or_else(bsl_change_date),
        entitlement: None,
    })
}

/// Read ~/.focusa/license.toml (per-project override) and construct a guard.
fn read_license_toml() -> Option<LicenseGuard> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".focusa/license.toml");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let table: toml::Value = toml::from_str(&raw).ok()?;
    Some(LicenseGuard {
        tier: parse_tier(table.get("tier")?.as_str()?)?,
        key_hash: table
            .get("key_hash")
            .and_then(|v| v.as_str())
            .map(String::from),
        customer_email: table
            .get("customer_email")
            .and_then(|v| v.as_str())
            .map(String::from),
        issued_at: parse_iso(table.get("issued_at")?.as_str()?)?,
        expires_at: table
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(parse_iso),
        bsl_change_date: parse_iso(table.get("bsl_change_date")?.as_str()?)
            .unwrap_or_else(bsl_change_date),
        entitlement: None,
    })
}

fn parse_tier(s: &str) -> Option<Tier> {
    match s.trim().to_ascii_lowercase().as_str() {
        "eval" => Some(Tier::Eval),
        "licensed" => Some(Tier::Licensed),
        "open" => Some(Tier::Open),
        _ => None,
    }
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Short SHA256 fingerprint (first 16 hex chars) of a license key, for logs.
pub fn sha256_short(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    format!("{:016x}", v)
}

/// Write a license record to ~/.config/focusa/license.json (used by installer).
pub fn persist_eval_license(home: &Path) -> std::io::Result<LicenseGuard> {
    let dir = home.join(".config/focusa");
    std::fs::create_dir_all(&dir)?;
    let guard = LicenseGuard::eval(7);
    let json = serde_json::to_string_pretty(&guard).map_err(std::io::Error::other)?;
    std::fs::write(dir.join("license.json"), json)?;
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec172_policy_types_reject_noncanonical_grants_and_aliases() {
        for alias in ["evaluation", "eval", "focusa_evaluation"] {
            assert!(serde_json::from_str::<AccessPosture>(&format!("\"{alias}\"")).is_err());
        }
        assert!(serde_json::from_str::<LicenseTypeCode>("\"caller_created_v1\"").is_err());
        assert!(serde_json::from_str::<ProductCode>("\"unknown_product\"").is_err());
        assert!(serde_json::from_str::<OperatorSeats>("\"two\"").is_err());

        let mut cross_product = LicenseTypeGrant::focusa_operator_v1();
        cross_product.product = ProductCode::UiaiEngine;
        assert_eq!(
            cross_product.validate(),
            Err(EntitlementPolicyTypeError::InvalidLicenseTypeGrant)
        );

        let focusa = LicenseTypeGrant::focusa_operator_v1();
        let uiai = LicenseTypeGrant::uiai_operator_v1();
        let bundle = CompositeGrant::operator_bundle_v1([focusa, uiai]).expect("canonical Bundle");
        assert_eq!(bundle.grants(), &[focusa, uiai]);
        assert_eq!(
            CompositeGrant::operator_bundle_v1([focusa, focusa]),
            Err(EntitlementPolicyTypeError::MalformedBundleUnion)
        );
        assert_eq!(
            CompositeGrant::operator_bundle_v1([uiai, focusa]),
            Err(EntitlementPolicyTypeError::MalformedBundleUnion)
        );
    }

    #[test]
    fn canonical_entitlement_projection_preserves_authority_fields() {
        let mut snapshot = authority::EntitlementSnapshot::unactivated("focusa", "node-001");
        snapshot.state = authority::EntitlementState::Active;
        snapshot.lease_id = Some("lease-001".to_string());
        snapshot.sequence = Some(42);
        snapshot.lease_digest = Some("sha256:lease".to_string());
        snapshot.features.insert("agent_runtime".to_string(), true);
        snapshot.limits.insert("active_sessions".to_string(), 4);
        snapshot.recovery_reason = None;

        let projection = entitlement_projection(Some(&snapshot)).expect("projection");
        assert_eq!(projection.state, "active");
        assert_eq!(projection.lease_id.as_deref(), Some("lease-001"));
        assert_eq!(projection.lease_sequence, Some(42));
        assert_eq!(projection.lease_digest.as_deref(), Some("sha256:lease"));
        assert_eq!(projection.features.get("agent_runtime"), Some(&true));
        assert_eq!(projection.limits.get("active_sessions"), Some(&4));
    }

    #[test]
    fn canonical_entitlement_projection_fails_closed_without_snapshot() {
        assert!(matches!(
            entitlement_projection(None),
            Err(LicenseError::EntitlementSnapshotMissing)
        ));
    }

    #[test]
    fn entitlement_decision_projection_projects_status_read_envelope() {
        let mut snapshot = authority::EntitlementSnapshot::unactivated("focusa", "node-001");
        snapshot.state = authority::EntitlementState::Active;
        snapshot.sequence = Some(42);

        let decision = entitlement_decision_projection(Some(&snapshot))
            .expect("decision projection");

        assert_eq!(decision.status, "read");
        assert_eq!(decision.entitlement_state, "active_paid");
        assert_eq!(decision.operation_id, LICENSE_STATUS_OPERATION_ID);
        assert_eq!(decision.operation_class, "read");
        assert_eq!(decision.capability_family, "read_projection");
        assert_eq!(decision.commercial_treatment, "read_allowance");
        assert_eq!(decision.reason_code, "read");
        assert_eq!(decision.recovery_action, "license_status");
        assert_eq!(decision.lease_sequence, 42);

        let body = serde_json::to_string(&decision).unwrap();
        assert!(!body.contains("lease_digest"));
        assert!(!body.contains("lease_id"));
        assert!(!body.contains("node_id"));
        assert_eq!(
            decision.policy_digest,
            embedded_entitlement_policy_registry().expect("registry").digest()
        );
    }

    #[test]
    fn entitlement_decision_projection_fails_closed_without_snapshot() {
        assert!(matches!(
            entitlement_decision_projection(None),
            Err(LicenseError::EntitlementSnapshotMissing)
        ));
    }

    #[test]
    fn entitlement_decision_projection_fails_closed_for_unactivated_snapshot() {
        let snapshot = authority::EntitlementSnapshot::unactivated("focusa", "node-001");
        let decision = entitlement_decision_projection(Some(&snapshot)).expect("decision");
        assert_eq!(decision.status, "deny");
        assert_eq!(decision.reason_code, "deny");
        assert_eq!(decision.recovery_action, "activate_evaluation_purchase_or_manage_entitlement");
        assert_eq!(decision.entitlement_state, "pending_unverified");
        assert_eq!(decision.lease_sequence, 0);
    }

    #[test]
    fn self_issued_eval_cannot_grant_local_eval() {
        let g = LicenseGuard::eval(7);
        assert!(g.check(Capability::LocalEval).is_denied());
    }

    #[test]
    fn self_issued_eval_cannot_grant_commercial_use() {
        let g = LicenseGuard::eval(7);
        assert!(g.check(Capability::CommercialUse).is_denied());
    }

    #[test]
    fn eval_tier_denies_hosted_mode() {
        let g = LicenseGuard::eval(7);
        let c = g.check(Capability::HostedMode);
        assert!(c.is_denied());
    }

    #[test]
    fn eval_tier_denies_product_embedding() {
        let g = LicenseGuard::eval(7);
        let c = g.check(Capability::ProductEmbedding);
        assert!(c.is_denied());
    }

    #[test]
    fn plaintext_licensed_tier_cannot_grant_commercial_use() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert!(g.check(Capability::CommercialUse).is_denied());
    }

    #[test]
    fn plaintext_licensed_tier_cannot_grant_hosted_mode() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert!(g.check(Capability::HostedMode).is_denied());
    }

    #[test]
    fn plaintext_open_tier_cannot_grant_capabilities() {
        let g = LicenseGuard {
            tier: Tier::Open,
            key_hash: None,
            customer_email: None,
            issued_at: Utc::now(),
            expires_at: None,
            bsl_change_date: bsl_change_date(),
            entitlement: None,
        };
        assert!(g.check(Capability::CommercialUse).is_denied());
        assert!(g.check(Capability::HostedMode).is_denied());
        assert!(g.check(Capability::ProductEmbedding).is_denied());
    }

    #[test]
    fn require_denies_without_signed_entitlement() {
        let g = LicenseGuard::eval(7);
        assert!(g.require(Capability::CommercialUse).is_err());
        assert!(g.require(Capability::HostedMode).is_err());
    }

    #[test]
    fn tier_label_round_trip() {
        assert_eq!(Tier::Eval.label(), "eval");
        assert_eq!(Tier::Licensed.label(), "licensed");
        assert_eq!(Tier::Open.label(), "open");
        for t in [
            Tier::Unactivated,
            Tier::RecoveryOnly,
            Tier::Entitled,
            Tier::OfflineGrace,
            Tier::Eval,
            Tier::Licensed,
            Tier::Open,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: Tier = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn base_product_resolution_one_signed_entitlement_gates_base_focusa() {
        // Active paid lease for product=focusa is a usable base entitlement.
        assert_eq!(
            resolve_base_focusa_product("focusa", PolicyEntitlementState::ActivePaid),
            BaseProductDecision::Entitled
        );
        assert!(BaseProductDecision::Entitled.permits_base_mutations());
        // Valid Offline Grace also gates the base product.
        assert_eq!(
            resolve_base_focusa_product("focusa", PolicyEntitlementState::OfflineGrace),
            BaseProductDecision::Entitled
        );
        // Verified but license-less is limited to the explicit manual one-project subset.
        assert_eq!(
            resolve_base_focusa_product("focusa", PolicyEntitlementState::VerifiedNoLicense),
            BaseProductDecision::Limited
        );
        assert!(!BaseProductDecision::Limited.permits_base_mutations());
        // Every other state denies value-producing mutations.
        for state in [
            PolicyEntitlementState::PendingUnverified,
            PolicyEntitlementState::Expired,
            PolicyEntitlementState::RefundedOrRevoked,
            PolicyEntitlementState::MissingOrCorrupt,
        ] {
            assert_eq!(
                resolve_base_focusa_product("focusa", state),
                BaseProductDecision::Denied
            );
        }
        // Wrong product is never the base gate.
        assert_eq!(
            resolve_base_focusa_product("uiai-engine", PolicyEntitlementState::ActivePaid),
            BaseProductDecision::Denied
        );
        assert_eq!(
            resolve_base_focusa_product("focusa-premium", PolicyEntitlementState::ActivePaid),
            BaseProductDecision::Denied
        );
        // Product identity is exact; normalization cannot turn caller input
        // into an authority-owned Focusa product grant.
        for product in ["FOCUSA", " focusa", "focusa ", "focusa/operator"] {
            assert_eq!(
                resolve_base_focusa_product(product, PolicyEntitlementState::ActivePaid),
                BaseProductDecision::Denied,
                "non-canonical product identity must fail closed: {product:?}"
            );
        }
    }

    #[test]
    fn base_product_resolution_requires_one_not_three_separate_features() {
        // Base capability does not require separately purchased core features:
        // with an ActivePaid product=focusa lease, the base gate is satisfied even
        // when legacy core identifiers are absent or individually false.
        let empty = BTreeMap::new();
        let decision = resolve_base_focusa_product("focusa", PolicyEntitlementState::ActivePaid);
        assert!(decision.permits_base_mutations());
        let projection = base_product_compatibility_projection(decision, &empty);
        for id in BASE_PRODUCT_CORE_COMPATIBILITY_IDS {
            assert_eq!(projection.get(id), Some(&true), "{id} resolves as base product");
        }

        // Stored false values are non-authoritative projection claims; the base
        // decision governs, never a separate feature purchase.
        let mut stored = BTreeMap::new();
        stored.insert("focusa.core.mission".to_string(), false);
        stored.insert("focusa.core.workpoint".to_string(), false);
        stored.insert("focusa.core.evidence".to_string(), false);
        let projection = base_product_compatibility_projection(decision, &stored);
        assert_eq!(projection.get("focusa.core.mission"), Some(&true));
        assert_eq!(projection.get("focusa.core.workpoint"), Some(&true));
        assert_eq!(projection.get("focusa.core.evidence"), Some(&true));

        // Denied base never projects core identifiers as granted, and never
        // manufactures a purchase from stored values.
        let denied = resolve_base_focusa_product("focusa", PolicyEntitlementState::Expired);
        let projection = base_product_compatibility_projection(denied, &stored);
        assert_eq!(projection.get("focusa.core.mission"), Some(&false));
        assert_eq!(projection.get("focusa.core.workpoint"), Some(&false));
        assert_eq!(projection.get("focusa.core.evidence"), Some(&false));
    }

    #[test]
    fn base_product_resolution_snapshot_projection_is_canonical_and_fails_closed() {
        let mut snapshot = authority::EntitlementSnapshot::unactivated("focusa", "node-001");
        snapshot.state = authority::EntitlementState::Active;
        snapshot.lease_id = Some("lease-base-001".to_string());
        snapshot.sequence = Some(7);
        let projection = base_product_projection(Some(&snapshot)).expect("projection");
        assert_eq!(projection.schema, "focusa.base_product_projection.v1");
        assert_eq!(projection.product, "focusa");
        assert_eq!(projection.decision, "entitled");
        assert!(projection.permits_base_mutations);
        assert_eq!(projection.compatibility.get("focusa.core.mission"), Some(&true));

        // No snapshot fails closed.
        assert!(matches!(
            base_product_projection(None),
            Err(LicenseError::EntitlementSnapshotMissing)
        ));

        // Offline Grace is usable.
        snapshot.state = authority::EntitlementState::OfflineGrace;
        assert!(base_product_projection(Some(&snapshot)).unwrap().permits_base_mutations);

        // Recovery-only is not base entitlement.
        snapshot.state = authority::EntitlementState::RecoveryOnly;
        let projection = base_product_projection(Some(&snapshot)).unwrap();
        assert_eq!(projection.decision, "denied");
        assert!(!projection.permits_base_mutations);
    }
}
