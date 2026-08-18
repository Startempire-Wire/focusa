//! Spec 172 §20 — complete runtime policy matrix at the core execution-guard
//! layer (atom focusa-vbcqu.20.15.24, 172.03.07).
//!
//! The focusa-core execution guard is the shared chokepoint for every
//! value-producing mutation (Spec 172 §11.4: no direct-core bypass). This
//! module proves, for the guard-representable policy states, that the guard's
//! decisions are equivalent to the pure resolver
//! (`crates/focusa-license/tests/spec172_runtime_policy.rs`), that denied
//! paths produce zero partial side effects, that recovery/read/export stay
//! reachable in every blocked state, and that unknown/future/dynamic surfaces
//! fail closed. The API route gate
//! (`crates/focusa-api/src/middleware/spec172_runtime_policy.rs`) projects the
//! same decisions.
//!
//! Exact verification: `cargo test --workspace spec172_runtime_policy`.

use chrono::{Duration, Utc};
use focusa_core::license::{
    EntitlementExecutionContext, EntitlementExecutionPolicy, evaluate_entitlement_execution,
    evaluate_entitlement_execution_for_project,
};
use focusa_core::limited_project::{ActiveProjectGuard, ActiveProjectSelection};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    BaseProductDecision, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, LicenseGuard, OperationClass,
    PolicyEntitlementState as State, RecoveryAllowance, authority_policy_state,
    base_product_projection, premium_family_feature_ids, reduce_entitlement_state,
    resolve_base_focusa_product,
};

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

fn now() -> chrono::DateTime<Utc> {
    Utc::now()
}

/// Guard-representable policy states. `VerifiedNoLicense` and `Expired` are
/// not signed-snapshot states (the former is an assertion-carried posture, the
/// latter an expired-lease boundary), so they are not constructed here.
fn snapshot_for(state: State) -> EntitlementSnapshot {
    match state {
        State::PendingUnverified => EntitlementSnapshot::unactivated("focusa", "node-matrix"),
        State::ActivePaid => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-matrix");
            snapshot.state = EntitlementState::Active;
            snapshot.lease_id = Some("lease-matrix".to_string());
            snapshot.lease_digest = Some("sha256:matrix".to_string());
            snapshot.sequence = Some(7);
            snapshot.expires_at = Some(now() + Duration::hours(1));
            snapshot
        }
        State::OfflineGrace => {
            let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-matrix");
            snapshot.state = EntitlementState::OfflineGrace;
            snapshot.lease_id = Some("lease-matrix".to_string());
            snapshot.lease_digest = Some("sha256:matrix".to_string());
            snapshot.sequence = Some(7);
            snapshot.offline_grace_until = Some(now() + Duration::hours(1));
            snapshot
        }
        State::RefundedOrRevoked => {
            EntitlementSnapshot::recovery_only("focusa", "node-matrix", "refunded_or_revoked")
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

/// Grant the registered premium feature so a Feature decision is reachable
/// and must equal the resolver's Feature row.
fn guard_with_premium_grant(state: State, family: Family) -> LicenseGuard {
    let mut snapshot = snapshot_for(state);
    for feature in premium_family_feature_ids(family) {
        snapshot.features.insert(feature.to_string(), true);
    }
    LicenseGuard::from_entitlement(snapshot)
}

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
        format!("matrix.{}", family.label()),
        operation_class,
        family,
        feature,
        None,
        RecoveryAllowance::None,
    )
}

fn expected_denial_code(reason: Reason, family: Family) -> &'static str {
    if reason == Reason::MissingInitiatingPolicy {
        "ENTITLEMENT_ROUTE_UNCLASSIFIED"
    } else if family == Family::BaseFocusa {
        "ENTITLEMENT_BASE_REQUIRED"
    } else {
        "ENTITLEMENT_REQUIRED"
    }
}

// ── 1. Core guard decisions are equivalent to the pure resolver ────────────

