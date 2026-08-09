//! UIAI Engine activation through the same EDD account (Spec 152E §7 account
//! model, §8 product/grant registry, §21 shared activation client, §23
//! acceptance matrix "UIAI purchase" / "Bundle purchase"; Specs 152, 150A,
//! 152A-D; Spec 172 overlay; Spec 158 implementation excluded).
//!
//! This module is the shared account/product contract for UIAI activation:
//! - exactly one verified EDD account identity is used for the whole UIAI
//!   activation — no duplicate customer identity is ever created;
//! - the mapping is server-owned: the client submits only the public product
//!   code (`uiai_operator_lifetime_v1`) and never EDD ids, prices, tiers,
//!   grants, limits, or commercial flags;
//! - an independent UIAI grant (`uiai-engine` product) is required: a
//!   Focusa-only account can never activate UIAI;
//! - product isolation is preserved: exact UIAI features/limits come only
//!   from the independent `uiai-engine` grant; the Focusa grant never
//!   satisfies UIAI scope and no cross-product lease is produced;
//! - the UIAI key/lease is delivered through the same registration (the
//!   shared `ActivationSession`) and bound to the single account, so UIAI and
//!   Focusa grants coexist on one EDD customer without merging products.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::authority::{EntitlementSnapshot, EntitlementState};

pub const UIAI_ACTIVATION_SCHEMA: &str = "focusa.uiai_activation_contract.v1";

/// Exact authority-owned product identifiers (Spec 152E §8). Casing,
/// whitespace, aliases, and prefixed values fail closed.
pub const PRODUCT_FOCUSA: &str = "focusa";
pub const PRODUCT_UIAI_ENGINE: &str = "uiai-engine";

/// Public product codes from the frozen EDD product registry
/// (`docs/contracts/spec152e-edd-product-registry.v1.json`). Clients submit
/// only these codes; the server-owned registry owns every other field.
pub const PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1: &str = "focusa_operator_lifetime_v1";
pub const PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1: &str = "uiai_operator_lifetime_v1";
pub const PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1: &str =
    "focusa_uiai_operator_bundle_lifetime_v1";

/// One verified EDD account identity. The UIAI activation (and any Focusa
/// grant on the same registration) resolves to this single account and EDD
/// customer: there is structurally no second customer identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiAccountIdentity {
    pub account_id: String,
    pub edd_customer_id: u64,
}

impl UiaiAccountIdentity {
    pub fn valid(&self) -> bool {
        !self.account_id.trim().is_empty() && self.edd_customer_id != 0
    }
}

/// Products the verified account holds (`focusa`, `uiai-engine`, or both for
/// the bundle). This set comes from the server-owned registry for the
/// account's verified purchases; clients can never add products here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountProductGrants {
    pub products: BTreeSet<String>,
}

impl AccountProductGrants {
    pub fn new(products: impl IntoIterator<Item = String>) -> Self {
        Self {
            products: products.into_iter().collect(),
        }
    }

    pub fn has_product(&self, product: &str) -> bool {
        self.products.contains(product)
    }

    /// A Focusa-only account holds exactly `focusa` and nothing else; it can
    /// never activate UIAI (no independent UIAI grant).
    pub fn focusa_only(&self) -> bool {
        self.products.len() == 1 && self.products.contains(PRODUCT_FOCUSA)
    }
}

/// Fail-closed UIAI activation errors. Every denial returns a typed safe
/// reason; nothing is silently normalized into a grant.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UiaiActivationError {
    #[error("the requested public product code is not the UIAI product mapping")]
    ProductMappingRequired,
    #[error("an independent UIAI grant is required; Focusa-only accounts cannot activate UIAI")]
    UiaiGrantRequired,
    #[error("the independent UIAI grant is not active and bound to this node")]
    UiaiGrantInvalid,
    #[error("UIAI scope is not an exact subset of the independent UIAI grant")]
    ProductIsolationViolation,
    #[error("a verified EDD account identity is required")]
    AccountIdentityRequired,
}

/// Exact UIAI grant projection, derived only from the independent
/// `uiai-engine` grant and bound to the single verified EDD account. This is
/// the machine-read contract for the UIAI key/lease delivered through the
/// same registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiaiGrantProjection {
    pub schema: String,
    pub product: String,
    pub public_code: String,
    pub account: UiaiAccountIdentity,
    pub node_id: String,
    pub grant_lease_id: String,
    pub grant_sequence: u64,
    pub features: BTreeSet<String>,
    pub limits: BTreeMap<String, u64>,
}

/// Same-account binding: the Focusa parent lease and the independent UIAI
/// grant lease must both be issued to the single verified EDD account (Spec
/// 152E §15 lease `subject_id`). One account, one EDD customer — no
/// duplicate customer identity.
pub fn same_account_binding(
    account: &UiaiAccountIdentity,
    focusa_parent: &EntitlementSnapshot,
    uiai_grant: &EntitlementSnapshot,
) -> bool {
    account.valid()
        && focusa_parent.subject_id.as_deref() == Some(account.account_id.as_str())
        && uiai_grant.subject_id.as_deref() == Some(account.account_id.as_str())
}

