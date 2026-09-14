use std::{collections::HashSet, fs, path::PathBuf};

use focusa_license::{
    OperatorFamilyInheritanceDecision, SPEC172_FOCUSA_OPERATOR_V1_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES, classify_operator_family_inheritance,
    is_focusa_verified_no_license_family_allowed,
};

fn allowed_focusa_families() -> &'static [&'static str] {
    &SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES
}

#[test]
fn spec172_family_inheritance_classifier_focusa_is_allowlist_driven_and_closed() {
    for family in allowed_focusa_families() {
        if *family == "manual_project" {
            assert!(is_focusa_verified_no_license_family_allowed(
                "focusa", family, 1
            ));
            assert!(!is_focusa_verified_no_license_family_allowed(
                "focusa", family, 2
            ));
        } else {
            assert!(is_focusa_verified_no_license_family_allowed(
                "focusa", family, 0
            ));
        }
    }

    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "focusa", family, 1
        ));
    }

    assert!(!is_focusa_verified_no_license_family_allowed(
        "focusa",
        "family_not_in_contract",
        1,
    ));
    assert!(!is_focusa_verified_no_license_family_allowed(
        "unknown",
        "manual_project",
        1
    ));
}

#[test]
fn spec172_family_inheritance_classifier_uiai_is_product_specific() {
    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }

    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }

    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }
}

#[test]
fn spec172_family_inheritance_registry_is_fail_closed_default() {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "../../docs/contracts/spec135/generated-contract-v1/operation-registry.json",
    ]
    .iter()
    .collect();
    let payload = fs::read_to_string(path).expect("operation registry file should exist");
    let registry: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");

    let operations = registry
        .get("operations")
        .and_then(serde_json::Value::as_array)
        .expect("operations list");
    assert_eq!(
        registry.get("operation_count"),
        Some(&serde_json::Value::from(157_u64))
    );

    let allowed: HashSet<&str> = allowed_focusa_families().iter().copied().collect();
    let mut covered = HashSet::new();

    for operation in operations {
        let operation_id = operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing>");
        let spec172_family = operation
            .get("spec172_family")
            .expect("spec172_family field must be present");

        if let Some(family) = spec172_family.as_str() {
            assert!(
                allowed.contains(family),
                "{operation_id} maps to non-allowlist family {family}"
            );
            covered.insert(family);
            continue;
        }

        assert!(
            spec172_family.is_null(),
            "{operation_id} has unknown spec172 family shape"
        );
    }

    for family in allowed {
        assert!(covered.contains(family), "no operation mapped to {family}");
    }
}

// ── Operator family inheritance classifier ──

#[test]
fn spec172_operator_family_inheritance_existing_family_inherits() {
    // All 10 Focusa Operator v1 families inherit when all five 8.2 conditions
    // are met.
    for family in SPEC172_FOCUSA_OPERATOR_V1_FAMILIES {
        let decision = classify_operator_family_inheritance(
            "focusa", family, true,  // known registered product
            true,  // known operator family
            true,  // known owner
            true,  // known side effect
            false, // no materially new hosted cost
        );
        assert_eq!(
            decision,
            OperatorFamilyInheritanceDecision::Inherit,
            "Operator family {family} must inherit when all 8.2 conditions are met"
        );
        assert!(decision.is_inherited());
        assert!(!decision.is_denied());
    }
}

#[test]
fn spec172_operator_family_inheritance_materially_new_family_excluded() {
    // A materially new family (not in Operator v1) is excluded pending
    // explicit assignment, even when all other conditions are met.
    let decision = classify_operator_family_inheritance(
        "focusa",
        "synthetic_new_customer_outcome",
        true,  // product is known
        false, // not a known operator family
        true,  // known owner
        true,  // known side effect
        false, // no hosted cost
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::ExcludedPendingAssignment,
        "materially new family must be excluded pending explicit assignment"
    );
    assert!(!decision.is_inherited());
    assert!(!decision.is_denied());
}

#[test]
fn spec172_operator_family_inheritance_unknown_product_denied() {
    // Unknown product denies all classification.
    let decision = classify_operator_family_inheritance(
        "unknown_product",
        "manual_workpoint",
        false, // unknown product
        true,  // known operator family
        true,  // known owner
        true,  // known side effect
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedUnknownProduct,
        "unknown product must be denied"
    );
    assert!(decision.is_denied());
}

#[test]
fn spec172_operator_family_inheritance_future_product_denied() {
    // A future (unregistered) product denied pending operator-approved
    // registration, even if the family name matches an existing family.
    let decision = classify_operator_family_inheritance(
        "synthetic_future_product",
        "manual_workpoint",
        true, // is registered as "known" but is not focusa/uiai
        true, // known operator family
        true, // known owner
        true, // known side effect
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedFutureProduct,
        "future product must be denied pending operator-approved registration"
    );
    assert!(decision.is_denied());
}

