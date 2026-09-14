use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    authority::{EntitlementSnapshot, EntitlementState},
    authority_client::SensitiveCredential,
    entitlement_policy::{
        SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
        SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    },
};

pub const UIAI_CHILD_TOKEN_SCHEMA: &str = "focusa.uiai_child_token.v1";
pub const UIAI_CHILD_TOKEN_MAX_TTL_MINUTES: i64 = 15;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiChildTokenRequest {
    pub request_id: Uuid,
    pub audience: String,
    pub node_id: String,
    pub client_id: String,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    pub requested_features: BTreeSet<String>,
    pub requested_limits: BTreeMap<String, u64>,
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityChildTokenEnvelope {
    pub schema: String,
    pub token: String,
    pub token_id: String,
    pub audience: String,
    pub node_id: String,
    pub client_id: String,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    pub features: BTreeSet<String>,
    pub limits: BTreeMap<String, u64>,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct CachedUiaiChildToken {
    pub token_id: String,
    pub credential: SensitiveCredential,
    pub node_id: String,
    pub parent_lease_id: String,
    pub parent_lease_sequence: u64,
    pub parent_lease_digest: String,
    pub uiai_grant_lease_id: String,
    pub uiai_grant_sequence: u64,
    /// The exact feature subset the authority issued this token with. Stored
    /// so a later lookup can reject a WIDENED token: if the current grant no
    /// longer contains every cached feature, the token is refused even though
    /// it is still within its TTL.
    pub features: BTreeSet<String>,
    pub limits: BTreeMap<String, u64>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiChildTokenReceipt {
    pub schema: String,
    pub token_id: String,
    pub request_id: Uuid,
    pub audience: String,
    pub parent_lease_sequence: u64,
    pub uiai_grant_sequence: u64,
    pub feature_count: usize,
    pub limit_count: usize,
    pub expires_at: DateTime<Utc>,
    pub token_persisted_in_receipt: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UiaiChildTokenError {
    #[error("Focusa parent entitlement is not active and bound")]
    ParentEntitlementInvalid,
    #[error("independent UIAI product entitlement is not active and bound")]
    UiaiGrantInvalid,
    #[error("Focusa parent and UIAI grant are not bound to the same EDD account")]
    AccountMismatch,
    #[error("requested child scope is not an exact subset of the UIAI grant")]
    ScopeNotGranted,
    #[error("authority response does not match request/parent authority")]
    AuthorityResponseMismatch,
    #[error("authority child token expiry is invalid")]
    InvalidExpiry,
    #[error("authority child token is missing")]
    TokenMissing,
    #[error("nonce has already been accepted")]
    NonceReplay,
}

#[derive(Default)]
pub struct UiaiChildTokenBroker {
    cache: BTreeMap<String, CachedUiaiChildToken>,
    accepted_nonces: BTreeSet<String>,
}

impl UiaiChildTokenBroker {
    pub fn validate_request(
        &self,
        request: &UiaiChildTokenRequest,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
        now: DateTime<Utc>,
    ) -> Result<(), UiaiChildTokenError> {
        if !active_bound(focusa_parent, "focusa", &request.node_id, now)
            || focusa_parent.lease_id.as_deref() != Some(request.parent_lease_id.as_str())
            || focusa_parent.sequence != Some(request.parent_lease_sequence)
            || focusa_parent.lease_digest.as_deref() != Some(request.parent_lease_digest.as_str())
        {
            return Err(UiaiChildTokenError::ParentEntitlementInvalid);
        }
        if !active_bound(uiai_grant, "uiai-engine", &request.node_id, now)
            || uiai_grant.lease_id.as_deref() != Some(request.uiai_grant_lease_id.as_str())
            || uiai_grant.sequence != Some(request.uiai_grant_sequence)
        {
            return Err(UiaiChildTokenError::UiaiGrantInvalid);
        }
        // Same EDD account: the Focusa parent and the independent UIAI grant
        // must be issued to the same account (Spec 152E §7/§15). Verified
        // leases always carry the authority-issued subject; when one side
        // proves an account and the other does not (or differs), the child
        // token fails closed rather than bridging two customers.
        if !same_evidence_account(focusa_parent, uiai_grant) {
            return Err(UiaiChildTokenError::AccountMismatch);
        }
        if request.audience.trim().is_empty()
            || request.client_id.trim().is_empty()
            || request.nonce.trim().is_empty()
            || self.accepted_nonces.contains(&request.nonce)
        {
            return Err(UiaiChildTokenError::NonceReplay);
        }
        if !request
            .requested_features
            .iter()
            .all(|feature| uiai_grant.features.get(feature).copied().unwrap_or(false))
            || request.requested_limits.iter().any(|(bucket, requested)| {
                *requested == 0 || *requested > uiai_grant.limits.get(bucket).copied().unwrap_or(0)
            })
        {
            return Err(UiaiChildTokenError::ScopeNotGranted);
        }
        Ok(())
    }

    pub fn accept_authority_token(
        &mut self,
        request: &UiaiChildTokenRequest,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
        envelope: AuthorityChildTokenEnvelope,
        now: DateTime<Utc>,
    ) -> Result<UiaiChildTokenReceipt, UiaiChildTokenError> {
        self.validate_request(request, focusa_parent, uiai_grant, now)?;
        if envelope.schema != UIAI_CHILD_TOKEN_SCHEMA
            || envelope.audience != request.audience
            || envelope.node_id != request.node_id
            || envelope.client_id != request.client_id
            || envelope.parent_lease_id != request.parent_lease_id
            || envelope.parent_lease_sequence != request.parent_lease_sequence
            || envelope.parent_lease_digest != request.parent_lease_digest
            || envelope.uiai_grant_lease_id != request.uiai_grant_lease_id
            || envelope.uiai_grant_sequence != request.uiai_grant_sequence
            || envelope.features != request.requested_features
            || envelope.limits != request.requested_limits
            || envelope.nonce != request.nonce
        {
            return Err(UiaiChildTokenError::AuthorityResponseMismatch);
        }
        let parent_bound = entitlement_bound(focusa_parent).unwrap_or(now);
        let uiai_bound = entitlement_bound(uiai_grant).unwrap_or(now);
        if envelope.issued_at > now
            || envelope.expires_at <= now
            || envelope.expires_at > now + Duration::minutes(UIAI_CHILD_TOKEN_MAX_TTL_MINUTES)
            || envelope.expires_at > parent_bound
            || envelope.expires_at > uiai_bound
        {
            return Err(UiaiChildTokenError::InvalidExpiry);
        }
        let credential = SensitiveCredential::new(envelope.token)
            .map_err(|_| UiaiChildTokenError::TokenMissing)?;
        let receipt = UiaiChildTokenReceipt {
            schema: "focusa.uiai_child_token_receipt.v1".into(),
            token_id: envelope.token_id.clone(),
            request_id: request.request_id,
            audience: request.audience.clone(),
            parent_lease_sequence: request.parent_lease_sequence,
            uiai_grant_sequence: request.uiai_grant_sequence,
            feature_count: request.requested_features.len(),
            limit_count: request.requested_limits.len(),
            expires_at: envelope.expires_at,
            token_persisted_in_receipt: false,
        };
        self.accepted_nonces.insert(request.nonce.clone());
        self.cache.insert(
            request.audience.clone(),
            CachedUiaiChildToken {
                token_id: envelope.token_id,
                credential,
                node_id: envelope.node_id.clone(),
                parent_lease_id: request.parent_lease_id.clone(),
                parent_lease_sequence: request.parent_lease_sequence,
                parent_lease_digest: request.parent_lease_digest.clone(),
                uiai_grant_lease_id: request.uiai_grant_lease_id.clone(),
                uiai_grant_sequence: request.uiai_grant_sequence,
                features: envelope.features.clone(),
                limits: envelope.limits.clone(),
                expires_at: envelope.expires_at,
            },
        );
        Ok(receipt)
    }

    pub fn cached(&self, audience: &str, now: DateTime<Utc>) -> Option<&CachedUiaiChildToken> {
        self.cache
            .get(audience)
            .filter(|token| token.expires_at > now)
    }

    pub fn revoke_parent(&mut self, lease_id: &str, minimum_sequence: u64) -> usize {
        let before = self.cache.len();
        self.cache.retain(|_, token| {
            token.parent_lease_id != lease_id || token.parent_lease_sequence >= minimum_sequence
        });
        before - self.cache.len()
    }

    /// Strict same-account binding (Spec 152E §7 / §15): the Focusa parent
    /// and the independent UIAI grant must both be issued to the single
    /// verified EDD account. The same-EDD-account UIAI activation adapter
    /// calls this before it accepts a child token; it fails closed unless
    /// both lease subjects equal the account id.
    pub fn validate_same_account_binding(
        &self,
        account: &crate::uiai_activation::UiaiAccountIdentity,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
    ) -> Result<(), UiaiChildTokenError> {
        if !crate::uiai_activation::same_account_binding(account, focusa_parent, uiai_grant) {
            return Err(UiaiChildTokenError::AccountMismatch);
        }
        Ok(())
    }

    /// Revalidate a cached child token against the CURRENT authority
    /// snapshots before a browser/proxy/MCP operation may use it (Spec 172
    /// §20.9 stale-client bypass fail-closed; Spec 152F §7 UIAI/browser
    /// adapter row). Rejects:
    /// - stale parents: the cached parent lease id/sequence/digest must still
    ///   equal the current Focusa parent, and the parent must remain
    ///   active/bound to the same node;
    /// - revoked or stale grants: the cached UIAI grant lease id/sequence must
    ///   still equal the current grant, and the grant must remain active;
    /// - widened tokens: every feature stored in the cached token must still
    ///   be an exact subset of the current grant feature allowlist.
    ///
    /// Pairing/device proof never appears here: only authority snapshots and
    /// the issued token can authorize a cached child token. The UIAI local
    /// task system (UIAI issue #5 integration) and browser/proxy/MCP adapters
    /// use this as the pre-execution gate.
    pub fn authorized_cached_token(
        &self,
        audience: &str,
        focusa_parent: &EntitlementSnapshot,
        uiai_grant: &EntitlementSnapshot,
        now: DateTime<Utc>,
    ) -> Option<&CachedUiaiChildToken> {
        let cached = self.cached(audience, now)?;
        if cached.parent_lease_id != focusa_parent.lease_id.as_deref().unwrap_or_default()
            || cached.parent_lease_sequence != focusa_parent.sequence.unwrap_or(0)
            || cached.parent_lease_digest
                != focusa_parent.lease_digest.as_deref().unwrap_or_default()
            || !active_bound(
                focusa_parent,
                crate::uiai_activation::PRODUCT_FOCUSA,
                &cached.node_id,
                now,
            )
        {
            return None;
        }
        if cached.uiai_grant_lease_id != uiai_grant.lease_id.as_deref().unwrap_or_default()
            || cached.uiai_grant_sequence != uiai_grant.sequence.unwrap_or(0)
            || !active_bound(
                uiai_grant,
                crate::uiai_activation::PRODUCT_UIAI_ENGINE,
                &cached.node_id,
                now,
            )
        {
            return None;
        }
        if !cached
            .features
            .iter()
            .all(|feature| uiai_grant.features.get(feature).copied().unwrap_or(false))
        {
            return None;
        }
        Some(cached)
    }
}

/// Canonical Spec 172 UIAI operation classification (Section 6.3).
///
/// Classifies a UIAI/browser operation as local base integration
/// (`PublicObservation`: provider-neutral public-web observation with bounded
/// local/approved capacity) versus remote/premium capability (`RemotePremium`:
/// click/fill/type/select/press browser mutation, persistence, unattended or
/// batch/scheduled automation, authenticated/private targets, and metered
/// hosted resources). The classification is authority metadata supplied by the
/// operation registry; it is never caller-selected and never grants capability
/// by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiaiOperationClass {
    PublicObservation,
    RemotePremium,
}

impl UiaiOperationClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PublicObservation => "public_observation",
            Self::RemotePremium => "remote_premium",
        }
    }
}

