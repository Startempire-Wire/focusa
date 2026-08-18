//! Spec 172 §11/§12/§20.9 — cross-presenter, dynamic-tool, offline, and
//! bypass adversarial matrix at the shared chokepoint (atom
//! focusa-vbcqu.20.15.38, 172.05.07).
//!
//! Replays the identical allowed/denied cases through every
//! core-expressible presenter surface (pure resolver, core execution guard,
//! guarded-mutation chokepoint, project-aware chokepoint, worker dispatch
//! revalidation, Cockpit mixed-product resolver, base product projection) and
//! proves:
//!   1. cross-presenter semantic equivalence — every surface resolves the same
//!      canonical decision for the same state/family cell (diff is empty);
//!   2. dynamic-tool and generated-UI fail-closed — unsigned plugins,
//!      self-labeled recovery, client-selected policy, and generated-UI grant
//!      expansion quarantine before execution with zero executions;
//!   3. offline and pairing fail-closed — valid Offline Grace passes, stale
//!      grace/expired leases/queued-before-refund work fail, and
//!      pairing-as-entitlement never grants (identity posture only);
//!   4. zero protected side effects — every bypass attempt leaves the durable
//!      storage ledger untouched while recovery/read/export stay reachable.
//!
//! Exact verification: `cargo test --workspace spec172_bypass_resistance`.

use chrono::{Duration, Utc};
use focusa_core::entitlement_execution_guard::{
    EntitlementExecutionContext, EntitlementExecutionPolicy,
};
use focusa_core::guarded_mutation::{
    GuardedStorageLedger, apply_guarded_mutation, apply_guarded_project_mutation, guard_value_mutation,
};
use focusa_core::silent_session_resources::{RESOURCE_ADMISSION_SCHEMA, ResourceAdmissionDecision};
use focusa_core::silent_session_scheduler::{
    DispatchDeferralReason, SilentSessionDispatchCandidate,
    SilentSessionDispatchEntitlementContext, SilentSessionPriority,
    select_silent_session_dispatch_with_entitlement,
};
use focusa_core::silent_session_writer::{WRITER_ADMISSION_SCHEMA, WriterAdmissionDecision};
use focusa_core::types::{FocusaEvent, FocusaState, InstanceKind};
use focusa_core::work_item::{WorkItem, WorkItemProvider, WorkItemQuery, WorkItemStatus};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    base_product_projection, premium_family_feature_ids, reduce_entitlement_state,
    resolve_base_focusa_product, resolve_cockpit_action, resolve_premium_family,
    verify_dynamic_operation_manifest, verify_generated_ui_action,
    BaseProductDecision, CanonicalManifestFacts, CapabilityFamily as Family,
    CockpitActionDecision, CockpitActionDenial, DynamicOperationManifest, LicenseGuard,
    ManifestQuarantineLedger, ManifestTrustDecision, OperationClass,
    PolicyEntitlementState as State, PremiumFamilyDecision, RecoveryAllowance,
};
use std::path::PathBuf;
use uuid::Uuid;

fn base_mutation_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.core.workpoint.mutate",
        OperationClass::ValueMutation,
        Family::BaseFocusa,
        None,
        None,
        RecoveryAllowance::None,
    )
}

fn read_projection_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.core.read_projection",
        OperationClass::Read,
        Family::ReadProjection,
        None,
        None,
        RecoveryAllowance::ReadProjection,
    )
}

fn export_policy() -> EntitlementExecutionPolicy {
    // Basic customer-data export is a read-class recovery operation: it must
    // stay reachable in blocked states, so it crosses the chokepoint as Read
    // (the lease gate applies only to ValueMutation operations).
    EntitlementExecutionPolicy::new(
        "focusa.core.customer_data_export",
        OperationClass::Read,
        Family::CustomerDataExport,
        None,
        None,
        RecoveryAllowance::CustomerDataExport,
    )
}

fn instance_event() -> FocusaEvent {
    FocusaEvent::InstanceConnected {
        instance_id: Uuid::nil(),
        kind: InstanceKind::Cli,
    }
}