#[test]
fn spec172_runtime_policy_core_guard_decisions_equivalent_to_resolver() {
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

            let guard = if family.is_optional_premium()
                && matches!(state, State::ActivePaid | State::OfflineGrace)
            {
                guard_with_premium_grant(state, family)
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
                    assert_eq!(
                        decision.reason_code,
                        resolver.reason().label(),
                        "{state:?}/{family:?}"
                    );
                }
                Posture::Base => {
                    let decision =
                        outcome.expect("base posture must pass with a usable signed lease");
                    assert_eq!(decision.status, "base", "{state:?}/{family:?}");
                    assert_eq!(
                        decision.reason_code,
                        resolver.reason().label(),
                        "{state:?}/{family:?}"
                    );
                }
                Posture::Feature => {
                    let decision =
                        outcome.expect("granted premium feature must pass the core guard");
                    assert_eq!(decision.status, "feature", "{state:?}/{family:?}");
                    assert_eq!(
                        decision.reason_code,
                        resolver.reason().label(),
                        "{state:?}/{family:?}"
                    );
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

// ── 2. Zero denied path produces protected partial side effects ────────────

/// A minimal guarded mutation ledger: a value-producing mutation only writes
/// after the core guard approves. Denied operations must never reach the
/// write, while recovery/read/export operations remain reachable so customer
/// data is never trapped or deleted.
#[derive(Default)]
struct MutationLedger {
    writes: Vec<String>,
}

impl MutationLedger {
    fn guarded_mutation(
        &mut self,
        guard: &LicenseGuard,
        policy: &EntitlementExecutionPolicy,
        write_label: &str,
    ) -> Result<String, String> {
        match evaluate_entitlement_execution(guard, policy, EntitlementExecutionContext::default())
        {
            Ok(_) => {
                self.writes.push(write_label.to_string());
                Ok(write_label.to_string())
            }
            Err(failure) => Err(failure.code),
        }
    }
}

#[test]
fn spec172_runtime_policy_denied_paths_have_zero_partial_side_effects() {
    let guard_states = [
        State::PendingUnverified,
        State::ActivePaid,
        State::OfflineGrace,
        State::RefundedOrRevoked,
        State::MissingOrCorrupt,
    ];

    let mut ledger = MutationLedger::default();
    let mut denied_cells = 0u32;
    let mut recovery_reachable_in_blocked = 0u32;

    for state in guard_states {
        for family in FAMILIES {
            if family == Family::InternalMaintenance {
                continue;
            }
            let resolver = reduce_entitlement_state(state, family, None);
            let guard = if family.is_optional_premium()
                && matches!(state, State::ActivePaid | State::OfflineGrace)
            {
                guard_with_premium_grant(state, family)
            } else {
                guard_for(state)
            };
            let label = format!("{:?}/{:?}", state, family);

            if resolver.posture() == Posture::Deny {
                denied_cells += 1;
                let before = ledger.writes.len();
                let result = ledger.guarded_mutation(&guard, &policy_for(family), &label);
                assert!(result.is_err(), "denied cell must refuse: {label}");
                assert_eq!(
                    ledger.writes.len(),
                    before,
                    "denied cell wrote a partial side effect: {label}"
                );

                // In the same blocked state the data-protection surfaces that
                // the resolver keeps reachable (recovery, basic export, read
                // projection) stay reachable through the guard — customer
                // data is never trapped or deleted by a denial.
                let mut reachable_in_state = 0u32;
                for reachable in [
                    Family::AccountRecovery,
                    Family::CustomerDataExport,
                    Family::ReadProjection,
                ] {
                    if reduce_entitlement_state(state, reachable, None).posture() == Posture::Deny {
                        continue; // surface itself denied in this state
                    }
                    let ok = evaluate_entitlement_execution(
                        &guard,
                        &policy_for(reachable),
                        EntitlementExecutionContext::default(),
                    )
                    .is_ok();
                    assert!(
                        ok,
                        "resolver-allowed protection surface must pass the guard in {label}"
                    );
                    reachable_in_state += 1;
                }
                assert!(
                    reachable_in_state >= 1,
                    "every blocked state keeps at least one protection surface: {label}"
                );
                recovery_reachable_in_blocked += 1;
            } else {
                // Non-denied cells either mutate (after the guard approves) or
                // are read/export/recovery surfaces that never write project
                // data; assert the guard's own decision is deterministic.
                let first = ledger.guarded_mutation(&guard, &policy_for(family), &label);
                let second = evaluate_entitlement_execution(
                    &guard,
                    &policy_for(family),
                    EntitlementExecutionContext::default(),
                );
                assert_eq!(
                    first.is_ok(),
                    second.is_ok(),
                    "guard decision must be deterministic for {label}"
                );
            }
        }
    }

    assert!(denied_cells > 0, "the matrix must contain denied cells");
    assert_eq!(
        recovery_reachable_in_blocked, denied_cells,
        "every denied cell keeps recovery/export/read reachable"
    );

    // The internal-maintenance boundary also fails closed without an initiator
    // and writes nothing.
    let before = ledger.writes.len();
    let maintenance = ledger.guarded_mutation(
        &guard_for(State::ActivePaid),
        &policy_for(Family::InternalMaintenance),
        "matrix/maintenance",
    );
    assert_eq!(
        maintenance,
        Err("ENTITLEMENT_ROUTE_UNCLASSIFIED".to_string())
    );
    assert_eq!(ledger.writes.len(), before, "maintenance wrote nothing");
}

// ── 3. Verified-limited one-project boundary at the core layer ─────────────

#[test]
fn spec172_runtime_policy_core_guard_verified_limited_project_boundary() {
    // The signed limited-access assertion carries the VerifiedNoLicense
    // posture; at the core layer the active-project guard enforces the
    // one-mutable-project rule exactly when the base decision is Limited.
    let selected = ActiveProjectSelection::new("/home/user/projects/focusa-a", "test");

    let allowed = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        "/home/user/projects/focusa-a",
        Some(&selected),
    );
    assert!(allowed.is_allowed(), "active project mutation is allowed");

    let second = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        "/home/user/projects/focusa-b",
        Some(&selected),
    );
    assert!(second.is_denied(), "second project mutation is denied");
    match second {
        focusa_core::limited_project::ProjectMutationDecision::DeniedSecondProject {
            active_project_root,
            attempted_project_root,
            ..
        } => {
            assert_eq!(active_project_root, "/home/user/projects/focusa-a");
            assert_eq!(attempted_project_root, "/home/user/projects/focusa-b");
        }
        _ => panic!("expected DeniedSecondProject"),
    }

    let no_selection = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        "/home/user/projects/any-project",
        None,
    );
    assert!(no_selection.is_denied(), "no explicit selection must deny");
    assert!(
        !no_selection.recovery_action().is_empty(),
        "denial must carry a recovery action"
    );

    // Paid entitlement bypasses the project guard (the base gate governs).
    assert!(
        ActiveProjectGuard::check_mutation(
            BaseProductDecision::Entitled,
            "/home/user/projects/any-project",
            None,
        )
        .is_allowed()
    );

    // The execution-guard integration accepts the explicit project decision:
    // a paid signed lease never triggers the project guard and returns the
    // resolver-equivalent base decision.
    let guard = guard_for(State::ActivePaid);
    let decision = evaluate_entitlement_execution_for_project(
        &guard,
        &policy_for(Family::BaseFocusa),
        EntitlementExecutionContext::default(),
        "/home/user/projects/focusa-a",
        Some(&selected),
    )
    .expect("paid base mutation passes the project-aware guard");
    assert_eq!(decision.status, "base");
    assert_eq!(decision.reason_code, Reason::RequireBase.label());
}

