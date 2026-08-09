//! Spec 172-overlaid Spec 152F complete entitlement-state grid acceptance.
//!
//! Replayable acceptance receipt for atom `focusa-vbcqu.20.14.43`
//! (152F.06.01). This module is compiled only under `cargo test` and is
//! reached by the exact verification filter
//! `cargo test --workspace spec152f_state_grid_acceptance`.
//!
//! It replays, from this API middleware seat (which owns both the focusa-core
//! execution guard and the focusa-license canonical resolver):
//!   1. policy golden vectors — the committed 7-state × 9-family fixture
//!      (`tests/fixtures/spec152f-entitlement-policy-cases.v1.json`) is
//!      replayed against the canonical resolver;
//!   2. canonical resolver — `reduce_entitlement_state` for every grid cell,
//!      positive and negative (caller-override) cases, deterministic and
//!      fail-closed;
//!   3. API/core guards — the focusa-core execution guard and this
//!      middleware's route gate produce commercial decisions identical to the
//!      pure resolver for every guard-representable state, and the recovery
//!      matrix stays reachable in every blocked state;
//!   4. Spec 172 overlay — the grid is bound to `verified_no_license` (an
//!      account/runtime posture, never a license and never Evaluation) and to
//!      Operator License Types only; no Evaluation state, no separately
//!      purchased premium family satisfies the base gate.
//!
//! No raw keys, tokens, customer identifiers, or prices appear in this
//! module; all snapshots are synthetic acceptance fixtures.

use super::*;
use focusa_core::limited_project::{ActiveProjectGuard, ActiveProjectSelection};
use focusa_license::{
    BaseProductDecision, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, LicenseTypeCode, OperationClass,
    PolicyEntitlementState as State, SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    authority_policy_state, base_product_projection, premium_family_feature_ids,
    reduce_entitlement_state, resolve_base_focusa_product, resolve_premium_family,
};
use focusa_license::authority::EntitlementSnapshot;
use focusa_license::uiai_activation::{
    PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1, PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1,
};

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/spec152f-entitlement-policy-cases.v1.json"
);

const STATES: [State; 7] = [
    State::PendingUnverified,
    State::VerifiedNoLicense,
    State::ActivePaid,
    State::OfflineGrace,
    State::Expired,
    State::RefundedOrRevoked,
    State::MissingOrCorrupt,
];

const FAMILIES: [Family; 9] = [
    Family::AccountRecovery,
    Family::ReadProjection,
    Family::BaseFocusa,
    Family::Automation,
    Family::TeamRemote,
    Family::ReleaseProof,
    Family::PremiumUpdates,
    Family::CustomerDataExport,
    Family::InternalMaintenance,
];

fn parse_state(value: &str) -> State {
    match value {
        "pending_unverified" => State::PendingUnverified,
        "verified_no_license" => State::VerifiedNoLicense,
        "active_paid" => State::ActivePaid,
        "offline_grace" => State::OfflineGrace,
        "expired" => State::Expired,
        "refunded_or_revoked" => State::RefundedOrRevoked,
        "missing_or_corrupt" => State::MissingOrCorrupt,
        other => panic!("unknown fixture state: {other}"),
    }
}

fn parse_family(value: &str) -> Family {
    match value {
        "account_recovery" => Family::AccountRecovery,
        "read_projection" => Family::ReadProjection,
        "base_focusa" => Family::BaseFocusa,
        "automation" => Family::Automation,
        "team_remote" => Family::TeamRemote,
        "release_proof" => Family::ReleaseProof,
        "premium_updates" => Family::PremiumUpdates,
        "customer_data_export" => Family::CustomerDataExport,
        "internal_maintenance" => Family::InternalMaintenance,
        other => panic!("unknown fixture family: {other}"),
    }
}