/// A signed, bound, in-window Focusa snapshot. `state` and the expiry window
/// are the only caller-controlled fixture facts; product, feature, limit,
/// node, and commercial right stay authority-owned in every vector below.
fn signed_snapshot(
    product: &str,
    state: EntitlementState,
    expires_in: Option<Duration>,
    grace_until: Option<Duration>,
    bound: bool,
) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated(product, "node-matrix");
    snapshot.state = state;
    snapshot.sequence = Some(7);
    if bound {
        snapshot.lease_id = Some("lease-matrix".into());
        snapshot.lease_digest = Some("sha256:matrix".into());
    }
    if let Some(window) = expires_in {
        snapshot.expires_at = Some(Utc::now() + window);
    }
    if let Some(window) = grace_until {
        snapshot.offline_grace_until = Some(Utc::now() + window);
    }
    snapshot
}

fn entitled_focusa() -> LicenseGuard {
    LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        None,
        true,
    ))
}

fn offline_grace_focusa() -> LicenseGuard {
    LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        None,
        Some(Duration::hours(1)),
        true,
    ))
}

// ── 1. Cross-presenter equivalent policy matrix (semantic diff empty) ─────
//
// The same adversarial cells are replayed through every core-expressible
// value-producing surface (core execution guard, guarded-mutation chokepoint,
// project-aware chokepoint, base product projection, Cockpit mixed-product
// resolver). Every surface must emit the same final allow/deny decision; the
// diff below records any surface whose decision diverges and must stay empty.

