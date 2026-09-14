//! One-account Focusa + UIAI bundle activation (Spec 152E §8 product/grant
//! registry, §21 shared activation client, §23 acceptance matrix "Bundle
//! purchase"; Spec 172 §4.1 price, §7.3 shared operator nodes, §9 bundle
//! composition; Specs 152, 150A, 152A-D; Spec 158 implementation excluded).
//!
//! The bundle is ONE commerce SKU/key: `focusa_uiai_operator_bundle_lifetime_v1`
//! at the frozen server-owned price USD 1254.60 grants the EXACT union of the
//! two underlying Operator v1 License Types (`focusa_operator_lifetime_v1` +
//! `uiai_operator_lifetime_v1`). It uses one verified account, one EDD order,
//! and one canonical human-facing EDD Software Licensing key; the signed
//! lease/child-token system carries explicit Focusa and UIAI product grants
//! on the SAME shared operator node identities (three shared nodes — never
//! six unrelated activations). There is no third independent feature catalog
//! and future products never enter the bundle automatically.
//!
//! Orchestration is atomic-or-typed-partial: a bundle purchase either
//! activates BOTH exact products on the one account or returns a typed
//! recoverable partial state that reuses the same order/registration handle —
//! no duplicate payment, no second account, no second license, no silent
//! partial success. The mapping is server-owned: clients submit only the
//! public bundle code; EDD ids, prices, tiers, grants, limits, and commercial
//! flags are never accepted as inputs.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::authority::{EntitlementSnapshot, EntitlementState};
use crate::uiai_activation::{
    AccountProductGrants, PRODUCT_FOCUSA, PRODUCT_UIAI_ENGINE,
    PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1, PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
    PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1, UiaiAccountIdentity, same_account_binding,
};

pub const BUNDLE_ACTIVATION_SCHEMA: &str = "focusa.bundle_activation_contract.v1";
pub const BUNDLE_ORDER_POLICY_SCHEMA: &str = "focusa.bundle_order_policy.v1";
pub const BUNDLE_PARTIAL_SCHEMA: &str = "focusa.bundle_partial_activation.v1";

/// Frozen server-owned bundle policy (Spec 172 §4.1, §9; frozen registry
/// `docs/contracts/spec152e-edd-product-registry.v1.json` bundle offer:
/// price 1254.60, one operator seat, three shared operator nodes, exact
/// union, whole-order refunds only, no third feature catalog).
pub const BUNDLE_PRICE_USD: &str = "1254.60";
pub const BUNDLE_PRICE_MINOR_UNITS: u64 = 125_460;
pub const BUNDLE_PRICE_AUTHORITY: &str = "spec172_server_owned";
pub const BUNDLE_GRANT_COMPOSITION: &str = "exact_union";
pub const BUNDLE_NODE_LIMIT: u32 = 3;
pub const BUNDLE_NODE_SET: &str = "operator_shared_v1";
pub const BUNDLE_OPERATOR_SEATS: u32 = 1;
pub const BUNDLE_REFUND_POLICY: &str = "whole_order_30_days";

/// The exact two underlying grants of the bundle SKU (Spec 172 §9.1). The
/// bundle never carries a third feature list and future products never enter
/// it automatically (§9.4).
pub const BUNDLE_GRANTS: [&str; 2] = [
    PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1,
    PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
];

/// EDD bundle order/item/license policy (Spec 172 §9.2/§9.3): one EDD order,
/// one canonical human key, exact-union grants, whole-order refunds only in
/// v1 (component partial refunds unsupported), future products excluded.
/// Server-owned; clients can never submit or override any of these values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleOrderPolicy {
    pub schema: String,
    pub public_code: String,
    pub grants: Vec<String>,
    pub grant_composition: String,
    pub price_usd: String,
    pub price_minor_units: u64,
    pub price_authority: String,
    pub operator_seats: u32,
    pub node_limit: u32,
    pub node_set: String,
    pub license_duration: String,
    pub one_edd_order: bool,
    pub one_human_key: bool,
    pub component_refunds_allowed: bool,
    pub refund_policy: String,
    pub future_products_included: bool,
    pub third_feature_catalog: bool,
}

