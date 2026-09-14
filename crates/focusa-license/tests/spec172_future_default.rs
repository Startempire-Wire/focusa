//! Spec 172 §4.3/§8/§9/§10/§12/§15 — future Navigator, materially new
//! families, future products, and hosted resources default to exclusion
//! (atom focusa-vbcqu.20.15.36, 172.05.05).
//!
//! Synthetic future-evolution fixtures: an attempt to add `Navigator`, a
//! materially new capability family, an unregistered future product, and a
//! hosted metered resource are each evaluated against the existing Operator /
//! Bundle / verified-no-license surfaces. The proof is fail-closed:
//!
//! - `Navigator` is not a License Type today; the attempt cannot mutate the
//!   frozen Operator grant, the exact two-grant Bundle union, or the
//!   verified-no-license allowlist, and a lifetime entitlement record refuses
//!   the Navigator type code.
//! - A materially new family is excluded pending explicit versioned
//!   assignment; it never enters the verified-no-license allowlist and denies
//!   in every policy state.
//! - A future product and a hosted metered resource are denied by the
//!   Operator family classifier (Section 8.3) and never enter the Bundle.
//! - A safe same-family implementation (same registered product, same
//!   customer-understandable outcome, fitting profile, no separate product,
//!   no materially new hosted cost) inherits the family without per-tool
//!   pricing (Section 8.2).
//! - Synthetic dynamic-operation manifests for all four future attempts
//!   quarantine at runtime intake (Section 12) with the stable
//!   `ENTITLEMENT_POLICY_UNKNOWN` error.
//!
//! No caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right is accepted anywhere; no raw key, token, email,
//! customer row, or card data appears.

use focusa_license::{
    CanonicalManifestFacts, CapabilityFamily as Family, CompositeGrant, DecisionReason as Reason,
    DynamicOperationManifest, ENTITLEMENT_POLICY_UNKNOWN, EntitlementPolicyPosture as Posture,
    EntitlementStateDecision, LicenseTypeCode, LicenseTypeGrant, ManifestQuarantineLedger,
    ManifestTrustDecision, OperatorFamilyInheritanceDecision, PolicyEntitlementState as State,
    ResourceRight, SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES, classify_operator_family_inheritance,
    is_focusa_verified_no_license_family_allowed, lifetime_entitlement::LifetimeEntitlement,
    reduce_entitlement_state, verify_dynamic_operation_manifest,
};

// ── Synthetic future-evolution fixtures (public non-production values) ─────
const SYNTHETIC_NAVIGATOR_TYPE: &str = "focusa_navigator_lifetime_v1";
const SYNTHETIC_NEW_FAMILY: &str = "synthetic_future_capability";
const SYNTHETIC_FUTURE_PRODUCT: &str = "synthetic_future_product";
const SYNTHETIC_HOSTED_FAMILY: &str = "hosted_metered_compute";

fn decision(state: State, family: Family, initiating: Option<Posture>) -> EntitlementStateDecision {
    reduce_entitlement_state(state, family, initiating)
}

#[test]
fn spec172_future_default_navigator_is_separate_and_never_mutates_operator() {
    // Navigator is not a License Type today: unknown type codes fail serde.
    assert!(
        serde_json::from_str::<LicenseTypeCode>(&format!("\"{SYNTHETIC_NAVIGATOR_TYPE}\""))
            .is_err()
    );
    assert!(serde_json::from_str::<LicenseTypeCode>("\"navigator\"").is_err());

    // The frozen Operator grant is byte-identical after the Navigator
    // attempt; nothing about Operator can be mutated by a future type.
    let operator = LicenseTypeGrant::focusa_operator_v1();
    assert_eq!(operator.validate(), Ok(()));
    assert_eq!(
        operator.license_type,
        LicenseTypeCode::FocusaOperatorLifetimeV1
    );
    assert_eq!(operator.hosted_resource, ResourceRight::HostedExcluded);

    // The Bundle union stays the exact two-grant composition; a Navigator
    // grant can never enter it.
    let uiai = LicenseTypeGrant::uiai_operator_v1();
    let bundle = CompositeGrant::operator_bundle_v1([operator, uiai]).expect("canonical Bundle");
    assert_eq!(bundle.grants(), &[operator, uiai]);
    // A caller-constructed "Navigator-like" grant differs from the canonical
    // Operator record and fails validation — there is no grant spell for a
    // future type, and it cannot be swapped into the Bundle.
    let mut forged = LicenseTypeGrant::focusa_operator_v1();
    forged.hosted_resource = ResourceRight::LocalIncluded;
    assert_eq!(
        forged.validate(),
        Err(focusa_license::EntitlementPolicyTypeError::InvalidLicenseTypeGrant)
    );
    assert_eq!(
        CompositeGrant::operator_bundle_v1([forged, uiai]),
        Err(focusa_license::EntitlementPolicyTypeError::MalformedBundleUnion)
    );

    // The lifetime entitlement registry refuses the Navigator type code for
    // the focusa product (separate stable code required, Section 10.2).
    let now = chrono::Utc::now();
    assert_eq!(
        LifetimeEntitlement::new(
            "focusa",
            SYNTHETIC_NAVIGATOR_TYPE,
            1,
            "697.00",
            "sha256:operator-v1",
            3,
            1,
            now,
        ),
        Err(
            focusa_license::lifetime_entitlement::LifetimeCredentialError::InvalidLicenseType(
                SYNTHETIC_NAVIGATOR_TYPE.to_string()
            )
        )
    );

    // Verified-no-license allowlist is untouched by the Navigator attempt.
    assert!(is_focusa_verified_no_license_family_allowed(
        "focusa",
        "manual_workpoint",
        1,
    ));
}