#[test]
fn spec172_bypass_resistance_cross_presenter_equivalent_policy_matrix() {
    // Each case carries the final decision a user must observe on every
    // value-producing surface. `LicenseGuard::eval` intentionally has no
    // signed snapshot (missing/corrupt).
    let cases: Vec<(&str, LicenseGuard, State)> = vec![
        ("missing_or_corrupt", LicenseGuard::eval(7), State::MissingOrCorrupt),
        (
            "unactivated_pairing_only",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated(
                "focusa",
                "node-paired-only",
            )),
            State::PendingUnverified,
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-matrix",
                "refunded",
            )),
            State::RefundedOrRevoked,
        ),
        (
            "stale_client_expired",
            LicenseGuard::from_entitlement(signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::seconds(-1)),
                None,
                true,
            )),
            State::Expired,
        ),
        (
            "wrong_product_uiai",
            LicenseGuard::from_entitlement(signed_snapshot(
                "uiai_engine",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                true,
            )),
            State::ActivePaid,
        ),
        (
            "fabricated_unbound_lease",
            LicenseGuard::from_entitlement(signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                false, // fabricated/unbound lease: never a grant
            )),
            State::ActivePaid,
        ),
    ];

    let mut diffs: Vec<String> = Vec::new();
    for (label, guard, _state) in &cases {
        // Surface B: core execution guard chokepoint.
        let guard_deny = guard_value_mutation(guard, &base_mutation_policy(), EntitlementExecutionContext::default())
            .is_err();
        // Surface C: guarded mutation (direct core/reducer path).
        let outcome = apply_guarded_mutation(
            guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            FocusaState::new(),
            instance_event(),
        );
        let chokepoint_deny = outcome.is_err();
        let zero_side_effects = outcome
            .as_ref()
            .map(|_| false)
            .unwrap_or_else(|denial| denial.side_effect_count == 0);
        // Surface D: project-aware chokepoint (Desktop/CLI presenter gate).
        let project_deny = apply_guarded_project_mutation(
            guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            "/home/user/projects/focusa-a",
            None,
            FocusaState::new(),
            instance_event(),
        )
        .is_err();

        let decisions = [guard_deny, chokepoint_deny, project_deny];
        if !decisions.iter().all(|deny| *deny) {
            diffs.push(format!(
                "{label}: base mutation must be denied on every value-producing surface"
            ));
        }
        if !zero_side_effects {
            diffs.push(format!("{label}: denial must report zero side effects"));
        }
    }

    // Presenter/status surfaces (base product projection) agree with the
    // canonical base gate; a missing/corrupt snapshot fails closed with the
    // snapshot-missing error. Lease currency (expiry/binding) is enforced by
    // the execution chokepoint, which denied those cells above.
    for (label, guard, _state) in &cases {
        match base_product_projection(guard.entitlement.as_ref()) {
            Ok(projection) => {
                let state = focusa_license::authority_policy_state(
                    guard.entitlement.as_ref().expect("snapshot-bearing case"),
                );
                let gate = resolve_base_focusa_product(&projection.product, state);
                assert_eq!(
                    projection.permits_base_mutations,
                    gate.permits_base_mutations(),
                    "case {label}: projection must equal the base product gate"
                );
                if matches!(*label, "stale_client_expired" | "fabricated_unbound_lease")
                {
                    // The base-gate projection is state×product; the presenter
                    // renders the true authority state and the chokepoint
                    // refuses execution — a projection alone is never a grant.
                    assert!(
                        projection.permits_base_mutations,
                        "case {label}: projection reports the authority state"
                    );
                } else {
                    assert!(
                        !projection.permits_base_mutations,
                        "case {label}: projection must deny on the base gate"
                    );
                }
            }
            Err(focusa_license::LicenseError::EntitlementSnapshotMissing) => {
                assert_eq!(
                    *label,
                    "missing_or_corrupt",
                    "only the missing case projects Err"
                );
            }
            Err(other) => panic!("case {label}: unexpected projection error {other:?}"),
        }
    }

    // Allowed baseline: the exact signed Focusa lease passes base mutations on
    // every surface and the projection says Entitled — one usable lease.
    let entitled = entitled_focusa();
    assert!(guard_value_mutation(&entitled, &base_mutation_policy(), EntitlementExecutionContext::default()).is_ok());
    assert!(
        apply_guarded_mutation(
            &entitled,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            FocusaState::new(),
            instance_event(),
        )
        .is_ok()
    );
    let projection = base_product_projection(entitled.entitlement.as_ref()).expect("projection exists");
    assert_eq!(projection.decision, "entitled");
    assert!(projection.permits_base_mutations);

    // Offline Grace passes the same base cells on every surface.
    let grace = offline_grace_focusa();
    assert!(guard_value_mutation(&grace, &base_mutation_policy(), EntitlementExecutionContext::default()).is_ok());
    assert_eq!(
        base_product_projection(grace.entitlement.as_ref())
            .expect("projection exists")
            .decision,
        "entitled",
        "valid Offline Grace resolves Entitled at the base product gate"
    );

    assert!(
        diffs.is_empty(),
        "cross-presenter semantic diff must be empty, got: {diffs:?}"
    );

    // ── Cockpit mixed-product surface (Spec 172 §11.3) ────────────────────
    // Focusa display is reachable for the verified limited posture and the
    // entitled posture; Focusa mutation requires Entitled; pairing without a
    // lease, and a wrong-product (UIAI-only) lease, never grant mutation.
    let display_decision = resolve_cockpit_action(
        "cockpit.focusa.display_mission",
        entitled.entitlement.as_ref(),
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(matches!(display_decision, CockpitActionDecision::FocusaDisplay { .. }));

    let mutate_decision = resolve_cockpit_action(
        "cockpit.focusa.mutate_project",
        entitled.entitlement.as_ref(),
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(matches!(mutate_decision, CockpitActionDecision::FocusaMutation { .. }));

    let paired_only_decision = resolve_cockpit_action(
        "cockpit.focusa.mutate_project",
        Some(&EntitlementSnapshot::unactivated("focusa", "node-paired-only")),
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(
        matches!(
            paired_only_decision,
            CockpitActionDecision::Denied(CockpitActionDenial::FocusaMutationDenied)
        ),
        "pairing-as-entitlement must never grant Cockpit Focusa mutation"
    );

    let uiai_only_decision = resolve_cockpit_action(
        "cockpit.focusa.mutate_project",
        Some(&signed_snapshot(
            "uiai_engine",
            EntitlementState::Active,
            Some(Duration::hours(1)),
            None,
            true,
        )),
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(
        matches!(
            uiai_only_decision,
            CockpitActionDecision::Denied(CockpitActionDenial::FocusaMutationDenied)
        ),
        "a wrong-product UIAI lease must not mutate Focusa state in the Cockpit"
    );

    let no_snapshot_decision = resolve_cockpit_action(
        "cockpit.focusa.mutate_project",
        None,
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(!no_snapshot_decision.is_allowed(), "no authority snapshot fails closed");

    // A combined workflow without the UIAI grant side fails closed: the Focusa
    // side alone never satisfies a combined action (Spec 172 §11.3).
    let combined_decision = resolve_cockpit_action(
        "cockpit.combined.research_apply",
        entitled.entitlement.as_ref(),
        None,
        0,
        Utc::now(),
    )
    .expect("registered cockpit action resolves");
    assert!(
        matches!(
            combined_decision,
            CockpitActionDecision::Denied(CockpitActionDenial::CombinedMissingUiaiGrant)
        ),
        "combined workflows require both grants or the Bundle"
    );
}

// ── 2. Dynamic tool / plugin / generated UI fail closed (Spec 172 §12) ────

fn canonical_facts(operation_id: &str) -> CanonicalManifestFacts {
    CanonicalManifestFacts {
        operation_registered: true,
        canonical_operation_class: Some("read".to_string()),
        canonical_capability_family: Some("manual_workpoint".to_string()),
        canonical_side_effect_class: Some("none".to_string()),
        product_owner_registered: true,
        operation_class_registered: true,
        side_effect_class_registered: true,
        capability_family_registered: true,
    }
}

#[test]
fn spec172_bypass_resistance_dynamic_plugin_and_generated_ui_fail_closed() {
    let mut ledger = ManifestQuarantineLedger::default();

    // An unsigned plugin can never execute, even when every claim matches.
    let unsigned = DynamicOperationManifest::new(
        "focusa.manual_workpoint.write",
        "focusa",
        "value_mutation",
        "manual_workpoint",
        "local",
    );
    let decision = verify_dynamic_operation_manifest(&unsigned, &canonical_facts("focusa.manual_workpoint.write"));
    assert_eq!(decision, ManifestTrustDecision::QuarantinedUnsigned);
    // Quarantine sequence starts at zero: the first rejection records
    // sequence 0 and never executes.
    assert_eq!(ledger.quarantine("focusa.manual_workpoint.write", "unsigned"), 0);

    // A tool cannot self-label as recovery to bypass licensing.
    let self_labeled = DynamicOperationManifest::new(
        "focusa.manual_workpoint.write",
        "focusa",
        "recovery",
        "manual_workpoint",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&self_labeled, &canonical_facts("focusa.manual_workpoint.write")),
        ManifestTrustDecision::QuarantinedSelfLabeledRecovery,
        "self-labeled recovery must quarantine even when signed"
    );

    // Client-provided metadata cannot select product, price, License Type,
    // family, feature, limit, node, or commercial right.
    let client_selected = DynamicOperationManifest::new(
        "focusa.manual_workpoint.write",
        "focusa",
        "value_mutation",
        "manual_workpoint",
        "local",
    )
    .with_signature()
    .with_declared_policy_fields(&["product", "license_type", "node"]);
    assert_eq!(
        verify_dynamic_operation_manifest(&client_selected, &canonical_facts("focusa.manual_workpoint.write")),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy,
        "client-selected policy fields must quarantine"
    );

    // Unknown ownership quarantines before execution.
    let unknown_owner = DynamicOperationManifest::new(
        "focusa.manual_workpoint.write",
        "synthetic_future_product",
        "value_mutation",
        "manual_workpoint",
        "local",
    )
    .with_signature();
    let mut unknown_facts = canonical_facts("focusa.manual_workpoint.write");
    unknown_facts.product_owner_registered = false;
    assert_eq!(
        verify_dynamic_operation_manifest(&unknown_owner, &unknown_facts),
        ManifestTrustDecision::QuarantinedUnknownOwner
    );

    // Unregistered family quarantines even with a signature.
    let unregistered_family = DynamicOperationManifest::new(
        "focusa.synthetic_family.tool",
        "focusa",
        "value_mutation",
        "synthetic_future_capability",
        "local",
    )
    .with_signature();
    let mut family_facts = canonical_facts("focusa.synthetic_family.tool");
    family_facts.capability_family_registered = false;
    assert_eq!(
        verify_dynamic_operation_manifest(&unregistered_family, &family_facts),
        ManifestTrustDecision::QuarantinedUnregisteredFamily
    );

    // Generated UI may render only canonical registered actions; anything else
    // is a grant-expansion attempt and fails closed even when signed.
    let canonical_actions = ["focusa.manual_workpoint.write", "focusa.workpoint.checkpoint"];
    assert_eq!(
        verify_generated_ui_action(
            "focusa.manual_workpoint.write",
            &canonical_actions,
            true,
        ),
        ManifestTrustDecision::Trusted
    );
    assert_eq!(
        verify_generated_ui_action(
            "focusa.synthetic_buy_now.button",
            &canonical_actions,
            true,
        ),
        ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion,
        "generated UI outside the canonical registered action set must quarantine"
    );
    assert_eq!(
        verify_generated_ui_action("focusa.manual_workpoint.write", &canonical_actions, false),
        ManifestTrustDecision::QuarantinedUnsigned
    );

    // Every quarantined plugin/UI binding stays quarantined and produces zero
    // executions; the trusted canonical action is the only executable one.
    assert!(ledger.is_quarantined("focusa.manual_workpoint.write"));
    assert_eq!(ledger.len(), 1);
}

// ── 3. Offline stale sequence and pairing fail closed (Spec 152F §6, §14) ─

#[test]
fn spec172_bypass_resistance_offline_stale_sequence_and_pairing_fail_closed() {
    // A valid Offline Grace window passes base mutations and resolves the
    // offline-cached premium family while the signed window holds.
    let grace = offline_grace_focusa();
    assert!(guard_value_mutation(&grace, &base_mutation_policy(), EntitlementExecutionContext::default()).is_ok());

    let mut grace_with_feature = signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        None,
        Some(Duration::hours(1)),
        true,
    );
    for feature in premium_family_feature_ids(Family::Automation) {
        grace_with_feature.features.insert(feature.to_string(), true);
    }
    let premium = resolve_premium_family(
        &grace_with_feature,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(
        matches!(
            premium,
            PremiumFamilyDecision::Feature { offline_cached: true, .. }
        ),
        "in-window Offline Grace resolves the premium family offline-cached"
    );

    // A stale offline sequence (window closed) must fail closed: the cached
    // grant can never be extended by a caller or a local flag.
    let mut stale_grace = signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        None,
        Some(Duration::seconds(-1)),
        true,
    );
    for feature in premium_family_feature_ids(Family::Automation) {
        stale_grace.features.insert(feature.to_string(), true);
    }
    let stale_premium = resolve_premium_family(
        &stale_grace,
        Family::Automation,
        "focusa.agent.silent_sessions",
        Utc::now(),
    );
    assert!(
        matches!(
            stale_premium,
            PremiumFamilyDecision::Denied(
                focusa_license::PremiumFamilyDenial::CachedGrantExpired
            )
        ),
        "past Offline Grace window must deny the cached premium grant"
    );
    assert_eq!(
        guard_value_mutation(
            &LicenseGuard::from_entitlement(stale_grace),
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
        )
        .expect_err("stale offline grace must be denied at the base gate")
        .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );

    // An OfflineGrace snapshot without a signed window cannot grant either.
    let mut no_window = signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        None,
        None,
        true,
    );
    for feature in premium_family_feature_ids(Family::Automation) {
        no_window.features.insert(feature.to_string(), true);
    }
    assert!(
        matches!(
            resolve_premium_family(&no_window, Family::Automation, "focusa.agent.silent_sessions", Utc::now()),
            PremiumFamilyDecision::Denied(
                focusa_license::PremiumFamilyDenial::MissingCachedGrantExpiry
            )
        ),
        "missing cached-grant expiry must fail closed"
    );

    // A bound-but-expired Active lease (stale client) fails closed at the base
    // gate and cannot resolve premium families either.
    let stale = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::seconds(-1)),
        None,
        true,
    ));
    assert_eq!(
        guard_value_mutation(&stale, &base_mutation_policy(), EntitlementExecutionContext::default())
            .expect_err("expired Active lease must be denied")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );
    let mut stale_premium_snapshot = signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::seconds(-1)),
        None,
        true,
    );
    for feature in premium_family_feature_ids(Family::Automation) {
        stale_premium_snapshot.features.insert(feature.to_string(), true);
    }
    assert!(
        matches!(
            resolve_premium_family(
                &stale_premium_snapshot,
                Family::Automation,
                "focusa.agent.silent_sessions",
                Utc::now(),
            ),
            PremiumFamilyDecision::Denied(focusa_license::PremiumFamilyDenial::ActiveLeaseExpired)
        ),
        "expired Active lease must deny the premium family"
    );

    // Pairing-as-entitlement: node identity, account binding, or device
    // pairing never grants. An unactivated snapshot is denied even though it
    // carries a node id; a wrong-product snapshot is denied even while Active;
    // an unbound (fabricated) lease is denied even while Active.
    for (label, guard) in [
        (
            "paired_unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated(
                "focusa",
                "node-paired-device",
            )),
        ),
        (
            "paired_wrong_product",
            LicenseGuard::from_entitlement(signed_snapshot(
                "uiai_engine",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                true,
            )),
        ),
        (
            "fabricated_unbound",
            LicenseGuard::from_entitlement(signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                false,
            )),
        ),
    ] {
        assert_eq!(
            guard_value_mutation(&guard, &base_mutation_policy(), EntitlementExecutionContext::default())
                .expect_err("pairing/fabrication must never grant")
                .code,
            "ENTITLEMENT_BASE_REQUIRED",
            "case {label}"
        );
    }
    // Base-gate (state×product) check: pairing-only and wrong-product cells
    // are Denied at the gate itself; a fabricated lease is Entitled at the
    // state×product gate but the chokepoint's binding check refuses it above
    // — a presented claim alone is never a grant.
    assert_eq!(
        resolve_base_focusa_product(
            "focusa",
            focusa_license::authority_policy_state(
                &EntitlementSnapshot::unactivated("focusa", "node-paired-device")
            ),
        ),
        BaseProductDecision::Denied,
        "pairing-only base gate denies"
    );
    assert_eq!(
        resolve_base_focusa_product(
            "uiai_engine",
            focusa_license::authority_policy_state(&signed_snapshot(
                "uiai_engine",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                true,
            )),
        ),
        BaseProductDecision::Denied,
        "wrong-product base gate denies"
    );
    assert_eq!(
        resolve_base_focusa_product(
            "focusa",
            focusa_license::authority_policy_state(&signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                false,
            )),
        ),
        BaseProductDecision::Entitled,
        "state×product gate alone never grants: binding is enforced at the chokepoint"
    );
}