/// Canonical Spec 172 UIAI public-observe/action/persistence classification
/// (Section 6.3).
///
/// Finer-grained than [`UiaiOperationClass`]: classifies what a UIAI/browser
/// operation does to the web surface:
/// - `PublicObserve` — provider-neutral public-web observation (public search,
///   Source-to-Markdown, public page read, accessibility snapshot, screenshot,
///   basic diagnostics);
/// - `Action` — browser mutation (click, fill, type, select, press, submit),
///   authenticated/private-target workflows, unattended/batch automation, and
///   metered hosted/premium resources;
/// - `Persistence` — cookie, authentication-state, or long-lived session
///   persistence.
///
/// The classification is canonical authority metadata owned by the operation
/// map; it is never caller-selected and never grants capability by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiaiActionPersistenceClass {
    PublicObserve,
    Action,
    Persistence,
}

impl UiaiActionPersistenceClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PublicObserve => "public_observe",
            Self::Action => "action",
            Self::Persistence => "persistence",
        }
    }

    /// Coarse capability-class projection used by the parent-policy resolver:
    /// only public observation is local base integration; action and
    /// persistence always resolve as remote/premium capability.
    pub const fn operation_class(self) -> UiaiOperationClass {
        match self {
            Self::PublicObserve => UiaiOperationClass::PublicObservation,
            Self::Action | Self::Persistence => UiaiOperationClass::RemotePremium,
        }
    }
}

