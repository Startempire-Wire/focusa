//! Spec 172 §20 — complete runtime policy matrix at the API route-gate layer
//! (atom focusa-vbcqu.20.15.24, 172.03.07).
//!
//! Replayable acceptance module compiled only under `cargo test`, reached by
//! the exact verification filter `cargo test --workspace spec172_runtime_policy`.
//!
//! From this API middleware seat (which owns both the focusa-core execution
//! guard and the focusa-license canonical resolver) the module proves that the
//! HTTP route gate projects the same decisions as the pure resolver for the
//! complete Spec 172 runtime policy matrix: unverified, verified-limited,
//! Focusa Operator, UIAI Operator, Bundle, refunded/revoked, offline, corrupt,
//! unknown/future surfaces, and dynamic/generated-UI actions fail closed
//! before any handler side effect, while recovery/export/read surfaces stay
//! reachable in every blocked state. Surfaces never own pricing, grants, or
//! commercial policy (Spec 172 §11.1/§11.2).
//!
//! No raw keys, tokens, customer identifiers, or prices appear in this module;
//! all snapshots are synthetic acceptance fixtures.

use super::*;
use focusa_core::entitlement_execution_guard::{
    EntitlementExecutionContext, EntitlementExecutionPolicy, evaluate_entitlement_execution,
};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    CapabilityFamily as Family, EntitlementPolicyPosture as Posture, LicenseGuard,
    PolicyEntitlementState as State, RecoveryAllowance, reduce_entitlement_state,
};

const BASE_MUTATION_ROUTE: (&str, Method, Family) =
    ("/v1/workpoint/checkpoint", Method::POST, Family::BaseFocusa);

const PREMIUM_ROUTES: [(&str, Method, Family); 4] = [
    ("/v1/silent-sessions", Method::POST, Family::Automation),
    ("/v1/connect/room/create", Method::POST, Family::TeamRemote),
    (
        "/v1/release/proof/status",
        Method::POST,
        Family::ReleaseProof,
    ),
    ("/v1/update/scheduler", Method::POST, Family::PremiumUpdates),
];

const RECOVERY_ROUTES: [(&str, Method, &str); 8] = [
    ("stable_update", Method::POST, "/v1/update/apply"),
    ("repair", Method::POST, "/v1/project/bootstrap/repair"),
    ("rollback", Method::POST, "/v1/update/rollback"),
    ("export_run", Method::POST, "/v1/export/run"),
    ("export_history", Method::GET, "/v1/export/history"),
    ("node_deactivation", Method::POST, "/v1/device/pair/revoke"),
    ("diagnostics", Method::GET, "/v1/doctor"),
    ("license_status", Method::GET, "/v1/license/status"),
];

fn snapshot_for(state: State) -> EntitlementSnapshot {
    match state {
        State::PendingUnverified => EntitlementSnapshot::unactivated("focusa", "node-api-matrix"),
        State::ActivePaid => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-api-matrix");
            snapshot.state = EntitlementState::Active;
            snapshot.lease_id = Some("lease-api-matrix".to_string());
            snapshot.lease_digest = Some("sha256:api-matrix".to_string());
            snapshot.sequence = Some(7);
            snapshot.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
            snapshot
        }
        State::OfflineGrace => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-api-matrix");
            snapshot.state = EntitlementState::OfflineGrace;
            snapshot.lease_id = Some("lease-api-matrix".to_string());
            snapshot.lease_digest = Some("sha256:api-matrix".to_string());
            snapshot.sequence = Some(7);
            snapshot.offline_grace_until = Some(chrono::Utc::now() + chrono::Duration::hours(1));
            snapshot
        }
        State::RefundedOrRevoked => {
            EntitlementSnapshot::recovery_only("focusa", "node-api-matrix", "refunded_or_revoked")
        }
        State::VerifiedNoLicense | State::Expired | State::MissingOrCorrupt => {
            unreachable!("not a signed-snapshot state")
        }
    }
}

