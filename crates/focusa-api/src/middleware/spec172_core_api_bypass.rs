//! Spec 172 §20.9 — core/API chokepoint and direct-call bypass resistance
//! vectors at the API route-gate layer (atom focusa-vbcqu.20.15.25, 172.04.01).
//!
//! Replayable acceptance module compiled only under `cargo test`, reached by
//! the exact verification filter `cargo test --workspace spec172_core_api_bypass`.
//!
//! This seat proves that the HTTP route gate and the focusa-core shared
//! chokepoint apply the same product/type/family decisions before any side
//! effect: stale clients (expired/revoked/unbound leases) are denied before
//! handler execution, wrong-method calls fail closed, direct calls cannot skip
//! the middleware by calling core or reducer code directly, and
//! recovery/read/export surfaces stay reachable in every blocked state.
//!
//! No raw email, key, token, customer row, credential, or card data appears in
//! this module; all snapshots are synthetic acceptance fixtures.

use super::*;
use focusa_core::entitlement_execution_guard::{
    EntitlementExecutionContext, EntitlementExecutionPolicy,
};
use focusa_core::guarded_mutation::guard_value_mutation;
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{CapabilityFamily as Family, LicenseGuard, OperationClass, RecoveryAllowance};

fn base_mutation_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "rest.v1.workpoint.checkpoint.post",
        OperationClass::ValueMutation,
        Family::BaseFocusa,
        None,
        None,
        RecoveryAllowance::None,
    )
}

fn stale_snapshots() -> Vec<(&'static str, EntitlementSnapshot)> {
    let mut expired = EntitlementSnapshot::unactivated("focusa", "node-api-bypass");
    expired.state = EntitlementState::Active;
    expired.lease_id = Some("lease-api-bypass".into());
    expired.lease_digest = Some("sha256:api-bypass".into());
    expired.sequence = Some(7);
    expired.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

    let mut unbound = EntitlementSnapshot::unactivated("focusa", "node-api-bypass");
    unbound.state = EntitlementState::Active;
    unbound.sequence = Some(7);
    unbound.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));

    vec![
        ("expired", expired),
        ("unbound", unbound),
        (
            "revoked",
            EntitlementSnapshot::recovery_only("focusa", "node-api-bypass", "revoked"),
        ),
    ]
}

// ── 1. Stale clients fail before any handler side effect ──────────────────

#[test]
fn spec172_core_api_bypass_stale_client_denied_before_handler() {
    // An expired, unbound, or revoked lease must never reach a value-producing
    // handler, even though the policy state grid alone would allow an Active
    // row. The API route gate checks lease currency; the core chokepoint must
    // agree for non-HTTP direct callers.
    for (label, snapshot) in stale_snapshots() {
        let guard = LicenseGuard::from_entitlement(snapshot);
        let denial = route_entitlement_denial(&guard, &Method::POST, "/v1/workpoint/checkpoint")
            .unwrap_or_else(|| panic!("stale client ({label}) must be denied by the API gate"));
        assert_eq!(
            denial.code, "ENTITLEMENT_BASE_REQUIRED",
            "stale client ({label}) must emit the base-required code"
        );

        let failure = guard_value_mutation(
            &guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
        )
        .expect_err("stale client ({label}) must be denied by the core chokepoint");
        assert_eq!(
            failure.code, "ENTITLEMENT_BASE_REQUIRED",
            "stale client ({label}) core chokepoint must emit the same code"
        );
    }
}

// ── 2. Wrong-method and unclassified mutation attempts fail closed ────────