#[test]
fn spec172_future_default_new_family_excluded_pending_versioned_assignment() {
    // Materially new family: excluded pending explicit versioned assignment.
    let decision = classify_operator_family_inheritance(
        "focusa",
        SYNTHETIC_NEW_FAMILY,
        true,  // focusa is a registered product
        false, // the family is NOT in the Operator v1 family set
        true,
        true,
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::ExcludedPendingAssignment
    );
    assert!(!decision.is_inherited());

    // The new family never enters the verified-no-license allowlist and
    // denies even with an active-project claim.
    assert!(!is_focusa_verified_no_license_family_allowed(
        "focusa",
        SYNTHETIC_NEW_FAMILY,
        1,
    ));
    // Same for the UIAI product boundary.
    assert!(!is_focusa_verified_no_license_family_allowed(
        "uiai_engine",
        SYNTHETIC_NEW_FAMILY,
        0,
    ));

    // A synthetic new family cannot be expressed in the typed capability
    // family set at all (unknown values fail serde), and every existing
    // blocked family denies in verified_no_license posture.
    assert!(serde_json::from_str::<Family>(&format!("\"{SYNTHETIC_NEW_FAMILY}\"")).is_err());
    for blocked in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "focusa", blocked, 1
        ));
    }
    for allowed in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(is_focusa_verified_no_license_family_allowed(
            "focusa", allowed, 1
        ));
    }
}

#[test]
fn spec172_future_default_future_product_and_hosted_resource_denied() {
    // Future (unregistered) product: denied pending operator-approved
    // registration (Section 15); namespace/marketing resemblance grants
    // nothing. The product is recognized as registered-pending; its owner
    // string is not a current product, so the attempt is denied as a future
    // product (not silently folded into Operator).
    let decision = classify_operator_family_inheritance(
        SYNTHETIC_FUTURE_PRODUCT,
        "synthetic_family",
        true, // recognized registered-pending product
        false,
        true,
        true,
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedFutureProduct
    );
    assert!(decision.is_denied());
    assert_eq!(
        classify_operator_family_inheritance(
            SYNTHETIC_FUTURE_PRODUCT,
            "manual_workpoint",
            true,
            true,
            true,
            true,
            false,
        ),
        OperatorFamilyInheritanceDecision::DeniedFutureProduct
    );
    // A completely unknown product denies even harder (fail closed).
    assert_eq!(
        classify_operator_family_inheritance(
            "unknown_product",
            "synthetic_family",
            false,
            false,
            true,
            true,
            false,
        ),
        OperatorFamilyInheritanceDecision::DeniedUnknownProduct
    );

    // Hosted metered resource: excluded unless explicitly listed (Sections
    // 7.2/8.3); a known family with a materially new hosted cost is denied,
    // and the Operator grant's hosted_resource right stays excluded.
    let hosted = classify_operator_family_inheritance(
        "focusa",
        "automation",
        true,
        true,
        true,
        true,
        true, // materially new hosted cost
    );
    assert_eq!(
        hosted,
        OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost
    );
    assert!(hosted.is_denied());
    let operator = LicenseTypeGrant::focusa_operator_v1();
    assert_eq!(operator.hosted_resource, ResourceRight::HostedExcluded);

    // A hosted-metrics family is not a registered family in any posture, and
    // it never reaches the verified-no-license allowlist.
    assert!(!is_focusa_verified_no_license_family_allowed(
        "focusa",
        SYNTHETIC_HOSTED_FAMILY,
        1,
    ));
    assert!(serde_json::from_str::<Family>(&format!("\"{SYNTHETIC_HOSTED_FAMILY}\"")).is_err());

    // Existing lifetime rights remain stable: the frozen Operator and Bundle
    // grants still validate, and the Bundle remains the exact two-grant union
    // (no future product or hosted right entered it).
    let bundle = CompositeGrant::operator_bundle_v1([
        LicenseTypeGrant::focusa_operator_v1(),
        LicenseTypeGrant::uiai_operator_v1(),
    ])
    .expect("canonical Bundle");
    for grant in bundle.grants() {
        assert_eq!(grant.validate(), Ok(()));
        assert_eq!(grant.hosted_resource, ResourceRight::HostedExcluded);
    }
}