/// The frozen posture/reason pair for one fixture decision string. `None`
/// posture means the row is `inherit` (resolved only with an initiating
/// posture); every other row carries an explicit posture.
fn parse_decision(value: &str) -> (Option<Posture>, Reason) {
    match value {
        "allow" => (Some(Posture::Allow), Reason::Allow),
        "allow_offline_only" => (Some(Posture::Allow), Reason::AllowOfflineOnly),
        "allow_existing_local_only" => (Some(Posture::Allow), Reason::AllowExistingLocalOnly),
        "read" => (Some(Posture::Read), Reason::Read),
        "read_local_only" => (Some(Posture::Read), Reason::ReadLocalOnly),
        "allow_verified_limited" => (Some(Posture::Allow), Reason::AllowVerifiedLimited),
        "require_base" => (Some(Posture::Base), Reason::RequireBase),
        "require_feature" => (Some(Posture::Feature), Reason::RequireFeature),
        "require_cached_feature" => (Some(Posture::Feature), Reason::RequireCachedFeature),
        "require_cached_feature_when_safe" => {
            (Some(Posture::Feature), Reason::RequireCachedFeatureWhenSafe)
        }
        "deny" => (Some(Posture::Deny), Reason::Deny),
        "inherit" => (None, Reason::Inherit),
        other => panic!("unknown fixture decision: {other}"),
    }
}

/// Synthetic signed-authority snapshot for every guard-representable policy
/// state. `VerifiedNoLicense` and `Expired` are not snapshot states: the
/// former is an account/runtime posture carried by a signed limited-access
/// assertion, and the latter is an expired lease enforced at the lease
/// validation boundary, so neither is constructed here.
fn snapshot_for(state: State) -> EntitlementSnapshot {
    match state {
        State::PendingUnverified => EntitlementSnapshot::unactivated("focusa", "node-accept"),
        State::ActivePaid => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-accept");
            snapshot.state = EntitlementState::Active;
            snapshot.lease_id = Some("lease-accept".to_string());
            snapshot.lease_digest = Some("sha256:accept".to_string());
            snapshot.sequence = Some(7);
            snapshot.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
            snapshot
        }
        State::OfflineGrace => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-accept");
            snapshot.state = EntitlementState::OfflineGrace;
            snapshot.lease_id = Some("lease-accept".to_string());
            snapshot.lease_digest = Some("sha256:accept".to_string());
            snapshot.sequence = Some(7);
            snapshot.offline_grace_until = Some(chrono::Utc::now() + chrono::Duration::hours(1));
            snapshot
        }
        State::RefundedOrRevoked => {
            let mut snapshot =
                EntitlementSnapshot::recovery_only("focusa", "node-accept", "refunded_or_revoked");
            snapshot.lease_id = Some("lease-refunded".to_string());
            snapshot.lease_digest = Some("sha256:refunded".to_string());
            snapshot.sequence = Some(1);
            snapshot
        }
        State::VerifiedNoLicense | State::Expired | State::MissingOrCorrupt => {
            unreachable!("not a signed-snapshot state")
        }
    }
}

/// The five policy states the execution guard can derive from a
/// `LicenseGuard` (`authority_policy_state`, with `MissingOrCorrupt` for a
/// guard without any signed entitlement).
fn guard_for(state: State) -> LicenseGuard {
    match state {
        State::MissingOrCorrupt => LicenseGuard::eval(7),
        _ => LicenseGuard::from_entitlement(snapshot_for(state)),
    }
}