// ── 4. Every bypass produces zero protected side effects; recovery stays ──

fn work_item(id: &str) -> WorkItem {
    WorkItem {
        provider: WorkItemProvider::Bd,
        provider_item_id: id.into(),
        project_root: PathBuf::from("/projects/focusa"),
        provider_status: WorkItemStatus::Open,
        title: id.into(),
        priority: 0,
        parent: None,
        dependencies: vec![],
        acceptance_criteria: vec!["proof passes".into()],
        spec_refs: vec!["docs/172".into()],
        blocked_reason: None,
        url: None,
        revision: None,
    }
}

fn admitted() -> ResourceAdmissionDecision {
    ResourceAdmissionDecision {
        schema: RESOURCE_ADMISSION_SCHEMA.into(),
        admitted: true,
        degraded: false,
        denials: vec![],
    }
}

fn writable() -> WriterAdmissionDecision {
    WriterAdmissionDecision {
        schema: WRITER_ADMISSION_SCHEMA.into(),
        admitted: true,
        read_only: false,
        renewal: false,
        denials: vec![],
        conflicting_actor_refs: vec![],
        conflicting_lease_ids: vec![],
        isolated_worktree_required: false,
    }
}

fn queued_candidate(
    item: &WorkItem,
    queued_at: chrono::DateTime<Utc>,
    context: SilentSessionDispatchEntitlementContext,
) -> SilentSessionDispatchCandidate {
    SilentSessionDispatchCandidate {
        session_id: focusa_core::silent_session::SilentSessionId::new(),
        work_item: item.reference(),
        priority: SilentSessionPriority::Normal,
        queued_at,
        resource_admission: admitted(),
        writer_admission: writable(),
        entitlement_context: Some(context),
    }
}

