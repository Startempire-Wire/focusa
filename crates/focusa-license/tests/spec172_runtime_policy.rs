//! Spec 172 §20 — complete runtime policy matrix (registry + pure resolver
//! vectors, atom focusa-vbcqu.20.15.24, 172.03.07).
//!
//! The canonical Spec 172 runtime policy matrix is replayed at the pure
//! resolver/registry layer: unverified, verified-limited (Focusa and UIAI),
//! Focusa Operator, UIAI Operator, Bundle, refunded/revoked, offline,
//! corrupt, unknown family/product/type, future Navigator, dynamic tool,
//! node/seat, and resource cases all resolve deterministically and fail
//! closed. Decisions here are the single source of truth that the focusa-core
//! guard (`crates/focusa-core/tests/spec172_runtime_policy.rs`) and the API
//! route gate (`crates/focusa-api/src/middleware/spec172_runtime_policy.rs`)
//! must project identically.
//!
//! Exact verification: `cargo test --workspace spec172_runtime_policy`.
//!
//! No caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right is accepted anywhere in this module; no raw key,
//! token, email, customer row, or card data appears.

use std::path::PathBuf;

use focusa_license::{
    AccessPosture, BaseProductDecision, CapabilityFamily as Family, CompositeGrant,
    DecisionReason as Reason, DynamicOperationManifest, EntitlementPolicyPosture as Posture,
    LicenseTypeCode, LicenseTypeGrant, LicenseTypeVersion, ManifestTrustDecision,
    OperatorFamilyInheritanceDecision, OperatorSeats, PolicyEntitlementState as State,
    ProductCode, ResourceRight, SaleStatus, SharedNodeLimit, SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES, classify_operator_family_inheritance,
    is_focusa_verified_no_license_family_allowed, reduce_entitlement_state,
    resolve_base_focusa_product, verify_dynamic_operation_manifest, verify_generated_ui_action,
    CanonicalManifestFacts, FORBIDDEN_CLIENT_POLICY_FIELDS, REGISTERED_OPERATION_CLASSES,
    REGISTERED_PRODUCT_OWNERS, REGISTERED_SIDE_EFFECT_CLASSES,
};

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

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/spec152f-entitlement-policy-cases.v1.json")
}

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

fn resolve(state: State, family: Family) -> focusa_license::EntitlementStateDecision {
    let initiating = if family == Family::InternalMaintenance {
        Some(Posture::Deny)
    } else {
        None
    };
    reduce_entitlement_state(state, family, initiating)
}

// ── 1. Complete state × family matrix (golden replay + determinism) ────────

#[test]
fn spec172_runtime_policy_resolver_replays_complete_state_family_matrix() {
    let raw = std::fs::read_to_string(fixture_path()).expect("golden fixture must exist");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("golden fixture JSON");

    assert_eq!(fixture["schema"], "focusa.spec152f.entitlement_policy_cases.v1");
    assert_eq!(fixture["grid_case_count"], 63);
    assert_eq!(fixture["state_count"], 7);
    assert_eq!(fixture["family_count"], 9);

    let cases = fixture["grid_cases"].as_array().expect("grid_cases array");
    assert_eq!(cases.len(), 63, "exactly 7 states × 9 families");

    let mut seen = std::collections::BTreeSet::new();
    for case in cases {
        let state = parse_state(case["state"].as_str().expect("state label"));
        let family = parse_family(case["family"].as_str().expect("family label"));
        let expected = case["expected_decision"].as_str().expect("expected decision");
        assert!(
            seen.insert((state.label().to_string(), family.label().to_string())),
            "duplicate fixture pair: {state:?}/{family:?}"
        );

        let decision = resolve(state, family);
        let (posture, reason) = parse_decision(expected);
        assert_eq!(
            decision.posture(),
            posture.unwrap_or(Posture::Deny),
            "golden posture for {state:?}/{family:?} (expected {expected})"
        );
        assert_eq!(
            decision.reason(),
            reason,
            "golden reason for {state:?}/{family:?} (expected {expected})"
        );
    }
    assert_eq!(seen.len(), 63, "fixture must cover every state/family pair");

    // Deterministic: replaying the whole matrix twice yields identical results.
    for state in STATES {
        for family in FAMILIES {
            assert_eq!(
                resolve(state, family),
                resolve(state, family),
                "resolver must be deterministic: {state:?}/{family:?}"
            );
        }
    }

    // Spec 172 overlay: no Evaluation state exists anywhere in the grid; an
    // unknown state fails serde; verified_no_license is a first-class posture.
    assert!(serde_json::from_str::<State>("\"evaluation\"").is_err());
    assert!(serde_json::from_str::<State>("\"unknown\"").is_err());
    assert_eq!(
        serde_json::from_str::<State>("\"verified_no_license\"").expect("posture"),
        State::VerifiedNoLicense
    );
    assert_eq!(State::VerifiedNoLicense.label(), "verified_no_license");
}

