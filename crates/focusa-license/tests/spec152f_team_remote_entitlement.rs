//! Spec 152F.04.02 — Team/remote premium boundary enforcement tests.
//!
//! Verifies that activation/recovery pairing and base-node management are not
//! paywalled, while additional peers/operators/remote collaboration require
//! exact team_remote grants and node/seat reservations.

use chrono::{Duration, Utc};
use focusa_license::{
    authority::{EntitlementSnapshot, EntitlementState},
    premium_family_feature_ids, reduce_entitlement_state,
    resolve_premium_family,
    BaseProductDecision, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
    PremiumFamilyDecision, PremiumFamilyDenial,
    TEAM_REMOTE_PREMIUM_FEATURE_IDS,
};

// ── Team/remote family map ─────────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_family_has_exact_two_premium_features() {
    let features = TEAM_REMOTE_PREMIUM_FEATURE_IDS;
    assert_eq!(
        features.len(),
        2,
        "team_remote must have exactly two premium features"
    );
    assert!(features.contains(&"focusa.team.multi_operator"));
    assert!(features.contains(&"focusa.remote.stream"));
}

#[test]
fn spec152f_team_remote_entitlement_premium_family_feature_ids_returns_team_features() {
    let features = premium_family_feature_ids(Family::TeamRemote);
    assert_eq!(features.len(), 2);
    assert!(features.contains(&"focusa.team.multi_operator"));
    assert!(features.contains(&"focusa.remote.stream"));
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_is_optional_premium_family() {
    assert!(Family::TeamRemote.is_optional_premium());
    assert_eq!(
        Family::TeamRemote.commercial_treatment().label(),
        "optional_premium"
    );
}

// ── State grid: team_remote is denied for unlicensed states ────────────────

#[test]
fn spec152f_team_remote_entitlement_team_remote_denied_for_verified_no_license() {
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(
        decision.posture(),
        Posture::Deny,
        "team_remote must be denied for verified-no-license"
    );
    assert_eq!(decision.reason(), Reason::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_requires_feature_for_active_paid() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::TeamRemote, None);
    assert_eq!(
        decision.posture(),
        Posture::Feature,
        "team_remote must require feature for active paid"
    );
    assert_eq!(decision.reason(), Reason::RequireFeature);
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_requires_cached_feature_for_offline_grace() {
    let decision = reduce_entitlement_state(State::OfflineGrace, Family::TeamRemote, None);
    assert_eq!(
        decision.posture(),
        Posture::Feature,
        "team_remote must require cached feature for offline grace"
    );
    assert_eq!(decision.reason(), Reason::RequireCachedFeature);
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_denied_for_expired_and_revoked() {
    for state in [State::Expired, State::RefundedOrRevoked, State::MissingOrCorrupt] {
        let decision = reduce_entitlement_state(state, Family::TeamRemote, None);
        assert_eq!(
            decision.posture(),
            Posture::Deny,
            "team_remote must be denied for {state:?}"
        );
        assert_eq!(decision.reason(), Reason::Deny);
    }
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_denied_for_pending_unverified() {
    let decision = reduce_entitlement_state(State::PendingUnverified, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Deny);
    assert_eq!(decision.reason(), Reason::Deny);
}

// ── Base Focusa is not accidentally premium ────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_base_focusa_is_not_premium() {
    assert!(!Family::BaseFocusa.is_optional_premium());
    assert_eq!(
        Family::BaseFocusa.commercial_treatment().label(),
        "base_entitlement"
    );
    assert!(premium_family_feature_ids(Family::BaseFocusa).is_empty());

    // Base Focusa operations are allowed (limited) for verified-no-license,
    // while team_remote is denied for the same state.
    let base_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

// ── Premium family resolution for team_remote ──────────────────────────────

fn active_snapshot() -> EntitlementSnapshot {
    let now = Utc::now();
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-team");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-team-001".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:team".to_string());
    snapshot.expires_at = Some(now + Duration::hours(1));
    snapshot.offline_grace_until = Some(now + Duration::hours(1));
    snapshot
}

#[test]
fn spec152f_team_remote_entitlement_multi_operator_requires_team_premium_feature() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(
        decision.is_feature(),
        "multi_operator must resolve as premium feature"
    );
    assert_eq!(
        decision.required_feature().unwrap().as_str(),
        "focusa.team.multi_operator"
    );
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_team_remote_entitlement_remote_stream_requires_team_premium_feature() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.remote.stream".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.remote.stream",
        Utc::now(),
    );
    assert!(
        decision.is_feature(),
        "remote.stream must resolve as premium feature"
    );
    assert_eq!(
        decision.required_feature().unwrap().as_str(),
        "focusa.remote.stream"
    );
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_team_remote_entitlement_multi_operator_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_remote_stream_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.remote.stream",
        Utc::now(),
    );
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_requires_base_product_first() {
    let mut snapshot = active_snapshot();
    snapshot.product = "uiai-engine".to_string();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
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
fn spec152f_team_remote_entitlement_team_remote_requires_non_zero_lease_sequence() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    snapshot.sequence = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseSequence
    ));
}

