//! Spec 152F.06.06 — resolver/middleware overhead measurement: bounded
//! observability counters, cached policy registry load, and deterministic
//! resolver hot paths.
//!
//! Run via: `cargo test --workspace spec152f_observability`
//!
//! These tests verify the bounded-result contract: fixed-capacity counters
//! keyed only by canonical family/reason/posture labels (never customer
//! secrets), a single validated embedded registry load (no per-request file
//! parsing or network dependency), and deterministic in-memory resolver paths.

use std::collections::{BTreeMap, BTreeSet};

use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    CapabilityFamily, DecisionReason, EntitlementDecisionCounters, EntitlementPolicyPosture,
    PolicyEntitlementState as State, PremiumFamilyDecision, embedded_entitlement_policy_registry,
    reduce_entitlement_state, resolve_base_focusa_product, resolve_premium_family,
};

fn active_snapshot() -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-obs");
    snapshot.state = EntitlementState::Active;
    snapshot.lease_id = Some("lease-obs".to_string());
    snapshot.sequence = Some(7);
    snapshot.lease_digest = Some("sha256:obs".to_string());
    snapshot
        .features
        .insert("focusa.agent.parallelism".to_string(), true);
    snapshot.limits.insert("concurrent_agents".to_string(), 8);
    snapshot
}

/// The exact set of canonical snapshot labels: one per enum variant, derived
/// from the same label/status methods the counters module uses. A snapshot key
/// outside this set would be evidence of raw authority data leaking in.
fn canonical_snapshot_keys() -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for family in [
        CapabilityFamily::AccountRecovery,
        CapabilityFamily::ReadProjection,
        CapabilityFamily::BaseFocusa,
        CapabilityFamily::Automation,
        CapabilityFamily::TeamRemote,
        CapabilityFamily::ReleaseProof,
        CapabilityFamily::PremiumUpdates,
        CapabilityFamily::CustomerDataExport,
        CapabilityFamily::InternalMaintenance,
    ] {
        keys.insert(format!("family.{}.count", family.label()));
    }
    for reason in [
        DecisionReason::Allow,
        DecisionReason::AllowVerifiedLimited,
        DecisionReason::Read,
        DecisionReason::ReadLocalOnly,
        DecisionReason::AllowExistingLocalOnly,
        DecisionReason::AllowOfflineOnly,
        DecisionReason::RequireBase,
        DecisionReason::RequireFeature,
        DecisionReason::RequireCachedFeature,
        DecisionReason::RequireCachedFeatureWhenSafe,
        DecisionReason::Inherit,
        DecisionReason::MissingInitiatingPolicy,
        DecisionReason::Deny,
    ] {
        keys.insert(format!("reason.{}.count", reason.label()));
    }
    for posture in [
        EntitlementPolicyPosture::Allow,
        EntitlementPolicyPosture::Read,
        EntitlementPolicyPosture::Base,
        EntitlementPolicyPosture::Feature,
        EntitlementPolicyPosture::Deny,
    ] {
        keys.insert(format!("posture.{}.count", posture.status()));
    }
    keys
}

#[test]
fn spec152f_observability_counters_are_bounded_label_only_and_secret_free() {
    let counters = EntitlementDecisionCounters::new();
    assert_eq!(counters.capacity(), 9 + 13 + 5, "fixed bounded capacity");

    // Every counter starts at zero; snapshot is label-only and its key set is
    // exactly the canonical label set — no raw authority identifier can appear.
    let snapshot: BTreeMap<String, u64> = counters.snapshot();
    assert_eq!(
        snapshot.len(),
        9 + 13 + 5,
        "exactly one slot per canonical variant"
    );
    assert_eq!(
        snapshot.keys().cloned().collect::<BTreeSet<_>>(),
        canonical_snapshot_keys(),
        "snapshot keys must be exactly the canonical family/reason/posture labels"
    );
    assert!(
        snapshot.values().all(|count| *count == 0),
        "fresh counters must be zero"
    );
}