// ── 2. Verified no-license limited mode (Focusa + UIAI) ───────────────────

#[test]
fn spec172_runtime_policy_verified_limited_allowlists_are_exact_and_closed() {
    // Focusa allowlist is exact.
    assert_eq!(
        SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
        [
            "manual_project",
            "manual_mission",
            "manual_focus_state",
            "manual_workpoint",
            "manual_trajectory",
            "manual_basic_evidence",
        ]
    );
    assert_eq!(
        SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
        ["automation", "team_remote", "release_proof", "premium_updates"]
    );
    // UIAI allowlist is exact.
    assert_eq!(
        SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
        [
            "public_search",
            "source_to_markdown",
            "public_page_read",
            "accessibility_snapshot",
            "screenshot",
            "basic_diagnostics",
        ]
    );
    assert_eq!(
        SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
        [
            "browser_action",
            "browser_persistence",
            "authenticated_private_targets",
            "unattended_browser_automation",
            "scheduled_batch_qa",
            "premium_hosted_resources",
        ]
    );

    // Every allowed Focusa family resolves (manual_project ≤ 1 mutable project);
    // every blocked family, unknown family, and unknown product is denied.
    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        let allowed = is_focusa_verified_no_license_family_allowed("focusa", family, 1);
        assert!(allowed, "allowlisted Focusa family must be allowed: {family}");
    }
    assert!(
        is_focusa_verified_no_license_family_allowed("focusa", "manual_project", 1)
            && !is_focusa_verified_no_license_family_allowed("focusa", "manual_project", 2),
        "manual_project is limited to one mutable project"
    );
    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(
            !is_focusa_verified_no_license_family_allowed("focusa", family, 1),
            "blocked Focusa family must be denied: {family}"
        );
    }
    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(
            is_focusa_verified_no_license_family_allowed("uiai_engine", family, 0),
            "allowlisted UIAI family must be allowed: {family}"
        );
    }
    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(
            !is_focusa_verified_no_license_family_allowed("uiai_engine", family, 0),
            "blocked UIAI family must be denied: {family}"
        );
    }
    assert!(
        !is_focusa_verified_no_license_family_allowed("focusa", "future_capability", 1),
        "unknown family must be denied"
    );
    assert!(
        !is_focusa_verified_no_license_family_allowed("uiai_engine", "manual_mission", 0),
        "cross-product family must be denied for UIAI"
    );
    assert!(
        !is_focusa_verified_no_license_family_allowed("future_product", "public_search", 0),
        "unknown product must be denied"
    );

    // The resolver's VerifiedNoLicense rows protect read/export/recovery while
    // value families deny.
    assert_eq!(
        resolve(State::VerifiedNoLicense, Family::ReadProjection).posture(),
        Posture::Read
    );
    assert_eq!(
        resolve(State::VerifiedNoLicense, Family::CustomerDataExport).posture(),
        Posture::Allow
    );
    assert_eq!(
        resolve(State::VerifiedNoLicense, Family::AccountRecovery).posture(),
        Posture::Allow
    );
    assert_eq!(
        resolve(State::VerifiedNoLicense, Family::BaseFocusa).reason(),
        Reason::AllowVerifiedLimited
    );
    for family in [Family::Automation, Family::TeamRemote, Family::ReleaseProof] {
        assert_eq!(
            resolve(State::VerifiedNoLicense, family).posture(),
            Posture::Deny,
            "blocked premium family must deny in verified limited mode: {family:?}"
        );
    }

    // Base product gate: verified no-license is Limited, never Entitled.
    assert_eq!(
        resolve_base_focusa_product("focusa", State::VerifiedNoLicense),
        BaseProductDecision::Limited
    );
    assert!(!resolve_base_focusa_product("focusa", State::VerifiedNoLicense).permits_base_mutations());
}