/// Product isolation proof: the Focusa grant never satisfies UIAI scope.
/// The independent `uiai-engine` grant is the only source of UIAI features
/// and limits; a Focusa-only grant (or a Focusa grant claiming UIAI scope)
/// fails closed. When no Focusa parent is asserted, the exact-subset checks
/// still read only the independent UIAI grant.
pub fn focusa_grant_never_satisfies_uiai(
    focusa_parent: Option<&EntitlementSnapshot>,
    uiai_grant: &EntitlementSnapshot,
    requested_features: &BTreeSet<String>,
    requested_limits: &BTreeMap<String, u64>,
) -> bool {
    focusa_parent.is_none_or(|parent| parent.product == PRODUCT_FOCUSA)
        && uiai_grant.product == PRODUCT_UIAI_ENGINE
        && uiai_scope_is_exact_subset(uiai_grant, requested_features, requested_limits)
}

/// Fail-closed UIAI activation decision: same EDD account, server-owned
/// product mapping, independent UIAI grant, exact grants, product isolation.
///
/// `resolve_uiai_activation` never accepts a caller-controlled product,
/// price, grant, feature, limit, or commercial flag: the requested public
/// code must be the exact UIAI mapping and every requested feature/limit must
/// be an exact subset of the independent `uiai-engine` grant.
pub fn resolve_uiai_activation(
    account: &UiaiAccountIdentity,
    account_grants: &AccountProductGrants,
    requested_public_code: &str,
    uiai_grant: &EntitlementSnapshot,
    requested_features: &BTreeSet<String>,
    requested_limits: &BTreeMap<String, u64>,
    now: DateTime<Utc>,
) -> Result<UiaiGrantProjection, UiaiActivationError> {
    if !account.valid() {
        return Err(UiaiActivationError::AccountIdentityRequired);
    }
    if requested_public_code != PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1 {
        return Err(UiaiActivationError::ProductMappingRequired);
    }
    if !account_grants.has_product(PRODUCT_UIAI_ENGINE) {
        // Focusa-only (or grant-less) accounts cannot activate UIAI: an
        // independent UIAI grant is always required.
        return Err(UiaiActivationError::UiaiGrantRequired);
    }
    if uiai_grant.product != PRODUCT_UIAI_ENGINE || !grant_active_bound(uiai_grant, now) {
        return Err(UiaiActivationError::UiaiGrantInvalid);
    }
    let lease_id = uiai_grant
        .lease_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(UiaiActivationError::UiaiGrantInvalid)?;
    let sequence = uiai_grant
        .sequence
        .filter(|value| *value > 0)
        .ok_or(UiaiActivationError::UiaiGrantInvalid)?;
    if !focusa_grant_never_satisfies_uiai(None, uiai_grant, requested_features, requested_limits) {
        return Err(UiaiActivationError::ProductIsolationViolation);
    }
    Ok(UiaiGrantProjection {
        schema: UIAI_ACTIVATION_SCHEMA.to_string(),
        product: PRODUCT_UIAI_ENGINE.to_string(),
        public_code: PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1.to_string(),
        account: account.clone(),
        node_id: uiai_grant.node_id.clone(),
        grant_lease_id: lease_id.to_string(),
        grant_sequence: sequence,
        features: requested_features.clone(),
        limits: requested_limits.clone(),
    })
}

/// Exact-subset eligibility of a requested UIAI scope against the independent
/// `uiai-engine` grant (features present and enabled, limits positive and not
/// above the grant).
pub fn uiai_scope_is_exact_subset(
    uiai_grant: &EntitlementSnapshot,
    requested_features: &BTreeSet<String>,
    requested_limits: &BTreeMap<String, u64>,
) -> bool {
    requested_features
        .iter()
        .all(|feature| uiai_grant.features.get(feature).copied().unwrap_or(false))
        && requested_limits.iter().all(|(bucket, requested)| {
            *requested > 0 && *requested <= uiai_grant.limits.get(bucket).copied().unwrap_or(0)
        })
}

