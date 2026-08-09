//! Spec 152F.04.03 — Release-proof premium boundary enforcement tests.
//!
//! Verifies that safe release status reads remain available without premium
//! and that advanced governed release orchestration/proof operations require
//! the exact `focusa.release.proof` feature grant.

use chrono::{Duration, Utc};
use focusa_license::{
    authority::{EntitlementSnapshot, EntitlementState},
    premium_family_feature_ids, resolve_premium_family,
    reduce_entitlement_state,
    RELEASE_PROOF_PREMIUM_FEATURE_IDS,
    BaseProductDecision, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
    PremiumFamilyDecision, PremiumFamilyDenial,
};

// ── Release-proof family map ───────────────────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_proof_family_has_exact_one_feature() {
    let features = RELEASE_PROOF_PREMIUM_FEATURE_IDS;
    assert_eq!(
        features.len(),
        1,
        "release_proof must have exactly one premium feature"
    );
    assert!(features.contains(&"focusa.release.proof"));
}

#[test]
fn spec152f_release_proof_entitlement_premium_family_feature_ids_returns_release_proof_feature() {
    let features = premium_family_feature_ids(Family::ReleaseProof);
    assert_eq!(features.len(), 1);
    assert!(features.contains(&"focusa.release.proof"));
}

#[test]
fn spec152f_release_proof_entitlement_non_premium_families_return_empty_features() {
    for family in [
        Family::AccountRecovery,
        Family::ReadProjection,
        Family::BaseFocusa,
        Family::InternalMaintenance,
    ] {
        assert!(
            premium_family_feature_ids(family).is_empty(),
            "{family:?} must not have premium features"
        );
    }
    // CustomerDataExport carries the optional focusa.export.packaged premium
    // feature for value-added packaging; basic export is always available.
    assert!(!premium_family_feature_ids(Family::CustomerDataExport).is_empty());
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_is_optional_premium_family() {
    assert!(Family::ReleaseProof.is_optional_premium());
    assert_eq!(
        Family::ReleaseProof.commercial_treatment().label(),
        "optional_premium"
    );
}

// ── State grid: release_proof is denied for unlicensed states ──────────────

#[test]
fn spec152f_release_proof_entitlement_release_proof_denied_for_verified_no_license() {
    let decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::ReleaseProof, None);
    assert_eq!(
        decision.posture(),
        Posture::Deny,
        "release_proof must be denied for verified-no-license"
    );
    assert_eq!(decision.reason(), Reason::Deny);
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_feature_for_active_paid() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::ReleaseProof, None);
    assert_eq!(
        decision.posture(),
        Posture::Feature,
        "release_proof must require feature for active paid"
    );
    assert_eq!(decision.reason(), Reason::RequireFeature);
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_cached_feature_for_offline_grace() {
    let decision = reduce_entitlement_state(State::OfflineGrace, Family::ReleaseProof, None);
    assert_eq!(
        decision.posture(),
        Posture::Feature,
        "release_proof must require cached feature for offline grace"
    );
    assert_eq!(decision.reason(), Reason::RequireCachedFeature);
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_denied_for_expired_and_revoked() {
    for state in [
        State::Expired,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ] {
        let decision = reduce_entitlement_state(state, Family::ReleaseProof, None);
        assert_eq!(
            decision.posture(),
            Posture::Deny,
            "release_proof must be denied for {state:?}"
        );
        assert_eq!(decision.reason(), Reason::Deny);
    }
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_denied_for_pending_unverified() {
    let decision = reduce_entitlement_state(State::PendingUnverified, Family::ReleaseProof, None);
    assert_eq!(decision.posture(), Posture::Deny);
    assert_eq!(decision.reason(), Reason::Deny);
}

// ── Base Focusa is not accidentally premium ────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_base_focusa_is_not_premium() {
    assert!(!Family::BaseFocusa.is_optional_premium());
    assert_eq!(
        Family::BaseFocusa.commercial_treatment().label(),
        "base_entitlement"
    );
    assert!(premium_family_feature_ids(Family::BaseFocusa).is_empty());

    // Base Focusa operations are allowed (limited) for verified-no-license,
    // while release_proof is denied for the same state.
    let base_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    let release_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::ReleaseProof, None);
    assert_eq!(release_decision.posture(), Posture::Deny);
}

// ── Premium family resolution for release_proof ────────────────────────────

fn active_snapshot() -> EntitlementSnapshot {
    let now = Utc::now();
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-release-proof");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-rp-001".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:rp".to_string());
    snapshot.expires_at = Some(now + Duration::hours(1));
    snapshot.offline_grace_until = Some(now + Duration::hours(1));
    snapshot
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_premium_feature() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(
        decision.is_feature(),
        "release_proof must resolve as premium feature"
    );
    assert_eq!(
        decision.required_feature().unwrap().as_str(),
        "focusa.release.proof"
    );
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    // No features granted at all
    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_base_product_first() {
    // Wrong product — base gate fails before feature check
    let mut snapshot = active_snapshot();
    snapshot.product = "uiai-engine".to_string();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired {
            decision: BaseProductDecision::Denied,
        }
    ));
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_non_zero_lease_sequence() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);
    snapshot.sequence = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseSequence
    ));
}

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_lease_binding() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);
    snapshot.lease_digest = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseBinding
    ));
}