// ── 3. Operator License Types, Bundle union, future Navigator ──────────────

#[test]
fn spec172_runtime_policy_operator_types_and_bundle_are_exact_union() {
    let focusa = LicenseTypeGrant::focusa_operator_v1();
    let uiai = LicenseTypeGrant::uiai_operator_v1();
    assert!(focusa.validate().is_ok());
    assert!(uiai.validate().is_ok());

    assert_eq!(focusa.product, ProductCode::Focusa);
    assert_eq!(focusa.license_type, LicenseTypeCode::FocusaOperatorLifetimeV1);
    assert_eq!(focusa.version, LicenseTypeVersion::V1);
    assert_eq!(focusa.sale_status, SaleStatus::ApprovedNotYetEnabled);
    assert_eq!(focusa.operator_seats, OperatorSeats::One);
    assert_eq!(focusa.node_limit, SharedNodeLimit::OperatorSharedV1Three);
    assert_eq!(focusa.local_resource, ResourceRight::LocalIncluded);
    assert_eq!(focusa.hosted_resource, ResourceRight::HostedExcluded);

    assert_eq!(uiai.product, ProductCode::UiaiEngine);
    assert_eq!(uiai.license_type, LicenseTypeCode::UiaiOperatorLifetimeV1);

    // Bundle = exact union of the two underlying grants, no third catalog.
    let bundle =
        CompositeGrant::operator_bundle_v1([focusa.clone(), uiai.clone()]).expect("bundle union");
    assert_eq!(bundle.grants(), &[focusa, uiai]);
    assert!(
        CompositeGrant::operator_bundle_v1([
            LicenseTypeGrant::focusa_operator_v1(),
            LicenseTypeGrant::focusa_operator_v1()
        ])
        .is_err(),
        "a Focusa-only 'bundle' is malformed"
    );
    assert!(
        CompositeGrant::operator_bundle_v1([
            LicenseTypeGrant::focusa_operator_v1(),
            LicenseTypeGrant::uiai_operator_v1()
        ])
        .is_ok(),
        "exact two-grant union is canonical"
    );

    // Future Navigator is not a License Type today; Operator naming persists
    // and future types cannot mutate it.
    assert!(serde_json::from_str::<LicenseTypeCode>("\"focusa_navigator_lifetime_v1\"").is_err());
    assert!(serde_json::from_str::<LicenseTypeCode>("\"navigator\"").is_err());
    assert_eq!(
        LicenseTypeGrant::focusa_operator_v1(),
        LicenseTypeGrant::focusa_operator_v1(),
        "Operator grant is immutable across evaluations"
    );

    // Future products are excluded: only Focusa and UIAI Engine are registered.
    assert_eq!(
        serde_json::from_str::<ProductCode>("\"focusa\"").expect("focusa"),
        ProductCode::Focusa
    );
    assert_eq!(
        serde_json::from_str::<ProductCode>("\"uiai_engine\"").expect("uiai_engine"),
        ProductCode::UiaiEngine
    );
    assert!(serde_json::from_str::<ProductCode>("\"future_product\"").is_err());
}