/// Resolve the frozen bundle order/item/license policy by public product code
/// only. Any other code — Focusa, UIAI, unknown, or prefixed — fails closed;
/// the client can never steer the mapping or the price.
pub fn resolve_bundle_order_policy(
    public_code: &str,
) -> Result<BundleOrderPolicy, BundleActivationError> {
    if public_code != PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1 {
        return Err(BundleActivationError::ProductMappingRequired);
    }
    Ok(BundleOrderPolicy {
        schema: BUNDLE_ORDER_POLICY_SCHEMA.to_string(),
        public_code: PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1.to_string(),
        grants: BUNDLE_GRANTS
            .iter()
            .map(|grant| grant.to_string())
            .collect(),
        grant_composition: BUNDLE_GRANT_COMPOSITION.to_string(),
        price_usd: BUNDLE_PRICE_USD.to_string(),
        price_minor_units: BUNDLE_PRICE_MINOR_UNITS,
        price_authority: BUNDLE_PRICE_AUTHORITY.to_string(),
        operator_seats: BUNDLE_OPERATOR_SEATS,
        node_limit: BUNDLE_NODE_LIMIT,
        node_set: BUNDLE_NODE_SET.to_string(),
        license_duration: "lifetime".to_string(),
        one_edd_order: true,
        one_human_key: true,
        component_refunds_allowed: false,
        refund_policy: BUNDLE_REFUND_POLICY.to_string(),
        future_products_included: false,
        third_feature_catalog: false,
    })
}

/// One exact product grant inside the bundle, derived ONLY from the signed
/// authority lease for that product. Feature/limit values come from the two
/// underlying License Type records — there is no third hand-copied list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleGrantProjection {
    pub schema: String,
    pub product: String,
    pub public_code: String,
    pub account: UiaiAccountIdentity,
    pub node_id: String,
    pub grant_lease_id: String,
    pub grant_sequence: u64,
    pub grant_lease_digest: String,
    pub features: BTreeSet<String>,
    pub limits: BTreeMap<String, u64>,
}

/// Full bundle activation projection: BOTH exact product grants settle on the
/// one verified account, one EDD order, one canonical human key, on the SAME
/// shared operator node identities. Atomic delivery — no partial grants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleActivationProjection {
    pub schema: String,
    pub public_code: String,
    pub account: UiaiAccountIdentity,
    pub order_handle: String,
    pub registration_id: String,
    pub node_id: String,
    pub posture: String,
    pub price_usd: String,
    pub price_authority: String,
    pub order_policy: BundleOrderPolicy,
    pub focusa: BundleGrantProjection,
    pub uiai_engine: BundleGrantProjection,
    /// The shared operator-node identities both products bind to (Spec 172
    /// §7.3: three shared nodes, never six unrelated activations). At
    /// activation time this is the current shared node; the adapter enforces
    /// that both grants bind the same identity.
    pub shared_node_identities: Vec<String>,
}

/// One grant that could not settle, with the typed reason and safe recovery
/// action. Never silently skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingBundleGrant {
    pub product: String,
    pub reason: String,
    pub retryable: bool,
    pub resume_action: String,
}

/// Typed recoverable partial state (Spec 172 §9.2): returned instead of a
/// silent half-activation when one underlying grant has not settled. It
/// reuses the SAME order/registration handle — no duplicate payment, no
/// second account, no second license — and identifies exactly which grant is
/// pending and how to recover (resume the same poll/order or authority
/// review).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePartialActivation {
    pub schema: String,
    pub public_code: String,
    pub account: UiaiAccountIdentity,
    pub order_handle: String,
    pub registration_id: String,
    pub node_id: String,
    pub settled_grants: BTreeSet<String>,
    pub pending_grants: Vec<PendingBundleGrant>,
    pub recovery_action: String,
    pub no_duplicate_payment: bool,
    pub no_duplicate_license: bool,
    pub one_edd_order: bool,
    pub one_account: bool,
}

/// Bundle orchestration result: BOTH exact products activated, or the typed
/// recoverable partial state. There is no third silent outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
// Boxing would alter the frozen public enum construction contract.
#[allow(clippy::large_enum_variant)]
pub enum BundleActivationOutcome {
    Activated(BundleActivationProjection),
    RecoverablePartial(BundlePartialActivation),
}

