//! Spec 152F.04.01 — Automation premium boundary enforcement tests.
//!
//! Verifies that interactive base use is not accidentally premium and that
//! unattended/parallel work cannot run without exact grants and capacity.

use chrono::{Duration, Utc};
use focusa_license::{
    authority::{EntitlementSnapshot, EntitlementState},
    premium_family_feature_ids, resolve_premium_family,
    reduce_entitlement_state,
    AUTOMATION_PREMIUM_FEATURE_IDS,
    BaseProductDecision, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
    PremiumFamilyDecision, PremiumFamilyDenial,
};

// ── Automation family map ──────────────────────────────────────────────────

#[test]
fn spec152f_automation_entitlement_automation_family_has_exact_two_premium_features() {
    let features = AUTOMATION_PREMIUM_FEATURE_IDS;
    assert_eq!(features.len(), 2, "automation must have exactly two premium features");
    assert!(features.contains(&"focusa.agent.parallelism"));
    assert!(features.contains(&"focusa.agent.silent_sessions"));
}

#[test]
fn spec152f_automation_entitlement_premium_family_feature_ids_returns_automation_features() {
    let features = premium_family_feature_ids(Family::Automation);
    assert_eq!(features.len(), 2);
    assert!(features.contains(&"focusa.agent.parallelism"));
    assert!(features.contains(&"focusa.agent.silent_sessions"));
}

#[test]
fn spec152f_automation_entitlement_non_premium_families_return_empty_features() {
    for family in [
        Family::AccountRecovery,
        Family::ReadProjection,
        Family::BaseFocusa,
        Family::CustomerDataExport,
        Family::InternalMaintenance,
    ] {
        assert!(
            premium_family_feature_ids(family).is_empty(),
            "{family:?} must not have premium features"
        );
    }
}

#[test]
fn spec152f_automation_entitlement_automation_is_optional_premium_family() {
    assert!(Family::Automation.is_optional_premium());
    assert_eq!(
        Family::Automation.commercial_treatment().label(),
        "optional_premium"
    );
}

// ── State grid: automation is denied for unlicensed states ─────────────────

#[test]
fn spec152f_automation_entitlement_automation_denied_for_verified_no_license() {
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::Automation, None);
    assert_eq!(decision.posture(), Posture::Deny, "automation must be denied for verified-no-license");
    assert_eq!(decision.reason(), Reason::Deny);
}

#[test]
fn spec152f_automation_entitlement_automation_requires_feature_for_active_paid() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::Automation, None);
    assert_eq!(decision.posture(), Posture::Feature, "automation must require feature for active paid");
    assert_eq!(decision.reason(), Reason::RequireFeature);
}

#[test]
fn spec152f_automation_entitlement_automation_requires_cached_feature_for_offline_grace() {
    let decision = reduce_entitlement_state(State::OfflineGrace, Family::Automation, None);
    assert_eq!(decision.posture(), Posture::Feature, "automation must require cached feature for offline grace");
    assert_eq!(decision.reason(), Reason::RequireCachedFeature);
}

#[test]
fn spec152f_automation_entitlement_automation_denied_for_expired_and_revoked() {
    for state in [State::Expired, State::RefundedOrRevoked, State::MissingOrCorrupt] {
        let decision = reduce_entitlement_state(state, Family::Automation, None);
        assert_eq!(decision.posture(), Posture::Deny, "automation must be denied for {state:?}");
        assert_eq!(decision.reason(), Reason::Deny);
    }
}

#[test]
fn spec152f_automation_entitlement_automation_denied_for_pending_unverified() {
    let decision = reduce_entitlement_state(State::PendingUnverified, Family::Automation, None);
    assert_eq!(decision.posture(), Posture::Deny);
    assert_eq!(decision.reason(), Reason::Deny);
}

// ── Base Focusa is not accidentally premium ────────────────────────────────

#[test]
fn spec152f_automation_entitlement_base_focusa_is_not_premium() {
    assert!(!Family::BaseFocusa.is_optional_premium());
    assert_eq!(
        Family::BaseFocusa.commercial_treatment().label(),
        "base_entitlement"
    );
    assert!(premium_family_feature_ids(Family::BaseFocusa).is_empty());

    // Base Focusa operations are allowed (limited) for verified-no-license,
    // while automation is denied for the same state.
    let base_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(base_decision.posture(), Posture::Allow);
    assert_eq!(base_decision.reason(), Reason::AllowVerifiedLimited);

    let auto_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::Automation, None);
    assert_eq!(auto_decision.posture(), Posture::Deny);
}