// ── 4. Refunded/revoked, offline, corrupt, unknown — fail closed, data safe ─

#[test]
fn spec172_runtime_policy_refund_revoke_offline_corrupt_preserve_recovery() {
    // Refunded/revoked: paid families deny; recovery/export/read remain.
    for family in [
        Family::BaseFocusa,
        Family::Automation,
        Family::TeamRemote,
        Family::ReleaseProof,
        Family::PremiumUpdates,
    ] {
        assert_eq!(
            resolve(State::RefundedOrRevoked, family).posture(),
            Posture::Deny,
            "refunded/revoked must deny value family {family:?}"
        );
    }
    for family in [
        Family::AccountRecovery,
        Family::CustomerDataExport,
        Family::ReadProjection,
    ] {
        assert_ne!(
            resolve(State::RefundedOrRevoked, family).posture(),
            Posture::Deny,
            "recovery/export/read must survive refund/revoke for {family:?}"
        );
    }
    assert_eq!(
        resolve_base_focusa_product("focusa", State::RefundedOrRevoked),
        BaseProductDecision::Denied
    );

    // Expired: same fail-closed value posture with read/export/recovery intact.
    for family in [Family::BaseFocusa, Family::Automation, Family::ReleaseProof] {
        assert_eq!(
            resolve(State::Expired, family).posture(),
            Posture::Deny,
            "expired must deny value family {family:?}"
        );
    }
    assert_eq!(
        resolve(State::Expired, Family::ReadProjection).posture(),
        Posture::Read
    );

    // Missing/corrupt: read is local-only, value families deny, recovery stays.
    assert_eq!(
        resolve(State::MissingOrCorrupt, Family::ReadProjection).reason(),
        Reason::ReadLocalOnly
    );
    assert_eq!(
        resolve(State::MissingOrCorrupt, Family::BaseFocusa).posture(),
        Posture::Deny
    );
    assert_eq!(
        resolve(State::MissingOrCorrupt, Family::AccountRecovery).posture(),
        Posture::Allow
    );

    // Offline grace: base usable, premium requires a cached feature when safe.
    assert_eq!(
        resolve(State::OfflineGrace, Family::BaseFocusa).posture(),
        Posture::Base
    );
    assert_eq!(
        resolve(State::OfflineGrace, Family::PremiumUpdates).reason(),
        Reason::RequireCachedFeatureWhenSafe
    );
    assert_eq!(
        resolve(State::OfflineGrace, Family::AccountRecovery).reason(),
        Reason::AllowOfflineOnly
    );
    assert_eq!(
        resolve_base_focusa_product("focusa", State::OfflineGrace),
        BaseProductDecision::Entitled
    );
}

// ── 5. Unknown family / product / type and future Navigator ────────────────

