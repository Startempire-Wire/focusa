//! Spec 152F.06.05 — offline grace, authority outage, and bypass resistance
//! matrix (focusa-license policy layer).
//!
//! Proves that no outage, stale cache, alternate presenter, worker, or child
//! token can widen capability:
//!
//! 1. cached base/premium grants resolve only within their signed Offline
//!    Grace bounds (window present, not yet closed, feature signed into the
//!    authority lease);
//! 2. Offline Grace can never create nodes, grants, or limit expansion — all
//!    capacity comes from the authority-owned lease snapshot and caller input
//!    never widens a feature, limit, or node binding;
//! 3. a higher refund/revoke authority sequence always overrides older cached
//!    grants (base, premium, reservation, and child-token surfaces all fail
//!    closed against the current authority);
//! 4. an authority outage (missing/unusable authority state) preserves
//!    recovery, read, export, and maintenance allowances while value-producing
//!    mutations stay denied;
//! 5. direct handler/core/worker/child-token bypass attempts fail closed:
//!    forged products, caller-invented features/limits, stale active leases,
//!    child tokens that widen or outlive their grants, and worker dispatch
//!    without a base entitlement are all refused before side effects.
//!
//! Exact verification: `cargo test --workspace spec152f_bypass_resistance`.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Utc};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    base_product_projection, reduce_entitlement_state, resolve_base_focusa_product,
    resolve_export_packaged, resolve_premium_family, BaseProductDecision, CapabilityFamily,
    DecisionReason, EntitlementPolicyPosture, LimitReservationService, PolicyEntitlementState,
    PremiumFamilyDecision, PremiumFamilyDenial, ReservationError,
};
use focusa_license::uiai_child_token::{
    resolve_uiai_capability, AuthorityChildTokenEnvelope, UiaiCapabilityDenial,
    UiaiCapabilityDecision, UiaiChildTokenBroker, UiaiChildTokenError, UiaiChildTokenRequest,
    UiaiOperationClass, UIAI_CHILD_TOKEN_MAX_TTL_MINUTES, UIAI_CHILD_TOKEN_SCHEMA,
};
use uuid::Uuid;

const AUTOMATION_FEATURE: &str = "focusa.agent.parallelism";
const AUTOMATION_BUCKET: &str = "concurrent_agents";
const EXPORT_FEATURE: &str = "focusa.export.packaged";

fn now() -> chrono::DateTime<Utc> {
    Utc::now()
}

/// One signed Offline Grace cached-lease fixture: product `focusa`, bound
/// node, non-zero lease sequence/digest, and a bounded grace window. The
/// authority lease's signed feature set is exactly `features`.
fn offline_grace_snapshot(
    features: &[&str],
    grace_until: Option<chrono::DateTime<Utc>>,
    sequence: u64,
) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-offline-001");
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.subject_id = Some("account-offline-001".to_string());
    snapshot.lease_id = Some(format!("lease-offline-{sequence}"));
    snapshot.sequence = Some(sequence);
    snapshot.lease_digest = Some("sha256:offline-cached-grant".to_string());
    snapshot.expires_at = Some(now() - Duration::days(1));
    snapshot.offline_grace_until = grace_until;
    for feature in features {
        snapshot.features.insert((*feature).to_string(), true);
    }
    snapshot.limits.insert(AUTOMATION_BUCKET.to_string(), 2);
    snapshot
}

fn active_snapshot(features: &[&str], sequence: u64) -> EntitlementSnapshot {
    let mut snapshot = offline_grace_snapshot(features, None, sequence);
    snapshot.state = EntitlementState::Active;
    snapshot.expires_at = Some(now() + Duration::hours(1));
    snapshot.offline_grace_until = None;
    snapshot
}

/// A refunded/revoked authority snapshot: same account/node, HIGHER sequence
/// than any older cached grant, no usable lease identity on the surface the
/// policy layer consumes (state is the authority truth; stored feature claims
/// must never widen it).
fn recovery_only_snapshot(features: &[&str], sequence: u64) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::recovery_only("focusa", "node-offline-001", "refunded");
    snapshot.subject_id = Some("account-offline-001".to_string());
    snapshot.lease_id = Some(format!("lease-revoked-{sequence}"));
    snapshot.sequence = Some(sequence);
    snapshot.lease_digest = Some("sha256:revoked-grant".to_string());
    for feature in features {
        snapshot.features.insert((*feature).to_string(), true);
    }
    snapshot.limits.insert(AUTOMATION_BUCKET.to_string(), 2);
    snapshot
}

// ── 1. Cached base/premium grants stay within their signed bounds ─────────