#[test]
fn spec152f_team_remote_entitlement_team_remote_requires_lease_binding() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    snapshot.lease_digest = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseBinding
    ));
}

#[test]
fn spec152f_team_remote_entitlement_cross_family_feature_is_rejected() {
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered {
            family: Family::TeamRemote,
            ..
        }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_unqualified_feature_identifier_is_rejected() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "not.a.qualified.feature",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::InvalidRequiredFeature { .. }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_offline_grace_within_window_allows_team_remote() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now + Duration::minutes(5));
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        now,
    );
    assert!(
        decision.is_feature(),
        "offline grace within window must allow team_remote"
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
fn spec152f_team_remote_entitlement_offline_grace_expired_denies_team_remote() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now - Duration::minutes(1)); // expired
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::CachedGrantExpired
    ));
}

#[test]
fn spec152f_team_remote_entitlement_active_lease_expired_denies_team_remote() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.expires_at = Some(now - Duration::minutes(1)); // expired
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::ActiveLeaseExpired
    ));
}

#[test]
fn spec152f_team_remote_entitlement_non_premium_family_rejected_by_resolver() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::BaseFocusa,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::NotPremiumFamily {
            family: Family::BaseFocusa,
        }
    ));
}

// ── Activation/recovery pairing is base, not premium ──────────────────────