#[test]
fn spec172_operator_family_inheritance_unknown_owner_denied() {
    // Unknown owner denies all classification.
    let decision = classify_operator_family_inheritance(
        "focusa",
        "manual_workpoint",
        true,  // known product
        true,  // known operator family
        false, // unknown owner
        true,  // known side effect
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedUnknownOwner,
        "unknown owner must be denied"
    );
    assert!(decision.is_denied());
}

#[test]
fn spec172_operator_family_inheritance_unknown_side_effect_denied() {
    // Unknown side effect denies all classification.
    let decision = classify_operator_family_inheritance(
        "focusa",
        "manual_workpoint",
        true,  // known product
        true,  // known operator family
        true,  // known owner
        false, // unknown side effect
        false,
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect,
        "unknown side effect must be denied"
    );
    assert!(decision.is_denied());
}

#[test]
fn spec172_operator_family_inheritance_hosted_cost_denied() {
    // A materially new hosted cost is denied even when the family is known.
    let decision = classify_operator_family_inheritance(
        "focusa",
        "manual_workpoint",
        true, // known product
        true, // known operator family
        true, // known owner
        true, // known side effect
        true, // materially new hosted cost
    );
    assert_eq!(
        decision,
        OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost,
        "materially new hosted cost must be denied"
    );
    assert!(decision.is_denied());
}

#[allow(clippy::type_complexity)]
#[test]
fn spec172_operator_family_inheritance_negative_vectors() {
    // Batch negative vectors: every combination that should NOT inherit.
    let negative_vectors: &[(
        &str,
        &str,
        bool,
        bool,
        bool,
        bool,
        bool,
        OperatorFamilyInheritanceDecision,
    )] = &[
        // (product, family, is_known_product, is_known_operator_family, is_known_owner, is_known_side_effect, has_hosted_cost, expected)
        (
            "unknown",
            "manual_workpoint",
            false,
            true,
            true,
            true,
            false,
            OperatorFamilyInheritanceDecision::DeniedUnknownProduct,
        ),
        (
            "navigator",
            "manual_workpoint",
            true,
            true,
            true,
            true,
            false,
            OperatorFamilyInheritanceDecision::DeniedFutureProduct,
        ),
        (
            "focusa",
            "manual_workpoint",
            true,
            true,
            false,
            true,
            false,
            OperatorFamilyInheritanceDecision::DeniedUnknownOwner,
        ),
        (
            "focusa",
            "manual_workpoint",
            true,
            true,
            true,
            false,
            false,
            OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect,
        ),
        (
            "focusa",
            "next_gen_ai",
            true,
            false,
            true,
            true,
            false,
            OperatorFamilyInheritanceDecision::ExcludedPendingAssignment,
        ),
        (
            "focusa",
            "manual_workpoint",
            true,
            true,
            true,
            true,
            true,
            OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost,
        ),
        (
            "uiai_engine",
            "browser_action",
            true,
            false,
            true,
            true,
            false,
            OperatorFamilyInheritanceDecision::ExcludedPendingAssignment,
        ),
        (
            "",
            "manual_workpoint",
            true,
            true,
            true,
            true,
            false,
            OperatorFamilyInheritanceDecision::DeniedFutureProduct,
        ),
    ];

    for (
        product,
        family,
        known_product,
        known_family,
        known_owner,
        known_side_effect,
        hosted,
        expected,
    ) in negative_vectors
    {
        let decision = classify_operator_family_inheritance(
            product,
            family,
            *known_product,
            *known_family,
            *known_owner,
            *known_side_effect,
            *hosted,
        );
        assert_eq!(
            decision, *expected,
            "negative vector ({product}, {family}) expected {expected:?}, got {decision:?}"
        );
    }
}

#[test]
fn spec172_operator_family_inheritance_label_roundtrip() {
    // Every decision variant has a stable label.
    let variants = &[
        OperatorFamilyInheritanceDecision::Inherit,
        OperatorFamilyInheritanceDecision::ExcludedPendingAssignment,
        OperatorFamilyInheritanceDecision::DeniedUnknownProduct,
        OperatorFamilyInheritanceDecision::DeniedUnknownOwner,
        OperatorFamilyInheritanceDecision::DeniedUnknownSideEffect,
        OperatorFamilyInheritanceDecision::DeniedFutureProduct,
        OperatorFamilyInheritanceDecision::DeniedMateriallyNewHostedCost,
    ];
    let mut seen = HashSet::new();
    for v in variants {
        let label = v.label();
        assert!(!label.is_empty(), "label must not be empty for {v:?}");
        assert!(seen.insert(label), "duplicate label {label}");
    }
}