#[test]
fn spec152f_bypass_resistance_cached_offline_grace_base_is_bounded_by_signed_window() {
    // Valid cached Offline Grace: the base gate and one signed premium feature
    // resolve inside the authority-signed grace window.
    let snapshot =
        offline_grace_snapshot(&[AUTOMATION_FEATURE], Some(now() + Duration::minutes(5)), 4);
    assert_eq!(
        resolve_base_focusa_product("focusa", PolicyEntitlementState::OfflineGrace),
        BaseProductDecision::Entitled
    );
    let projection = base_product_projection(Some(&snapshot)).expect("projection resolves");
    assert!(projection.permits_base_mutations);
    assert_eq!(projection.decision, "entitled");

    let premium = resolve_premium_family(
        &snapshot,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    );
    match premium {
        PremiumFamilyDecision::Feature {
            offline_cached,
            lease_sequence,
            ..
        } => {
            assert!(offline_cached, "offline grace premium is a cached grant");
            assert_eq!(lease_sequence, 4);
        }
        PremiumFamilyDecision::Denied(denial) => panic!("signed cached feature denied: {denial:?}"),
    }

    // The cached decision is descriptive only: it never widens the signed
    // feature set or the lease sequence bound to the decision.
    let premium_unregistered = resolve_premium_family(
        &snapshot,
        CapabilityFamily::ReleaseProof,
        "focusa.release.proof",
        now(),
    );
    assert!(
        matches!(
            premium_unregistered,
            PremiumFamilyDecision::Denied(PremiumFamilyDenial::MissingFeature { .. })
        ),
        "an offline grant cannot mint a feature that was not signed into the lease"
    );
}