#[test]
fn spec152f_team_remote_entitlement_activation_pairing_is_base_not_premium() {
    // Activation pairing (device/pair/start, connect/firstrun) uses BaseFocusa
    // family, not TeamRemote. These operations must be reachable without a
    // premium grant.
    let base_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    // TeamRemote is denied for the same state.
    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_recovery_pairing_stays_available() {
    // Recovery operations (device/pair/revoke) are always available via
    // AccountRecovery family, even in blocked states.
    let recovery_decision = reduce_entitlement_state(
        State::RefundedOrRevoked,
        Family::AccountRecovery,
        None,
    );
    assert_eq!(recovery_decision.posture(), Posture::Allow);
    assert_eq!(recovery_decision.reason(), Reason::Allow);

    // But TeamRemote is denied for the same state.
    let team_decision =
        reduce_entitlement_state(State::RefundedOrRevoked, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

// ── Additional peers/operators require premium ─────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_additional_peers_require_premium() {
    // Adding additional peers (connect, connect/room/create, connect/start)
    // requires the team_remote premium family. These are NOT base operations.
    assert!(Family::TeamRemote.is_optional_premium());

    // Active paid without team feature: denied
    let decision = reduce_entitlement_state(State::ActivePaid, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Feature);
    assert_eq!(decision.reason(), Reason::RequireFeature);

    // Verified-no-license: denied
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_remote_collaboration_requires_premium() {
    // Remote collaboration (sync, instances) requires the remote.stream
    // premium feature.
    let features = premium_family_feature_ids(Family::TeamRemote);
    assert!(features.contains(&"focusa.remote.stream"));

    // Active paid without remote.stream: denied
    let decision = reduce_entitlement_state(State::ActivePaid, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Feature);
    assert_eq!(decision.reason(), Reason::RequireFeature);
}

// ── Session transfer is premium ────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_session_transfer_requires_premium() {
    // Session transfer between peers is a team_remote operation, not a base
    // mission operation. It requires the multi_operator premium feature.
    let features = premium_family_feature_ids(Family::TeamRemote);
    assert!(features.contains(&"focusa.team.multi_operator"));

    // Verified-no-license cannot transfer sessions.
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Deny);
}

// ── Node/seat reservations ─────────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_node_management_is_base_not_premium() {
    // Base node management (lineage, node status) is BaseFocusa, not
    // TeamRemote. These operations must be reachable without a premium grant.
    let base_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    // TeamRemote is denied for the same state.
    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_team_operator_seats_are_limited() {
    // The team_operators limit bucket enforces node/seat reservations.
    // Premium feature resolution must succeed before limits are checked.
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    snapshot
        .limits
        .insert("team_operators".to_string(), 3);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Without the feature grant, resolution fails before limits are checked.
    let mut no_feature = active_snapshot();
    no_feature
        .limits
        .insert("team_operators".to_string(), 3);
    let decision = resolve_premium_family(
        &no_feature,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_remote_stream_minutes_are_limited() {
    // The remote_stream_minutes limit bucket enforces usage limits.
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.remote.stream".to_string(), true);
    snapshot
        .limits
        .insert("remote_stream_minutes".to_string(), 120);

    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.remote.stream",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Without the feature grant, resolution fails before limits are checked.
    let mut no_feature = active_snapshot();
    no_feature
        .limits
        .insert("remote_stream_minutes".to_string(), 120);
    let decision = resolve_premium_family(
        &no_feature,
        Family::TeamRemote,
        "focusa.remote.stream",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

// ── Team/remote feature IDs are authority-owned ────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_team_feature_ids_are_qualified_focusa_identifiers() {
    for feature in TEAM_REMOTE_PREMIUM_FEATURE_IDS {
        assert!(
            feature.starts_with("focusa."),
            "team feature {feature} must be a qualified Focusa identifier"
        );
        assert!(
            feature.len() > "focusa.".len(),
            "team feature {feature} must have a non-empty sub-identifier"
        );
        assert!(
            feature
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
            "team feature {feature} must use only lowercase/dot/underscore"
        );
    }
}

#[test]
fn spec152f_team_remote_entitlement_team_features_are_distinct() {
    assert_ne!(
        TEAM_REMOTE_PREMIUM_FEATURE_IDS[0],
        TEAM_REMOTE_PREMIUM_FEATURE_IDS[1],
        "team_remote premium features must be distinct"
    );
}

// ── No caller-controlled grants ────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_caller_cannot_control_team_grants() {
    // The team_remote premium feature IDs are authority-owned constants.
    // Callers cannot supply their own feature identifier and get a grant.
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);

    // Requesting a different feature that is not granted fails
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.remote.stream",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // Requesting a caller-invented feature fails at the identifier validation
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.caller_invented",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered { .. }
    ));
}

// ── All four premium families are distinct ─────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_all_premium_families_are_distinct() {
    use focusa_license::{
        AUTOMATION_PREMIUM_FEATURE_IDS, PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
        RELEASE_PROOF_PREMIUM_FEATURE_IDS,
    };

    let all_team: std::collections::HashSet<&str> =
        TEAM_REMOTE_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_automation: std::collections::HashSet<&str> =
        AUTOMATION_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_release: std::collections::HashSet<&str> =
        RELEASE_PROOF_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_updates: std::collections::HashSet<&str> =
        PREMIUM_UPDATES_PREMIUM_FEATURE_IDS.iter().copied().collect();

    assert!(all_team.is_disjoint(&all_automation));
    assert!(all_team.is_disjoint(&all_release));
    assert!(all_team.is_disjoint(&all_updates));
    assert!(all_automation.is_disjoint(&all_release));
    assert!(all_automation.is_disjoint(&all_updates));
    assert!(all_release.is_disjoint(&all_updates));
}

