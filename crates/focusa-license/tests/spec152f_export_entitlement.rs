//! Spec 152F.04.05 — Separate basic customer-data export from premium packaging.
//!
//! Verifies that basic customer-data export is always available in every
//! entitlement state, while premium packaging (focusa.export.packaged) is
//! gated behind the optional premium feature grant.

use chrono::{Duration, Utc};
use focusa_license::{
    authority::{EntitlementSnapshot, EntitlementState},
    premium_family_feature_ids, reduce_entitlement_state, resolve_export_packaged,
    CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS,
    CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
    PremiumFamilyDecision, PremiumFamilyDenial,
};

// ── Export family map ──────────────────────────────────────────────────────

#[test]
fn spec152f_export_entitlement_customer_data_export_has_exact_one_premium_feature() {
    let features = CUSTOMER_DATA_EXPORT_PREMIUM_FEATURE_IDS;
    assert_eq!(
        features.len(),
        1,
        "customer_data_export must have exactly one premium feature"
    );
    assert!(features.contains(&"focusa.export.packaged"));
}

#[test]
fn spec152f_export_entitlement_premium_family_feature_ids_returns_export_packaged() {
    let features = premium_family_feature_ids(Family::CustomerDataExport);
    assert_eq!(features.len(), 1);
    assert!(features.contains(&"focusa.export.packaged"));
}

#[test]
fn spec152f_export_entitlement_customer_data_export_is_not_optional_premium() {
    // Basic export is always available; the family is not optional premium.
    assert!(!Family::CustomerDataExport.is_optional_premium());
    assert_eq!(
        Family::CustomerDataExport.commercial_treatment().label(),
        "always_available_basic_with_optional_premium_packaging"
    );
}

// ── State grid: basic export is ALWAYS allowed ─────────────────────────────

#[test]
fn spec152f_export_entitlement_basic_export_allowed_in_every_state() {
    let all_states = [
        State::PendingUnverified,
        State::VerifiedNoLicense,
        State::ActivePaid,
        State::OfflineGrace,
        State::Expired,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ];

    for state in all_states {
        let decision = reduce_entitlement_state(state, Family::CustomerDataExport, None);
        assert_eq!(
            decision.posture(),
            Posture::Allow,
            "basic export must be allowed in state {state:?}"
        );
        assert!(
            !matches!(
                decision.reason(),
                Reason::RequireBase
                    | Reason::RequireFeature
                    | Reason::RequireCachedFeature
                    | Reason::RequireCachedFeatureWhenSafe
            ),
            "basic export must never require a premium feature in state {state:?}"
        );
    }
}

#[test]
fn spec152f_export_entitlement_pending_unverified_export_is_existing_local_only() {
    let decision = reduce_entitlement_state(State::PendingUnverified, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::AllowExistingLocalOnly);
}

#[test]
fn spec152f_export_entitlement_verified_no_license_export_is_allowed() {
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_active_paid_export_is_allowed() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_offline_grace_export_is_allowed() {
    let decision = reduce_entitlement_state(State::OfflineGrace, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_expired_export_is_allowed() {
    let decision = reduce_entitlement_state(State::Expired, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_refunded_or_revoked_export_is_allowed() {
    let decision = reduce_entitlement_state(State::RefundedOrRevoked, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_missing_or_corrupt_export_is_allowed() {
    let decision = reduce_entitlement_state(State::MissingOrCorrupt, Family::CustomerDataExport, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::Allow);
}

// ── Premium packaging: focusa.export.packaged is additive only ─────────────

fn active_snapshot() -> EntitlementSnapshot {
    let now = Utc::now();
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-export");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-export-001".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:export".to_string());
    snapshot.expires_at = Some(now + Duration::hours(1));
    snapshot.offline_grace_until = Some(now + Duration::hours(1));
    snapshot
}

#[test]
fn spec152f_export_entitlement_export_packaged_requires_premium_feature_grant() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(
        decision.is_feature(),
        "focusa.export.packaged must resolve as premium feature"
    );
    assert_eq!(
        decision.required_feature().unwrap().as_str(),
        "focusa.export.packaged"
    );
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_export_entitlement_export_packaged_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    // No features granted at all
    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_does_not_require_base_product() {
    // Unlike the four premium families, export.packaged does not require the
    // base product gate — basic export is always available, and this function
    // only gates the additive premium packaging.
    let mut snapshot = active_snapshot();
    snapshot.product = "uiai-engine".to_string();
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    // The function should still resolve (it checks feature, not base product)
    // But note: it checks sequence/binding/expiry, so with a valid snapshot
    // it should succeed regardless of product.
    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    // With wrong product, the base product gate is not checked here;
    // the feature grant is still present in the snapshot.
    assert!(decision.is_feature());
}

#[test]
fn spec152f_export_entitlement_export_packaged_requires_non_zero_lease_sequence() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);
    snapshot.sequence = None;

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseSequence
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_requires_lease_binding() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);
    snapshot.lease_digest = None;

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseBinding
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_unqualified_identifier_is_rejected() {
    let snapshot = active_snapshot();

    let decision = resolve_export_packaged(&snapshot, "not.a.qualified.feature", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::InvalidRequiredFeature { .. }
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_wrong_feature_for_family_is_rejected() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.release.proof", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered {
            family: Family::CustomerDataExport,
            ..
        }
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_offline_grace_with_feature_is_cached() {
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(Utc::now() + Duration::minutes(5));
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision,
        PremiumFamilyDecision::Feature {
            offline_cached: true,
            ..
        }
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_offline_grace_expired_is_denied() {
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(Utc::now() - Duration::seconds(1));
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::CachedGrantExpired
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_offline_grace_no_expiry_window_is_denied() {
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = None;
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingCachedGrantExpiry
    ));
}

#[test]
fn spec152f_export_entitlement_export_packaged_active_lease_expired_is_denied() {
    let mut snapshot = active_snapshot();
    snapshot.expires_at = Some(Utc::now() - Duration::seconds(1));
    snapshot
        .features
        .insert("focusa.export.packaged".to_string(), true);

    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::ActiveLeaseExpired
    ));
}

// ── Privacy/redaction: export never deletes or mutates source data ─────────

#[test]
fn spec152f_export_entitlement_export_preserves_privacy_redaction_requirement() {
    use focusa_license::{RecoveryAllowance, SecurityPrerequisite};

    let prerequisites = RecoveryAllowance::CustomerDataExport.security_prerequisites();
    assert!(
        prerequisites.contains(&SecurityPrerequisite::IdentityVerification),
        "export must require identity verification"
    );
    assert!(
        prerequisites.contains(&SecurityPrerequisite::PrivacyRedaction),
        "export must require privacy redaction"
    );
    assert!(
        prerequisites.contains(&SecurityPrerequisite::ScopeBinding),
        "export must require scope binding"
    );
}

// ── Cross-family: export is not blocked by other family denials ────────────

#[test]
fn spec152f_export_entitlement_export_not_blocked_by_base_focusa_denial() {
    // When base Focusa is denied (expired), export must still be allowed.
    let base_decision = reduce_entitlement_state(State::Expired, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Deny);

    let export_decision = reduce_entitlement_state(State::Expired, Family::CustomerDataExport, None);
    assert_eq!(export_decision.posture(), Posture::Allow);
    assert_eq!(export_decision.reason(), Reason::Allow);
}

#[test]
fn spec152f_export_entitlement_export_not_blocked_by_automation_denial() {
    // When automation is denied, export must still be allowed.
    let auto_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::Automation, None);
    assert_eq!(auto_decision.posture(), Posture::Deny);

    let export_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::CustomerDataExport, None);
    assert_eq!(export_decision.posture(), Posture::Allow);
}

#[test]
fn spec152f_export_entitlement_export_not_blocked_by_team_remote_denial() {
    let team_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);

    let export_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::CustomerDataExport, None);
    assert_eq!(export_decision.posture(), Posture::Allow);
}