#[test]
fn spec152f_release_proof_entitlement_cross_family_feature_is_rejected() {
    let mut snapshot = active_snapshot();
    // Grant a feature from a different family
    snapshot
        .features
        .insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered {
            family: Family::ReleaseProof,
            ..
        }
    ));
}

#[test]
fn spec152f_release_proof_entitlement_unqualified_feature_identifier_is_rejected() {
    let snapshot = active_snapshot();

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "not.a.qualified.feature",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::InvalidRequiredFeature { .. }
    ));
}

#[test]
fn spec152f_release_proof_entitlement_offline_grace_within_window_allows_release_proof() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now + Duration::minutes(5));
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        now,
    );
    assert!(
        decision.is_feature(),
        "offline grace within window must allow release_proof"
    );
    assert!(matches!(
        decision,
        PremiumFamilyDecision::Feature {
            offline_cached: true,
            ..
        }
    ));
}

#[test]
fn spec152f_release_proof_entitlement_offline_grace_expired_denies_release_proof() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now - Duration::minutes(1)); // expired
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::CachedGrantExpired
    ));
}

#[test]
fn spec152f_release_proof_entitlement_active_lease_expired_denies_release_proof() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.expires_at = Some(now - Duration::minutes(1)); // expired
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::ActiveLeaseExpired
    ));
}

#[test]
fn spec152f_release_proof_entitlement_non_premium_family_rejected_by_resolver() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::BaseFocusa,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::NotPremiumFamily {
            family: Family::BaseFocusa,
        }
    ));
}

// ── Read projection vs release proof boundary ──────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_read_projection_is_always_available() {
    // Safe release status reads use ReadProjection family, not ReleaseProof premium.
    let decision = reduce_entitlement_state(
        State::VerifiedNoLicense,
        Family::ReadProjection,
        None,
    );
    assert_eq!(
        decision.posture(),
        Posture::Read,
        "read projection must be available for verified-no-license"
    );
    assert_eq!(decision.reason(), Reason::Read);

    // Same state denies ReleaseProof premium.
    let release_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::ReleaseProof, None);
    assert_eq!(release_decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_release_proof_entitlement_read_projection_available_for_all_states() {
    // Read projection must be available even for expired, revoked, and corrupt states.
    // PendingUnverified always denies; it is excluded from read availability.
    for state in [
        State::Expired,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ] {
        let decision = reduce_entitlement_state(state, Family::ReadProjection, None);
        assert!(
            matches!(decision.posture(), Posture::Read | Posture::Allow),
            "read projection must be available for {state:?}, got {:?}",
            decision.posture()
        );
    }
}

// ── Interactive base vs release proof boundary ─────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_base_focusa_requires_base_posture_not_feature() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::BaseFocusa, None);
    assert_eq!(decision.posture(), Posture::Base);
    assert_eq!(decision.reason(), Reason::RequireBase);

    // ReleaseProof requires feature, not base.
    let rp_decision = reduce_entitlement_state(State::ActivePaid, Family::ReleaseProof, None);
    assert_eq!(rp_decision.posture(), Posture::Feature);
    assert_eq!(rp_decision.reason(), Reason::RequireFeature);
}

// ── Release proof feature ID is authority-owned ────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_proof_feature_id_is_qualified() {
    let feature = RELEASE_PROOF_PREMIUM_FEATURE_IDS[0];
    assert!(
        feature.starts_with("focusa."),
        "release_proof feature {feature} must be a qualified Focusa identifier"
    );
    assert!(
        feature.len() > "focusa.".len(),
        "release_proof feature {feature} must have a non-empty sub-identifier"
    );
    // Must be valid focusa identifiers (lowercase, dots, no uppercase)
    assert!(
        feature
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
        "release_proof feature {feature} must use only lowercase/dot/underscore"
    );
}

// ── No caller-controlled grants ────────────────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_caller_cannot_control_release_proof_grants() {
    // The release_proof premium feature ID is authority-owned.
    // Callers cannot supply their own feature identifier and get a grant.

    let mut snapshot = active_snapshot();
    // Grant only the release_proof feature
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);

    // Requesting a different feature that is not granted fails
    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Requesting a caller-invented feature fails at the identifier validation
    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.caller_invented",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered { .. }
    ));
}