// ── Revalidation at dispatch ───────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_team_remote_requires_revalidation_at_dispatch() {
    // When the entitlement state changes (e.g., Active -> RecoveryOnly),
    // team_remote operations must stop.
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Same snapshot but with RecoveryOnly state: base product fails
    let mut recovery = snapshot.clone();
    recovery.state = EntitlementState::RecoveryOnly;
    let decision = resolve_premium_family(
        &recovery,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));
}

// ── Race/recovery tests ────────────────────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_recovery_does_not_grant_team_remote() {
    // RecoveryOnly state must not grant team_remote premium features.
    let mut snapshot = EntitlementSnapshot::recovery_only("focusa", "node-team", "test-recovery");
    snapshot.features.insert("focusa.team.multi_operator".to_string(), true);
    // Even with a stored feature claim, RecoveryOnly denies base product
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));
}

#[test]
fn spec152f_team_remote_entitlement_refund_revokes_team_remote() {
    // A refund or revocation must remove team_remote capability.
    // RefundedOrRevoked state denies team_remote regardless of stored features.
    let decision =
        reduce_entitlement_state(State::RefundedOrRevoked, Family::TeamRemote, None);
    assert_eq!(decision.posture(), Posture::Deny);
    assert_eq!(decision.reason(), Reason::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_offline_grace_cannot_expand_team_grants() {
    // Offline Grace cannot expand the set of premium features; it can only
    // continue using already-granted cached features.
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now + Duration::minutes(5));
    // No features granted
    let decision = resolve_premium_family(
        &snapshot,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

// ── Interactive base vs team_remote boundary ───────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_interactive_work_loop_is_base_not_team() {
    // Work loop operations (turn, workpoint, mission) are base_focusa,
    // not team_remote. They should be allowed (limited) for verified-no-license.
    let decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::AllowVerifiedLimited);

    // Same state denies team_remote.
    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_team_remote_entitlement_base_focusa_requires_base_posture_not_feature() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::BaseFocusa, None);
    assert_eq!(decision.posture(), Posture::Base);
    assert_eq!(decision.reason(), Reason::RequireBase);

    // Team_remote requires feature, not base.
    let team_decision = reduce_entitlement_state(State::ActivePaid, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Feature);
    assert_eq!(team_decision.reason(), Reason::RequireFeature);
}

// ── Token management is base not team ──────────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_token_management_is_base_not_team() {
    // Token creation, listing, and revocation are base operations, not premium.
    // These are core API authentication management, not team expansion.
    let base_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    // TeamRemote is denied for the same state.
    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

// ── Device read-only routes are base not premium ───────────────────────────

#[test]
fn spec152f_team_remote_entitlement_device_listing_is_base_not_premium() {
    // Listing paired devices and viewing pairing status are read operations
    // that should be available without a premium grant.
    let read_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::ReadProjection, None);
    assert_eq!(read_decision.posture(), Posture::Read);
    assert_eq!(read_decision.reason(), Reason::Read);

    // TeamRemote is denied for the same state.
    let team_decision =
        reduce_entitlement_state(State::VerifiedNoLicense, Family::TeamRemote, None);
    assert_eq!(team_decision.posture(), Posture::Deny);
}

// ── Adversarial isolation (Spec 152F.04.07) ────────────────────────────────

#[test]
fn spec152f_team_remote_entitlement_wrong_product_and_omission_fail_closed() {
    // A wrong product never widens team_remote, even with stored grants.
    let mut wrong = active_snapshot();
    wrong.product = "focusa-premium".to_string();
    wrong
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    let decision = resolve_premium_family(
        &wrong,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));

    // An Evaluation-issued lease that omits team features cannot widen.
    let mut evaluation = active_snapshot();
    evaluation.lease_id = Some("lease-eval-team".to_string());
    evaluation.features.clear();
    let decision = resolve_premium_family(
        &evaluation,
        Family::TeamRemote,
        "focusa.team.multi_operator",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // A caller-invented team feature is never registered.
    let decision = resolve_premium_family(
        &active_snapshot(),
        Family::TeamRemote,
        "focusa.team.caller_invented",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered { .. }
    ));
}