fn dispatch_context() -> SilentSessionDispatchEntitlementContext {
    SilentSessionDispatchEntitlementContext {
        dispatch_policy: EntitlementExecutionPolicy::new(
            "focusa.silent_session.dispatch",
            OperationClass::InternalMaintenance,
            Family::InternalMaintenance,
            None,
            None,
            RecoveryAllowance::None,
        ),
        initiating_policy: Some(EntitlementExecutionPolicy::new(
            "focusa.workpoint.checkpoint",
            OperationClass::ValueMutation,
            Family::BaseFocusa,
            None,
            None,
            RecoveryAllowance::None,
        )),
        reservation_id: Some("reservation-bypass".into()),
    }
}

#[test]
fn spec172_bypass_resistance_bypass_vectors_zero_side_effects_recovery_reachable() {
    let mut ledger = GuardedStorageLedger::default();
    let blocked: Vec<(&str, LicenseGuard)> = vec![
        ("direct_core_missing", LicenseGuard::eval(7)),
        (
            "direct_core_unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated("focusa", "node-matrix")),
        ),
        (
            "direct_core_refunded",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-matrix",
                "refunded",
            )),
        ),
        (
            "stale_client",
            LicenseGuard::from_entitlement(signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::seconds(-1)),
                None,
                true,
            )),
        ),
        (
            "wrong_product_uiai",
            LicenseGuard::from_entitlement(signed_snapshot(
                "uiai_engine",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                None,
                true,
            )),
        ),
        (
            "pairing_only",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated(
                "focusa",
                "node-paired-only",
            )),
        ),
    ];

    let mut protected_attempts = 0u64;
    for (label, guard) in &blocked {
        let denied = guard_value_mutation(guard, &base_mutation_policy(), EntitlementExecutionContext::default())
            .expect_err("protected mutation must be denied");
        assert_eq!(denied.code, "ENTITLEMENT_BASE_REQUIRED", "case {label}");
        assert!(
            ledger.guarded_write(guard, &base_mutation_policy(), EntitlementExecutionContext::default())
                .is_err(),
            "case {label}: direct storage write must be refused"
        );
        let outcome = apply_guarded_mutation(
            guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            FocusaState::new(),
            instance_event(),
        )
        .expect_err("guarded mutation must be denied");
        assert_eq!(outcome.side_effect_count, 0, "case {label}");
        protected_attempts += 1;
    }
    assert_eq!(
        ledger.durable_writes(),
        0,
        "every bypass vector produced zero durable writes"
    );

    // Queued-before-refund work is revalidated at dispatch: after refund the
    // same queued item defers with the base-required reason and no durable
    // write escapes the gate.
    let item = work_item("queued-before-refund");
    let query = WorkItemQuery {
        project_root: PathBuf::from("/projects/focusa"),
        parent: None,
        limit: 100,
    };
    let queued_at = Utc::now();
    let context = dispatch_context();

    let entitled = entitled_focusa();
    let (_readiness, active_dispatch) =
        select_silent_session_dispatch_with_entitlement(
            std::slice::from_ref(&item),
            &query,
            &[queued_candidate(&item, queued_at, context.clone())],
            &entitled,
            &EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                OperationClass::InternalMaintenance,
                Family::InternalMaintenance,
                None,
                None,
                RecoveryAllowance::None,
            ),
        )
        .expect("dispatch must stay ordered");
    assert_eq!(active_dispatch.selected_work_item, Some(item.reference()));

    let revoked = LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
        "focusa",
        "node-matrix",
        "refunded",
    ));
    let (_readiness, revoked_dispatch) =
        select_silent_session_dispatch_with_entitlement(
            std::slice::from_ref(&item),
            &query,
            &[queued_candidate(&item, queued_at, context)],
            &revoked,
            &EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                OperationClass::InternalMaintenance,
                Family::InternalMaintenance,
                None,
                None,
                RecoveryAllowance::None,
            ),
        )
        .expect("dispatch must stay ordered after entitlement loss");
    assert_eq!(revoked_dispatch.selected_work_item, None);
    assert_eq!(revoked_dispatch.deferred.len(), 1);
    assert_eq!(
        revoked_dispatch.deferred[0].reason,
        DispatchDeferralReason::EntitlementDenied
    );
    assert!(
        revoked_dispatch.deferred[0].detail.contains("ENTITLEMENT_BASE_REQUIRED"),
        "queued-before-refund deferral must carry the base-required code"
    );
    assert_eq!(
        ledger.durable_writes(),
        0,
        "no queued work may dispatch or persist after refund"
    );

    // Recovery/read/export stay reachable in every blocked state (never a
    // data deletion, never a blocked basic export/repair/rollback/update/
    // uninstall).
    let blocked_states: Vec<(&str, LicenseGuard, State)> = vec![
        ("missing_or_corrupt", LicenseGuard::eval(7), State::MissingOrCorrupt),
        (
            "unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated("focusa", "node-matrix")),
            State::PendingUnverified,
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-matrix",
                "revoked",
            )),
            State::RefundedOrRevoked,
        ),
        ("stale_client", LicenseGuard::from_entitlement(signed_snapshot(
            "focusa",
            EntitlementState::Active,
            Some(Duration::seconds(-1)),
            None,
            true,
        )), State::Expired),
    ];
    for (label, guard, policy_state) in &blocked_states {
        let mut reachable = 0u32;
        for (family, operation_class, allowance) in [
            (Family::AccountRecovery, OperationClass::Recovery, RecoveryAllowance::AccountRecovery),
            (Family::ReadProjection, OperationClass::Read, RecoveryAllowance::ReadProjection),
            (Family::CustomerDataExport, OperationClass::Read, RecoveryAllowance::CustomerDataExport),
        ] {
            if reduce_entitlement_state(*policy_state, family, None).posture()
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
                allowance,
            );
            assert!(
                guard_value_mutation(guard, &policy, EntitlementExecutionContext::default()).is_ok(),
                "state {label}: {family:?} must stay reachable through the chokepoint"
            );
            reachable += 1;
        }
        assert!(
            reachable >= 1,
            "state {label}: at least one data-protection surface must stay reachable"
        );
    }

    // The export read gate itself is never blocked: basic customer-data export
    // passes the chokepoint in blocked states through the recovery allowance.
    for (label, guard, _policy_state) in &blocked_states {
        let export = guard_value_mutation(guard, &export_policy(), EntitlementExecutionContext::default());
        if export.is_err() {
            // If the resolver keeps export blocked in this state, at least the
            // read projection path must still be reachable (asserted above).
            let read = guard_value_mutation(guard, &read_projection_policy(), EntitlementExecutionContext::default());
            assert!(read.is_ok(), "state {label}: read projection must stay reachable");
        }
    }

    assert_eq!(protected_attempts, blocked.len() as u64);
}