fn grant_active_bound(snapshot: &EntitlementSnapshot, now: DateTime<Utc>) -> bool {
    snapshot
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

    fn active_grant(
        product: &str,
        features: &[(&str, bool)],
        now: DateTime<Utc>,
    ) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(product, "node-001");
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = Some("account-001".to_string());
        snapshot.lease_id = Some(format!("lease-{product}"));
        snapshot.sequence = Some(11);
        snapshot.lease_digest = Some("sha256:uiai-grant-digest".to_string());
        snapshot.expires_at = Some(now + chrono::Duration::hours(1));
        for (feature, enabled) in features {
            snapshot.features.insert(feature.to_string(), *enabled);
        }
        snapshot.limits.insert("uiai_nodes".to_string(), 3);
        snapshot
    }

    fn account() -> UiaiAccountIdentity {
        UiaiAccountIdentity {
            account_id: "account-001".to_string(),
            edd_customer_id: 1001,
        }
    }

    #[test]
    fn focusa_only_account_cannot_activate_uiai() {
        let now = Utc::now();
        let grants = AccountProductGrants::new([PRODUCT_FOCUSA.to_string()]);
        assert!(grants.focusa_only());
        let uiai_grant = active_grant(PRODUCT_UIAI_ENGINE, &[("uiai.engine.core", true)], now);
        assert_eq!(
            resolve_uiai_activation(
                &account(),
                &grants,
                PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
                &uiai_grant,
                &BTreeSet::from(["uiai.engine.core".to_string()]),
                &BTreeMap::new(),
                now,
            ),
            Err(UiaiActivationError::UiaiGrantRequired),
        );
    }

    #[test]
    fn uiai_account_activates_exact_grants_without_duplicate_customer() {
        let now = Utc::now();
        let grants = AccountProductGrants::new([
            PRODUCT_FOCUSA.to_string(),
            PRODUCT_UIAI_ENGINE.to_string(),
        ]);
        assert!(!grants.focusa_only());
        let uiai_grant = active_grant(PRODUCT_UIAI_ENGINE, &[("uiai.engine.core", true)], now);
        let mut limits = BTreeMap::new();
        limits.insert("uiai_nodes".to_string(), 2);
        let projection = resolve_uiai_activation(
            &account(),
            &grants,
            PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
            &uiai_grant,
            &BTreeSet::from(["uiai.engine.core".to_string()]),
            &limits,
            now,
        )
        .expect("UIAI account activates exact grants");
        assert_eq!(projection.product, PRODUCT_UIAI_ENGINE);
        assert_eq!(
            projection.public_code,
            PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1
        );
        assert_eq!(projection.account, account());
        assert_eq!(projection.grant_lease_id, "lease-uiai-engine");
        assert_eq!(projection.grant_sequence, 11);
        // Single identity: exactly one account and one EDD customer appear in
        // the projection, so no duplicate customer identity is possible.
        assert_eq!(projection.account.account_id, "account-001");
        assert_eq!(projection.account.edd_customer_id, 1001);
    }

    #[test]
    fn wrong_requested_code_fails_closed() {
        let now = Utc::now();
        let grants = AccountProductGrants::new([
            PRODUCT_FOCUSA.to_string(),
            PRODUCT_UIAI_ENGINE.to_string(),
        ]);
        let uiai_grant = active_grant(PRODUCT_UIAI_ENGINE, &[], now);
        // The client can never steer the mapping to another product code.
        for wrong in [
            PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1,
            PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
            "uiai_operator_lifetime_v2",
            "",
        ] {
            assert_eq!(
                resolve_uiai_activation(
                    &account(),
                    &grants,
                    wrong,
                    &uiai_grant,
                    &BTreeSet::new(),
                    &BTreeMap::new(),
                    now,
                ),
                Err(UiaiActivationError::ProductMappingRequired),
            );
        }
    }

    #[test]
    fn exact_subset_isolation_and_invalid_grant_deny() {
        let now = Utc::now();
        let grants = AccountProductGrants::new([
            PRODUCT_FOCUSA.to_string(),
            PRODUCT_UIAI_ENGINE.to_string(),
        ]);
        let mut uiai_grant = active_grant(PRODUCT_UIAI_ENGINE, &[("uiai.engine.core", true)], now);
        // Feature not granted by the independent UIAI grant -> isolation.
        assert_eq!(
            resolve_uiai_activation(
                &account(),
                &grants,
                PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
                &uiai_grant,
                &BTreeSet::from(["focusa.agent.parallelism".to_string()]),
                &BTreeMap::new(),
                now,
            ),
            Err(UiaiActivationError::ProductIsolationViolation),
        );
        // Limit above the grant -> isolation.
        let mut limits = BTreeMap::new();
        limits.insert("uiai_nodes".to_string(), 99);
        assert_eq!(
            resolve_uiai_activation(
                &account(),
                &grants,
                PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
                &uiai_grant,
                &BTreeSet::new(),
                &limits,
                now,
            ),
            Err(UiaiActivationError::ProductIsolationViolation),
        );
        // Unactivated grant -> invalid.
        uiai_grant.state = EntitlementState::Unactivated;
        assert_eq!(
            resolve_uiai_activation(
                &account(),
                &grants,
                PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
                &uiai_grant,
                &BTreeSet::new(),
                &BTreeMap::new(),
                now,
            ),
            Err(UiaiActivationError::UiaiGrantInvalid),
        );
    }

    #[test]
    fn same_account_binding_is_strict_and_never_duplicates_customer() {
        let now = Utc::now();
        let mut focusa = active_grant(PRODUCT_FOCUSA, &[], now);
        let mut uiai = active_grant(PRODUCT_UIAI_ENGINE, &[], now);
        focusa.lease_id = Some("lease-focusa".to_string());
        assert!(same_account_binding(&account(), &focusa, &uiai));
        // Different account on the UIAI grant -> not the same EDD customer.
        uiai.subject_id = Some("account-002".to_string());
        assert!(!same_account_binding(&account(), &focusa, &uiai));
        // Missing identity fails closed.
        assert!(!same_account_binding(
            &UiaiAccountIdentity {
                account_id: String::new(),
                edd_customer_id: 0,
            },
            &focusa,
            &uiai,
        ));
    }
}