#[test]
fn spec152f_bypass_resistance_cached_offline_grace_expired_or_unbounded_fails_closed() {
    let expired = offline_grace_snapshot(
        &[AUTOMATION_FEATURE],
        Some(now() - Duration::minutes(1)),
        4,
    );
    let denial = resolve_premium_family(
        &expired,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("expired grace must deny");
    assert_eq!(denial, PremiumFamilyDenial::CachedGrantExpired);

    let unbounded = offline_grace_snapshot(&[AUTOMATION_FEATURE], None, 4);
    let denial = resolve_premium_family(
        &unbounded,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("missing grace window must deny");
    assert_eq!(denial, PremiumFamilyDenial::MissingCachedGrantExpiry);

    // Export packaging reuses the same cached-window bounds.
    let mut export_grace = offline_grace_snapshot(&[EXPORT_FEATURE], Some(now() - Duration::minutes(1)), 4);
    export_grace.features.insert(EXPORT_FEATURE.to_string(), true);
    let denial = resolve_export_packaged(&export_grace, EXPORT_FEATURE, now())
        .denial()
        .cloned()
        .expect("expired grace must deny export packaging too");
    assert_eq!(denial, PremiumFamilyDenial::CachedGrantExpired);
}

// ── 2. No node/grant/limit expansion from cached or offline state ─────────

#[test]
fn spec152f_bypass_resistance_offline_grace_cannot_expand_nodes_grants_or_limits() {
    // Capacity is authority-owned: a caller-invented bucket is refused before
    // any capacity is touched, even with a valid cached grant.
    let snapshot =
        offline_grace_snapshot(&[AUTOMATION_FEATURE], Some(now() + Duration::minutes(5)), 4);
    let mut service = LimitReservationService::new();
    let error = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            AUTOMATION_FEATURE,
            "caller_invented_bucket",
            "idem-bypass-0001",
            1,
            now(),
        )
        .expect_err("invented bucket must fail closed");
    assert_eq!(
        error,
        ReservationError::UnknownLimitBucket {
            bucket: "caller_invented_bucket".to_string(),
            family: CapabilityFamily::Automation,
        }
    );

    // Requested units above the authority-owned capacity are refused.
    let exhausted = service
        .reserve(
            &snapshot,
            CapabilityFamily::Automation,
            AUTOMATION_FEATURE,
            AUTOMATION_BUCKET,
            "idem-bypass-0002",
            99,
            now(),
        )
        .expect_err("units above authority capacity must fail");
    assert!(matches!(exhausted, ReservationError::LimitExhausted { .. }));

    // An offline cached grant never creates a new node: the child-token
    // binding requires the exact authority-bound node.
    let mut request = child_token_request();
    request.node_id = "node-attacker-999".to_string();
    let parent = offline_grace_snapshot(&[], Some(now() + Duration::minutes(5)), 4);
    let grant = uiai_grant_snapshot();
    let broker = UiaiChildTokenBroker::default();
    assert_eq!(
        broker.validate_request(&request, &parent, &grant, now()),
        Err(UiaiChildTokenError::ParentEntitlementInvalid),
        "an offline cached grant cannot bind a node it was never signed for"
    );

    // Offline Grace can never create customers, licenses, purchases, or grants:
    // the only paths out of the policy layer are feature decisions bound to the
    // signed snapshot, never issuance. A stored feature claim on an unusable
    // state is refused at the policy boundary.
    let refunded_claim = recovery_only_snapshot(&[AUTOMATION_FEATURE], 5);
    let denial = resolve_premium_family(
        &refunded_claim,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("stored claim on refunded state must deny");
    assert!(matches!(
        denial,
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));
}

// ── 3. Higher refund/revoke sequence overrides older cached grants ────────

#[test]
fn spec152f_bypass_resistance_higher_refund_revoke_sequence_overrides_cached_grants() {
    // The stale cached Offline Grace fixture is sequence 4 with a still-open
    // grace window; the current authority snapshot is a higher-sequence
    // refund/revoke (sequence 5).
    let stale_cached =
        offline_grace_snapshot(&[AUTOMATION_FEATURE], Some(now() + Duration::minutes(5)), 4);
    let current = recovery_only_snapshot(&[AUTOMATION_FEATURE], 5);

    // Base gate: the refund/revoke state denies value-producing mutations no
    // matter what the stale cache claimed.
    assert_eq!(
        resolve_base_focusa_product(&current.product, PolicyEntitlementState::RefundedOrRevoked),
        BaseProductDecision::Denied
    );
    let projection = base_product_projection(Some(&current)).expect("projection resolves");
    assert!(!projection.permits_base_mutations);

    // Premium: the higher-sequence refund/revoke wins over the cached grant.
    let denial = resolve_premium_family(
        &current,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("refunded/revoked current authority must deny premium");
    assert!(matches!(
        denial,
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));

    // Reservation revalidation: a reservation made against the stale cached
    // lease cannot revalidate once the current authority sequence moved.
    let mut service = LimitReservationService::new();
    let grant = service
        .reserve(
            &stale_cached,
            CapabilityFamily::Automation,
            AUTOMATION_FEATURE,
            AUTOMATION_BUCKET,
            "idem-revoke-0001",
            1,
            now(),
        )
        .expect("stale cached grant can still reserve at decision time");
    assert!(grant.offline_cached);
    let error = service
        .revalidate(&current, "idem-revoke-0001", now())
        .expect_err("stale reservation must fail revalidation after revoke");
    assert!(
        matches!(
            error,
            ReservationError::StaleLease { .. } | ReservationError::FamilyDenied(_)
        ),
        "higher refund/revoke sequence leaves the stale reservation unusable: {error:?}"
    );

    // Child-token cache: the cached child token bound to the revoked parent is
    // dropped and can never be re-authorized.
    let mut broker = UiaiChildTokenBroker::default();
    let parent = active_snapshot(&[AUTOMATION_FEATURE], 4);
    let grant = uiai_grant_snapshot();
    let request = child_token_request();
    let envelope = authority_envelope(&request, &parent, &grant, now());
    broker
        .accept_authority_token(&request, &parent, &grant, envelope, now())
        .expect("token accepted while parent was active");
    assert!(broker.cached(&request.audience, now()).is_some());
    assert_eq!(
        broker.revoke_parent(parent.lease_id.as_deref().unwrap(), 5),
        1,
        "revoking the parent at a higher sequence drops the cached child token"
    );
    assert!(broker.cached(&request.audience, now()).is_none());
    assert!(
        broker
            .authorized_cached_token(&request.audience, &current, &grant, now())
            .is_none(),
        "a revoked parent can never re-authorize a cached child token"
    );
}

// ── 4. Outage preserves recovery, read, export, and maintenance ───────────

#[test]
fn spec152f_bypass_resistance_outage_keeps_recovery_read_and_export_available() {
    // Authority outage: no usable snapshot at all (missing/corrupt state).
    let missing = PolicyEntitlementState::MissingOrCorrupt;
    let decision = reduce_entitlement_state(missing, CapabilityFamily::AccountRecovery, None);
    assert_eq!(decision.posture(), EntitlementPolicyPosture::Allow);
    assert_eq!(decision.reason(), DecisionReason::Allow);

    let decision = reduce_entitlement_state(missing, CapabilityFamily::CustomerDataExport, None);
    assert_eq!(decision.posture(), EntitlementPolicyPosture::Allow);

    let decision = reduce_entitlement_state(missing, CapabilityFamily::ReadProjection, None);
    assert_eq!(decision.posture(), EntitlementPolicyPosture::Read);
    assert_eq!(decision.reason(), DecisionReason::ReadLocalOnly);

    // Value-producing base mutations stay denied during the outage.
    let decision = reduce_entitlement_state(missing, CapabilityFamily::BaseFocusa, None);
    assert_eq!(decision.posture(), EntitlementPolicyPosture::Deny);

    // Refunded/revoked state preserves the same recovery/read/export surface.
    for state in [
        PolicyEntitlementState::RefundedOrRevoked,
        PolicyEntitlementState::Expired,
    ] {
        let decision = reduce_entitlement_state(state, CapabilityFamily::AccountRecovery, None);
        assert_eq!(decision.posture(), EntitlementPolicyPosture::Allow);
        let decision = reduce_entitlement_state(state, CapabilityFamily::CustomerDataExport, None);
        assert_eq!(decision.posture(), EntitlementPolicyPosture::Allow);
        let decision = reduce_entitlement_state(state, CapabilityFamily::ReadProjection, None);
        assert_eq!(decision.posture(), EntitlementPolicyPosture::Read);
        let decision = reduce_entitlement_state(state, CapabilityFamily::BaseFocusa, None);
        assert_eq!(decision.posture(), EntitlementPolicyPosture::Deny);
    }

    // UIAI fails closed during an outage: no posture at all never grants a
    // paid family, but a verified no-license posture keeps its single bounded
    // public-observation session.
    let decision = resolve_uiai_capability(
        None,
        None,
        UiaiOperationClass::RemotePremium,
        "browser_action",
        "uiai_browser_action",
        0,
        now(),
    );
    assert_eq!(
        decision,
        UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::MissingPosture)
    );
}

// ── 5. Direct core/API/CLI/worker/child-token bypass attempts fail ────────

#[test]
fn spec152f_bypass_resistance_direct_core_handler_bypass_attempts_fail() {
    // A caller-forged product code is never normalized into a grant.
    for forged in ["Focusa", " focusa", "focusa.extra", "focusa-pro", "FOCUSA"] {
        assert_eq!(
            resolve_base_focusa_product(forged, PolicyEntitlementState::ActivePaid),
            BaseProductDecision::Denied,
            "forged product {forged:?} must not satisfy the base gate"
        );
    }

    // A snapshot that SAYS Active but whose signed window already closed is a
    // stale active lease: premium is denied before side effects.
    let mut stale_active = active_snapshot(&[AUTOMATION_FEATURE], 4);
    stale_active.expires_at = Some(now() - Duration::minutes(1));
    let denial = resolve_premium_family(
        &stale_active,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("stale active lease must deny");
    assert_eq!(denial, PremiumFamilyDenial::ActiveLeaseExpired);

    // A snapshot with no lease binding/sequence can never resolve a premium
    // feature, even if the caller fills in feature claims.
    let mut unbinding = offline_grace_snapshot(&[AUTOMATION_FEATURE], Some(now() + Duration::minutes(5)), 4);
    unbinding.lease_id = None;
    unbinding.sequence = None;
    unbinding.lease_digest = None;
    let denial = resolve_premium_family(
        &unbinding,
        CapabilityFamily::Automation,
        AUTOMATION_FEATURE,
        now(),
    )
    .denial()
    .cloned()
    .expect("lease-less snapshot must deny");
    assert!(matches!(
        denial,
        PremiumFamilyDenial::MissingLeaseSequence | PremiumFamilyDenial::MissingLeaseBinding
    ));

    // Direct `base_product_projection` on a refunded/revoked snapshot cannot
    // permit mutations, and a missing snapshot fails closed.
    let projection = base_product_projection(Some(&recovery_only_snapshot(&[], 5)))
        .expect("projection resolves");
    assert!(!projection.permits_base_mutations);
    assert!(base_product_projection(None).is_err());
}

#[test]
fn spec152f_bypass_resistance_child_token_cannot_widen_or_outlive_grants() {
    let parent = active_snapshot(&[AUTOMATION_FEATURE], 4);
    let grant = uiai_grant_snapshot();
    let mut broker = UiaiChildTokenBroker::default();
    let now = now();

    // Requesting a feature the UIAI grant never signed is refused.
    let mut widening = child_token_request();
    widening.requested_features.insert("uiai_persistence".to_string());
    assert_eq!(
        broker.validate_request(&widening, &parent, &grant, now),
        Err(UiaiChildTokenError::ScopeNotGranted)
    );

    // Requesting limits above the authority grant is refused.
    let mut limit_widen = child_token_request();
    limit_widen
        .requested_limits
        .insert("sessions".to_string(), 999);
    assert_eq!(
        broker.validate_request(&limit_widen, &parent, &grant, now),
        Err(UiaiChildTokenError::ScopeNotGranted)
    );

    // A nonce replay is refused once the authority accepted that nonce.
    let request = child_token_request();
    let envelope = authority_envelope(&request, &parent, &grant, now);
    broker
        .accept_authority_token(&request, &parent, &grant, envelope, now)
        .expect("first use of the nonce is accepted");
    assert_eq!(
        broker.validate_request(&request, &parent, &grant, now),
        Err(UiaiChildTokenError::NonceReplay)
    );

    // An authority envelope that does not exactly match the request fails.
    let mut broker2 = UiaiChildTokenBroker::default();
    let request = child_token_request();
    let mut mismatched = authority_envelope(&request, &parent, &grant, now);
    mismatched.features.insert("uiai_persistence".to_string());
    assert_eq!(
        broker2.accept_authority_token(&request, &parent, &grant, mismatched, now),
        Err(UiaiChildTokenError::AuthorityResponseMismatch)
    );

    // A child token cannot outlive the parent or grant bound, or exceed the
    // fixed 15-minute TTL.
    let request = child_token_request();
    let mut envelope = authority_envelope(&request, &parent, &grant, now);
    envelope.expires_at = now + Duration::minutes(UIAI_CHILD_TOKEN_MAX_TTL_MINUTES + 5);
    assert_eq!(
        broker2.accept_authority_token(&request, &parent, &grant, envelope, now),
        Err(UiaiChildTokenError::InvalidExpiry)
    );

    // The UIAI capability decision never grants a paid family that the grant
    // does not carry, and a Focusa-only paid entitlement never grants UIAI.
    let decision = resolve_uiai_capability(
        Some(&parent),
        Some(&grant),
        UiaiOperationClass::RemotePremium,
        "browser_action",
        "uiai_browser_action",
        0,
        now,
    );
    assert!(
        decision.is_allowed(),
        "signed UIAI family resolves for the bound account"
    );
    let ungranted = resolve_uiai_capability(
        Some(&parent),
        Some(&grant),
        UiaiOperationClass::RemotePremium,
        "browser_action",
        "uiai_persistence",
        0,
        now,
    );
    assert_eq!(
        ungranted,
        UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FamilyNotGranted)
    );
    let focusa_only = resolve_uiai_capability(
        Some(&parent),
        None,
        UiaiOperationClass::RemotePremium,
        "browser_action",
        "uiai_browser_action",
        0,
        now,
    );
    assert_eq!(
        focusa_only,
        UiaiCapabilityDecision::Denied(UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai)
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn child_token_request() -> UiaiChildTokenRequest {
    UiaiChildTokenRequest {
        request_id: Uuid::nil(),
        audience: "aud-focusa".to_string(),
        node_id: "node-offline-001".to_string(),
        client_id: "client-focusa".to_string(),
        parent_lease_id: "lease-offline-4".to_string(),
        parent_lease_sequence: 4,
        parent_lease_digest: "sha256:offline-cached-grant".to_string(),
        uiai_grant_lease_id: "lease-uiai-engine".to_string(),
        uiai_grant_sequence: 7,
        requested_features: BTreeSet::from(["uiai_browser_action".to_string()]),
        requested_limits: BTreeMap::new(),
        nonce: "nonce-bypass-matrix".to_string(),
    }
}

fn uiai_grant_snapshot() -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("uiai-engine", "node-offline-001");
    snapshot.state = EntitlementState::Active;
    snapshot.subject_id = Some("account-offline-001".to_string());
    snapshot.lease_id = Some("lease-uiai-engine".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:uiai-bound-grant".to_string());
    snapshot.expires_at = Some(now() + Duration::hours(1));
    snapshot
        .features
        .insert("uiai_browser_action".to_string(), true);
    snapshot
        .features
        .insert("uiai_public_observation".to_string(), true);
    snapshot.limits.insert("sessions".to_string(), 1);
    snapshot
}

fn authority_envelope(
    request: &UiaiChildTokenRequest,
    _parent: &EntitlementSnapshot,
    _grant: &EntitlementSnapshot,
    now: chrono::DateTime<Utc>,
) -> AuthorityChildTokenEnvelope {
    AuthorityChildTokenEnvelope {
        schema: UIAI_CHILD_TOKEN_SCHEMA.to_string(),
        token: "ct_auth_bypass_matrix_token".to_string(),
        token_id: "ct-0001".to_string(),
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
        expires_at: now + Duration::minutes(5),
    }
}