// ── Premium family resolution for automation ───────────────────────────────

fn active_snapshot() -> EntitlementSnapshot {
    let now = Utc::now();
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-automation");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-auto-001".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:auto".to_string());
    snapshot.expires_at = Some(now + Duration::hours(1));
    snapshot.offline_grace_until = Some(now + Duration::hours(1));
    snapshot
}

#[test]
fn spec152f_automation_entitlement_silent_sessions_requires_automation_premium_feature() {
    let mut snapshot = active_snapshot();
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(decision.is_feature(), "silent_sessions must resolve as premium feature");
    assert_eq!(decision.required_feature().unwrap().as_str(), "focusa.agent.silent_sessions");
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_automation_entitlement_parallelism_requires_automation_premium_feature() {
    let mut snapshot = active_snapshot();
    snapshot.features.insert("focusa.agent.parallelism".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.parallelism",
        Utc::now(),
    );
    assert!(decision.is_feature(), "parallelism must resolve as premium feature");
    assert_eq!(decision.required_feature().unwrap().as_str(), "focusa.agent.parallelism");
    assert_eq!(decision.lease_sequence(), Some(7));
}

#[test]
fn spec152f_automation_entitlement_silent_sessions_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    // No features granted at all
    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_automation_entitlement_parallelism_denied_without_feature_grant() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.parallelism",
        Utc::now(),
    );
    assert!(decision.denial().is_some());
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

#[test]
fn spec152f_automation_entitlement_automation_requires_base_product_first() {
    // Wrong product — base gate fails before feature check
    let mut snapshot = active_snapshot();
    snapshot.product = "uiai-engine".to_string();
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
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
fn spec152f_automation_entitlement_automation_requires_non_zero_lease_sequence() {
    let mut snapshot = active_snapshot();
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);
    snapshot.sequence = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseSequence
    ));
}

#[test]
fn spec152f_automation_entitlement_automation_requires_lease_binding() {
    let mut snapshot = active_snapshot();
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);
    snapshot.lease_digest = None;

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingLeaseBinding
    ));
}

#[test]
fn spec152f_automation_entitlement_cross_family_feature_is_rejected() {
    let mut snapshot = active_snapshot();
    // Grant a feature from a different family
    snapshot.features.insert("focusa.release.proof".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.release.proof",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered {
            family: Family::Automation,
            ..
        }
    ));
}

#[test]
fn spec152f_automation_entitlement_unqualified_feature_identifier_is_rejected() {
    let snapshot = active_snapshot();

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "not.a.qualified.feature",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::InvalidRequiredFeature { .. }
    ));
}

#[test]
fn spec152f_automation_entitlement_offline_grace_within_window_allows_automation() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now + Duration::minutes(5));
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        now,
    );
    assert!(decision.is_feature(), "offline grace within window must allow automation");
    assert!(matches!(
        decision,
        PremiumFamilyDecision::Feature {
            offline_cached: true,
            ..
        }
    ));
}

#[test]
fn spec152f_automation_entitlement_offline_grace_expired_denies_automation() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.state = EntitlementState::OfflineGrace;
    snapshot.offline_grace_until = Some(now - Duration::minutes(1)); // expired
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::CachedGrantExpired
    ));
}

#[test]
fn spec152f_automation_entitlement_active_lease_expired_denies_automation() {
    let now = Utc::now();
    let mut snapshot = active_snapshot();
    snapshot.expires_at = Some(now - Duration::minutes(1)); // expired
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        now,
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::ActiveLeaseExpired
    ));
}

#[test]
fn spec152f_automation_entitlement_non_premium_family_rejected_by_resolver() {
    let snapshot = active_snapshot();
    let decision = resolve_premium_family(
        &snapshot,
        Family::BaseFocusa,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::NotPremiumFamily {
            family: Family::BaseFocusa,
        }
    ));
}

// ── Interactive base vs automation boundary ────────────────────────────────