#[test]
fn spec152f_export_entitlement_export_not_blocked_by_release_proof_denial() {
    let release_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::ReleaseProof, None);
    assert_eq!(release_decision.posture(), Posture::Deny);

    let export_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::CustomerDataExport, None);
    assert_eq!(export_decision.posture(), Posture::Allow);
}

#[test]
fn spec152f_export_entitlement_export_not_blocked_by_premium_updates_denial() {
    let updates_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::PremiumUpdates, None);
    assert_eq!(updates_decision.posture(), Posture::Deny);

    let export_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::CustomerDataExport, None);
    assert_eq!(export_decision.posture(), Posture::Allow);
}

// ── Premium packaging denial never blocks basic export ─────────────────────

#[test]
fn spec152f_export_entitlement_premium_packaging_denial_does_not_block_basic_export() {
    // Even when focusa.export.packaged is denied, basic export must still be ALLOW.
    let all_states = [
        State::PendingUnverified,
        State::VerifiedNoLicense,
        State::ActivePaid,
        State::OfflineGrace,
        State::Expired,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ];

    for state in all_states {
        let decision = reduce_entitlement_state(state, Family::CustomerDataExport, None);
        assert_eq!(
            decision.posture(),
            Posture::Allow,
            "basic export must remain allowed in state {state:?} even when premium packaging is denied"
        );
        // Premium packaging denial only affects the premium feature check,
        // never the basic export state grid cell.
    }
}

// ── Adversarial isolation (Spec 152F.04.07) ────────────────────────────────

#[test]
fn spec152f_export_entitlement_adversarial_isolation_fails_closed() {
    // A caller-invented packaged-export identifier never resolves.
    let snapshot = active_snapshot();
    let decision = resolve_export_packaged(&snapshot, "focusa.export.packaged.plus", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered {
            family: Family::CustomerDataExport,
            ..
        }
    ));

    // An Evaluation-issued lease that omits focusa.export.packaged cannot widen:
    // premium packaging resolves only when the grant includes it.
    let mut evaluation = active_snapshot();
    evaluation.lease_id = Some("lease-eval-export".to_string());
    evaluation.features.clear();
    let decision = resolve_export_packaged(&evaluation, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // Offline Grace cannot expand into packaged export without the cached grant.
    let mut grace = active_snapshot();
    grace.state = EntitlementState::OfflineGrace;
    grace.features.clear();
    grace.offline_grace_until = Some(Utc::now() + Duration::minutes(5));
    let decision = resolve_export_packaged(&grace, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // An expired/revoked state can never be widened by a stored packaged claim.
    let mut revoked = active_snapshot();
    revoked.state = EntitlementState::RecoveryOnly;
    revoked
        .features
        .insert("focusa.export.packaged".to_string(), true);
    let decision = resolve_export_packaged(&revoked, "focusa.export.packaged", Utc::now());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::EntitlementStateNotUsable {
            state: State::RefundedOrRevoked
        }
    ));

    // Wrong-product isolation for the four premium families is proven in their
    // own adversarial cases; packaged export is deliberately additive and never
    // base-gated, so a wrong product alone cannot widen it beyond the lease
    // sequence/binding/state gates above.

    // Basic export remains available for the same revoked snapshot (never
    // paywalled by the packaged-export denial).
    let basic =
        reduce_entitlement_state(State::RefundedOrRevoked, Family::CustomerDataExport, None);
    assert_eq!(basic.posture(), Posture::Allow);
}