/// Canonical operation policy for one capability family. Premium families
/// carry their registered required feature; non-premium families never do.
fn policy_for(family: Family) -> EntitlementExecutionPolicy {
    let (operation_class, feature) = match family {
        Family::AccountRecovery => (OperationClass::Recovery, None),
        Family::ReadProjection => (OperationClass::Read, None),
        Family::BaseFocusa => (OperationClass::ValueMutation, None),
        Family::Automation => (
            OperationClass::ValueMutation,
            Some("focusa.agent.silent_sessions"),
        ),
        Family::TeamRemote => (
            OperationClass::ValueMutation,
            Some("focusa.team.multi_operator"),
        ),
        Family::ReleaseProof => (OperationClass::ValueMutation, Some("focusa.release.proof")),
        Family::PremiumUpdates => (
            OperationClass::ValueMutation,
            Some("focusa.update.unattended"),
        ),
        Family::CustomerDataExport => (OperationClass::ValueMutation, None),
        Family::InternalMaintenance => (OperationClass::InternalMaintenance, None),
    };
    EntitlementExecutionPolicy::new(
        format!("accept.{}", family.label()),
        operation_class,
        family,
        feature,
        None,
        RecoveryAllowance::None,
    )
}

fn expected_denial_code(resolver_reason: Reason, family: Family) -> &'static str {
    if resolver_reason == Reason::MissingInitiatingPolicy {
        "ENTITLEMENT_ROUTE_UNCLASSIFIED"
    } else if family == Family::BaseFocusa {
        "ENTITLEMENT_BASE_REQUIRED"
    } else {
        "ENTITLEMENT_REQUIRED"
    }
}

// ── 1. Policy golden vectors ───────────────────────────────────────────────

#[test]
fn spec152f_state_grid_acceptance_golden_vectors_replay_all_63_cells() {
    let raw = std::fs::read_to_string(FIXTURE_PATH).expect("golden fixture must exist");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("golden fixture JSON");

    assert_eq!(fixture["schema"], "focusa.spec152f.entitlement_policy_cases.v1");
    assert_eq!(fixture["policy_id"], "focusa-simple-entitlement");
    assert_eq!(fixture["policy_version"], 1);
    assert_eq!(fixture["grid_case_count"], 63);
    assert_eq!(fixture["state_count"], 7);
    assert_eq!(fixture["family_count"], 9);
    assert_eq!(fixture["feature_compatibility_count"], 15);

    let cases = fixture["grid_cases"].as_array().expect("grid_cases array");
    assert_eq!(cases.len(), 63, "exactly 7 states × 9 families");

    let mut seen = std::collections::BTreeSet::new();
    for case in cases {
        let state_label = case["state"].as_str().expect("state label");
        let family_label = case["family"].as_str().expect("family label");
        let expected = case["expected_decision"].as_str().expect("expected decision");
        assert_eq!(
            case["case_id"],
            format!("{state_label}::{family_label}"),
            "fixture case_id must be state::family"
        );
        assert!(
            seen.insert((state_label.to_string(), family_label.to_string())),
            "duplicate fixture pair: {state_label}::{family_label}"
        );

        let state = parse_state(state_label);
        let family = parse_family(family_label);
        let (posture, reason) = parse_decision(expected);
        let initiating = if family == Family::InternalMaintenance {
            Some(Posture::Deny)
        } else {
            None
        };
        let decision = reduce_entitlement_state(state, family, initiating);
        assert_eq!(
            decision.posture(),
            posture.unwrap_or(Posture::Deny),
            "golden posture for {state_label}::{family_label} (expected {expected})"
        );
        assert_eq!(
            decision.reason(),
            reason,
            "golden reason for {state_label}::{family_label} (expected {expected})"
        );
    }
    assert_eq!(seen.len(), 63, "fixture must cover every state/family pair");
}

// ── 2. Canonical resolver ──────────────────────────────────────────────────