#[test]
fn spec152f_automation_entitlement_interactive_work_loop_is_base_not_automation() {
    // Work loop operations (turn start/append/complete) are base_focusa,
    // not automation. They should be allowed (limited) for verified-no-license.
    let decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::BaseFocusa, None);
    assert_eq!(decision.posture(), Posture::Allow);
    assert_eq!(decision.reason(), Reason::AllowVerifiedLimited);

    // Same state denies automation.
    let auto_decision = reduce_entitlement_state(State::VerifiedNoLicense, Family::Automation, None);
    assert_eq!(auto_decision.posture(), Posture::Deny);
}

#[test]
fn spec152f_automation_entitlement_base_focusa_requires_base_posture_not_feature() {
    let decision = reduce_entitlement_state(State::ActivePaid, Family::BaseFocusa, None);
    assert_eq!(decision.posture(), Posture::Base);
    assert_eq!(decision.reason(), Reason::RequireBase);

    // Automation requires feature, not base.
    let auto_decision = reduce_entitlement_state(State::ActivePaid, Family::Automation, None);
    assert_eq!(auto_decision.posture(), Posture::Feature);
    assert_eq!(auto_decision.reason(), Reason::RequireFeature);
}

// ── Automation feature IDs are authority-owned ─────────────────────────────

#[test]
fn spec152f_automation_entitlement_automation_feature_ids_are_qualified_focusa_identifiers() {
    for feature in AUTOMATION_PREMIUM_FEATURE_IDS {
        assert!(
            feature.starts_with("focusa."),
            "automation feature {feature} must be a qualified Focusa identifier"
        );
        assert!(
            feature.len() > "focusa.".len(),
            "automation feature {feature} must have a non-empty sub-identifier"
        );
        // Must be valid focusa identifiers (lowercase, dots, no uppercase)
        assert!(
            feature
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
            "automation feature {feature} must use only lowercase/dot/underscore"
        );
    }
}

#[test]
fn spec152f_automation_entitlement_automation_features_are_distinct() {
    // The two automation features must be distinct
    assert_ne!(
        AUTOMATION_PREMIUM_FEATURE_IDS[0],
        AUTOMATION_PREMIUM_FEATURE_IDS[1],
        "automation premium features must be distinct"
    );
}

// ── Subagent surfaces are automation not base ──────────────────────────────

#[test]
fn spec152f_automation_entitlement_subagent_requires_automation_not_base() {
    // Subagent operations are automation, not base_focusa.
    // Base focusa covers interactive work loops; subagent is autonomous.
    assert!(Family::Automation.is_optional_premium());
    assert!(!Family::BaseFocusa.is_optional_premium());

    // Subagent feature (silent_sessions) maps to automation family
    let features = premium_family_feature_ids(Family::Automation);
    assert!(features.contains(&"focusa.agent.silent_sessions"));
}

// ── Provider parallelism surfaces are automation not base ──────────────────

#[test]
fn spec152f_automation_entitlement_provider_parallelism_requires_automation_not_base() {
    // Provider parallelism (conformance, contracts) is automation, not base.
    let features = premium_family_feature_ids(Family::Automation);
    assert!(features.contains(&"focusa.agent.parallelism"));
    assert!(!premium_family_feature_ids(Family::BaseFocusa).contains(&"focusa.agent.parallelism"));
}

// ── Limits tests ───────────────────────────────────────────────────────────

#[test]
fn spec152f_automation_entitlement_automation_limits_are_separate_from_base_limits() {
    // Automation has its own limit buckets (silent_session_runs, parallel_agents)
    // that are separate from base_focusa limit buckets (workpoints, missions, evidence_records).
    // The limit buckets are defined in the route entitlement table and the
    // daemon dispatch operation policy. This test verifies the separation at
    // the family level.

    // Base focusa is not premium and does not use feature-based limits
    assert!(!Family::BaseFocusa.is_optional_premium());

    // Automation is premium and uses feature-based limits
    assert!(Family::Automation.is_optional_premium());

    // Each automation feature maps to exactly one limit bucket via the
    // route entitlement table and daemon dispatch policy.
    // The feature identifiers themselves encode the limit domain:
    assert!(AUTOMATION_PREMIUM_FEATURE_IDS.iter().any(|f| f.contains("silent_sessions")));
    assert!(AUTOMATION_PREMIUM_FEATURE_IDS.iter().any(|f| f.contains("parallelism")));
}