/// One canonical Focusa UIAI operation-map row (Spec 172 §6.3; Spec 152F §7
/// UIAI/browser adapter row).
///
/// Every field is server-owned constant metadata:
/// - `operation_id` is the shared Focusa/UIAI vector identifier;
/// - `class` is the public-observe/action/persistence classification;
/// - `limited_family` is the verified-no-license family label for this
///   operation (allowlisted or blocked);
/// - `paid_feature` is the paid UIAI Operator v1 family feature this
///   operation may resolve under a paid grant. Metered/hosted/private-right
///   operations carry features outside [`SPEC172_UIAI_PAID_FAMILY_FEATURES`]
///   so they fail closed even for paid grants (Spec 172 §7.2: paid UIAI never
///   includes paid proxies, hosted compute, paid model usage, or
///   authenticated/private targets unless explicitly listed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiaiOperationMapEntry {
    pub operation_id: &'static str,
    pub class: UiaiActionPersistenceClass,
    pub limited_family: &'static str,
    pub paid_feature: &'static str,
}

/// Canonical Focusa UIAI operation map (Spec 172 §6.3).
///
/// Shared Focusa/UIAI vectors: the six verified-no-license public-observe
/// operations, the six §6.3 blocked families (browser action, persistence,
/// authenticated/private targets, unattended automation, scheduled/batch QA,
/// premium hosted resources), each with its paid UIAI Operator v1 family
/// feature. Unknown operation ids fail closed in the resolver; no caller can
/// extend, rename, or reclassify a row.
pub const SPEC172_UIAI_OPERATION_MAP: &[UiaiOperationMapEntry] = &[
    // Public observation — verified-no-license allowlist (§6.3 first block).
    UiaiOperationMapEntry {
        operation_id: "public_search",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "public_search",
        paid_feature: "uiai_public_observation",
    },
    UiaiOperationMapEntry {
        operation_id: "source_to_markdown",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "source_to_markdown",
        paid_feature: "uiai_public_observation",
    },
    UiaiOperationMapEntry {
        operation_id: "public_page_read",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "public_page_read",
        paid_feature: "uiai_public_observation",
    },
    UiaiOperationMapEntry {
        operation_id: "accessibility_snapshot",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "accessibility_snapshot",
        paid_feature: "uiai_public_observation",
    },
    UiaiOperationMapEntry {
        operation_id: "screenshot",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "screenshot",
        paid_feature: "uiai_public_observation",
    },
    UiaiOperationMapEntry {
        operation_id: "basic_diagnostics",
        class: UiaiActionPersistenceClass::PublicObserve,
        limited_family: "basic_diagnostics",
        paid_feature: "uiai_diagnostics",
    },
    // Browser mutation — action class, blocked in limited mode, paid action
    // family under an active `uiai-engine` grant.
    UiaiOperationMapEntry {
        operation_id: "browser_click",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    UiaiOperationMapEntry {
        operation_id: "browser_fill",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    UiaiOperationMapEntry {
        operation_id: "browser_type",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    UiaiOperationMapEntry {
        operation_id: "browser_select",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    UiaiOperationMapEntry {
        operation_id: "browser_press",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    UiaiOperationMapEntry {
        operation_id: "browser_submit",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "browser_action",
        paid_feature: "uiai_browser_action",
    },
    // Persistence — persistence class, blocked in limited mode, paid
    // persistence family under an active grant.
    UiaiOperationMapEntry {
        operation_id: "cookie_persistence",
        class: UiaiActionPersistenceClass::Persistence,
        limited_family: "browser_persistence",
        paid_feature: "uiai_persistence",
    },
    UiaiOperationMapEntry {
        operation_id: "auth_state_persistence",
        class: UiaiActionPersistenceClass::Persistence,
        limited_family: "browser_persistence",
        paid_feature: "uiai_persistence",
    },
    UiaiOperationMapEntry {
        operation_id: "session_persistence",
        class: UiaiActionPersistenceClass::Persistence,
        limited_family: "browser_persistence",
        paid_feature: "uiai_persistence",
    },
    // Authenticated/private targets: blocked in limited mode and never in the
    // paid Operator v1 families (no canonical paid feature -> fail closed).
    UiaiOperationMapEntry {
        operation_id: "authenticated_private_dashboard",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "authenticated_private_targets",
        paid_feature: "uiai_authenticated_private_targets",
    },
    // Unattended browser automation: blocked in limited mode and not included
    // in paid Operator v1 (no canonical paid feature -> fail closed).
    UiaiOperationMapEntry {
        operation_id: "unattended_browser_automation",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "unattended_browser_automation",
        paid_feature: "uiai_unattended_automation",
    },
    // Scheduled/batch responsive QA: blocked in limited mode; paid batch
    // family under an active grant (Spec 172 §7.2 batch/responsive).
    UiaiOperationMapEntry {
        operation_id: "scheduled_batch_qa",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "scheduled_batch_qa",
        paid_feature: "uiai_batch_responsive",
    },
    // Metered hosted/premium resources: blocked in limited mode and never in
    // the paid Operator v1 families (no canonical paid feature -> fail closed
    // for every grant; Spec 172 §7.2).
    UiaiOperationMapEntry {
        operation_id: "premium_proxy",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "premium_hosted_resources",
        paid_feature: "uiai_premium_proxy",
    },
    UiaiOperationMapEntry {
        operation_id: "hosted_capacity",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "premium_hosted_resources",
        paid_feature: "uiai_hosted_capacity",
    },
    UiaiOperationMapEntry {
        operation_id: "paid_model_calls",
        class: UiaiActionPersistenceClass::Action,
        limited_family: "premium_hosted_resources",
        paid_feature: "uiai_paid_model_calls",
    },
];

/// Typed failure for operation-map resolution. Unknown operation ids fail
/// closed before any entitlement decision or UI side effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum UiaiOperationError {
    UnknownOperation,
}

/// Look up the canonical map row for a UIAI/browser operation id.
///
/// `None` for any unknown, prefixed, aliased, or caller-invented id: the map
/// is the single authority for UIAI operation vectors.
pub fn classify_uiai_operation(operation_id: &str) -> Option<&'static UiaiOperationMapEntry> {
    SPEC172_UIAI_OPERATION_MAP
        .iter()
        .find(|entry| entry.operation_id == operation_id)
}

/// Product-isolation adapter: resolve a canonical UIAI/browser operation
/// vector against the parent policy and the independent `uiai-engine` grant
/// (Spec 152F §7 UIAI/browser adapter row; Spec 172 §6.3 / §7.2).
///
/// This is the shared Focusa/UIAI entry point browser, proxy, MCP, and
/// taskgraph adapters call BEFORE any child token is minted or any UI side
/// effect runs. Only the operation id (a canonical shared vector) and
/// authority snapshots are consumed: product, price, License Type, family,
/// feature, limit, node, and commercial rights are never caller-controlled.
/// Unknown operation ids fail closed with [`UiaiOperationError`].
pub fn resolve_uiai_operation_capability(
    operation_id: &str,
    focusa_parent: Option<&EntitlementSnapshot>,
    uiai_grant: Option<&EntitlementSnapshot>,
    active_session_count: u32,
    now: DateTime<Utc>,
) -> Result<UiaiCapabilityDecision, UiaiOperationError> {
    let entry =
        classify_uiai_operation(operation_id).ok_or(UiaiOperationError::UnknownOperation)?;
    Ok(resolve_uiai_capability(
        focusa_parent,
        uiai_grant,
        entry.class.operation_class(),
        entry.limited_family,
        entry.paid_feature,
        active_session_count,
        now,
    ))
}

/// Canonical paid UIAI Operator v1 family features (Spec 172 §7.2; frozen
/// UIAI local/product families from the EDD bundle-isolation contract). These
/// are the only feature identifiers a paid `uiai-engine` grant may bind to a
/// UIAI/browser child-token decision. Callers can neither extend nor rename
/// this set, and no Focusa feature satisfies a UIAI family.
pub const SPEC172_UIAI_PAID_FAMILY_FEATURES: [&str; 7] = [
    "uiai_public_observation",
    "uiai_browser_action",
    "uiai_persistence",
    "uiai_diagnostics",
    "uiai_proof_packets",
    "uiai_batch_responsive",
    "uiai_supported_integrations",
];

/// Typed denial reasons for the parent-policy UIAI capability decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum UiaiCapabilityDenial {
    /// No verified posture (and no paid grant) is present at all.
    MissingPosture,
    /// A Focusa-only paid entitlement can never grant UIAI capability
    /// (Spec 172 §3.7 / §20.5: "Focusa-only entitlement cannot execute UIAI
    /// paid operations").
    FocusaOnlyCannotGrantUiai,
    /// Remote/premium UIAI capability requires an active paid UIAI grant;
    /// verified no-license limited mode never provides it.
    UiaiGrantRequired,
    /// Verified no-license limited mode is restricted to ONE foreground,
    /// ephemeral, public-web observation session (Spec 172 §6.3).
    LimitedModeRestricted,
    /// The `uiai-engine` grant is not active, sequence-bound, or bound to the
    /// parent node.
    UiaiGrantInvalid,
    /// The Focusa parent and UIAI grant are not bound to the same EDD
    /// account (Spec 152E §7 / §15).
    AccountMismatch,
    /// The operation's paid UIAI family feature is not granted by the
    /// authority grant.
    FamilyNotGranted,
}

/// Canonical parent-policy UIAI/browser capability decision (Spec 152F §7
/// UIAI/browser adapter row; Spec 172 §6.3 / §7.2).
///
/// Every UIAI/browser/proxy/MCP operation resolves its entitlement through
/// this decision BEFORE any child token is minted or presented:
/// - verified no-license: exactly one ephemeral public-observe session;
/// - paid UIAI Operator/Bundle: the granted paid UIAI family, bound to the
///   current parent Focusa lease and UIAI grant sequence;
/// - Focusa-only paid entitlement: denied — it never grants UIAI.
///
/// Only authority snapshots and operation metadata are consumed; pairing,
/// device proof, authentication state, and caller-selected products/prices/
/// grants never influence the decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum UiaiCapabilityDecision {
    /// Verified no-license: exactly one foreground, ephemeral, public-web
    /// observation session with bounded local/approved capacity.
    VerifiedNoLicensePublicObservation {
        session_quota: u32,
    },
    /// Paid UIAI family from an active, bound `uiai-engine` grant. The parent
    /// Focusa lease (when the account holds one) and the UIAI grant sequence
    /// are returned so the child token can never outlive or widen them.
    PaidFamily {
        family: String,
        parent_lease_id: String,
        parent_sequence: u64,
        uiai_grant_sequence: u64,
    },
    Denied(UiaiCapabilityDenial),
}

impl UiaiCapabilityDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            Self::VerifiedNoLicensePublicObservation { .. } | Self::PaidFamily { .. }
        )
    }

    pub fn denial(&self) -> Option<&UiaiCapabilityDenial> {
        match self {
            Self::Denied(denial) => Some(denial),
            _ => None,
        }
    }
}