#[test]
fn spec172_runtime_policy_unknown_and_future_fail_closed_in_inheritance() {
    // Unknown family in the verified-limited classifier denies.
    assert!(!is_focusa_verified_no_license_family_allowed("focusa", "unknown_family", 1));

    // Operator inheritance: a known family with all 8.2 conditions inherits.
    let inherited = classify_operator_family_inheritance(
        "focusa",
        "automation",
        true,
        true,
        true,
        true,
        false,
    );
    assert_eq!(inherited, OperatorFamilyInheritanceDecision::Inherit);
    assert!(inherited.is_inherited());

    // Materially new family: excluded pending explicit assignment.
    let excluded = classify_operator_family_inheritance(
        "focusa",
        "future_capability_family",
        true,
        false,
        true,
        true,
        false,
    );
    assert_eq!(excluded, OperatorFamilyInheritanceDecision::ExcludedPendingAssignment);
    assert!(!excluded.is_inherited());

    // Unknown product / future product / unknown owner / unknown side effect
    // all deny.
    assert_eq!(
        classify_operator_family_inheritance("focusa", "automation", false, true, true, true, false),
        OperatorFamilyInheritanceDecision::DeniedUnknownProduct
    );
    assert_eq!(
        classify_operator_family_inheritance("future_product", "automation", true, true, true, true, false),
        OperatorFamilyInheritanceDecision::DeniedFutureProduct
    );
    assert_eq!(
        classify_operator_family_inheritance("focusa", "automation", true, true, false, true, false),
        OperatorFamilyInheritanceDecision::DeniedUnknownOwner
    );
    assert_eq!(
        classify_operator_family_inheritance("focusa", "automation", true, true, true, false, false),
        OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect
    );
    // Materially new hosted cost denies even for a known family.
    assert_eq!(
        classify_operator_family_inheritance("focusa", "automation", true, true, true, true, true),
        OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost
    );
}

// ── 6. Dynamic tools and generated UI fail closed ──────────────────────────

#[test]
fn spec172_runtime_policy_dynamic_tool_and_generated_ui_fail_closed() {
    // Canonical registry constants are pinned.
    assert_eq!(REGISTERED_PRODUCT_OWNERS, ["focusa", "uiai_engine"]);
    assert_eq!(
        REGISTERED_OPERATION_CLASSES,
        ["read", "value_mutation", "recovery", "internal_maintenance"]
    );
    assert_eq!(
        REGISTERED_SIDE_EFFECT_CLASSES,
        ["none", "local", "remote", "external"]
    );
    assert!(!FORBIDDEN_CLIENT_POLICY_FIELDS.is_empty());

    // A signed manifest whose claims match the canonical registry is trusted.
    let facts = CanonicalManifestFacts {
        operation_registered: true,
        canonical_operation_class: Some("value_mutation".into()),
        canonical_capability_family: Some("automation".into()),
        canonical_side_effect_class: Some("local".into()),
        product_owner_registered: true,
        operation_class_registered: true,
        side_effect_class_registered: true,
        capability_family_registered: true,
    };
    let trusted = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "automation",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&trusted, &facts),
        ManifestTrustDecision::Trusted
    );

    // Every bypass vector quarantines: unsigned, unknown owner, unknown
    // mutation class, unknown side effect, unregistered family, self-labeled
    // recovery, client-selected policy.
    let unsigned = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "automation",
        "local",
    );
    assert_eq!(
        verify_dynamic_operation_manifest(&unsigned, &facts),
        ManifestTrustDecision::QuarantinedUnsigned
    );

    let unknown_owner = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "future_product",
        "value_mutation",
        "automation",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&unknown_owner, &facts),
        ManifestTrustDecision::QuarantinedUnknownOwner
    );

    let unknown_mutation = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "unattended_mutation",
        "automation",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&unknown_mutation, &facts),
        ManifestTrustDecision::QuarantinedUnknownMutation
    );

    let unknown_side_effect = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "automation",
        "external_metaclass",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&unknown_side_effect, &facts),
        ManifestTrustDecision::QuarantinedUnknownSideEffect
    );

    let unregistered_family = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "future_capability_family",
        "local",
    )
    .with_signature();
    let unregistered_facts = CanonicalManifestFacts {
        capability_family_registered: false,
        ..facts.clone()
    };
    assert_eq!(
        verify_dynamic_operation_manifest(&unregistered_family, &unregistered_facts),
        ManifestTrustDecision::QuarantinedUnregisteredFamily
    );

    // A manifest that claims a canonical family string that differs from the
    // canonical registry entry quarantines as client-selected policy (grant
    // expansion attempt).
    let mismatched_family = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "future_capability_family",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&mismatched_family, &facts),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );

    // A tool cannot self-label as recovery to bypass licensing.
    let self_labeled_recovery = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "recovery",
        "automation",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&self_labeled_recovery, &facts),
        ManifestTrustDecision::QuarantinedSelfLabeledRecovery
    );

    // Client-declared commercial policy fields quarantine.
    let client_policy = DynamicOperationManifest::new(
        "focusa.agent.silent_sessions.run",
        "focusa",
        "value_mutation",
        "automation",
        "local",
    )
    .with_signature()
    .with_declared_policy_fields(&["license_type", "price"]);
    assert_eq!(
        verify_dynamic_operation_manifest(&client_policy, &facts),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );

    // Generated UI: unsigned or unregistered actions fail closed; a signed
    // registered action is trusted and can only render canonical actions.
    assert_eq!(
        verify_generated_ui_action("focusa.workpoint.checkpoint", &["focusa.workpoint.checkpoint"], false),
        ManifestTrustDecision::QuarantinedUnsigned
    );
    assert_eq!(
        verify_generated_ui_action("focusa.workpoint.checkpoint", &["focusa.workpoint.checkpoint"], true),
        ManifestTrustDecision::Trusted
    );
    assert_eq!(
        verify_generated_ui_action("focusa.workpoint.checkpoint", &["focusa.mission.record"], true),
        ManifestTrustDecision::QuarantinedGeneratedUiGrantExpansion
    );
}