// ── Revalidation at dispatch ───────────────────────────────────────────────

#[test]
fn spec152f_automation_entitlement_automation_requires_revalidation_at_dispatch() {
    // When the entitlement state changes (e.g., Active -> RecoveryOnly),
    // automation dispatch must stop. This is tested in the silent session
    // scheduler's entitlement revalidation tests (scheduler_entitlement_revalidation_stops_when_entitlement_is_recovered_or_revoked).
    // Here we verify the policy layer that makes that possible.

    // Active paid with automation feature: premium family resolves
    let mut snapshot = active_snapshot();
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);
    let decision = resolve_premium_family(
        &snapshot, Family::Automation, "focusa.agent.silent_sessions", Utc::now(),
    );
    assert!(decision.is_feature());

    // Same snapshot but with RecoveryOnly state: base product fails
    let mut recovery = snapshot.clone();
    recovery.state = EntitlementState::RecoveryOnly;
    let decision = resolve_premium_family(
        &recovery, Family::Automation, "focusa.agent.silent_sessions", Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::BaseProductRequired { .. }
    ));
}

// ── Reserve limits before spawn ────────────────────────────────────────────

#[test]
fn spec152f_automation_entitlement_automation_limits_reserved_before_spawn() {
    // The automation premium family requires both the feature grant AND
    // a limit reservation before spawning unattended work. This is enforced
    // by the route entitlement middleware (Idempotency-Key header required
    // for limit-bucketed routes) and the daemon dispatch ledger.

    // The feature resolution must succeed before limit reservation is possible.
    let mut snapshot = active_snapshot();
    snapshot
        .features
        .insert("focusa.agent.silent_sessions".to_string(), true);
    snapshot
        .limits
        .insert("silent_session_runs".to_string(), 4);

    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(decision.is_feature());

    // Without the feature grant, resolution fails before limits are checked.
    let mut no_feature = active_snapshot();
    no_feature
        .limits
        .insert("silent_session_runs".to_string(), 4);
    let decision = resolve_premium_family(
        &no_feature,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));
}

// ── No caller-controlled grants ────────────────────────────────────────────

#[test]
fn spec152f_automation_entitlement_caller_cannot_control_automation_grants() {
    // The automation premium feature IDs are authority-owned constants.
    // Callers cannot supply their own feature identifier and get a grant.
    // Only the exact registered feature identifiers are accepted.

    let mut snapshot = active_snapshot();
    // Grant only the silent_sessions feature
    snapshot.features.insert("focusa.agent.silent_sessions".to_string(), true);

    // Requesting a different feature that is not granted fails
    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.parallelism",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::MissingFeature { .. }
    ));

    // Requesting a caller-invented feature fails at the identifier validation
    let decision = resolve_premium_family(
        &snapshot,
        Family::Automation,
        "focusa.agent.caller_invented",
        Utc::now(),
    );
    assert!(matches!(
        decision.denial().unwrap(),
        PremiumFamilyDenial::FeatureNotRegistered { .. }
    ));
}

// ── All four premium families are distinct ─────────────────────────────────

#[test]
fn spec152f_automation_entitlement_all_premium_families_are_distinct() {
    use focusa_license::{
        PREMIUM_UPDATES_PREMIUM_FEATURE_IDS,
        RELEASE_PROOF_PREMIUM_FEATURE_IDS,
        TEAM_REMOTE_PREMIUM_FEATURE_IDS,
    };

    let all_automation: std::collections::HashSet<&str> =
        AUTOMATION_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_team: std::collections::HashSet<&str> =
        TEAM_REMOTE_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_release: std::collections::HashSet<&str> =
        RELEASE_PROOF_PREMIUM_FEATURE_IDS.iter().copied().collect();
    let all_updates: std::collections::HashSet<&str> =
        PREMIUM_UPDATES_PREMIUM_FEATURE_IDS.iter().copied().collect();

    // No feature belongs to more than one premium family
    assert!(all_automation.is_disjoint(&all_team));
    assert!(all_automation.is_disjoint(&all_release));
    assert!(all_automation.is_disjoint(&all_updates));
    assert!(all_team.is_disjoint(&all_release));
    assert!(all_team.is_disjoint(&all_updates));
    assert!(all_release.is_disjoint(&all_updates));
}