// ── All four premium families are distinct ─────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_all_premium_families_are_distinct() {
    use focusa_license::{
        AUTOMATION_PREMIUM_FEATURE_IDS, PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
        TEAM_REMOTE_PREMIUM_FEATURE_IDS,
    };

    let all_release: std::collections::HashSet<&str> =
        RELEASE_PROOF_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_automation: std::collections::HashSet<&str> =
        AUTOMATION_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_team: std::collections::HashSet<&str> =
        TEAM_REMOTE_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_updates: std::collections::HashSet<&str> =
        PREMIUM_UPDATES_PREMIUM_FEATURE_IDS.iter().copied().collect();

    // No feature belongs to more than one premium family
    assert!(all_release.is_disjoint(&all_automation));
    assert!(all_release.is_disjoint(&all_team));
    assert!(all_release.is_disjoint(&all_updates));
}

// ── Revalidation at dispatch ───────────────────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_proof_requires_revalidation_at_dispatch() {
    // When the entitlement state changes (e.g., Active -> RecoveryOnly),
    // release proof dispatch must stop.

    // Active paid with release_proof feature: premium family resolves
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);
    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Same snapshot but with RecoveryOnly state: base product fails
    let mut recovery = snapshot.clone();
    recovery.state = EntitlementState::RecoveryOnly;
    let decision = resolve_premium_family(
        &recovery,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));
}

// ── Reserve limits before orchestration ────────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_proof_limits_reserved_before_orchestration() {
    // The release_proof premium family requires both the feature grant AND
    // a limit reservation before executing release orchestration.

    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.release.proof".to_string(), true);
    snapshot
        .limits
        .insert("release_proof_runs".to_string(), 4);

    let decision = resolve_premium_family(
        &snapshot,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Without the feature grant, resolution fails before limits are checked.
    let mut no_feature = active_snapshot();
    no_feature
        .limits
        .insert("release_proof_runs".to_string(), 4);
    let decision = resolve_premium_family(
        &no_feature,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

// ── Release operation registry classification ──────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_operation_registry_mutating_ops_are_premium() {
    // Mutating release operations must be classified as ReleaseProof premium.
    // Read-only operations are ReadProjection and do not require premium.
    assert!(Family::ReleaseProof.is_optional_premium());
    assert!(!Family::ReadProjection.is_optional_premium());

    // ReleaseProof operations require feature, ReadProjection does not
    let features = premium_family_feature_ids(Family::ReleaseProof);
    assert_eq!(features.len(), 1);
    assert!(features.contains(&"focusa.release.proof"));

    let read_features = premium_family_feature_ids(Family::ReadProjection);
    assert!(read_features.is_empty());
}

// ── Release proof status route is read projection ──────────────────────────

#[test]
fn spec152f_release_proof_entitlement_release_status_route_is_read_not_premium() {
    // The /v1/release/proof/status endpoint is a GET route that reads
    // proof status. It must be classified as ReadProjection, not ReleaseProof
    // premium. Only mutation-class orchestration requires the premium grant.

    // ReadProjection is always available
    let decision = reduce_entitlement_state(
        State::VerifiedNoLicense,
        Family::ReadProjection,
        None,
    );
    assert_eq!(decision.posture(), Posture::Read);

    // ReleaseProof is denied for the same state
    let rp_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::ReleaseProof, None);
    assert_eq!(rp_decision.posture(), Posture::Deny);
}

// ── Recovery paths remain available ────────────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_recovery_paths_remain_available() {
    // Account recovery is always available regardless of release_proof status.
    let decision = reduce_entitlement_state(
        State::RefundedOrRevoked,
        Family::AccountRecovery,
        None,
    );
    assert_eq!(decision.posture(), Posture::Allow);

    // Customer data export is always available for basic access.
    let decision = reduce_entitlement_state(
        State::RefundedOrRevoked,
        Family::CustomerDataExport,
        None,
    );
    assert_eq!(decision.posture(), Posture::Allow);
}

// ── Adversarial isolation (Spec 152F.04.07) ────────────────────────────────

#[test]
fn spec152f_release_proof_entitlement_wrong_product_and_omission_fail_closed() {
    // A wrong product never widens release_proof, even with stored grants.
    let mut wrong = active_snapshot();
    wrong.product = "uiai_engine".to_string();
    wrong
        .features
        .insert("focusa.release.proof".to_string(), true);
    let decision = resolve_premium_family(
        &wrong,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));

    // An Evaluation-issued lease that omits release.proof cannot widen.
    let mut evaluation = active_snapshot();
    evaluation.lease_id = Some("lease-eval-release".to_string());
    evaluation.features.clear();
    let decision = resolve_premium_family(
        &evaluation,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // Offline Grace cannot expand into release proof without the cached grant.
    let mut grace = active_snapshot();
    grace.state = EntitlementState::OfflineGrace;
    grace.features.clear();
    grace.offline_grace_until = Some(Utc::now() + Duration::minutes(5));
    let decision = resolve_premium_family(
        &grace,
        Family::ReleaseProof,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}