/// Fail-closed bundle activation errors. Every denial returns a typed safe
/// reason; nothing is silently normalized into a grant.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BundleActivationError {
    #[error("the requested public product code is not the bundle product mapping")]
    ProductMappingRequired,
    #[error(
        "the bundle requires both the focusa and uiai-engine grants on the one verified account"
    )]
    BundleGrantRequired,
    #[error("a verified EDD account identity is required")]
    AccountIdentityRequired,
    #[error("the bundle lease subjects do not all equal the one verified EDD account")]
    BundleAccountMismatch,
    #[error(
        "the two bundle grants are bound to different node identities; shared operator nodes are required"
    )]
    SharedNodeIdentityViolation,
    #[error("the bundle grant resolution is incomplete; typed recoverable partial state returned")]
    Incomplete,
}

/// One verified EDD account, one EDD order, one canonical human key (Spec 172
/// §9.2): a bundle purchase either activates BOTH exact products on the one
/// account or returns the typed recoverable partial state. The client submits
/// only the public bundle code; EDD ids, prices, tiers, grants, limits, and
/// commercial flags are never accepted — the two grant projections are
/// derived entirely from the signed authority leases.
///
/// `order_handle` and `registration_id` identify the one EDD order and the
/// one registration; the partial state reuses them so recovery never creates
/// a duplicate payment, account, or license.
#[allow(clippy::too_many_arguments)]
pub fn resolve_bundle_activation(
    account: &UiaiAccountIdentity,
    account_grants: &AccountProductGrants,
    requested_public_code: &str,
    focusa_grant: &EntitlementSnapshot,
    uiai_grant: &EntitlementSnapshot,
    order_handle: &str,
    registration_id: &str,
    now: DateTime<Utc>,
) -> Result<BundleActivationOutcome, BundleActivationError> {
    if !account.valid() {
        return Err(BundleActivationError::AccountIdentityRequired);
    }
    if requested_public_code != PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1 {
        return Err(BundleActivationError::ProductMappingRequired);
    }
    // The bundle requires BOTH exact product grants on the one account.
    if !account_grants.has_product(PRODUCT_FOCUSA)
        || !account_grants.has_product(PRODUCT_UIAI_ENGINE)
    {
        return Err(BundleActivationError::BundleGrantRequired);
    }
    // One verified EDD account for both leases: no duplicate customer
    // identity and no bridging of two customers through the bundle.
    if !same_account_binding(account, focusa_grant, uiai_grant) {
        return Err(BundleActivationError::BundleAccountMismatch);
    }
    // Shared operator node identities (Spec 172 §7.3): both products bind the
    // same node instead of creating unrelated activations.
    if focusa_grant.node_id != uiai_grant.node_id {
        return Err(BundleActivationError::SharedNodeIdentityViolation);
    }
    let order_policy = resolve_bundle_order_policy(requested_public_code)?;
    let focusa_ready =
        focusa_grant.product == PRODUCT_FOCUSA && grant_active_bound(focusa_grant, now);
    let uiai_ready =
        uiai_grant.product == PRODUCT_UIAI_ENGINE && grant_active_bound(uiai_grant, now);
    if focusa_ready && uiai_ready {
        let node_id = focusa_grant.node_id.clone();
        return Ok(BundleActivationOutcome::Activated(
            BundleActivationProjection {
                schema: BUNDLE_ACTIVATION_SCHEMA.to_string(),
                public_code: requested_public_code.to_string(),
                account: account.clone(),
                order_handle: order_handle.to_string(),
                registration_id: registration_id.to_string(),
                node_id: node_id.clone(),
                posture: "bundle".to_string(),
                price_usd: order_policy.price_usd.clone(),
                price_authority: order_policy.price_authority.clone(),
                order_policy,
                focusa: grant_projection(
                    BUNDLE_ACTIVATION_SCHEMA,
                    PRODUCT_FOCUSA,
                    PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1,
                    account,
                    focusa_grant,
                ),
                uiai_engine: grant_projection(
                    BUNDLE_ACTIVATION_SCHEMA,
                    PRODUCT_UIAI_ENGINE,
                    PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
                    account,
                    uiai_grant,
                ),
                shared_node_identities: vec![node_id],
            },
        ));
    }
    // Typed recoverable partial state: never a silent partial success, and
    // recovery reuses the SAME order and registration.
    let mut settled = BTreeSet::new();
    let mut pending = Vec::new();
    if focusa_ready {
        settled.insert(PRODUCT_FOCUSA.to_string());
    } else {
        pending.push(pending_grant(PRODUCT_FOCUSA, focusa_grant, now));
    }
    if uiai_ready {
        settled.insert(PRODUCT_UIAI_ENGINE.to_string());
    } else {
        pending.push(pending_grant(PRODUCT_UIAI_ENGINE, uiai_grant, now));
    }
    let recovery_action = if pending.iter().all(|grant| grant.retryable) {
        "resume_poll_same_order"
    } else {
        "authority_review"
    };
    Ok(BundleActivationOutcome::RecoverablePartial(
        BundlePartialActivation {
            schema: BUNDLE_PARTIAL_SCHEMA.to_string(),
            public_code: requested_public_code.to_string(),
            account: account.clone(),
            order_handle: order_handle.to_string(),
            registration_id: registration_id.to_string(),
            node_id: focusa_grant.node_id.clone(),
            settled_grants: settled,
            pending_grants: pending,
            recovery_action: recovery_action.to_string(),
            no_duplicate_payment: true,
            no_duplicate_license: true,
            one_edd_order: true,
            one_account: true,
        },
    ))
}