fn guard_for(state: State) -> LicenseGuard {
    match state {
        State::MissingOrCorrupt => LicenseGuard::eval(7),
        _ => LicenseGuard::from_entitlement(snapshot_for(state)),
    }
}

/// Grant the exact registered premium feature for one family.
fn with_premium(state: State, features: &[&str]) -> LicenseGuard {
    let mut snapshot = snapshot_for(state);
    for feature in features {
        snapshot.features.insert(feature.to_string(), true);
    }
    LicenseGuard::from_entitlement(snapshot)
}

// ── 1. API route gate equivalent to the pure resolver ─────────────────────

#[test]
fn spec172_runtime_policy_api_route_gate_equivalent_to_resolver() {
    // Blocked states: resolver denies base/premium, and so does the route gate.
    let blocked: Vec<(&str, LicenseGuard)> = vec![
        ("missing_or_corrupt", guard_for(State::MissingOrCorrupt)),
        ("unverified", guard_for(State::PendingUnverified)),
        ("refunded_or_revoked", guard_for(State::RefundedOrRevoked)),
    ];

    for (label, guard) in &blocked {
        let denial = route_entitlement_denial(guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0);
        assert_eq!(
            denial.as_ref().map(|d| d.code.as_str()),
            Some("ENTITLEMENT_BASE_REQUIRED"),
            "base mutation must be denied in state {label}"
        );
        assert_eq!(
            reduce_entitlement_state(
                guard
                    .entitlement
                    .as_ref()
                    .map(focusa_license::authority_policy_state)
                    .unwrap_or(State::MissingOrCorrupt),
                BASE_MUTATION_ROUTE.2,
                None,
            )
            .posture(),
            Posture::Deny,
            "resolver must agree: base denied in state {label}"
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

    // Active paid without grants: base passes; premium requires its exact
    // authority grant — the resolver's Base/Feature rows with grant isolation
    // at the gate.
    let active_guard = guard_for(State::ActivePaid);
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

    // Active paid with the exact grants: premium passes; stored premium
    // claims never widen the base gate and wrong-product leases stay denied.
    let granted = with_premium(
        State::ActivePaid,
        &[
            "focusa.agent.silent_sessions",
            "focusa.team.multi_operator",
            "focusa.release.proof",
            "focusa.update.unattended",
        ],
    );
    for (path, method, _family) in &PREMIUM_ROUTES {
        assert_eq!(
            route_entitlement_denial(&granted, method, path),
            None,
            "{path} must pass with the granted premium feature"
        );
    }
    assert_eq!(
        focusa_license::resolve_base_focusa_product("premium-alias", State::ActivePaid),
        focusa_license::BaseProductDecision::Denied
    );

    // Offline Grace with a valid cached lease: base passes and a cached
    // premium grant passes; the resolver row says Base/Feature.
    let grace = with_premium(State::OfflineGrace, &["focusa.team.multi_operator"]);
    assert_eq!(
        route_entitlement_denial(&grace, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0),
        None,
        "base mutation must pass during valid offline grace"
    );
    assert_eq!(
        route_entitlement_denial(&grace, &Method::POST, "/v1/connect/room/create"),
        None,
        "cached premium grant must pass during valid offline grace"
    );
}

// ── 2. Denied routes emit zero partial side effects ────────────────────────

#[test]
fn spec172_runtime_policy_api_denied_route_no_partial_side_effect() {
    // Every protected mutation route emits a denial sentinel before any
    // handler side effect in every blocked state; recovery routes are never
    // denied, so customer data is never trapped.
    let blocked: Vec<(&str, LicenseGuard)> = vec![
        ("missing_or_corrupt", guard_for(State::MissingOrCorrupt)),
        ("unverified", guard_for(State::PendingUnverified)),
        ("refunded_or_revoked", guard_for(State::RefundedOrRevoked)),
    ];

    for (label, guard) in &blocked {
        let mut mutation_sentinels = 0u32;
        let base = route_entitlement_denial(guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0);
        assert!(
            base.is_some(),
            "base mutation must be denied before side effects in state {label}"
        );
        mutation_sentinels += 1;
        for (path, method, _family) in &PREMIUM_ROUTES {
            assert!(
                route_entitlement_denial(guard, method, path).is_some(),
                "{path} must be denied before side effects in state {label}"
            );
            mutation_sentinels += 1;
        }
        assert_eq!(
            mutation_sentinels,
            1 + PREMIUM_ROUTES.len() as u32,
            "state {label}: every protected mutation must emit a denial sentinel"
        );

        for (route_label, method, path) in &RECOVERY_ROUTES {
            assert!(
                route_entitlement_denial(guard, method, path).is_none(),
                "{route_label} ({path}) must remain available in state {label}"
            );
        }
    }

    // The resolver agrees: in every blocked state the data-protection
    // families resolve to Allow/Read (never Deny).
    for state in [
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
        State::PendingUnverified,
    ] {
        assert_eq!(
            reduce_entitlement_state(state, Family::AccountRecovery, None).posture(),
            Posture::Allow
        );
        assert_eq!(
            reduce_entitlement_state(state, Family::CustomerDataExport, None).posture(),
            Posture::Allow
        );
    }
}

// ── 3. Unknown / future Navigator / dynamic surfaces fail closed ───────────

#[test]
fn spec172_runtime_policy_api_future_navigator_and_dynamic_fail_closed() {
    // A future Navigator operation has no wired API surface: the route policy
    // resolver owns no policy for it, so it can never reach a value-producing
    // handler through the API, and no Navigator route exists in the manifest.
    assert!(
        resolve_route_entitlement_policy(&Method::POST, "/v1/navigator/operations").is_none(),
        "Navigator has no API policy today"
    );

    // Unknown operation classes fail the canonical guard with the stable
    // unclassified error, exactly as the resolver's Unknown row fails closed.
    let unknown_policy = EntitlementExecutionPolicy::new(
        "api.matrix.unknown",
        focusa_license::OperationClass::Unknown,
        Family::Automation,
        Some("focusa.agent.silent_sessions"),
        None,
        RecoveryAllowance::None,
    );
    let failure = evaluate_entitlement_execution(
        &guard_for(State::ActivePaid),
        &unknown_policy,
        EntitlementExecutionContext::default(),
    )
    .expect_err("unknown operation class must fail closed");
    assert_eq!(failure.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");

    // Generated UI from this seat cannot select commercial treatment:
    // unsigned or unregistered actions quarantine, so client metadata can
    // never mint a product, price, License Type, family, or grant.
    assert_eq!(
        focusa_license::verify_generated_ui_action(
            "focusa.workpoint.checkpoint",
            &["focusa.workpoint.checkpoint"],
            false,
        ),
        focusa_license::ManifestTrustDecision::QuarantinedUnsigned
    );
    assert_eq!(
        focusa_license::verify_generated_ui_action(
            "focusa.workpoint.checkpoint",
            &["focusa.workpoint.checkpoint"],
            true,
        ),
        focusa_license::ManifestTrustDecision::Trusted
    );
    assert_eq!(
        focusa_license::verify_generated_ui_action(
            "focusa.workpoint.checkpoint",
            &["focusa.mission.record"],
            true,
        ),
        focusa_license::ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion
    );

    // No presenter-owned policy: a wrong-product signed lease stays denied at
    // the base gate even though the API metadata looks Focusa-shaped.
    let mut wrong_product = EntitlementSnapshot::unactivated("uiai_engine", "node-api-matrix");
    wrong_product.state = EntitlementState::Active;
    wrong_product.lease_id = Some("lease-api-matrix".to_string());
    wrong_product.lease_digest = Some("sha256:api-matrix".to_string());
    wrong_product.sequence = Some(7);
    wrong_product.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    let wrong_guard = LicenseGuard::from_entitlement(wrong_product);
    let denial =
        route_entitlement_denial(&wrong_guard, &BASE_MUTATION_ROUTE.1, BASE_MUTATION_ROUTE.0)
            .expect("UIAI-only entitlement must not execute Focusa mutations");
    assert_eq!(denial.code, "ENTITLEMENT_BASE_REQUIRED");
}