// ── 4. Unknown / future / dynamic surfaces fail closed at the guard ────────

#[test]
fn spec172_runtime_policy_core_guard_unknown_future_and_dynamic_fail_closed() {
    // Unknown operation class fails before any state evaluation.
    let unknown_policy = EntitlementExecutionPolicy::new(
        "matrix.unknown_operation",
        OperationClass::Unknown,
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

    // A materially new (future) family is not an expressible CapabilityFamily:
    // serde parsing fails, so no operation can carry it into the guard.
    assert!(serde_json::from_str::<Family>("\"future_capability_family\"").is_err());

    // A future product cannot satisfy the base gate.
    assert_eq!(
        resolve_base_focusa_product("future_product", State::ActivePaid),
        BaseProductDecision::Denied
    );
    assert_eq!(
        resolve_base_focusa_product("focusa", State::RefundedOrRevoked),
        BaseProductDecision::Denied
    );

    // Without a signed entitlement the base gate fails closed even when the
    // caller asks for a legacy core identifier.
    let eval_guard = LicenseGuard::eval(7);
    assert!(eval_guard.entitlement.is_none());
    assert!(base_product_projection(eval_guard.entitlement.as_ref()).is_err());
    let denied = evaluate_entitlement_execution(
        &eval_guard,
        &policy_for(Family::BaseFocusa),
        EntitlementExecutionContext::default(),
    )
    .expect_err("self-issued eval must never satisfy the base gate");
    assert_eq!(denied.code, "ENTITLEMENT_BASE_REQUIRED");

    // RecoveryOnly snapshot: base denied, recovery/export/read still usable.
    let revoked = guard_for(State::RefundedOrRevoked);
    assert_eq!(
        authority_policy_state(revoked.entitlement.as_ref().unwrap()),
        State::RefundedOrRevoked
    );
    assert!(
        !base_product_projection(revoked.entitlement.as_ref())
            .expect("projection")
            .permits_base_mutations
    );
    for reachable in [
        Family::AccountRecovery,
        Family::CustomerDataExport,
        Family::ReadProjection,
    ] {
        assert!(
            evaluate_entitlement_execution(
                &revoked,
                &policy_for(reachable),
                EntitlementExecutionContext::default(),
            )
            .is_ok(),
            "data-protection surface must stay reachable after revoke: {reachable:?}"
        );
    }
}