// ── 7. Node/seat/resource limits are authority-only ────────────────────────

#[test]
fn spec172_runtime_policy_node_seat_and_resource_limits_are_authority_only() {
    // One operator seat per License Type; up to three registered operator
    // nodes; CLI/TUI/Pi/menubar/Desktop/Cockpit on the same node do not
    // consume separate nodes; the Bundle shares the same three nodes.
    assert_eq!(serde_json::to_value(OperatorSeats::One).unwrap(), "one");
    assert_eq!(
        serde_json::to_value(SharedNodeLimit::OperatorSharedV1Three).unwrap(),
        "operator_shared_v1_three"
    );

    // Hosted resources are excluded for every Operator grant.
    let focusa = LicenseTypeGrant::focusa_operator_v1();
    let uiai = LicenseTypeGrant::uiai_operator_v1();
    assert_eq!(focusa.hosted_resource, ResourceRight::HostedExcluded);
    assert_eq!(uiai.hosted_resource, ResourceRight::HostedExcluded);

    // The commercial type system cannot express a widened grant: seats, node
    // limits, and hosted rights have exactly one canonical variant each, so a
    // caller cannot supply or select more than one seat, more than three
    // nodes, or any hosted resource right.
    assert!(serde_json::from_str::<OperatorSeats>("\"two\"").is_err());
    assert!(
        serde_json::from_str::<SharedNodeLimit>("\"operator_shared_v1_four\"").is_err()
    );
    assert!(
        serde_json::from_str::<ResourceRight>("\"hosted_included\"").is_err()
    );

    // The exact authority-owned constant is required: any structurally
    // different grant fails canonical validation (e.g., a Focusa code claimed
    // on the UIAI product).
    let other_owner = LicenseTypeGrant {
        product: ProductCode::UiaiEngine,
        ..focusa
    };
    assert!(other_owner.validate().is_err());

    // Access postures have no paid spelling for verified no-license.
    assert_eq!(
        serde_json::to_value(AccessPosture::VerifiedNoLicense).unwrap(),
        "verified_no_license"
    );
    assert!(serde_json::from_str::<AccessPosture>("\"evaluation\"").is_err());
    assert!(serde_json::from_str::<AccessPosture>("\"trial\"").is_err());

    // Operator v1 family set is frozen and exact.
    assert_eq!(
        SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
        [
            "manual_project",
            "manual_mission",
            "manual_focus_state",
            "manual_workpoint",
            "manual_trajectory",
            "manual_basic_evidence",
            "automation",
            "team_remote",
            "release_proof",
            "premium_updates",
        ]
    );
}