#[test]
fn spec172_core_api_bypass_wrong_method_and_unclassified_fail_closed() {
    // A mutation attempt on a read-classified route via the wrong method is
    // not a sanctioned read: it must require entitlement and fail closed
    // before any side effect rather than slip through a read exemption.
    assert!(
        route_requires_entitlement(&Method::POST, "/v1/workpoint/current"),
        "wrong-method mutation attempt must require entitlement"
    );
    let denial = route_entitlement_denial(
        &LicenseGuard::eval(7),
        &Method::POST,
        "/v1/workpoint/current",
    )
    .expect("wrong-method mutation attempt must be denied before side effects");
    assert_eq!(
        denial.code, "ENTITLEMENT_BASE_REQUIRED",
        "wrong-method mutation attempt resolves as a base mutation and is denied"
    );

    // A premium-family read route cannot be exempted by its declared method
    // alone: without a usable signed lease it is denied (premium state is not
    // readable anonymously).
    let premium_read =
        route_entitlement_denial(&LicenseGuard::eval(7), &Method::GET, "/v1/connect/rooms")
            .expect("premium read without a lease must be denied");
    assert_eq!(premium_read.code, "ENTITLEMENT_BASE_REQUIRED");

    // An entirely unclassified mutation route has no entitlement descriptor
    // and is blocked fail-closed.
    let unclassified = route_entitlement_denial(
        &LicenseGuard::eval(7),
        &Method::POST,
        "/v1/unclassified/mutation",
    )
    .expect("unclassified mutation must be denied");
    assert_eq!(unclassified.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");

    // The core chokepoint agrees for the canonical mutation operation: a
    // self-issued eval guard never satisfies the base gate.
    let failure = guard_value_mutation(
        &LicenseGuard::eval(7),
        &base_mutation_policy(),
        EntitlementExecutionContext::default(),
    )
    .expect_err("self-issued eval must never satisfy the base gate at the chokepoint");
    assert_eq!(failure.code, "ENTITLEMENT_BASE_REQUIRED");
}

// ── 3. Direct calls cannot skip the middleware decision ───────────────────

#[test]
fn spec172_core_api_bypass_direct_call_cannot_skip_middleware() {
    // A wrong-product signed lease (UIAI-only) must not execute Focusa
    // mutations through the API gate, and the core chokepoint must deny the
    // same operation for a non-HTTP caller — no presenter-owned policy.
    let mut uiai = EntitlementSnapshot::unactivated("uiai_engine", "node-api-bypass");
    uiai.state = EntitlementState::Active;
    uiai.lease_id = Some("lease-api-bypass".into());
    uiai.lease_digest = Some("sha256:api-bypass".into());
    uiai.sequence = Some(7);
    uiai.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    let uiai_guard = LicenseGuard::from_entitlement(uiai);

    let api_denial =
        route_entitlement_denial(&uiai_guard, &Method::POST, "/v1/workpoint/checkpoint")
            .expect("UIAI-only entitlement must not execute Focusa mutations via the API");
    assert_eq!(api_denial.code, "ENTITLEMENT_BASE_REQUIRED");

    let core_failure = guard_value_mutation(
        &uiai_guard,
        &base_mutation_policy(),
        EntitlementExecutionContext::default(),
    )
    .expect_err("UIAI-only entitlement must not execute Focusa mutations via direct core calls");
    assert_eq!(core_failure.code, "ENTITLEMENT_BASE_REQUIRED");

    // With the exact Focusa signed lease both gates approve the same mutation.
    let mut focusa = EntitlementSnapshot::unactivated("focusa", "node-api-bypass");
    focusa.state = EntitlementState::Active;
    focusa.lease_id = Some("lease-api-bypass".into());
    focusa.lease_digest = Some("sha256:api-bypass".into());
    focusa.sequence = Some(7);
    focusa.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    let focusa_guard = LicenseGuard::from_entitlement(focusa);

    assert_eq!(
        route_entitlement_denial(&focusa_guard, &Method::POST, "/v1/workpoint/checkpoint"),
        None,
        "exact Focusa lease must pass the API gate"
    );
    assert!(
        guard_value_mutation(
            &focusa_guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
        )
        .is_ok(),
        "exact Focusa lease must pass the core chokepoint"
    );
}

// ── 4. Recovery/read/export stay reachable in every blocked state ─────────

#[test]
fn spec172_core_api_bypass_recovery_and_read_remain_reachable_when_blocked() {
    // In every blocked state (missing, unactivated, refunded/revoked), the
    // API recovery routes pass the gate and the core chokepoint keeps
    // recovery/export/read operations reachable, so customer data is never
    // trapped or deleted by a denial.
    // Every blocked state maps to a policy state; only the families the
    // resolver keeps reachable may pass the chokepoint, and at least one
    // data-protection surface stays reachable in every blocked state.
    let blocked: Vec<(&str, LicenseGuard, focusa_license::PolicyEntitlementState)> = vec![
        (
            "missing_or_corrupt",
            LicenseGuard::eval(7),
            focusa_license::PolicyEntitlementState::MissingOrCorrupt,
        ),
        (
            "unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated(
                "focusa",
                "node-api-bypass",
            )),
            focusa_license::PolicyEntitlementState::PendingUnverified,
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-api-bypass",
                "revoked",
            )),
            focusa_license::PolicyEntitlementState::RefundedOrRevoked,
        ),
    ];

    let recovery_routes: [(&str, Method, &str); 5] = [
        ("stable_update", Method::POST, "/v1/update/apply"),
        ("repair", Method::POST, "/v1/project/bootstrap/repair"),
        ("rollback", Method::POST, "/v1/update/rollback"),
        ("export_run", Method::POST, "/v1/export/run"),
        ("diagnostics", Method::GET, "/v1/doctor"),
    ];

    for (state_label, guard, policy_state) in &blocked {
        for (route_label, method, path) in &recovery_routes {
            let denial = route_entitlement_denial(guard, method, path);
            assert!(
                denial.is_none(),
                "{route_label} ({path}) must remain available in state {state_label}, got: {denial:?}"
            );
        }

        let mut reachable_in_state = 0u32;
        for (family, operation_class, recovery_allowance) in [
            (
                Family::AccountRecovery,
                OperationClass::Recovery,
                RecoveryAllowance::AccountRecovery,
            ),
            (
                Family::CustomerDataExport,
                OperationClass::Read,
                RecoveryAllowance::CustomerDataExport,
            ),
            (
                Family::ReadProjection,
                OperationClass::Read,
                RecoveryAllowance::ReadProjection,
            ),
        ] {
            if focusa_license::reduce_entitlement_state(*policy_state, family, None).posture()
                == focusa_license::EntitlementPolicyPosture::Deny
            {
                continue; // resolver keeps this family blocked in this state
            }
            let policy = EntitlementExecutionPolicy::new(
                format!("bypass.matrix.{}", family.label()),
                operation_class,
                family,
                None,
                None,
                recovery_allowance,
            );
            assert!(
                guard_value_mutation(guard, &policy, EntitlementExecutionContext::default())
                    .is_ok(),
                "state {state_label}: {family:?} must stay reachable through the chokepoint"
            );
            reachable_in_state += 1;
        }
        assert!(
            reachable_in_state >= 1,
            "state {state_label}: at least one data-protection surface must stay reachable"
        );

        // Protected mutations stay denied in the same blocked states.
        let denial = route_entitlement_denial(guard, &Method::POST, "/v1/workpoint/checkpoint")
            .expect("protected mutation must be denied in state {state_label}");
        assert_eq!(denial.code, "ENTITLEMENT_BASE_REQUIRED");
    }
}