/// True when the snapshot is an authority-signed verified no-license limited
/// posture: no paid lease identity, sequence, or digest is present. Such a
/// posture is a permanent limited-access assertion, not a license.
fn is_limited_posture(snapshot: &EntitlementSnapshot) -> bool {
    snapshot.lease_id.as_deref().is_none_or(str::is_empty)
        && snapshot.sequence.is_none_or(|sequence| sequence == 0)
        && snapshot.lease_digest.as_deref().is_none_or(str::is_empty)
}

/// Resolve the canonical UIAI/browser capability for one operation against
/// the parent policy (Spec 152F §7 UIAI/browser adapter row; Spec 172 §6.3,
/// §7.2, §20.5).
///
/// - `focusa_parent`: the current Focusa snapshot for this node/account
///   (`None` when the account holds no Focusa lease/posture, e.g. a UIAI-only
///   purchase).
/// - `uiai_grant`: the independent `uiai-engine` grant (`None` when the
///   account has no paid UIAI grant).
/// - `operation_class`: the canonical operation classification.
/// - `limited_family`: the verified-no-license family label for this
///   operation (e.g. `public_search`, `browser_action`).
/// - `paid_feature`: the paid UIAI Operator family feature for this operation
///   (e.g. `uiai_public_observation`, `uiai_browser_action`).
/// - `active_session_count`: currently active UIAI sessions (resource gate).
///
/// Fail-closed rules:
/// 1. verified no-license -> one foreground ephemeral public-observe session;
/// 2. Focusa-only paid entitlement -> never UIAI;
/// 3. paid UIAI grant -> the granted paid family, bound to the parent lease
///    and the grant sequence, with same-account and same-node binding;
/// 4. unknown postures, unknown families, and ungranted features deny.
pub fn resolve_uiai_capability(
    focusa_parent: Option<&EntitlementSnapshot>,
    uiai_grant: Option<&EntitlementSnapshot>,
    operation_class: UiaiOperationClass,
    limited_family: &str,
    paid_feature: &str,
    active_session_count: u32,
    now: DateTime<Utc>,
) -> UiaiCapabilityDecision {
    let Some(grant) = uiai_grant else {
        // No paid UIAI grant. Only a verified no-license posture may observe.
        return match focusa_parent {
            Some(parent) if is_limited_posture(parent) => {
                if operation_class != UiaiOperationClass::PublicObservation {
                    return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::UiaiGrantRequired);
                }
                if !SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.contains(&limited_family)
                    || SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES.contains(&limited_family)
                    || active_session_count >= 1
                {
                    return UiaiCapabilityDecision::Denied(
                        UiaiCapabilityDenial::LimitedModeRestricted,
                    );
                }
                UiaiCapabilityDecision::VerifiedNoLicensePublicObservation { session_quota: 1 }
            }
            Some(_) => {
                // Paid Focusa entitlement without a UIAI grant: never UIAI.
                UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai)
            }
            None => UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::MissingPosture),
        };
    };

    // Paid UIAI grant path (Operator or Bundle). The grant must be active,
    // sequence-bound, and bound to the same node as the parent.
    if !active_bound(
        grant,
        crate::uiai_activation::PRODUCT_UIAI_ENGINE,
        &grant.node_id,
        now,
    ) {
        return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::UiaiGrantInvalid);
    }
    let Some(grant_sequence) = grant.sequence.filter(|sequence| *sequence > 0) else {
        return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::UiaiGrantInvalid);
    };
    if let Some(parent) = focusa_parent {
        if parent.node_id != grant.node_id {
            return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::UiaiGrantInvalid);
        }
        if !same_evidence_account(parent, grant) {
            return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::AccountMismatch);
        }
    }
    if !SPEC172_UIAI_PAID_FAMILY_FEATURES.contains(&paid_feature)
        || !grant.features.get(paid_feature).copied().unwrap_or(false)
    {
        return UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FamilyNotGranted);
    }

    let (parent_lease_id, parent_sequence) = match focusa_parent {
        Some(parent) if !is_limited_posture(parent) => (
            parent.lease_id.clone().unwrap_or_default(),
            parent.sequence.unwrap_or(0),
        ),
        _ => (String::new(), 0),
    };
    UiaiCapabilityDecision::PaidFamily {
        family: paid_feature.to_string(),
        parent_lease_id,
        parent_sequence,
        uiai_grant_sequence: grant_sequence,
    }
}