fn grant_projection(
    schema: &str,
    product: &str,
    public_code: &str,
    account: &UiaiAccountIdentity,
    snapshot: &EntitlementSnapshot,
) -> BundleGrantProjection {
    BundleGrantProjection {
        schema: schema.to_string(),
        product: product.to_string(),
        public_code: public_code.to_string(),
        account: account.clone(),
        node_id: snapshot.node_id.clone(),
        grant_lease_id: snapshot.lease_id.clone().unwrap_or_default(),
        grant_sequence: snapshot.sequence.unwrap_or(0),
        grant_lease_digest: snapshot.lease_digest.clone().unwrap_or_default(),
        features: snapshot
            .features
            .iter()
            .filter(|(_, enabled)| **enabled)
            .map(|(feature, _)| feature.clone())
            .collect(),
        limits: snapshot
            .limits
            .iter()
            .filter(|(_, limit)| **limit > 0)
            .map(|(bucket, limit)| (bucket.clone(), *limit))
            .collect(),
    }
}

fn pending_grant(
    product: &str,
    snapshot: &EntitlementSnapshot,
    now: DateTime<Utc>,
) -> PendingBundleGrant {
    if snapshot.product != product {
        PendingBundleGrant {
            product: product.to_string(),
            reason: "grant_product_mismatch".to_string(),
            retryable: false,
            resume_action: "authority_review".to_string(),
        }
    } else if !grant_active_bound(snapshot, now) {
        PendingBundleGrant {
            product: product.to_string(),
            reason: "grant_inactive_or_unbound".to_string(),
            retryable: true,
            resume_action: "resume_poll_same_order".to_string(),
        }
    } else {
        PendingBundleGrant {
            product: product.to_string(),
            reason: "grant_pending_delivery".to_string(),
            retryable: true,
            resume_action: "resume_poll_same_order".to_string(),
        }
    }
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

    fn active_grant(product: &str, now: DateTime<Utc>) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(product, "node-001");
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = Some("account-001".to_string());
        snapshot.lease_id = Some(format!("lease-{product}"));
        snapshot.sequence = Some(11);
        snapshot.lease_digest = Some("sha256:bundle-grant-digest".to_string());
        snapshot.expires_at = Some(now + chrono::Duration::hours(1));
        snapshot.features.insert(format!("{product}.core"), true);
        snapshot.limits.insert("operator_nodes".to_string(), 3);
        snapshot
    }

    fn account() -> UiaiAccountIdentity {
        UiaiAccountIdentity {
            account_id: "account-001".to_string(),
            edd_customer_id: 1001,
        }
    }

    fn bundle_grants() -> AccountProductGrants {
        AccountProductGrants::new([PRODUCT_FOCUSA.to_string(), PRODUCT_UIAI_ENGINE.to_string()])
    }

    #[test]
    fn bundle_order_policy_is_exact_two_product_union_at_frozen_price() {
        let policy = resolve_bundle_order_policy(PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1)
            .expect("bundle policy resolves from the exact public code");
        assert_eq!(
            policy.grants,
            vec![
                PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1.to_string(),
                PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1.to_string(),
            ]
        );
        assert_eq!(policy.price_usd, BUNDLE_PRICE_USD);
        assert_eq!(policy.price_minor_units, 125_460);
        assert_eq!(policy.grant_composition, "exact_union");
        assert_eq!(policy.operator_seats, 1);
        assert_eq!(policy.node_limit, 3);
        assert_eq!(policy.node_set, "operator_shared_v1");
        assert!(policy.one_edd_order && policy.one_human_key);
        assert!(!policy.component_refunds_allowed);
        assert!(!policy.future_products_included);
        assert!(!policy.third_feature_catalog);
        // Wrong codes can never steer the mapping or the price.
        for wrong in [
            PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1,
            PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
            "focusa_uiai_operator_bundle_lifetime_v2",
            "",
        ] {
            assert_eq!(
                resolve_bundle_order_policy(wrong),
                Err(BundleActivationError::ProductMappingRequired),
            );
        }
    }

    #[test]
    fn bundle_activates_both_exact_products_on_one_account() {
        let now = Utc::now();
        let focusa = active_grant(PRODUCT_FOCUSA, now);
        let uiai = active_grant(PRODUCT_UIAI_ENGINE, now);
        let outcome = resolve_bundle_activation(
            &account(),
            &bundle_grants(),
            PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
            &focusa,
            &uiai,
            "edd-order-4001",
            "registration-9001",
            now,
        )
        .expect("bundle activates");
        match outcome {
            BundleActivationOutcome::Activated(projection) => {
                assert_eq!(projection.account, account());
                assert_eq!(projection.posture, "bundle");
                assert_eq!(projection.price_usd, "1254.60");
                assert_eq!(projection.focusa.product, PRODUCT_FOCUSA);
                assert_eq!(projection.uiai_engine.product, PRODUCT_UIAI_ENGINE);
                assert_eq!(projection.focusa.grant_lease_id, "lease-focusa");
                assert_eq!(projection.uiai_engine.grant_lease_id, "lease-uiai-engine");
                assert_eq!(projection.focusa.node_id, projection.uiai_engine.node_id);
                assert_eq!(
                    projection.shared_node_identities,
                    vec!["node-001".to_string()]
                );
                // One account + one EDD customer appear in both grants.
                assert_eq!(projection.focusa.account.edd_customer_id, 1001);
                assert_eq!(projection.uiai_engine.account.edd_customer_id, 1001);
            }
            BundleActivationOutcome::RecoverablePartial(_) => {
                panic!("both grants active must activate, not partial")
            }
        }
    }

    #[test]
    fn bundle_wrong_code_or_missing_grant_fails_closed() {
        let now = Utc::now();
        let focusa = active_grant(PRODUCT_FOCUSA, now);
        let uiai = active_grant(PRODUCT_UIAI_ENGINE, now);
        for wrong in [
            PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1,
            PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
            "focusa_uiai_operator_bundle_lifetime_v2",
            "",
        ] {
            assert_eq!(
                resolve_bundle_activation(
                    &account(),
                    &bundle_grants(),
                    wrong,
                    &focusa,
                    &uiai,
                    "edd-order-4001",
                    "registration-9001",
                    now,
                ),
                Err(BundleActivationError::ProductMappingRequired),
            );
        }
        // A Focusa-only (or grant-less) account can never activate the bundle.
        let focusa_only = AccountProductGrants::new([PRODUCT_FOCUSA.to_string()]);
        assert!(focusa_only.focusa_only());
        assert_eq!(
            resolve_bundle_activation(
                &account(),
                &focusa_only,
                PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
                &focusa,
                &uiai,
                "edd-order-4001",
                "registration-9001",
                now,
            ),
            Err(BundleActivationError::BundleGrantRequired),
        );
    }

    #[test]
    fn bundle_partial_state_is_typed_recoverable_and_never_duplicates() {
        let now = Utc::now();
        let focusa = active_grant(PRODUCT_FOCUSA, now);
        // UIAI grant not yet settled (still unactivated but issued to the
        // same verified EDD account).
        let mut uiai_pending = EntitlementSnapshot::unactivated(PRODUCT_UIAI_ENGINE, "node-001");
        uiai_pending.subject_id = Some("account-001".to_string());
        let outcome = resolve_bundle_activation(
            &account(),
            &bundle_grants(),
            PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
            &focusa,
            &uiai_pending,
            "edd-order-4001",
            "registration-9001",
            now,
        )
        .expect("typed partial state is returned, not an error");
        match outcome {
            BundleActivationOutcome::RecoverablePartial(partial) => {
                assert_eq!(
                    partial.settled_grants,
                    BTreeSet::from(["focusa".to_string()])
                );
                assert_eq!(partial.pending_grants.len(), 1);
                assert_eq!(partial.pending_grants[0].product, PRODUCT_UIAI_ENGINE);
                assert_eq!(
                    partial.pending_grants[0].reason,
                    "grant_inactive_or_unbound"
                );
                assert!(partial.pending_grants[0].retryable);
                assert_eq!(partial.recovery_action, "resume_poll_same_order");
                // Recovery reuses the SAME order and registration: no
                // duplicate payment, account, or license.
                assert_eq!(partial.order_handle, "edd-order-4001");
                assert_eq!(partial.registration_id, "registration-9001");
                assert!(partial.no_duplicate_payment);
                assert!(partial.no_duplicate_license);
                assert!(partial.one_edd_order);
                assert!(partial.one_account);
            }
            BundleActivationOutcome::Activated(_) => {
                panic!("unsettled UIAI grant must return typed partial state")
            }
        }
        // A product mismatch is non-retryable and routes to authority review.
        let wrong = active_grant("other-product", now);
        let outcome = resolve_bundle_activation(
            &account(),
            &bundle_grants(),
            PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
            &wrong,
            &uiai_pending,
            "edd-order-4001",
            "registration-9001",
            now,
        )
        .expect("typed partial state returned");
        if let BundleActivationOutcome::RecoverablePartial(partial) = outcome {
            assert_eq!(partial.recovery_action, "authority_review");
            // The product mismatch is non-retryable; the still-unsettled UIAI
            // grant stays retryable. Either way recovery is typed and never
            // silently skipped.
            assert!(partial.pending_grants.iter().any(|grant| !grant.retryable));
            assert!(
                partial
                    .pending_grants
                    .iter()
                    .any(|grant| grant.reason == "grant_product_mismatch")
            );
        } else {
            panic!("expected recoverable partial");
        }
    }

    #[test]
    fn bundle_same_account_binding_and_shared_nodes_are_strict() {
        let now = Utc::now();
        let focusa = active_grant(PRODUCT_FOCUSA, now);
        let mut uiai = active_grant(PRODUCT_UIAI_ENGINE, now);
        // Different EDD account on the UIAI grant -> account mismatch.
        uiai.subject_id = Some("account-002".to_string());
        assert_eq!(
            resolve_bundle_activation(
                &account(),
                &bundle_grants(),
                PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
                &focusa,
                &uiai,
                "edd-order-4001",
                "registration-9001",
                now,
            ),
            Err(BundleActivationError::BundleAccountMismatch),
        );
        // Different node bindings -> shared-node violation, never six
        // unrelated activations.
        uiai = active_grant(PRODUCT_UIAI_ENGINE, now);
        uiai.node_id = "node-002".to_string();
        assert_eq!(
            resolve_bundle_activation(
                &account(),
                &bundle_grants(),
                PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
                &focusa,
                &uiai,
                "edd-order-4001",
                "registration-9001",
                now,
            ),
            Err(BundleActivationError::SharedNodeIdentityViolation),
        );
        // Missing identity fails closed.
        assert_eq!(
            resolve_bundle_activation(
                &UiaiAccountIdentity {
                    account_id: String::new(),
                    edd_customer_id: 0,
                },
                &bundle_grants(),
                PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1,
                &focusa,
                &uiai,
                "edd-order-4001",
                "registration-9001",
                now,
            ),
            Err(BundleActivationError::AccountIdentityRequired),
        );
    }
}