#[test]
fn spec152f_state_grid_acceptance_resolver_is_deterministic_and_fails_closed() {
    for state in STATES {
        for family in FAMILIES {
            let first = reduce_entitlement_state(state, family, None);
            let second = reduce_entitlement_state(state, family, None);
            assert_eq!(first, second, "resolver must be deterministic: {state:?}/{family:?}");
        }
    }

    // Internal maintenance inherits its initiating posture and never grants
    // itself; without an initiator it fails closed.
    for state in STATES {
        let missing = reduce_entitlement_state(state, Family::InternalMaintenance, None);
        assert_eq!(missing.posture(), Posture::Deny);
        assert_eq!(missing.reason(), Reason::MissingInitiatingPolicy);

        let inherited =
            reduce_entitlement_state(state, Family::InternalMaintenance, Some(Posture::Deny));
        assert_eq!(inherited.reason(), Reason::Inherit);
        assert_eq!(inherited.posture(), Posture::Deny);
    }

    // Spec 172 overlay: no Evaluation state exists anywhere in the grid, and
    // verified_no_license is a first-class posture (never a license, never a
    // trial).
    assert!(serde_json::from_str::<State>("\"evaluation\"").is_err());
    assert!(serde_json::from_str::<State>("\"unknown\"").is_err());
    assert_eq!(
        serde_json::from_str::<State>("\"verified_no_license\"").expect("posture"),
        State::VerifiedNoLicense
    );
    assert_eq!(State::VerifiedNoLicense.label(), "verified_no_license");
}

// ── 3. Core execution guard identical to resolver ──────────────────────────