fn entitlement_bound(snapshot: &EntitlementSnapshot) -> Option<DateTime<Utc>> {
    match snapshot.state {
        EntitlementState::Active => snapshot.expires_at,
        EntitlementState::OfflineGrace => snapshot.offline_grace_until,
        EntitlementState::Unactivated | EntitlementState::RecoveryOnly => None,
    }
}

/// Same-account evidence check: when either snapshot carries a lease
/// `subject_id` (account UUID), both must carry the SAME account. Synthetic
/// snapshots without lease subjects cannot prove an account split and pass;
/// verified leases always carry the authority-issued subject, so a mismatch
/// or a missing subject on one side fails closed.
fn same_evidence_account(parent: &EntitlementSnapshot, grant: &EntitlementSnapshot) -> bool {
    match (&parent.subject_id, &grant.subject_id) {
        (Some(parent_account), Some(grant_account)) => parent_account == grant_account,
        (None, None) => true,
        _ => false,
    }
}

fn active_bound(
    snapshot: &EntitlementSnapshot,
    product: &str,
    node: &str,
    now: DateTime<Utc>,
) -> bool {
    snapshot.product == product
        && snapshot.node_id == node
        && snapshot
            .lease_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        && snapshot.sequence.is_some_and(|value| value > 0)
        && snapshot
            .lease_digest
            .as_deref()
            .is_some_and(|value| value.starts_with("sha256:"))
        && match snapshot.state {
            EntitlementState::Active => snapshot.expires_at.is_some_and(|expiry| expiry > now),
            EntitlementState::OfflineGrace => snapshot
                .offline_grace_until
                .is_some_and(|expiry| expiry > now),
            EntitlementState::Unactivated | EntitlementState::RecoveryOnly => false,
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uiai_activation::{PRODUCT_FOCUSA, PRODUCT_UIAI_ENGINE, UiaiAccountIdentity};

    fn bound_snapshot(product: &str, subject: Option<&str>) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(product, "node-001");
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = subject.map(str::to_string);
        snapshot.lease_id = Some(format!("lease-{product}"));
        snapshot.sequence = Some(7);
        snapshot.lease_digest = Some("sha256:bound-grant-digest".to_string());
        snapshot.expires_at = Some(Utc::now() + Duration::hours(1));
        snapshot
            .features
            .insert("uiai.engine.core".to_string(), true);
        snapshot
    }

    fn request() -> UiaiChildTokenRequest {
        UiaiChildTokenRequest {
            request_id: Uuid::nil(),
            audience: "aud-focusa".to_string(),
            node_id: "node-001".to_string(),
            client_id: "client-focusa".to_string(),
            parent_lease_id: "lease-focusa".to_string(),
            parent_lease_sequence: 7,
            parent_lease_digest: "sha256:bound-grant-digest".to_string(),
            uiai_grant_lease_id: "lease-uiai-engine".to_string(),
            uiai_grant_sequence: 7,
            requested_features: BTreeSet::from(["uiai.engine.core".to_string()]),
            requested_limits: BTreeMap::new(),
            nonce: "nonce-same-account".to_string(),
        }
    }

    #[test]
    fn same_account_leases_pass_and_different_accounts_fail_closed() {
        let now = Utc::now();
        let focusa = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let uiai = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        let broker = UiaiChildTokenBroker::default();
        assert_eq!(
            broker.validate_request(&request(), &focusa, &uiai, now),
            Ok(())
        );
        // A UIAI grant bound to a different EDD account fails closed.
        let other_customer = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-002"));
        assert_eq!(
            broker.validate_request(&request(), &focusa, &other_customer, now),
            Err(UiaiChildTokenError::AccountMismatch)
        );
        // One side proving an account and the other not also fails closed.
        let no_subject = bound_snapshot(PRODUCT_UIAI_ENGINE, None);
        assert_eq!(
            broker.validate_request(&request(), &focusa, &no_subject, now),
            Err(UiaiChildTokenError::AccountMismatch)
        );
    }

    #[test]
    fn strict_same_account_binding_requires_the_single_verified_identity() {
        let focusa = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let uiai = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        let broker = UiaiChildTokenBroker::default();
        let account = UiaiAccountIdentity {
            account_id: "account-001".to_string(),
            edd_customer_id: 1001,
        };
        assert!(
            broker
                .validate_same_account_binding(&account, &focusa, &uiai)
                .is_ok()
        );
        // No duplicate customer identity: a second customer on either lease
        // or an empty account identity is rejected.
        let second_customer = UiaiAccountIdentity {
            account_id: "account-002".to_string(),
            edd_customer_id: 1002,
        };
        assert_eq!(
            broker.validate_same_account_binding(&second_customer, &focusa, &uiai),
            Err(UiaiChildTokenError::AccountMismatch)
        );
        assert_eq!(
            broker.validate_same_account_binding(
                &UiaiAccountIdentity {
                    account_id: String::new(),
                    edd_customer_id: 0,
                },
                &focusa,
                &uiai,
            ),
            Err(UiaiChildTokenError::AccountMismatch)
        );
    }

    fn limited_posture() -> EntitlementSnapshot {
        EntitlementSnapshot::unactivated(PRODUCT_FOCUSA, "node-001")
    }

    fn grant_with_features(features: &[&str]) -> EntitlementSnapshot {
        let mut grant = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        for feature in features {
            grant.features.insert(feature.to_string(), true);
        }
        grant
    }

    fn paid_uiai_grant() -> EntitlementSnapshot {
        let mut grant = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        grant
            .features
            .insert("uiai_browser_action".to_string(), true);
        grant
    }

    #[test]
    fn uiai_capability_limited_mode_is_one_foreground_ephemeral_public_observe_session() {
        let now = Utc::now();
        let limited = limited_posture();
        // Exactly one foreground ephemeral public-observe session.
        assert_eq!(
            resolve_uiai_capability(
                Some(&limited),
                None,
                UiaiOperationClass::PublicObservation,
                "public_search",
                "uiai_public_observation",
                0,
                now,
            ),
            UiaiCapabilityDecision::VerifiedNoLicensePublicObservation { session_quota: 1 }
        );
        // A second concurrent session is denied (resource gate).
        assert_eq!(
            resolve_uiai_capability(
                Some(&limited),
                None,
                UiaiOperationClass::PublicObservation,
                "public_search",
                "uiai_public_observation",
                1,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::LimitedModeRestricted)
        );
        // Remote/premium capability is never available without a paid grant.
        assert_eq!(
            resolve_uiai_capability(
                Some(&limited),
                None,
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::UiaiGrantRequired)
        );
        // A blocked limited-mode family is denied even for observation.
        assert_eq!(
            resolve_uiai_capability(
                Some(&limited),
                None,
                UiaiOperationClass::PublicObservation,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::LimitedModeRestricted)
        );
        // No verified posture at all fails closed.
        assert_eq!(
            resolve_uiai_capability(
                None,
                None,
                UiaiOperationClass::PublicObservation,
                "public_search",
                "uiai_public_observation",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::MissingPosture)
        );
    }

    #[test]
    fn uiai_capability_focusa_only_never_grants_uiai() {
        let now = Utc::now();
        let focusa_paid = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        for operation_class in [
            UiaiOperationClass::PublicObservation,
            UiaiOperationClass::RemotePremium,
        ] {
            assert_eq!(
                resolve_uiai_capability(
                    Some(&focusa_paid),
                    None,
                    operation_class,
                    "public_search",
                    "uiai_public_observation",
                    0,
                    now,
                ),
                UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai),
                "Focusa-only paid entitlement must never grant UIAI: {operation_class:?}"
            );
        }
    }

    #[test]
    fn uiai_capability_paid_grant_binds_paid_families_to_parent_and_grant_sequence() {
        let now = Utc::now();
        let focusa_paid = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let grant = paid_uiai_grant();
        assert_eq!(
            resolve_uiai_capability(
                Some(&focusa_paid),
                Some(&grant),
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::PaidFamily {
                family: "uiai_browser_action".to_string(),
                parent_lease_id: "lease-focusa".to_string(),
                parent_sequence: 7,
                uiai_grant_sequence: 7,
            }
        );
        // UIAI-only purchase (no Focusa parent) still resolves paid families.
        assert_eq!(
            resolve_uiai_capability(
                None,
                Some(&grant),
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::PaidFamily {
                family: "uiai_browser_action".to_string(),
                parent_lease_id: String::new(),
                parent_sequence: 0,
                uiai_grant_sequence: 7,
            }
        );
        // An ungranted family feature fails closed.
        let ungranted = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        assert_eq!(
            resolve_uiai_capability(
                Some(&focusa_paid),
                Some(&ungranted),
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FamilyNotGranted)
        );
        // A caller-invented feature identifier is never accepted.
        assert_eq!(
            resolve_uiai_capability(
                Some(&focusa_paid),
                Some(&grant),
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_caller_invented_feature",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FamilyNotGranted)
        );
        // A grant bound to a different EDD account fails closed.
        let other_customer = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-002"));
        assert_eq!(
            resolve_uiai_capability(
                Some(&focusa_paid),
                Some(&other_customer),
                UiaiOperationClass::RemotePremium,
                "browser_action",
                "uiai_browser_action",
                0,
                now,
            ),
            UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::AccountMismatch)
        );
    }

    #[test]
    fn spec172_operation_map_classifies_observe_action_and_persistence() {
        // Every verified-no-license allowlisted family is PublicObserve and
        // every one of its operations resolves as local base integration.
        for operation_id in [
            "public_search",
            "source_to_markdown",
            "public_page_read",
            "accessibility_snapshot",
            "screenshot",
            "basic_diagnostics",
        ] {
            let entry = classify_uiai_operation(operation_id).expect(operation_id);
            assert_eq!(
                entry.class,
                UiaiActionPersistenceClass::PublicObserve,
                "{operation_id}"
            );
            assert_eq!(
                entry.class.operation_class(),
                UiaiOperationClass::PublicObservation,
                "{operation_id}"
            );
            assert!(
                SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.contains(&entry.limited_family),
                "{operation_id} limited family must be allowlisted"
            );
        }
        // Browser mutation is the action class and binds the paid action family.
        for operation_id in [
            "browser_click",
            "browser_fill",
            "browser_type",
            "browser_select",
            "browser_press",
            "browser_submit",
        ] {
            let entry = classify_uiai_operation(operation_id).expect(operation_id);
            assert_eq!(
                entry.class,
                UiaiActionPersistenceClass::Action,
                "{operation_id}"
            );
            assert_eq!(entry.limited_family, "browser_action", "{operation_id}");
            assert_eq!(entry.paid_feature, "uiai_browser_action", "{operation_id}");
        }
        // Persistence is its own class and binds the paid persistence family.
        for operation_id in [
            "cookie_persistence",
            "auth_state_persistence",
            "session_persistence",
        ] {
            let entry = classify_uiai_operation(operation_id).expect(operation_id);
            assert_eq!(
                entry.class,
                UiaiActionPersistenceClass::Persistence,
                "{operation_id}"
            );
            assert_eq!(
                entry.limited_family, "browser_persistence",
                "{operation_id}"
            );
            assert_eq!(entry.paid_feature, "uiai_persistence", "{operation_id}");
        }
        // Authenticated/private, unattended, and hosted rights carry NO
        // canonical paid Operator v1 feature: they fail closed even for paid
        // grants (Spec 172 §7.2).
        for operation_id in [
            "authenticated_private_dashboard",
            "unattended_browser_automation",
            "premium_proxy",
            "hosted_capacity",
            "paid_model_calls",
        ] {
            let entry = classify_uiai_operation(operation_id).expect(operation_id);
            assert!(
                !SPEC172_UIAI_PAID_FAMILY_FEATURES.contains(&entry.paid_feature),
                "{operation_id} must carry no canonical paid feature"
            );
            assert_eq!(
                entry.class.operation_class(),
                UiaiOperationClass::RemotePremium
            );
        }
        // Unknown, prefixed, and aliased ids never resolve.
        assert_eq!(classify_uiai_operation("caller_invented_operation"), None);
        assert_eq!(classify_uiai_operation("screenshot2"), None);
        assert_eq!(classify_uiai_operation(""), None);
        // Every map row is unique and every limited family label is canonical.
        let mut ids: Vec<&str> = SPEC172_UIAI_OPERATION_MAP
            .iter()
            .map(|entry| entry.operation_id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SPEC172_UIAI_OPERATION_MAP.len());
        for entry in SPEC172_UIAI_OPERATION_MAP {
            assert!(
                SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.contains(&entry.limited_family)
                    || SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES
                        .contains(&entry.limited_family),
                "{} limited family must be canonical",
                entry.operation_id
            );
        }
    }

    #[test]
    fn spec172_limited_mode_allows_only_one_foreground_public_observe_session() {
        let now = Utc::now();
        let limited = limited_posture();
        // Exactly one foreground ephemeral public-observe session.
        for operation_id in [
            "public_search",
            "source_to_markdown",
            "public_page_read",
            "accessibility_snapshot",
            "screenshot",
            "basic_diagnostics",
        ] {
            assert_eq!(
                resolve_uiai_operation_capability(operation_id, Some(&limited), None, 0, now),
                Ok(UiaiCapabilityDecision::VerifiedNoLicensePublicObservation { session_quota: 1 }),
                "{operation_id} must be allowed once in limited mode"
            );
        }
        // A second concurrent session is denied before any side effect.
        assert_eq!(
            resolve_uiai_operation_capability("public_search", Some(&limited), None, 1, now),
            Ok(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::LimitedModeRestricted
            ))
        );
        // Every action/persistence/hosted operation fails closed in limited mode.
        for operation_id in [
            "browser_click",
            "browser_fill",
            "browser_type",
            "browser_select",
            "browser_press",
            "browser_submit",
            "cookie_persistence",
            "auth_state_persistence",
            "session_persistence",
            "authenticated_private_dashboard",
            "unattended_browser_automation",
            "scheduled_batch_qa",
            "premium_proxy",
            "hosted_capacity",
            "paid_model_calls",
        ] {
            assert_eq!(
                resolve_uiai_operation_capability(operation_id, Some(&limited), None, 0, now),
                Ok(UiaiCapabilityDecision::Denied(
                    UiaiCapabilityDenial::UiaiGrantRequired
                )),
                "{operation_id} must fail closed in limited mode"
            );
        }
        // Unknown operations fail before any decision.
        assert_eq!(
            resolve_uiai_operation_capability(
                "caller_invented_operation",
                Some(&limited),
                None,
                0,
                now
            ),
            Err(UiaiOperationError::UnknownOperation)
        );
    }

    #[test]
    fn spec172_paid_boundary_requires_granted_family_and_denies_hosted_rights() {
        let now = Utc::now();
        let focusa_paid = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let action_grant = grant_with_features(&["uiai_browser_action"]);
        // Paid browser action proceeds bound to the parent lease and grant sequence.
        assert_eq!(
            resolve_uiai_operation_capability(
                "browser_click",
                Some(&focusa_paid),
                Some(&action_grant),
                0,
                now,
            ),
            Ok(UiaiCapabilityDecision::PaidFamily {
                family: "uiai_browser_action".to_string(),
                parent_lease_id: "lease-focusa".to_string(),
                parent_sequence: 7,
                uiai_grant_sequence: 7,
            })
        );
        // Persistence requires the paid uiai_persistence grant feature.
        assert_eq!(
            resolve_uiai_operation_capability(
                "cookie_persistence",
                Some(&focusa_paid),
                Some(&action_grant),
                0,
                now,
            ),
            Ok(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FamilyNotGranted
            ))
        );
        let persistence_grant = grant_with_features(&["uiai_persistence"]);
        assert_eq!(
            resolve_uiai_operation_capability(
                "cookie_persistence",
                Some(&focusa_paid),
                Some(&persistence_grant),
                0,
                now,
            ),
            Ok(UiaiCapabilityDecision::PaidFamily {
                family: "uiai_persistence".to_string(),
                parent_lease_id: "lease-focusa".to_string(),
                parent_sequence: 7,
                uiai_grant_sequence: 7,
            })
        );
        // Hosted/private rights are denied even for paid grants (no canonical
        // paid feature; Spec 172 §7.2 never includes them).
        for operation_id in [
            "authenticated_private_dashboard",
            "unattended_browser_automation",
            "premium_proxy",
            "hosted_capacity",
            "paid_model_calls",
        ] {
            assert_eq!(
                resolve_uiai_operation_capability(
                    operation_id,
                    Some(&focusa_paid),
                    Some(&action_grant),
                    0,
                    now,
                ),
                Ok(UiaiCapabilityDecision::Denied(
                    UiaiCapabilityDenial::FamilyNotGranted
                )),
                "{operation_id} must deny even for paid grants"
            );
        }
        // Focusa-only paid entitlement never grants UIAI — even observation.
        assert_eq!(
            resolve_uiai_operation_capability("public_search", Some(&focusa_paid), None, 0, now),
            Ok(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai
            ))
        );
        // No posture at all fails closed.
        assert_eq!(
            resolve_uiai_operation_capability("public_search", None, None, 0, now),
            Ok(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::MissingPosture
            ))
        );
    }

    #[test]
    fn cached_child_tokens_reject_stale_parents_and_widened_grants() {
        let now = Utc::now();
        let focusa = bound_snapshot(PRODUCT_FOCUSA, Some("account-001"));
        let grant = bound_snapshot(PRODUCT_UIAI_ENGINE, Some("account-001"));
        let mut broker = UiaiChildTokenBroker::default();
        let request = request();
        let envelope = AuthorityChildTokenEnvelope {
            schema: UIAI_CHILD_TOKEN_SCHEMA.to_string(),
            token: "tok-authority-issued".to_string(),
            token_id: "ct-001".to_string(),
            audience: request.audience.clone(),
            node_id: request.node_id.clone(),
            client_id: request.client_id.clone(),
            parent_lease_id: request.parent_lease_id.clone(),
            parent_lease_sequence: request.parent_lease_sequence,
            parent_lease_digest: request.parent_lease_digest.clone(),
            uiai_grant_lease_id: request.uiai_grant_lease_id.clone(),
            uiai_grant_sequence: request.uiai_grant_sequence,
            features: request.requested_features.clone(),
            limits: request.requested_limits.clone(),
            nonce: request.nonce.clone(),
            issued_at: now,
            expires_at: now + Duration::minutes(UIAI_CHILD_TOKEN_MAX_TTL_MINUTES),
        };
        let receipt = broker
            .accept_authority_token(&request, &focusa, &grant, envelope, now)
            .expect("authority token accepted");
        assert_eq!(receipt.feature_count, 1);

        // Current parent and grant: cached token remains authorized.
        assert!(
            broker
                .authorized_cached_token("aud-focusa", &focusa, &grant, now)
                .is_some()
        );

        // Stale parent (sequence advanced): cached token is rejected.
        let mut advanced = focusa.clone();
        advanced.sequence = Some(8);
        assert!(
            broker
                .authorized_cached_token("aud-focusa", &advanced, &grant, now)
                .is_none(),
            "stale parent sequence must reject the cached token"
        );

        // Revoked grant: cached token is rejected.
        let mut revoked = grant.clone();
        revoked.state = EntitlementState::RecoveryOnly;
        assert!(
            broker
                .authorized_cached_token("aud-focusa", &focusa, &revoked, now)
                .is_none(),
            "revoked grant must reject the cached token"
        );

        // Widened token: the grant no longer contains the cached feature.
        let mut narrowed = grant.clone();
        narrowed.features.remove("uiai.engine.core");
        assert!(
            broker
                .authorized_cached_token("aud-focusa", &focusa, &narrowed, now)
                .is_none(),
            "widened cached token must be rejected when the grant narrows"
        );

        // Expired token is rejected by the underlying TTL filter.
        assert!(
            broker
                .authorized_cached_token(
                    "aud-focusa",
                    &focusa,
                    &grant,
                    now + Duration::minutes(UIAI_CHILD_TOKEN_MAX_TTL_MINUTES + 1),
                )
                .is_none()
        );
    }
}