#[test]
fn spec172_future_default_safe_same_family_implementation_inherits() {
    // Section 8.2: every canonical Operator v1 family inherits a new
    // implementation when all five conditions hold — no per-tool pricing.
    for family in SPEC172_FOCUSA_OPERATOR_V1_FAMILIES {
        let decision = classify_operator_family_inheritance(
            "focusa", family, true,  // same registered product
            true,  // family is included in Operator v1
            true,  // known owner
            true,  // known side-effect class
            false, // no materially new hosted cost
        );
        assert_eq!(
            decision,
            OperatorFamilyInheritanceDecision::Inherit,
            "family {family} must inherit"
        );
        assert!(decision.is_inherited());
    }

    // Missing any one of the five conditions fails closed for that family:
    // the family is no longer known, or the product is unknown.
    assert_eq!(
        classify_operator_family_inheritance(
            "focusa",
            "manual_workpoint",
            true,
            true,
            false,
            true,
            false
        ),
        OperatorFamilyInheritanceDecision::DeniedUnknownOwner
    );
    assert_eq!(
        classify_operator_family_inheritance(
            "focusa",
            "manual_workpoint",
            true,
            true,
            true,
            false,
            false
        ),
        OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect
    );
    assert_eq!(
        classify_operator_family_inheritance(
            "focusa",
            "manual_workpoint",
            true,
            true,
            true,
            true,
            true
        ),
        OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost
    );

    // Internal maintenance inherits the initiating operation's posture
    // (Section 12: a maintenance sub-operation never broadens the policy);
    // with no initiating policy it denies instead of inventing one.
    let inherited = decision(
        State::ActivePaid,
        Family::InternalMaintenance,
        Some(Posture::Allow),
    );
    assert_eq!(inherited.posture(), Posture::Allow);
    assert_eq!(inherited.reason(), Reason::Inherit);
    let orphaned = decision(State::ActivePaid, Family::InternalMaintenance, None);
    assert_eq!(orphaned.posture(), Posture::Deny);
    assert_eq!(orphaned.reason(), Reason::MissingInitiatingPolicy);
}

