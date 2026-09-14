//! Spec 152F.06.06 — entitlement resolver, middleware-path, and policy registry
//! benchmark.
//!
//! `cargo bench -p focusa-license entitlement_policy`
//!
//! Every scenario is deterministic, bounded, in-memory, and secret-free: no
//! file parsing, no network, no random input, and no customer data. Each
//! scenario prints a ns/op receipt that feeds the performance budget recorded
//! in `docs/evidence/spec152f/focusa-vbcqu.20.14.48-acceptance.txt`.
//!
//! Covered paths:
//! - hot cached decision: the per-request state-grid reduction,
//! - cold validation: full typed `ResolvedEntitlementPolicy` invariant checks,
//! - premium + limit decision: registered premium feature plus signed limit
//!   reservation (the API middleware/scheduler revalidation resolver path),
//! - denial: base-product and premium denials,
//! - policy registry load: one validated cold load plus warm cached lookups.

use std::hint::black_box;
use std::time::Instant;

use chrono::{DateTime, Utc};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::limit_reservation::LimitReservationService;
use focusa_license::{
    CapabilityFamily, CommercialTreatment, DecisionReason, OperationClass, PolicyActivation,
    PolicyEntitlementState, RecoveryAllowance, ResolvedEntitlementPolicy,
    embedded_entitlement_policy_registry, reduce_entitlement_state, resolve_base_focusa_product,
    resolve_premium_family,
};

const WARMUP: u32 = 2_000;
const ITERATIONS: u64 = 300_000;

fn measure(name: &str, mut operation: impl FnMut()) {
    for _ in 0..WARMUP {
        operation();
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        operation();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / ITERATIONS as f64;
    let ops_per_sec = ITERATIONS as f64 / elapsed.as_secs_f64();
    println!(
        "bench {name}: {ns_per_op:.1} ns/op, {ops_per_sec:.0} ops/s \
         ({ITERATIONS} iterations; deterministic, in-memory, secret-free)"
    );
}

/// Active signed `focusa` snapshot with one registered automation feature and
/// one signed limit unit. Deterministic fixture; no chrono time arithmetic
/// needed because `expires_at` stays `None` (the resolver treats a missing
/// expiry as no expiry boundary).
fn active_automation_snapshot() -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-bench");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-bench".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:bench".to_string());
    snapshot
        .features
        .insert("focusa.agent.parallelism".to_string(), true);
    snapshot.limits.insert("concurrent_agents".to_string(), 8);
    snapshot
}

/// Active signed `focusa` snapshot with NO premium feature grant: every
/// premium family resolution must fail closed.
fn active_snapshot_without_features() -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-bench");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-bench".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:bench".to_string());
    snapshot
}

fn cold_validated_base_policy() -> ResolvedEntitlementPolicy {
    ResolvedEntitlementPolicy::try_new(
        OperationClass::ValueMutation,
        CapabilityFamily::BaseFocusa,
        CommercialTreatment::BaseEntitlement,
        PolicyActivation::Active,
        PolicyEntitlementState::ActivePaid,
        None,
        None,
        RecoveryAllowance::None,
        DecisionReason::RequireBase,
    )
    .expect("benchmark fixture must pass cold validation")
}

fn main() {
    println!(
        "focusa-license entitlement_policy benchmark (spec152f.06.06) \
         — resolver + middleware resolver path + policy registry load"
    );

    // 1. Hot cached decision: the per-request state-grid reduction.
    measure("hot_cached_decision", || {
        let decision = reduce_entitlement_state(
            PolicyEntitlementState::ActivePaid,
            CapabilityFamily::BaseFocusa,
            None,
        );
        black_box(decision);
    });

    // 2. Cold validation: full typed policy construction with every invariant.
    measure("cold_validation", || {
        let policy = cold_validated_base_policy();
        black_box(policy);
    });

    // 3. Premium + limit decision (middleware / scheduler revalidation path):
    //    registered premium feature resolution plus a signed limit reservation.
    let snapshot = active_automation_snapshot();
    let now: DateTime<Utc> = Utc::now();
    measure("premium_plus_limit_decision", || {
        let decision = resolve_premium_family(
            &snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.parallelism",
            now,
        );
        assert!(
            decision.is_feature(),
            "fixture must resolve to a feature grant"
        );
        let mut service = LimitReservationService::new();
        let grant = service
            .reserve(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.parallelism",
                "concurrent_agents",
                "bench-key",
                1,
                now,
            )
            .expect("fixture must reserve one signed unit");
        black_box((decision, grant));
    });

    // 4. Denial: base-product denial and premium missing-feature denial.
    let denied_snapshot = active_snapshot_without_features();
    measure("denial_decision", || {
        let base = resolve_base_focusa_product("focusa", PolicyEntitlementState::RefundedOrRevoked);
        assert!(!base.permits_base_mutations(), "denied fixture");
        let premium = resolve_premium_family(
            &denied_snapshot,
            CapabilityFamily::Automation,
            "focusa.agent.parallelism",
            now,
        );
        assert!(premium.denial().is_some(), "denied fixture");
        black_box((base, premium));
    });

    // 5. Policy registry load path: one validated cold load, then warm cached
    //    lookups. The registry is `include_str!`-compiled and `OnceLock`-cached,
    //    so there is no per-request file parsing or network dependency.
    let start = Instant::now();
    let registry = embedded_entitlement_policy_registry()
        .expect("embedded policy registry must load and validate");
    let cold_load_ns = start.elapsed().as_nanos();
    println!(
        "bench policy_registry_cold_load: {cold_load_ns} ns (one validated schema+digest load)"
    );
    black_box(registry);
    measure("policy_registry_warm_lookup", || {
        let families = registry.family_count();
        let license_types = registry.license_type_count();
        let digest = registry.digest();
        black_box((families, license_types, digest));
    });

    println!("bench complete: all scenarios bounded, deterministic, in-memory, secret-free");
}