#[test]
fn spec152f_observability_counters_record_by_family_reason_and_posture() {
    let counters = EntitlementDecisionCounters::new();
    for _ in 0..3 {
        counters.record_decision(
            CapabilityFamily::BaseFocusa,
            DecisionReason::RequireBase,
            EntitlementPolicyPosture::Base,
        );
    }
    counters.record_decision(
        CapabilityFamily::Automation,
        DecisionReason::Deny,
        EntitlementPolicyPosture::Deny,
    );
    counters.record_decision(
        CapabilityFamily::AccountRecovery,
        DecisionReason::Allow,
        EntitlementPolicyPosture::Allow,
    );

    assert_eq!(counters.total(), 5);
    assert_eq!(counters.family_count(CapabilityFamily::BaseFocusa), 3);
    assert_eq!(counters.family_count(CapabilityFamily::Automation), 1);
    assert_eq!(counters.reason_count(DecisionReason::RequireBase), 3);
    assert_eq!(counters.reason_count(DecisionReason::Deny), 1);
    assert_eq!(counters.posture_count(EntitlementPolicyPosture::Base), 3);
    assert_eq!(counters.posture_count(EntitlementPolicyPosture::Deny), 1);
    assert_eq!(counters.posture_count(EntitlementPolicyPosture::Allow), 1);

    let snapshot = counters.snapshot();
    assert_eq!(snapshot["family.base_focusa.count"], 3);
    assert_eq!(snapshot["reason.deny.count"], 1);
    assert_eq!(snapshot["posture.deny.count"], 1);
    // Capacity is fixed regardless of workload.
    assert_eq!(counters.capacity(), 9 + 13 + 5);
}

#[test]
fn spec152f_observability_policy_registry_load_is_cached_and_deterministic() {
    // The embedded registry is compiled in via include_str! and loaded exactly
    // once through a OnceLock: repeated calls return the same instance, so a
    // per-request policy load can never re-parse files or hit the network.
    let first = embedded_entitlement_policy_registry().expect("registry loads");
    let second = embedded_entitlement_policy_registry().expect("registry loads");
    assert!(
        std::ptr::eq(first, second),
        "registry must be a single cached instance"
    );

    assert!(!first.digest().is_empty());
    assert!(first.family_count() > 0, "registry must carry families");
    assert!(
        first.license_type_count() > 0,
        "registry must carry license types"
    );
    let canonical_json = first.canonical_json();
    assert!(
        !canonical_json.is_empty() && canonical_json.contains("entitlement_policy"),
        "canonical registry document is present"
    );
}

#[test]
fn spec152f_observability_resolver_hot_path_is_deterministic_and_bounded() {
    // The per-request state-grid reduction is a pure function: identical inputs
    // always produce identical bounded decisions, with no I/O or network.
    for _ in 0..100 {
        let decision =
            reduce_entitlement_state(State::ActivePaid, CapabilityFamily::BaseFocusa, None);
        assert_eq!(decision.posture(), EntitlementPolicyPosture::Base);
        assert_eq!(decision.reason(), DecisionReason::RequireBase);
    }
    let denial =
        reduce_entitlement_state(State::RefundedOrRevoked, CapabilityFamily::BaseFocusa, None);
    assert_eq!(denial.posture(), EntitlementPolicyPosture::Deny);
    assert_eq!(denial.reason(), DecisionReason::Deny);
}

#[test]
fn spec152f_observability_middleware_resolver_sequence_is_bounded_and_records() {
    // The sequence the API middleware/scheduler revalidation executes per
    // request: base gate, then premium family resolution, then recording a
    // label-only counter. All steps are in-memory and deterministic.
    let snapshot = active_snapshot();
    let now = chrono::Utc::now();

    let base = resolve_base_focusa_product(&snapshot.product, State::ActivePaid);
    assert_eq!(base.label(), "entitled");

    let decision: PremiumFamilyDecision = resolve_premium_family(
        &snapshot,
        CapabilityFamily::Automation,
        "focusa.agent.parallelism",
        now,
    );
    assert!(decision.is_feature(), "registered feature must resolve");

    let counters = EntitlementDecisionCounters::new();
    counters.record_decision(
        CapabilityFamily::Automation,
        DecisionReason::RequireFeature,
        EntitlementPolicyPosture::Feature,
    );
    assert_eq!(counters.family_count(CapabilityFamily::Automation), 1);
    assert_eq!(counters.reason_count(DecisionReason::RequireFeature), 1);
    assert_eq!(counters.posture_count(EntitlementPolicyPosture::Feature), 1);

    // Denial path records without leaking the snapshot identity.
    let mut denied = active_snapshot();
    denied.features.clear();
    let denial = resolve_premium_family(
        &denied,
        CapabilityFamily::Automation,
        "focusa.agent.parallelism",
        now,
    );
    assert!(denial.denial().is_some());
    counters.record_decision(
        CapabilityFamily::Automation,
        DecisionReason::Deny,
        EntitlementPolicyPosture::Deny,
    );
    let snapshot_labels = counters.snapshot();
    assert_eq!(
        snapshot_labels.keys().cloned().collect::<BTreeSet<_>>(),
        canonical_snapshot_keys(),
        "counters must never expose raw authority identifiers"
    );
    assert_eq!(snapshot_labels["reason.deny.count"], 1);
}