#[test]
fn spec172_future_default_dynamic_operation_fixtures_fail_closed() {
    // Synthetic dynamic-operation manifests for the four future attempts all
    // quarantine at runtime intake (Section 12) with the stable error code,
    // and the safe same-family implementation trusts without widening.
    let mut ledger = ManifestQuarantineLedger::default();

    // 1. Future Navigator tool declares a client-selected License Type.
    let navigator_tool = DynamicOperationManifest::new(
        "focusa.navigator.synthetic_tool",
        "focusa",
        "value_mutation",
        "manual_workpoint",
        "local",
    )
    .with_signature()
    .with_declared_policy_fields(&["license_type"]);
    let facts = CanonicalManifestFacts {
        operation_registered: true,
        canonical_operation_class: Some("value_mutation".into()),
        canonical_capability_family: Some("manual_workpoint".into()),
        canonical_side_effect_class: Some("local".into()),
        product_owner_registered: true,
        operation_class_registered: true,
        side_effect_class_registered: true,
        capability_family_registered: true,
    };
    assert_eq!(
        verify_dynamic_operation_manifest(&navigator_tool, &facts),
        ManifestTrustDecision::QuarantinedClientSelectedPolicy
    );

    // 2. Materially new family is not a registered family.
    let new_family_tool = DynamicOperationManifest::new(
        "focusa.synthetic_new_family.tool",
        "focusa",
        "value_mutation",
        SYNTHETIC_NEW_FAMILY,
        "local",
    )
    .with_signature();
    let mut unregistered_family = facts.clone();
    unregistered_family.capability_family_registered = false;
    unregistered_family.canonical_capability_family = None;
    assert_eq!(
        verify_dynamic_operation_manifest(&new_family_tool, &unregistered_family),
        ManifestTrustDecision::QuarantinedUnregisteredFamily
    );

    // 3. Future product is not a registered product owner.
    let future_product_tool = DynamicOperationManifest::new(
        "synthetic_future_product.tool",
        SYNTHETIC_FUTURE_PRODUCT,
        "value_mutation",
        "synthetic_family",
        "remote",
    )
    .with_signature();
    let mut unknown_owner = facts.clone();
    unknown_owner.product_owner_registered = false;
    assert_eq!(
        verify_dynamic_operation_manifest(&future_product_tool, &unknown_owner),
        ManifestTrustDecision::QuarantinedUnknownOwner
    );

    // 4. Hosted metered resource declares a commercial right.
    let hosted_tool = DynamicOperationManifest::new(
        "focusa.hosted_metered.tool",
        "focusa",
        "value_mutation",
        SYNTHETIC_HOSTED_FAMILY,
        "external",
    )
    .with_signature()
    .with_declared_policy_fields(&["commercial_right"]);
    let mut hosted_facts = facts.clone();
    hosted_facts.capability_family_registered = false;
    hosted_facts.canonical_capability_family = None;
    assert_eq!(
        verify_dynamic_operation_manifest(&hosted_tool, &hosted_facts),
        ManifestTrustDecision::QuarantinedUnregisteredFamily
    );

    // 5. Safe same-family implementation trusts: canonical registered
    // operation, signed manifest, exact match — inherited policy, no
    // per-tool pricing.
    let safe_tool = DynamicOperationManifest::new(
        "focusa.manual_workpoint.improved_implementation",
        "focusa",
        "value_mutation",
        "manual_workpoint",
        "local",
    )
    .with_signature();
    assert_eq!(
        verify_dynamic_operation_manifest(&safe_tool, &facts),
        ManifestTrustDecision::Trusted
    );

    // The quarantine ledger blocks all four future attempts and never the
    // safe implementation; every record carries the stable error code.
    let mut quarantined = 0;
    for (tool, expected) in [
        (
            &navigator_tool,
            ManifestTrustDecision::QuarantinedClientSelectedPolicy,
        ),
        (
            &new_family_tool,
            ManifestTrustDecision::QuarantinedUnregisteredFamily,
        ),
        (
            &future_product_tool,
            ManifestTrustDecision::QuarantinedUnknownOwner,
        ),
        (
            &hosted_tool,
            ManifestTrustDecision::QuarantinedUnregisteredFamily,
        ),
    ] {
        let decision = verify_dynamic_operation_manifest(tool, &facts_for(tool));
        assert_eq!(
            decision, expected,
            "unexpected verdict for {}",
            tool.operation_id
        );
        assert!(decision.is_quarantined());
        assert_eq!(decision.stable_error(), ENTITLEMENT_POLICY_UNKNOWN);
        ledger.quarantine(&tool.operation_id, decision.label());
        quarantined += 1;
    }
    assert_eq!(quarantined, 4);
    assert_eq!(ledger.len(), 4);
    assert!(ledger.is_quarantined("focusa.navigator.synthetic_tool"));
    assert!(ledger.is_quarantined("focusa.synthetic_new_family.tool"));
    assert!(ledger.is_quarantined("synthetic_future_product.tool"));
    assert!(ledger.is_quarantined("focusa.hosted_metered.tool"));
    assert!(!ledger.is_quarantined("focusa.manual_workpoint.improved_implementation"));
    for record in ledger.records() {
        assert_eq!(record.stable_error, ENTITLEMENT_POLICY_UNKNOWN);
    }
}

/// Recompute the canonical facts for each quarantined future attempt so the
/// ledger replay uses the same fail-closed gates as the direct assertions.
fn facts_for(tool: &DynamicOperationManifest) -> CanonicalManifestFacts {
    match tool.operation_id.as_str() {
        "focusa.navigator.synthetic_tool" => CanonicalManifestFacts {
            operation_registered: true,
            canonical_operation_class: Some("value_mutation".into()),
            canonical_capability_family: Some("manual_workpoint".into()),
            canonical_side_effect_class: Some("local".into()),
            product_owner_registered: true,
            operation_class_registered: true,
            side_effect_class_registered: true,
            capability_family_registered: true,
        },
        "focusa.synthetic_new_family.tool" => CanonicalManifestFacts {
            operation_registered: true,
            canonical_operation_class: Some("value_mutation".into()),
            canonical_capability_family: None,
            canonical_side_effect_class: Some("local".into()),
            product_owner_registered: true,
            operation_class_registered: true,
            side_effect_class_registered: true,
            capability_family_registered: false,
        },
        "synthetic_future_product.tool" => CanonicalManifestFacts {
            operation_registered: true,
            canonical_operation_class: Some("value_mutation".into()),
            canonical_capability_family: Some("synthetic_family".into()),
            canonical_side_effect_class: Some("remote".into()),
            product_owner_registered: false,
            operation_class_registered: true,
            side_effect_class_registered: true,
            capability_family_registered: true,
        },
        _ => CanonicalManifestFacts {
            operation_registered: true,
            canonical_operation_class: Some("value_mutation".into()),
            canonical_capability_family: None,
            canonical_side_effect_class: Some("external".into()),
            product_owner_registered: true,
            operation_class_registered: true,
            side_effect_class_registered: true,
            capability_family_registered: false,
        },
    }
}