#[test]
fn spec152f_state_grid_acceptance_core_guard_identical_to_resolver() {
    let guard_states = [
        State::PendingUnverified,
        State::ActivePaid,
        State::OfflineGrace,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ];

    for state in guard_states {
        for family in FAMILIES {
            if family == Family::InternalMaintenance {
                continue; // exercised separately below with an initiator
            }
            let resolver = reduce_entitlement_state(state, family, None);

            // Grant the registered premium feature so the guard's Feature
            // decision is reachable and must equal the resolver's Feature row.
            let guard = if family.is_optional_premium()
                && matches!(state, State::ActivePaid | State::OfflineGrace)
            {
                let mut snap = snapshot_for(state);
                for feature in premium_family_feature_ids(family) {
                    snap.features.insert(feature.to_string(), true);
                }
                LicenseGuard::from_entitlement(snap)
            } else {
                guard_for(state)
            };

            let outcome = evaluate_entitlement_execution(
                &guard,
                &policy_for(family),
                EntitlementExecutionContext::default(),
            );

            match resolver.posture() {
                Posture::Allow | Posture::Read => {
                    let decision = outcome.expect("allow/read must pass the core guard");
                    assert_eq!(
                        decision.status,
                        resolver.posture().status(),
                        "{state:?}/{family:?}"
                    );
                    assert_eq!(decision.reason_code, resolver.reason().label());
                }
                Posture::Base => {
                    let decision = outcome.expect("base posture must pass with a usable signed lease");
                    assert_eq!(decision.status, "base", "{state:?}/{family:?}");
                    assert_eq!(decision.reason_code, resolver.reason().label());
                }
                Posture::Feature => {
                    let decision = outcome.expect("granted premium feature must pass the core guard");
                    assert_eq!(decision.status, "feature", "{state:?}/{family:?}");
                    assert_eq!(decision.reason_code, resolver.reason().label());
                }
                Posture::Deny => {
                    let failure = outcome.expect_err("denied cell must fail the core guard");
                    assert_eq!(
                        failure.code,
                        expected_denial_code(resolver.reason(), family),
                        "{state:?}/{family:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn spec152f_state_grid_acceptance_internal_maintenance_core_guard_matches_resolver() {
    for state in [
        State::PendingUnverified,
        State::ActivePaid,
        State::OfflineGrace,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ] {
        let guard = guard_for(state);

        // With an initiator the core guard inherits it (resolver: Inherit).
        // An Allow initiator passes; a Deny initiator fails closed with the
        // inherited posture.
        let with_allow = evaluate_entitlement_execution(
            &guard,
            &policy_for(Family::InternalMaintenance),
            EntitlementExecutionContext {
                initiating_posture: Some(Posture::Allow),
                ..EntitlementExecutionContext::default()
            },
        )
        .expect("internal maintenance with allow initiator must inherit");
        assert_eq!(with_allow.status, "allow");
        assert_eq!(with_allow.reason_code, Reason::Inherit.label());

        let with_deny = evaluate_entitlement_execution(
            &guard,
            &policy_for(Family::InternalMaintenance),
            EntitlementExecutionContext {
                initiating_posture: Some(Posture::Deny),
                ..EntitlementExecutionContext::default()
            },
        )
        .expect_err("internal maintenance with deny initiator must fail closed");
        assert_eq!(with_deny.code, "ENTITLEMENT_REQUIRED");

        // Without an initiator it fails closed (resolver: MissingInitiatingPolicy).
        let without_initiator = evaluate_entitlement_execution(
            &guard,
            &policy_for(Family::InternalMaintenance),
            EntitlementExecutionContext::default(),
        )
        .expect_err("internal maintenance without initiator must fail closed");
        assert_eq!(without_initiator.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");
    }
}

// ── 4. Grant isolation: premium families are never separately purchased ────

#[test]
fn spec152f_state_grid_acceptance_premium_grant_isolation_matches_resolver() {
    // Resolver says Feature for premium families in ActivePaid; the core
    // guard enforces the exact authority-owned feature grant: without the
    // grant it denies, with the grant it resolves to Feature. A caller can
    // never purchase or self-issue a premium family.
    let mut snapshot = snapshot_for(State::ActivePaid);
    let guard_without = LicenseGuard::from_entitlement(snapshot.clone());
    let failure = evaluate_entitlement_execution(
        &guard_without,
        &policy_for(Family::TeamRemote),
        EntitlementExecutionContext::default(),
    )
    .expect_err("premium without grant must be denied");
    assert_eq!(failure.code, "ENTITLEMENT_FEATURE_REQUIRED");

    snapshot
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    let guard_with = LicenseGuard::from_entitlement(snapshot);
    let decision = evaluate_entitlement_execution(
        &guard_with,
        &policy_for(Family::TeamRemote),
        EntitlementExecutionContext::default(),
    )
    .expect("premium with grant must pass");
    assert_eq!(decision.status, "feature");
    assert_eq!(decision.reason_code, Reason::RequireFeature.label());

    // resolve_premium_family agrees at the policy boundary.
    let granted = resolve_premium_family(
        &guard_with.entitlement.as_ref().unwrap(),
        Family::TeamRemote,
        "focusa.team.multi_operator",
        chrono::Utc::now(),
    );
    assert!(granted.is_feature());

    // Stored premium feature claims never widen the base gate: only product +
    // policy state decide. A stored premium grant on a non-usable posture or
    // wrong product stays limited/denied.
    assert_eq!(
        resolve_base_focusa_product(
            "focusa",
            State::VerifiedNoLicense
        ),
        BaseProductDecision::Limited
    );
    assert_eq!(
        resolve_base_focusa_product("focusa", State::RefundedOrRevoked),
        BaseProductDecision::Denied
    );
    assert_eq!(
        resolve_base_focusa_product("premium-alias", State::ActivePaid),
        BaseProductDecision::Denied
    );

    // The canonical projection confirms the compatibility claim: legacy core
    // identifiers are base-product claims, not separately purchased features.
    let projection =
        base_product_projection(guard_with.entitlement.as_ref()).expect("base projection");
    assert!(projection.permits_base_mutations);
    for id in focusa_license::BASE_PRODUCT_CORE_COMPATIBILITY_IDS {
        assert_eq!(projection.compatibility.get(id), Some(&true));
    }
}

// ── 5. API route gate identical to resolver + recovery matrix ──────────────

const BASE_MUTATION_ROUTE: (&str, Method, Family) =
    ("/v1/workpoint/checkpoint", Method::POST, Family::BaseFocusa);

const PREMIUM_ROUTES: [(&str, Method, Family); 4] = [
    ("/v1/silent-sessions", Method::POST, Family::Automation),
    ("/v1/connect/room/create", Method::POST, Family::TeamRemote),
    ("/v1/release/proof/status", Method::POST, Family::ReleaseProof),
    ("/v1/update/scheduler", Method::POST, Family::PremiumUpdates),
];

#[test]
fn spec152f_state_grid_acceptance_api_route_gate_identical_to_resolver() {
    // Blocked states: resolver denies base/premium, and so does the route gate.
    let blocked = [
        ("missing", LicenseGuard::eval(7)),
        (
            "unactivated",
            LicenseGuard::from_entitlement(snapshot_for(State::PendingUnverified)),
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(snapshot_for(State::RefundedOrRevoked)),
        ),
    ];

    for (label, guard) in &blocked {
        let denial = route_entitlement_denial(guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0);
        assert_eq!(
            denial.as_ref().map(|d| d.code.as_str()),
            Some("ENTITLEMENT_BASE_REQUIRED"),
            "base mutation must be denied in state {label}"
        );
        for (path, method, _family) in &PREMIUM_ROUTES {
            let denial = route_entitlement_denial(guard, method, path);
            let code = denial.expect("premium route must be denied").code;
            assert!(
                matches!(
                    code.as_str(),
                    "ENTITLEMENT_BASE_REQUIRED"
                        | "ENTITLEMENT_REQUIRED"
                        | "ENTITLEMENT_FEATURE_REQUIRED"
                ),
                "{path} in state {label}: unexpected code {code}"
            );
        }
    }

    // Active paid without grants: base passes, premium requires its grant —
    // exactly the resolver's Base/Feature rows with grant isolation at the gate.
    let mut active = snapshot_for(State::ActivePaid);
    let active_guard = LicenseGuard::from_entitlement(active.clone());
    assert_eq!(
        route_entitlement_denial(&active_guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0),
        None,
        "base mutation must pass with a valid signed lease"
    );
    for (path, method, _family) in &PREMIUM_ROUTES {
        let denial = route_entitlement_denial(&active_guard, method, path)
            .expect("premium without grant must be denied");
        assert_eq!(denial.code, "ENTITLEMENT_FEATURE_REQUIRED", "{path}");
    }

    // Active paid with the exact authority grant: premium passes.
    active
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    active
        .features
        .insert("focusa.agent.silent_sessions".to_string(), true);
    active
        .features
        .insert("focusa.release.proof".to_string(), true);
    active
        .features
        .insert("focusa.update.unattended".to_string(), true);
    let granted_guard = LicenseGuard::from_entitlement(active);
    for (path, method, _family) in &PREMIUM_ROUTES {
        assert_eq!(
            route_entitlement_denial(&granted_guard, method, path),
            None,
            "{path} must pass with the granted premium feature"
        );
    }

    // Offline Grace with a valid cached lease: base passes; premium requires
    // the cached grant.
    let mut grace = snapshot_for(State::OfflineGrace);
    grace
        .features
        .insert("focusa.team.multi_operator".to_string(), true);
    let grace_guard = LicenseGuard::from_entitlement(grace);
    assert_eq!(
        route_entitlement_denial(&grace_guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0),
        None,
        "base mutation must pass during valid offline grace"
    );
    assert_eq!(
        route_entitlement_denial(&grace_guard, &Method::POST, "/v1/connect/room/create"),
        None,
        "cached premium grant must pass during valid offline grace"
    );

    // Expired lease: the resolver's Expired row denies value production and
    // keeps recovery; the route gate enforces the same boundary via lease
    // validation before any handler runs.
    let mut expired = EntitlementSnapshot::unactivated("focusa", "node-accept");
    expired.state = EntitlementState::Active;
    expired.lease_id = Some("lease-expired".to_string());
    expired.lease_digest = Some("sha256:expired".to_string());
    expired.sequence = Some(7);
    expired.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
    let expired_guard = LicenseGuard::from_entitlement(expired);
    assert_eq!(
        route_entitlement_denial(&expired_guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0)
            .expect("expired base must be denied")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );
    for (path, method, _family) in &PREMIUM_ROUTES {
        assert!(
            route_entitlement_denial(&expired_guard, method, path).is_some(),
            "{path} must be denied for an expired lease"
        );
    }
}

#[test]
fn spec152f_state_grid_acceptance_recovery_matrix_matches_resolver_rows() {
    // Resolver rows: AccountRecovery / CustomerDataExport are Allow in every
    // state, ReadProjection is Read in every state; the API recovery routes
    // for those families must never be denied, in any blocked state.
    let recovery_routes: Vec<(&str, &Method, &str)> = vec![
        ("stable_update", &Method::POST, "/v1/update/apply"),
        ("repair", &Method::POST, "/v1/project/bootstrap/repair"),
        ("rollback", &Method::POST, "/v1/update/rollback"),
        ("export_run", &Method::POST, "/v1/export/run"),
        ("export_status", &Method::GET, "/v1/export/status"),
        ("export_history", &Method::GET, "/v1/export/history"),
        ("export_manifest", &Method::GET, "/v1/export/manifest/manifest-1"),
        ("node_deactivation", &Method::POST, "/v1/device/pair/revoke"),
        ("pairing_status", &Method::GET, "/v1/device/pair/status"),
        ("diagnostics", &Method::GET, "/v1/doctor"),
        ("diagnostics_closure", &Method::GET, "/v1/doctor/closure"),
        ("license_status", &Method::GET, "/v1/license/status"),
    ];

    let blocked_guards: Vec<(&str, LicenseGuard)> = vec![
        ("missing", LicenseGuard::eval(7)),
        (
            "unactivated",
            LicenseGuard::from_entitlement(snapshot_for(State::PendingUnverified)),
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(snapshot_for(State::RefundedOrRevoked)),
        ),
        ("expired", {
            let mut expired = EntitlementSnapshot::unactivated("focusa", "node-accept");
            expired.state = EntitlementState::Active;
            expired.lease_id = Some("lease-expired".to_string());
            expired.lease_digest = Some("sha256:expired".to_string());
            expired.sequence = Some(7);
            expired.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
            LicenseGuard::from_entitlement(expired)
        }),
    ];

    for (label, guard) in &blocked_guards {
        for (route_label, method, path) in &recovery_routes {
            assert!(
                route_entitlement_denial(guard, method, path).is_none(),
                "{route_label} ({path}) must remain available in state {label}"
            );
        }
    }

    // And the resolver agrees for every blocked state: recovery families are
    // Allow/Read across Expired / RefundedOrRevoked / MissingOrCorrupt
    // (account_recovery and customer_data_export allow; read_projection
    // reads). PendingUnverified is stricter: read_projection is denied until
    // the operator's identity is verified (P7: authentication is not
    // entitlement, but unverified accounts get no read projection).
    for state in [
        State::Expired,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ] {
        assert_eq!(
            reduce_entitlement_state(state, Family::AccountRecovery, None).posture(),
            Posture::Allow
        );
        assert_eq!(
            reduce_entitlement_state(state, Family::CustomerDataExport, None).posture(),
            Posture::Allow
        );
        assert_eq!(
            reduce_entitlement_state(state, Family::ReadProjection, None).posture(),
            Posture::Read
        );
    }
    assert_eq!(
        reduce_entitlement_state(State::PendingUnverified, Family::AccountRecovery, None)
            .posture(),
        Posture::Allow
    );
    assert_eq!(
        reduce_entitlement_state(State::PendingUnverified, Family::CustomerDataExport, None)
            .posture(),
        Posture::Allow
    );
    assert_eq!(
        reduce_entitlement_state(State::PendingUnverified, Family::ReadProjection, None).posture(),
        Posture::Deny,
        "unverified accounts get no read projection"
    );
}

// ── 6. Spec 172 overlay: verified_no_license + Operator License Types ──────

#[test]
fn spec152f_state_grid_acceptance_spec172_overlay_verified_no_license_and_operator_types() {
    // Operator License Types are the only paid product codes: Operator
    // Lifetime v1 for Focusa and UIAI (and the composite bundle SKU). No
    // Evaluation code, tier, or separately purchased premium family is a
    // License Type.
    assert_eq!(PUBLIC_CODE_FOCUSA_OPERATOR_LIFETIME_V1, "focusa_operator_lifetime_v1");
    assert_eq!(PUBLIC_CODE_UIAI_OPERATOR_LIFETIME_V1, "uiai_operator_lifetime_v1");
    let operator_codes = [
        LicenseTypeCode::FocusaOperatorLifetimeV1,
        LicenseTypeCode::UiaiOperatorLifetimeV1,
    ];
    for code in operator_codes {
        let serialized = serde_json::to_string(&code).expect("license type code");
        assert!(
            serialized.contains("operator_lifetime_v1"),
            "{serialized} must be an Operator License Type, not Evaluation"
        );
        assert!(!serialized.contains("evaluation"));
    }

    // Operator v1 freezes ten capability families; the verified-no-license
    // allowlist is six explicit manual families and the blocked set is exactly
    // the four optional premium families.
    assert_eq!(SPEC172_FOCUSA_OPERATOR_V1_FAMILIES.len(), 10);
    assert_eq!(SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES.len(), 6);
    assert_eq!(SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES.len(), 4);
    for blocked in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        let family = parse_family(blocked);
        assert!(
            family.is_optional_premium(),
            "{blocked} must be one of the four optional premium families"
        );
        // Resolver agreement: every blocked premium family is denied in the
        // verified_no_license posture.
        assert_eq!(
            reduce_entitlement_state(State::VerifiedNoLicense, family, None).posture(),
            Posture::Deny,
            "{blocked} must be denied for verified_no_license"
        );
    }

    // verified_no_license limited access is allowlist-driven and exactly one
    // manual project may be mutated: the core project guard enforces the
    // one-project subset on the resolver's Limited decision.
    let limited = resolve_base_focusa_product("focusa", State::VerifiedNoLicense);
    assert_eq!(limited, BaseProductDecision::Limited);
    let active_selection = ActiveProjectSelection::new("/synthetic/one", "acceptance");
    assert!(ActiveProjectGuard::check_mutation(limited, "/synthetic/one", Some(&active_selection)).is_allowed());
    assert!(
        ActiveProjectGuard::check_mutation(limited, "/synthetic/two", Some(&active_selection)).is_denied()
    );
    assert!(ActiveProjectGuard::check_mutation(limited, "/synthetic/one", None).is_denied());

    // Evaluation has no signed entitlement and never satisfies the base gate
    // at the resolver, core, or API surface.
    let eval = LicenseGuard::eval(7);
    assert!(eval.entitlement.is_none());
    assert_eq!(
        authority_policy_state_absent(&eval),
        State::MissingOrCorrupt
    );
    assert_eq!(
        resolve_base_focusa_product("focusa", authority_policy_state_absent(&eval)),
        BaseProductDecision::Denied
    );
    let failure = evaluate_entitlement_execution(
        &eval,
        &policy_for(Family::BaseFocusa),
        EntitlementExecutionContext::default(),
    )
    .expect_err("self-issued Evaluation must fail the core guard");
    assert_eq!(failure.code, "ENTITLEMENT_BASE_REQUIRED");
    assert_eq!(
        route_entitlement_denial(&eval, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0)
            .expect("Evaluation must be denied at the API")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );
}

/// `authority_policy_state` for a guard with no signed snapshot: the guard
/// path maps the absence to MissingOrCorrupt.
fn authority_policy_state_absent(guard: &LicenseGuard) -> State {
    guard
        .entitlement
        .as_ref()
        .map(authority_policy_state)
        .unwrap_or(State::MissingOrCorrupt)
}
